use std::marker::PhantomData;

use ffi;
use LuaContext;

use AsLua;
use AsMutLua;
use Push;
use PushGuard;
use LuaRead;
use Void;

/// Represents a table stored in the Lua context.
///
/// Loading this type mutably borrows the Lua context.
pub struct LuaTable<L> {
    table: L,
    index: i32,
}
impl<L> LuaTable<L> {
    /// Return the index on the stack of this table, assuming -(offset - 1)
    /// items have been pushed to the stack since it was loaded.
    #[inline]
    fn offset(&self, offset: i32) -> i32 {
        if self.index < 0 {
            // If this table was indexed from the top of the stack, its current
            // index will have been pushed down by the newly-pushed items.
            self.index + offset
        } else {
            // If this table was indexed from the bottom of the stack, its
            // current position will be unchanged.
            self.index
        }
    }
}

unsafe impl<'lua, L> AsLua<'lua> for LuaTable<L>
    where L: AsLua<'lua>
{
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.table.as_lua()
    }
}

unsafe impl<'lua, L> AsMutLua<'lua> for LuaTable<L>
    where L: AsMutLua<'lua>
{
    #[inline]
    fn as_mut_lua(&mut self) -> LuaContext {
        self.table.as_mut_lua()
    }
}

impl<'lua, L> LuaRead<L> for LuaTable<L>
    where L: AsMutLua<'lua>
{
    #[inline]
    fn lua_read_at_position(mut lua: L, index: i32) -> Result<LuaTable<L>, L> {
        if unsafe { ffi::lua_istable(lua.as_mut_lua().0, index) } {
            Ok(LuaTable {
                table: lua,
                index: index,
            })
        } else {
            Err(lua)
        }
    }
}

/// Iterator that enumerates the content of a Lua table.
// while the LuaTableIterator is active, the current key is constantly pushed over the table
pub struct LuaTableIterator<'t, L: 't, K, V> {
    table: &'t mut LuaTable<L>,
    finished: bool, // if true, the key is not on the stack anymore
    raw_lua: LuaContext,
    marker: PhantomData<(K, V)>,
}

unsafe impl<'t, 'lua, L, K, V> AsLua<'lua> for LuaTableIterator<'t, L, K, V>
    where L: AsMutLua<'lua>
{
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.table.as_lua()
    }
}

unsafe impl<'t, 'lua, L, K, V> AsMutLua<'lua> for LuaTableIterator<'t, L, K, V>
    where L: AsMutLua<'lua>
{
    #[inline]
    fn as_mut_lua(&mut self) -> LuaContext {
        self.table.as_mut_lua()
    }
}

