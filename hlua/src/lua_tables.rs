use std::marker::PhantomData;

use ffi;
use LuaContext;

use AsLua;
use AsMutLua;
use LuaRead;
use Push;
use PushGuard;
use PushOne;
use Void;

/// Represents a table stored in the Lua context.
///
/// Just like you can read variables as integers and strings, you can also read Lua table by
/// requesting a `LuaTable` object. Doing so will mutably borrow the object which you got the table
/// from.
///
/// # Example: reading a global variable
///
/// ```
/// let mut lua = hlua::Lua::new();
/// lua.execute::<()>("a = {28, 92, 17};").unwrap();
///
/// let mut table: hlua::LuaTable<_> = lua.get("a").unwrap();
/// for (k, v) in table.iter::<i32, i32>().filter_map(|e| e) {
///     println!("{} => {}", k, v);
/// }
/// ```
///
#[derive(Debug)]
pub struct LuaTable<L> {
    table: L,
    index: i32,
}

impl<L> LuaTable<L> {
    // Return the index on the stack of this table, assuming -(offset - 1)
    // items have been pushed to the stack since it was loaded.
    // For example if you push one element over the table, call `offset(-1)` to know where the
    // table is.
    #[inline]
    fn offset(&self, offset: i32) -> i32 {
        if self.index >= 0 || self.index == ffi::LUA_REGISTRYINDEX {
            // If this table is the registry or was indexed from the bottom of the stack, its
            // current position will be unchanged.
            self.index
        } else {
            // If this table was indexed from the top of the stack, its current
            // index will have been pushed down by the newly-pushed items.
            self.index + offset
        }
    }
}

unsafe impl<'lua, L> AsLua<'lua> for LuaTable<L>
where
    L: AsLua<'lua>,
{
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.table.as_lua()
    }
}

unsafe impl<'lua, L> AsMutLua<'lua> for LuaTable<L>
where
    L: AsMutLua<'lua>,
{
    #[inline]
    fn as_mut_lua(&mut self) -> LuaContext {
        self.table.as_mut_lua()
    }
}

impl<'lua, L> LuaRead<L> for LuaTable<L>
where
    L: AsMutLua<'lua>,
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

