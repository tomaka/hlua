use super::ffi;
use super::Index;
use super::Push;
use super::CopyRead;
use super::ConsumeRead;
use super::LoadedVariable;

use std::mem;

use libc;

use AsMutLua;
use LuaRead;

macro_rules! integer_impl(
    ($t:ident) => (
        impl<L> Push<L> for $t where L: AsMutLua {
            fn push_to_lua(self, lua: &mut L) -> uint {
                unsafe { ffi::lua_pushinteger(lua.as_mut_lua(), self as ffi::lua_Integer) };
                1
            }
        }

        impl<'l, L> LuaRead<'l, L> for $t where L: AsMutLua {
            fn lua_read(lua: &mut L, index: i32) -> Option<$t> {
                let mut success = unsafe { mem::uninitialized() };
                let val = unsafe { ffi::lua_tointegerx(lua.as_lua(), index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
    );
);

integer_impl!(int);
integer_impl!(i8);
integer_impl!(i16);
integer_impl!(i32);
//integer_impl!(i64)   // data loss

macro_rules! unsigned_impl(
    ($t:ident) => (
        impl<L: AsLua> Push<L> for $t {
            fn push_to_lua(self, lua: &mut L) -> uint {
                unsafe { ffi::lua_pushunsigned(lua.as_lua(), self as ffi::lua_Unsigned) };
                1
            }
        }

        impl<'l, L> LuaRead<'l, L> for $t where L: AsMutLua {
            fn lua_read(lua: &mut L, index: i32) -> Option<$t> {
                let mut success = unsafe { mem::uninitialized() };
                let val = unsafe { ffi::lua_tounsignedx(lua.as_lua(), index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
    );
);

unsigned_impl!(uint);
unsigned_impl!(u8);
unsigned_impl!(u16);
unsigned_impl!(u32);
//unsigned_impl!(u64);   // data loss

macro_rules! numeric_impl(
    ($t:ident) => (
        impl<L: AsLua> Push<L> for $t {
            fn push_to_lua(self, lua: &mut L) -> uint {
                unsafe { ffi::lua_pushnumber(lua.as_lua(), self as f64) };
                1
            }
        }

        impl<'l, L> LuaRead<'l, L> for $t where L: AsMutLua {
            fn lua_read(lua: &mut L, index: i32) -> Option<$t> {
                let mut success = unsafe { mem::uninitialized() };
                let val = unsafe { ffi::lua_tonumberx(lua.as_lua(), index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
    );
);

numeric_impl!(f32);
numeric_impl!(f64);

impl<L> Push<L> for String where L: AsLua {
    fn push_to_lua(self, lua: &mut L) -> uint {
        unsafe { ffi::lua_pushstring(lua.as_lua(), self.to_c_str().into_inner()) };
        1
    }
}

impl<'l, L> LuaRead<'l, L> for String where L: AsMutLua {
    fn lua_read(lua: &mut L, index: i32) -> Option<$t> {
        let mut size: libc::size_t = unsafe { mem::uninitialized() };
        let c_str_raw = unsafe { ffi::lua_tolstring(lua.as_lua(), index, &mut size) };
        if c_str_raw.is_null() {
            return None;
        }

        unsafe { ::std::c_str::CString::new(c_str_raw, false) }.as_str().map(|s| s.to_string())
    }
}

impl<'str, L: AsLua> Push<L> for &'str str {
    fn push_to_lua(self, lua: &mut L) -> uint {
        unsafe { ffi::lua_pushstring(lua.as_lua(), self.to_c_str().into_inner()) }
        1
    }
}

impl<L: AsLua> Push<L> for bool {
    fn push_to_lua(self, lua: &mut L) -> uint {
        unsafe { ffi::lua_pushboolean(lua.as_lua(), self.clone() as ::libc::c_int) };
        1
    }
}

impl<L: AsLua> CopyRead<L> for bool {
    fn read_from_lua(lua: &mut L, index: i32) -> Option<bool> {
        if unsafe { ffi::lua_isboolean(lua.as_lua(), index) } != true {
            return None;
        }

        Some(unsafe { ffi::lua_toboolean(lua.as_lua(), index) != 0 })
    }
}

impl<'a, L: AsLua> ConsumeRead<'a, L> for bool {
    fn read_from_variable(var: LoadedVariable<'a, L>) -> Result<bool, LoadedVariable<'a, L>> {
        match CopyRead::read_from_lua(var.lua, -1) {
            None => Err(var),
            Some(a) => Ok(a)
        }
    }
}

impl<L: AsLua> Index<L> for bool {
}

impl<L: AsLua> Push<L> for () {
    fn push_to_lua(self, _: &mut L) -> uint {
        0
    }
}

impl<L: AsLua> CopyRead<L> for () {
    fn read_from_lua(_: &mut L, _: i32) -> Option<()> {
        Some(())
    }
}

impl<'a, L: AsLua> ConsumeRead<'a, L> for () {
    fn read_from_variable(_: LoadedVariable<'a, L>) -> Result<(), LoadedVariable<'a, L>> {
        Ok(())
    }
}
