use std::mem;
use std::slice;
use std::str;
use std::ops::Deref;

use ffi;
use libc;

use AnyLuaValue;
use AnyLuaString;
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
            ffi::lua_pushlstring(lua.as_mut_lua().0,
                                 self.as_bytes().as_ptr() as *const _,
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

impl<'lua, L> PushOne<L> for String where L: AsMutLua<'lua> {}

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

        let c_slice = unsafe { slice::from_raw_parts(c_str_raw as *const u8, size) };
        let maybe_string = String::from_utf8(c_slice.to_vec());
        match maybe_string {
            Ok(string) => Ok(string),
            Err(_) => Err(lua),
        }
    }
}

impl<'lua, L> Push<L> for AnyLuaString
    where L: AsMutLua<'lua>
{
    type Err = Void;      // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

    #[inline]
    fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
        let AnyLuaString(v) = self;
        unsafe {
            ffi::lua_pushlstring(lua.as_mut_lua().0,
                                 v[..].as_ptr() as *const _,
                                 v[..].len() as libc::size_t);

            let raw_lua = lua.as_lua();
            Ok(PushGuard {
                lua: lua,
                size: 1,
                raw_lua: raw_lua,
            })
        }
    }
}

impl<'lua, L> LuaRead<L> for AnyLuaString
    where L: AsLua<'lua>
{
    #[inline]
    fn lua_read_at_position(lua: L, index: i32) -> Result<AnyLuaString, L> {
        let mut size: libc::size_t = unsafe { mem::uninitialized() };
        let c_str_raw = unsafe { ffi::lua_tolstring(lua.as_lua().0, index, &mut size) };
        if c_str_raw.is_null() {
            return Err(lua);
        }

        let c_slice = unsafe { slice::from_raw_parts(c_str_raw as *const u8, size) };
        Ok(AnyLuaString(c_slice.to_vec()))
    }
}

impl<'lua, 's, L> Push<L> for &'s str
    where L: AsMutLua<'lua>
{
    type Err = Void;      // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

    #[inline]
    fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
        unsafe {
            ffi::lua_pushlstring(lua.as_mut_lua().0,
                                 self.as_bytes().as_ptr() as *const _,
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

impl<'lua, 's, L> PushOne<L> for &'s str where L: AsMutLua<'lua> {}

/// String on the Lua stack.
///
/// It is faster -but less convenient- to read a `StringInLua` rather than a `String` because you
/// avoid any allocation.
///
/// The `StringInLua` derefs to `str`.
///
/// # Example
///
/// ```
/// let mut lua = hlua::Lua::new();
/// lua.set("a", "hello");
///
/// let s: hlua::StringInLua<_> = lua.get("a").unwrap();
/// println!("{}", &*s);    // Prints "hello".
/// ```
#[derive(Debug)]
pub struct StringInLua<L> {
    lua: L,
    c_str_raw: *const libc::c_char,
    size: libc::size_t,
}

impl<'lua, L> LuaRead<L> for StringInLua<L>
    where L: AsLua<'lua>
{
    #[inline]
    fn lua_read_at_position(lua: L, index: i32) -> Result<StringInLua<L>, L> {
        let mut size: libc::size_t = unsafe { mem::uninitialized() };
        let c_str_raw = unsafe { ffi::lua_tolstring(lua.as_lua().0, index, &mut size) };
        if c_str_raw.is_null() {
            return Err(lua);
        }

        let c_slice = unsafe { slice::from_raw_parts(c_str_raw as *const u8, size) };
        match str::from_utf8(c_slice) {
            Ok(_) => (),
            Err(_) => return Err(lua)
        };

        Ok(StringInLua {
            lua: lua,
            c_str_raw: c_str_raw,
            size: size,
        })
    }
}

impl<L> Deref for StringInLua<L> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        let c_slice = unsafe { slice::from_raw_parts(self.c_str_raw as *const u8, self.size) };
        match str::from_utf8(c_slice) {
            Ok(s) => s,
            Err(_) => unreachable!()        // Checked earlier
        }
    }
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

impl<'lua, L> PushOne<L> for bool where L: AsMutLua<'lua> {}

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

impl<'lua, L, T, E> Push<L> for Option<T>
where T: Push<L, Err = E>,
      L: AsMutLua<'lua>
{
    type Err = E;

    #[inline]
    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (E, L)> {
        match self {
            Some(val) => val.push_to_lua(lua),
            None => Ok(AnyLuaValue::LuaNil.push_no_err(lua)),
        }
    }
}

impl<'lua, L, T, E> PushOne<L> for Option<T>
where T: PushOne<L, Err = E>,
      L: AsMutLua<'lua>
{
}

#[cfg(test)]
mod tests {
    use AnyLuaValue;
    use AnyLuaString;
    use Lua;
    use StringInLua;

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

        assert_eq!(lua.execute::<String>("return 'abc'").unwrap(), "abc");
        assert_eq!(lua.execute::<u32>("return #'abc'").unwrap(), 3);
        assert_eq!(lua.execute::<u32>("return #'a\\x00c'").unwrap(), 3);
        assert_eq!(lua.execute::<AnyLuaString>("return 'a\\x00c'").unwrap().0, vec!(97, 0, 99));
        assert_eq!(lua.execute::<AnyLuaString>("return 'a\\x00c'").unwrap().0.len(), 3);
        assert_eq!(lua.execute::<AnyLuaString>("return '\\x01\\xff'").unwrap().0, vec!(1, 255));
        lua.execute::<String>("return 'a\\x00\\xc0'").unwrap_err();
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

    #[test]
    fn string_on_lua() {
        let mut lua = Lua::new();

        lua.set("a", "aaa");
        {
            let x: StringInLua<_> = lua.get("a").unwrap();
            assert_eq!(&*x, "aaa");
        }
        
        lua.set("a", 18);
        {
            let x: StringInLua<_> = lua.get("a").unwrap();
            assert_eq!(&*x, "18");
        }
    }

    #[test]
    fn push_opt() {
        let mut lua = Lua::new();

        lua.set("some", ::function0(|| Some(123)));
        lua.set("none", ::function0(|| Option::None::<i32>));

        match lua.execute::<i32>("return some()") {
            Ok(123) => {}
            unexpected => panic!("{:?}", unexpected),
        }

        match lua.execute::<AnyLuaValue>("return none()") {
            Ok(AnyLuaValue::LuaNil) => {}
            unexpected => panic!("{:?}", unexpected),
        }

        lua.set("no_value", None::<i32>);
        lua.set("some_value", Some("Hello!"));

        assert_eq!(lua.get("no_value"), None::<String>);
        assert_eq!(lua.get("some_value"), Some("Hello!".to_string()));
    }
}
