extern crate libc;
extern crate std;
extern crate sync;

use super::liblua;
use super::Lua;
use super::Pushable;

extern fn wrapper1(lua: *mut liblua::lua_State) -> libc::c_int {
    unsafe {
        let argumentsCount = liblua::lua_gettop(lua);

        let data = liblua::lua_touserdata(lua, liblua::lua_upvalueindex(1));
        let wrapper2ptr = liblua::lua_touserdata(lua, liblua::lua_upvalueindex(2));
        let wrapper2: fn(*mut liblua::lua_State, libc::c_int, *mut libc::c_void)->libc::c_int = std::mem::transmute(wrapper2ptr);

        wrapper2(lua, argumentsCount, data)
    }
}

fn wrapper2closure<Ret: Pushable>(lua: *mut liblua::lua_State, argumentsCount: libc::c_int, p: &||->Ret) -> libc::c_int {
    unimplemented!()
    /*let ret = p();
    ret.push_to_lua(&mut Lua{lua:lua});
    1*/
}

fn wrapper2fn<Ret: Pushable>(lua: *mut liblua::lua_State, argumentsCount: libc::c_int, p: fn()->Ret) -> libc::c_int {
    let ret = p();
    let mut tmpLua = Lua{lua:lua};
    ret.push_to_lua(&mut tmpLua);
    unsafe { std::mem::forget(tmpLua) };   // do not call lua_close on this temporary context
    1
}

impl<Ret: Pushable> Pushable for ||:'static->Ret {
    fn push_to_lua(&self, lua: &mut Lua) {
        // pushing the std::raw::Closure as a userdata
        let rawClosure: &std::raw::Closure = unsafe { std::mem::transmute(self) };
        let mut userDataPtr = unsafe { liblua::lua_newuserdata(lua.lua, std::mem::size_of_val(rawClosure) as libc::size_t) };
        let userData: &mut std::raw::Closure = unsafe { std::mem::transmute(userDataPtr) };
        userData.code = rawClosure.code;
        userData.env = rawClosure.env;

        // pushing wrapper2 as a lightuserdata
        let wrapper2: &fn(*mut liblua::lua_State, libc::c_int, &||->Ret)->libc::c_int = &wrapper2closure;
        unsafe { liblua::lua_pushlightuserdata(lua.lua, std::mem::transmute(wrapper2)) };

        // pushing wrapper1 as a closure
        unsafe { liblua::lua_pushcclosure(lua.lua, std::mem::transmute(wrapper1), 2) };
    }
}

impl<Ret: Pushable> Pushable for fn()->Ret {
    fn push_to_lua(&self, lua: &mut Lua) {
        // pushing the function pointer as a lightuserdata
        unsafe { liblua::lua_pushlightuserdata(lua.lua, std::mem::transmute(*self)) };

        // pushing wrapper2 as a lightuserdata
        let wrapper2: fn(*mut liblua::lua_State, libc::c_int, fn()->Ret)->libc::c_int = wrapper2fn;
        unsafe { liblua::lua_pushlightuserdata(lua.lua, std::mem::transmute(wrapper2)) };

        // pushing wrapper1 as a closure
        unsafe { liblua::lua_pushcclosure(lua.lua, std::mem::transmute(wrapper1), 2) };
    }
}

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
}
