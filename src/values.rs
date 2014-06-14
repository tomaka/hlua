extern crate libc;
extern crate std;

use super::liblua;
use super::Index;
use super::Lua;
use super::Pushable;
use super::Readable;

macro_rules! integer_impl(
    ($t:ident) => (
        impl Pushable for $t {
            fn push_to_lua(&self, lua: &Lua) {
                unsafe { liblua::lua_pushinteger(lua.lua, *self as liblua::lua_Integer) }
            }
        }
        impl Readable for $t {
            fn read_from_lua(lua: &Lua, index: i32) -> Option<$t> {
                unsafe {
                    let success: libc::c_int = std::mem::uninitialized();
                    let val = liblua::lua_tointegerx(lua.lua, index, std::mem::transmute(&success));
                    match success {
                        0 => None,
                        _ => Some(val as $t)
                    }
                }
            }
        }
        impl Index for $t {
        }
    );
)

integer_impl!(int)
integer_impl!(i8)
integer_impl!(i16)
integer_impl!(i32)
//integer_impl!(i64)   // data loss

macro_rules! unsigned_impl(
    ($t:ident) => (
        impl Pushable for $t {
            fn push_to_lua(&self, lua: &Lua) {
                unsafe { liblua::lua_pushunsigned(lua.lua, *self as liblua::lua_Unsigned) }
            }
        }
        impl Readable for $t {
            fn read_from_lua(lua: &Lua, index: i32) -> Option<$t> {
                unsafe {
                    let success: libc::c_int = std::mem::uninitialized();
                    let val = liblua::lua_tounsignedx(lua.lua, index, std::mem::transmute(&success));
                    match success {
                        0 => None,
                        _ => Some(val as $t)
                    }
                }
            }
        }
        impl Index for $t {
        }
    );
)

unsigned_impl!(uint)
unsigned_impl!(u8)
unsigned_impl!(u16)
unsigned_impl!(u32)
//unsigned_impl!(u64)   // data loss

macro_rules! numeric_impl(
    ($t:ident) => (
        impl Pushable for $t {
            fn push_to_lua(&self, lua: &Lua) {
                unsafe { liblua::lua_pushnumber(lua.lua, *self as f64) }
            }
        }
        impl Readable for $t {
            fn read_from_lua(lua: &Lua, index: i32) -> Option<$t> {
                unsafe {
                    let success: libc::c_int = std::mem::uninitialized();
                    let val = liblua::lua_tonumberx(lua.lua, index, std::mem::transmute(&success));
                    match success {
                        0 => None,
                        _ => Some(val as $t)
                    }
                }
            }
        }
        impl Index for $t {
        }
    );
)

numeric_impl!(f32)
numeric_impl!(f64)

impl Pushable for std::string::String {
    fn push_to_lua(&self, lua: &Lua) {
        unsafe { liblua::lua_pushstring(lua.lua, self.to_c_str().unwrap()) }
    }
}

impl Readable for String {
    fn read_from_lua(lua: &Lua, index: i32) -> Option<std::string::String> {
        unsafe {
            let size: libc::size_t = std::mem::uninitialized();
            let cStr = liblua::lua_tolstring(lua.lua, index, std::mem::transmute(&size));
            if cStr.is_null() {
                return None;
            }

            // I can't manage to make this compile properly, so we just transmute into a static slice
            let slice = std::slice::raw::buf_as_slice(cStr as *u8, size as uint, |buf| { let b:&'static [u8] = std::mem::transmute(buf); b });
            String::from_utf8(Vec::from_slice(slice)).ok()
        }
    }
}

impl Index for String {
}

impl<'a> Pushable for &'a str {
    fn push_to_lua(&self, lua: &Lua) {
        unsafe {
            liblua::lua_pushstring(lua.lua, self.to_c_str().unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn readwrite_ints() {
        let mut lua = super::super::Lua::new();

        lua.set("a", 2).unwrap();
        let x: int = lua.get("a").unwrap();
        assert_eq!(x, 2);
    }
}