impl<'lua, L> LuaTable<L>
where
    L: AsMutLua<'lua>,
{
    /// Destroys the `LuaTable` and returns its inner Lua context. Useful when it takes Lua by
    /// value.
    // TODO: find an example where it is useful
    #[inline]
    pub fn into_inner(self) -> L {
        self.table
    }

    /// Iterates over the elements inside the table.
    // TODO: doc
    #[inline]
    pub fn iter<K, V>(&mut self) -> LuaTableIterator<L, K, V> {
        unsafe {
            ffi::lua_pushnil(self.table.as_mut_lua().0);

            let raw_lua = self.table.as_lua();
            LuaTableIterator {
                table: self,
                finished: false,
                raw_lua: raw_lua,
                marker: PhantomData,
            }
        }
    }

    /// Loads a value in the table given its index.
    ///
    /// The index must implement the `PushOne` trait and the return type must implement the
    /// `LuaRead` trait. See
    /// [the documentation at the crate root](index.html#pushing-and-loading-values) for more
    /// information.
    ///
    /// # Example: reading a table inside of a table.
    ///
    /// ```
    /// let mut lua = hlua::Lua::new();
    /// lua.execute::<()>("a = { 9, { 8, 7 }, 6 }").unwrap();
    ///
    /// let mut table = lua.get::<hlua::LuaTable<_>, _>("a").unwrap();
    ///
    /// assert_eq!(table.get::<i32, _, _>(1).unwrap(), 9);
    /// assert_eq!(table.get::<i32, _, _>(3).unwrap(), 6);
    ///
    /// {
    ///     let mut subtable: hlua::LuaTable<_> = table.get(2).unwrap();
    ///     assert_eq!(subtable.get::<i32, _, _>(1).unwrap(), 8);
    ///     assert_eq!(subtable.get::<i32, _, _>(2).unwrap(), 7);
    /// }
    /// ```
    ///
    #[inline]
    pub fn get<'a, R, I, E>(&'a mut self, index: I) -> Option<R>
    where
        R: LuaRead<PushGuard<&'a mut LuaTable<L>>>,
        I: for<'b> PushOne<&'b mut &'a mut LuaTable<L>, Err = E>,
        E: Into<Void>,
    {
        unsafe {
            // Because of a weird borrow error, we need to push the index by borrowing `&mut &mut L`
            // instead of `&mut L`. `self` matches `&mut L`, so in theory we could do `&mut self`.
            // But in practice `self` isn't mutable, so we need to move it into `me` first.
            // TODO: remove this by simplifying the PushOne requirement ; however this is complex
            //       because of the empty_array method
            let mut me = self;

            index.push_no_err(&mut me).assert_one_and_forget();
            ffi::lua_gettable(me.as_mut_lua().0, me.offset(-1));

            let raw_lua = me.as_lua();
            let guard = PushGuard {
                lua: me,
                size: 1,
                raw_lua: raw_lua,
            };

            if ffi::lua_isnil(raw_lua.0, -1) {
                None
            } else {
                LuaRead::lua_read(guard).ok()
            }
        }
    }

    /// Loads a value in the table, with the result capturing the table by value.
    // TODO: doc
    #[inline]
    pub fn into_get<R, I, E>(mut self, index: I) -> Result<R, PushGuard<Self>>
    where
        R: LuaRead<PushGuard<LuaTable<L>>>,
        I: for<'b> PushOne<&'b mut LuaTable<L>, Err = E>,
        E: Into<Void>,
    {
        unsafe {
            index.push_no_err(&mut self).assert_one_and_forget();

            ffi::lua_gettable(self.as_mut_lua().0, self.offset(-1));

            let raw_lua = self.as_lua();
            let guard = PushGuard {
                lua: self,
                size: 1,
                raw_lua: raw_lua,
            };

            if ffi::lua_isnil(raw_lua.0, -1) {
                Err(guard)
            } else {
                LuaRead::lua_read(guard)
            }
        }
    }

    /// Inserts or modifies an elements of the table.
    ///
    /// Contrary to `checked_set`, can only be called when writing the key and value cannot fail
    /// (which is the case for most types).
    ///
    /// The index and the value must both implement the `PushOne` trait. See
    /// [the documentation at the crate root](index.html#pushing-and-loading-values) for more
    /// information.
    // TODO: doc
    #[inline]
    pub fn set<I, V, Ei, Ev>(&mut self, index: I, value: V)
    where
        I: for<'r> PushOne<&'r mut LuaTable<L>, Err = Ei>,
        V: for<'r, 's> PushOne<&'r mut PushGuard<&'s mut LuaTable<L>>, Err = Ev>,
        Ei: Into<Void>,
        Ev: Into<Void>,
    {
        match self.checked_set(index, value) {
            Ok(()) => (),
            Err(_) => unreachable!(),
        }
    }

    /// Inserts or modifies an elements of the table.
    ///
    /// Returns an error if we failed to write the key and the value. This can only happen for a
    /// limited set of types. You are encouraged to use the `set` method if writing cannot fail.
    // TODO: doc
    #[inline]
    pub fn checked_set<I, V, Ke, Ve>(
        &mut self,
        index: I,
        value: V,
    ) -> Result<(), CheckedSetError<Ke, Ve>>
    where
        I: for<'r> PushOne<&'r mut LuaTable<L>, Err = Ke>,
        V: for<'r, 's> PushOne<&'r mut PushGuard<&'s mut LuaTable<L>>, Err = Ve>,
    {
        unsafe {
            let raw_lua = self.as_mut_lua().0;
            let my_offset = self.offset(-2);

            let mut guard = match index.push_to_lua(self) {
                Ok(guard) => {
                    assert_eq!(guard.size, 1);
                    guard
                }
                Err((err, _)) => {
                    return Err(CheckedSetError::KeyPushError(err));
                }
            };

            match value.push_to_lua(&mut guard) {
                Ok(pushed) => {
                    assert_eq!(pushed.size, 1);
                    pushed.forget()
                }
                Err((err, _)) => {
                    return Err(CheckedSetError::ValuePushError(err));
                }
            };

            guard.forget();
            ffi::lua_settable(raw_lua, my_offset);
            Ok(())
        }
    }

    /// Inserts an empty array, then loads it.
    #[inline]
    pub fn empty_array<'s, I, E>(&'s mut self, index: I) -> LuaTable<PushGuard<&'s mut LuaTable<L>>>
    where
        I: for<'a> PushOne<&'a mut &'s mut LuaTable<L>, Err = E> + Clone,
        E: Into<Void>,
    {
        // TODO: cleaner implementation
        unsafe {
            let mut me = self;
            match index.clone().push_to_lua(&mut me) {
                Ok(pushed) => {
                    assert_eq!(pushed.size, 1);
                    pushed.forget()
                }
                Err(_) => panic!(), // TODO:
            };

            match Vec::<u8>::with_capacity(0).push_to_lua(&mut me) {
                Ok(pushed) => pushed.forget(),
                Err(_) => panic!(), // TODO:
            };

            ffi::lua_settable(me.as_mut_lua().0, me.offset(-2));

            me.get(index).unwrap()
        }
    }

    /// Obtains or creates the metatable of the table.
    ///
    /// A metatable is an additional table that can be attached to a table or a userdata. It can
    /// contain anything, but its most interesting usage are the following special methods:
    ///
    /// - If non-nil, the `__index` entry of the metatable is used as a function whenever the user
    ///   tries to read a non-existing entry in the table or userdata. Its signature is
    ///   `(object, index) -> value`.
    /// - If non-nil, the `__newindex` entry of the metatable is used as a function whenever the
    ///   user tries to write a non-existing entry in the table or userdata. Its signature is
    ///   `(object, index, value)`.
    /// - If non-nil, the `__lt`, `__le` and `__eq` entries correspond respectively to operators
    ///    `<`, `<=` and `==`. Their signature is `(a, b) -> bool`. Other operators are
    ///   automatically derived from these three functions.
    /// - If non-nil, the `__add`, `__mul`, `__sub`, `__div`, `__unm`, `__pow` and `__concat`
    ///   entries correspond to operators `+`, `*`, `-`, `/`, `-` (unary), `^` and `..`. Their
    ///   signature is `(a, b) -> result`.
    /// - If non-nil, the `__gc` entry is called whenever the garbage collector is about to drop
    ///   the object. Its signature is simply `(obj)`. Remember that usercode is able to modify
    ///   the metatable as well, so there is no strong guarantee that this is actually going to be
    ///   called.
    ///
    /// Interestingly enough, a metatable can also have a metatable. For example if you try to
    /// access a non-existing field in a table, Lua will look for the `__index` function in its
    /// metatable. If that function doesn't exist, it will try to use the `__index` function of the
    /// metatable's metatable in order to get the `__index` function of the metatable. This can
    /// go on infinitely.
    ///
    /// # Example
    ///
    /// ```
    /// use hlua::Lua;
    /// use hlua::LuaTable;
    /// use hlua::AnyLuaValue;
    ///
    /// let mut lua = Lua::new();
    /// lua.execute::<()>("a = {}").unwrap();
    ///
    /// {
    ///     let mut table: LuaTable<_> = lua.get("a").unwrap();
    ///     let mut metatable = table.get_or_create_metatable();
    ///     metatable.set("__index", hlua::function2(|_: AnyLuaValue, var: String| -> AnyLuaValue {
    ///         println!("The user tried to access non-existing index {:?}", var);
    ///         AnyLuaValue::LuaNil
    ///     }));
    /// }
    /// ```
    #[inline]
    pub fn get_or_create_metatable(mut self) -> LuaTable<PushGuard<L>> {
        unsafe {
            // We put the metatable at the top of the stack.
            if ffi::lua_getmetatable(self.table.as_mut_lua().0, self.index) == 0 {
                // No existing metatable ; create one then set it and reload it.
                ffi::lua_newtable(self.table.as_mut_lua().0);
                ffi::lua_setmetatable(self.table.as_mut_lua().0, self.offset(-1));
                let r = ffi::lua_getmetatable(self.table.as_mut_lua().0, self.index);
                debug_assert!(r != 0);
            }

            let raw_lua = self.as_lua();
            LuaTable {
                table: PushGuard {
                    lua: self.table,
                    size: 1,
                    raw_lua: raw_lua,
                },
                index: -1,
            }
        }
    }

    /// Builds the `LuaTable` that yields access to the registry.
    ///
    /// The registry is a special table available from anywhere and that is not directly
    /// accessible from Lua code. It can be used to store whatever you want to keep in memory.
    ///
    /// # Example
    ///
    /// ```
    /// use hlua::Lua;
    /// use hlua::LuaTable;
    ///
    /// let mut lua = Lua::new();
    ///
    /// let mut table = LuaTable::registry(&mut lua);
    /// table.set(3, "hello");
    /// ```
    #[inline]
    pub fn registry(lua: L) -> LuaTable<L> {
        LuaTable {
            table: lua,
            index: ffi::LUA_REGISTRYINDEX,
        }
    }
}

