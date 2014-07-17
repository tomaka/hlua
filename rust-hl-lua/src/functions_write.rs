use super::ffi;
use { Lua, Push, CopyRead };

// this function is the main entry point when Lua wants to call one of our functions
extern fn wrapper1(lua: *mut ffi::lua_State) -> ::libc::c_int {
    // we load the pointer to the wrapper2 function from an upvalue (an upvalue is a value that was pushed alongside our function)
    let wrapper2raw = unsafe { ffi::lua_touserdata(lua, ffi::lua_upvalueindex(2)) };
    let wrapper2: fn(*mut ffi::lua_State)->::libc::c_int = unsafe { ::std::mem::transmute(wrapper2raw) };

    wrapper2(lua)
}

// this function is called when Lua wants to call one of our functions
fn wrapper2<T: AnyCallable>(lua: *mut ffi::lua_State) -> ::libc::c_int {
    // loading the object that we want to call from the Lua context
    let dataRaw = unsafe { ffi::lua_touserdata(lua, ffi::lua_upvalueindex(1)) };
    let data: &mut T = unsafe { ::std::mem::transmute(dataRaw) };

    data.load_args_and_call(lua)
}

// this trait should be implemented on objects that are pushed to be callbacks
trait AnyCallable {
    fn load_args_and_call(&mut self, lua: *mut ffi::lua_State) -> ::libc::c_int;
}

// should be implemented by objects that can be called
// this will be removed in favor of std::ops::Fn when it is widely supported
// "Args" should be a tuple containing the parameters
trait Callable<'lua, Args: CopyRead, Ret: Push<Lua<'lua>>> {
    fn do_call(&mut self, args: Args) -> Ret;
}

impl<'lua, Args: CopyRead, Ret: Push<Lua<'lua>>, T: Callable<'lua, Args, Ret>> AnyCallable for T {
    fn load_args_and_call(&mut self, lua: *mut ffi::lua_State)
        -> ::libc::c_int
    {
        // creating a temporary Lua context in order to pass it to push & read functions
        let mut tmpLua = Lua { lua: lua, marker: ::std::kinds::marker::ContravariantLifetime, must_be_closed: false, inside_callback: true } ;

        // trying to read the arguments
        let argumentsCount = unsafe { ffi::lua_gettop(lua) } as int;
        let args = match CopyRead::read_from_lua(&mut tmpLua, -argumentsCount as ::libc::c_int) {      // TODO: what if the user has the wrong params?
            None => {
                let errMsg = format!("wrong parameter types for callback function");
                errMsg.push_to_lua(&mut tmpLua);
                unsafe { ffi::lua_error(lua); }
                unreachable!()
            },
            Some(a) => a
        };

        let retValue = self.do_call(args);

        // pushing back the result of the function on the stack
        let nb = retValue.push_to_lua(&mut tmpLua);

        nb as ::libc::c_int
    }
}

// this macro will allow us to handle multiple parameters count
macro_rules! Push_function(
    ($s:ident, $args:ident, $b:block | $($ty:ident),*) => (
        impl<'lua, Ret: Push<Lua<'lua>> $(, $ty : CopyRead+Clone)*> Push<Lua<'lua>> for fn($($ty),*)->Ret {
            fn push_to_lua(self, lua: &mut Lua) -> uint {
                // pushing the function pointer as a userdata
                let luaDataRaw = unsafe { ffi::lua_newuserdata(lua.lua, ::std::mem::size_of_val(&self) as ::libc::size_t) };
                let luaData: *mut fn($($ty),*)->Ret = unsafe { ::std::mem::transmute(luaDataRaw) };
                unsafe { ::std::ptr::write(luaData, self) };

                // pushing wrapper2 as a lightuserdata
                let wrapper2: fn(*mut ffi::lua_State)->::libc::c_int = wrapper2::<fn($($ty),*)->Ret>;
                unsafe { ffi::lua_pushlightuserdata(lua.lua, ::std::mem::transmute(wrapper2)) };

                // pushing wrapper1 as a closure
                unsafe { ffi::lua_pushcclosure(lua.lua, ::std::mem::transmute(wrapper1), 2) };

                1
            }
        }

        #[allow(unused_variable)]
        impl<'lua, Ret: Push<Lua<'lua>> $(, $ty : CopyRead+Clone)*> Callable<'lua,($($ty),*),Ret> for fn($($ty),*)->Ret {
            fn do_call(&mut $s, $args: ($($ty),*)) -> Ret {
                $b
            }
        }

        impl<'lua, Ret: Push<Lua<'lua>> $(, $ty : CopyRead+Clone)*> Push<Lua<'lua>> for |$($ty),*|:'lua->Ret {
            fn push_to_lua(self, lua: &mut Lua) -> uint {
                // pushing the function pointer as a userdata
                let luaDataRaw = unsafe { ffi::lua_newuserdata(lua.lua, ::std::mem::size_of_val(&self) as ::libc::size_t) };
                let luaData: *mut |$($ty),*|->Ret = unsafe { ::std::mem::transmute(luaDataRaw) };
                unsafe { ::std::ptr::write(luaData, self) };

                // pushing wrapper2 as a lightuserdata
                let wrapper2: fn(*mut ffi::lua_State)->::libc::c_int = wrapper2::<|$($ty),*|:'lua->Ret>;
                unsafe { ffi::lua_pushlightuserdata(lua.lua, ::std::mem::transmute(wrapper2)) };

                // pushing wrapper1 as a closure
                unsafe { ffi::lua_pushcclosure(lua.lua, ::std::mem::transmute(wrapper1), 2) };

                1
            }
        }

        #[allow(unused_variable)]
        impl<'lua, Ret: Push<Lua<'lua>> $(, $ty : CopyRead+Clone)*> Callable<'lua,($($ty),*),Ret> for |$($ty),*|:'lua->Ret {
            fn do_call(&mut $s, $args: ($($ty),*)) -> Ret {
                $b
            }
        }
    );
)

Push_function!(self, args, { (*self)() } | )
Push_function!(self, args, { (*self)(args) } | Arg1 )
Push_function!(self, args, { (*self)(args.ref0().clone(), args.ref1().clone()) } | Arg1, Arg2 )
Push_function!(self, args, { (*self)(args.ref0().clone(), args.ref1().clone(), args.ref2().clone()) } | Arg1, Arg2, Arg3 )
Push_function!(self, args, { (*self)(args.ref0().clone(), args.ref1().clone(), args.ref2().clone(), args.ref3().clone()) } | Arg1, Arg2, Arg3, Arg4 )
Push_function!(self, args, { (*self)(args.ref0().clone(), args.ref1().clone(), args.ref2().clone(), args.ref3().clone(), args.ref4().clone()) } | Arg1, Arg2, Arg3, Arg4, Arg5 )
// TODO: finish



impl<'lua, T: Push<Lua<'lua>>, E: ::std::fmt::Show> Push<Lua<'lua>> for Result<T,E> {
    fn push_to_lua(self, lua: &mut Lua<'lua>) -> uint {
        if !lua.inside_callback {
            fail!("cannot push a Result object except as a function return type")
        }

        match self {
            Ok(val) => val.push_to_lua(lua),
            Err(val) => {
                let msg = format!("{}", val);
                msg.push_to_lua(lua);
                unsafe { ffi::lua_error(lua.lua); }
                unreachable!()
            }
        }
    }
}
