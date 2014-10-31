#![crate_name = "rust-hl-lua"]
#![crate_type = "lib"]
#![comment = "Lua bindings for Rust"]
#![license = "MIT"]
#![allow(visible_private_types)]
#![feature(macro_rules)]
#![feature(unsafe_destructor)]

extern crate libc;
extern crate collections;

use std::io::IoError;
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
}

/// Trait for objects that have access to a Lua context.
/// The lifetime parameter is the lifetime of the Lua context.
pub trait HasLua {
    fn use_lua(&mut self) -> *mut ffi::lua_State;
}

impl<'lua> HasLua for Lua<'lua> {
    fn use_lua(&mut self) -> *mut ffi::lua_State {
        self.lua
    }
}

/// Object which allows access to a Lua variable.
#[doc(hidden)]
pub struct LoadedVariable<'var, L: 'var> {
    lua: &'var mut L,
    size: uint,       // number of elements over "lua"
}

impl<'var, 'lua, L: HasLua> HasLua for LoadedVariable<'var, L> {
    fn use_lua(&mut self) -> *mut ffi::lua_State {
        self.lua.use_lua()
    }
}

/// Should be implemented by whatever type is pushable on the Lua stack.
#[unstable]
pub trait Push<L> {
    /// Pushes the value on the top of the stack.
    /// Must return the number of elements pushed.
    ///
    /// You can implement this for any type you want by redirecting to call to
    /// another implementation (for example `5.push_to_lua`) or by calling `userdata::push_userdata`
    fn push_to_lua(self, lua: &mut L) -> uint;
}

/// Should be implemented by types that can be read by consomming a LoadedVariable.
#[unstable]
pub trait ConsumeRead<'a, L> {
    /// Returns the LoadedVariable in case of failure.
    fn read_from_variable(var: LoadedVariable<'a, L>) -> Result<Self, LoadedVariable<'a, L>>;
}

/// Should be implemented by whatever type can be read by copy from the Lua stack.
#[unstable]
pub trait CopyRead<L> {
    /// Reads an object from the Lua stack.
    ///
    /// Similar to Push, you can implement this trait for your own types either by
    /// redirecting the calls to another implementation or by calling userdata::read_copy_userdata
    ///
    /// # Arguments
    ///  * `lua` - The Lua object to read from
    ///  * `index` - The index on the stack to read from
    fn read_from_lua(lua: &mut L, index: i32) -> Option<Self>;
}

/// Types that can be indices in Lua tables.
#[unstable]
pub trait Index<L>: Push<L> + CopyRead<L> {
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

    /// There was an IoError while reading the source code to execute.
    ReadError(IoError),

    /// The call to `execute` has requested the wrong type of data.
    WrongType
}


// this alloc function is required to create a lua state.
extern "C" fn alloc(_ud: *mut libc::c_void, ptr: *mut libc::c_void, _osize: libc::size_t, nsize: libc::size_t) -> *mut libc::c_void {
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
    panic!("PANIC: unprotected error in call to Lua API ({})\n", err);
}

impl<'lua> Lua<'lua> {
    /// Builds a new Lua context.
    ///
    /// # Panic
    /// The function panics if lua_newstate fails (which indicates lack of memory).
    #[stable]
    pub fn new() -> Lua<'lua> {
        let lua = unsafe { ffi::lua_newstate(alloc, std::ptr::null_mut()) };
        if lua.is_null() {
            panic!("lua_newstate failed");
        }

        unsafe { ffi::lua_atpanic(lua, panic) };

        Lua {
            lua: lua,
            marker: ContravariantLifetime,
            must_be_closed: true,
        }
    }

    /// Takes an existing lua_State and build a Lua object from it.
    ///
    /// # Arguments
    ///  * close_at_the_end: if true, lua_close will be called on the lua_State on the destructor
    #[unstable]
    pub unsafe fn from_existing_state<T>(lua: *mut T, close_at_the_end: bool) -> Lua<'lua> {
        Lua {
            lua: std::mem::transmute(lua),
            marker: ContravariantLifetime,
            must_be_closed: close_at_the_end,
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
    pub fn execute<'a, T: CopyRead<LoadedVariable<'a, Lua<'lua>>>>(&'a mut self, code: &str) -> Result<T, LuaError> {
        let mut f = try!(functions_read::LuaFunction::load(self, code));
        f.call()
    }

    /// Executes some Lua code on the context.
    #[unstable]
    pub fn execute_from_reader<'a, T: CopyRead<LoadedVariable<'a, Lua<'lua>>>, R: std::io::Reader + 'static>(&'a mut self, code: R) -> Result<T, LuaError> {
        let mut f = try!(functions_read::LuaFunction::load_from_reader(self, code));
        f.call()
    }

    /// Loads the value of a global variable.
    #[unstable]
    pub fn load<'a, I: Str, V: ConsumeRead<'a, Lua<'lua>>>(&'a mut self, index: I) -> Option<V> {
        unsafe { ffi::lua_getglobal(self.lua, index.as_slice().to_c_str().unwrap()); }
        ConsumeRead::read_from_variable(LoadedVariable { lua: self, size: 1 }).ok()
    }

    /// Loads the value of a global variable as a table.
    #[unstable]
    pub fn load_table<'a, I: Str>(&'a mut self, index: I) -> Option<LuaTable<Lua<'lua>>> {
        self.load(index)
    }

    /// Reads the value of a global variable by copying it.
    #[unstable]
    pub fn get<I: Str, V: CopyRead<Lua<'lua>>>(&mut self, index: I) -> Option<V> {
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
    pub fn load_new_table<'var>(&'var mut self) -> LuaTable<'var, Lua<'lua>> {
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

// TODO: crashes the compiler
#[unsafe_destructor]
impl<'a, L: HasLua> Drop for LoadedVariable<'a, L> {
    fn drop(&mut self) {
        unsafe { ffi::lua_pop(self.use_lua(), self.size as libc::c_int) }
    }
}
