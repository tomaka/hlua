extern crate libc;
extern crate std;
extern crate sync;

use super::liblua;
use super::Index;
use super::Lua;
use super::Pushable;
use super::Readable;

extern fn wrapper1(lua: *mut liblua::lua_State) -> libc::c_int {
    unsafe {
        let argumentsCount = liblua::lua_gettop(lua);

        let data = liblua::lua_touserdata(lua, liblua::lua_upvalueindex(2));
        let wrapper2ptr = liblua::lua_touserdata(lua, liblua::lua_upvalueindex(1));
        let wrapper2: &fn(*mut liblua::lua_State, libc::c_int, *mut libc::c_void)->libc::c_int = std::mem::transmute(wrapper2ptr);

        (*wrapper2)(lua, argumentsCount, data)
    }
}

/*fn wrapper2proc<Ret: Pushable>(lua: *mut liblua::lua_State, argumentsCount: libc::c_int, p: &proc()->Ret) -> libc::c_int {
    let ret = p();
    ret.push_to_lua(Lua{lua:lua});
    1
}*/

fn wrapper2fn<Ret: Pushable>(lua: *mut liblua::lua_State, argumentsCount: libc::c_int, p: &fn()->Ret) -> libc::c_int {
    let ret = (*p)();
    ret.push_to_lua(&Lua{lua:lua});
    1
}

/*impl<Ret: Pushable> Pushable for proc()->Ret {
    fn push_to_lua(&self, lua: &Lua) {
        unsafe {
            // pushing the std::raw::Procedure as a userdata
            let rawProc: &std::raw::Procedure = std::mem::transmute(self);
            let mut userDataPtr = liblua::lua_newuserdata(lua.lua, std::mem::size_of_val(rawProc) as libc::size_t);
            let userData: &mut std::raw::Procedure = std::mem::transmute(userDataPtr);
            userData.code = rawProc.code;
            userData.env = rawProc.env;

            // pushing wrapper2 as a lightuserdata
            let wrapper2: &fn(*mut liblua::lua_State, libc::c_int, &proc()->Ret)->libc::c_int = &wrapper2proc;
            liblua::lua_pushlightuserdata(lua.lua, std::mem::transmute(wrapper2));

            // pushing wrapper1 as a closure
            liblua::lua_pushcclosure(lua.lua, std::mem::transmute(wrapper1), 2);
        }
    }
}*/

impl<Ret: Pushable> Pushable for fn()->Ret {
    fn push_to_lua(&self, lua: &Lua) {
        unsafe {
            // pushing the function pointer as a lightuserdata
            liblua::lua_pushlightuserdata(lua.lua, std::mem::transmute(self));

            // pushing wrapper2 as a lightuserdata
            let wrapper2: &fn(*mut liblua::lua_State, libc::c_int, &fn()->Ret)->libc::c_int = &wrapper2fn;
            liblua::lua_pushlightuserdata(lua.lua, std::mem::transmute(wrapper2));

            // pushing wrapper1 as a closure
            liblua::lua_pushcclosure(lua.lua, std::mem::transmute(wrapper1), 2);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn execute() {
        let mut lua = super::super::Lua::new();

        let val: int = lua.execute("return 5").unwrap();
        assert_eq!(val, 5);
    }

    /*#[test]
    fn simple_function() {
        let mut lua = super::super::Lua::new();

        fn ret5() -> int { 5 };
        lua.set("ret5", &ret5);

        let val: int = lua.execute("return ret5()").unwrap();
        assert_eq!(val, 5);
    }*/
}
