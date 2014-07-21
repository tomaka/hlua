use ffi;
use { Lua, ConsumeRead, CopyRead, LoadedVariable, LuaError, ExecutionError, WrongType, SyntaxError };

#[unstable]
pub struct LuaFunction<'a,'lua> {
    variable: LoadedVariable<'a,'lua>
}

struct ReadData {
    reader: Box<::std::io::Reader>,
    buffer: [u8, ..128]
}

extern fn reader(_: *mut ffi::lua_State, dataRaw: *mut ::libc::c_void, size: *mut ::libc::size_t) -> *const ::libc::c_char {
    let data: &mut ReadData = unsafe { ::std::mem::transmute(dataRaw) };

    match data.reader.read(data.buffer.as_mut_slice()) {
        Ok(len) => unsafe { (*size) = len as ::libc::size_t },
        Err(_) => unsafe { (*size) = 0 }
    };

    data.buffer.as_ptr() as *const ::libc::c_char
}

impl<'a,'lua> LuaFunction<'a,'lua> {
    pub fn call<V: CopyRead<Lua<'lua>>>(&mut self) -> Result<V, LuaError> {
        // calling pcall pops the parameters and pushes output
        let pcallReturnValue = unsafe { ffi::lua_pcall(self.variable.lua.lua, 0, 1, 0) };     // TODO: 

        // if pcall succeeded, returning
        if pcallReturnValue == 0 {
            return match CopyRead::read_from_lua(self.variable.lua, -1) {
                None => Err(WrongType),
                Some(x) => Ok(x)
            };
        }

        // an error occured during execution
        if pcallReturnValue == ffi::LUA_ERRMEM {
            fail!("lua_pcall returned LUA_ERRMEM");
        }

        if pcallReturnValue == ffi::LUA_ERRRUN {
            let errorMsg: String = CopyRead::read_from_lua(self.variable.lua, -1).expect("can't find error message at the top of the Lua stack");
            unsafe { ffi::lua_pop(self.variable.lua.lua, 1) };
            return Err(ExecutionError(errorMsg));
        }

        fail!("Unknown error code returned by lua_pcall: {}", pcallReturnValue)
    }

    pub fn load_from_reader<R: ::std::io::Reader + 'static>(lua: &'a mut Lua<'lua>, code: R)
        -> Result<LuaFunction<'a,'lua>, LuaError>
    {
        let readdata = ReadData { reader: box code, buffer: unsafe { ::std::mem::uninitialized() } };

        let loadReturnValue = "chunk".with_c_str(|chunk|
            unsafe { ffi::lua_load(lua.lua, reader, ::std::mem::transmute(&readdata), chunk, ::std::ptr::null()) }
        );

        if loadReturnValue == 0 {
            return Ok(LuaFunction{
                variable: LoadedVariable{
                    lua: lua,
                    size: 1
                }
            });
        }

        let errorMsg: String = CopyRead::read_from_lua(lua, -1).expect("can't find error message at the top of the Lua stack");
        unsafe { ffi::lua_pop(lua.lua, 1) };

        if loadReturnValue == ffi::LUA_ERRMEM {
            fail!("LUA_ERRMEM");
        }
        if loadReturnValue == ffi::LUA_ERRSYNTAX {
            return Err(SyntaxError(errorMsg));
        }

        fail!("Unknown error while calling lua_load");
    }

    pub fn load(lua: &'a mut Lua<'lua>, code: &str)
        -> Result<LuaFunction<'a,'lua>, LuaError>
    {
        let reader = ::std::io::MemReader::new(code.to_c_str().as_bytes().init().to_owned());
        LuaFunction::load_from_reader(lua, reader)
    }
}

// TODO: return Result<Ret, ExecutionError> instead
/*impl<'a, 'lua, Ret: CopyRead> ::std::ops::FnMut<(), Ret> for LuaFunction<'a,'lua> {
    fn call_mut(&mut self, _: ()) -> Ret {
        self.call().unwrap()
    }
}*/

impl<'a,'lua> ConsumeRead<'a,'lua> for LuaFunction<'a,'lua> {
    fn read_from_variable(var: LoadedVariable<'a, 'lua>)
        -> Result<LuaFunction<'a, 'lua>, LoadedVariable<'a, 'lua>>
    {
        if unsafe { ffi::lua_isfunction(var.lua.lua, -1) } {
            Ok(LuaFunction{ variable: var })
        } else {
            Err(var)
        }
    }
}
