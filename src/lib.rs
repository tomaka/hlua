#![crate_id = "rust-hl-lua"]
#![crate_type = "lib"]
#![comment = "Lua bindings for Rust"]
#![license = "MIT"]
#![allow(visible_private_types)]
#![feature(macro_rules)]

extern crate libc;
extern crate std;

mod liblua;
mod functions;
mod tables;
mod userdata;
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
    size: int       // number of elements at the top of the stack
}

/**
 * Should be implemented by whatever type is pushable on the Lua stack
 */
pub trait Pushable {
    fn push_to_lua(&self, &Lua);
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
    fn read_from_lua(lua: &Lua, index: i32) -> Option<Self>;
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

// TODO: don't put this here
extern {
    pub fn luaL_loadstring(L: *mut liblua::lua_State, s: *libc::c_char) -> libc::c_int;
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
        try!(self.load(code));
        self.call_stack_top()
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


    fn load(&mut self, code: &str) -> Result<(), ExecutionError> {
        let loadReturnValue = unsafe { luaL_loadstring(self.lua, code.to_c_str().unwrap()) };

        if loadReturnValue == 0 {
            return Ok(());
        }

        let errorMsg: String = Readable::read_from_lua(self, -1).unwrap();
        unsafe { liblua::lua_pop(self.lua, 1) };

        if loadReturnValue == liblua::LUA_ERRMEM {
            fail!("LUA_ERRMEM");
        }
        if loadReturnValue == liblua::LUA_ERRSYNTAX {
            return Err(SyntaxError(errorMsg));
        }

        fail!("Unknown error while calling lua_load");
    }

    fn call_stack_top<T: Readable>(&mut self) -> Result<T, ExecutionError> {
        // calling pcall pops the parameters and pushes output
        let pcallReturnValue = unsafe { liblua::lua_pcall(self.lua, 0, 1, 0) };     // TODO: 

        // if pcall succeeded, returning
        if pcallReturnValue == 0 {
            return match Readable::read_from_lua(self, -1) {
                None => fail!("Wrong type"),       // TODO: add to executionerror
                Some(x) => Ok(x)
            };
        }

        // an error occured during execution
        if pcallReturnValue == liblua::LUA_ERRMEM {
            fail!("LUA_ERRMEM");
        }

        if pcallReturnValue == liblua::LUA_ERRRUN {
            let errorMsg: String = Readable::read_from_lua(self, -1).unwrap();
            unsafe { liblua::lua_pop(self.lua, 1) };
            return Err(ExecError(errorMsg));
        }

        fail!("Unknown error code returned by lua_pcall")
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
