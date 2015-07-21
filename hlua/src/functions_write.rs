use ffi;
use libc;

use AsLua;
use AsMutLua;
use LuaContext;
use LuaRead;
use Push;
use PushGuard;

use std::marker::PhantomData;
use std::fmt::Debug;
use std::mem;
use std::ptr;

macro_rules! impl_function {
    ($name:ident, $($p:ident),*) => (
        /// Wraps a type that implements `FnMut` so that it can be used by hlua.
        ///
        /// This is only needed because of a limitation in Rust's inferrence system.
        pub fn $name<Z, R $(, $p)*>(f: Z) -> Function<Z, ($($p,)*), R> where Z: FnMut($($p),*) -> R {
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
pub struct Function<F, P, R> {
    function: F,
    marker: PhantomData<(P, R)>,
}

/// Trait implemented on `Function` to mimic `FnMut`.
pub trait FunctionExt<P> {
    type Output;

    fn call_mut(&mut self, params: P) -> Self::Output;
}

macro_rules! impl_function_ext {
    () => (
        impl<Z, R> FunctionExt<()> for Function<Z, (), R> where Z: FnMut() -> R {
            type Output = R;

            #[allow(non_snake_case)]
            fn call_mut(&mut self, _: ()) -> Self::Output {
                (self.function)()
            }
        }

        impl<L, Z, R> Push<L> for Function<Z, (), R>
                where L: AsMutLua,
                      Z: FnMut() -> R,
                      R: for<'a> Push<&'a mut InsideCallback> + 'static
        {
            fn push_to_lua(self, mut lua: L) -> PushGuard<L> {
                unsafe {
                    // pushing the function pointer as a userdata
                    let lua_data = ffi::lua_newuserdata(lua.as_mut_lua().0,
                                                        mem::size_of::<Z>() as libc::size_t);
                    let lua_data: *mut Z = mem::transmute(lua_data);
                    ptr::write(lua_data, self.function);

                    // pushing wrapper as a closure
                    let wrapper: extern fn(*mut ffi::lua_State) -> libc::c_int = wrapper::<Self, _, R>;
                    ffi::lua_pushcclosure(lua.as_mut_lua().0, wrapper, 1);
                    PushGuard { lua: lua, size: 1 }
                }
            }
        }
    );

    ($($p:ident),+) => (
        impl<Z, R $(,$p)*> FunctionExt<($($p,)*)> for Function<Z, ($($p,)*), R> where Z: FnMut($($p),*) -> R {
            type Output = R;

            #[allow(non_snake_case)]
            fn call_mut(&mut self, params: ($($p,)*)) -> Self::Output {
                let ($($p,)*) = params;
                (self.function)($($p),*)
            }
        }

        impl<L, Z, R $(,$p: 'static)+> Push<L> for Function<Z, ($($p,)*), R>
                where L: AsMutLua,
                      Z: FnMut($($p),*) -> R,
                      ($($p,)*): for<'p> LuaRead<&'p mut InsideCallback>,
                      R: for<'a> Push<&'a mut InsideCallback> + 'static
        {
            fn push_to_lua(self, mut lua: L) -> PushGuard<L> {
                unsafe {
                    // pushing the function pointer as a userdata
                    let lua_data = ffi::lua_newuserdata(lua.as_mut_lua().0,
                                                        mem::size_of::<Z>() as libc::size_t);
                    let lua_data: *mut Z = mem::transmute(lua_data);
                    ptr::write(lua_data, self.function);

                    // pushing wrapper as a closure
                    let wrapper: extern fn(*mut ffi::lua_State) -> libc::c_int = wrapper::<Self, _, R>;
                    ffi::lua_pushcclosure(lua.as_mut_lua().0, wrapper, 1);
                    PushGuard { lua: lua, size: 1 }
                }
            }
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
pub struct InsideCallback {
    lua: LuaContext,
}

unsafe impl<'a> AsLua for &'a InsideCallback {
    fn as_lua(&self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'a> AsLua for &'a mut InsideCallback {
    fn as_lua(&self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'a> AsMutLua for &'a mut InsideCallback {
    fn as_mut_lua(&mut self) -> LuaContext {
        self.lua
    }
}

impl<'a, T, E> Push<&'a mut InsideCallback> for Result<T, E>
                where T: Push<&'a mut InsideCallback> +
                         for<'b> Push<&'b mut &'a mut InsideCallback>,
                      E: Debug
{
    fn push_to_lua(self, mut lua: &'a mut InsideCallback) -> PushGuard<&'a mut InsideCallback> {
        match self {
            Ok(val) => val.push_to_lua(lua),
            Err(val) => {
                let msg = format!("{:?}", val);
                msg.push_to_lua(&mut lua).forget();
                unsafe { ffi::lua_error(lua.as_mut_lua().0); }
                unreachable!();
            }
        }
    }
}

// this function is called when Lua wants to call one of our functions
extern fn wrapper<T, P, R>(lua: *mut ffi::lua_State) -> libc::c_int
                           where T: FunctionExt<P, Output=R>,
                                 P: for<'p> LuaRead<&'p mut InsideCallback> + 'static,
                                 R: for<'p> Push<&'p mut InsideCallback>
{
    // loading the object that we want to call from the Lua context
    let data_raw = unsafe { ffi::lua_touserdata(lua, ffi::lua_upvalueindex(1)) };
    let data: &mut T = unsafe { mem::transmute(data_raw) };

    // creating a temporary Lua context in order to pass it to push & read functions
    let mut tmp_lua = InsideCallback { lua: LuaContext(lua) };

    // trying to read the arguments
    let arguments_count = unsafe { ffi::lua_gettop(lua) } as i32;
    let args = match LuaRead::lua_read_at_position(&mut tmp_lua, -arguments_count as libc::c_int) {      // TODO: what if the user has the wrong params?
        Err(_) => {
            let err_msg = format!("wrong parameter types for callback function");
            err_msg.push_to_lua(&mut tmp_lua).forget();
            unsafe { ffi::lua_error(lua); }
            unreachable!()
        },
        Ok(a) => a
    };

    let ret_value = data.call_mut(args);

    // pushing back the result of the function on the stack
    let nb = ret_value.push_to_lua(&mut tmp_lua).forget();
    nb as libc::c_int
}
