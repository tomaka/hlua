
use std::convert::From;

use ffi;

use AsLua;
use AsMutLua;

use LuaRead;
use LuaTable;
use Push;
use PushGuard;
use PushOne;
use Void;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AnyLuaString(pub Vec<u8>);

/// Represents any value that can be stored by Lua
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AnyHashableLuaValue {
    LuaString(String),
    LuaAnyString(AnyLuaString),
    LuaNumber(i32),
    LuaBoolean(bool),
    LuaArray(Vec<(AnyHashableLuaValue, AnyHashableLuaValue)>),
    LuaNil,

    /// The "Other" element is (hopefully) temporary and will be replaced by "Function" and "Userdata".
    /// A panic! will trigger if you try to push a Other.
    LuaOther,
}

/// Represents any value that can be stored by Lua
#[derive(Clone, Debug, PartialEq)]
pub enum AnyLuaValue {
    LuaString(String),
    LuaAnyString(AnyLuaString),
    LuaNumber(f64),
    LuaBoolean(bool),
    LuaArray(Vec<(AnyLuaValue, AnyLuaValue)>),
    LuaNil,

    /// The "Other" element is (hopefully) temporary and will be replaced by "Function" and "Userdata".
    /// A panic! will trigger if you try to push a Other.
    LuaOther,
}

impl<'lua, L> Push<L> for AnyLuaValue
where
    L: AsMutLua<'lua>,
{
    type Err = Void; // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

    #[inline]
    fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
        let raw_lua = lua.as_lua();
        match self {
            AnyLuaValue::LuaString(val) => val.push_to_lua(lua),
            AnyLuaValue::LuaAnyString(val) => val.push_to_lua(lua),
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
                let size = val
                    .push_no_err(&mut lua as &mut AsMutLua<'lua>)
                    .forget_internal();

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
where
    L: AsMutLua<'lua>,
{
    #[inline]
    fn lua_read_at_position(mut lua: L, index: i32) -> Result<AnyLuaValue, L> {
        // If we know that the value on the stack is a string, we should try
        // to parse it as a string instead of a number or boolean, so that
        // values such as '1.10' don't become `AnyLuaValue::LuaNumber(1.1)`.
        let data_type = unsafe { ffi::lua_type(lua.as_lua().0, index) };
        if data_type == ffi::LUA_TSTRING {
            let mut lua =
                match LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index) {
                    Ok(v) => return Ok(AnyLuaValue::LuaString(v)),
                    Err(lua) => lua,
                };

            let _lua = match LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index) {
                Ok(v) => return Ok(AnyLuaValue::LuaAnyString(v)),
                Err(lua) => lua,
            };

            Ok(AnyLuaValue::LuaOther)
        } else {
            let mut lua =
                match LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index) {
                    Ok(v) => return Ok(AnyLuaValue::LuaNumber(v)),
                    Err(lua) => lua,
                };

            let mut lua =
                match LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index) {
                    Ok(v) => return Ok(AnyLuaValue::LuaBoolean(v)),
                    Err(lua) => lua,
                };

            let mut lua =
                match LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index) {
                    Ok(v) => return Ok(AnyLuaValue::LuaString(v)),
                    Err(lua) => lua,
                };

            let lua = match LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index) {
                Ok(v) => return Ok(AnyLuaValue::LuaAnyString(v)),
                Err(lua) => lua,
            };

            if unsafe { ffi::lua_isnil(lua.as_lua().0, index) } {
                return Ok(AnyLuaValue::LuaNil);
            }

            let table: Result<LuaTable<_>, _> = LuaRead::lua_read_at_position(lua, index);
            let _lua = match table {
                Ok(mut v) => {
                    return Ok(AnyLuaValue::LuaArray(
                        v.iter::<AnyLuaValue, AnyLuaValue>()
                            .filter_map(|e| e)
                            .collect(),
                    ))
                }
                Err(lua) => lua,
            };

            Ok(AnyLuaValue::LuaOther)
        }
    }
}

