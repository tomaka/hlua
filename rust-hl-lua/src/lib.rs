#![feature(unsafe_destructor)]

extern crate "lua52-sys" as ffi;
extern crate libc;

use std::ffi::{CStr, CString};
use std::io::Read;
use std::io::Error as IoError;
use std::borrow::Borrow;
use std::marker::PhantomData;

pub use functions_read::LuaFunction;
pub use lua_tables::LuaTable;

pub mod any;
pub mod functions_read;
pub mod lua_tables;
//pub mod userdata;

mod functions_write;
mod rust_tables;
mod values;
mod tuples;


/// Main object of the library.
///
/// The lifetime parameter corresponds to the lifetime of the content of the Lua context.
pub struct Lua<'lua> {
    lua: LuaContext,
    must_be_closed: bool,
    marker: PhantomData<&'lua ()>,
}

///
pub struct PushGuard<L> where L: AsMutLua {
    lua: L,
    size: i32,
}

/// Trait for objects that have access to a Lua context.
pub unsafe trait AsLua {
    fn as_lua(&self) -> LuaContext;
}

/// Trait for objects that have access to a Lua context.
pub unsafe trait AsMutLua: AsLua {
    fn as_mut_lua(&mut self) -> LuaContext;
}

/// Represents a raw Lua context.
#[derive(Copy, Clone)]
#[allow(raw_pointer_derive)]
pub struct LuaContext(*mut ffi::lua_State);
unsafe impl Send for LuaContext {}

