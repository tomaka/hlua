use {HasLua, CopyRead, ConsumeRead, LoadedVariable, Push, Index};
use ffi;

#[unstable]
pub struct LuaTable<'var, L> {
    variable: LoadedVariable<'var, L>
}

impl<'var, 'lua, L: HasLua> HasLua for LuaTable<'var, L> {
    fn use_lua(&mut self) -> *mut ffi::lua_State {
        self.variable.use_lua()
    }
}

// while the LuaTableIterator is active, the current key is constantly pushed over the table
#[unstable]
pub struct LuaTableIterator<'var, 'table, L> {
    table: &'table mut LuaTable<'var, L>
}

impl<'var, 'lua, 'table, L: HasLua> HasLua for LuaTableIterator<'var, 'table, L> {
    fn use_lua(&mut self) -> *mut ffi::lua_State {
        self.table.use_lua()
    }
}

impl<'var, 'lua, L: HasLua> ConsumeRead<'var, L> for LuaTable<'var, L> {
    fn read_from_variable(mut var: LoadedVariable<'var, L>)
        -> Result<LuaTable<'var, L>, LoadedVariable<'var, L>>
    {
        if unsafe { ffi::lua_istable(var.use_lua(), -1) } {
            Ok(LuaTable{ variable: var })
        } else {
            Err(var)
        }
    }
}

impl<'var, 'lua, L: HasLua> LuaTable<'var, L> {
    pub fn iter<'me>(&'me mut self)
        -> LuaTableIterator<'var, 'me, L>
    {
        unsafe { ffi::lua_pushnil(self.variable.use_lua()) };
        LuaTableIterator { table: self }
    }

    pub fn get<R: CopyRead<LuaTable<'var, L>>, I: Index<LuaTable<'var, L>>>(&mut self, index: I) -> Option<R> {
        index.push_to_lua(self);
        unsafe { ffi::lua_gettable(self.use_lua(), -2); }
        let value = CopyRead::read_from_lua(self, -1);
        unsafe { ffi::lua_pop(self.use_lua(), 1); }
        value
    }

    pub fn set<I: Index<LuaTable<'var, L>>, V: Push<LuaTable<'var, L>>>(&mut self, index: I, value: V) {
        index.push_to_lua(self);
        value.push_to_lua(self);
        unsafe { ffi::lua_settable(self.use_lua(), -3); }
    }

    // Obtains or create the metatable of the table
    pub fn get_or_create_metatable(mut self) -> LuaTable<'var, L> {
        let result = unsafe { ffi::lua_getmetatable(self.variable.use_lua(), -1) };

        if result == 0 {
            unsafe {
                ffi::lua_newtable(self.variable.use_lua());
                ffi::lua_setmetatable(self.variable.use_lua(), -2);
                let r = ffi::lua_getmetatable(self.variable.use_lua(), -1);
                assert!(r != 0);
            }
        }

        // note: it would be cleaner to create another table, but cannot manage to make it compile
        self.variable.size += 1;
        self
    }
}

impl<'a, 'b, 'lua, L: HasLua, K: CopyRead<LuaTableIterator<'a, 'b, L>>, V: CopyRead<LuaTableIterator<'a, 'b, L>>>
    Iterator<Option<(K, V)>> for LuaTableIterator<'a, 'b, L>
{
    fn next(&mut self)
        -> Option<Option<(K,V)>>
    {
        // this call pushes the next key and value on the stack
        if unsafe { ffi::lua_next(self.table.use_lua(), -2) } == 0 {
            return None
        }

        let key = CopyRead::read_from_lua(self, -2);
        let value = CopyRead::read_from_lua(self, -1);

        // removing the value, leaving only the key on the top of the stack
        unsafe { ffi::lua_pop(self.table.use_lua(), 1) };

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
