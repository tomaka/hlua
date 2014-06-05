#![crate_id = "lua"]
#![crate_type = "lib"]
#![comment = "Lua bindings for Rust"]
#![license = "MIT"]
#![allow(visible_private_types)]

extern crate libc;
extern crate std;

mod liblua;
pub mod value;

/**
 * Main object of the library
 */
pub struct Lua {
	lua: *mut liblua::lua_State
}

/**
 * Object which allows access to a Lua variable
 */
pub struct VariableAccessor<'a, TVariableLocation> {
	lua: &'a mut Lua,
	location: TVariableLocation
}

/**
 * Should be implemented by whatever type is pushable on the Lua stack
 */
pub trait Pushable {
	fn push_to_lua(self, &Lua);
}

/**
 * Should be implemented by whatever type can be read from the Lua stack
 */
pub trait Readable {
	/**
	 * # Arguments
	 *  * `lua` - The Lua object to read from
	 *  * `index` - The index on the stack to read from
	 */
	fn read_from_lua(lua: &Lua, index: i32) -> Option<Self>;
}

/**
 * Types that can be indices in Lua tables
 */
pub trait Index: Pushable + Readable {
}

/**
 * Object which can store variables
 */
trait Dropbox<TIndex: Index, TPushable: Pushable> {
	fn store(&self, &Lua, &TIndex, TPushable);
}

/**
 * Object which you can read variables from
 */
trait Readbox<TIndex: Index, TReadable: Readable> {
	fn read(&self, &Lua, &TIndex) -> Option<TReadable>;
}

struct VariableLocation<'a, TIndex, TPrev> {
	index: &'a TIndex,
	prev: TPrev
}

/**
 * Represents the global variables
 */
pub struct Globals;


extern "C" fn alloc(_ud: *mut libc::c_void, ptr: *mut libc::c_void, _osize: libc::size_t, nsize: libc::size_t) -> *mut libc::c_void {
    unsafe {
        if nsize == 0 {
            libc::free(ptr as *mut libc::c_void);
            std::ptr::mut_null()
        } else {
            libc::realloc(ptr, nsize)
        }
    }
}

impl Lua {
	/**
	 * Builds a new Lua context
	 * # Failure
	 * The function fails if lua_newstate fails (which indicates lack of memory)
	 */
	pub fn new() -> Lua {
		let lua = unsafe { liblua::lua_newstate(alloc, std::ptr::mut_null()) };
		if lua.is_null() {
			fail!("lua_newstate failed");
		}

		Lua { lua: lua }
	}

	/**
	 * Executes some Lua code on the context
	 */
	pub fn execute<T: Readable>(&mut self, code: &std::string::String) -> T {
		unimplemented!()
	}

	pub fn access<'a, 'b, TIndex: Index>(&'a mut self, index: &'b TIndex) -> VariableAccessor<'a, VariableLocation<'b, TIndex, Globals>> {
		VariableAccessor {
			lua: self,
			location: VariableLocation { index: index, prev: Globals }
		}
	}
}

impl<'a, 'b, TIndex: Index, TValue: Pushable, TDropbox: Dropbox<TIndex, TValue>> VariableAccessor<'a, VariableLocation<'b, TIndex, TDropbox>> {
	pub fn set(&mut self, value: TValue) {
		self.location.store(self.lua, value)
	}
}

impl<'a, 'b, TIndex: Index, TValue: Readable, TReadbox: Readbox<TIndex, TValue>> VariableAccessor<'a, VariableLocation<'b, TIndex, TReadbox>> {
	pub fn get(&self) -> Option<TValue> {
		self.location.read(self.lua)
	}
}

impl<'a, TIndex: Index, TValue: Pushable, TDropbox: Dropbox<TIndex, TValue>> VariableLocation<'a, TIndex, TDropbox> {
	fn store(&self, lua: &Lua, value: TValue) {
		self.prev.store(lua, self.index, value)
	}
}

impl<'a, TIndex: Index, TValue: Readable, TReadbox: Readbox<TIndex, TValue>> VariableLocation<'a, TIndex, TReadbox> {
	fn read(&self, lua: &Lua) -> Option<TValue> {
		self.prev.read(lua, self.index)
	}
}

impl<TValue: Pushable> Dropbox<std::string::String, TValue> for Globals {
	fn store(&self, lua: &Lua, index: &std::string::String, value: TValue) {
		unsafe {
			value.push_to_lua(lua);
			liblua::lua_setglobal(lua.lua, index.to_c_str().unwrap());
		}
	}
}

impl<TValue: Readable> Readbox<std::string::String, TValue> for Globals {
	fn read(&self, lua: &Lua, index: &std::string::String) -> Option<TValue> {
		unsafe {
			liblua::lua_getglobal(lua.lua, index.to_c_str().unwrap());
			let value = Readable::read_from_lua(lua, -1);
			liblua::lua_pop(lua.lua, 1);
			value
		}
	}
}