impl<'lua, L> LuaTable<L>
    where L: AsMutLua<'lua>
{
    /// Destroys the LuaTable and returns its inner Lua context. Useful when it takes Lua by value.
    #[inline]
    pub fn into_inner(self) -> L {
        self.table
    }

    /// Iterates over the elements inside the table.
    #[inline]
    pub fn iter<K, V>(&mut self) -> LuaTableIterator<L, K, V> {
        unsafe { ffi::lua_pushnil(self.table.as_mut_lua().0) };

        let raw_lua = self.table.as_lua();
        LuaTableIterator {
            table: self,
            finished: false,
            raw_lua: raw_lua,
            marker: PhantomData,
        }
    }

    /// Loads a value in the table given its index.
    #[inline]
    pub fn get<'a, R, I>(&'a mut self, index: I) -> Option<R>
        where R: LuaRead<PushGuard<&'a mut LuaTable<L>>>,
              I: for<'b> Push<&'b mut &'a mut LuaTable<L>>
    {
        unsafe {
            let mut me = self;
            match index.push_to_lua(&mut me) {
                Ok(pushed) => pushed.forget(),
                Err(_) => unreachable!()
            };
            ffi::lua_gettable(me.as_mut_lua().0, me.offset(-1));
            if unsafe { ffi::lua_isnil(me.as_lua().0, -1) } {
                let raw_lua = me.as_lua();
                let _guard = PushGuard { lua: me, size: 1, raw_lua: raw_lua };
                return None;
            }
            let raw_lua = me.as_lua();
            let guard = PushGuard { lua: me, size: 1, raw_lua: raw_lua };
            LuaRead::lua_read(guard).ok()
        }
    }

    /// Loads a value in the table, with the result capturing the table by value.
    #[inline]
    pub fn into_get<'a, R, I>(self, index: I) -> Result<R, PushGuard<Self>>
        where R: LuaRead<PushGuard<LuaTable<L>>>,
              I: for<'b> Push<&'b mut LuaTable<L>>
    {
        unsafe {
            let mut me = self;
            match index.push_to_lua(&mut me) {
                Ok(pushed) => pushed.forget(),
                Err(_) => unreachable!()
            };
            ffi::lua_gettable(me.as_mut_lua().0, me.offset(-1));
            let is_nil = ffi::lua_isnil(me.as_mut_lua().0, -1);
            let raw_lua = me.as_lua();
            let guard = PushGuard { lua: me, size: 1, raw_lua: raw_lua };
            if is_nil {
                Err(guard)
            } else {
                LuaRead::lua_read(guard)
            }
        }
    }

    /// Inserts or modifies an elements of the table.
    #[inline]
    pub fn set<'s, I, V>(&'s mut self, index: I, value: V)
        where I: for<'a> Push<&'a mut &'s mut LuaTable<L>, Err = Void>,
              V: for<'a> Push<&'a mut &'s mut LuaTable<L>, Err = Void>
    {
        match self.checked_set(index, value) {
            Ok(()) => (),
            Err(_) => unreachable!()
        }
    }

    /// Inserts or modifies an elements of the table.
    #[inline]
    pub fn checked_set<'s, I, V, E>(&'s mut self, index: I, value: V) -> Result<(), E>
        where I: for<'a> Push<&'a mut &'s mut LuaTable<L>, Err = E>,        // TODO: different err type
              V: for<'a> Push<&'a mut &'s mut LuaTable<L>, Err = E>
    {
        unsafe {
            let mut me = self;

            match index.push_to_lua(&mut me) {
                Ok(pushed) => pushed.forget(),
                Err((err, _)) => return Err(err),       // FIXME: panic safety
            };

            match value.push_to_lua(&mut me) {
                Ok(pushed) => pushed.forget(),
                Err((err, _)) => return Err(err),       // FIXME: panic safety
            };

            ffi::lua_settable(me.as_mut_lua().0, me.offset(-2));
            Ok(())
        }
    }

    /// Inserts an empty array, then loads it.
    #[inline]
    pub fn empty_array<'s, I>(&'s mut self, index: I) -> LuaTable<PushGuard<&'s mut LuaTable<L>>>
        where I: for<'a> Push<&'a mut &'s mut LuaTable<L>> + Clone
    {
        // TODO: cleaner implementation
        unsafe {
            let mut me = self;
            match index.clone().push_to_lua(&mut me) {
                Ok(pushed) => pushed.forget(),
                Err(_) => panic!()      // TODO:
            };

            match Vec::<u8>::with_capacity(0).push_to_lua(&mut me) {
                Ok(pushed) => pushed.forget(),
                Err(_) => panic!()      // TODO:
            };

            ffi::lua_settable(me.as_mut_lua().0, me.offset(-2));

            me.get(index).unwrap()
        }
    }

    /// Obtains or create the metatable of the table.
    #[inline]
    pub fn get_or_create_metatable(mut self) -> LuaTable<PushGuard<L>> {
        let result = unsafe { ffi::lua_getmetatable(self.table.as_mut_lua().0, self.index) };

        if result == 0 {
            unsafe {
                ffi::lua_newtable(self.table.as_mut_lua().0);
                ffi::lua_setmetatable(self.table.as_mut_lua().0, self.offset(-1));
                let r = ffi::lua_getmetatable(self.table.as_mut_lua().0, self.index);
                assert!(r != 0);
            }
        }

        let raw_lua = self.as_lua();
        LuaTable {
            table: PushGuard {
                lua: self.table,
                size: 1,
                raw_lua: raw_lua,
            },
            index: -1, // After creating the metatable, it will be on top of the stack.
        }
    }
}

impl<'t, 'lua, L, K, V> Iterator for LuaTableIterator<'t, L, K, V>
    where L: AsMutLua<'lua> + 't,
          K: for<'i, 'j> LuaRead<&'i mut &'j mut LuaTableIterator<'t, L, K, V>> + 'static,
          V: for<'i, 'j> LuaRead<&'i mut &'j mut LuaTableIterator<'t, L, K, V>> + 'static
{
    type Item = Option<(K, V)>;

    #[inline]
    fn next(&mut self) -> Option<Option<(K, V)>> {
        if self.finished {
            return None;
        }

        // this call pushes the next key and value on the stack
        if unsafe { ffi::lua_next(self.table.as_mut_lua().0, self.table.offset(-1)) } == 0 {
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

impl<'t, L, K, V> Drop for LuaTableIterator<'t, L, K, V> {
    #[inline]
    fn drop(&mut self) {
        if !self.finished {
            unsafe { ffi::lua_pop(self.raw_lua.0, 1) }
        }
    }
}
