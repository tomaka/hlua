use ffi;
use libc;

use std::error::Error;
use std::fmt;
use std::io::Cursor;
use std::io::Read;
use std::io::Error as IoError;
use std::mem;
use std::ptr;

use AsLua;
use AsMutLua;

use LuaContext;
use LuaRead;
use LuaError;
use Push;
use PushGuard;
use PushOne;
use Void;

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
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct LuaFunction<L> {
    variable: L,
}

unsafe impl<'lua, L> AsLua<'lua> for LuaFunction<L>
    where L: AsLua<'lua>
{
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.variable.as_lua()
    }
}

unsafe impl<'lua, L> AsMutLua<'lua> for LuaFunction<L>
    where L: AsMutLua<'lua>
{
    #[inline]
    fn as_mut_lua(&mut self) -> LuaContext {
        self.variable.as_mut_lua()
    }
}

impl<'lua, L> LuaFunction<L>
    where L: AsMutLua<'lua>
{
    /// Calls the function. Doesn't allow passing parameters.
    ///
    /// TODO: will eventually disappear and get replaced with `call_with_args`
    ///
    /// Returns an error if there is an error while executing the Lua code (eg. a function call
    /// returns an error), or if the requested return type doesn't match the actual return type.
    ///
    /// > **Note**: In order to pass parameters, see `call_with_args` instead.
    #[inline]
    pub fn call<'a, V>(&'a mut self) -> Result<V, LuaError>
        where V: LuaRead<PushGuard<&'a mut L>>
    {
        match self.call_with_args(()) {
            Ok(v) => Ok(v),
            Err(LuaFunctionCallError::LuaError(err)) => Err(err),
            Err(LuaFunctionCallError::PushError(_)) => unreachable!(),
        }
    }

    /// Calls the function with parameters.
    ///
    /// TODO: should be eventually be renamed to `call`
    ///
    /// You can either pass a single value by passing a single value, or multiple parameters by
    /// passing a tuple.
    /// If you pass a tuple, the first element of the tuple will be the first argument, the second
    /// element of the tuple the second argument, and so on.
    ///
    /// Returns an error if there is an error while executing the Lua code (eg. a function call
    /// returns an error), if the requested return type doesn't match the actual return type, or
    /// if we failed to push an argument.
    ///
    /// # Example
    ///
    /// ```
    /// let mut lua = hlua::Lua::new();
    /// lua.execute::<()>("function sub(a, b) return a - b end").unwrap();
    ///
    /// let mut foo: hlua::LuaFunction<_> = lua.get("sub").unwrap();
    /// let result: i32 = foo.call_with_args((18, 4)).unwrap();
    /// assert_eq!(result, 14);
    /// ```
    #[inline]
    pub fn call_with_args<'a, V, A, E>(&'a mut self, args: A) -> Result<V, LuaFunctionCallError<E>>
        where A: for<'r> Push<&'r mut LuaFunction<L>, Err = E>,
              V: LuaRead<PushGuard<&'a mut L>>
    {
        // calling pcall pops the parameters and pushes output
        let (pcall_return_value, pushed_value) = unsafe {
            // lua_pcall pops the function, so we have to make a copy of it
            ffi::lua_pushvalue(self.variable.as_mut_lua().0, -1);
            let num_pushed = match args.push_to_lua(self) {
                Ok(g) => g.forget_internal(),
                Err((err, _)) => return Err(LuaFunctionCallError::PushError(err)),
            };
            let pcall_return_value = ffi::lua_pcall(self.variable.as_mut_lua().0, num_pushed, 1, 0);     // TODO: num ret values

            let raw_lua = self.variable.as_lua();
            let guard = PushGuard {
                lua: &mut self.variable,
                size: 1,
                raw_lua: raw_lua,
            };

            (pcall_return_value, guard)
        };

        match pcall_return_value {
            0 => match LuaRead::lua_read(pushed_value) {
                Err(_) => Err(LuaFunctionCallError::LuaError(LuaError::WrongType)),
                Ok(x) => Ok(x),
            },
            ffi::LUA_ERRMEM => panic!("lua_pcall returned LUA_ERRMEM"),
            ffi::LUA_ERRRUN => {
                let error_msg: String = LuaRead::lua_read(pushed_value)
                    .ok()
                    .expect("can't find error message at the top of the Lua stack");
                Err(LuaFunctionCallError::LuaError(LuaError::ExecutionError(error_msg)))
            }
            _ => panic!("Unknown error code returned by lua_pcall: {}", pcall_return_value),
        }
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

/// Error that can happen when calling a `LuaFunction`.
// TODO: implement Error on this
#[derive(Debug)]
pub enum LuaFunctionCallError<E> {
    /// Error while executing the function.
    LuaError(LuaError),
    /// Error while pushing one of the parameters.
    PushError(E),
}

impl<E> fmt::Display for LuaFunctionCallError<E>
    where E: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LuaFunctionCallError::LuaError(ref lua_error) => write!(f, "Lua error: {}", lua_error),
            LuaFunctionCallError::PushError(ref err) => {
                write!(f, "Error while pushing arguments: {}", err)
            }
        }
    }
}

