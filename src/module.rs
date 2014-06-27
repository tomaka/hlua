#![crate_id = "rust_hl_lua_module"]
#![crate_type = "dylib"]
#![comment = "Lua bindings for Rust, module library"]
#![license = "MIT"]
#![allow(visible_private_types)]
#![feature(macro_rules, plugin_registrar, quote)]

extern crate libc;
extern crate std;
extern crate rust_hl_lua;

#[macro_export]
macro_rules! lua_module(
    ($name:expr) => (
        mod ffi {
            pub struct lua_State;
            #[link(name = "lua5.2")]
            extern {
                pub fn lua_createtable(L: *lua_State, narr: ::libc::c_int, nrec: ::libc::c_int);
            }
        }
        
        #[no_mangle]
        extern "C" fn luaopen_mylib(lua: *ffi::lua_State) -> ::libc::c_int {
            unsafe { ffi::lua_createtable(lua, 0, 0); }
            1
        }
    )
)
