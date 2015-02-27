extern crate "rust-hl-lua" as lua;

#[test]
fn basic() {
    let mut lua = lua::Lua::new();

    let mut f = lua::LuaFunction::load(&mut lua, "return 5;").unwrap();

    let val: i32 = f.call().unwrap();
    assert_eq!(val, 5);
}

#[test]
fn syntax_error() {
    let mut lua = lua::Lua::new();

    assert!(lua::LuaFunction::load(&mut lua, "azerazer").is_err());
}

#[test]
fn execution_error() {
    let mut lua = lua::Lua::new();

    let mut f = lua::LuaFunction::load(&mut lua, "return a:hello()").unwrap();

    let val: Result<i32, lua::LuaError> = f.call();
    assert!(val.is_err());
}
