//! High-level zero-cost bindings for Lua
//!
//! Lua is an interpreted programming language. This crate allows you to execute Lua code.
//!
//! # General usage
//!
//! In order to execute Lua code you first need a *Lua context*, which is represented in this
//! library with [the `Lua` struct](struct.Lua.html). You can then call
//! [the `execute` method](struct.Lua.html#method.execute) on this object.
//!
//! For example:
//!
//! ```
//! use hlua::Lua;
//!
//! let mut lua = Lua::new();
//! lua.execute::<()>("a = 12 * 5").unwrap();
//! ```
//!
//! This example puts the value `60` in the global variable `a`. The values of all global variables
//! are stored within the `Lua` struct. If you execute multiple Lua scripts on the same context,
//! each script will have access to the same global variables that were modified by the previous
//! scripts.
//!
//! In order to do something actually useful with Lua, we will need to make Lua and Rust
//! communicate with each other. This can be done in four ways:
//!
//! - You can use methods on the `Lua` struct to read or write the values of global variables with
//!   the [`get`](struct.Lua.html#method.get) and [`set`](struct.Lua.html#method.set) methods. For
//!   example you can write to a global variable with a Lua script then read it from Rust, or you
//!   can write to a global variable from Rust then read it from a Lua script.
//!
//! - The Lua script that you execute with the [`execute`](struct.Lua.html#method.execute) method
//!   can return a value.
//!
//! - You can set the value of a global variable to a Rust functions or closures, which can then be
//!   invoked with a Lua script. See [the `Function` struct](struct.Function.html) for more
//!   information. For example if you set the value of the global variable `foo` to a Rust
//!   function, you can then call it from Lua with `foo()`.
//!
//! - Similarly you can set the value of a global variable to a Lua function, then call it from
//!   Rust. The function call can return a value.
//!
//! Which method(s) you use depends on which API you wish to expose to your Lua scripts.
//!
//! # Pushing and loading values
//!
//! The interface between Rust and Lua involves two things:
//!
//! - Sending values from Rust to Lua, which is known as *pushing* the value.
//! - Sending values from Lua to Rust, which is known as *loading* the value.
//!
//! Pushing (ie. sending from Rust to Lua) can be done with
//! [the `set` method](struct.Lua.html#method.set):
//!
//! ```
//! # use hlua::Lua;
//! # let mut lua = Lua::new();
//! lua.set("a", 50);
//! ```
//!
//! You can push values that implement [the `Push` trait](trait.Push.html) or
//! [the `PushOne` trait](trait.PushOne.html) depending on the situation:
//!
//! - Integers, floating point numbers and booleans.
//! - `String` and `&str`.
//! - Any Rust function or closure whose parameters and loadable and whose return type is pushable.
//!   See the documentation of [the `Function` struct](struct.Function.html) for more information.
//! - [The `AnyLuaValue` struct](struct.AnyLuaValue.html). This enumeration represents any possible
//!   value in Lua.
//! - The [`LuaCode`](struct.LuaCode.html) and
//!   [`LuaCodeFromReader`](struct.LuaCodeFromReader.html) structs. Since pushing these structs can
//!   result in an error, you need to use [`checked_set`](struct.Lua.html#method.checked_set)
//!   instead of `set`.
//! - `Vec`s and `HashMap`s whose content is pushable.
//! - As a special case, `Result` can be pushed only as the return type of a Rust function or
//!   closure. If they contain an error, the Rust function call is considered to have failed.
//! - As a special case, tuples can be pushed when they are the return type of a Rust function or
//!   closure. They implement `Push` but not `PushOne`.
//! - TODO: userdata
//!
//! Loading (ie. sending from Lua to Rust) can be done with
//! [the `get` method](struct.Lua.html#method.get):
//!
//! ```no_run
//! # use hlua::Lua;
//! # let mut lua = Lua::new();
//! let a: i32 = lua.get("a").unwrap();
//! ```
//!
//! You can load values that implement [the `LuaRead` trait](trait.LuaRead.html):
//!
//! - Integers, floating point numbers and booleans.
//! - `String` and [`StringInLua`](struct.StringInLua.html) (ie. the equivalent of `&str`). Loading
//!   the latter has no cost while loading a `String` performs an allocation.
//! - Any function (Lua or Rust), with [the `LuaFunction` struct](struct.LuaFunction.html). This
//!   can then be used to execute the function.
//! - [The `AnyLuaValue` struct](struct.AnyLuaValue.html). This enumeration represents any possible
//!   value in Lua.
//! - [The `LuaTable` struct](struct.LuaTable.html). This struct represents a table in Lua, where
//!   keys and values can be of different types. The table can then be iterated and individual
//!   elements can be loaded or modified.
//! - As a special case, tuples can be loaded when they are the return type of a Lua function or as
//!   the return type of [`execute`](struct.Lua.html#method.execute).
//! - TODO: userdata
//!

