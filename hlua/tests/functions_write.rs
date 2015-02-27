extern crate "hlua" as lua;

#[test]
fn simple_function() {
    let mut lua = lua::Lua::new();

    fn ret5() -> i32 { 5 };
    lua.set("ret5", ret5);

    let val: i32 = lua.execute("return ret5()").unwrap();
    assert_eq!(val, 5);
}

#[test]
fn one_argument() {
    let mut lua = lua::Lua::new();

    fn plus_one(val: i32) -> i32 { val + 1 };
    lua.set("plus_one", plus_one);

    let val: i32 = lua.execute("return plus_one(3)").unwrap();
    assert_eq!(val, 4);
}

#[test]
fn two_arguments() {
    let mut lua = lua::Lua::new();

    fn add(val1: i32, val2: i32) -> i32 { val1 + val2 };
    lua.set("add", add);

    let val: i32 = lua.execute("return add(3, 7)").unwrap();
    assert_eq!(val, 10);
}

#[test]
fn wrong_arguments_types() {
    let mut lua = lua::Lua::new();

    fn add(val1: i32, val2: i32) -> i32 { val1 + val2 };
    lua.set("add", add);

    match lua.execute::<i32>("return add(3, \"hello\")") {
        Err(lua::LuaError::ExecutionError(_)) => (),
        _ => panic!()
    }
}

#[test]
fn return_result() {
    let mut lua = lua::Lua::new();

    fn always_fails() -> Result<i32, &'static str> { Err("oops, problem") };
    lua.set("always_fails", always_fails);

    match lua.execute::<()>("always_fails()") {
        Err(lua::LuaError::ExecutionError(_)) => (),
        _ => panic!()
    }
}

#[test]
fn closures() {
    let mut lua = lua::Lua::new();

    lua.set("add", |a:i32, b:i32| a + b);
    lua.set("sub", |a:i32, b:i32| a - b);

    let val1: i32 = lua.execute("return add(3, 7)").unwrap();
    assert_eq!(val1, 10);

    let val2: i32 = lua.execute("return sub(5, 2)").unwrap();
    assert_eq!(val2, 3);
}

#[test]
fn closures_lifetime() {
    fn t(f: |i32,i32|->i32) {
        let mut lua = lua::Lua::new();

        lua.set("add", f);

        let val1: i32 = lua.execute("return add(3, 7)").unwrap();
        assert_eq!(val1, 10);
    }

    t(|a,b| a+b);
}

#[test]
fn closures_extern_access() {
    let mut a = 5i;

    {
        let mut lua = lua::Lua::new();

        lua.set("inc", || a += 1);
        for _ in range(0i, 15) {
            lua.execute::<()>("inc()").unwrap();
        }
    }

    assert_eq!(a, 20)
}
