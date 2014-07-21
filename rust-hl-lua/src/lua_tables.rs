use {HasLua, Lua, CopyRead, ConsumeRead, LoadedVariable, Push, Index};
use ffi;

#[unstable]
pub struct LuaTable<'var, 'lua> {
    variable: LoadedVariable<'var, 'lua>
}

impl<'var, 'lua> HasLua<'lua> for LuaTable<'var, 'lua> {
    fn use_lua(&mut self) -> *mut ffi::lua_State {
        self.variable.use_lua()
    }
}

// while the LuaTableIterator is active, the current key is constantly pushed over the table
#[unstable]
pub struct LuaTableIterator<'var, 'lua, 'table> {
    table: &'table mut LuaTable<'var, 'lua>
}

impl<'var, 'lua, 'table> HasLua<'lua> for LuaTableIterator<'var, 'lua, 'table> {
    fn use_lua(&mut self) -> *mut ffi::lua_State {
        self.table.use_lua()
    }
}

impl<'var, 'lua> ConsumeRead<'var, 'lua> for LuaTable<'var, 'lua> {
    fn read_from_variable(var: LoadedVariable<'var, 'lua>)
        -> Result<LuaTable<'var, 'lua>, LoadedVariable<'var, 'lua>>
    {
        if unsafe { ffi::lua_istable(var.lua.lua, -1) } {
            Ok(LuaTable{ variable: var })
        } else {
            Err(var)
        }
    }
}

impl<'var, 'lua> LuaTable<'var, 'lua> {
    pub fn iter<'me>(&'me mut self)
        -> LuaTableIterator<'var, 'lua, 'me>
    {
        unsafe { ffi::lua_pushnil(self.variable.lua.lua) };
        LuaTableIterator { table: self }
    }

    pub fn get<R: CopyRead<LuaTable<'var, 'lua>>, I: Index<Lua<'lua>>>(&mut self, index: I) -> Option<R> {
        index.push_to_lua(self.variable.lua);
        unsafe { ffi::lua_gettable(self.variable.lua.lua, -2); }
        let value = CopyRead::read_from_lua(self, -1);
        unsafe { ffi::lua_pop(self.variable.lua.lua, 1); }
        value
    }

    pub fn set<I: Index<Lua<'lua>>, V: Push<Lua<'lua>>>(&mut self, index: I, value: V) {
        index.push_to_lua(self.variable.lua);
        value.push_to_lua(self.variable.lua);
        unsafe { ffi::lua_settable(self.variable.lua.lua, -3); }
    }

    // Obtains or create the metatable of the table
    pub fn get_or_create_metatable(mut self) -> LuaTable<'var, 'lua> {
        let result = unsafe { ffi::lua_getmetatable(self.variable.lua.lua, -1) };

        if result == 0 {
            unsafe {
                ffi::lua_newtable(self.variable.lua.lua);
                ffi::lua_setmetatable(self.variable.lua.lua, -2);
                let r = ffi::lua_getmetatable(self.variable.lua.lua, -1);
                assert!(r != 0);
            }
        }

        // note: it would be cleaner to create another table, but cannot manage to make it compile
        self.variable.size += 1;
        self
    }
}

impl<'a, 'b, 'c, K: CopyRead<LuaTableIterator<'a, 'b, 'c>>, V: CopyRead<LuaTableIterator<'a, 'b, 'c>>>
    Iterator<Option<(K, V)>> for LuaTableIterator<'a, 'b, 'c>
{
    fn next(&mut self)
        -> Option<Option<(K,V)>>
    {
        // this call pushes the next key and value on the stack
        if unsafe { ffi::lua_next(self.table.variable.lua.lua, -2) } == 0 {
            return None
        }

        let key = CopyRead::read_from_lua(self, -2);
        let value = CopyRead::read_from_lua(self, -1);

        // removing the value, leaving only the key on the top of the stack
        unsafe { ffi::lua_pop(self.table.variable.lua.lua, 1) };

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
        unsafe { ffi::lua_pop(self.table.variable.lua.lua, 1) }
    }
}*/
