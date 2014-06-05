extern crate libc;
extern crate std;
use super::liblua;
use super::Index;
use super::Lua;
use super::Pushable;
use super::Readable;

impl Pushable for i32 {
	fn push_to_lua(self, lua: &Lua) {
		unsafe { liblua::lua_pushinteger(lua.lua, self) }
	}
}

impl Readable for i32 {
	fn read_from_lua(lua: &Lua, index: i32) -> Option<i32> {
		unsafe {
			let mut success: libc::c_int = std::mem::uninitialized();
			let val = liblua::lua_tointegerx(lua.lua, index, std::mem::transmute(&success));
			match success {
				0 => None,
				_ => Some(val)
			}
		}
	}
}

impl Index for i32 {
}

impl Pushable for std::string::String {
	fn push_to_lua(self, lua: &Lua) {
		unsafe {
			liblua::lua_pushstring(lua.lua, self.to_c_str().unwrap())
		}
	}
}

impl Readable for std::string::String {
	fn read_from_lua(lua: &Lua, index: i32) -> Option<std::string::String> {
		unsafe {
			let mut size: libc::size_t = std::mem::uninitialized();
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

impl Index for std::string::String {
}

/*impl<'a> Pushable for &'a str {
	fn push_to_lua(self, lua: &Lua) {
		std::string::String::from_str(self).push_to_lua(lua)
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

impl<T:Pushable> Pushable for std::vec::Vec<T> {
	fn push_to_lua(self, lua: &Lua) {
		unimplemented!()
	}
}

impl<'a, TRetValue> Pushable for ||:'a -> TRetValue {
	fn push_to_lua(self, lua: &Lua) {
		//unsafe { liblua::lua_pushinteger(lua.lua, self) }
	}
}

impl<TRetValue: Pushable> Pushable for fn() -> TRetValue {
	fn push_to_lua(self, lua: &Lua) {
		extern "C" fn wrapper(state: *liblua::lua_State) {

		}
	}
}

impl<TParam1: Readable, TRetValue: Pushable> Pushable for fn(TParam1) -> TRetValue {
	fn push_to_lua(self, lua: &Lua) {
		//unsafe { liblua::lua_pushinteger(lua.lua, self) }
	}
}

