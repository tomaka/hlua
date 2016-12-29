use ffi;
use libc;

use AsLua;
use AsMutLua;
use LuaContext;
use LuaRead;
use Push;
use PushGuard;
use PushOne;
use Void;

use std::marker::PhantomData;
use std::fmt::Debug;
use std::mem;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

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
/// `Err`. The error will be a regular Lua error and will be propagated either to the `execute`
/// function, or can be caught with the `pcall` Lua function.
///
/// The error type of the `Result` must implement the `Debug` trait, and will be turned into a
/// Lua string.
///
/// ```
/// use hlua::Lua;
/// let mut lua = Lua::new();
/// 
/// lua.set("err", hlua::function0(move || -> Result<i32, &'static str> {
///     Err("something wrong happened")
/// }));
///
/// let ret = lua.execute::<()>("a = err()");
/// assert!(ret.is_err());
/// ```
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

// TODO: with one argument we should require LuaRead<&'a mut InsideCallback<'lua>> and
//       not LuaRead<&'a InsideCallback<'lua>>

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
                      R: for<'a> Push<&'a mut InsideCallback<'lua>> + 'static
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
                      R: for<'a> Push<&'a mut InsideCallback<'lua>> + 'static
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

        impl<'lua, L, Z, R $(,$p)+> Push<L> for Function<Z, ($($p,)*), R>
                where L: AsMutLua<'lua>,
                      Z: 'lua + FnMut($($p),*) -> R,
                      ($($p,)*): LuaRead<Arc<InsideCallback<'lua>>>,
                      R: for<'a> Push<&'a mut InsideCallback<'lua>> + 'static
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

                    // pushing wrapper as a closure
                    let wrapper: extern fn(*mut ffi::lua_State) -> libc::c_int = wrapper::<Self, _, R>;
                    ffi::lua_pushcclosure(lua.as_mut_lua().0, wrapper, 1);
                    let raw_lua = lua.as_lua();
                    Ok(PushGuard { lua: lua, size: 1, raw_lua: raw_lua })
                }
            }
        }

        impl<'lua, L, Z, R $(,$p)+> PushOne<L> for Function<Z, ($($p,)*), R>
                where L: AsMutLua<'lua>,
                      Z: 'lua + FnMut($($p),*) -> R,
                      ($($p,)*): LuaRead<Arc<InsideCallback<'lua>>>,
                      R: for<'a> Push<&'a mut InsideCallback<'lua>> + 'static
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

pub struct LuaMutex<T> {
    lua: Arc<InsideCallback<'static>>,      // TODO: I couldn't make it work for non-'static
    index: i32,
    marker: PhantomData<T>,
}

impl<T> LuaRead<Arc<InsideCallback<'static>>> for LuaMutex<T>
    where T: LuaRead<InsideCallbackLockGuard<'static>>
{
    #[inline]
    fn lua_read_at_position(lua: Arc<InsideCallback<'static>>, index: i32)
                            -> Result<Self, Arc<InsideCallback<'static>>>
    {
        Ok(LuaMutex {
            lua: lua,
            index: index,
            marker: PhantomData,
        })
    }
}

impl<T> LuaMutex<T>
    where T: LuaRead<InsideCallbackLockGuard<'static>>
{
    #[inline]
    pub fn lock(&self) -> Option<T> {
        let lock = InsideCallback::lock(self.lua.clone());

        match T::lua_read_at_position(lock, self.index) {
            Ok(v) => Some(v),
            Err(_) => None
        }
    }
}

/// Opaque type that represents the Lua context when inside a callback.
///
/// Some types (like `Result`) can only be returned from a callback and not written inside a
/// Lua variable. This type is here to enforce this restriction.
pub struct InsideCallback<'lua> {
    lua: LuaContext,
    mutex: AtomicBool,
    marker: PhantomData<&'lua ()>,
}

impl<'lua> InsideCallback<'lua> {
    #[inline]
    pub fn lock(me: Arc<InsideCallback<'lua>>) -> InsideCallbackLockGuard<'lua> {
        let old = me.mutex.swap(true, Ordering::SeqCst);
        if old {
            panic!("Can't lock the InsideCallback twice simultaneously");
        }

        InsideCallbackLockGuard {
            lua: me
        }
    }
}

unsafe impl<'a, 'lua> AsLua<'lua> for Arc<InsideCallback<'lua>> {
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'a, 'lua> AsLua<'lua> for &'a mut InsideCallback<'lua> {
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'a, 'lua> AsMutLua<'lua> for &'a mut InsideCallback<'lua> {
    #[inline]
    fn as_mut_lua(&mut self) -> LuaContext {
        self.lua
    }
}

pub struct InsideCallbackLockGuard<'lua> {
    lua: Arc<InsideCallback<'lua>>
}

unsafe impl<'lua> AsLua<'lua> for InsideCallbackLockGuard<'lua> {
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.lua.lua
    }
}

