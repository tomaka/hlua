extern crate lua52_sys as ffi;
extern crate libc;
#[macro_use] extern crate quick_error;

use std::ffi::{CStr, CString};
use std::io::Read;
use std::io::Error as IoError;
use std::borrow::Borrow;
use std::marker::PhantomData;

pub use functions_read::LuaFunction;
pub use functions_write::{Function, InsideCallback};
pub use functions_write::{function0, function1, function2, function3, function4, function5};
pub use functions_write::{function6, function7, function8, function9, function10};
pub use lua_tables::LuaTable;
pub mod any;
pub mod functions_read;
pub mod lua_tables;
pub mod userdata;

mod functions_write;
mod macros;
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

/// RAII guard for a value pushed on the stack.
pub struct PushGuard<L> where L: AsMutLua {
    lua: L,
    size: i32,
}

impl<L> PushGuard<L> where L: AsMutLua {
    /// Prevents the value from being poped when the `PushGuard` is destroyed, and returns the
    /// number of elements on the stack.
    fn forget(mut self) -> i32 {
        let size = self.size;
        self.size = 0;
        size
    }

    /// Destroys the guard, popping the value. Returns the inner part,
    /// which returns access when using by-value capture.
    pub fn into_inner(mut self) -> L {
        use std::{mem, ptr};

        let mut res;
        unsafe {
            res = mem::uninitialized();
            ptr::copy_nonoverlapping(&self.lua, &mut res, 1);
            if self.size != 0 {
                ffi::lua_pop(self.lua.as_mut_lua().0, self.size);
            }
        };
        mem::forget(self);

        res
    }
}

/// Trait for objects that have access to a Lua context. When using a context returned by a
/// `AsLua`, you are not allowed to modify the stack.
pub unsafe trait AsLua {
    fn as_lua(&self) -> LuaContext;
}

/// Trait for objects that have access to a Lua context. You are allowed to modify the stack, but
/// it must be in the same state as it was when you started.
pub unsafe trait AsMutLua: AsLua {
    fn as_mut_lua(&mut self) -> LuaContext;
}

/// Opaque type that contains the raw Lua context.
#[derive(Copy, Clone)]
#[allow(raw_pointer_derive)]
pub struct LuaContext(*mut ffi::lua_State);
unsafe impl Send for LuaContext {}

