## rust-lua

This library is a high-level binding for Lua. You don't have access to the Lua stack, all you can do is read and write variables.

### How to compile?

Compile:

    rustc src/lib.rs

Build docs:
    
    rustdoc src/lib.rs

### How to use it?

#### Reading and writing variables

    let mut lua = Lua::new();     // mutable is mandatory
    lua.set("x", 2);
    lua.execute("x = x + 1");
    let x = lua.get("x").unwrap();  // x is equal to 3

Reading and writing global variables of the Lua context can be done with `set` and `get`.
The `get` function returns an `Option<T>` 

The types that can be read and written are: `int`, `std::string::String`, ... (TODO)

#### Writing functions

    fn add(a: int, b: int) -> int {
        a + b
    }

    lua.set("add", &add);
    lua.execute("local c = add(2, 4)");
    lua.get("c").unwrap();  // return 6
