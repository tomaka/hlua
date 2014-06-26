extern crate libc;
extern crate std;

use super::liblua;
use super::Lua;
use super::Pushable;

impl<T: Pushable> Pushable for Vec<T> {
    fn push_to_lua(&self, lua: &mut Lua) -> uint {
        self.as_slice().push_to_lua(lua)
    }
}

impl<'a, T: Pushable> Pushable for &'a [T] {
    fn push_to_lua(&self, lua: &mut Lua) -> uint {
        // creating empty table
        unsafe { liblua::lua_newtable(lua.lua) };

        for i in range(0, self.len()) {
            (i+1).push_to_lua(lua);
            self[i].push_to_lua(lua);
            unsafe { liblua::lua_settable(lua.lua, -3) };
        }

        1
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn write() {
        let mut lua = super::super::Lua::new();

        lua.set("a", vec!(9i, 8, 7)).unwrap();

        // TODO: test if it worked once reading tables is supported
    }
}