unsafe impl<'a, 'lua> AsLua for Lua<'lua> {
    fn as_lua(&self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'lua> AsMutLua for Lua<'lua> {
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

/// Types that can be given to a Lua context, for example with `lua.set()` or as a return value
/// of a function.
pub trait Push<L> where L: AsMutLua {
    /// Pushes the value on the top of the stack.
    ///
    /// Must return a guard representing the elements that have been pushed.
    ///
    /// You can implement this for any type you want by redirecting to call to
    /// another implementation (for example `5.push_to_lua`) or by calling
    /// `userdata::push_userdata`.
    fn push_to_lua(self, lua: L) -> PushGuard<L>;
}

/// Types that can be obtained from a Lua context.
///
/// Most types that implement `Push` also implement `LuaRead`, but this is not always the case
/// (for example `&'static str` implements `Push` but not `LuaRead`).
pub trait LuaRead<L>: Sized where L: AsLua {
    /// Reads the data from Lua.
    fn lua_read(lua: L) -> Result<Self, L> {
        LuaRead::lua_read_at_position(lua, -1)
    }

    /// Reads the data from Lua at a given position.
    fn lua_read_at_position(lua: L, index: i32) -> Result<Self, L>;
}

quick_error! {
    /// Error that can happen when executing Lua code.
    #[derive(Debug)]
    pub enum LuaError {
        /// There was a syntax error when parsing the Lua code.
        SyntaxError(err: String) {
            display("Syntax error: {}", err)
            description(&err[..])
        }

        /// There was an error during execution of the Lua code
        /// (for example not enough parameters for a function call).
        ExecutionError(err: String) {
            display("Execution error: {}", err)
            description(&err[..])
        }

        /// There was an IoError while reading the source code to execute.
        ReadError(err: IoError) {
            from()
            cause(err)
            display("I/O error: {}", err)
            description("I/O error")
        }

        /// The call to `execute` has requested the wrong type of data.
        WrongType {
            description("The call to `execute` has requested \
                         the wrong type of data")
        }
    }
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

    /// Takes an existing `lua_State` and build a Lua object from it.
    ///
    /// # Arguments
    ///
    ///  * `close_at_the_end`: if true, lua_close will be called on the lua_State on the destructor
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
    pub fn execute<'a, T>(&'a mut self, code: &str) -> Result<T, LuaError>
                          where T: for<'g> LuaRead<PushGuard<&'g mut PushGuard<&'a mut Lua<'lua>>>>
    {
        let mut f = try!(functions_read::LuaFunction::load(self, code));
        f.call()
    }

    /// Executes some Lua code on the context.
    pub fn execute_from_reader<'a, T, R>(&'a mut self, code: R) -> Result<T, LuaError>
            where T: for<'g> LuaRead<PushGuard<&'g mut PushGuard<&'a mut Lua<'lua>>>>,
                  R: Read
    {
        let mut f = try!(functions_read::LuaFunction::load_from_reader(self, code));
        f.call()
    }

    /// Reads the value of a global variable.
    pub fn get<'l, V, I>(&'l mut self, index: I) -> Option<V>
                         where I: Borrow<str>, V: LuaRead<PushGuard<&'l mut Lua<'lua>>>
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe { ffi::lua_getglobal(self.lua.0, index.as_ptr()); }
        if unsafe { ffi::lua_isnil(self.as_lua().0, -1) } {
            let _guard = PushGuard { lua: self, size: 1 };
            return None;
        }
        let guard = PushGuard { lua: self, size: 1 };
        LuaRead::lua_read(guard).ok()
    }

    /// Reads the value of a global, capturing the context by value.
    pub fn into_get<V, I>(self, index: I) -> Result<V, PushGuard<Self>>
        where I: Borrow<str>, V: LuaRead<PushGuard<Lua<'lua>>>
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe { ffi::lua_getglobal(self.lua.0, index.as_ptr()); }
        let is_nil = unsafe { ffi::lua_isnil(self.as_lua().0, -1) };
        let guard = PushGuard { lua: self, size: 1 };
        if is_nil {
            Err(guard)
        } else {
            LuaRead::lua_read(guard)
        }
    }

    /// Modifies the value of a global variable.
    pub fn set<I, V>(&mut self, index: I, value: V)
                         where I: Borrow<str>, for<'a> V: Push<&'a mut Lua<'lua>>
    {
        let index = CString::new(index.borrow()).unwrap();
        value.push_to_lua(self).forget();
        unsafe { ffi::lua_setglobal(self.lua.0, index.as_ptr()); }
    }

    /// Inserts an empty array, then loads it.
    pub fn empty_array<'a, I>(&'a mut self, index: I) -> LuaTable<PushGuard<&'a mut Lua<'lua>>>
                              where I: Borrow<str>
    {
        // TODO: cleaner implementation
        let mut me = self;
        let index2 = CString::new(index.borrow()).unwrap();
        Vec::<u8>::with_capacity(0).push_to_lua(&mut me).forget();
        unsafe { ffi::lua_setglobal(me.lua.0, index2.as_ptr()); }

        me.get(index).unwrap()
    }
}

impl<'lua> Drop for Lua<'lua> {
    fn drop(&mut self) {
        if self.must_be_closed {
            unsafe { ffi::lua_close(self.lua.0) }
        }
    }
}

impl<L> Drop for PushGuard<L> where L: AsMutLua {
    fn drop(&mut self) {
        if self.size != 0 {
            unsafe { ffi::lua_pop(self.lua.as_mut_lua().0, self.size); }
        }
    }
}
