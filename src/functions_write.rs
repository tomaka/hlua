use super::ffi;
use { Lua, Pushable, CopyReadable };

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
trait Callable<'lua, Args: CopyReadable, Ret: Pushable<'lua>> {
    fn do_call(&mut self, args: Args) -> Ret;
}

impl<'lua, Args: CopyReadable, Ret: Pushable<'lua>, T: Callable<'lua, Args, Ret>> AnyCallable for T {
    fn load_args_and_call(&mut self, lua: *mut ffi::lua_State)
        -> ::libc::c_int
    {
        // creating a temporary Lua context in order to pass it to push & read functions
        let mut tmpLua = Lua { lua: lua, marker: ::std::kinds::marker::ContravariantLifetime, must_be_closed: false, inside_callback: true } ;

        // trying to read the arguments
        let argumentsCount = unsafe { ffi::lua_gettop(lua) } as int;
        let args = match CopyReadable::read_from_lua(&mut tmpLua, -argumentsCount as ::libc::c_int) {      // TODO: what if the user has the wrong params?
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
macro_rules! pushable_function(
    ($b:block | $($ty:ident),*) => (
        impl<'lua, Ret: Pushable<'lua> $(, $ty : CopyReadable+Clone)*> Pushable<'lua> for fn($($ty),*)->Ret {
            fn push_to_lua(self, lua: &mut Lua) -> uint {
                // pushing the function pointer as a userdata
                let luaDataRaw = unsafe { ffi::lua_newuserdata(lua.lua, ::std::mem::size_of_val(&self) as ::libc::size_t) };
                let luaData: &mut fn($($ty),*)->Ret = unsafe { ::std::mem::transmute(luaDataRaw) };
                (*luaData) = self;

                // pushing wrapper2 as a lightuserdata
                let wrapper2: fn(*mut ffi::lua_State)->::libc::c_int = wrapper2::<fn($($ty),*)->Ret>;
                unsafe { ffi::lua_pushlightuserdata(lua.lua, ::std::mem::transmute(wrapper2)) };

                // pushing wrapper1 as a closure
                unsafe { ffi::lua_pushcclosure(lua.lua, ::std::mem::transmute(wrapper1), 2) };

                1
            }
        }

        #[allow(unused_variable)]
        impl<'lua, Ret: Pushable<'lua> $(, $ty : CopyReadable+Clone)*> Callable<'lua,($($ty),*),Ret> for fn($($ty),*)->Ret {
            fn do_call(&mut self, args: ($($ty),*)) -> Ret {
                $b
            }
        }

        impl<'lua, Ret: Pushable<'lua> $(, $ty : CopyReadable+Clone)*> Pushable<'lua> for |$($ty),*|:'lua->Ret {
            fn push_to_lua(self, lua: &mut Lua) -> uint {
                // pushing the function pointer as a userdata
                let luaDataRaw = unsafe { ffi::lua_newuserdata(lua.lua, ::std::mem::size_of_val(&self) as ::libc::size_t) };
                let luaData: &mut |$($ty),*|->Ret = unsafe { ::std::mem::transmute(luaDataRaw) };
                (*luaData) = self;

                // pushing wrapper2 as a lightuserdata
                let wrapper2: fn(*mut ffi::lua_State)->::libc::c_int = wrapper2::<|$($ty),*|:'lua->Ret>;
                unsafe { ffi::lua_pushlightuserdata(lua.lua, ::std::mem::transmute(wrapper2)) };

                // pushing wrapper1 as a closure
                unsafe { ffi::lua_pushcclosure(lua.lua, ::std::mem::transmute(wrapper1), 2) };

                1
            }
        }

        #[allow(unused_variable)]
        impl<'lua, Ret: Pushable<'lua> $(, $ty : CopyReadable+Clone)*> Callable<'lua,($($ty),*),Ret> for |$($ty),*|:'lua->Ret {
            fn do_call(&mut self, args: ($($ty),*)) -> Ret {
                $b
            }
        }
    );
)

pushable_function!({ (*self)() } | )
pushable_function!({ (*self)(args) } | Arg1 )
pushable_function!({ (*self)(args.ref0().clone(), args.ref1().clone()) } | Arg1, Arg2 )
pushable_function!({ (*self)(args.ref0().clone(), args.ref1().clone(), args.ref2().clone()) } | Arg1, Arg2, Arg3 )
pushable_function!({ (*self)(args.ref0().clone(), args.ref1().clone(), args.ref2().clone(), args.ref3().clone()) } | Arg1, Arg2, Arg3, Arg4 )
pushable_function!({ (*self)(args.ref0().clone(), args.ref1().clone(), args.ref2().clone(), args.ref3().clone(), args.ref4().clone()) } | Arg1, Arg2, Arg3, Arg4, Arg5 )
// TODO: finish



impl<'lua, T: Pushable<'lua>, E: ::std::fmt::Show> Pushable<'lua> for Result<T,E> {
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


#[cfg(test)]
mod tests {
    use Lua;

    #[test]
    fn simple_function() {
        let mut lua = Lua::new();

        fn ret5() -> int { 5 };
        lua.set("ret5", ret5);

        let val: int = lua.execute("return ret5()").unwrap();
        assert_eq!(val, 5);
    }

    #[test]
    fn one_argument() {
        let mut lua = Lua::new();

        fn plus_one(val: int) -> int { val + 1 };
        lua.set("plus_one", plus_one);

        let val: int = lua.execute("return plus_one(3)").unwrap();
        assert_eq!(val, 4);
    }

    #[test]
    fn two_arguments() {
        let mut lua = Lua::new();

        fn add(val1: int, val2: int) -> int { val1 + val2 };
        lua.set("add", add);

        let val: int = lua.execute("return add(3, 7)").unwrap();
        assert_eq!(val, 10);
    }

    #[test]
    fn wrong_arguments_types() {
        let mut lua = Lua::new();

        fn add(val1: int, val2: int) -> int { val1 + val2 };
        lua.set("add", add);

        match lua.execute::<int>("return add(3, \"hello\")") {
            Err(::ExecutionError(_)) => (),
            _ => fail!()
        }
    }

    #[test]
    fn return_result() {
        let mut lua = Lua::new();

        fn always_fails() -> Result<int, &'static str> { Err("oops, problem") };
        lua.set("always_fails", always_fails);

        match lua.execute::<()>("always_fails()") {
            Err(::ExecutionError(_)) => (),
            _ => fail!()
        }
    }

    #[test]
    fn closures() {
        let mut lua = Lua::new();

        lua.set("add", |a:int, b:int| a + b);
        lua.set("sub", |a:int, b:int| a - b);

        let val1: int = lua.execute("return add(3, 7)").unwrap();
        assert_eq!(val1, 10);

        let val2: int = lua.execute("return sub(5, 2)").unwrap();
        assert_eq!(val2, 3);
    }

    #[test]
    fn closures_lifetime() {
        fn t(f: |int,int|->int) {
            let mut lua = Lua::new();

            lua.set("add", f);

            let val1: int = lua.execute("return add(3, 7)").unwrap();
            assert_eq!(val1, 10);
        }

        t(|a,b| a+b);
    }

    #[test]
    fn closures_extern_access() {
        let mut a = 5i;

        {
            let mut lua = Lua::new();

            lua.set("inc", || a += 1);
            for _ in range(0i, 15) {
                lua.execute::<()>("inc()").unwrap();
            }
        }

        assert_eq!(a, 20)
    }
}
