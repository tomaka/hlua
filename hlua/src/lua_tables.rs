use std::marker::PhantomData;

use ffi;
use LuaContext;

use AsLua;
use AsMutLua;
use Push;
use PushGuard;
use LuaRead;

/// Represents a table stored in the Lua context.
///
/// Loading this type mutably borrows the Lua context.
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
    fn lua_read_at_position(mut lua: L, index: i32) -> Result<LuaTable<L>, L> {
        assert!(index == -1);   // FIXME: not sure if it's working
        if unsafe { ffi::lua_istable(lua.as_mut_lua().0, index) } {
            Ok(LuaTable { table: lua })
        } else {
            Err(lua)
        }
    }
}

/// Iterator that enumerates the content of a Lua table.
// while the LuaTableIterator is active, the current key is constantly pushed over the table
pub struct LuaTableIterator<'t, L: 't, K, V> where L: AsMutLua {
    table: &'t mut LuaTable<L>,
    finished: bool,     // if true, the key is not on the stack anymore
    marker: PhantomData<(K, V)>,
}

unsafe impl<'t, L, K, V> AsLua for LuaTableIterator<'t, L, K, V> where L: AsMutLua {
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
    /// Iterates over the elements inside the table.
    pub fn iter<K, V>(&mut self) -> LuaTableIterator<L, K, V> {
        unsafe { ffi::lua_pushnil(self.table.as_mut_lua().0) };

        LuaTableIterator {
            table: self,
            finished: false,
            marker: PhantomData,
        }
    }

    /// Loads a value in the table given its index.
    pub fn get<'a, R, I>(&'a mut self, index: I) -> Option<R>
                         where R: LuaRead<PushGuard<&'a mut LuaTable<L>>>,
                               I: for<'b> Push<&'b mut &'a mut LuaTable<L>>
    {
        let mut me = self;
        index.push_to_lua(&mut me).forget();
        unsafe { ffi::lua_gettable(me.as_mut_lua().0, -2); }
        let guard = PushGuard { lua: me, size: 1 };
        LuaRead::lua_read(guard).ok()
    }

    /// Inserts or modifies an elements of the table.
    pub fn set<'s, I, V>(&'s mut self, index: I, value: V)
                         where I: for<'a> Push<&'a mut &'s mut LuaTable<L>>,
                               V: for<'a> Push<&'a mut &'s mut LuaTable<L>>
    {
        let mut me = self;
        index.push_to_lua(&mut me).forget();
        value.push_to_lua(&mut me).forget();
        unsafe { ffi::lua_settable(me.as_mut_lua().0, -3); }
    }

    /// Inserts an empty array, then loads it.
    pub fn empty_array<'s, I>(&'s mut self, index: I) -> LuaTable<PushGuard<&'s mut LuaTable<L>>>
                              where I: for<'a> Push<&'a mut &'s mut LuaTable<L>> + Clone
    {
        // TODO: cleaner implementation
        let mut me = self;
        index.clone().push_to_lua(&mut me).forget();
        Vec::<u8>::with_capacity(0).push_to_lua(&mut me).forget();
        unsafe { ffi::lua_settable(me.as_mut_lua().0, -3); }

        me.get(index).unwrap()
    }

    /// Obtains or create the metatable of the table.
    pub fn get_or_create_metatable(mut self) -> LuaTable<PushGuard<L>> {
        let result = unsafe { ffi::lua_getmetatable(self.table.as_mut_lua().0, -1) };

        if result == 0 {
            unsafe {
                ffi::lua_newtable(self.table.as_mut_lua().0);
                ffi::lua_setmetatable(self.table.as_mut_lua().0, -2);
                let r = ffi::lua_getmetatable(self.table.as_mut_lua().0, -1);
                assert!(r != 0);
            }
        }

        LuaTable {
            table: PushGuard { lua: self.table, size: 1 }
        }
    }
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
        let key = LuaRead::lua_read_at_position(&mut me, -2).ok();
        let value = LuaRead::lua_read_at_position(&mut me, -1).ok();

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

impl<'t, L, K, V> Drop for LuaTableIterator<'t, L, K, V> where L: AsMutLua + 't {
    fn drop(&mut self) {
        if !self.finished {
            unsafe { ffi::lua_pop(self.table.table.as_mut_lua().0, 1) }
        }
    }
}