// Export the version of lua52_sys in use by this crate. This allows clients to perform low-level
// Lua operations without worrying about semver.
extern crate libc;
#[doc(hidden)]
pub extern crate lua52_sys as ffi;

use std::borrow::Borrow;
use std::convert::From;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt;
use std::io;
use std::io::Error as IoError;
use std::io::Read;
use std::marker::PhantomData;

pub use any::{AnyHashableLuaValue, AnyLuaString, AnyLuaValue};
pub use functions_write::{function0, function1, function2, function3, function4, function5};
pub use functions_write::{function10, function6, function7, function8, function9};
pub use functions_write::{Function, InsideCallback};
pub use lua_functions::LuaFunction;
pub use lua_functions::LuaFunctionCallError;
pub use lua_functions::{LuaCode, LuaCodeFromReader};
pub use lua_tables::LuaTable;
pub use lua_tables::LuaTableIterator;
pub use tuples::TuplePushError;
pub use userdata::UserdataOnStack;
pub use userdata::{push_userdata, read_userdata};
pub use values::StringInLua;

mod any;
mod functions_write;
mod lua_functions;
mod lua_tables;
mod macros;
mod rust_tables;
mod tuples;
mod userdata;
mod values;

/// Main object of the library.
///
/// The lifetime parameter corresponds to the lifetime of the content of the Lua context.
///
/// # About panic safety
///
/// This type isn't panic safe. This means that if a panic happens while you were using the `Lua`,
/// then it will probably stay in a corrupt state. Trying to use the `Lua` again will most likely
/// result in another panic but shouldn't result in unsafety.
#[derive(Debug)]
pub struct Lua<'lua> {
    lua: LuaContext,
    must_be_closed: bool,
    marker: PhantomData<&'lua ()>,
}

/// RAII guard for a value pushed on the stack.
///
/// You shouldn't have to manipulate this type directly unless you are fiddling with the
/// library's internals.
#[derive(Debug)]
pub struct PushGuard<L> {
    lua: L,
    size: i32,
    raw_lua: LuaContext,
}

impl<'lua, L> PushGuard<L>
where
    L: AsMutLua<'lua>,
{
    /// Creates a new `PushGuard` from this Lua context representing `size` items on the stack.
    /// When this `PushGuard` is destroyed, `size` items will be popped.
    ///
    /// This is unsafe because the Lua stack can be corrupted if this is misused.
    #[inline]
    pub unsafe fn new(mut lua: L, size: i32) -> Self {
        let raw_lua = lua.as_mut_lua();
        PushGuard { lua, size, raw_lua }
    }

    #[inline]
    fn assert_one_and_forget(self) -> i32 {
        assert_eq!(self.size, 1);
        self.forget_internal()
    }

    /// Returns the number of elements managed by this `PushGuard`.
    #[inline]
    pub fn size(&self) -> i32 {
        self.size
    }

    /// Prevents the value from being popped when the `PushGuard` is destroyed, and returns the
    /// number of elements on the Lua stack.
    ///
    /// This is unsafe because the Lua stack can be corrupted if this is misused.
    #[inline]
    pub unsafe fn forget(self) -> i32 {
        self.forget_internal()
    }

    /// Internal crate-only version of `forget`. It is generally assumed that code within this
    /// crate that calls this method knows what it is doing.
    #[inline]
    fn forget_internal(mut self) -> i32 {
        let size = self.size;
        self.size = 0;
        size
    }

    /// Destroys the guard, popping the value. Returns the inner part,
    /// which returns access when using by-value capture.
    #[inline]
    pub fn into_inner(mut self) -> L {
        unsafe {
            use std::{mem, ptr};

            let mut res;
            res = mem::MaybeUninit::uninit();
            ptr::copy_nonoverlapping(&self.lua, res.as_mut_ptr(), 1);
            if self.size != 0 {
                ffi::lua_pop(self.lua.as_mut_lua().0, self.size);
            }

            mem::forget(self);

            res.assume_init()
        }
    }
}

