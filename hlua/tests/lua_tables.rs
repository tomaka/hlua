extern crate hlua;

use hlua::Lua;
use hlua::LuaTable;

#[test]
fn iterable() {
    let mut lua = Lua::new();

    let _:() = lua.execute("a = { 9, 8, 7 }").unwrap();

    let mut table = lua.get::<LuaTable<_>, _>("a").unwrap();
    let mut counter = 0;

    for (key, value) in table.iter().filter_map(|e| e) {
        let _: u32 = key;
        let _: u32 = value;
        assert_eq!(key + value, 10);
        counter += 1;
    }

    assert_eq!(counter, 3);
}

#[test]
fn iterable_multipletimes() {
    let mut lua = Lua::new();

    let _:() = lua.execute("a = { 9, 8, 7 }").unwrap();

    let mut table = lua.get::<LuaTable<_>, _>("a").unwrap();

    for _ in (0 .. 10) {
        let table_content: Vec<Option<(u32, u32)>> = table.iter().collect();
        assert_eq!(table_content, vec![ Some((1,9)), Some((2,8)), Some((3,7)) ]);
    }
}

#[test]
fn get_set() {
    let mut lua = Lua::new();

    let _:() = lua.execute("a = { 9, 8, 7 }").unwrap();
    let mut table = lua.get::<LuaTable<_>, _>("a").unwrap();

    let x: i32 = table.get(2).unwrap();
    assert_eq!(x, 8);

    table.set(3, "hello");
    let y: String = table.get(3).unwrap();
    assert_eq!(y, "hello");

    let z: i32 = table.get(1).unwrap();
    assert_eq!(z, 9);
}

#[test]
fn table_over_table() {
    let mut lua = Lua::new();

    let _:() = lua.execute("a = { 9, { 8, 7 }, 6 }").unwrap();
    let mut table = lua.get::<LuaTable<_>, _>("a").unwrap();

    let x: i32 = table.get(1).unwrap();
    assert_eq!(x, 9);

    {
        let mut subtable = table.get::<LuaTable<_>, _>(2).unwrap();

        let y: i32 = subtable.get(1).unwrap();
        assert_eq!(y, 8);

        let z: i32 = subtable.get(2).unwrap();
        assert_eq!(z, 7);
    }

    let w: i32 = table.get(3).unwrap();
    assert_eq!(w, 6);
}

#[test]
fn metatable() {
    let mut lua = Lua::new();

    let _:() = lua.execute("a = { 9, 8, 7 }").unwrap();

    {
        let table = lua.get::<LuaTable<_>, _>("a").unwrap();

        let mut metatable = table.get_or_create_metatable();
        fn handler() -> i32 { 5 };
        metatable.set("__add".to_string(), hlua::function(handler));
    }

    let r: i32 = lua.execute("return a + a").unwrap();
    assert_eq!(r, 5);
}
