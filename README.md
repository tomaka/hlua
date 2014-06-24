## rust-hl-lua

This library is a high-level binding for Lua 5.2. You don't have access to the Lua stack, all you can do is read and write variables.

[![Build Status](https://travis-ci.org/Tomaka17/rust-hl-lua.svg?branch=master)](https://travis-ci.org/Tomaka17/rust-hl-lua)

*Important*: the library is broken for the moment because of a bug in the rust compiler (see https://github.com/mozilla/rust/issues/13853 and https://github.com/mozilla/rust/issues/14377)

### How to install it?

Add this to the `Cargo.toml` file of your project

    [dependencies.rust-hl-lua]
    git = "https://github.com/Tomaka17/rust-hl-lua"

If you don't use cargo yet, just compile with `rustc src/lib.rs`. You can also generate the docs with `rustdoc src/lib.rs`.

In the future, this library will directly include the Lua C library if cargo allows this.

### How to use it?

    extern crate rust-hl-lua;
    use rust-hl-lua::Lua;

The `Lua` struct is the main element of this library. It represents a context in which you can execute Lua code.

    let mut lua = Lua::new();     // mutable is mandatory

#### Reading and writing variables

    lua.set("x", 2);
    lua.execute("x = x + 1").unwrap();
    let x: int = lua.get("x").unwrap();  // x is equal to 3

Reading and writing global variables of the Lua context can be done with `set` and `get`.
The `get` function returns an `Option<T>` 

The base types that can be read and written are: `int`, `i8`, `i16`, `i32`, `uint`, `u8`, `u16`, `u32`, `f32`, `f64`, `bool`, `String`.

If you wish so, you can also add other types by implementing the `Pushable` and `Readable` traits.

#### Executing Lua

    let x: uint = lua.execute("return 6 * 2;").unwrap();    // equals 12

The `execute` function returns a `Result<Readable, ExecutionError>`.

#### Writing functions

    fn add(a: int, b: int) -> int {
        a + b
    }
    
    lua.set("add", add);
    lua.execute("local c = add(2, 4)");
    lua.get("c").unwrap();  // return 6
    
In Lua, functions are exactly like regular variables.

### Roadmap

 - [ ] Reading/writing from a Lua table
 - [ ] Reading/writing containers
 - [ ] Iterating through Lua tables
 - [ ] Reading or loading a function and calling it later
 - [ ] Support for static closures
 - [ ] Access to the metatables of objects
 - [ ] Access to the registry
