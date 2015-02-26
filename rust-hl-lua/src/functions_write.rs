use super::ffi;
use {AsLua, Push, CopyRead};
use std::kinds::marker::ContravariantLifetime;

// this function is the main entry point when Lua wants to call one of our functions
extern fn wrapper1(lua: *mut ffi::lua_State) -> ::libc::c_int {
    // we load the pointer to the wrapper2 function from an upvalue (an upvalue is a value that was pushed alongside our function)
    let wrapper2raw = unsafe { ffi::lua_touserdata(lua, ffi::lua_upvalueindex(2)) };
    let wrapper2: fn(*mut ffi::lua_State)->::libc::c_int = unsafe { ::std::mem::transmute(wrapper2raw) };

    wrapper2(lua)
}

// this function is called when Lua wants to call one of our functions
fn wrapper2<T>(lua: *mut ffi::lua_State) -> ::libc::c_int where T: AnyCallable {
    // loading the object that we want to call from the Lua context
    let data_raw = unsafe { ffi::lua_touserdata(lua, ffi::lua_upvalueindex(1)) };
    let data: &mut T = unsafe { ::std::mem::transmute(data_raw) };

    AnyCallable::load_args_and_call(data, lua)
}

// this trait should be implemented on objects that are pushed to be callbacks
trait AnyCallable {
    fn load_args_and_call(&mut self, lua: *mut ffi::lua_State) -> ::libc::c_int;
}

// lua context used inside callbacks
#[doc(hidden)]
pub struct InsideCallback<'lua> {
    lua: *mut ffi::lua_State,
    marker: ContravariantLifetime<'lua>,
}

impl<'lua> AsLua for InsideCallback<'lua> {
    fn as_lua(&mut self) -> *mut ffi::lua_State {
        self.lua
    }
}

// should be implemented by objects that can be called
// this will be removed in favor of std::ops::Fn when it is widely supported
// "Args" should be a tuple containing the parameters
trait Callable<'lua, Args: CopyRead<InsideCallback<'lua>>, Ret: Push<InsideCallback<'lua>>> {
    fn do_call(&mut self, args: Args) -> Ret;
}

impl<'lua, Args: CopyRead<InsideCallback<'lua>>, Ret: Push<InsideCallback<'lua>>, T: Callable<'lua, Args, Ret>> AnyCallable for T {
    fn load_args_and_call(&mut self, lua: *mut ffi::lua_State)
        -> ::libc::c_int
    {
        // creating a temporary Lua context in order to pass it to push & read functions
        let mut tmp_lua = InsideCallback { lua: lua, marker: ::std::kinds::marker::ContravariantLifetime } ;

        // trying to read the arguments
        let arguments_count = unsafe { ffi::lua_gettop(lua) } as int;
        let args = match CopyRead::read_from_lua(&mut tmp_lua, -arguments_count as ::libc::c_int) {      // TODO: what if the user has the wrong params?
            None => {
                let err_msg = format!("wrong parameter types for callback function");
                err_msg.push_to_lua(&mut tmp_lua);
                unsafe { ffi::lua_error(lua); }
                unreachable!()
            },
            Some(a) => a
        };

        let ret_value = self.do_call(args);

        // pushing back the result of the function on the stack
        let nb = ret_value.push_to_lua(&mut tmp_lua);

        nb as ::libc::c_int
    }
}

