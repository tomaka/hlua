extern crate lua = "rust-hl-lua";
use lua::Lua;
use lua::any::{AnyLuaValue, Number, String, Boolean};

#[test]
fn read_numbers() {
    let mut lua = Lua::new();

    lua.set("a", "-2");
    lua.set("b", 3.5f32);

    let x: AnyLuaValue = lua.get("a").unwrap();
    assert_eq!(x, Number(-2.0));

    let y: AnyLuaValue = lua.get("b").unwrap();
    assert_eq!(y, Number(3.5));
}

#[test]
fn read_strings() {
    let mut lua = Lua::new();

    lua.set("a", "hello");
    lua.set("b", "3x");
    lua.set("c", "false");

    let x: AnyLuaValue = lua.get("a").unwrap();
    assert_eq!(x, String("hello".to_string()));

    let y: AnyLuaValue = lua.get("b").unwrap();
    assert_eq!(y, String("3x".to_string()));

    let z: AnyLuaValue = lua.get("c").unwrap();
    assert_eq!(z, String("false".to_string()));
}

#[test]
fn read_booleans() {
    let mut lua = Lua::new();

    lua.set("a", true);
    lua.set("b", false);

    let x: AnyLuaValue = lua.get("a").unwrap();
    assert_eq!(x, Boolean(true));

    let y: AnyLuaValue = lua.get("b").unwrap();
    assert_eq!(y, Boolean(false));
}

#[test]
fn push_numbers() {
    let mut lua = Lua::new();

    lua.set("a", Number(3.0));

    let x: int = lua.get("a").unwrap();
    assert_eq!(x, 3);
}

#[test]
fn push_strings() {
    let mut lua = Lua::new();

    lua.set("a", String("hello".to_string()));

    let x: String = lua.get("a").unwrap();
    assert_eq!(x.as_slice(), "hello");
}

#[test]
fn push_booleans() {
    let mut lua = Lua::new();

    lua.set("a", Boolean(true));

    let x: bool = lua.get("a").unwrap();
    assert_eq!(x, true);
}
