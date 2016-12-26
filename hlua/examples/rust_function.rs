extern crate hlua;

// To see the generated assembly, run:
// cargo rustc --release --example rust_function -- --emit=asm

fn main() {
    let mut lua = hlua::Lua::new();

    lua.set("foo", hlua::function1(|val: i32| val * 5));

    let val: i32 = lua.execute(r#"return foo(8)"#).unwrap();
    assert_eq!(val, 40);
}
