extern crate libc;
extern crate std;
extern crate sync;

use super::liblua;
use { Lua, Pushable, CopyReadable };

// this function is the main entry point when Lua wants to call one of our functions
extern fn wrapper1(lua: *mut liblua::lua_State) -> libc::c_int {
    // we load the pointer to the wrapper2 function from an upvalue (an upvalue is a value that was pushed alongside our function)
    let wrapper2raw = unsafe { liblua::lua_touserdata(lua, liblua::lua_upvalueindex(2)) };
    let wrapper2: fn(*mut liblua::lua_State)->libc::c_int = unsafe { std::mem::transmute(wrapper2raw) };

    wrapper2(lua)
}

// this function is called when Lua wants to call one of our functions
fn wrapper2<T: AnyCallable>(lua: *mut liblua::lua_State) -> libc::c_int {
    // loading the object that we want to call from the Lua context
    // TODO: in the future the value to load will not be the upvalue itself, but the value pointed by the upvalue
    let dataRaw = unsafe { liblua::lua_touserdata(lua, liblua::lua_upvalueindex(1)) };
    let data: &T = unsafe { std::mem::transmute(&dataRaw) };

    data.load_args_and_call(lua)
}

// this trait should be implemented on objects that are pushed to be callbacks
trait AnyCallable {
    fn load_args_and_call(&self, lua: *mut liblua::lua_State) -> libc::c_int;
}

// should be implemented by objects that can be called
// this will be removed in favor of std::ops::Fn when it is widely supported
// "Args" should be a tuple containing the parameters
trait Callable<Args: CopyReadable, Ret: Pushable> {
    fn do_call(&self, args: Args) -> Ret;
}

impl<Args: CopyReadable, Ret: Pushable, T: Callable<Args, Ret>> AnyCallable for T {
    fn load_args_and_call(&self, lua: *mut liblua::lua_State)
        -> libc::c_int
    {
        // creating a temporary Lua context in order to pass it to push & read functions
        // this is actually pretty dangerous (even if we forget it at the end) because in case of unwinding lua_close will be called
        let mut tmpLua = Lua{lua:lua};

        // trying to read the arguments
        let argumentsCount = unsafe { liblua::lua_gettop(lua) } as int;
        let args = match CopyReadable::read_from_lua(&mut tmpLua, -argumentsCount as libc::c_int) {      // TODO: what if the user has the wrong params?
            None => {
                let errMsg = format!("wrong parameter types for callback function");
                errMsg.push_to_lua(&mut tmpLua);
                unsafe { liblua::lua_error(lua); }
                unreachable!()
            },
            Some(a) => a
        };

        let retValue = self.do_call(args);

        // pushing back the result of the function on the stack
        let nb = retValue.push_to_lua(&mut tmpLua);

        // do not call lua_close on this temporary context
        unsafe { std::mem::forget(tmpLua) };

        nb as libc::c_int
    }
}

// this macro will allow us to handle multiple parameters count
macro_rules! pushable_function(
    ($b:block | $($ty:ident),*) => (
        impl<Ret: Pushable $(, $ty : CopyReadable+Clone)*> Pushable for fn($($ty),*)->Ret {
            fn push_to_lua(&self, lua: &mut Lua) -> uint {
                // pushing the function pointer as a lightuserdata
                // TODO: should be pushed as a real user data instead, for compatibility with non-functions
                unsafe { liblua::lua_pushlightuserdata(lua.lua, std::mem::transmute(*self)) };

                // pushing wrapper2 as a lightuserdata
                let wrapper2: fn(*mut liblua::lua_State)->libc::c_int = wrapper2::<fn($($ty),*)->Ret>;
                unsafe { liblua::lua_pushlightuserdata(lua.lua, std::mem::transmute(wrapper2)) };

                // pushing wrapper1 as a closure
                unsafe { liblua::lua_pushcclosure(lua.lua, std::mem::transmute(wrapper1), 2) };

                1
            }
        }

        #[allow(unused_variable)]
        impl<Ret: Pushable $(, $ty : CopyReadable+Clone)*> Callable<($($ty),*),Ret> for fn($($ty),*)->Ret {
            fn do_call(&self, args: ($($ty),*)) -> Ret {
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



#[cfg(test)]
mod tests {
    #[test]
    fn simple_function() {
        let mut lua = super::super::Lua::new();

        fn ret5() -> int { 5 };
        lua.set("ret5", ret5);

        let val: int = lua.execute("return ret5()").unwrap();
        assert_eq!(val, 5);
    }

    #[test]
    fn one_argument() {
        let mut lua = super::super::Lua::new();

        fn plus_one(val: int) -> int { val + 1 };
        lua.set("plus_one", plus_one);

        let val: int = lua.execute("return plus_one(3)").unwrap();
        assert_eq!(val, 4);
    }

    #[test]
    fn two_arguments() {
        let mut lua = super::super::Lua::new();

        fn add(val1: int, val2: int) -> int { val1 + val2 };
        lua.set("add", add);

        let val: int = lua.execute("return add(3, 7)").unwrap();
        assert_eq!(val, 10);
    }

    #[test]
    fn wrong_arguments_types() {
        let mut lua = super::super::Lua::new();

        fn add(val1: int, val2: int) -> int { val1 + val2 };
        lua.set("add", add);

        match lua.execute("return add(3, \"hello\")") {
            Ok(x) => { let _: int = x; fail!() },
            Err(_) => ()        // TODO: check for execerror
        }
    }
}
