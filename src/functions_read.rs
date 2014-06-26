extern crate libc;
extern crate std;

use super::liblua;
use super::Lua;
use super::ConsumeReadable;
use super::CopyReadable;
use super::LoadedVariable;
use super::{ ExecutionError, ExecError };

pub struct LuaFunction<'a> {
    variable: LoadedVariable<'a>
}

// TODO: decide whether to keep this or not
extern {
    pub fn luaL_loadstring(L: *mut liblua::lua_State, s: *libc::c_char) -> libc::c_int;
}

impl<'a> LuaFunction<'a> {
    pub fn call<V: CopyReadable>(&mut self) -> Result<V, ExecutionError> {
        // calling pcall pops the parameters and pushes output
        let pcallReturnValue = unsafe { liblua::lua_pcall(self.variable.lua.lua, 0, 1, 0) };     // TODO: 

        // if pcall succeeded, returning
        if pcallReturnValue == 0 {
            return match CopyReadable::read_from_lua(self.variable.lua, -1) {
                None => fail!("Wrong type"),       // TODO: add to executionerror
                Some(x) => Ok(x)
            };
        }

        // an error occured during execution
        if pcallReturnValue == liblua::LUA_ERRMEM {
            fail!("lua_pcall returned LUA_ERRMEM");
        }

        if pcallReturnValue == liblua::LUA_ERRRUN {
            let errorMsg: String = CopyReadable::read_from_lua(self.variable.lua, -1).unwrap();
            unsafe { liblua::lua_pop(self.variable.lua.lua, 1) };
            return Err(ExecError(errorMsg));
        }

        fail!("Unknown error code returned by lua_pcall: {}", pcallReturnValue)
    }

    pub fn load<'a>(lua: &'a mut Lua, code: &str)
        -> Result<LuaFunction<'a>, super::ExecutionError>
    {
        let loadReturnValue = unsafe { luaL_loadstring(lua.lua, code.to_c_str().unwrap()) };

        if loadReturnValue == 0 {
            return Ok(LuaFunction{
                variable: LoadedVariable{
                    lua: lua,
                    size: 1
                }
            });
        }

        let errorMsg: String = CopyReadable::read_from_lua(lua, -1).unwrap();
        unsafe { liblua::lua_pop(lua.lua, 1) };

        if loadReturnValue == liblua::LUA_ERRMEM {
            fail!("LUA_ERRMEM");
        }
        if loadReturnValue == liblua::LUA_ERRSYNTAX {
            return Err(super::SyntaxError(errorMsg));
        }

        fail!("Unknown error while calling lua_load");
    }
}

// TODO: return Result<Ret, ExecutionError> instead
impl<'a, Ret: CopyReadable> std::ops::FnMut<(), Ret> for LuaFunction<'a> {
    fn call_mut(&mut self, _: ()) -> Ret {
        self.call().unwrap()
    }
}

impl<'a> ConsumeReadable<'a> for LuaFunction<'a> {
    fn read_from_variable(var: LoadedVariable<'a>)
        -> Result<LuaFunction<'a>, LoadedVariable<'a>>
    {
        if unsafe { liblua::lua_isfunction(var.lua.lua, -1) } {
            Ok(LuaFunction{ variable: var })
        } else {
            Err(var)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {
        let mut lua = super::super::Lua::new();

        let mut f = super::LuaFunction::load(&mut lua, "return 5;").unwrap();

        let val: int = f.call().unwrap();
        assert_eq!(val, 5);
    }

    #[test]
    fn syntax_error() {
        let mut lua = super::super::Lua::new();

        assert!(super::LuaFunction::load(&mut lua, "azerazer").is_err());
    }

    #[test]
    fn execution_error() {
        let mut lua = super::super::Lua::new();

        let mut f = super::LuaFunction::load(&mut lua, "return a:hello()").unwrap();

        let val: Result<int, super::super::ExecutionError> = f.call();
        assert!(val.is_err());
    }
}
