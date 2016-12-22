use std::ffi::{CStr, CString};
use std::mem;

use ffi;
use libc;

use AsLua;
use AsMutLua;
use LuaRead;
use Push;
use PushGuard;

macro_rules! integer_impl(
    ($t:ident) => (
        impl<L> Push<L> for $t where L: AsMutLua {
            #[inline]
            fn push_to_lua(self, mut lua: L) -> PushGuard<L> {
                unsafe { ffi::lua_pushinteger(lua.as_mut_lua().0, self as ffi::lua_Integer) };
                PushGuard { lua: lua, size: 1 }
            }
        }

        impl<L> LuaRead<L> for $t where L: AsLua {
            #[inline]
            fn lua_read_at_position(lua: L, index: i32) -> Result<$t, L> {
                let mut success = unsafe { mem::uninitialized() };
                let val = unsafe { ffi::lua_tointegerx(lua.as_lua().0, index, &mut success) };
                match success {
                    0 => Err(lua),
                    _ => Ok(val as $t)
                }
            }
        }
    );
);

integer_impl!(i8);
integer_impl!(i16);
integer_impl!(i32);
//integer_impl!(i64)   // data loss

macro_rules! unsigned_impl(
    ($t:ident) => (
        impl<L> Push<L> for $t where L: AsMutLua {
            #[inline]
            fn push_to_lua(self, mut lua: L) -> PushGuard<L> {
                unsafe { ffi::lua_pushunsigned(lua.as_mut_lua().0, self as ffi::lua_Unsigned) };
                PushGuard { lua: lua, size: 1 }
            }
        }

        impl<L> LuaRead<L> for $t where L: AsLua {
            #[inline]
            fn lua_read_at_position(lua: L, index: i32) -> Result<$t, L> {
                let mut success = unsafe { mem::uninitialized() };
                let val = unsafe { ffi::lua_tounsignedx(lua.as_lua().0, index, &mut success) };
                match success {
                    0 => Err(lua),
                    _ => Ok(val as $t)
                }
            }
        }
    );
);

unsigned_impl!(u8);
unsigned_impl!(u16);
unsigned_impl!(u32);
//unsigned_impl!(u64);   // data loss

macro_rules! numeric_impl(
    ($t:ident) => (
        impl<L> Push<L> for $t where L: AsMutLua {
            #[inline]
            fn push_to_lua(self, mut lua: L) -> PushGuard<L> {
                unsafe { ffi::lua_pushnumber(lua.as_mut_lua().0, self as f64) };
                PushGuard { lua: lua, size: 1 }
            }
        }

        impl<L> LuaRead<L> for $t where L: AsLua {
            #[inline]
            fn lua_read_at_position(lua: L, index: i32) -> Result<$t, L> {
                let mut success = unsafe { mem::uninitialized() };
                let val = unsafe { ffi::lua_tonumberx(lua.as_lua().0, index, &mut success) };
                match success {
                    0 => Err(lua),
                    _ => Ok(val as $t)
                }
            }
        }
    );
);

numeric_impl!(f32);
numeric_impl!(f64);

impl<L> Push<L> for String where L: AsMutLua {
    #[inline]
    fn push_to_lua(self, mut lua: L) -> PushGuard<L> {
        let value = CString::new(&self[..]).unwrap();
        unsafe { ffi::lua_pushstring(lua.as_mut_lua().0, value.as_ptr()) };
        PushGuard { lua: lua, size: 1 }
    }
}

impl<L> LuaRead<L> for String where L: AsLua {
    #[inline]
    fn lua_read_at_position(lua: L, index: i32) -> Result<String, L> {
        let mut size: libc::size_t = unsafe { mem::uninitialized() };
        let c_str_raw = unsafe { ffi::lua_tolstring(lua.as_lua().0, index, &mut size) };
        if c_str_raw.is_null() {
            return Err(lua);
        }

        let c_str = unsafe { CStr::from_ptr(c_str_raw) };
        let c_str = String::from_utf8(c_str.to_bytes().to_vec()).unwrap();

        Ok(c_str)
    }
}

impl<'s, L> Push<L> for &'s str where L: AsMutLua {
    #[inline]
    fn push_to_lua(self, mut lua: L) -> PushGuard<L> {
        let value = CString::new(&self[..]).unwrap();
        unsafe { ffi::lua_pushstring(lua.as_mut_lua().0, value.as_ptr()) };
        PushGuard { lua: lua, size: 1 }
    }
}

impl<L> Push<L> for bool where L: AsMutLua {
    #[inline]
    fn push_to_lua(self, mut lua: L) -> PushGuard<L> {
        unsafe { ffi::lua_pushboolean(lua.as_mut_lua().0, self.clone() as libc::c_int) };
        PushGuard { lua: lua, size: 1 }
    }
}

impl<L> LuaRead<L> for bool where L: AsLua {
    #[inline]
    fn lua_read_at_position(lua: L, index: i32) -> Result<bool, L> {
        if unsafe { ffi::lua_isboolean(lua.as_lua().0, index) } != true {
            return Err(lua);
        }

        Ok(unsafe { ffi::lua_toboolean(lua.as_lua().0, index) != 0 })
    }
}

impl<L> Push<L> for () where L: AsMutLua {
    #[inline]
    fn push_to_lua(self, lua: L) -> PushGuard<L> {
        PushGuard { lua: lua, size: 0 }
    }
}

impl<L> LuaRead<L> for () where L: AsLua {
    #[inline]
    fn lua_read_at_position(_: L, _: i32) -> Result<(), L> {
        Ok(())
    }
}
