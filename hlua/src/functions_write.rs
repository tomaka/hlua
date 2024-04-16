use ffi;
use libc;

use AnyLuaValue;
use AsLua;
use AsMutLua;
use LuaContext;
use LuaRead;
use Push;
use PushGuard;
use PushOne;
use Void;

use std::fmt::Display;
use std::marker::PhantomData;
use std::mem;
use std::ptr;

macro_rules! impl_function {
    ($name:ident, $($p:ident),*) => (
        /// Wraps a type that implements `FnMut` so that it can be used by hlua.
        ///
        /// This is needed because of a limitation in Rust's inferrence system. Even though in
        /// practice functions and closures always have a fixed number of parameters, the `FnMut`
        /// trait of Rust was designed so that it allows calling the same closure with a varying
        /// number of parameters. The consequence however is that there is no way of inferring
        /// with the trait alone many parameters a function or closure expects.
        #[inline]
        pub fn $name<Z, R $(, $p)*>(f: Z) -> Function<Z, ($($p,)*), R>
            where Z: FnMut($($p),*) -> R
        {
            Function {
                function: f,
                marker: PhantomData,
            }
        }
    )
}

impl_function!(function0,);
impl_function!(function1, A);
impl_function!(function2, A, B);
impl_function!(function3, A, B, C);
impl_function!(function4, A, B, C, D);
impl_function!(function5, A, B, C, D, E);
impl_function!(function6, A, B, C, D, E, F);
impl_function!(function7, A, B, C, D, E, F, G);
impl_function!(function8, A, B, C, D, E, F, G, H);
impl_function!(function9, A, B, C, D, E, F, G, H, I);
impl_function!(function10, A, B, C, D, E, F, G, H, I, J);

/// Opaque type containing a Rust function or closure.
///
/// In order to build an instance of this struct, you need to use one of the `functionN` functions.
/// There is one function for each possible number of parameter. For example if you have a function
/// with two parameters, you must use [`function2`](fn.function2.html).
/// Example:
///
/// ```
/// let f: hlua::Function<_, _, _> = hlua::function2(move |a: i32, b: i32| { });
/// ```
///
/// > **Note**: In practice you will never need to build an object of type `Function` as an
/// > intermediary step. Instead you will most likely always immediately push the function, like
/// > in the code below.
///
/// You can push a `Function` object like any other value:
///
/// ```
/// use hlua::Lua;
/// let mut lua = Lua::new();
///
/// lua.set("foo", hlua::function1(move |a: i32| -> i32 {
///     a * 5
/// }));
/// ```
///
/// The function can then be called from Lua:
///
/// ```
/// # use hlua::Lua;
/// # let mut lua = Lua::new();
/// # lua.set("foo", hlua::function1(move |a: i32| -> i32 { a * 5 }));
/// lua.execute::<()>("a = foo(12)").unwrap();
///
/// assert_eq!(lua.get::<i32, _>("a").unwrap(), 60);
/// ```
///
/// Remember that in Lua functions are regular variables, so you can do something like this
/// for example:
///
/// ```
/// # use hlua::Lua;
/// # let mut lua = Lua::new();
/// # lua.set("foo", hlua::function1(move |a: i32| -> i32 { a * 5 }));
/// lua.execute::<()>("bar = foo; a = bar(12)").unwrap();
/// ```
///
/// # Multiple return values
///
/// The Lua language supports functions that return multiple values at once.
///
/// In order to return multiple values from a Rust function, you can return a tuple. The elements
/// of the tuple will be returned in order.
///
/// ```
/// use hlua::Lua;
/// let mut lua = Lua::new();
///
/// lua.set("values", hlua::function0(move || -> (i32, i32, i32) {
///     (12, 24, 48)
/// }));
///
/// lua.execute::<()>("a, b, c = values()").unwrap();
///
/// assert_eq!(lua.get::<i32, _>("a").unwrap(), 12);
/// assert_eq!(lua.get::<i32, _>("b").unwrap(), 24);
/// assert_eq!(lua.get::<i32, _>("c").unwrap(), 48);
/// ```
///
/// # Using `Result`
///
/// If you want to return an error to the Lua script, you can use a `Result` that contains an
/// `Err`. The error will be returned to Lua as two values: A `nil` value and the error message.
///
/// The error type of the `Result` must implement the `Display` trait, and will be turned into a
/// Lua string.
///
/// ```
/// use hlua::Lua;
/// let mut lua = Lua::new();
/// lua.openlibs();
///
/// lua.set("err", hlua::function0(move || -> Result<i32, &'static str> {
///     Err("something wrong happened")
/// }));
///
/// lua.execute::<()>(r#"
///     res, err = err();
///     assert(res == nil);
///     assert(err == "something wrong happened");
/// "#).unwrap();
/// ```
///
/// This also allows easy use of `assert` to act like `.unwrap()` in Rust:
///
/// ```
/// use hlua::Lua;
/// let mut lua = Lua::new();
/// lua.openlibs();
///
/// lua.set("err", hlua::function0(move || -> Result<i32, &'static str> {
///     Err("something wrong happened")
/// }));
///
/// let ret = lua.execute::<()>("res = assert(err())");
/// assert!(ret.is_err());
/// ```
#[derive(Debug)]
pub struct Function<F, P, R> {
    function: F,
    marker: PhantomData<(P, R)>,
}

