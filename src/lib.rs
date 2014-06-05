#![crate_id = "lua"]
#![crate_type = "lib"]
#![comment = "Lua bindings for Rust"]
#![license = "MIT"]
#![allow(visible_private_types)]
#![feature(macro_rules)]

extern crate libc;
extern crate std;

mod liblua;
mod functions;
mod tables;
mod values;

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
	fn store(&self, &Lua, &TIndex, TPushable) -> Result<(), &'static str>;
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
	pub fn execute<T: Readable>(&mut self, code: &String) -> T {
		unimplemented!()
	}

	pub fn access<'a, 'b, TIndex: Index>(&'a mut self, index: &'b TIndex) -> VariableAccessor<'a, VariableLocation<'b, TIndex, Globals>> {
		VariableAccessor {
			lua: self,
			location: VariableLocation { index: index, prev: Globals }
		}
	}

	pub fn get<V: Readable>(&mut self, index: &String) -> Option<V> {
		self.access(index).get()
	}

	pub fn set<V: Pushable>(&mut self, index: &String, value: V) -> Result<(), &'static str> {
		self.access(index).set(value)
	}
}

impl<'a, 'b, TIndex: Index, TValue: Pushable, TDropbox: Dropbox<TIndex, TValue>> VariableAccessor<'a, VariableLocation<'b, TIndex, TDropbox>> {
	pub fn set(&mut self, value: TValue) -> Result<(), &'static str> {
		let loc = &self.location;
		loc.prev.store(self.lua, loc.index, value)
	}
}

impl<'a, 'b, TIndex: Index, TValue: Readable, TReadbox: Readbox<TIndex, TValue>> VariableAccessor<'a, VariableLocation<'b, TIndex, TReadbox>> {
	pub fn get(&self) -> Option<TValue> {
		let loc = &self.location;
		loc.prev.read(self.lua, loc.index)
	}
}

impl<TValue: Pushable> Dropbox<String, TValue> for Globals {
	fn store(&self, lua: &Lua, index: &String, value: TValue) -> Result<(), &'static str> {
		value.push_to_lua(lua);
		unsafe { liblua::lua_setglobal(lua.lua, index.to_c_str().unwrap()); }
		Ok(())
	}
}

impl<TValue: Readable> Readbox<String, TValue> for Globals {
	fn read(&self, lua: &Lua, index: &String) -> Option<TValue> {
		unsafe {
			liblua::lua_getglobal(lua.lua, index.to_c_str().unwrap());
			let value = Readable::read_from_lua(lua, -1);
			liblua::lua_pop(lua.lua, 1);
			value
		}
	}
}

/*impl<'a, TIndex: Index, TValue: Pushable, TIndex2: Index, TPrev: Readbox<TIndex2, >> Dropbox<TIndex, TValue> for VariableLocation<'a, TIndex2, TPrev> {
	fn store(&self, lua: &Lua, index: &TIndex, value: TValue) -> Result<(), &'static str> {
		match self.prev.read(lua, self.index) {
			Some(_) => (),
			None => return Err("Could not load the table")
		}
		index.push_to_lua(lua);
		value.push_to_lua(lua);
		unsafe { liblua::lua_settable(lua.lua, -3); }
		Ok(())
	}
}*/