/// Trait for objects that have access to a Lua context. When using a context returned by a
/// `AsLua`, you are not allowed to modify the stack.
// TODO: the lifetime should be an associated lifetime instead
pub unsafe trait AsLua<'lua> {
    fn as_lua(&self) -> LuaContext;
}

/// Trait for objects that have access to a Lua context. You are allowed to modify the stack, but
/// it must be in the same state as it was when you started.
// TODO: the lifetime should be an associated lifetime instead
pub unsafe trait AsMutLua<'lua>: AsLua<'lua> {
    /// Returns the raw Lua context.
    fn as_mut_lua(&mut self) -> LuaContext;
}

/// Opaque type that contains the raw Lua context.
// TODO: probably no longer necessary
#[derive(Copy, Clone, Debug)]
pub struct LuaContext(*mut ffi::lua_State);

impl LuaContext {
    /// Return a pointer to the inner `lua_State` for this context. This is an escape hatch that
    /// lets the caller perform arbitrary operations against the FFI directly.
    ///
    /// Be careful: performing operations on this state might invalidate assumptions made in
    /// higher-level APIs. For example, pushing a value onto the Lua stack will cause `PushGuard`s
    /// in Rust code to be out of sync with the Lua stack.
    #[doc(hidden)]
    #[inline]
    pub fn state_ptr(&self) -> *mut ffi::lua_State {
        self.0
    }
}

unsafe impl Send for LuaContext {}

unsafe impl<'a, 'lua> AsLua<'lua> for Lua<'lua> {
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'lua> AsMutLua<'lua> for Lua<'lua> {
    #[inline]
    fn as_mut_lua(&mut self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'lua, L> AsLua<'lua> for PushGuard<L>
where
    L: AsMutLua<'lua>,
{
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.lua.as_lua()
    }
}

unsafe impl<'lua, L> AsMutLua<'lua> for PushGuard<L>
where
    L: AsMutLua<'lua>,
{
    #[inline]
    fn as_mut_lua(&mut self) -> LuaContext {
        self.lua.as_mut_lua()
    }
}

unsafe impl<'a, 'lua, L: ?Sized> AsLua<'lua> for &'a L
where
    L: AsLua<'lua>,
{
    #[inline]
    fn as_lua(&self) -> LuaContext {
        (**self).as_lua()
    }
}

unsafe impl<'a, 'lua, L: ?Sized> AsLua<'lua> for &'a mut L
where
    L: AsLua<'lua>,
{
    #[inline]
    fn as_lua(&self) -> LuaContext {
        (**self).as_lua()
    }
}

unsafe impl<'a, 'lua, L: ?Sized> AsMutLua<'lua> for &'a mut L
where
    L: AsMutLua<'lua>,
{
    #[inline]
    fn as_mut_lua(&mut self) -> LuaContext {
        (**self).as_mut_lua()
    }
}

/// Types that can be given to a Lua context, for example with `lua.set()` or as a return value
/// of a function.
pub trait Push<L> {
    /// Error that can happen when pushing a value.
    type Err;

    /// Pushes the value on the top of the stack.
    ///
    /// Must return a guard representing the elements that have been pushed.
    ///
    /// You can implement this for any type you want by redirecting to call to
    /// another implementation (for example `5.push_to_lua`) or by calling
    /// `userdata::push_userdata`.
    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (Self::Err, L)>;

    /// Same as `push_to_lua` but can only succeed and is only available if `Err` is `Void`.
    // TODO: when https://github.com/rust-lang/rust/issues/20041 is fixed, use `Self::Err == Void`
    #[inline]
    fn push_no_err<E>(self, lua: L) -> PushGuard<L>
    where
        Self: Sized,
        Self: Push<L, Err = E>,
        E: Into<Void>,
    {
        match self.push_to_lua(lua) {
            Ok(p) => p,
            Err(_) => unreachable!(),
        }
    }
}

/// Extension trait for `Push`. Guarantees that only one element will be pushed.
///
/// This should be implemented on most types that implement `Push`, except for tuples.
///
/// > **Note**: Implementing this trait on a type that pushes multiple elements will most likely
/// > result in panics.
// Note for the implementation: since this trait is not unsafe, it is mostly a hint. Functions can
// require this trait if they only accept one pushed element, but they must also add a runtime
// assertion to make sure that only one element was actually pushed.
pub trait PushOne<L>: Push<L> {}

/// Type that cannot be instantiated.
///
/// Will be replaced with `!` eventually (https://github.com/rust-lang/rust/issues/35121).
#[derive(Debug, Copy, Clone)]
pub enum Void {}

impl fmt::Display for Void {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unreachable!("Void cannot be instantiated")
    }
}

