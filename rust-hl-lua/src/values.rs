use super::ffi;
use super::Index;
use super::Lua;
use super::Push;
use super::CopyRead;
use super::ConsumeRead;
use super::LoadedVariable;
use HasLua;

macro_rules! integer_impl(
    ($t:ident) => (
        impl<'lua, L: HasLua<'lua>> Push<L> for $t {
            fn push_to_lua(self, lua: &mut L) -> uint {
                unsafe { ffi::lua_pushinteger(lua.use_lua(), self as ffi::lua_Integer) };
                1
            }
        }
        impl<'lua, L: HasLua<'lua>> CopyRead<L> for $t {
            fn read_from_lua(lua: &mut L, index: i32) -> Option<$t> {
                let mut success: ::libc::c_int = unsafe { ::std::mem::uninitialized() };
                let val = unsafe { ffi::lua_tointegerx(lua.use_lua(), index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
        impl<'a,'lua> ConsumeRead<'a,'lua> for $t {
            fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<$t, LoadedVariable<'a, 'lua>> {
                match CopyRead::read_from_lua(var.lua, -1) {
                    None => Err(var),
                    Some(a) => Ok(a)
                }
            }
        }
        impl<'lua, L: HasLua<'lua>> Index<L> for $t {
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
        impl<'lua, L: HasLua<'lua>> Push<L> for $t {
            fn push_to_lua(self, lua: &mut L) -> uint {
                unsafe { ffi::lua_pushunsigned(lua.use_lua(), self as ffi::lua_Unsigned) };
                1
            }
        }
        impl<'lua, L: HasLua<'lua>> CopyRead<L> for $t {
            fn read_from_lua(lua: &mut L, index: i32) -> Option<$t> {
                let mut success: ::libc::c_int = unsafe { ::std::mem::uninitialized() };
                let val = unsafe { ffi::lua_tounsignedx(lua.use_lua(), index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
        impl<'a,'lua> ConsumeRead<'a,'lua> for $t {
            fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<$t, LoadedVariable<'a, 'lua>> {
                match CopyRead::read_from_lua(var.lua, -1) {
                    None => Err(var),
                    Some(a) => Ok(a)
                }
            }
        }
        impl<'lua, L: HasLua<'lua>> Index<L> for $t {
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
        impl<'lua, L: HasLua<'lua>> Push<L> for $t {
            fn push_to_lua(self, lua: &mut L) -> uint {
                unsafe { ffi::lua_pushnumber(lua.use_lua(), self as f64) };
                1
            }
        }
        impl<'lua, L: HasLua<'lua>> CopyRead<L> for $t {
            fn read_from_lua(lua: &mut L, index: i32) -> Option<$t> {
                let mut success: ::libc::c_int = unsafe { ::std::mem::uninitialized() };
                let val = unsafe { ffi::lua_tonumberx(lua.use_lua(), index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
        impl<'a,'lua> ConsumeRead<'a,'lua> for $t {
            fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<$t, LoadedVariable<'a, 'lua>> {
                match CopyRead::read_from_lua(var.lua, -1) {
                    None => Err(var),
                    Some(a) => Ok(a)
                }
            }
        }
        impl<'lua, L: HasLua<'lua>> Index<L> for $t {
        }
    );
)

numeric_impl!(f32)
numeric_impl!(f64)

impl<'lua, L: HasLua<'lua>> Push<L> for String {
    fn push_to_lua(self, lua: &mut L) -> uint {
        unsafe { ffi::lua_pushstring(lua.use_lua(), self.to_c_str().unwrap()) };
        1
    }
}

impl<'lua, L: HasLua<'lua>> CopyRead<L> for String {
    fn read_from_lua(lua: &mut L, index: i32) -> Option<String> {
        let mut size: ::libc::size_t = unsafe { ::std::mem::uninitialized() };
        let cStrRaw = unsafe { ffi::lua_tolstring(lua.use_lua(), index, &mut size) };
        if cStrRaw.is_null() {
            return None;
        }

        unsafe { ::std::c_str::CString::new(cStrRaw, false) }.as_str().map(|s| s.to_string())
    }
}

impl<'a,'lua> ConsumeRead<'a,'lua> for String {
    fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<String, LoadedVariable<'a, 'lua>> {
        match CopyRead::read_from_lua(var.lua, -1) {
            None => Err(var),
            Some(a) => Ok(a)
        }
    }
}

impl<'lua, L: HasLua<'lua>> Index<L> for String {
}

impl<'lua, 'str, L: HasLua<'lua>> Push<L> for &'str str {
    fn push_to_lua(self, lua: &mut L) -> uint {
        unsafe { ffi::lua_pushstring(lua.use_lua(), self.to_c_str().unwrap()) }
        1
    }
}

impl<'lua, L: HasLua<'lua>> Push<L> for bool {
    fn push_to_lua(self, lua: &mut L) -> uint {
        unsafe { ffi::lua_pushboolean(lua.use_lua(), self.clone() as ::libc::c_int) };
        1
    }
}

impl<'lua, L: HasLua<'lua>> CopyRead<L> for bool {
    fn read_from_lua(lua: &mut L, index: i32) -> Option<bool> {
        if unsafe { ffi::lua_isboolean(lua.use_lua(), index) } != true {
            return None;
        }

        Some(unsafe { ffi::lua_toboolean(lua.use_lua(), index) != 0 })
    }
}

impl<'a,'lua> ConsumeRead<'a,'lua> for bool {
    fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<bool, LoadedVariable<'a, 'lua>> {
        match CopyRead::read_from_lua(var.lua, -1) {
            None => Err(var),
            Some(a) => Ok(a)
        }
    }
}

impl<'lua, L: HasLua<'lua>> Index<L> for bool {
}

impl<'lua, L: HasLua<'lua>> Push<L> for () {
    fn push_to_lua(self, _: &mut L) -> uint {
        0
    }
}

impl<'lua, L: HasLua<'lua>> CopyRead<L> for () {
    fn read_from_lua(_: &mut L, _: i32) -> Option<()> {
        Some(())
    }
}

impl<'a,'lua> ConsumeRead<'a,'lua> for () {
    fn read_from_variable(_: LoadedVariable<'a, 'lua>) -> Result<(), LoadedVariable<'a, 'lua>> {
        Ok(())
    }
}
