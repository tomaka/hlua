use std::ffi::CStr;
use std::mem;

use ffi;
use libc;

use AsLua;
use AsMutLua;
use LuaRead;
use Push;
use PushGuard;
use PushOne;
use Void;

macro_rules! integer_impl(
    ($t:ident) => (
        impl<'lua, L> Push<L> for $t where L: AsMutLua<'lua> {
            type Err = Void;      // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

            #[inline]
            fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
                unsafe { ffi::lua_pushinteger(lua.as_mut_lua().0, self as ffi::lua_Integer) };
                let raw_lua = lua.as_lua();
                Ok(PushGuard { lua: lua, size: 1, raw_lua: raw_lua })
            }
        }
        
        impl<'lua, L> PushOne<L> for $t where L: AsMutLua<'lua> {
        }

        impl<'lua, L> LuaRead<L> for $t where L: AsLua<'lua> {
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
// integer_impl!(i64)   // data loss

macro_rules! unsigned_impl(
    ($t:ident) => (
        impl<'lua, L> Push<L> for $t where L: AsMutLua<'lua> {
            type Err = Void;      // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

            #[inline]
            fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
                unsafe { ffi::lua_pushunsigned(lua.as_mut_lua().0, self as ffi::lua_Unsigned) };
                let raw_lua = lua.as_lua();
                Ok(PushGuard { lua: lua, size: 1, raw_lua: raw_lua })
            }
        }
        
        impl<'lua, L> PushOne<L> for $t where L: AsMutLua<'lua> {
        }

        impl<'lua, L> LuaRead<L> for $t where L: AsLua<'lua> {
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
// unsigned_impl!(u64);   // data loss

macro_rules! numeric_impl(
    ($t:ident) => (
        impl<'lua, L> Push<L> for $t where L: AsMutLua<'lua> {
            type Err = Void;      // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

            #[inline]
            fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
                unsafe { ffi::lua_pushnumber(lua.as_mut_lua().0, self as f64) };
                let raw_lua = lua.as_lua();
                Ok(PushGuard { lua: lua, size: 1, raw_lua: raw_lua })
            }
        }
        
        impl<'lua, L> PushOne<L> for $t where L: AsMutLua<'lua> {
        }

        impl<'lua, L> LuaRead<L> for $t where L: AsLua<'lua> {
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

impl<'lua, L> Push<L> for String
    where L: AsMutLua<'lua>
{
    type Err = Void;      // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

    #[inline]
    fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
        unsafe {
            ffi::lua_pushlstring(lua.as_mut_lua().0, self.as_bytes().as_ptr() as *const _,
                                 self.as_bytes().len() as libc::size_t);

            let raw_lua = lua.as_lua();
            Ok(PushGuard {
                lua: lua,
                size: 1,
                raw_lua: raw_lua,
            })
        }
    }
}

impl<'lua, L> PushOne<L> for String
    where L: AsMutLua<'lua>
{
}

impl<'lua, L> LuaRead<L> for String
    where L: AsLua<'lua>
{
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

impl<'lua, 's, L> Push<L> for &'s str
    where L: AsMutLua<'lua>
{
    type Err = Void;      // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

    #[inline]
    fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
        unsafe {
            ffi::lua_pushlstring(lua.as_mut_lua().0, self.as_bytes().as_ptr() as *const _,
                                 self.as_bytes().len() as libc::size_t);

            let raw_lua = lua.as_lua();
            Ok(PushGuard {
                lua: lua,
                size: 1,
                raw_lua: raw_lua,
            })
        }
    }
}

impl<'lua, 's, L> PushOne<L> for &'s str
    where L: AsMutLua<'lua>
{
}

impl<'lua, L> Push<L> for bool
    where L: AsMutLua<'lua>
{
    type Err = Void;      // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

    #[inline]
    fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
        unsafe { ffi::lua_pushboolean(lua.as_mut_lua().0, self.clone() as libc::c_int) };
        let raw_lua = lua.as_lua();
        Ok(PushGuard {
            lua: lua,
            size: 1,
            raw_lua: raw_lua,
        })
    }
}

impl<'lua, L> PushOne<L> for bool
    where L: AsMutLua<'lua>
{
}

impl<'lua, L> LuaRead<L> for bool
    where L: AsLua<'lua>
{
    #[inline]
    fn lua_read_at_position(lua: L, index: i32) -> Result<bool, L> {
        if unsafe { ffi::lua_isboolean(lua.as_lua().0, index) } != true {
            return Err(lua);
        }

        Ok(unsafe { ffi::lua_toboolean(lua.as_lua().0, index) != 0 })
    }
}

impl<'lua, L> Push<L> for ()
    where L: AsMutLua<'lua>
{
    type Err = Void;      // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

    #[inline]
    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (Void, L)> {
        let raw_lua = lua.as_lua();

        Ok(PushGuard {
            lua: lua,
            size: 0,
            raw_lua: raw_lua,
        })
    }
}

impl<'lua, L> LuaRead<L> for ()
    where L: AsLua<'lua>
{
    #[inline]
    fn lua_read_at_position(_: L, _: i32) -> Result<(), L> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use Lua;

    #[test]
    fn read_i32s() {
        let mut lua = Lua::new();

        lua.set("a", 2);

        let x: i32 = lua.get("a").unwrap();
        assert_eq!(x, 2);

        let y: i8 = lua.get("a").unwrap();
        assert_eq!(y, 2);

        let z: i16 = lua.get("a").unwrap();
        assert_eq!(z, 2);

        let w: i32 = lua.get("a").unwrap();
        assert_eq!(w, 2);

        let a: u32 = lua.get("a").unwrap();
        assert_eq!(a, 2);

        let b: u8 = lua.get("a").unwrap();
        assert_eq!(b, 2);

        let c: u16 = lua.get("a").unwrap();
        assert_eq!(c, 2);

        let d: u32 = lua.get("a").unwrap();
        assert_eq!(d, 2);
    }

    #[test]
    fn write_i32s() {
        // TODO:

        let mut lua = Lua::new();

        lua.set("a", 2);
        let x: i32 = lua.get("a").unwrap();
        assert_eq!(x, 2);
    }

    #[test]
    fn readwrite_floats() {
        let mut lua = Lua::new();

        lua.set("a", 2.51234 as f32);
        lua.set("b", 3.4123456789 as f64);

        let x: f32 = lua.get("a").unwrap();
        assert!(x - 2.51234 < 0.000001);

        let y: f64 = lua.get("a").unwrap();
        assert!(y - 2.51234 < 0.000001);

        let z: f32 = lua.get("b").unwrap();
        assert!(z - 3.4123456789 < 0.000001);

        let w: f64 = lua.get("b").unwrap();
        assert!(w - 3.4123456789 < 0.000001);
    }

    #[test]
    fn readwrite_bools() {
        let mut lua = Lua::new();

        lua.set("a", true);
        lua.set("b", false);

        let x: bool = lua.get("a").unwrap();
        assert_eq!(x, true);

        let y: bool = lua.get("b").unwrap();
        assert_eq!(y, false);
    }

    #[test]
    fn readwrite_strings() {
        let mut lua = Lua::new();

        lua.set("a", "hello");
        lua.set("b", "hello".to_string());

        let x: String = lua.get("a").unwrap();
        assert_eq!(x, "hello");

        let y: String = lua.get("b").unwrap();
        assert_eq!(y, "hello");
    }

    #[test]
    fn i32_to_string() {
        let mut lua = Lua::new();

        lua.set("a", 2);

        let x: String = lua.get("a").unwrap();
        assert_eq!(x, "2");
    }

    #[test]
    fn string_to_i32() {
        let mut lua = Lua::new();

        lua.set("a", "2");
        lua.set("b", "aaa");

        let x: i32 = lua.get("a").unwrap();
        assert_eq!(x, 2);

        let y: Option<i32> = lua.get("b");
        assert!(y.is_none());
    }
}
