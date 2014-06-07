extern crate libc;
extern crate std;

use super::liblua;
use super::Lua;
use super::Pushable;
use super::Readable;

pub struct UserData<T> {
    value: T
}

impl<T:Clone> UserData<T> {
    pub fn new(val: T) -> UserData<T> {
        UserData{value: val}
    }
}

// TODO: handle destructors

impl<T:Clone> Pushable for UserData<T> {
    fn push_to_lua(&self, lua: &Lua) {
        unsafe {
            let dataRaw = liblua::lua_newuserdata(lua.lua, std::mem::size_of_val(&self.value) as libc::size_t);
            let data: &mut T = std::mem::transmute(dataRaw);
            (*data) = self.value.clone();
        }
    }
}

impl<T:Clone> Readable for UserData<T> {
    fn read_from_lua(lua: &Lua, index: i32) -> Option<UserData<T>> {
        unsafe {
            // TODO: check type
            let dataPtr = liblua::lua_touserdata(lua.lua, index);
            let data: &T = std::mem::transmute(dataPtr);
            Some(UserData{value: data.clone()})
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn readwrite() {
        let mut lua = super::super::Lua::new();
        let d = super::UserData::new(2);

        lua.set("a", d);
        let x: super::UserData<int> = lua.get("a").unwrap();
        assert_eq!(x.value, 2)
    }
}