/// Types that can be obtained from a Lua context.
///
/// Most types that implement `Push` also implement `LuaRead`, but this is not always the case
/// (for example `&'static str` implements `Push` but not `LuaRead`).
pub trait LuaRead<L>: Sized {
    /// Reads the data from Lua.
    #[inline]
    fn lua_read(lua: L) -> Result<Self, L> {
        LuaRead::lua_read_at_position(lua, -1)
    }

    /// Reads the data from Lua at a given position.
    fn lua_read_at_position(lua: L, index: i32) -> Result<Self, L>;
}

/// Error that can happen when executing Lua code.
#[derive(Debug)]
pub enum LuaError {
    /// There was a syntax error when parsing the Lua code.
    SyntaxError(String),

    /// There was an error during execution of the Lua code
    /// (for example not enough parameters for a function call).
    ExecutionError(String),

    /// There was an IoError while reading the source code to execute.
    ReadError(IoError),

    /// The call to `execute` has requested the wrong type of data.
    WrongType,
}

impl fmt::Display for LuaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use LuaError::*;

        match *self {
            SyntaxError(ref s) => write!(f, "Syntax error: {}", s),
            ExecutionError(ref s) => write!(f, "Execution error: {}", s),
            ReadError(ref e) => write!(f, "Read error: {}", e),
            WrongType => write!(f, "Wrong type returned by Lua"),
        }
    }
}

impl Error for LuaError {
    fn description(&self) -> &str {
        use LuaError::*;

        match *self {
            SyntaxError(ref s) => &s,
            ExecutionError(ref s) => &s,
            ReadError(_) => "read error",
            WrongType => "wrong type returned by Lua",
        }
    }

    fn cause(&self) -> Option<&Error> {
        use LuaError::*;

        match *self {
            SyntaxError(_) => None,
            ExecutionError(_) => None,
            ReadError(ref e) => Some(e),
            WrongType => None,
        }
    }
}

impl From<io::Error> for LuaError {
    fn from(e: io::Error) -> Self {
        LuaError::ReadError(e)
    }
}

impl<'lua> Lua<'lua> {
    /// Builds a new empty Lua context.
    ///
    /// There are no global variables and the registry is totally empty. Even the functions from
    /// the standard library can't be used.
    ///
    /// If you want to use the Lua standard library in the scripts of this context, see
    /// [the openlibs method](#method.openlibs)
    ///
    /// # Example
    ///
    /// ```
    /// use hlua::Lua;
    /// let mut lua = Lua::new();
    /// ```
    ///
    /// # Panic
    ///
    /// The function panics if the underlying call to `lua_newstate` fails
    /// (which indicates lack of memory).
    #[inline]
    pub fn new() -> Lua<'lua> {
        let lua = unsafe { ffi::lua_newstate(alloc, std::ptr::null_mut()) };
        if lua.is_null() {
            panic!("lua_newstate failed");
        }

        // this alloc function is required to create a lua state.
        extern "C" fn alloc(
            _ud: *mut libc::c_void,
            ptr: *mut libc::c_void,
            _osize: libc::size_t,
            nsize: libc::size_t,
        ) -> *mut libc::c_void {
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
            let err = unsafe { CStr::from_ptr(err) };
            let err = String::from_utf8(err.to_bytes().to_vec()).unwrap();
            panic!("PANIC: unprotected error in call to Lua API ({})\n", err);
        }

        unsafe { ffi::lua_atpanic(lua, panic) };