impl<E> From<LuaError> for LuaFunctionCallError<E> {
    #[inline]
    fn from(err: LuaError) -> LuaFunctionCallError<E> {
        LuaFunctionCallError::LuaError(err)
    }
}

impl From<LuaFunctionCallError<Void>> for LuaError {
    #[inline]
    fn from(err: LuaFunctionCallError<Void>) -> LuaError {
        match err {
            LuaFunctionCallError::LuaError(lua_error) => lua_error,
            LuaFunctionCallError::PushError(_) => unreachable!("Void cannot be instantiated"),
        }
    }
}

impl<E> Error for LuaFunctionCallError<E>
    where E: Error
{
    fn description(&self) -> &str {
        match *self {
            LuaFunctionCallError::LuaError(_) => "Lua error",
            LuaFunctionCallError::PushError(_) => "error while pushing arguments",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            LuaFunctionCallError::LuaError(ref lua_error) => Some(lua_error),
            LuaFunctionCallError::PushError(ref err) => Some(err),
        }
    }
}

impl Error for LuaFunctionCallError<Void> {
    fn description(&self) -> &str {
        match *self {
            LuaFunctionCallError::LuaError(_) => "Lua error",
            _ => unreachable!("Void cannot be instantiated"),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            LuaFunctionCallError::LuaError(ref lua_error) => Some(lua_error),
            _ => unreachable!("Void cannot be instantiated"),
        }
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
    use LuaFunctionCallError;
    use LuaTable;
    use Void;

    use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read};
    use std::error::Error;

    #[test]
    fn basic() {
        let mut lua = Lua::new();
        let mut f = LuaFunction::load(&mut lua, "return 5;").unwrap();
        let val: i32 = f.call().unwrap();
        assert_eq!(val, 5);
    }

    #[test]
    fn args() {
        let mut lua = Lua::new();
        lua.execute::<()>("function foo(a) return a * 5 end").unwrap();
        let val: i32 = lua.get::<LuaFunction<_>, _>("foo").unwrap().call_with_args(3).unwrap();
        assert_eq!(val, 15);
    }

    #[test]
    fn args_in_order() {
        let mut lua = Lua::new();
        lua.execute::<()>("function foo(a, b) return a - b end").unwrap();
        let val: i32 = lua.get::<LuaFunction<_>, _>("foo").unwrap().call_with_args((5, 3)).unwrap();
        assert_eq!(val, 2);
    }

    #[test]
    fn syntax_error() {
        let mut lua = Lua::new();
        match LuaFunction::load(&mut lua, "azerazer") {
            Err(LuaError::SyntaxError(_)) => (),
            _ => panic!(),
        };
    }

    #[test]
    fn execution_error() {
        let mut lua = Lua::new();
        let mut f = LuaFunction::load(&mut lua, "return a:hello()").unwrap();
        match f.call::<()>() {
            Err(LuaError::ExecutionError(_)) => (),
            _ => panic!(),
        };
    }

    #[test]
    fn wrong_type() {
        let mut lua = Lua::new();
        let mut f = LuaFunction::load(&mut lua, "return 12").unwrap();
        match f.call::<LuaFunction<_>>() {
            Err(LuaError::WrongType) => (),
            _ => panic!(),
        };
    }

    #[test]
    fn call_and_read_table() {
        let mut lua = Lua::new();
        let mut f = LuaFunction::load(&mut lua, "return {1, 2, 3};").unwrap();
        let mut val: LuaTable<_> = f.call().unwrap();
        assert_eq!(val.get::<u8, _, _>(2).unwrap(), 2);
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

    #[test]
    fn execute_from_reader_errors_if_cant_read() {
        struct Reader { };

        impl Read for Reader {
            fn read(&mut self, _: &mut [u8]) -> ::std::io::Result<usize> {
                use std::io::{Error, ErrorKind};
                Err(Error::new(ErrorKind::Other, "oh no!"))
            }
        }

        let mut lua = Lua::new();
        let reader = Reader { };
        let res: Result<(), _> = lua.execute_from_reader(reader);
        match res {
            Ok(_) => panic!("Reading succeded"),
            Err(LuaError::ReadError(e)) => { assert_eq!("oh no!", e.description()) },
            Err(_) => panic!("Unexpected error happened"),
        }
    }

    fn _assert_error() {
        // Compile-time trait checks.
        fn _assert<T: Error>(_: T) {}

        _assert(LuaFunctionCallError::LuaError::<Void>(LuaError::WrongType));
        _assert(LuaFunctionCallError::PushError(IoError::new(IoErrorKind::Other, "Test")));
    }
}
