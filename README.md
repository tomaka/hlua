## hlua

This library is a high-level binding for Lua 5.2. You don't have access to the Lua stack, all you can do is read/write variables (including callbacks) and execute Lua code.

[![Build Status](https://travis-ci.org/tomaka/hlua.svg?branch=master)](https://travis-ci.org/tomaka/hlua)

### How to install it?

Add this to the `Cargo.toml` file of your project

```toml
[dependencies]
hlua = "0.3"
```

### How to use it?

```rust
extern crate hlua;
use hlua::Lua;
```

The `Lua` struct is the main element of this library. It represents a context in which you can execute Lua code.

```rust
let mut lua = Lua::new();     // mutable is mandatory
```

**[You can check the documentation here](http://docs.rs/hlua)**.

#### Reading and writing variables

```rust
lua.set("x", 2);
lua.execute::<()>("x = x + 1").unwrap();
let x: i32 = lua.get("x").unwrap();  // x is equal to 3
```

Reading and writing global variables of the Lua context can be done with `set` and `get`.
The `get` function returns an `Option<T>` and does a copy of the value.

The base types that can be read and written are: `i8`, `i16`, `i32`, `u8`, `u16`, `u32`, `f32`, `f64`, `bool`, `String`. `&str` can be written but not read.

If you wish so, you can also add other types by implementing the `Push` and `LuaRead` traits.

#### Executing Lua

```rust
let x: u32 = lua.execute("return 6 * 2;").unwrap();    // equals 12
```

The `execute` function takes a `&str` and returns a `Result<T, ExecutionError>` where `T: LuaRead`.

You can also call `execute_from_reader` which takes a `std::io::Read` as parameter.
For example you can easily execute the content of a file like this:

```rust
lua.execute_from_reader::<()>(File::open(&Path::new("script.lua")).unwrap())
```

#### Writing functions

In order to write a function, you must wrap it around `hlua::functionX` where `X` is the number of parameters. This is for the moment a limitation of Rust's inferrence system.

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

lua.set("add", hlua::function2(add));
lua.execute::<()>("local c = add(2, 4)");   // calls the `add` function above
let c: i32 = lua.get("c").unwrap();   // returns 6
```

In Lua, functions are exactly like regular variables.

You can write regular functions as well as closures:

```rust
lua.set("mul", hlua::function2(|a: i32, b: i32| a * b));
```

Note that the lifetime of the Lua context must be equal to or shorter than the lifetime of closures. This is enforced at compile-time.

```rust
let mut a = 5i;

{
    let mut lua = Lua::new();

    lua.set("inc", || a += 1);    // borrows 'a'
    for i in (0 .. 15) {
        lua.execute::<()>("inc()").unwrap();
    }
} // unborrows `a`

assert_eq!(a, 20)
```

##### Error handling

If your Rust function returns a `Result` object which contains an error, then a Lua error will be triggered.

#### Manipulating Lua tables

Manipulating a Lua table can be done by reading a `LuaTable` object. This can be achieved easily by reading a `LuaTable` object.

```rust
let mut table: hlua::LuaTable<_> = lua.get("a").unwrap();
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
lua.execute::<()>("
    function get_five() 
        return 5
    end");

let get_five: hlua::LuaFunction<_> = lua.get("get_five").unwrap();
let value: i32 = get_five.call().unwrap();
assert_eq!(value, 5);
```

This object holds a mutable reference of `Lua`, so you can't read or modify anything in the Lua context while the `get_five` variable exists.
It is not possible to store the function for the moment, but it may be in the future.

#### Reading and writing Rust containers

*(note: not yet possible to read all containers, see below)*

It is possible to read and write whole Rust containers at once:

```rust
lua.set("a", [ 12, 13, 14, 15 ]);
let hashmap: HashMap<i32, f64> = [1., 2., 3.].into_iter().enumerate().map(|(k, v)| (k as i32, *v as f64)).collect();
lua.set("v", hashmap);
```

If the container has single elements, then the indices will be numerical. For example in the code above, the `12` will be at index `1`, the `13` at index `2`, etc.

If the container has tuples of two elements, then the first one will be considered as the key and the second one as the value.

This can be useful to create APIs:

```rust
fn foo() { }
fn bar() { }

lua.set("mylib", [
    ("foo", hlua::function0(foo)),
    ("bar", hlua::function0(bar))
]);

lua.execute::<()>("mylib.foo()");
```

It is possible to read a `Vec<AnyLuaValue>`:

```rust
        let mut lua = Lua::new();

        lua.execute::<()>(r#"v = { 1, 2, 3 }"#).unwrap();

        let read: Vec<_> = lua.get("v").unwrap();
        assert_eq!(
            read,
            [1., 2., 3.].iter()
                .map(|x| AnyLuaValue::LuaNumber(*x)).collect::<Vec<_>>());
```

In case table represents sparse array, has non-numeric keys, or
indices not starting at 1, `.get()` will return `None`, as Rust's
`Vec` doesn't support these features.

It is possible to read a `HashMap<AnyHashableLuaValue, AnyLuaValue>`:

```rust
let mut lua = Lua::new();

lua.execute::<()>(r#"v = { [-1] = -1, ["foo"] = 2, [2.] = 42 }"#).unwrap();

let read: HashMap<_, _> = lua.get("v").unwrap();
assert_eq!(read[&AnyHashableLuaValue::LuaNumber(-1)], AnyLuaValue::LuaNumber(-1.));
assert_eq!(read[&AnyHashableLuaValue::LuaString("foo".to_owned())], AnyLuaValue::LuaNumber(2.));
assert_eq!(read[&AnyHashableLuaValue::LuaNumber(2)], AnyLuaValue::LuaNumber(42.));
assert_eq!(read.len(), 3);
```

#### User data

**(note: the API here is very unstable for the moment)**

When you expose functions to Lua, you may wish to read or write more elaborate objects. This is called a **user data**.

To do so, you should implement the `Push`, `CopyRead` and `ConsumeRead` for your types.
This is usually done by redirecting the call to `userdata::push_userdata`.

```rust
struct Foo;

impl<L> hlua::Push<L> for Foo where L: hlua::AsMutLua<'lua> {
    fn push_to_lua(self, lua: L) -> hlua::PushGuard<L> {
        lua::userdata::push_userdata(self, lua,
            |mut metatable| {
                // you can define all the member functions of Foo here
                // see the official Lua documentation for metatables
                metatable.set("__call", hlua::function0(|| println!("hello from foo")))
            })
    }
}

fn main() {
    let mut lua = lua::Lua::new();
    lua.set("foo", Foo);
    lua.execute::<()>("foo()");       // prints "hello from foo"
}
```

### Creating a Lua module

**Note: OBSOLETE ; this is still some pre-Rust-1.0 stuff**

This library also includes a second library named `rust-hl-lua-module` which allows you to create Lua modules in Rust.

To use it, add this to `Cargo.toml`:

```toml
[dependencies.rust-hl-lua-modules]
git = "https://github.com/tomaka/hlua"
```

Then you can use it like this:

```rust
#![feature(phase)]
#[!plugin(rust-hl-lua-modules)]

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

**Restrictions**: 
 - `fail!()` will crash the program.
 - If you spawn tasks, they will have to end before the hand is given back to lua.

### Contributing

Contributions are welcome!
