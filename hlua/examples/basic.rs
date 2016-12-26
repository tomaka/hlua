extern crate hlua;

// To see the generated assembly, run:
// cargo rustc --release --example basic -- --emit=asm

fn main() {
    let mut lua = hlua::Lua::new();

    lua.set("a", 12);
    let val: i32 = lua.execute(r#"return a * 5;"#).unwrap();

    assert_eq!(val, 60);
}
