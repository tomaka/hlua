extern crate hlua;

use hlua::Lua;
use hlua::AnyLuaValue;

#[test]
fn read_numbers() {
    let mut lua = Lua::new();

    lua.set("a", "-2");
    lua.set("b", 3.5f32);

    let x: AnyLuaValue = lua.get("a").unwrap();
    assert_eq!(x, AnyLuaValue::LuaNumber(-2.0));

    let y: AnyLuaValue = lua.get("b").unwrap();
    assert_eq!(y, AnyLuaValue::LuaNumber(3.5));
}

#[test]
fn read_strings() {
    let mut lua = Lua::new();

    lua.set("a", "hello");
    lua.set("b", "3x");
    lua.set("c", "false");

    let x: AnyLuaValue = lua.get("a").unwrap();
    assert_eq!(x, AnyLuaValue::LuaString("hello".to_string()));

    let y: AnyLuaValue = lua.get("b").unwrap();
    assert_eq!(y, AnyLuaValue::LuaString("3x".to_string()));

    let z: AnyLuaValue = lua.get("c").unwrap();
    assert_eq!(z, AnyLuaValue::LuaString("false".to_string()));
}

#[test]
fn read_booleans() {
    let mut lua = Lua::new();

    lua.set("a", true);
    lua.set("b", false);

    let x: AnyLuaValue = lua.get("a").unwrap();
    assert_eq!(x, AnyLuaValue::LuaBoolean(true));

    let y: AnyLuaValue = lua.get("b").unwrap();
    assert_eq!(y, AnyLuaValue::LuaBoolean(false));
}

#[test]
fn push_numbers() {
    let mut lua = Lua::new();

    lua.set("a", AnyLuaValue::LuaNumber(3.0));

    let x: i32 = lua.get("a").unwrap();
    assert_eq!(x, 3);
}

#[test]
fn push_strings() {
    let mut lua = Lua::new();

    lua.set("a", AnyLuaValue::LuaString("hello".to_string()));

    let x: String = lua.get("a").unwrap();
    assert_eq!(x, "hello");
}

#[test]
fn push_booleans() {
    let mut lua = Lua::new();

    lua.set("a", AnyLuaValue::LuaBoolean(true));

    let x: bool = lua.get("a").unwrap();
    assert_eq!(x, true);
}

#[test]
fn push_nil() {
    let mut lua = Lua::new();

    lua.set("a", AnyLuaValue::LuaNil);

    let x: Option<i32> = lua.get("a");
    assert!(x.is_none(),
            "x is a Some value when it should be a None value. X: {:?}",
            x);
}
