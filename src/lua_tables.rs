use { CopyReadable, ConsumeReadable, LoadedVariable, Pushable };
use liblua;

pub struct LuaTable<'a> {
    variable: LoadedVariable<'a>
}

// while the LuaTableIterator is active, the current key is constantly pushed over the table
pub struct LuaTableIterator<'a, 'b> {
    table: &'b mut LuaTable<'a>
}

impl<'a> ConsumeReadable<'a> for LuaTable<'a> {
    fn read_from_variable(var: LoadedVariable<'a>)
        -> Result<LuaTable<'a>, LoadedVariable<'a>>
    {
        if unsafe { liblua::lua_istable(var.lua.lua, -1) } {
            Ok(LuaTable{ variable: var })
        } else {
            Err(var)
        }
    }
}

impl<'a> LuaTable<'a> {
    pub fn iter<'b>(&'b mut self)
        -> LuaTableIterator<'a, 'b>
    {
        ().push_to_lua(self.variable.lua);
        LuaTableIterator { table: self }
    }
}

impl<'a, 'b, K: CopyReadable, V: CopyReadable> Iterator<Option<(K,V)>> for LuaTableIterator<'a, 'b> {
    fn next(&mut self)
        -> Option<Option<(K,V)>>
    {
        // this call pushes the next key and value on the stack
        if unsafe { liblua::lua_next(self.table.variable.lua.lua, -2) } == 0 {
            return None
        }

        let key = CopyReadable::read_from_lua(self.table.variable.lua, -2);
        let value = CopyReadable::read_from_lua(self.table.variable.lua, -1);

        // removing the value, leaving only the key on the top of the stack
        unsafe { liblua::lua_pop(self.table.variable.lua.lua, 1) };

        //
        if key.is_none() || value.is_none() {
            Some(None)
        } else {
            Some(Some((key.unwrap(), value.unwrap())))
        }
    }
}

// TODO: this destructor crashes the compiler
/*impl<'a, 'b> Drop for LuaTableIterator<'a, 'b> {
    fn drop(&mut self) {
        unsafe { liblua::lua_pop(self.table.variable.lua.lua, 1) }
    }
}*/

#[cfg(test)]
mod tests {
    use Lua;
    use lua_tables::LuaTable;

    #[test]
    fn iterable() {
        let mut lua = Lua::new();

        let _: () = lua.execute("a = { 9, 8, 7 }").unwrap();

        let mut table: LuaTable = lua.get("a").unwrap();

        let tableContent: Vec<Option<(uint, uint)>> = table.iter().collect();

        assert_eq!(tableContent, vec!( Some((1,9)), Some((2,8)), Some((3,7)) ));
    }
}
