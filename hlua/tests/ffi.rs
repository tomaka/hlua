//! Test to make sure the low-level API can be accessed from hlua.

extern crate hlua;

use hlua::AsLua;

#[test]
fn get_version() {
    let lua = hlua::Lua::new();
    let state_ptr = lua.as_lua().state_ptr();

    let version = unsafe { *hlua::ffi::lua_version(state_ptr) as i32 };
    // 502 = Lua 5.2
    assert_eq!(version, 502);
}
