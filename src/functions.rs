extern crate libc;
extern crate std;

use super::liblua;
use super::Index;
use super::Lua;
use super::Pushable;
use super::Readable;

impl<'a, TRetValue> Pushable for ||:'a -> TRetValue {
    fn push_to_lua(self, lua: &Lua) {
        //unsafe { liblua::lua_pushinteger(lua.lua, self) }
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