/// Error returned by the `checked_set` function.
// TODO: implement `Error` on this type
#[derive(Debug, Copy, Clone)]
pub enum CheckedSetError<K, V> {
    /// Error while pushing the key.
    KeyPushError(K),
    /// Error while pushing the value.
    ValuePushError(V),
}

/// Iterator that enumerates the content of a Lua table.
///
/// See `LuaTable::iter` for more info.
// Implementation note: While the LuaTableIterator is active, the current key is constantly
// pushed over the table. The destructor takes care of removing it.
#[derive(Debug)]
pub struct LuaTableIterator<'t, L: 't, K, V> {
    table: &'t mut LuaTable<L>,
    finished: bool, // if true, the key is not on the stack anymore
    raw_lua: LuaContext,
    marker: PhantomData<(K, V)>,
}

unsafe impl<'t, 'lua, L, K, V> AsLua<'lua> for LuaTableIterator<'t, L, K, V>
where
    L: AsMutLua<'lua>,
{
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.table.as_lua()
    }
}

unsafe impl<'t, 'lua, L, K, V> AsMutLua<'lua> for LuaTableIterator<'t, L, K, V>
where
    L: AsMutLua<'lua>,
{
    #[inline]
    fn as_mut_lua(&mut self) -> LuaContext {
        self.table.as_mut_lua()
    }
}

