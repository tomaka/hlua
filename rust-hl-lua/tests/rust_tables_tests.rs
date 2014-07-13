extern crate lua = "rust-hl-lua";
use lua::{Lua, LuaTable};

#[test]
fn write() {
    let mut lua = Lua::new();

    lua.set("a", vec!(9i, 8, 7));

    let mut table: LuaTable = lua.load("a").unwrap();

    let values: Vec<(int,int)> = table.iter().filter_map(|e| e).collect();
    assert_eq!(values, vec!( (1, 9), (2, 8), (3, 7) ));
}
