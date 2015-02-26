use std::marker::PhantomData;

use ffi;
use LuaContext;

use AsLua;
use AsMutLua;

use LuaRead;

/// 
pub struct LuaTable<L> {
    table: L,
}

unsafe impl<L> AsLua for LuaTable<L> where L: AsLua {
    fn as_lua(&self) -> LuaContext {
        self.table.as_lua()
    }
}

unsafe impl<L> AsMutLua for LuaTable<L> where L: AsMutLua {
    fn as_mut_lua(&mut self) -> LuaContext {
        self.table.as_mut_lua()
    }
}

impl<L> LuaRead<L> for LuaTable<L> where L: AsMutLua {
    fn lua_read_at_position(mut lua: L, index: i32) -> Option<LuaTable<L>> {
        assert!(index == -1);   // FIXME: not sure if it's working
        if unsafe { ffi::lua_istable(lua.as_mut_lua().0, index) } {
            Some(LuaTable { table: lua })
        } else {
            None
        }
    }
}

// while the LuaTableIterator is active, the current key is constantly pushed over the table
pub struct LuaTableIterator<'t, L: 't, K, V> {
    table: &'t mut LuaTable<L>,
    finished: bool,     // if true, the key is not on the stack anymore
    marker: PhantomData<(K, V)>,
}

unsafe impl<'t, L, K, V> AsLua for LuaTableIterator<'t, L, K, V> where L: AsLua {
    fn as_lua(&self) -> LuaContext {
        self.table.as_lua()
    }
}

unsafe impl<'t, L, K, V> AsMutLua for LuaTableIterator<'t, L, K, V> where L: AsMutLua {
    fn as_mut_lua(&mut self) -> LuaContext {
        self.table.as_mut_lua()
    }
}

impl<L> LuaTable<L> where L: AsMutLua {
    pub fn iter<K, V>(&mut self) -> LuaTableIterator<L, K, V> {
        unsafe { ffi::lua_pushnil(self.table.as_mut_lua().0) };

        LuaTableIterator {
            table: self,
            finished: false,
            marker: PhantomData,
        }
    }

    /*pub fn load<'a, R: ConsumeRead<'a, LuaTable<'var, L>>, I: Index<LuaTable<'var, L>>>(&'a mut self, index: I) -> Option<R> {
        index.push_to_lua(self);
        unsafe { ffi::lua_gettable(self.as_lua(), -2); }
        let var = LoadedVariable{lua: self, size: 1};
        ConsumeRead::read_from_variable(var).ok()
    }

    pub fn load_table<'a, I: Index<LuaTable<'var, L>>>(&'a mut self, index: I) -> Option<LuaTable<'a, LuaTable<'var, L>>> {
        self.load(index)
    }

    pub fn get<R: CopyRead<LuaTable<'var, L>>, I: Index<LuaTable<'var, L>>>(&mut self, index: I) -> Option<R> {
        index.push_to_lua(self);
        unsafe { ffi::lua_gettable(self.as_lua(), -2); }
        let value = CopyRead::read_from_lua(self, -1);
        unsafe { ffi::lua_pop(self.as_lua(), 1); }
        value
    }

    pub fn set<I: Index<LuaTable<'var, L>>, V: Push<LuaTable<'var, L>>>(&mut self, index: I, value: V) {
        index.push_to_lua(self);
        value.push_to_lua(self);
        unsafe { ffi::lua_settable(self.as_lua(), -3); }
    }

    // Obtains or create the metatable of the table
    pub fn get_or_create_metatable(mut self) -> LuaTable<'var, L> {
        let result = unsafe { ffi::lua_getmetatable(self.variable.as_lua(), -1) };

        if result == 0 {
            unsafe {
                ffi::lua_newtable(self.variable.as_lua());
                ffi::lua_setmetatable(self.variable.as_lua(), -2);
                let r = ffi::lua_getmetatable(self.variable.as_lua(), -1);
                assert!(r != 0);
            }
        }

        // note: it would be cleaner to create another table, but cannot manage to make it compile
        self.variable.size += 1;
        self
    }*/
}

impl<'t, L, K, V> Iterator for LuaTableIterator<'t, L, K, V>
                  where L: AsMutLua + 't,
                        K: for<'i, 'j> LuaRead<&'i mut &'j mut LuaTableIterator<'t, L, K, V>> + 'static,
                        V: for<'i, 'j> LuaRead<&'i mut &'j mut LuaTableIterator<'t, L, K, V>> + 'static
{
    type Item = Option<(K, V)>;

    fn next(&mut self) -> Option<Option<(K,V)>> {
        if self.finished {
            return None;
        }

        // this call pushes the next key and value on the stack
        if unsafe { ffi::lua_next(self.table.as_mut_lua().0, -2) } == 0 {
            self.finished = true;
            return None;
        }

        let mut me = self;
        let key = LuaRead::lua_read_at_position(&mut me, -2);
        let value = LuaRead::lua_read_at_position(&mut me, -1);

        // removing the value, leaving only the key on the top of the stack
        unsafe { ffi::lua_pop(me.table.as_mut_lua().0, 1) };

        //
        if key.is_none() || value.is_none() {
            Some(None)
        } else {
            Some(Some((key.unwrap(), value.unwrap())))
        }
    }
}

#[unsafe_destructor]
impl<'t, L, K, V> Drop for LuaTableIterator<'t, L, K, V> where L: AsMutLua + 't {
    fn drop(&mut self) {
        if !self.finished {
            unsafe { ffi::lua_pop(self.table.table.as_mut_lua().0, 1) }
        }
    }
}
