extern crate libc;
extern crate std;

use super::liblua;
use super::Index;
use super::Lua;
use super::Pushable;
use super::Readable;

extern fn wrapper(lua: *mut liblua::lua_State) -> libc::c_int {
    /*const auto toCall = static_cast<TFunctionObject*>(lua_touserdata(state, lua_upvalueindex(1)));
    return callback(state, toCall, lua_gettop(state)).release();*/
    0
}
/*
impl<TRetValue: Pushable> Pushable for proc() -> TRetValue {
    fn push_to_lua(self, lua: &Lua) {
    }
}

impl<TRetValue: Pushable> Pushable for fn() -> TRetValue {
    fn push_to_lua(self, lua: &Lua) {
    }
}

impl<TParam1: Readable, TRetValue: Pushable> Pushable for fn(TParam1) -> TRetValue {
    fn push_to_lua(self, lua: &Lua) {
        //unsafe { liblua::lua_pushinteger(lua.lua, self) }
    }
}

*/