        Lua {
            lua: LuaContext(lua),
            must_be_closed: true,
            marker: PhantomData,
        }
    }

    /// Takes an existing `lua_State` and build a Lua object from it.
    ///
    /// If `close_at_the_end` is true, `lua_close` will be called on the `lua_State` in the
    /// destructor.
    #[inline]
    pub unsafe fn from_existing_state<T>(lua: *mut T, close_at_the_end: bool) -> Lua<'lua> {
        Lua {
            lua: std::mem::transmute(lua),
            must_be_closed: close_at_the_end,
            marker: PhantomData,
        }
    }

    /// Opens all standard Lua libraries.
    ///
    /// See the reference for the standard library here:
    /// https://www.lua.org/manual/5.2/manual.html#6
    ///
    /// This is done by calling `luaL_openlibs`.
    ///
    /// # Example
    ///
    /// ```
    /// use hlua::Lua;
    /// let mut lua = Lua::new();
    /// lua.openlibs();
    /// ```
    #[inline]
    pub fn openlibs(&mut self) {
        unsafe { ffi::luaL_openlibs(self.lua.0) }
    }

    // Helper to import library
    fn open_helper(&mut self, modname: &str, func: ffi::lua_CFunction2) {
        unsafe {
            // No need to handle error since input is hardcoded
            // See https://doc.rust-lang.org/std/ffi/struct.CString.html#method.new
            let name = CString::new(modname).unwrap();
            ffi::luaL_requiref(
                self.lua.0,
                name.as_ptr(),
                func,
                1
            );
            ffi::lua_pop(self.lua.0, 1);
        }
    }

    /// Opens base library.
    ///
    /// https://www.lua.org/manual/5.2/manual.html#pdf-luaopen_base
    #[inline]
    pub fn open_base(&mut self) {
        self.open_helper("_G", ffi::luaopen_base);
    }

    /// Opens bit32 library.
    ///
    /// https://www.lua.org/manual/5.2/manual.html#pdf-luaopen_bit32
    #[inline]
    pub fn open_bit32(&mut self) {
        self.open_helper("bit32", ffi::luaopen_bit32);
    }

    /// Opens coroutine library.
    ///
    /// https://www.lua.org/manual/5.2/manual.html#pdf-luaopen_coroutine
    #[inline]
    pub fn open_coroutine(&mut self) {
        self.open_helper("coroutine", ffi::luaopen_coroutine);
    }

    /// Opens debug library.
    ///
    /// https://www.lua.org/manual/5.2/manual.html#pdf-luaopen_debug
    #[inline]
    pub fn open_debug(&mut self) {
        self.open_helper("debug", ffi::luaopen_debug);
    }

    /// Opens io library.
    ///
    /// https://www.lua.org/manual/5.2/manual.html#pdf-luaopen_io
    #[inline]
    pub fn open_io(&mut self) {
        self.open_helper("io", ffi::luaopen_io);
    }

    /// Opens math library.
    ///
    /// https://www.lua.org/manual/5.2/manual.html#pdf-luaopen_math
    #[inline]
    pub fn open_math(&mut self) {
        self.open_helper("math", ffi::luaopen_math);
    }

    /// Opens os library.
    ///
    /// https://www.lua.org/manual/5.2/manual.html#pdf-luaopen_os
    #[inline]
    pub fn open_os(&mut self) {
        self.open_helper("os", ffi::luaopen_os);
    }

    /// Opens package library.
    ///
    /// https://www.lua.org/manual/5.2/manual.html#pdf-luaopen_package
    #[inline]
    pub fn open_package(&mut self) {
        self.open_helper("package", ffi::luaopen_package);
    }

    /// Opens string library.
    ///
    /// https://www.lua.org/manual/5.2/manual.html#pdf-luaopen_string
    #[inline]
    pub fn open_string(&mut self) {
        self.open_helper("string", ffi::luaopen_string);
    }

    /// Opens table library.
    ///
    /// https://www.lua.org/manual/5.2/manual.html#pdf-luaopen_table
    #[inline]
    pub fn open_table(&mut self) {
        self.open_helper("table", ffi::luaopen_table);
    }

    /// Executes some Lua code in the context.
    ///
    /// The code will have access to all the global variables you set with methods such as `set`.
    /// Every time you execute some code in the context, the code can modify these global variables.
    ///
    /// The template parameter of this function is the return type of the expression that is being
    /// evaluated.
    /// In order to avoid compilation error, you should call this function either by doing
    /// `lua.execute::<T>(...)` or `let result: T = lua.execute(...);` where `T` is the type of
    /// the expression.
    /// The function will return an error if the actual return type of the expression doesn't
    /// match the template parameter.
    ///
    /// The return type must implement the `LuaRead` trait. See
    /// [the documentation at the crate root](index.html#pushing-and-loading-values) for more
    /// information.
    ///
    /// # Examples
    ///
    /// Without a return value:
    ///
    /// ```
    /// use hlua::Lua;
    /// let mut lua = Lua::new();
    /// lua.execute::<()>("function multiply_by_two(a) return a * 2 end").unwrap();
    /// lua.execute::<()>("twelve = multiply_by_two(6)").unwrap();
    /// ```
    ///
    /// With a return value:
    ///
    /// ```
    /// use hlua::Lua;
    /// let mut lua = Lua::new();
    ///
    /// let twelve: i32 = lua.execute("return 3 * 4;").unwrap();
    /// let sixty = lua.execute::<i32>("return 6 * 10;").unwrap();
    /// ```
    #[inline]
    pub fn execute<'a, T>(&'a mut self, code: &str) -> Result<T, LuaError>
    where
        T: for<'g> LuaRead<PushGuard<&'g mut PushGuard<&'a mut Lua<'lua>>>>,
    {
        let mut f = try!(lua_functions::LuaFunction::load(self, code));
        f.call()
    }

    /// Executes some Lua code on the context.
    ///
    /// This does the same thing as [the `execute` method](#method.execute), but the code to
    /// execute is loaded from an object that implements `Read`.
    ///
    /// Use this method when you potentially have a large amount of code (for example if you read
    /// the code from a file) in order to avoid having to put everything in memory first before
    /// passing it to the Lua interpreter.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use hlua::Lua;
    ///
    /// let mut lua = Lua::new();
    /// let script = File::open("script.lua").unwrap();
    /// lua.execute_from_reader::<(), _>(script).unwrap();
    /// ```
    #[inline]
    pub fn execute_from_reader<'a, T, R>(&'a mut self, code: R) -> Result<T, LuaError>
    where
        T: for<'g> LuaRead<PushGuard<&'g mut PushGuard<&'a mut Lua<'lua>>>>,
        R: Read,
    {
        let mut f = try!(lua_functions::LuaFunction::load_from_reader(self, code));
        f.call()
    }

    /// Reads the value of a global variable.
    ///
    /// Returns `None` if the variable doesn't exist or has the wrong type.
    ///
    /// The type must implement the `LuaRead` trait. See
    /// [the documentation at the crate root](index.html#pushing-and-loading-values) for more
    /// information.
    ///
    /// # Example
    ///
    /// ```
    /// use hlua::Lua;
    /// let mut lua = Lua::new();
    /// lua.execute::<()>("a = 5").unwrap();
    /// let a: i32 = lua.get("a").unwrap();
    /// assert_eq!(a, 5);
    /// ```
    #[inline]
    pub fn get<'l, V, I>(&'l mut self, index: I) -> Option<V>
    where
        I: Borrow<str>,
        V: LuaRead<PushGuard<&'l mut Lua<'lua>>>,
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe {
            ffi::lua_getglobal(self.lua.0, index.as_ptr());
        }
        if unsafe { ffi::lua_isnil(self.as_lua().0, -1) } {
            let raw_lua = self.as_lua();
            let _guard = PushGuard {
                lua: self,
                size: 1,
                raw_lua: raw_lua,
            };
            return None;
        }
        let raw_lua = self.as_lua();
        let guard = PushGuard {
            lua: self,
            size: 1,
            raw_lua: raw_lua,
        };
        LuaRead::lua_read(guard).ok()
    }

    /// Reads the value of a global, capturing the context by value.
    #[inline]
    pub fn into_get<V, I>(self, index: I) -> Result<V, PushGuard<Self>>
    where
        I: Borrow<str>,
        V: LuaRead<PushGuard<Lua<'lua>>>,
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe {
            ffi::lua_getglobal(self.lua.0, index.as_ptr());
        }
        let is_nil = unsafe { ffi::lua_isnil(self.as_lua().0, -1) };
        let raw_lua = self.as_lua();
        let guard = PushGuard {
            lua: self,
            size: 1,
            raw_lua: raw_lua,
        };
        if is_nil {
            Err(guard)
        } else {
            LuaRead::lua_read(guard)
        }
    }

    /// Modifies the value of a global variable.
    ///
    /// If you want to write an array, you are encouraged to use
    /// [the `empty_array` method](#method.empty_array) instead.
    ///
    /// The type must implement the `PushOne` trait. See
    /// [the documentation at the crate root](index.html#pushing-and-loading-values) for more
    /// information.
    ///
    /// # Example
    ///
    /// ```
    /// use hlua::Lua;
    /// let mut lua = Lua::new();
    ///
    /// lua.set("a", 12);
    /// let six: i32 = lua.execute("return a / 2;").unwrap();
    /// assert_eq!(six, 6);
    /// ```
    #[inline]
    pub fn set<I, V, E>(&mut self, index: I, value: V)
    where
        I: Borrow<str>,
        for<'a> V: PushOne<&'a mut Lua<'lua>, Err = E>,
        E: Into<Void>,
    {
        match self.checked_set(index, value) {
            Ok(_) => (),
            Err(_) => unreachable!(),
        }
    }

    /// Modifies the value of a global variable.
    // TODO: docs
    #[inline]
    pub fn checked_set<I, V, E>(&mut self, index: I, value: V) -> Result<(), E>
    where
        I: Borrow<str>,
        for<'a> V: PushOne<&'a mut Lua<'lua>, Err = E>,
    {
        unsafe {
            // TODO: can be simplified
            let mut me = self;
            ffi::lua_pushglobaltable(me.lua.0);
            match index.borrow().push_to_lua(&mut me) {
                Ok(pushed) => {
                    debug_assert_eq!(pushed.size, 1);
                    pushed.forget()
                }
                Err(_) => unreachable!(),
            };
            match value.push_to_lua(&mut me) {
                Ok(pushed) => {
                    assert_eq!(pushed.size, 1);
                    pushed.forget()
                }
                Err((err, lua)) => {
                    ffi::lua_pop(lua.lua.0, 2);
                    return Err(err);
                }
            };
            ffi::lua_settable(me.lua.0, -3);
            ffi::lua_pop(me.lua.0, 1);
            Ok(())
        }
    }

    /// Sets the value of a global variable to an empty array, then loads it.
    ///
    /// This is the function you should use if you want to set the value of a global variable to
    /// an array. After calling it, you will obtain a `LuaTable` object which you can then fill
    /// with the elements of the array.
    ///
    /// # Example
    ///
    /// ```
    /// use hlua::Lua;
    /// let mut lua = Lua::new();
    /// lua.openlibs();     // Necessary for `ipairs`.
    ///
    /// {
    ///     let mut array = lua.empty_array("my_values");
    ///     array.set(1, 10);       // Don't forget that Lua arrays are indexed from 1.
    ///     array.set(2, 15);
    ///     array.set(3, 20);
    /// }
    ///
    /// let sum: i32 = lua.execute(r#"
    ///     local sum = 0
    ///     for i, val in ipairs(my_values) do
    ///         sum = sum + val
    ///     end
    ///     return sum
    /// "#).unwrap();
    ///
    /// assert_eq!(sum, 45);
    /// ```
    #[inline]
    pub fn empty_array<'a, I>(&'a mut self, index: I) -> LuaTable<PushGuard<&'a mut Lua<'lua>>>
    where
        I: Borrow<str>,
    {
        unsafe {
            let mut me = self;
            ffi::lua_pushglobaltable(me.lua.0);
            match index.borrow().push_to_lua(&mut me) {
                Ok(pushed) => pushed.forget(),
                Err(_) => unreachable!(),
            };
            ffi::lua_newtable(me.lua.0);
            ffi::lua_settable(me.lua.0, -3);
            ffi::lua_pop(me.lua.0, 1);

            // TODO: cleaner implementation
            me.get(index).unwrap()
        }
    }

    /// Loads the array containing the global variables.
    ///
    /// In lua, the global variables accessible from the lua code are all part of a table which
    /// you can load here.
    ///
    /// # Examples
    ///
    /// The function can be used to write global variables, just like `set`.
    ///
    /// ```
    /// use hlua::Lua;
    /// let mut lua = Lua::new();
    /// lua.globals_table().set("a", 5);
    /// assert_eq!(lua.get::<i32, _>("a"), Some(5));
    /// ```
    ///
    /// A more useful feature for this function is that it allows you to set the metatable of the
    /// global variables. See TODO for more info.
    ///
    /// ```
    /// use hlua::Lua;
    /// use hlua::AnyLuaValue;
    ///
    /// let mut lua = Lua::new();
    /// {
    ///     let mut metatable = lua.globals_table().get_or_create_metatable();
    ///     metatable.set("__index", hlua::function2(|_: AnyLuaValue, var: String| -> AnyLuaValue {
    ///         println!("The user tried to access the variable {:?}", var);
    ///         AnyLuaValue::LuaNumber(48.0)
    ///     }));
    /// }
    ///
    /// let b: i32 = lua.execute("return b * 2;").unwrap();
    /// // -> The user tried to access the variable "b"
    ///
    /// assert_eq!(b, 96);
    /// ```
    #[inline]
    pub fn globals_table<'a>(&'a mut self) -> LuaTable<PushGuard<&'a mut Lua<'lua>>> {
        unsafe {
            ffi::lua_pushglobaltable(self.lua.0);
        }
        let raw_lua = self.as_lua();
        let guard = PushGuard {
            lua: self,
            size: 1,
            raw_lua: raw_lua,
        };
        LuaRead::lua_read(guard).ok().unwrap()
    }
}

