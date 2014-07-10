use {Lua, Pushable, CopyReadable, ConsumeReadable, LoadedVariable};

/// Represents any value that can be stored by Lua
#[experimental]
#[deriving(Clone,Show,PartialEq)]
pub enum AnyLuaValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<(AnyLuaValue, AnyLuaValue)>),

    /// The "Other" element is (hopefully) temporary and will be replaced by "Function" and "Userdata".
    /// A fail! will trigger if you try to push a Other.
    Other
}

impl<'lua> Pushable<'lua> for AnyLuaValue {
    fn push_to_lua(self, lua: &mut Lua) -> uint {
        match self {
            String(val) => val.push_to_lua(lua),
            Number(val) => val.push_to_lua(lua),
            Boolean(val) => val.push_to_lua(lua),
            Array(val) => val.push_to_lua(lua),
            Other => fail!("can't push a AnyLuaValue of type Other")
        }
    }
}

impl CopyReadable for AnyLuaValue {
    fn read_from_lua<'lua>(lua: &mut Lua<'lua>, index: i32) -> Option<AnyLuaValue> {
        None
            .or_else(|| CopyReadable::read_from_lua(lua, index).map(|v| Number(v)))
            .or_else(|| CopyReadable::read_from_lua(lua, index).map(|v| Boolean(v)))
            .or_else(|| CopyReadable::read_from_lua(lua, index).map(|v| String(v)))
            //.or_else(|| CopyReadable::read_from_lua(lua, index).map(|v| Array(v)))
            .or_else(|| Some(Other))
    }
}

impl<'a,'lua> ConsumeReadable<'a,'lua> for AnyLuaValue {
    fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<AnyLuaValue, LoadedVariable<'a, 'lua>> {
        match CopyReadable::read_from_lua(var.lua, -1) {
            None => Err(var),
            Some(a) => Ok(a)
        }
    }
}

#[cfg(test)]
mod tests {
    use Lua;
    use super::{AnyLuaValue, Number, String, Boolean};

    #[test]
    fn read_numbers() {
        let mut lua = Lua::new();

        lua.set("a", "-2");
        lua.set("b", 3.5f32);

        let x: AnyLuaValue = lua.get("a").unwrap();
        assert_eq!(x, Number(-2.0));

        let y: AnyLuaValue = lua.get("b").unwrap();
        assert_eq!(y, Number(3.5));
    }

    #[test]
    fn read_strings() {
        let mut lua = Lua::new();

        lua.set("a", "hello");
        lua.set("b", "3x");
        lua.set("c", "false");

        let x: AnyLuaValue = lua.get("a").unwrap();
        assert_eq!(x, String("hello".to_string()));

        let y: AnyLuaValue = lua.get("b").unwrap();
        assert_eq!(y, String("3x".to_string()));

        let z: AnyLuaValue = lua.get("c").unwrap();
        assert_eq!(z, String("false".to_string()));
    }

    #[test]
    fn read_booleans() {
        let mut lua = Lua::new();

        lua.set("a", true);
        lua.set("b", false);

        let x: AnyLuaValue = lua.get("a").unwrap();
        assert_eq!(x, Boolean(true));

        let y: AnyLuaValue = lua.get("b").unwrap();
        assert_eq!(y, Boolean(false));
    }

    #[test]
    fn push_numbers() {
        let mut lua = Lua::new();

        lua.set("a", Number(3.0));

        let x: int = lua.get("a").unwrap();
        assert_eq!(x, 3);
    }

    #[test]
    fn push_strings() {
        let mut lua = Lua::new();

        lua.set("a", String("hello".to_string()));

        let x: String = lua.get("a").unwrap();
        assert_eq!(x.as_slice(), "hello");
    }

    #[test]
    fn push_booleans() {
        let mut lua = Lua::new();

        lua.set("a", Boolean(true));

        let x: bool = lua.get("a").unwrap();
        assert_eq!(x, true);
    }
}