/// Trait implemented on `Function` to mimic `FnMut`.
///
/// We could in theory use the `FnMut` trait instead of this one, but it is still unstable.
pub trait FunctionExt<P> {
    type Output;

    fn call_mut(&mut self, params: P) -> Self::Output;
}

// Called when an object inside Lua is being dropped.
#[inline]
extern "C" fn closure_destructor_wrapper<T>(lua: *mut ffi::lua_State) -> libc::c_int {
    unsafe {
        let obj = ffi::lua_touserdata(lua, -1);
        ptr::drop_in_place((obj as *mut u8) as *mut T);
        0
    }
}

macro_rules! impl_function_ext {
    () => (
        impl<Z, R> FunctionExt<()> for Function<Z, (), R> where Z: FnMut() -> R {
            type Output = R;

            #[allow(non_snake_case)]
            #[inline]
            fn call_mut(&mut self, _: ()) -> Self::Output {
                (self.function)()
            }
        }

        impl<'lua, L, Z, R> Push<L> for Function<Z, (), R>
                where L: AsMutLua<'lua>,
                      Z: 'lua + FnMut() -> R,
                      R: for<'a> Push<&'a mut InsideCallback> + 'static
        {
            type Err = Void;      // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

            #[inline]
            fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
                unsafe {
                    // pushing the function pointer as a userdata
                    let lua_data = ffi::lua_newuserdata(lua.as_mut_lua().0,
                                                        mem::size_of::<Z>() as libc::size_t);
                    let lua_data: *mut Z = mem::transmute(lua_data);
                    ptr::write(lua_data, self.function);

                    let lua_raw = lua.as_mut_lua();

                    // Creating a metatable.
                    ffi::lua_newtable(lua.as_mut_lua().0);

                    // Index "__gc" in the metatable calls the object's destructor.

                    // TODO: Could use std::intrinsics::needs_drop to avoid that if not needed.
                    // After some discussion on IRC, it would be acceptable to add a reexport in libcore
                    // without going through the RFC process.
                    {
                        match "__gc".push_to_lua(&mut lua) {
                            Ok(p) => p.forget(),
                            Err(_) => unreachable!(),
                        };

                        ffi::lua_pushcfunction(lua.as_mut_lua().0, closure_destructor_wrapper::<Z>);
                        ffi::lua_settable(lua.as_mut_lua().0, -3);
                    }
                    ffi::lua_setmetatable(lua_raw.0, -2);

                    // pushing wrapper as a closure
                    let wrapper: extern fn(*mut ffi::lua_State) -> libc::c_int = wrapper::<Self, _, R>;
                    ffi::lua_pushcclosure(lua.as_mut_lua().0, wrapper, 1);
                    let raw_lua = lua.as_lua();
                    Ok(PushGuard { lua: lua, size: 1, raw_lua: raw_lua })
                }
            }
        }

        impl<'lua, L, Z, R> PushOne<L> for Function<Z, (), R>
                where L: AsMutLua<'lua>,
                      Z: 'lua + FnMut() -> R,
                      R: for<'a> Push<&'a mut InsideCallback> + 'static
        {
        }
    );

    ($($p:ident),+) => (
        impl<Z, R $(,$p)*> FunctionExt<($($p,)*)> for Function<Z, ($($p,)*), R> where Z: FnMut($($p),*) -> R {
            type Output = R;

            #[allow(non_snake_case)]
            #[inline]
            fn call_mut(&mut self, params: ($($p,)*)) -> Self::Output {
                let ($($p,)*) = params;
                (self.function)($($p),*)
            }
        }

        impl<'lua, L, Z, R $(,$p: 'static)+> Push<L> for Function<Z, ($($p,)*), R>
                where L: AsMutLua<'lua>,
                      Z: 'lua + FnMut($($p),*) -> R,
                      ($($p,)*): for<'p> LuaRead<&'p mut InsideCallback>,
                      R: for<'a> Push<&'a mut InsideCallback> + 'static
        {
            type Err = Void;      // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

            #[inline]
            fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
                unsafe {
                    // pushing the function pointer as a userdata
                    let lua_data = ffi::lua_newuserdata(lua.as_mut_lua().0,
                                                        mem::size_of::<Z>() as libc::size_t);
                    let lua_data: *mut Z = mem::transmute(lua_data);
                    ptr::write(lua_data, self.function);

                    let lua_raw = lua.as_mut_lua();

                    // Creating a metatable.
                    ffi::lua_newtable(lua.as_mut_lua().0);

                    // Index "__gc" in the metatable calls the object's destructor.

                    // TODO: Could use std::intrinsics::needs_drop to avoid that if not needed.
                    // After some discussion on IRC, it would be acceptable to add a reexport in libcore
                    // without going through the RFC process.
                    {
                        match "__gc".push_to_lua(&mut lua) {
                            Ok(p) => p.forget_internal(),
                            Err(_) => unreachable!(),
                        };

                        ffi::lua_pushcfunction(lua.as_mut_lua().0, closure_destructor_wrapper::<Z>);
                        ffi::lua_settable(lua.as_mut_lua().0, -3);
                    }
                    ffi::lua_setmetatable(lua_raw.0, -2);

                    // pushing wrapper as a closure
                    let wrapper: extern fn(*mut ffi::lua_State) -> libc::c_int = wrapper::<Self, _, R>;
                    ffi::lua_pushcclosure(lua.as_mut_lua().0, wrapper, 1);
                    let raw_lua = lua.as_lua();
                    Ok(PushGuard { lua: lua, size: 1, raw_lua: raw_lua })
                }
            }
        }

        impl<'lua, L, Z, R $(,$p: 'static)+> PushOne<L> for Function<Z, ($($p,)*), R>
                where L: AsMutLua<'lua>,
                      Z: 'lua + FnMut($($p),*) -> R,
                      ($($p,)*): for<'p> LuaRead<&'p mut InsideCallback>,
                      R: for<'a> Push<&'a mut InsideCallback> + 'static
        {
        }
    )
}