impl<'lua> Drop for Lua<'lua> {
    #[inline]
    fn drop(&mut self) {
        if self.must_be_closed {
            unsafe { ffi::lua_close(self.lua.0) }
        }
    }
}

impl<L> Drop for PushGuard<L> {
    #[inline]
    fn drop(&mut self) {
        if self.size != 0 {
            unsafe {
                ffi::lua_pop(self.raw_lua.0, self.size);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use Lua;
    use LuaError;
    use LuaTable;

    #[test]
    fn open_base_opens_base_library() {
        let mut lua = Lua::new();
        match lua.execute::<()>("return assert(true)") {
            Err(LuaError::ExecutionError(_)) => {}
            Err(_) => panic!("Wrong error"),
            Ok(_) => panic!("Unexpected success"),
        }
        lua.open_base();
        let result: bool = lua.execute("return assert(true)").unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn opening_all_libraries_doesnt_panic() {
        let mut lua = Lua::new();
        lua.open_base();
        lua.open_bit32();
        lua.open_coroutine();
        lua.open_debug();
        lua.open_io();
        lua.open_math();
        lua.open_os();
        lua.open_package();
        lua.open_string();
        lua.open_table();
    }

    #[test]
    fn opening_base() {
        let mut lua = Lua::new();
        lua.open_base();
        let _: LuaTable<_> = lua.get("_G").unwrap();
    }

    #[test]
    fn opening_bit32() {
        let mut lua = Lua::new();
        lua.open_bit32();
        let _: LuaTable<_> = lua.get("bit32").unwrap();
    }

    #[test]
    fn opening_coroutine() {
        let mut lua = Lua::new();
        lua.open_coroutine();
        let _: LuaTable<_> = lua.get("coroutine").unwrap();
    }

    #[test]
    fn opening_debug() {
        let mut lua = Lua::new();
        lua.open_debug();
        let _: LuaTable<_> = lua.get("debug").unwrap();
    }

    #[test]
    fn opening_io() {
        let mut lua = Lua::new();
        lua.open_io();
        let _: LuaTable<_> = lua.get("io").unwrap();
    }

    #[test]
    fn opening_math() {
        let mut lua = Lua::new();
        lua.open_math();
        let _: LuaTable<_> = lua.get("math").unwrap();
    }

    #[test]
    fn opening_os() {
        let mut lua = Lua::new();
        lua.open_os();
        let _: LuaTable<_> = lua.get("os").unwrap();
    }

    #[test]
    fn opening_package() {
        let mut lua = Lua::new();
        lua.open_package();
        let _: LuaTable<_> = lua.get("package").unwrap();
    }

    #[test]
    fn opening_string() {
        let mut lua = Lua::new();
        lua.open_string();
        let _: LuaTable<_> = lua.get("string").unwrap();
    }

    #[test]
    fn opening_table() {
        let mut lua = Lua::new();
        lua.open_table();
        let _: LuaTable<_> = lua.get("table").unwrap();
    }
}