unsafe impl<'a, 'lua> AsLua for &'a Lua<'lua> {
    fn as_lua(&self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'a, 'lua> AsLua for &'a mut Lua<'lua> {
    fn as_lua(&self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'a, 'lua> AsMutLua for &'a mut Lua<'lua> {
    fn as_mut_lua(&mut self) -> LuaContext {
        self.lua
    }
}

unsafe impl<L> AsLua for PushGuard<L> where L: AsMutLua {
    fn as_lua(&self) -> LuaContext {
        self.lua.as_lua()
    }
}

unsafe impl<L> AsMutLua for PushGuard<L> where L: AsMutLua {
    fn as_mut_lua(&mut self) -> LuaContext {
        self.lua.as_mut_lua()
    }
}

unsafe impl<'a, L> AsLua for &'a L where L: AsLua {
    fn as_lua(&self) -> LuaContext {
        (**self).as_lua()
    }
}

unsafe impl<'a, L> AsLua for &'a mut L where L: AsLua {
    fn as_lua(&self) -> LuaContext {
        (**self).as_lua()
    }
}

unsafe impl<'a, L> AsMutLua for &'a mut L where L: AsMutLua {
    fn as_mut_lua(&mut self) -> LuaContext {
        (**self).as_mut_lua()
    }
}

/// Should be implemented by whatever type is pushable on the Lua stack.
pub trait Push<L> where L: AsMutLua {
    /// Pushes the value on the top of the stack.
    /// Must return the number of elements pushed.
    ///
    /// You can implement this for any type you want by redirecting to call to
    /// another implementation (for example `5.push_to_lua`) or by calling `userdata::push_userdata`
    fn push_to_lua(self, lua: L) -> PushGuard<L>;
}

/// Reads the data from Lua.
pub trait LuaRead<L>: Sized where L: AsLua {
    /// Reads the data from Lua.
    fn lua_read(lua: L) -> Option<Self> {
        LuaRead::lua_read_at_position(lua, -1)
    }

    /// Reads the data from Lua.
    fn lua_read_at_position(lua: L, index: i32) -> Option<Self>;
}

/// Error that can happen when executing Lua code.
#[derive(Debug)]
pub enum LuaError {
    /// There was a syntax error when parsing the Lua code.
    SyntaxError(String),

    /// There was an error during execution of the Lua code
    /// (for example not enough parameters for a function call).
    ExecutionError(String),

    /// There was an IoError while reading the source code to execute.
    ReadError(IoError),

    /// The call to `execute` has requested the wrong type of data.
    WrongType,
}

impl<'lua> Lua<'lua> {
    /// Builds a new Lua context.
    ///
    /// # Panic
    ///
    /// The function panics if the underlying call to `lua_newstate` fails
    /// (which indicates lack of memory).
    pub fn new() -> Lua<'lua> {
        let lua = unsafe { ffi::lua_newstate(alloc, std::ptr::null_mut()) };
        if lua.is_null() {
            panic!("lua_newstate failed");
        }

        // this alloc function is required to create a lua state.
        extern "C" fn alloc(_ud: *mut libc::c_void, ptr: *mut libc::c_void, _osize: libc::size_t,
                            nsize: libc::size_t) -> *mut libc::c_void
        {
            unsafe {
                if nsize == 0 {
                    libc::free(ptr as *mut libc::c_void);
                    std::ptr::null_mut()
                } else {
                    libc::realloc(ptr, nsize)
                }
            }
        }

        // called whenever lua encounters an unexpected error.
        extern "C" fn panic(lua: *mut ffi::lua_State) -> libc::c_int {
            let err = unsafe { ffi::lua_tostring(lua, -1) };
            let err = unsafe { CStr::from_ptr(err) };
            let err = String::from_utf8(err.to_bytes().to_vec()).unwrap();
            panic!("PANIC: unprotected error in call to Lua API ({})\n", err);
        }

        unsafe { ffi::lua_atpanic(lua, panic) };

        Lua {
            lua: LuaContext(lua),
            must_be_closed: true,
            marker: PhantomData,
        }
    }

    /// Takes an existing lua_State and build a Lua object from it.
    ///
    /// # Arguments
    ///  * close_at_the_end: if true, lua_close will be called on the lua_State on the destructor
    pub unsafe fn from_existing_state<T>(lua: *mut T, close_at_the_end: bool) -> Lua<'lua> {
        Lua {
            lua: std::mem::transmute(lua),
            must_be_closed: close_at_the_end,
            marker: PhantomData,
        }
    }

    /// Opens all standard Lua libraries.
    /// This is done by calling `luaL_openlibs`.
    pub fn openlibs(&mut self) {
        unsafe { ffi::luaL_openlibs(self.lua.0) }
    }

    /// Executes some Lua code on the context.
    pub fn execute<'a, T>(&'a mut self, code: &str) -> Result<T, LuaError> where T: for<'g> LuaRead<&'g mut PushGuard<&'a mut Lua<'lua>>> {
        let mut f = try!(functions_read::LuaFunction::load(self, code));
        f.call()
    }

    /// Executes some Lua code on the context.
    pub fn execute_from_reader<'a, T, R: Read + 'static>(&'a mut self, code: R) -> Result<T, LuaError> where T: for<'g> LuaRead<&'g mut PushGuard<&'a mut Lua<'lua>>> {
        let mut f = try!(functions_read::LuaFunction::load_from_reader(self, code));
        f.call()
    }

    /// Reads the value of a global variable by copying it.
    pub fn get<'l, I, V>(&'l mut self, index: I) -> Option<V>
                         where I: Borrow<str>, V: LuaRead<&'l mut Lua<'lua>>
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe { ffi::lua_getglobal(self.lua.0, index.as_ptr()); }
        LuaRead::lua_read(self)
    }

    /// Modifies the value of a global variable.
    pub fn set<'a, I, V>(&'a mut self, index: I, value: V)
                         where I: Borrow<str>, V: Push<&'a mut Lua<'lua>>
    {
        let index = CString::new(index.borrow()).unwrap();
        let guard = value.push_to_lua(self);
        unsafe { ffi::lua_setglobal(self.lua.0, index.as_ptr()); }
        unsafe { std::mem::forget(guard) }
    }
}

#[unsafe_destructor]
impl<'lua> Drop for Lua<'lua> {
    fn drop(&mut self) {
        if self.must_be_closed {
            unsafe { ffi::lua_close(self.lua.0) }
        }
    }
}

#[unsafe_destructor]
impl<L> Drop for PushGuard<L> where L: AsMutLua {
    fn drop(&mut self) {
        unsafe { ffi::lua_pop(self.lua.as_mut_lua().0, self.size); }
    }
}