impl<'lua, L> Push<L> for AnyHashableLuaValue
where
    L: AsMutLua<'lua>,
{
    type Err = Void; // TODO: use `!` instead (https://github.com/rust-lang/rust/issues/35121)

    #[inline]
    fn push_to_lua(self, mut lua: L) -> Result<PushGuard<L>, (Void, L)> {
        let raw_lua = lua.as_lua();
        match self {
            AnyHashableLuaValue::LuaString(val) => val.push_to_lua(lua),
            AnyHashableLuaValue::LuaAnyString(val) => val.push_to_lua(lua),
            AnyHashableLuaValue::LuaNumber(val) => val.push_to_lua(lua),
            AnyHashableLuaValue::LuaBoolean(val) => val.push_to_lua(lua),
            AnyHashableLuaValue::LuaArray(val) => {
                // Pushing a `Vec<(AnyHashableLuaValue, AnyHashableLuaValue)>` on a `L` requires calling the
                // function that pushes a `AnyHashableLuaValue` on a `&mut L`, which in turns requires
                // calling the function that pushes a `AnyHashableLuaValue` on a `&mut &mut L`, and so on.
                // In order to avoid this infinite recursion, we push the array on a
                // `&mut AsMutLua` instead.

                // We also need to destroy and recreate the push guard, otherwise the type parameter
                // doesn't match.
                let size = val
                    .push_no_err(&mut lua as &mut AsMutLua<'lua>)
                    .forget_internal();

                Ok(PushGuard {
                    lua: lua,
                    size: size,
                    raw_lua: raw_lua,
                })
            }
            AnyHashableLuaValue::LuaNil => {
                unsafe {
                    ffi::lua_pushnil(lua.as_mut_lua().0);
                }
                Ok(PushGuard {
                    lua: lua,
                    size: 1,
                    raw_lua: raw_lua,
                })
            } // Use ffi::lua_pushnil.
            AnyHashableLuaValue::LuaOther => {
                panic!("can't push a AnyHashableLuaValue of type Other")
            }
        }
    }
}

impl<'lua, L> PushOne<L> for AnyHashableLuaValue where L: AsMutLua<'lua> {}

impl<'lua, L> LuaRead<L> for AnyHashableLuaValue
where
    L: AsMutLua<'lua>,
{
    #[inline]
    fn lua_read_at_position(mut lua: L, index: i32) -> Result<AnyHashableLuaValue, L> {
        let data_type = unsafe { ffi::lua_type(lua.as_lua().0, index) };
        if data_type == ffi::LUA_TSTRING {
            let mut lua =
                match LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index) {
                    Ok(v) => return Ok(AnyHashableLuaValue::LuaString(v)),
                    Err(lua) => lua,
                };

            let _lua = match LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index) {
                Ok(v) => return Ok(AnyHashableLuaValue::LuaAnyString(v)),
                Err(lua) => lua,
            };

            Ok(AnyHashableLuaValue::LuaOther)
        } else {
            let mut lua =
                match LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index) {
                    Ok(v) => return Ok(AnyHashableLuaValue::LuaNumber(v)),
                    Err(lua) => lua,
                };

            let mut lua =
                match LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index) {
                    Ok(v) => return Ok(AnyHashableLuaValue::LuaBoolean(v)),
                    Err(lua) => lua,
                };

            let mut lua =
                match LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index) {
                    Ok(v) => return Ok(AnyHashableLuaValue::LuaString(v)),
                    Err(lua) => lua,
                };

            let mut lua =
                match LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index) {
                    Ok(v) => return Ok(AnyHashableLuaValue::LuaAnyString(v)),
                    Err(lua) => lua,
                };

            if unsafe { ffi::lua_isnil(lua.as_lua().0, index) } {
                return Ok(AnyHashableLuaValue::LuaNil);
            }

            let table: Result<LuaTable<_>, _> =
                LuaRead::lua_read_at_position(&mut lua as &mut AsMutLua<'lua>, index);
            let _lua = match table {
                Ok(mut v) => {
                    return Ok(AnyHashableLuaValue::LuaArray(
                        v.iter::<AnyHashableLuaValue, AnyHashableLuaValue>()
                            .filter_map(|e| e)
                            .collect(),
                    ))
                }
                Err(lua) => lua,
            };

            Ok(AnyHashableLuaValue::LuaOther)
        }
    }
}

