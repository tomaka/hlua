#![crate_type = "dylib"]
#![feature(phase)]

#[phase(plugin)]
extern crate rust_hl_lua_module;

extern crate libc;


lua_module!("mylib")
