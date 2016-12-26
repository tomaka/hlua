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
use PushOne;

/// Wrapper around a `&str`. When pushed, the content will be parsed as Lua code and turned into a
/// function.
///
/// Since pushing this value can fail in case of a parsing error, you must use the `checked_set`
/// method instead of `set`.
///
/// > **Note**: This struct is a wrapper around `LuaCodeFromReader`. There's no advantage in using
/// > it except that it is more convenient. More advanced usages (such as returning a Lua function
/// > from a Rust function) can be done with `LuaCodeFromReader`.
///
/// # Example
///
/// ```
/// let mut lua = hlua::Lua::new();
/// lua.checked_set("hello", hlua::LuaCode("return 5")).unwrap();
///
/// let r: i32 = lua.execute("return hello();").unwrap();
/// assert_eq!(r, 5);
/// ```
pub struct LuaCode<'a>(pub &'a str);

impl<'lua, 'c, L> Push<L> for LuaCode<'c>
    where L: AsMutLua<'lua>
{
    type Err = LuaError;

    #[inline]
    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (LuaError, L)> {
        LuaCodeFromReader(Cursor::new(self.0.as_bytes())).push_to_lua(lua)
    }
}

impl<'lua, 'c, L> PushOne<L> for LuaCode<'c> where L: AsMutLua<'lua> {}

/// Wrapper around a `Read` object. When pushed, the content will be parsed as Lua code and turned
/// into a function.
///
/// Since pushing this value can fail in case of a reading error or a parsing error, you must use
/// the `checked_set` method instead of `set`.
///
/// # Example: returning a Lua function from a Rust function
///
/// ```
/// use std::io::Cursor;
///
/// let mut lua = hlua::Lua::new();
///
/// lua.set("call_rust", hlua::function0(|| -> hlua::LuaCodeFromReader<Cursor<String>> {
///     let lua_code = "return 18;";
///     return hlua::LuaCodeFromReader(Cursor::new(lua_code.to_owned()));
/// }));
///
/// let r: i32 = lua.execute("local lua_func = call_rust(); return lua_func();").unwrap();
/// assert_eq!(r, 18);
/// ```
pub struct LuaCodeFromReader<R>(pub R);

