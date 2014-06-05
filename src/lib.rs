#![crate_id = "lua"]
#![crate_type = "lib"]
#![comment = "Lua bindings for Rust"]
#![license = "MIT"]

extern crate libc;
extern crate std;

mod liblua;
pub mod value;

pub struct Lua {
	lua: *mut liblua::lua_State
}

pub struct VariableAccessor<'a, TIndexFollower> {
	lua: &'a mut Lua,
	location: TIndexFollower
}

pub trait Pushable {
	fn push_to_lua(self, &Lua);
}

pub trait Readable {
	fn read_from_lua(&Lua, i32) -> Option<Self>;
}

pub trait Index: Pushable + Readable {
}

trait Dropbox<TIndex: Index, TPushable: Pushable> {
	fn store(&self, &Lua, &TIndex, TPushable);
}

trait Readbox<TIndex: Index, TReadable: Readable> {
	fn read(&self, &Lua, &TIndex) -> Option<TReadable>;
}

pub struct IndexFollower<'a, TIndex, TPrev> {
	index: &'a TIndex,
	prev: TPrev
}

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
	pub fn new() -> Lua {
		let lua = unsafe { liblua::lua_newstate(alloc, std::ptr::mut_null()) };
		if lua.is_null() {
			fail!("lua_newstate failed");
		}

		Lua { lua: lua }
	}

	pub fn execute<T: Readable>(&mut self, code: &std::string::String) -> T {
		unimplemented!()
	}

	pub fn access<'a, 'b, TIndex: Index>(&'a mut self, index: &'b TIndex) -> VariableAccessor<'a, IndexFollower<'b, TIndex, Globals>> {
		VariableAccessor {
			lua: self,
			location: IndexFollower { index: index, prev: Globals }
		}
	}
}

impl<'a, 'b, TIndex: Index, TValue: Pushable, TDropbox: Dropbox<TIndex, TValue>> VariableAccessor<'a, IndexFollower<'b, TIndex, TDropbox>> {
	pub fn set(&mut self, value: TValue) {
		self.location.store(self.lua, value)
	}
}

impl<'a, 'b, TIndex: Index, TValue: Readable, TReadbox: Readbox<TIndex, TValue>> VariableAccessor<'a, IndexFollower<'b, TIndex, TReadbox>> {
	pub fn get(&self) -> Option<TValue> {
		self.location.read(self.lua)
	}
}

impl<'a, TIndex: Index, TValue: Pushable, TDropbox: Dropbox<TIndex, TValue>> IndexFollower<'a, TIndex, TDropbox> {
	fn store(&self, lua: &Lua, value: TValue) {
		self.prev.store(lua, self.index, value)
	}
}

impl<'a, TIndex: Index, TValue: Readable, TReadbox: Readbox<TIndex, TValue>> IndexFollower<'a, TIndex, TReadbox> {
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
