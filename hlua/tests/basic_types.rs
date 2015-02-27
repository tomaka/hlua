extern crate hlua;

use hlua::Lua;

#[test]
fn read_i32s() {
    let mut lua = Lua::new();

    lua.set("a", 2);

    let x: i32 = lua.get("a").unwrap();
    assert_eq!(x, 2);

    let y: i8 = lua.get("a").unwrap();
    assert_eq!(y, 2);

    let z: i16 = lua.get("a").unwrap();
    assert_eq!(z, 2);

    let w: i32 = lua.get("a").unwrap();
    assert_eq!(w, 2);

    let a: u32 = lua.get("a").unwrap();
    assert_eq!(a, 2);

    let b: u8 = lua.get("a").unwrap();
    assert_eq!(b, 2);

    let c: u16 = lua.get("a").unwrap();
    assert_eq!(c, 2);

    let d: u32 = lua.get("a").unwrap();
    assert_eq!(d, 2);
}

#[test]
fn write_i32s() {
    // TODO: 

    let mut lua = Lua::new();

    lua.set("a", 2);
    let x: i32 = lua.get("a").unwrap();
    assert_eq!(x, 2);
}

#[test]
fn readwrite_floats() {
    let mut lua = Lua::new();

    lua.set("a", 2.51234 as f32);
    lua.set("b", 3.4123456789 as f64);

    let x: f32 = lua.get("a").unwrap();
    assert!(x - 2.51234 < 0.000001);

    let y: f64 = lua.get("a").unwrap();
    assert!(y - 2.51234 < 0.000001);

    let z: f32 = lua.get("b").unwrap();
    assert!(z - 3.4123456789 < 0.000001);

    let w: f64 = lua.get("b").unwrap();
    assert!(w - 3.4123456789 < 0.000001);
}

#[test]
fn readwrite_bools() {
    let mut lua = Lua::new();

    lua.set("a", true);
    lua.set("b", false);

    let x: bool = lua.get("a").unwrap();
    assert_eq!(x, true);

    let y: bool = lua.get("b").unwrap();
    assert_eq!(y, false);
}

#[test]
fn readwrite_strings() {
    let mut lua = Lua::new();

    lua.set("a", "hello");
    lua.set("b", "hello".to_string());

    let x: String = lua.get("a").unwrap();
    assert_eq!(x.as_slice(), "hello");

    let y: String = lua.get("b").unwrap();
    assert_eq!(y.as_slice(), "hello");
}

#[test]
fn i32_to_string() {
    let mut lua = Lua::new();

    lua.set("a", 2);

    let x: String = lua.get("a").unwrap();
    assert_eq!(x.as_slice(), "2");
}

#[test]
fn string_to_i32() {
    let mut lua = Lua::new();

    lua.set("a", "2");
    lua.set("b", "aaa");

    let x: i32 = lua.get("a").unwrap();
    assert_eq!(x, 2);

    let y: Option<i32> = lua.get("b");
    assert!(y.is_none());
}
