extern crate hlua;

use hlua::Lua;

fn main() {
    let mut lua = Lua::new();

    // defining the API of "myLib"
    lua.set("myLib", &[
        ("add", add),
        ("sub", sub),
        ("mul", mul)
    ]);


    // redirecting calls from stdin to Lua
    // try typing "return myLib.add(12, 3)"
    for line in std::io::stdin().lines() {
        match lua.execute(line.unwrap().as_slice()) {
            Err(err) => println!("{}", err),
            Ok(a) => { let _:int = a; println!("{}", a) }
        }
    }

}

// callback for "myLib.add"
fn add(a: int, b: int) -> int {
    a + b
}

// callback for "myLib.sub"
fn sub(a: int, b: int) -> int {
    a - b
}

// callback for "myLib.mul"
fn mul(a: int, b: int) -> int {
    a * b
}