impl<'t, 'lua, L, K, V> Iterator for LuaTableIterator<'t, L, K, V>
where
    L: AsMutLua<'lua> + 't,
    K: for<'i, 'j> LuaRead<&'i mut &'j mut LuaTableIterator<'t, L, K, V>> + 'static,
    V: for<'i, 'j> LuaRead<&'i mut &'j mut LuaTableIterator<'t, L, K, V>> + 'static,
{
    type Item = Option<(K, V)>;

    #[inline]
    fn next(&mut self) -> Option<Option<(K, V)>> {
        unsafe {
            if self.finished {
                return None;
            }

            // As a reminder, the key is always at the top of the stack unless `finished` is true.

            // This call pops the current key and pushes the next key and value at the top.
            if ffi::lua_next(self.table.as_mut_lua().0, self.table.offset(-1)) == 0 {
                self.finished = true;
                return None;
            }

            // Reading the key and value.
            let mut me = self;
            let key = LuaRead::lua_read_at_position(&mut me, -2).ok();
            let value = LuaRead::lua_read_at_position(&mut me, -1).ok();

            // Removing the value, leaving only the key on the top of the stack.
            ffi::lua_pop(me.table.as_mut_lua().0, 1);

            if key.is_none() || value.is_none() {
                Some(None)
            } else {
                Some(Some((key.unwrap(), value.unwrap())))
            }
        }
    }
}

