use {HasLua, Push, CopyRead, ConsumeRead, LoadedVariable};

/// Represents any value that can be stored by Lua
#[experimental]
#[deriving(Clone,Show,PartialEq)]
pub enum AnyLuaValue {
    LuaString(String),
    LuaNumber(f64),
    LuaBoolean(bool),
    LuaArray(Vec<(AnyLuaValue, AnyLuaValue)>),

    /// The "Other" element is (hopefully) temporary and will be replaced by "Function" and "Userdata".
    /// A fail! will trigger if you try to push a Other.
    LuaOther
}

impl<L: HasLua> Push<L> for AnyLuaValue {
    fn push_to_lua(self, lua: &mut L) -> uint {
        match self {
            LuaString(val) => val.push_to_lua(lua),
            LuaNumber(val) => val.push_to_lua(lua),
            LuaBoolean(val) => val.push_to_lua(lua),
            LuaArray(val) => val.push_to_lua(lua),
            LuaOther => fail!("can't push a AnyLuaValue of type Other")
        }
    }
}

impl<L: HasLua> CopyRead<L> for AnyLuaValue {
    fn read_from_lua(lua: &mut L, index: i32) -> Option<AnyLuaValue> {
        None
            .or_else(|| CopyRead::read_from_lua(lua, index).map(|v| LuaNumber(v)))
            .or_else(|| CopyRead::read_from_lua(lua, index).map(|v| LuaBoolean(v)))
            .or_else(|| CopyRead::read_from_lua(lua, index).map(|v| LuaString(v)))
            //.or_else(|| CopyRead::read_from_lua(lua, index).map(|v| LuaArray(v)))
            .or_else(|| Some(LuaOther))
    }
}

impl<'a, L: HasLua> ConsumeRead<'a, L> for AnyLuaValue {
    fn read_from_variable(mut var: LoadedVariable<'a, L>) -> Result<AnyLuaValue, LoadedVariable<'a, L>> {
        match CopyRead::read_from_lua(&mut var, -1) {
            None => Err(var),
            Some(a) => Ok(a)
        }
    }
}
