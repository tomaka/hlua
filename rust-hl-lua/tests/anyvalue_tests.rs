extern crate "rust-hl-lua" as lua;
use lua::Lua;
use lua::any::{AnyLuaValue,LuaNumber, LuaString, LuaBoolean};

#[test]
fn read_numbers() {
    let mut lua = Lua::new();

    lua.set("a", "-2");
    lua.set("b", 3.5f32);

    let x: AnyLuaValue = lua.get("a").unwrap();
    assert_eq!(x, LuaNumber(-2.0));

    let y: AnyLuaValue = lua.get("b").unwrap();
    assert_eq!(y, LuaNumber(3.5));
}

#[test]
fn read_strings() {
    let mut lua = Lua::new();

    lua.set("a", "hello");
    lua.set("b", "3x");
    lua.set("c", "false");

    let x: AnyLuaValue = lua.get("a").unwrap();
    assert_eq!(x, LuaString("hello".to_string()));

    let y: AnyLuaValue = lua.get("b").unwrap();
    assert_eq!(y, LuaString("3x".to_string()));

    let z: AnyLuaValue = lua.get("c").unwrap();
    assert_eq!(z, LuaString("false".to_string()));
}

#[test]
fn read_booleans() {
    let mut lua = Lua::new();

    lua.set("a", true);
    lua.set("b", false);

    let x: AnyLuaValue = lua.get("a").unwrap();
    assert_eq!(x, LuaBoolean(true));

    let y: AnyLuaValue = lua.get("b").unwrap();
    assert_eq!(y, LuaBoolean(false));
}

#[test]
fn push_numbers() {
    let mut lua = Lua::new();

    lua.set("a", LuaNumber(3.0));

    let x: int = lua.get("a").unwrap();
    assert_eq!(x, 3);
}

#[test]
fn push_strings() {
    let mut lua = Lua::new();

    lua.set("a", LuaString("hello".to_string()));

    let x: String = lua.get("a").unwrap();
    assert_eq!(x.as_slice(), "hello");
}

#[test]
fn push_booleans() {
    let mut lua = Lua::new();

    lua.set("a", LuaBoolean(true));

    let x: bool = lua.get("a").unwrap();
    assert_eq!(x, true);
}
