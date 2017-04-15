use ffi;

use AsLua;
use AsMutLua;

use Push;
use PushGuard;
use PushOne;
use LuaRead;
use Void;

/// Represents any value that can be stored by Lua
#[derive(Clone, Debug, PartialEq)]
pub enum AnyLuaValue {
    LuaString(String),
    LuaPlainString(Vec<u8>),
    LuaNumber(f64),
    LuaBoolean(bool),
    LuaArray(Vec<(AnyLuaValue, AnyLuaValue)>),
    LuaNil,

    /// The "Other" element is (hopefully) temporary and will be replaced by "Function" and "Userdata".
    /// A panic! will trigger if you try to push a Other.
    LuaOther,
}

impl<'lua, L> Push<L> for AnyLuaValue
    where L: AsMutLua<'lua>
{
    type Err = Void;      // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

    #[inline]
    fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
        let raw_lua = lua.as_lua();
        match self {
            AnyLuaValue::LuaString(val) => val.push_to_lua(lua),
            AnyLuaValue::LuaPlainString(val) => val.push_to_lua(lua),
            AnyLuaValue::LuaNumber(val) => val.push_to_lua(lua),
            AnyLuaValue::LuaBoolean(val) => val.push_to_lua(lua),
            AnyLuaValue::LuaArray(val) => {
                // Pushing a `Vec<(AnyLuaValue, AnyLuaValue)>` on a `L` requires calling the
                // function that pushes a `AnyLuaValue` on a `&mut L`, which in turns requires
                // calling the function that pushes a `AnyLuaValue` on a `&mut &mut L`, and so on.
                // In order to avoid this infinite recursion, we push the array on a
                // `&mut AsMutLua` instead.

                // We also need to destroy and recreate the push guard, otherwise the type parameter
                // doesn't match.
                let size = val.push_no_err(&mut lua as &mut AsMutLua<'lua>).forget();

                Ok(PushGuard {
                    lua: lua,
                    size: size,
                    raw_lua: raw_lua,
                })
            }
            AnyLuaValue::LuaNil => {
                unsafe {
                    ffi::lua_pushnil(lua.as_mut_lua().0);
                }
                Ok(PushGuard {
                    lua: lua,
                    size: 1,
                    raw_lua: raw_lua,
                })
            } // Use ffi::lua_pushnil.
            AnyLuaValue::LuaOther => panic!("can't push a AnyLuaValue of type Other"),
        }
    }
}

impl<'lua, L> PushOne<L> for AnyLuaValue where L: AsMutLua<'lua> {}

impl<'lua, L> LuaRead<L> for AnyLuaValue
    where L: AsLua<'lua>
{
    #[inline]
    fn lua_read_at_position(lua: L, index: i32) -> Result<AnyLuaValue, L> {
        let lua = match LuaRead::lua_read_at_position(&lua, index) {
            Ok(v) => return Ok(AnyLuaValue::LuaNumber(v)),
            Err(lua) => lua,
        };

        let lua = match LuaRead::lua_read_at_position(&lua, index) {
            Ok(v) => return Ok(AnyLuaValue::LuaBoolean(v)),
            Err(lua) => lua,
        };

        let lua = match LuaRead::lua_read_at_position(&lua, index) {
            Ok(v) => return Ok(AnyLuaValue::LuaString(v)),
            Err(lua) => lua,
        };

        let lua = match LuaRead::lua_read_at_position(&lua, index) {
            Ok(v) => return Ok(AnyLuaValue::LuaPlainString(v)),
            Err(lua) => lua,
        };

        if unsafe { ffi::lua_isnil(lua.as_lua().0, index) } {
            return Ok(AnyLuaValue::LuaNil);
        }

        // let _lua = match LuaRead::lua_read_at_position(&lua, index) {
        // Ok(v) => return Ok(AnyLuaValue::LuaArray(v)),
        // Err(lua) => lua
        // };

        Ok(AnyLuaValue::LuaOther)
    }
}

#[cfg(test)]
mod tests {
    use Lua;
    use AnyLuaValue;

    #[test]
    fn read_numbers() {
        let mut lua = Lua::new();

        lua.set("a", "-2");
        lua.set("b", 3.5f32);

        let x: AnyLuaValue = lua.get("a").unwrap();
        assert_eq!(x, AnyLuaValue::LuaNumber(-2.0));

        let y: AnyLuaValue = lua.get("b").unwrap();
        assert_eq!(y, AnyLuaValue::LuaNumber(3.5));
    }

    #[test]
    fn read_strings() {
        let mut lua = Lua::new();

        lua.set("a", "hello");
        lua.set("b", "3x");
        lua.set("c", "false");

        let x: AnyLuaValue = lua.get("a").unwrap();
        assert_eq!(x, AnyLuaValue::LuaString("hello".to_string()));

        let y: AnyLuaValue = lua.get("b").unwrap();
        assert_eq!(y, AnyLuaValue::LuaString("3x".to_string()));

        let z: AnyLuaValue = lua.get("c").unwrap();
        assert_eq!(z, AnyLuaValue::LuaString("false".to_string()));
    }

    #[test]
    fn read_booleans() {
        let mut lua = Lua::new();

        lua.set("a", true);
        lua.set("b", false);

        let x: AnyLuaValue = lua.get("a").unwrap();
        assert_eq!(x, AnyLuaValue::LuaBoolean(true));

        let y: AnyLuaValue = lua.get("b").unwrap();
        assert_eq!(y, AnyLuaValue::LuaBoolean(false));
    }

    #[test]
    fn push_numbers() {
        let mut lua = Lua::new();

        lua.set("a", AnyLuaValue::LuaNumber(3.0));

        let x: i32 = lua.get("a").unwrap();
        assert_eq!(x, 3);
    }

    #[test]
    fn push_strings() {
        let mut lua = Lua::new();

        lua.set("a", AnyLuaValue::LuaString("hello".to_string()));

        let x: String = lua.get("a").unwrap();
        assert_eq!(x, "hello");
    }

    #[test]
    fn push_booleans() {
        let mut lua = Lua::new();

        lua.set("a", AnyLuaValue::LuaBoolean(true));

        let x: bool = lua.get("a").unwrap();
        assert_eq!(x, true);
    }

    #[test]
    fn push_nil() {
        let mut lua = Lua::new();

        lua.set("a", AnyLuaValue::LuaNil);

        let x: Option<i32> = lua.get("a");
        assert!(x.is_none(),
                "x is a Some value when it should be a None value. X: {:?}",
                x);
    }


    #[test]
    fn non_utf_8_string() {
        let mut lua = Lua::new();
        let a = lua.execute::<AnyLuaValue>(r"return '\xff\xfe\xff\xfe'").unwrap();
        match a {
            AnyLuaValue::LuaPlainString(v) => {
                assert_eq!(Vec::from(&b"\xff\xfe\xff\xfe"[..]), v);
            },
            _ => panic!("Decoded to wrong variant"),
        }
    }
}