impl<'lua, L, R> Push<L> for LuaCodeFromReader<R>
    where L: AsMutLua<'lua>,
          R: Read
{
    type Err = LuaError;

    #[inline]
    fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (LuaError, L)> {
        unsafe {
            struct ReadData<R> {
                reader: R,
                buffer: [u8; 128],
                triggered_error: Option<IoError>,
            }

            let mut read_data = ReadData {
                reader: self.0,
                buffer: mem::uninitialized(),
                triggered_error: None,
            };

            extern "C" fn reader<R>(_: *mut ffi::lua_State,
                                    data: *mut libc::c_void,
                                    size: *mut libc::size_t)
                                    -> *const libc::c_char
                where R: Read
            {
                unsafe {
                    let data: *mut ReadData<R> = data as *mut _;
                    let data: &mut ReadData<R> = &mut *data;

                    if data.triggered_error.is_some() {
                        (*size) = 0;
                        return data.buffer.as_ptr() as *const libc::c_char;
                    }

                    match data.reader.read(&mut data.buffer) {
                        Ok(len) => (*size) = len as libc::size_t,
                        Err(e) => {
                            (*size) = 0;
                            data.triggered_error = Some(e);
                        }
                    };

                    data.buffer.as_ptr() as *const libc::c_char
                }
            }

            let (load_return_value, pushed_value) = {
                let code = ffi::lua_load(lua.as_mut_lua().0,
                                         reader::<R>,
                                         &mut read_data as *mut ReadData<_> as *mut libc::c_void,
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

            if read_data.triggered_error.is_some() {
                let error = read_data.triggered_error.unwrap();
                return Err((LuaError::ReadError(error), pushed_value.into_inner()));
            }

            if load_return_value == 0 {
                return Ok(pushed_value);
            }

            let error_msg: String = LuaRead::lua_read(&pushed_value)
                .ok()
                .expect("can't find error message at the top of the Lua stack");

            if load_return_value == ffi::LUA_ERRMEM {
                panic!("LUA_ERRMEM");
            }

            if load_return_value == ffi::LUA_ERRSYNTAX {
                return Err((LuaError::SyntaxError(error_msg), pushed_value.into_inner()));
            }

            panic!("Unknown error while calling lua_load");
        }
    }
}

impl<'lua, L, R> PushOne<L> for LuaCodeFromReader<R>
    where L: AsMutLua<'lua>,
          R: Read
{
}

/// Handle to a function in the Lua context.
///
/// Just like you can read variables as integers and strings, you can also read Lua functions by
/// requesting a `LuaFunction` object. Once you have a `LuaFunction` you can call it with `call()`.
///
/// > **Note**: Passing parameters when calling the function is not yet implemented.
///
/// # Example
///
/// ```
/// let mut lua = hlua::Lua::new();
/// lua.execute::<()>("function foo() return 12 end").unwrap();
///
/// let mut foo: hlua::LuaFunction<_> = lua.get("foo").unwrap();
/// let result: i32 = foo.call().unwrap();
/// assert_eq!(result, 12);
/// ```
// TODO: example for how to get a LuaFunction as a parameter of a Rust function
pub struct LuaFunction<L> {
    variable: L,
}

impl<'lua, L> LuaFunction<L>
    where L: AsMutLua<'lua>
{
    /// Calls the function.
    ///
    /// Returns an error if there is an error while executing the Lua code (eg. a function call
    /// returns an error), or if the requested return type doesn't match the actual return type.
    ///
    /// > **Note**: Passing parameters when calling is not yet implemented.
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

        panic!("Unknown error code returned by lua_pcall: {}",
               pcall_return_value)
    }

    /// Builds a new `LuaFunction` from the code of a reader.
    ///
    /// Returns an error if reading from the `Read` object fails or if there is a syntax error in
    /// the code.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let mut lua = hlua::Lua::new();
    ///
    /// let mut f = hlua::LuaFunction::load_from_reader(&mut lua, Cursor::new("return 8")).unwrap();
    /// let ret: i32 = f.call().unwrap();
    /// assert_eq!(ret, 8);
    /// ```
    #[inline]
    pub fn load_from_reader<R>(lua: L, code: R) -> Result<LuaFunction<PushGuard<L>>, LuaError>
        where R: Read
    {
        match LuaCodeFromReader(code).push_to_lua(lua) {
            Ok(pushed) => Ok(LuaFunction { variable: pushed }),
            Err((err, _)) => Err(err),
        }
    }

    /// Builds a new `LuaFunction` from a raw string.
    ///
    /// > **Note**: This is just a wrapper around `load_from_reader`. There is no advantage in
    /// > using `load` except that it is more convenient.
    // TODO: remove this function? it's only a thin wrapper and it's for a very niche situation
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

#[cfg(test)]
mod tests {
    use Lua;
    use LuaError;
    use LuaFunction;
    use LuaTable;

    #[test]
    fn basic() {
        let mut lua = Lua::new();
        let mut f = LuaFunction::load(&mut lua, "return 5;").unwrap();
        let val: i32 = f.call().unwrap();
        assert_eq!(val, 5);
    }

    #[test]
    fn syntax_error() {
        let mut lua = Lua::new();
        match LuaFunction::load(&mut lua, "azerazer") {
            Err(LuaError::SyntaxError(_)) => (),
            _ => panic!()
        };
    }

    #[test]
    fn execution_error() {
        let mut lua = Lua::new();
        let mut f = LuaFunction::load(&mut lua, "return a:hello()").unwrap();
        match f.call::<()>() {
            Err(LuaError::ExecutionError(_)) => (),
            _ => panic!()
        };
    }

    #[test]
    fn wrong_type() {
        let mut lua = Lua::new();
        let mut f = LuaFunction::load(&mut lua, "return 12").unwrap();
        match f.call::<LuaFunction<_>>() {
            Err(LuaError::WrongType) => (),
            _ => panic!()
        };
    }

    #[test]
    fn call_and_read_table() {
        let mut lua = Lua::new();
        let mut f = LuaFunction::load(&mut lua, "return {1, 2, 3};").unwrap();
        let mut val: LuaTable<_> = f.call().unwrap();
        assert_eq!(val.get::<u8, _>(2).unwrap(), 2);
    }

    #[test]
    fn lua_function_returns_function() {
        let mut lua = Lua::new();
        lua.execute::<()>("function foo() return 5 end").unwrap();
        let mut bar = LuaFunction::load(&mut lua, "return foo;").unwrap();
        let mut foo: LuaFunction<_> = bar.call().unwrap();
        let val: i32 = foo.call().unwrap();
        assert_eq!(val, 5);
    }

    // TODO: test for reading error
}