impl From<AnyHashableLuaValue> for AnyLuaValue {
    fn from(value: AnyHashableLuaValue) -> AnyLuaValue {
        match value {
            AnyHashableLuaValue::LuaString(string) => {
                AnyLuaValue::LuaString(string)
            },
            AnyHashableLuaValue::LuaAnyString(any_string) => {
                AnyLuaValue::LuaAnyString(any_string)
            },
            AnyHashableLuaValue::LuaNumber(integral) => {
                AnyLuaValue::LuaNumber(integral as f64)
            },
            AnyHashableLuaValue::LuaBoolean(boolean) => {
                AnyLuaValue::LuaBoolean(boolean)
            },
            AnyHashableLuaValue::LuaArray(array) => {
                AnyLuaValue::LuaArray(array.into_iter().map(|(key, value)|
                    (From::from(key), From::from(value))
                ).collect())
            },
            AnyHashableLuaValue::LuaNil => {
                AnyLuaValue::LuaNil
            },
            AnyHashableLuaValue::LuaOther => {
                AnyLuaValue::LuaOther
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use AnyHashableLuaValue;
    use AnyLuaString;
    use AnyLuaValue;
    use Lua;

    #[test]
    fn read_numbers() {
        let mut lua = Lua::new();

        lua.set("a", "-2");
        lua.set("b", 3.5f32);
        lua.set("c", -2.0f32);

        let x: AnyLuaValue = lua.get("a").unwrap();
        assert_eq!(x, AnyLuaValue::LuaString("-2".to_owned()));

        let y: AnyLuaValue = lua.get("b").unwrap();
        assert_eq!(y, AnyLuaValue::LuaNumber(3.5));

        let z: AnyLuaValue = lua.get("c").unwrap();
        assert_eq!(z, AnyLuaValue::LuaNumber(-2.0));
    }

    #[test]
    fn read_hashable_numbers() {
        let mut lua = Lua::new();

        lua.set("a", -2.0f32);
        lua.set("b", 4.0f32);
        lua.set("c", "4");

        let x: AnyHashableLuaValue = lua.get("a").unwrap();
        assert_eq!(x, AnyHashableLuaValue::LuaNumber(-2));

        let y: AnyHashableLuaValue = lua.get("b").unwrap();
        assert_eq!(y, AnyHashableLuaValue::LuaNumber(4));

        let z: AnyHashableLuaValue = lua.get("c").unwrap();
        assert_eq!(z, AnyHashableLuaValue::LuaString("4".to_owned()));
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
    fn read_hashable_strings() {
        let mut lua = Lua::new();

        lua.set("a", "hello");
        lua.set("b", "3x");
        lua.set("c", "false");

        let x: AnyHashableLuaValue = lua.get("a").unwrap();
        assert_eq!(x, AnyHashableLuaValue::LuaString("hello".to_string()));

        let y: AnyHashableLuaValue = lua.get("b").unwrap();
        assert_eq!(y, AnyHashableLuaValue::LuaString("3x".to_string()));

        let z: AnyHashableLuaValue = lua.get("c").unwrap();
        assert_eq!(z, AnyHashableLuaValue::LuaString("false".to_string()));
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
    fn read_hashable_booleans() {
        let mut lua = Lua::new();

        lua.set("a", true);
        lua.set("b", false);

        let x: AnyHashableLuaValue = lua.get("a").unwrap();
        assert_eq!(x, AnyHashableLuaValue::LuaBoolean(true));

        let y: AnyHashableLuaValue = lua.get("b").unwrap();
        assert_eq!(y, AnyHashableLuaValue::LuaBoolean(false));
    }

    #[test]
    fn read_tables() {
        let mut lua = Lua::new();
        lua.execute::<()>(
            "
        a = {x = 12, y = 19}
        b = {z = a, w = 'test string'}
        c = {'first', 'second'}
        ",
        )
        .unwrap();

        fn get<'a>(table: &'a AnyLuaValue, key: &str) -> &'a AnyLuaValue {
            let test_key = AnyLuaValue::LuaString(key.to_owned());
            match table {
                &AnyLuaValue::LuaArray(ref vec) => {
                    let &(_, ref value) = vec
                        .iter()
                        .find(|&&(ref key, _)| key == &test_key)
                        .expect("key not found");
                    value
                }
                _ => panic!("not a table"),
            }
        }

        fn get_numeric<'a>(table: &'a AnyLuaValue, key: usize) -> &'a AnyLuaValue {
            let test_key = AnyLuaValue::LuaNumber(key as f64);
            match table {
                &AnyLuaValue::LuaArray(ref vec) => {
                    let &(_, ref value) = vec
                        .iter()
                        .find(|&&(ref key, _)| key == &test_key)
                        .expect("key not found");
                    value
                }
                _ => panic!("not a table"),
            }
        }

        let a: AnyLuaValue = lua.get("a").unwrap();
        assert_eq!(get(&a, "x"), &AnyLuaValue::LuaNumber(12.0));
        assert_eq!(get(&a, "y"), &AnyLuaValue::LuaNumber(19.0));

        let b: AnyLuaValue = lua.get("b").unwrap();
        assert_eq!(get(&get(&b, "z"), "x"), get(&a, "x"));
        assert_eq!(get(&get(&b, "z"), "y"), get(&a, "y"));

        let c: AnyLuaValue = lua.get("c").unwrap();
        assert_eq!(
            get_numeric(&c, 1),
            &AnyLuaValue::LuaString("first".to_owned())
        );
        assert_eq!(
            get_numeric(&c, 2),
            &AnyLuaValue::LuaString("second".to_owned())
        );
    }

    #[test]
    fn read_hashable_tables() {
        let mut lua = Lua::new();
        lua.execute::<()>(
            "
        a = {x = 12, y = 19}
        b = {z = a, w = 'test string'}
        c = {'first', 'second'}
        ",
        )
        .unwrap();

        fn get<'a>(table: &'a AnyHashableLuaValue, key: &str) -> &'a AnyHashableLuaValue {
            let test_key = AnyHashableLuaValue::LuaString(key.to_owned());
            match table {
                &AnyHashableLuaValue::LuaArray(ref vec) => {
                    let &(_, ref value) = vec
                        .iter()
                        .find(|&&(ref key, _)| key == &test_key)
                        .expect("key not found");
                    value
                }
                _ => panic!("not a table"),
            }
        }

