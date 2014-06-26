#![crate_id = "rust-hl-lua"]
#![crate_type = "lib"]
#![comment = "Lua bindings for Rust"]
#![license = "MIT"]
#![allow(visible_private_types)]
#![feature(macro_rules)]

extern crate libc;
extern crate std;

pub mod functions_read;
mod functions_write;
mod liblua;
mod tables;
pub mod userdata;
mod values;

/**
 * Main object of the library
 */
pub struct Lua {
    lua: *mut liblua::lua_State
}

/**
 * Object which allows access to a Lua variable
 */
pub struct LoadedVariable<'a> {
    lua: &'a mut Lua,
    size: uint       // number of elements at the top of the stack
}

/**
 * Should be implemented by whatever type is pushable on the Lua stack
 */
pub trait Pushable {
    /**
     * Pushes the value on the top of the stack
     * Must return the number of elements pushed
     */
    fn push_to_lua(&self, lua: &mut Lua) -> uint;

    /**
     * Pushes over another element
     */
    fn push_over<'a>(&self, mut over: LoadedVariable<'a>)
        -> (LoadedVariable<'a>, uint)
    {
        let val = self.push_to_lua(over.lua);
        over.size += val;
        (over, val)
    }
}

/**
 * Should be implemented by whatever type can be read from the Lua stack
 */
pub trait Readable {
    /**
     * # Arguments
     *  * `lua` - The Lua object to read from
     *  * `index` - The index on the stack to read from
     */
    fn read_from_lua(lua: &mut Lua, index: i32) -> Option<Self>;
}

/**
 * Types that can be indices in Lua tables
 */
pub trait Index: Pushable + Readable {
}

/**
 * Object which can store variables
 */
pub trait Table<I, LV> {
    /// Loads the given index at the top of the stack
    fn get<V: Readable>(&mut self, &I) -> Option<V>;
    /// Stores the value in the table
    fn set<V: Pushable>(&mut self, &I, V) -> Result<(), &'static str>;
    ///
    fn access(self, index: &I) -> LV;
}

/**
 * Represents the global variables
 */
struct Globals<'a> {
    lua: &'a mut Lua
}

/**
 * Error that can happen when executing Lua code
 */
#[deriving(Show)]
pub enum ExecutionError {
    SyntaxError(String),
    ExecError(String)
}

/**
 * 
 */
pub type UserData<T> = userdata::UserData<T>;



// this alloc function is required to create a lua state
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

// called whenever lua encounters an unexpected error
extern "C" fn panic(lua: *mut liblua::lua_State) -> libc::c_int {
    let err = unsafe { liblua::lua_tostring(lua, -1) };
    fail!("PANIC: unprotected error in call to Lua API ({})\n", err);
}

impl Lua {
    /**
     * Builds a new Lua context
     * # Failure
     * The function fails if lua_newstate fails (which indicates lack of memory)
     */
    pub fn new() -> Lua {
        let lua = unsafe { liblua::lua_newstate(alloc, std::ptr::mut_null()) };
        if lua.is_null() {
            fail!("lua_newstate failed");
        }

        unsafe { liblua::lua_atpanic(lua, panic) };

        Lua { lua: lua }
    }

    /**
     * Executes some Lua code on the context
     */
    pub fn execute<T: Readable>(&mut self, code: &str) -> Result<T, ExecutionError> {
        let mut f = try!(functions_read::LuaFunction::load(self, code));
        f.call()
    }

    pub fn access<'a, I: Str>(&'a mut self, index: I) -> LoadedVariable<'a> {
        let g = Globals{lua: self};
        g.access(&index)
    }

    /**
     * Reads the value of a global variable
     */
    pub fn get<I: Str, V: Readable>(&mut self, index: I) -> Option<V> {
        let mut g = Globals{lua: self};
        g.get(&index)
    }

    /**
     * Modifies the value of a global variable
     */
    pub fn set<I: Str, V: Pushable>(&mut self, index: I, value: V) -> Result<(), &'static str> {
        let mut g = Globals{lua: self};
        g.set(&index, value)
    }
}

impl Drop for Lua {
    fn drop(&mut self) {
        unsafe { liblua::lua_close(self.lua) }
    }
}

// TODO: this destructor crash the compiler
// https://github.com/mozilla/rust/issues/13853
// https://github.com/mozilla/rust/issues/14377
/*impl<'a> Drop for LoadedVariable<'a> {
    fn drop(&mut self) {
        unsafe { liblua::lua_pop(self.lua.lua, self.size as i32) }
    }
}*/

impl<'a, I: Str> Table<I, LoadedVariable<'a>> for Globals<'a> {
    fn get<V: Readable>(&mut self, index: &I) -> Option<V> {
        unsafe { liblua::lua_getglobal(self.lua.lua, index.as_slice().to_c_str().unwrap()); }
        let val = Readable::read_from_lua(self.lua, -1);
        unsafe { liblua::lua_pop(self.lua.lua, 1); }
        val
    }

    fn set<V: Pushable>(&mut self, index: &I, value: V) -> Result<(), &'static str> {
        value.push_to_lua(self.lua);
        unsafe { liblua::lua_setglobal(self.lua.lua, index.as_slice().to_c_str().unwrap()); }
        Ok(())
    }

    fn access(self, index: &I) -> LoadedVariable<'a> {
        unsafe {
            liblua::lua_getglobal(self.lua.lua, index.as_slice().to_c_str().unwrap());
        }

        // TODO: check if not null

        LoadedVariable {
            lua: self.lua,
            size: 1
        }
    }
}

impl<'a, I: Index> Table<I, LoadedVariable<'a>> for LoadedVariable<'a> {
    fn get<V: Readable>(&mut self, index: &I) -> Option<V> {
        index.push_to_lua(self.lua);
        unsafe { liblua::lua_gettable(self.lua.lua, -2); }
        let val = Readable::read_from_lua(self.lua, -1);
        unsafe { liblua::lua_pop(self.lua.lua, 1); }
        val
    }

    fn set<V: Pushable>(&mut self, index: &I, value: V) -> Result<(), &'static str> {
        value.push_to_lua(self.lua);
        index.push_to_lua(self.lua);
        unsafe { liblua::lua_settable(self.lua.lua, -3); }
        unsafe { liblua::lua_pop(self.lua.lua, 1); }
        Ok(())
    }

    fn access(self, index: &I) -> LoadedVariable<'a> {
        index.push_to_lua(self.lua);
        unsafe { liblua::lua_gettable(self.lua.lua, -2); }

        LoadedVariable {
            lua: self.lua,
            size: self.size + 1
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn globals_readwrite() {
        let mut lua = super::Lua::new();

        lua.set("a", 2).unwrap();
        let x: int = lua.get("a").unwrap();
        assert_eq!(x, 2)
    }

    #[test]
    fn execute() {
        let mut lua = super::Lua::new();

        let val: int = lua.execute("return 5").unwrap();
        assert_eq!(val, 5);
    }

    // TODO: doesn't compile, have absolutely NO IDEA why
    /*#[test]
    fn table_readwrite() {
        let mut lua = super::Lua::new();

        lua.execute("a = { foo = 5 }");

        assert_eq!(lua.access("a").get(&"foo").unwrap(), 2);

        {   let access = lua.access("a");
            access.set(5, 3);
            assert_eq!(access.get(5), 3);
        }
    }*/
}
