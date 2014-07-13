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