        fn get_numeric<'a>(table: &'a AnyHashableLuaValue, key: usize) -> &'a AnyHashableLuaValue {
            let test_key = AnyHashableLuaValue::LuaNumber(key as i32);
            match table {
                &AnyHashableLuaValue::LuaArray(ref vec) => {
                    let &(_, ref value) = vec
                        .iter()
                        .find(|&&(ref key, _)| key == &test_key)
                        .expect("key not found");
                    value
                }
                _ => panic!("not a table"),
            }
        }

        let a: AnyHashableLuaValue = lua.get("a").unwrap();
        assert_eq!(get(&a, "x"), &AnyHashableLuaValue::LuaNumber(12));
        assert_eq!(get(&a, "y"), &AnyHashableLuaValue::LuaNumber(19));

        let b: AnyHashableLuaValue = lua.get("b").unwrap();
        assert_eq!(get(&get(&b, "z"), "x"), get(&a, "x"));
        assert_eq!(get(&get(&b, "z"), "y"), get(&a, "y"));

        let c: AnyHashableLuaValue = lua.get("c").unwrap();
        assert_eq!(
            get_numeric(&c, 1),
            &AnyHashableLuaValue::LuaString("first".to_owned())
        );
        assert_eq!(
            get_numeric(&c, 2),
            &AnyHashableLuaValue::LuaString("second".to_owned())
        );
    }

    #[test]
    fn push_numbers() {
        let mut lua = Lua::new();

        lua.set("a", AnyLuaValue::LuaNumber(3.0));

        let x: i32 = lua.get("a").unwrap();
        assert_eq!(x, 3);
    }

    #[test]
    fn push_hashable_numbers() {
        let mut lua = Lua::new();

        lua.set("a", AnyHashableLuaValue::LuaNumber(3));

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
    fn push_hashable_strings() {
        let mut lua = Lua::new();

        lua.set("a", AnyHashableLuaValue::LuaString("hello".to_string()));

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
    fn push_hashable_booleans() {
        let mut lua = Lua::new();

        lua.set("a", AnyHashableLuaValue::LuaBoolean(true));

        let x: bool = lua.get("a").unwrap();
        assert_eq!(x, true);
    }

    #[test]
    fn push_nil() {
        let mut lua = Lua::new();

        lua.set("a", AnyLuaValue::LuaNil);

        let x: Option<i32> = lua.get("a");
        assert!(
            x.is_none(),
            "x is a Some value when it should be a None value. X: {:?}",
            x
        );
    }

    #[test]
    fn push_hashable_nil() {
        let mut lua = Lua::new();

        lua.set("a", AnyHashableLuaValue::LuaNil);

        let x: Option<i32> = lua.get("a");
        assert!(
            x.is_none(),
            "x is a Some value when it should be a None value. X: {:?}",
            x
        );
    }

    #[test]
    fn non_utf_8_string() {
        let mut lua = Lua::new();
        let a = lua
            .execute::<AnyLuaValue>(r"return '\xff\xfe\xff\xfe'")
            .unwrap();
        match a {
            AnyLuaValue::LuaAnyString(AnyLuaString(v)) => {
                assert_eq!(Vec::from(&b"\xff\xfe\xff\xfe"[..]), v);
            }
            _ => panic!("Decoded to wrong variant"),
        }
    }
}
