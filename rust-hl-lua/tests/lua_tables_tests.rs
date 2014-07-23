extern crate lua = "rust-hl-lua";
use lua::{Lua, LuaTable};

#[test]
fn iterable() {
    let mut lua = Lua::new();

    let _:() = lua.execute("a = { 9, 8, 7 }").unwrap();

    let mut table = lua.load_table("a").unwrap();
    let mut counter = 0u;

    for (key, value) in table.iter().filter_map(|e| e) {
        let _: uint = key;
        let _: uint = value;
        assert_eq!(key + value, 10);
        counter += 1;
    }

    assert_eq!(counter, 3);
}

#[test]
fn iterable_multipletimes() {
    let mut lua = Lua::new();

    let _:() = lua.execute("a = { 9, 8, 7 }").unwrap();

    let mut table = lua.load_table("a").unwrap();

    for _ in range(0u, 10) {
        let tableContent: Vec<Option<(uint, uint)>> = table.iter().collect();
        assert_eq!(tableContent, vec!( Some((1,9)), Some((2,8)), Some((3,7)) ));
    }
}

#[test]
fn get_set() {
    let mut lua = Lua::new();

    let _:() = lua.execute("a = { 9, 8, 7 }").unwrap();
    let mut table = lua.load_table("a").unwrap();

    let x: int = table.get(2i).unwrap();
    assert_eq!(x, 8);

    table.set(3i, "hello");
    let y: String = table.get(3i).unwrap();
    assert_eq!(y.as_slice(), "hello");

    let z: int = table.get(1i).unwrap();
    assert_eq!(z, 9);
}

#[test]
fn metatable() {
    let mut lua = Lua::new();

    let _:() = lua.execute("a = { 9, 8, 7 }").unwrap();

    {
        let table = lua.load_table("a").unwrap();

        let mut metatable = table.get_or_create_metatable();
        fn handler() -> int { 5 };
        metatable.set("__add".to_string(), handler);
    }

    let r: int = lua.execute("return a + a").unwrap();
    assert_eq!(r, 5);
}
