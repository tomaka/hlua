use {AsLua, Push, CopyRead, ConsumeRead, LoadedVariable};

/// Represents any value that can be stored by Lua
#[experimental]
#[deriving(Clone,Show,PartialEq)]
pub enum AnyLuaValue {
    LuaString(String),
    LuaNumber(f64),
    LuaBoolean(bool),
    LuaArray(Vec<(AnyLuaValue, AnyLuaValue)>),

    /// The "Other" element is (hopefully) temporary and will be replaced by "Function" and "Userdata".
    /// A panic! will trigger if you try to push a Other.
    LuaOther
}

impl<L> Push<L> for AnyLuaValue where L: AsLua {
    fn push_to_lua(self, lua: &mut L) -> uint {
        match self {
            AnyLuaValue::LuaString(val) => val.push_to_lua(lua),
            AnyLuaValue::LuaNumber(val) => val.push_to_lua(lua),
            AnyLuaValue::LuaBoolean(val) => val.push_to_lua(lua),
            AnyLuaValue::LuaArray(val) => val.push_to_lua(lua),
            AnyLuaValue::LuaOther => panic!("can't push a AnyLuaValue of type Other")
        }
    }
}

impl<L> CopyRead<L> for AnyLuaValue where L: AsLua {
    fn read_from_lua(lua: &mut L, index: i32) -> Option<AnyLuaValue> {
        None
            .or_else(|| CopyRead::read_from_lua(lua, index).map(|v| AnyLuaValue::LuaNumber(v)))
            .or_else(|| CopyRead::read_from_lua(lua, index).map(|v| AnyLuaValue::LuaBoolean(v)))
            .or_else(|| CopyRead::read_from_lua(lua, index).map(|v| AnyLuaValue::LuaString(v)))
            //.or_else(|| CopyRead::read_from_lua(lua, index).map(|v| LuaArray(v)))
            .or_else(|| Some(AnyLuaValue::LuaOther))
    }
}

impl<'a, L> ConsumeRead<'a, L> for AnyLuaValue where L: AsLua {
    fn read_from_variable(mut var: LoadedVariable<'a, L>) -> Result<AnyLuaValue, LoadedVariable<'a, L>> {
        match CopyRead::read_from_lua(&mut var, -1) {
            None => Err(var),
            Some(a) => Ok(a)
        }
    }
}
