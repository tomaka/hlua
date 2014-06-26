extern crate libc;
extern crate std;
extern crate sync;

use super::liblua;
use { Lua, Pushable, CopyReadable };

extern fn wrapper1(lua: *mut liblua::lua_State) -> libc::c_int {
    // TODO: in the future the value to load will not be the upvalue itself, but the value pointed by the upvalue
    let wrapper2raw = unsafe { liblua::lua_touserdata(lua, liblua::lua_upvalueindex(2)) };
    let wrapper2: fn(*mut liblua::lua_State)->libc::c_int = unsafe { std::mem::transmute(wrapper2raw) };
    wrapper2(lua)
}

fn wrapper2<T: AnyCallable>(lua: *mut liblua::lua_State) -> libc::c_int {
    let dataRaw = unsafe { liblua::lua_touserdata(lua, liblua::lua_upvalueindex(1)) };
    let data: &T = unsafe { std::mem::transmute(&dataRaw) };

    data.load_args_and_call(lua)
}

trait AnyCallable {
    fn load_args_and_call(&self, lua: *mut liblua::lua_State)
        -> libc::c_int;
}

trait Callable<Args: CopyReadable, Ret: Pushable> {
    fn do_call(&self, args: Args) -> Ret;
}

impl<Args: CopyReadable, Ret: Pushable, T: Callable<Args, Ret>> AnyCallable for T {
    fn load_args_and_call(&self, lua: *mut liblua::lua_State)
        -> libc::c_int
    {
        let mut tmpLua = Lua{lua:lua};      // this is actually pretty dangerous (even if we forget it at the end) because in case of unwinding lua_close will be called

        let argumentsCount = unsafe { liblua::lua_gettop(lua) } as uint;

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
        let nb = retValue.push_to_lua(&mut tmpLua);

        unsafe { std::mem::forget(tmpLua) };   // do not call lua_close on this temporary context

        nb as libc::c_int
    }
}

macro_rules! pushable_function(
    ($b:block | $($ty:ident),*) => (
        impl<Ret: Pushable $(, $ty : CopyReadable+Clone)*> Pushable for fn($($ty),*)->Ret {
            fn push_to_lua(&self, lua: &mut Lua) -> uint {
                // pushing the function pointer as a lightuserdata
                unsafe { liblua::lua_pushlightuserdata(lua.lua, std::mem::transmute(*self)) };

                // pushing wrapper2 as a lightuserdata
                let wrapper2 = wrapper2::<fn($($ty),*)->Ret>;
                unsafe { liblua::lua_pushlightuserdata(lua.lua, std::mem::transmute(wrapper2)) };

                // pushing wrapper1 as a closure
                unsafe { liblua::lua_pushcclosure(lua.lua, std::mem::transmute(wrapper1), 2) };

                1
            }
        }

        impl<Ret: Pushable $(, $ty : CopyReadable+Clone)*> Callable<($($ty),*),Ret> for fn($($ty),*)->Ret {
            fn do_call(&self, args: ($($ty),*))
                -> Ret
            {
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
        lua.set("ret5", ret5).unwrap();

        let val: int = lua.execute("return ret5()").unwrap();
        assert_eq!(val, 5);
    }

    #[test]
    fn one_argument() {
        let mut lua = super::super::Lua::new();

        fn plus_one(val: int) -> int { val + 1 };
        lua.set("plus_one", plus_one).unwrap();

        let val: int = lua.execute("return plus_one(3)").unwrap();
        assert_eq!(val, 4);
    }

    #[test]
    fn two_arguments() {
        let mut lua = super::super::Lua::new();

        fn add(val1: int, val2: int) -> int { val1 + val2 };
        lua.set("add", add).unwrap();

        let val: int = lua.execute("return add(3, 7)").unwrap();
        assert_eq!(val, 10);
    }

    #[test]
    fn wrong_arguments_types() {
        let mut lua = super::super::Lua::new();

        fn add(val1: int, val2: int) -> int { val1 + val2 };
        lua.set("add", add).unwrap();

        match lua.execute("return add(3, \"hello\")") {
            Ok(x) => { let a: int = x; fail!() },
            Err(_) => ()        // TODO: check for execerror
        }
    }
}