impl_function_ext!();
impl_function_ext!(A);
impl_function_ext!(A, B);
impl_function_ext!(A, B, C);
impl_function_ext!(A, B, C, D);
impl_function_ext!(A, B, C, D, E);
impl_function_ext!(A, B, C, D, E, F);
impl_function_ext!(A, B, C, D, E, F, G);
impl_function_ext!(A, B, C, D, E, F, G, H);
impl_function_ext!(A, B, C, D, E, F, G, H, I);
impl_function_ext!(A, B, C, D, E, F, G, H, I, J);

/// Opaque type that represents the Lua context when inside a callback.
///
/// Some types (like `Result`) can only be returned from a callback and not written inside a
/// Lua variable. This type is here to enforce this restriction.
#[derive(Debug)]
pub struct InsideCallback {
    lua: LuaContext,
}

unsafe impl<'a, 'lua> AsLua<'lua> for &'a InsideCallback {
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'a, 'lua> AsLua<'lua> for &'a mut InsideCallback {
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'a, 'lua> AsMutLua<'lua> for &'a mut InsideCallback {
    #[inline]
    fn as_mut_lua(&mut self) -> LuaContext {
        self.lua
    }
}

impl<'a, T, E, P> Push<&'a mut InsideCallback> for Result<T, E>
where
    T: Push<&'a mut InsideCallback, Err = P>
        + for<'b> Push<&'b mut &'a mut InsideCallback, Err = P>,
    E: Display,
{
    type Err = P;

    #[inline]
    fn push_to_lua(
        self,
        lua: &'a mut InsideCallback,
    ) -> Result<PushGuard<&'a mut InsideCallback>, (P, &'a mut InsideCallback)> {
        match self {
            Ok(val) => val.push_to_lua(lua),
            Err(val) => Ok((AnyLuaValue::LuaNil, format!("{}", val)).push_no_err(lua)),
        }
    }
}

impl<'a, T, E, P> PushOne<&'a mut InsideCallback> for Result<T, E>
where
    T: PushOne<&'a mut InsideCallback, Err = P>
        + for<'b> PushOne<&'b mut &'a mut InsideCallback, Err = P>,
    E: Display,
{
}

// this function is called when Lua wants to call one of our functions
#[inline]
extern "C" fn wrapper<T, P, R>(lua: *mut ffi::lua_State) -> libc::c_int
where
    T: FunctionExt<P, Output = R>,
    P: for<'p> LuaRead<&'p mut InsideCallback> + 'static,
    R: for<'p> Push<&'p mut InsideCallback>,
{
    // loading the object that we want to call from the Lua context
    let data_raw = unsafe { ffi::lua_touserdata(lua, ffi::lua_upvalueindex(1)) };
    let data: &mut T = unsafe { mem::transmute(data_raw) };

    // creating a temporary Lua context in order to pass it to push & read functions
    let mut tmp_lua = InsideCallback {
        lua: LuaContext(lua),
    };

    // trying to read the arguments
    let arguments_count = unsafe { ffi::lua_gettop(lua) } as i32;
    let args = match LuaRead::lua_read_at_position(&mut tmp_lua, -arguments_count as libc::c_int) {
        // TODO: what if the user has the wrong params?
        Err(_) => {
            let err_msg = format!("wrong parameter types for callback function");
            match err_msg.push_to_lua(&mut tmp_lua) {
                Ok(p) => p.forget_internal(),
                Err(_) => unreachable!(),
            };
            unsafe {
                ffi::lua_error(lua);
            }
            unreachable!()
        }
        Ok(a) => a,
    };

    let ret_value = data.call_mut(args);

    // pushing back the result of the function on the stack
    let nb = match ret_value.push_to_lua(&mut tmp_lua) {
        Ok(p) => p.forget_internal(),
        Err(_) => panic!(), // TODO: wrong
    };
    nb as libc::c_int
}

#[cfg(test)]
mod tests {
    use function0;
    use function1;
    use function2;
    use Lua;
    use LuaError;

