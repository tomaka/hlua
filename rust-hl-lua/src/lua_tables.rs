use {HasLua, CopyRead, ConsumeRead, LoadedVariable, Push, Index};
use ffi;

#[unstable]
pub struct LuaTable<L> {
    lua: L,
}

impl<L: HasLua> HasLua for LuaTable<L> {
    fn use_lua(&mut self) -> *mut ffi::lua_State {
        self.variable.use_lua()
    }
}

// while the LuaTableIterator is active, the current key is constantly pushed over the table
#[unstable]
pub struct LuaTableIterator<'table, L: 'table> {
    table: &'table mut LuaTable<L>,
    finished: bool,     // if true, the key is not on the stack anymore
}

impl<'table, L: HasLua + 'table> HasLua for LuaTableIterator<'table, L> {
    fn use_lua(&mut self) -> *mut ffi::lua_State {
        self.table.use_lua()
    }
}

impl<'a, L: HasLua + 'a> ConsumeRead<'a, L> for LuaTable<L> {
    fn read_from_variable(mut var: LoadedVariable<L>)
        -> Result<LuaTable<L>, LoadedVariable<L>>
    {
        if unsafe { ffi::lua_istable(var.use_lua(), -1) } {
            Ok(LuaTable{ variable: var })
        } else {
            Err(var)
        }
    }
}

impl<L: HasLua> LuaTable<L> {
    pub fn iter<'t>(&'t mut self)
        -> LuaTableIterator<'t, L>
    {
        unsafe { ffi::lua_pushnil(self.variable.use_lua()) };
        LuaTableIterator{table: self, finished: false}
    }

    pub fn load<'a, R: ConsumeRead<'a, LuaTable<L>>, I: Index<LuaTable<L>>>(&'a mut self, index: I) -> Option<R> {
        index.push_to_lua(self);
        unsafe { ffi::lua_gettable(self.use_lua(), -2); }
        let var = LoadedVariable{lua: self, size: 1};
        ConsumeRead::read_from_variable(var).ok()
    }

    pub fn load_table<'a, I: Index<LuaTable<L>>>(&'a mut self, index: I) -> Option<LuaTable<LuaTable<L>>> {
        self.load(index)
    }

    pub fn get<R: CopyRead<LuaTable<L>>, I: Index<LuaTable<L>>>(&mut self, index: I) -> Option<R> {
        index.push_to_lua(self);
        unsafe { ffi::lua_gettable(self.use_lua(), -2); }
        let value = CopyRead::read_from_lua(self, -1);
        unsafe { ffi::lua_pop(self.use_lua(), 1); }
        value
    }

    pub fn set<I: Index<LuaTable<L>>, V: Push<LuaTable<L>>>(&mut self, index: I, value: V) {
        index.push_to_lua(self);
        value.push_to_lua(self);
        unsafe { ffi::lua_settable(self.use_lua(), -3); }
    }

    // Obtains or create the metatable of the table
    pub fn get_or_create_metatable(mut self) -> LuaTable<L> {
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

impl<'a, L: HasLua, K: CopyRead<LuaTableIterator<'a, L>>, V: CopyRead<LuaTableIterator<'a, L>>>
    Iterator<Option<(K, V)>> for LuaTableIterator<'a, L>
{
    fn next(&mut self)
        -> Option<Option<(K,V)>>
    {
        if self.finished {
            return None
        }

        // this call pushes the next key and value on the stack
        if unsafe { ffi::lua_next(self.table.use_lua(), -2) } == 0 {
            self.finished = true;
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

#[unsafe_destructor]
impl<'a, L: HasLua> Drop for LuaTableIterator<'a, L> {
    fn drop(&mut self) {
        if !self.finished {
            unsafe { ffi::lua_pop(self.table.variable.use_lua(), 1) }
        }
    }
}