// this macro will allow us to handle multiple parameters count
macro_rules! Push_function(
    ($s:ident, $args:ident, $b:block | $($ty:ident),*) => (
        impl<'lua, L: AsLua, Ret: Push<InsideCallback<'lua>> $(, $ty : CopyRead<InsideCallback<'lua>>+Clone)*> Push<L> for fn($($ty),*)->Ret {
            fn push_to_lua(self, lua: &mut L) -> uint {
                // pushing the function pointer as a userdata
                let lua_data_raw = unsafe { ffi::lua_newuserdata(lua.as_lua(), ::std::mem::size_of_val(&self) as ::libc::size_t) };
                let lua_data: *mut fn($($ty),*)->Ret = unsafe { ::std::mem::transmute(lua_data_raw) };
                unsafe { ::std::ptr::write(lua_data, self) };

                // pushing wrapper2 as a lightuserdata
                let wrapper2: fn(*mut ffi::lua_State)->::libc::c_int = wrapper2::<fn($($ty),*)->Ret>;
                unsafe { ffi::lua_pushlightuserdata(lua.as_lua(), ::std::mem::transmute(wrapper2)) };

                // pushing wrapper1 as a closure
                unsafe { ffi::lua_pushcclosure(lua.as_lua(), ::std::mem::transmute(wrapper1), 2) };

                1
            }
        }

        #[allow(unused_variables)]
        impl<'lua, Ret: Push<InsideCallback<'lua>> $(, $ty : CopyRead<InsideCallback<'lua>>+Clone)*> Callable<'lua,($($ty),*),Ret> for fn($($ty),*)->Ret {
            fn do_call(&mut $s, $args: ($($ty),*)) -> Ret {
                $b
            }
        }

        impl<'lua, L: AsLua, Ret: Push<InsideCallback<'lua>> $(, $ty : CopyRead<InsideCallback<'lua>>+Clone)*> Push<L> for |$($ty),*|:'lua->Ret {
            fn push_to_lua(self, lua: &mut L) -> uint {
                // pushing the function pointer as a userdata
                let lua_data_raw = unsafe { ffi::lua_newuserdata(lua.as_lua(), ::std::mem::size_of_val(&self) as ::libc::size_t) };
                let lua_data: *mut |$($ty),*|->Ret = unsafe { ::std::mem::transmute(lua_data_raw) };
                unsafe { ::std::ptr::write(lua_data, self) };

                // pushing wrapper2 as a lightuserdata
                let wrapper2: fn(*mut ffi::lua_State)->::libc::c_int = wrapper2::<|$($ty),*|:'lua->Ret>;
                unsafe { ffi::lua_pushlightuserdata(lua.as_lua(), ::std::mem::transmute(wrapper2)) };

                // pushing wrapper1 as a closure
                unsafe { ffi::lua_pushcclosure(lua.as_lua(), ::std::mem::transmute(wrapper1), 2) };

                1
            }
        }

        #[allow(unused_variables)]
        impl<'lua, Ret: Push<InsideCallback<'lua>> $(, $ty : CopyRead<InsideCallback<'lua>>+Clone)*> Callable<'lua,($($ty),*),Ret> for |$($ty),*|:'lua->Ret {
            fn do_call(&mut $s, $args: ($($ty),*)) -> Ret {
                $b
            }
        }
    );
);

Push_function!(self, args, { (*self)() } | );
Push_function!(self, args, { (*self)(args) } | Arg1 );
Push_function!(self, args, { (*self)(args.ref0().clone(), args.ref1().clone()) } | Arg1, Arg2 );
Push_function!(self, args, { (*self)(args.ref0().clone(), args.ref1().clone(), args.ref2().clone()) } | Arg1, Arg2, Arg3 );
Push_function!(self, args, { (*self)(args.ref0().clone(), args.ref1().clone(), args.ref2().clone(), args.ref3().clone()) } | Arg1, Arg2, Arg3, Arg4 );
Push_function!(self, args, { (*self)(args.ref0().clone(), args.ref1().clone(), args.ref2().clone(), args.ref3().clone(), args.ref4().clone()) } | Arg1, Arg2, Arg3, Arg4, Arg5 );
// TODO: finish



impl<'lua, T: Push<InsideCallback<'lua>>, E: ::std::fmt::Show> Push<InsideCallback<'lua>> for Result<T,E> {
    fn push_to_lua(self, lua: &mut InsideCallback<'lua>) -> uint {
        match self {
            Ok(val) => val.push_to_lua(lua),
            Err(val) => {
                let msg = format!("{}", val);
                msg.push_to_lua(lua);
                unsafe { ffi::lua_error(lua.as_lua()); }
                unreachable!()
            }
        }
    }
}
