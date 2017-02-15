extern crate hlua;

use hlua::AnyLuaValue;

use std::io::prelude::*;
use std::io::{stdin, stdout};

fn main() {
    let mut lua = hlua::Lua::new();
    lua.openlibs();

    let stdin = stdin();
    loop {
        print!("> ");
        stdout().flush().unwrap();

        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();

        match lua.execute::<AnyLuaValue>(&line) {
            Ok(value) => println!("{:?}", value),
            Err(e) => println!("error: {:?}", e),
        }
    }
}