unsafe impl<'lua> AsMutLua<'lua> for InsideCallbackLockGuard<'lua> {
    #[inline]
    fn as_mut_lua(&mut self) -> LuaContext {
        self.lua.lua
    }
}

impl<'lua> Drop for InsideCallbackLockGuard<'lua> {
    #[inline]
    fn drop(&mut self) {
        let old = self.lua.mutex.swap(false, Ordering::SeqCst);
        debug_assert_eq!(old, true);
    }
}

impl<'a, 'lua, T, E, P> Push<&'a mut InsideCallback<'lua>> for Result<T, E>
    where T: Push<&'a mut InsideCallback<'lua>, Err = P> + for<'b> Push<&'b mut &'a mut InsideCallback<'lua>, Err = P>,
          E: Debug
{
    type Err = P;

    #[inline]
    fn push_to_lua(self, mut lua: &'a mut InsideCallback<'lua>) -> Result<PushGuard<&'a mut InsideCallback<'lua>>, (P, &'a mut InsideCallback<'lua>)> {
        unsafe {
            match self {
                Ok(val) => val.push_to_lua(lua),
                Err(val) => {
                    let msg = format!("{:?}", val);
                    match msg.push_to_lua(&mut lua) {
                        Ok(pushed) => pushed.forget(),
                        Err(_) => unreachable!()
                    };
                    ffi::lua_error(lua.as_mut_lua().0);
                    unreachable!();
                }
            }
        }
    }
}

impl<'a, 'lua, T, E, P> PushOne<&'a mut InsideCallback<'lua>> for Result<T, E>
    where T: PushOne<&'a mut InsideCallback<'lua>, Err = P> + for<'b> PushOne<&'b mut &'a mut InsideCallback<'lua>, Err = P>,
          E: Debug
{
}

// this function is called when Lua wants to call one of our functions
#[inline]
extern "C" fn wrapper<'lua, T, P, R>(lua: *mut ffi::lua_State) -> libc::c_int
    where T: FunctionExt<P, Output = R>,
          P: LuaRead<Arc<InsideCallback<'lua>>>,
          R: for<'p> Push<&'p mut InsideCallback<'lua>>
{
    // loading the object that we want to call from the Lua context
    let data_raw = unsafe { ffi::lua_touserdata(lua, ffi::lua_upvalueindex(1)) };
    let data: &mut T = unsafe { mem::transmute(data_raw) };

    // creating a temporary Lua context in order to pass it to push & read functions
    let mut tmp_lua = Arc::new(InsideCallback {
        lua: LuaContext(lua),
        mutex: AtomicBool::new(false),
        marker: PhantomData,
    });

    // trying to read the arguments
    let arguments_count = unsafe { ffi::lua_gettop(lua) } as i32;
    let args = match LuaRead::lua_read_at_position(tmp_lua.clone(), -arguments_count as libc::c_int) {      // TODO: what if the user has the wrong params?
        Err(_) => {
            let err_msg = format!("wrong parameter types for callback function");
            match err_msg.push_to_lua(Arc::get_mut(&mut tmp_lua).unwrap()) {
                Ok(p) => p.forget(),
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
    let nb = match ret_value.push_to_lua(Arc::get_mut(&mut tmp_lua).unwrap()) {
        Ok(p) => p.forget(),
        Err(_) => panic!(),      // TODO: wrong
    };
    nb as libc::c_int
}

#[cfg(test)]
mod tests {
    use Lua;
    use LuaError;
    use LuaFunction;
    use LuaMutex;
    use function0;
    use function1;
    use function2;

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

        fn always_fails() -> Result<i32, &'static str> {
            Err("oops, problem")
        };
        lua.set("always_fails", function0(always_fails));

        match lua.execute::<()>("always_fails()") {
            Err(LuaError::ExecutionError(_)) => (),
            _ => panic!(),
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
            where F: Fn(i32, i32) -> i32
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
    fn lua_mutex_basic() {
        let mut lua = Lua::new();

        lua.set("foo", function1(|a: LuaMutex<LuaFunction<_>>|  {
            let mut a = a.lock().unwrap();
            assert_eq!(a.call::<i32>().unwrap(), 5);
        }));

        lua.execute::<()>("function bar() return 5 end").unwrap();
        lua.execute::<()>("foo(bar)").unwrap();
    }

    /* TODO: make compile
    #[test]
    fn lua_mutex_two() {
        let mut lua = Lua::new();

        lua.set("foo", function2(|a: LuaMutex<LuaFunction<_>>, b: LuaMutex<LuaFunction<_>>|  {
            {
                let a = a.lock().unwrap();
                assert_eq!(a.call::<i32>().unwrap(), 5);
            }

            {
                let b = b.lock().unwrap();
                assert_eq!(b.call::<i32>().unwrap(), 5);
            }
        }));

        lua.execute::<()>("function bar() return 5 end").unwrap();
        lua.execute::<()>("foo(bar)").unwrap();
    }*/
}
