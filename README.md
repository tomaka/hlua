## rust-lua

This library is a high-level binding for Lua. You don't have access to the Lua stack, all you can do is read and write variables.

### How to compile?

    rustc src/lib.rs

### How to use it?

    let mut lua = Lua::new();     // mutable is mandatory
    lua.set("x", 2);
    lua.execute("x = x + 1");
    let x = lua.get("x").unwrap();  // return 3

### Documentation

#### Reading and writing variables

    let mut lua = Lua::new();
    lua.set("x", 5);
    lua.execute("x = x + 2;");
    let x = lua.get("x").unwrap();

Reading and writing global variables of the Lua context can be done with `set` and `get`.
The `get` function returns an `Option<T>` 

The types that can be read and written are: `int`, `std::string::String`, ... (TODO)
