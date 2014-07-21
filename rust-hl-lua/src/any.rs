use {Lua, HasLua, Push, CopyRead, ConsumeRead, LoadedVariable};

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

impl<L: HasLua> Push<L> for AnyLuaValue {
    fn push_to_lua(self, lua: &mut L) -> uint {
        match self {
            String(val) => val.push_to_lua(lua),
            Number(val) => val.push_to_lua(lua),
            Boolean(val) => val.push_to_lua(lua),
            Array(val) => val.push_to_lua(lua),
            Other => fail!("can't push a AnyLuaValue of type Other")
        }
    }
}

impl<L: HasLua> CopyRead<L> for AnyLuaValue {
    fn read_from_lua(lua: &mut L, index: i32) -> Option<AnyLuaValue> {
        None
            .or_else(|| CopyRead::read_from_lua(lua, index).map(|v| Number(v)))
            .or_else(|| CopyRead::read_from_lua(lua, index).map(|v| Boolean(v)))
            .or_else(|| CopyRead::read_from_lua(lua, index).map(|v| String(v)))
            //.or_else(|| CopyRead::read_from_lua(lua, index).map(|v| Array(v)))
            .or_else(|| Some(Other))
    }
}

impl<'a,'lua> ConsumeRead<'a,'lua> for AnyLuaValue {
    fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<AnyLuaValue, LoadedVariable<'a, 'lua>> {
        match CopyRead::read_from_lua(var.lua, -1) {
            None => Err(var),
            Some(a) => Ok(a)
        }
    }
}