impl<'t, L, K, V> Drop for LuaTableIterator<'t, L, K, V> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            if !self.finished {
                ffi::lua_pop(self.raw_lua.0, 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use function0;
    use Lua;
    use LuaTable;
    use PushGuard;

    #[test]
    fn iterable() {
        let mut lua = Lua::new();

        let _: () = lua.execute("a = { 9, 8, 7 }").unwrap();

        let mut table = lua.get::<LuaTable<_>, _>("a").unwrap();
        let mut counter = 0;

        for (key, value) in table.iter().filter_map(|e| e) {
            let _: u32 = key;
            let _: u32 = value;
            assert_eq!(key + value, 10);
            counter += 1;
        }

        assert_eq!(counter, 3);
    }

    #[test]
    fn iterable_multipletimes() {
        let mut lua = Lua::new();

        let _: () = lua.execute("a = { 9, 8, 7 }").unwrap();

        let mut table = lua.get::<LuaTable<_>, _>("a").unwrap();

        for _ in 0..10 {
            let table_content: Vec<Option<(u32, u32)>> = table.iter().collect();
            assert_eq!(
                table_content,
                vec![Some((1, 9)), Some((2, 8)), Some((3, 7))]
            );
        }
    }

    #[test]
    fn get_set() {
        let mut lua = Lua::new();

        let _: () = lua.execute("a = { 9, 8, 7 }").unwrap();
        let mut table = lua.get::<LuaTable<_>, _>("a").unwrap();

        let x: i32 = table.get(2).unwrap();
        assert_eq!(x, 8);

        table.set(3, "hello");
        let y: String = table.get(3).unwrap();
        assert_eq!(y, "hello");

        let z: i32 = table.get(1).unwrap();
        assert_eq!(z, 9);
    }

    #[test]
    fn table_over_table() {
        let mut lua = Lua::new();

        lua.execute::<()>("a = { 9, { 8, 7 }, 6 }").unwrap();
        let mut table = lua.get::<LuaTable<_>, _>("a").unwrap();

        let x: i32 = table.get(1).unwrap();
        assert_eq!(x, 9);

        {
            let mut subtable = table.get::<LuaTable<_>, _, _>(2).unwrap();

            let y: i32 = subtable.get(1).unwrap();
            assert_eq!(y, 8);

            let z: i32 = subtable.get(2).unwrap();
            assert_eq!(z, 7);
        }

        let w: i32 = table.get(3).unwrap();
        assert_eq!(w, 6);
    }

    #[test]
    fn metatable() {
        let mut lua = Lua::new();

        let _: () = lua.execute("a = { 9, 8, 7 }").unwrap();

        {
            let table = lua.get::<LuaTable<_>, _>("a").unwrap();

            let mut metatable = table.get_or_create_metatable();
            fn handler() -> i32 {
                5
            };
            metatable.set("__add".to_string(), function0(handler));
        }

        let r: i32 = lua.execute("return a + a").unwrap();
        assert_eq!(r, 5);
    }

    #[test]
    fn empty_array() {
        let mut lua = Lua::new();

        {
            let mut array = lua.empty_array("a");
            array.set("b", 3)
        }

        let mut table: LuaTable<_> = lua.get("a").unwrap();
        assert!(3 == table.get("b").unwrap());
    }

    #[test]
    fn by_value() {
        let mut lua = Lua::new();

        {
            let mut array = lua.empty_array("a");
            {
                let mut array2 = array.empty_array("b");
                array2.set("c", 3);
            }
        }

        let table: LuaTable<PushGuard<Lua>> = lua.into_get("a").ok().unwrap();
        let mut table2: LuaTable<PushGuard<LuaTable<PushGuard<Lua>>>> =
            table.into_get("b").ok().unwrap();
        assert!(3 == table2.get("c").unwrap());
        let table: LuaTable<PushGuard<Lua>> = table2.into_inner().into_inner();
        // do it again to make sure the stack is still sane
        let mut table2: LuaTable<PushGuard<LuaTable<PushGuard<Lua>>>> =
            table.into_get("b").ok().unwrap();
        assert!(3 == table2.get("c").unwrap());
        let table: LuaTable<PushGuard<Lua>> = table2.into_inner().into_inner();
        let _lua: Lua = table.into_inner().into_inner();
    }

    #[test]
    fn registry() {
        let mut lua = Lua::new();

        let mut table = LuaTable::registry(&mut lua);
        table.set(3, "hello");
        let y: String = table.get(3).unwrap();
        assert_eq!(y, "hello");
    }

    #[test]
    fn registry_metatable() {
        let mut lua = Lua::new();

        let registry = LuaTable::registry(&mut lua);
        let mut metatable = registry.get_or_create_metatable();
        metatable.set(3, "hello");
    }
}
