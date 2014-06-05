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
			fn push_to_lua(self, lua: &Lua) {
				unsafe { liblua::lua_pushinteger(lua.lua, self as i32) }
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
			fn push_to_lua(self, lua: &Lua) {
				unsafe { liblua::lua_pushunsigned(lua.lua, self as u32) }
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
			fn push_to_lua(self, lua: &Lua) {
				unsafe { liblua::lua_pushnumber(lua.lua, self as f64) }
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
	fn push_to_lua(self, lua: &Lua) {
		unsafe {
			liblua::lua_pushstring(lua.lua, self.to_c_str().unwrap())
		}
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

			// TODO: check this transmute, why is from_raw_parts taking a mut ptr?
			let val = std::string::String::from_raw_parts(size as uint, size as uint, std::mem::transmute(cStr));

			Some(val)
		}
	}
}

impl Index for String {
}

/*impl<'a> Pushable for &'a str {
	fn push_to_lua(self, lua: &Lua) {
		String::from_str(self).push_to_lua(lua)
	}
}

impl Readable for ~str {
	fn read_from_lua(lua: &Lua, index: i32) -> Option<~str> {
		let raw: Option<std::string::String> = Readable::read_from_lua(lua, index);
		// TODO: doesn't compile
		match raw {
			None => None,
			Some(s) => Some(box s.as_slice() as ~str)
		}
	}
}

impl Index for ~str {
}*/

