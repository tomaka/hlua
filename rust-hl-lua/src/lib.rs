#![crate_name = "rust-hl-lua"]
#![crate_type = "lib"]
#![comment = "Lua bindings for Rust"]
#![license = "MIT"]
#![allow(visible_private_types)]
#![feature(macro_rules)]
#![feature(unsafe_destructor)]

extern crate libc;
extern crate collections;

use std::kinds::marker::ContravariantLifetime;

pub use lua_tables::LuaTable;
pub use functions_read::LuaFunction;

pub mod any;
pub mod functions_read;
pub mod lua_tables;
pub mod userdata;

mod ffi;
mod functions_write;
mod rust_tables;
mod tuples;
mod values;


/// Main object of the library.
/// The lifetime parameter corresponds to the lifetime of the Lua object itself.
#[unstable]
pub struct Lua<'lua> {
    lua: *mut ffi::lua_State,
    marker: ContravariantLifetime<'lua>,
    must_be_closed: bool,
    inside_callback: bool           // if true, we are inside a callback
}

/// Trait for objects that have access to a Lua context.
pub trait HasLua {
    fn use_lua(&mut self) -> *mut ffi::lua_State;
}

impl<'lua> HasLua for Lua<'lua> {
    fn use_lua(&mut self) -> *mut ffi::lua_State {
        self.lua
    }
}

/// Object which allows access to a Lua variable.
struct LoadedVariable<'var, 'lua> {
    lua: &'var mut Lua<'lua>,
    size: uint       // number of elements at the top of the stack
}

/// Should be implemented by whatever type is pushable on the Lua stack.
#[unstable]
pub trait Push<L: HasLua> {
    /// Pushes the value on the top of the stack.
    /// Must return the number of elements pushed.
    ///
    /// You can implement this for any type you want by redirecting to call to
    /// another implementation (for example `5.push_to_lua`) or by calling `userdata::push_userdata`
    fn push_to_lua(self, lua: &mut L) -> uint;
}

/// Should be implemented by types that can be read by consomming a LoadedVariable.
#[unstable]
pub trait ConsumeRead<'a, 'lua> {
    /// Returns the LoadedVariable in case of failure.
    fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<Self, LoadedVariable<'a, 'lua>>;
}

/// Should be implemented by whatever type can be read by copy from the Lua stack.
#[unstable]
pub trait CopyRead : Clone {
    /// Reads an object from the Lua stack.
    ///
    /// Similar to Push, you can implement this trait for your own types either by
    /// redirecting the calls to another implementation or by calling userdata::read_copy_userdata
    ///
    /// # Arguments
    ///  * `lua` - The Lua object to read from
    ///  * `index` - The index on the stack to read from
    fn read_from_lua<'lua>(lua: &mut Lua<'lua>, index: i32) -> Option<Self>;
}

/// Types that can be indices in Lua tables.
#[unstable]
pub trait Index<L>: Push<L> + CopyRead {
}

/// Error that can happen when executing Lua code.
#[deriving(Show)]
#[unstable]
pub enum LuaError {
    /// There was a syntax error when parsing the Lua code.
    SyntaxError(String),

    /// There was an error during execution of the Lua code
    /// (for example not enough parameters for a function call).
    ExecutionError(String),

    /// The call to `execute` has requested the wrong type of data.
    WrongType
}


// this alloc function is required to create a lua state.
extern "C" fn alloc(_ud: *mut libc::c_void, ptr: *mut libc::c_void, _osize: libc::size_t, nsize: libc::size_t) -> *mut libc::c_void {
    unsafe {
        if nsize == 0 {
            libc::free(ptr as *mut libc::c_void);
            std::ptr::mut_null()
        } else {
            libc::realloc(ptr, nsize)
        }
    }
}

// called whenever lua encounters an unexpected error.
extern "C" fn panic(lua: *mut ffi::lua_State) -> libc::c_int {
    let err = unsafe { ffi::lua_tostring(lua, -1) };
    fail!("PANIC: unprotected error in call to Lua API ({})\n", err);
}

