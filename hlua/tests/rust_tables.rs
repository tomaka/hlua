extern crate "hlua" as lua;
extern crate test;
use lua::{Lua, LuaTable};

use std::collections::{HashMap, HashSet};
use test::Bencher;

#[test]
fn write() {
    let mut lua = Lua::new();

    lua.set("a", vec![9, 8, 7]);

    let mut table: LuaTable<_> = lua.get("a").unwrap();

    let values: Vec<(i32, i32)> = table.iter().filter_map(|e| e).collect();
    assert_eq!(values, vec!( (1, 9), (2, 8), (3, 7) ));
}

#[test]
fn write_map() {
    let mut lua = Lua::new();

    let mut map = HashMap::new();
    map.insert(5, 8);
    map.insert(13, 21);
    map.insert(34, 55);

    lua.set("a", map.clone());

    let mut table: LuaTable<_> = lua.get("a").unwrap();

    let values: HashMap<i32, i32> = table.iter().filter_map(|e| e).collect();
    assert_eq!(values, map);
}

#[test]
fn write_set() {
    let mut lua = Lua::new();

    let mut set = HashSet::new();
    set.insert(5);
    set.insert(8);
    set.insert(13);
    set.insert(21);
    set.insert(34);
    set.insert(55);

    lua.set("a", set.clone());

    let mut table: LuaTable<_> = lua.get("a").unwrap();

    let values: HashSet<i32> = table.iter().filter_map(|e| e)
                                           .map(|(elem, set): (i32, bool)| {
        assert!(set);
        elem
    }).collect();

    assert_eq!(values, set);
}

#[bench]
fn new_map(b: &mut Bencher) {
    let mut lua = Lua::new();

    let mut map = HashMap::new();
    map.insert(5, 8);
    map.insert(13, 21);
    map.insert(34, 55);

    b.iter(|| {
        lua.set("a", map.clone());
    })
}

#[bench]
fn new_large_map(b: &mut Bencher) {
    let mut lua = Lua::new();

    let mut map = HashMap::new();
    for i in range(0, 500) {
        map.insert(i, i + 1);
    }

    b.iter(|| {
        lua.set("a", map.clone());
    })
}
