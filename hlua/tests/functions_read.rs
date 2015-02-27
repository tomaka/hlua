extern crate hlua;

#[test]
fn basic() {
    let mut lua = hlua::Lua::new();

    let mut f = hlua::LuaFunction::load(&mut lua, "return 5;").unwrap();

    let val: i32 = f.call().unwrap();
    assert_eq!(val, 5);
}

#[test]
fn syntax_error() {
    let mut lua = hlua::Lua::new();

    assert!(hlua::LuaFunction::load(&mut lua, "azerazer").is_err());
}

#[test]
fn execution_error() {
    let mut lua = hlua::Lua::new();

    let mut f = hlua::LuaFunction::load(&mut lua, "return a:hello()").unwrap();

    let val: Result<i32, hlua::LuaError> = f.call();
    assert!(val.is_err());
}