impl<'lua> Lua<'lua> {
    /// Builds a new Lua context.
    /// 
    /// # Failure
    /// The function fails if lua_newstate fails (which indicates lack of memory).
    #[stable]
    pub fn new() -> Lua {
        let lua = unsafe { ffi::lua_newstate(alloc, std::ptr::mut_null()) };
        if lua.is_null() {
            fail!("lua_newstate failed");
        }

        unsafe { ffi::lua_atpanic(lua, panic) };

        Lua {
            lua: lua,
            marker: ContravariantLifetime,
            must_be_closed: true,
            inside_callback: false
        }
    }

    /// Takes an existing lua_State and build a Lua object from it.
    /// 
    /// # Arguments
    ///  * close_at_the_end: if true, lua_close will be called on the lua_State on the destructor
    #[unstable]
    pub unsafe fn from_existing_state<T>(lua: *mut T, close_at_the_end: bool) -> Lua {
        Lua {
            lua: std::mem::transmute(lua),
            marker: ContravariantLifetime,
            must_be_closed: close_at_the_end,
            inside_callback: false
        }
    }

    /// Opens all standard Lua libraries.
    /// This is done by calling `luaL_openlibs`.
    #[unstable]
    pub fn openlibs(&mut self) {
        unsafe { ffi::luaL_openlibs(self.lua) }
    }

    /// Executes some Lua code on the context.
    #[unstable]
    pub fn execute<T: CopyRead>(&mut self, code: &str) -> Result<T, LuaError> {
        let mut f = try!(functions_read::LuaFunction::load(self, code));
        f.call()
    }

    /// Executes some Lua code on the context.
    #[unstable]
    pub fn execute_from_reader<T: CopyRead, R: std::io::Reader + 'static>(&mut self, code: R) -> Result<T, LuaError> {
        let mut f = try!(functions_read::LuaFunction::load_from_reader(self, code));
        f.call()
    }

    /// Loads the value of a global variable.
    #[unstable]
    pub fn load<'a, I: Str, V: ConsumeRead<'a, 'lua>>(&'a mut self, index: I) -> Option<V> {
        unsafe { ffi::lua_getglobal(self.lua, index.as_slice().to_c_str().unwrap()); }
        ConsumeRead::read_from_variable(LoadedVariable { lua: self, size: 1 }).ok()
    }

    /// Reads the value of a global variable by copying it.
    #[unstable]
    pub fn get<I: Str, V: CopyRead>(&mut self, index: I) -> Option<V> {
        unsafe { ffi::lua_getglobal(self.lua, index.as_slice().to_c_str().unwrap()); }
        CopyRead::read_from_lua(self, -1)
    }

    /// Modifies the value of a global variable.
    #[unstable]
    pub fn set<I: Str, V: Push<Lua<'lua>>>(&mut self, index: I, value: V) {
        value.push_to_lua(self);
        unsafe { ffi::lua_setglobal(self.lua, index.as_slice().to_c_str().unwrap()); }
    }

    #[unstable]
    pub fn load_new_table<'var>(&'var mut self) -> LuaTable<'var, 'lua> {
        unsafe { ffi::lua_newtable(self.lua) };
        ConsumeRead::read_from_variable(LoadedVariable { lua: self, size: 1 }).ok().unwrap()
    }
}

#[unsafe_destructor]
impl<'lua> Drop for Lua<'lua> {
    fn drop(&mut self) {
        if self.must_be_closed {
            unsafe { ffi::lua_close(self.lua) }
        }
    }
}

#[unsafe_destructor]
impl<'a, 'lua> Drop for LoadedVariable<'a, 'lua> {
    fn drop(&mut self) {
        unsafe { ffi::lua_pop(self.lua.lua, self.size as libc::c_int) }
    }
}
