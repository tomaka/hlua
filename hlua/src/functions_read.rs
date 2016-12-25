use ffi;
use libc;

use std::io::Cursor;
use std::io::Read;
use std::io::Error as IoError;
use std::mem;
use std::ptr;

use AsMutLua;

use LuaRead;
use LuaError;
use Push;
use PushGuard;

pub struct LuaCode<'a>(&'a str);

impl<'lua, 'c, L> Push<L> for LuaCode<'c> where L: AsMutLua<'lua> {
    #[inline]
    fn push_to_lua(self, lua: L) -> PushGuard<L> {
        LuaCodeFromReader(Cursor::new(self.0.as_bytes())).push_to_lua(lua)
    }
}

pub struct LuaCodeFromReader<R>(R);

impl<'lua, L, R> Push<L> for LuaCodeFromReader<R>
    where L: AsMutLua<'lua>,
          R: Read
{
    #[inline]
    fn push_to_lua(self, mut lua: L) -> PushGuard<L> {
        struct ReadData<R> {
            reader: R,
            buffer: [u8; 128],
            triggered_error: Option<IoError>,
        }

        extern "C" fn reader<R>(_: *mut ffi::lua_State,
                                data_raw: *mut libc::c_void,
                                size: *mut libc::size_t)
                                -> *const libc::c_char
            where R: Read
        {
            let data: &mut ReadData<R> = unsafe { mem::transmute(data_raw) };

            if data.triggered_error.is_some() {
                unsafe { (*size) = 0 }
                return data.buffer.as_ptr() as *const libc::c_char;
            }

            match data.reader.read(&mut data.buffer) {
                Ok(len) => unsafe { (*size) = len as libc::size_t },
                Err(e) => {
                    unsafe { (*size) = 0 }
                    data.triggered_error = Some(e)
                }
            };

            data.buffer.as_ptr() as *const libc::c_char
        }

        let readdata = ReadData {
            reader: self.0,
            buffer: unsafe { ::std::mem::uninitialized() },
            triggered_error: None,
        };

        let (load_return_value, pushed_value) = unsafe {
            let code = ffi::lua_load(lua.as_mut_lua().0,
                                     reader::<R>,
                                     mem::transmute(&readdata),
                                     b"chunk\0".as_ptr() as *const _,
                                     ptr::null());                         
            let raw_lua = lua.as_lua();
            (code,
             PushGuard {
                 lua: lua,
                 size: 1,
                 raw_lua: raw_lua,
             })
        };

        if readdata.triggered_error.is_some() {
            let error = readdata.triggered_error.unwrap();
            panic!()    // TODO: return Err(LuaError::ReadError(error));
        }

        if load_return_value == 0 {
            return pushed_value;
        }

        let error_msg: String = LuaRead::lua_read(pushed_value)
            .ok()
            .expect("can't find error message at the top of the Lua stack");

        if load_return_value == ffi::LUA_ERRMEM {
            panic!("LUA_ERRMEM");
        }

        if load_return_value == ffi::LUA_ERRSYNTAX {
            panic!() // TODO: return Err(LuaError::SyntaxError(error_msg));
        }

        panic!("Unknown error while calling lua_load");
    }
}

///
pub struct LuaFunction<L> {
    variable: L,
}

impl<'lua, L> LuaFunction<L>
    where L: AsMutLua<'lua>
{
    /// Calls the `LuaFunction`.
    #[inline]
    pub fn call<'a, V>(&'a mut self) -> Result<V, LuaError>
        where V: LuaRead<PushGuard<&'a mut L>>
    {
        // calling pcall pops the parameters and pushes output
        let (pcall_return_value, pushed_value) = unsafe {
            // lua_pcall pops the function, so we have to make a copy of it
            ffi::lua_pushvalue(self.variable.as_mut_lua().0, -1);
            let pcall_return_value = ffi::lua_pcall(self.variable.as_mut_lua().0, 0, 1, 0);     // TODO: arguments
            let raw_lua = self.variable.as_lua();
            (pcall_return_value,
             PushGuard {
                 lua: &mut self.variable,
                 size: 1,
                 raw_lua: raw_lua,
             })
        };

        // if pcall succeeded, returning
        if pcall_return_value == 0 {
            return match LuaRead::lua_read(pushed_value) {
                Err(_) => Err(LuaError::WrongType),
                Ok(x) => Ok(x),
            };
        }

        // an error occured during execution
        if pcall_return_value == ffi::LUA_ERRMEM {
            panic!("lua_pcall returned LUA_ERRMEM");
        }

        if pcall_return_value == ffi::LUA_ERRRUN {
            let error_msg: String = LuaRead::lua_read(pushed_value)
                .ok()
                .expect("can't find error message at the top of the Lua stack");
            return Err(LuaError::ExecutionError(error_msg));
        }

        panic!("Unknown error code returned by lua_pcall: {}", pcall_return_value)
    }

    /// Builds a new `LuaFunction` from the code of a reader.
    #[inline]
    pub fn load_from_reader<R>(lua: L, code: R) -> Result<LuaFunction<PushGuard<L>>, LuaError>
        where R: Read
    {
        let pushed = LuaCodeFromReader(code).push_to_lua(lua);
        Ok(LuaFunction { variable: pushed })
    }

    /// Builds a new `LuaFunction` from a raw string.
    #[inline]
    pub fn load(lua: L, code: &str) -> Result<LuaFunction<PushGuard<L>>, LuaError> {
        let reader = Cursor::new(code.as_bytes());
        LuaFunction::load_from_reader(lua, reader)
    }
}

// TODO: return Result<Ret, ExecutionError> instead
// impl<'a, 'lua, Ret: CopyRead> ::std::ops::FnMut<(), Ret> for LuaFunction<'a,'lua> {
// fn call_mut(&mut self, _: ()) -> Ret {
// self.call().unwrap()
// }
// }

impl<'lua, L> LuaRead<L> for LuaFunction<L>
    where L: AsMutLua<'lua>
{
    #[inline]
    fn lua_read_at_position(mut lua: L, index: i32) -> Result<LuaFunction<L>, L> {
        assert!(index == -1);   // FIXME:
        if unsafe { ffi::lua_isfunction(lua.as_mut_lua().0, -1) } {
            Ok(LuaFunction { variable: lua })
        } else {
            Err(lua)
        }
    }
}
