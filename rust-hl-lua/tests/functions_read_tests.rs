extern crate lua = "rust-hl-lua";

#[test]
fn basic() {
    let mut lua = lua::Lua::new();

    let mut f = lua::LuaFunction::load(&mut lua, "return 5;").unwrap();

    let val: int = f.call().unwrap();
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

    let val: Result<int, lua::LuaError> = f.call();
    assert!(val.is_err());
}