    use std::sync::Arc;

    #[test]
    fn simple_function() {
        let mut lua = Lua::new();

        fn ret5() -> i32 {
            5
        };
        lua.set("ret5", function0(ret5));

        let val: i32 = lua.execute("return ret5()").unwrap();
        assert_eq!(val, 5);
    }

    #[test]
    fn one_argument() {
        let mut lua = Lua::new();

        fn plus_one(val: i32) -> i32 {
            val + 1
        };
        lua.set("plus_one", function1(plus_one));

        let val: i32 = lua.execute("return plus_one(3)").unwrap();
        assert_eq!(val, 4);
    }

    #[test]
    fn two_arguments() {
        let mut lua = Lua::new();

        fn add(val1: i32, val2: i32) -> i32 {
            val1 + val2
        };
        lua.set("add", function2(add));

        let val: i32 = lua.execute("return add(3, 7)").unwrap();
        assert_eq!(val, 10);
    }

    #[test]
    fn wrong_arguments_types() {
        let mut lua = Lua::new();

        fn add(val1: i32, val2: i32) -> i32 {
            val1 + val2
        };
        lua.set("add", function2(add));

        match lua.execute::<i32>("return add(3, \"hello\")") {
            Err(LuaError::ExecutionError(_)) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn return_result() {
        let mut lua = Lua::new();
        lua.openlibs();

        fn always_fails() -> Result<i32, &'static str> {
            Err("oops, problem")
        };
        lua.set("always_fails", function0(always_fails));

        match lua.execute::<()>(
            r#"
            local res, err = always_fails();
            assert(res == nil);
            assert(err == "oops, problem");
        "#,
        ) {
            Ok(()) => {}
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn closures() {
        let mut lua = Lua::new();

        lua.set("add", function2(|a: i32, b: i32| a + b));
        lua.set("sub", function2(|a: i32, b: i32| a - b));

        let val1: i32 = lua.execute("return add(3, 7)").unwrap();
        assert_eq!(val1, 10);

        let val2: i32 = lua.execute("return sub(5, 2)").unwrap();
        assert_eq!(val2, 3);
    }

    #[test]
    fn closures_lifetime() {
        fn t<F>(f: F)
        where
            F: Fn(i32, i32) -> i32,
        {
            let mut lua = Lua::new();

            lua.set("add", function2(f));

            let val1: i32 = lua.execute("return add(3, 7)").unwrap();
            assert_eq!(val1, 10);
        }

        t(|a, b| a + b);
    }

    #[test]
    fn closures_extern_access() {
        let mut a = 5;

        {
            let mut lua = Lua::new();

            lua.set("inc", function0(|| a += 1));
            for _ in 0..15 {
                lua.execute::<()>("inc()").unwrap();
            }
        }

        assert_eq!(a, 20)
    }

    #[test]
    fn closures_drop_env() {
        static mut DID_DESTRUCTOR_RUN: bool = false;

        #[derive(Debug)]
        struct Foo {};
        impl Drop for Foo {
            fn drop(&mut self) {
                unsafe {
                    DID_DESTRUCTOR_RUN = true;
                }
            }
        }
        {
            let foo = Arc::new(Foo {});

            {
                let mut lua = Lua::new();

                lua.set("print_foo", function0(move || println!("{:?}", foo)));
            }
        }
        assert_eq!(unsafe { DID_DESTRUCTOR_RUN }, true);
    }
}
