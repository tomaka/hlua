use ffi;
use libc;

use std::ffi::CString;
use std::ffi::IntoBytes;
use std::io::Cursor;
use std::io::Read;
use std::io::Error as IoError;
use std::mem;
use std::ptr;

use AsMutLua;

use LuaRead;
use LuaError;
use PushGuard;

///
pub struct LuaFunction<L> {
    variable: L
}

struct ReadData {
    reader: Box<Read + 'static>,
    buffer: [u8 ; 128],
    triggered_error: Option<IoError>,
}

extern fn reader(_: *mut ffi::lua_State, data_raw: *mut libc::c_void, size: *mut libc::size_t) -> *const libc::c_char {
    let data: &mut ReadData = unsafe { mem::transmute(data_raw) };

    if data.triggered_error.is_some() {
        unsafe { (*size) = 0 }
        return data.buffer.as_ptr() as *const libc::c_char;
    }

    match data.reader.read(data.buffer.as_mut_slice()) {
        Ok(len) =>
            unsafe { (*size) = len as libc::size_t },
        Err(e) => {
            unsafe { (*size) = 0 }
            data.triggered_error = Some(e)
        },
    };

    data.buffer.as_ptr() as *const libc::c_char
}

impl<L> LuaFunction<L> where L: AsMutLua {
    pub fn call<V>(&mut self) -> Result<V, LuaError> where V: for<'a> LuaRead<&'a mut L> {
        // calling pcall pops the parameters and pushes output
        let pcall_return_value = unsafe { ffi::lua_pcall(self.variable.as_mut_lua().0, 0, 1, 0) };     // TODO:

        // if pcall succeeded, returning
        if pcall_return_value == 0 {
            return match LuaRead::lua_read(&mut self.variable) {
                None => Err(LuaError::WrongType),
                Some(x) => Ok(x)
            };
        }

        // an error occured during execution
        if pcall_return_value == ffi::LUA_ERRMEM {
            panic!("lua_pcall returned LUA_ERRMEM");
        }

        if pcall_return_value == ffi::LUA_ERRRUN {
            let error_msg: String = LuaRead::lua_read(&mut self.variable).expect("can't find error message at the top of the Lua stack");
            unsafe { ffi::lua_pop(self.variable.as_mut_lua().0, 1) };
            return Err(LuaError::ExecutionError(error_msg));
        }

        panic!("Unknown error code returned by lua_pcall: {}", pcall_return_value)
    }

    pub fn load_from_reader<R>(mut lua: L, code: R) -> Result<LuaFunction<PushGuard<L>>, LuaError>
                               where R: Read + 'static
    {
        let readdata = ReadData {
            reader: Box::new(code),
            buffer: unsafe { ::std::mem::uninitialized() },
            triggered_error: None,
        };


        let chunk_name = CString::new("chunk").unwrap();
        let load_return_value = unsafe { ffi::lua_load(lua.as_mut_lua().0, reader, mem::transmute(&readdata), chunk_name.as_ptr(), ptr::null()) };

        if readdata.triggered_error.is_some() {
            let error = readdata.triggered_error.unwrap();
            return Err(LuaError::ReadError(error));
        }

        if load_return_value == 0 {
            return Ok(LuaFunction{
                variable: PushGuard {
                    lua: lua,
                    size: 1
                }
            });
        }

        let error_msg: String = LuaRead::lua_read(&mut lua).expect("can't find error message at the top of the Lua stack");
        unsafe { ffi::lua_pop(lua.as_mut_lua().0, 1) };

        if load_return_value == ffi::LUA_ERRMEM {
            panic!("LUA_ERRMEM");
        }
        if load_return_value == ffi::LUA_ERRSYNTAX {
            return Err(LuaError::SyntaxError(error_msg));
        }

        panic!("Unknown error while calling lua_load");
    }

    pub fn load(lua: L, code: &str) -> Result<LuaFunction<PushGuard<L>>, LuaError> {
        let code = code.into_bytes();
        let reader = Cursor::new(code);
        LuaFunction::load_from_reader(lua, reader)
    }
}

// TODO: return Result<Ret, ExecutionError> instead
/*impl<'a, 'lua, Ret: CopyRead> ::std::ops::FnMut<(), Ret> for LuaFunction<'a,'lua> {
    fn call_mut(&mut self, _: ()) -> Ret {
        self.call().unwrap()
    }
}*/

impl<L> LuaRead<L> for LuaFunction<L> where L: AsMutLua {
    fn lua_read_at_position(mut lua: L, index: i32)
                            -> Option<LuaFunction<L>>
    {
        assert!(index == -1);   // FIXME:
        if unsafe { ffi::lua_isfunction(lua.as_mut_lua().0, -1) } {
            Some(LuaFunction { variable: lua })
        } else {
            None
        }
    }
}