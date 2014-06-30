extern crate libc;
extern crate std;

use liblua;
use { Lua, ConsumeReadable, CopyReadable, LoadedVariable, LuaError, ExecutionError, WrongType, SyntaxError };

pub struct LuaFunction<'a> {
    variable: LoadedVariable<'a>
}

struct ReadData {
    reader: Box<std::io::Reader>,
    buffer: [u8, ..128]
}

extern fn reader(_: *mut liblua::lua_State, dataRaw: *mut libc::c_void, size: *mut libc::size_t) -> *const libc::c_char {
    let data: &mut ReadData = unsafe { std::mem::transmute(dataRaw) };

    match data.reader.read(data.buffer.as_mut_slice()) {
        Ok(len) => unsafe { (*size) = len as libc::size_t },
        Err(_) => unsafe { (*size) = 0 }
    };

    data.buffer.as_ptr() as *const libc::c_char
}

impl<'a> LuaFunction<'a> {
    pub fn call<V: CopyReadable>(&mut self) -> Result<V, LuaError> {
        // calling pcall pops the parameters and pushes output
        let pcallReturnValue = unsafe { liblua::lua_pcall(self.variable.lua.lua, 0, 1, 0) };     // TODO: 

        // if pcall succeeded, returning
        if pcallReturnValue == 0 {
            return match CopyReadable::read_from_lua(self.variable.lua, -1) {
                None => Err(WrongType),
                Some(x) => Ok(x)
            };
        }

        // an error occured during execution
        if pcallReturnValue == liblua::LUA_ERRMEM {
            fail!("lua_pcall returned LUA_ERRMEM");
        }

        if pcallReturnValue == liblua::LUA_ERRRUN {
            let errorMsg: String = CopyReadable::read_from_lua(self.variable.lua, -1).expect("can't find error message at the top of the Lua stack");
            unsafe { liblua::lua_pop(self.variable.lua.lua, 1) };
            return Err(ExecutionError(errorMsg));
        }

        fail!("Unknown error code returned by lua_pcall: {}", pcallReturnValue)
    }

    pub fn load_from_reader<'a, R: std::io::Reader + 'static>(lua: &'a mut Lua, code: R)
        -> Result<LuaFunction<'a>, LuaError>
    {
        let readdata = ReadData { reader: box code, buffer: unsafe { std::mem::uninitialized() } };

        let loadReturnValue = "chunk".with_c_str(|chunk|
            unsafe { liblua::lua_load(lua.lua, reader, std::mem::transmute(&readdata), chunk, std::ptr::null()) }
        );

        if loadReturnValue == 0 {
            return Ok(LuaFunction{
                variable: LoadedVariable{
                    lua: lua,
                    size: 1
                }
            });
        }

        let errorMsg: String = CopyReadable::read_from_lua(lua, -1).expect("can't find error message at the top of the Lua stack");
        unsafe { liblua::lua_pop(lua.lua, 1) };

        if loadReturnValue == liblua::LUA_ERRMEM {
            fail!("LUA_ERRMEM");
        }
        if loadReturnValue == liblua::LUA_ERRSYNTAX {
            return Err(SyntaxError(errorMsg));
        }

        fail!("Unknown error while calling lua_load");
    }

    pub fn load<'a>(lua: &'a mut Lua, code: &str)
        -> Result<LuaFunction<'a>, LuaError>
    {
        let reader = std::io::MemReader::new(code.to_c_str().as_bytes().init().to_owned());
        LuaFunction::load_from_reader(lua, reader)
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
    use { Lua, LuaError };

    #[test]
    fn basic() {
        let mut lua = Lua::new();

        let mut f = super::LuaFunction::load(&mut lua, "return 5;").unwrap();

        let val: int = f.call().unwrap();
        assert_eq!(val, 5);
    }

    #[test]
    fn syntax_error() {
        let mut lua = Lua::new();

        assert!(super::LuaFunction::load(&mut lua, "azerazer").is_err());
    }

    #[test]
    fn execution_error() {
        let mut lua = Lua::new();

        let mut f = super::LuaFunction::load(&mut lua, "return a:hello()").unwrap();

        let val: Result<int, LuaError> = f.call();
        assert!(val.is_err());
    }
}
