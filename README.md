## rust-hl-lua

This library is a high-level binding for Lua 5.2. You don't have access to the Lua stack, all you can do is read/write variables (including callbacks) and execute Lua code.

[![Build Status](https://travis-ci.org/tomaka/rust-hl-lua.svg?branch=master)](https://travis-ci.org/tomaka/rust-hl-lua)

### How to install it?

Add this to the `Cargo.toml` file of your project

```toml
[dependencies.rust-hl-lua]
git = "https://github.com/tomaka/rust-hl-lua"
```

In the future, this library will directly include the Lua C library if cargo allows this.

### How to use it?

```rust
extern crate lua = "rust-hl-lua";
use lua::Lua;
```

The `Lua` struct is the main element of this library. It represents a context in which you can execute Lua code.

```rust
let mut lua = Lua::new();     // mutable is mandatory
```

[You can check the documentation here](http://rust-ci.org/tomaka/rust-hl-lua/doc/rust-hl-lua/).

#### Reading and writing variables

```rust
lua.set("x", 2);
lua.execute("x = x + 1").unwrap();
let x: int = lua.get("x").unwrap();  // x is equal to 3
```

Reading and writing global variables of the Lua context can be done with `set` and `get`.
The `get` function returns an `Option<T>` and does a copy of the value.

The base types that can be read and written are: `int`, `i8`, `i16`, `i32`, `uint`, `u8`, `u16`, `u32`, `f32`, `f64`, `bool`, `String`.

If you wish so, you can also add other types by implementing the `Push` and `CopyRead`/`ConsumeRead` traits.

#### Executing Lua

```rust
let x: uint = lua.execute("return 6 * 2;").unwrap();    // equals 12
```

The `execute` function takes a `&str` and returns a `Result<CopyRead, ExecutionError>`.

You can also call `execute_from_reader` which takes a `std::io::Reader` as parameter.
For example you can easily execute the content of a file like this:

```rust
lua.execute_from_reader(File::open(&Path::new("script.lua")).unwrap())
```

#### Writing functions

```rust
fn add(a: int, b: int) -> int {
    a + b
}

lua.set("add", add);
lua.execute("local c = add(2, 4)");
lua.get("c").unwrap();  // return 6
```
    
In Lua, functions are exactly like regular variables.

You can write regular functions as well as closures:

```rust
lua.set("mul", |a:int,b:int| a*b);
```

Note that the lifetime of the Lua context must be equal to or shorter than the lifetime of closures. This is enforced at compile-time.

```rust
let mut a = 5i;

{
    let mut lua = Lua::new();

    lua.set("inc", || a += 1);
    for i in range(0i, 15) {
        lua.execute::<()>("inc()").unwrap();
    }
} // unborrows `a`

assert_eq!(a, 20)
```

##### Error handling

If your Rust function returns a `Result` object which contains an error, then a Lua error will be triggered.

#### Manipulating Lua tables

Manipulating a Lua table can be done by reading a `LuaTable` object. This can be achieved easily by calling `load_table`.

```rust
let mut table = lua.load_table("a").unwrap();
```

You can then iterate through the table with the `.iter()` function. Note that the value returned by the iterator is an `Option<(Key, Value)>`, the `Option` being empty when either the key or the value is not convertible to the requested type. The `filter_map` function (provided by the standard `Iterator` trait) is very useful when dealing with this.

```rust
for (key, value) in table.iter().filter_map(|e| e) {
    ...
}
```

You can also retreive and modify individual indices:

```rust
let x = table.get("a").unwrap();
table.set("b", "hello");
```

#### Calling Lua functions

You can call Lua functions by reading a `functions_read::LuaFunction`.

```rust
lua.execute("
    function get_five() 
        return 5
    end");

let get_five: functions_read::LuaFunction = lua.load("get_five").unwrap();
let value: int = get_five().unwrap();
assert_eq!(value, 5);
```

This object holds a mutable reference of `Lua`, so you can't read or modify anything in the Lua context while the `get_five` variable exists.
It is not possible to store the function for the moment, but it may be in the future.

#### Reading and writing Rust containers

*(note: not yet possible to read containers)*

It is possible to read and write whole Rust containers at once:

```rust```
lua.set("a", [ 12, 13, 14, 15 ]);
```

If the container has single elements, then the indices will be numerical. For example in the code above, the `12` will be at index `1`, the `13` at index `2`, etc.

If the container has tuples of two elements, then the first one will be considered as the key and the second one as the value.

This can be useful to create APIs:

```rust
fn foo() { }
fn bar() { }

lua.set("mylib", [
    ("foo", foo),
    ("bar", bar)
]);

lua.execute("mylib.foo()");
```

#### User data

**(note: the API here is very unstable for the moment)**

When you expose functions to Lua, you may wish to read or write more elaborate objects. This is called a **user data**.

To do so, you should implement the `Push`, `CopyRead` and `ConsumeRead` for your types.
This is usually done by redirecting the call to `userdata::push_userdata`.

```rust
struct Foo;

impl<'a> lua::Push<'a> for Foo {
    fn push_to_lua(self, lua: &mut lua::Lua<'a>) -> uint {
        lua::userdata::push_userdata(self, lua,
            |metatable| {
                // you can define all the member functions of Foo here
                // see the official Lua documentation for metatables
                metatable.set("__call", || println!("hello from foo"))
            })
    }
}

fn main() {
    let mut lua = lua::Lua::new();
    lua.set("foo", Foo);
    lua.execute("foo()");       // prints "hello from foo"
}
```

### Creating a Lua module

This library also includes a second library named `rust-hl-lua-module` which allows you to create Lua modules in Rust.

To use it, add this to `Cargo.toml`:

```toml
[dependencies.rust-hl-lua-modules]
git = "https://github.com/tomaka/rust-hl-lua"
```

Then you can use it like this:

```rust
#![feature(phase)]

#[phase(plugin)]
extern crate lua_mod = "rust-hl-lua-modules";

#[export_lua_module]
pub mod mylib {         // <-- must be the name of the Lua module
    static PI: f32 = 3.141592;

    fn function1(a: int, b: int) -> int {
        a + b
    }

    fn function2(a: int) -> int {
        a + 5
    }

    #[lua_module_init]
    fn init() {
        println!("module initialized!")
    }
}
```

This module will then be usable by Lua:

```lua
> mylib = require("mylib")
module initialized!
> return mylib.function1(2, 4)
6
> return mylib.PI
3.141592
```

Two syntax extensions are defined:
 - `#[export_lua_module]`: Must be put in front of a module. The name of the module must be the same as the name of your Lua module.
 - `#[lua_module_init]`: Can be put in front of a function inside the module. This function will be executed when the module is loaded.

### Roadmap

 - [ ] Reading/writing inside Lua tables
   - [x] Iterating through tables
   - [x] Reading elements by value
   - [ ] Reading functions and sub-tables
 - [ ] Reading/writing containers
   - [x] Vectors and slices
   - [ ] HashMaps and HashSets
 - [x] Writing functions
   - [x] Basic support
   - [x] Functions with parameters
   - [x] Closures
 - [ ] Reading or loading a function and calling it later
   - [x] Basic support
   - [ ] Passing parameters
 - [ ] Access to the metatables of tables and user data
 - [ ] Access to the registry
 - [ ] Better user data
 - [x] Allow writing Lua binary modules in Rust
