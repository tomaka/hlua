use { CopyReadable, ConsumeReadable, LoadedVariable, Pushable, Index };
use ffi;

#[unstable]
pub struct LuaTable<'var, 'lua> {
    variable: LoadedVariable<'var, 'lua>
}

// while the LuaTableIterator is active, the current key is constantly pushed over the table
#[unstable]
pub struct LuaTableIterator<'var, 'lua, 'table> {
    table: &'table mut LuaTable<'var, 'lua>
}

impl<'var, 'lua> ConsumeReadable<'var, 'lua> for LuaTable<'var, 'lua> {
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

    pub fn get<R: CopyReadable, I: Index<'lua>>(&mut self, index: I) -> Option<R> {
        index.push_to_lua(self.variable.lua);
        unsafe { ffi::lua_gettable(self.variable.lua.lua, -2); }
        let value = CopyReadable::read_from_lua(self.variable.lua, -1);
        unsafe { ffi::lua_pop(self.variable.lua.lua, 1); }
        value
    }

    pub fn set<I: Index<'lua>, V: Pushable<'lua>>(&mut self, index: I, value: V) {
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

impl<'a, 'b, 'c, K: CopyReadable, V: CopyReadable> Iterator<Option<(K,V)>> for LuaTableIterator<'a, 'b, 'c> {
    fn next(&mut self)
        -> Option<Option<(K,V)>>
    {
        // this call pushes the next key and value on the stack
        if unsafe { ffi::lua_next(self.table.variable.lua.lua, -2) } == 0 {
            return None
        }

        let key = CopyReadable::read_from_lua(self.table.variable.lua, -2);
        let value = CopyReadable::read_from_lua(self.table.variable.lua, -1);

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

#[cfg(test)]
mod tests {
    use Lua;
    use lua_tables::LuaTable;

    #[test]
    fn iterable() {
        let mut lua = Lua::new();

        let _:() = lua.execute("a = { 9, 8, 7 }").unwrap();

        let mut table: LuaTable = lua.get("a").unwrap();
        let mut counter = 0u;

        for (key, value) in table.iter().filter_map(|e| e) {
            let _: uint = key;
            let _: uint = value;
            assert_eq!(key + value, 10);
            counter += 1;
        }

        assert_eq!(counter, 3);
    }

    #[test]
    fn iterable_multipletimes() {
        let mut lua = Lua::new();

        let _:() = lua.execute("a = { 9, 8, 7 }").unwrap();

        let mut table: LuaTable = lua.get("a").unwrap();

        for _ in range(0u, 10) {
            let tableContent: Vec<Option<(uint, uint)>> = table.iter().collect();
            assert_eq!(tableContent, vec!( Some((1,9)), Some((2,8)), Some((3,7)) ));
        }
    }

    #[test]
    fn get_set() {
        let mut lua = Lua::new();

        let _:() = lua.execute("a = { 9, 8, 7 }").unwrap();
        let mut table: LuaTable = lua.get("a").unwrap();

        let x: int = table.get(2i).unwrap();
        assert_eq!(x, 8);

        table.set(3i, "hello");
        let y: String = table.get(3i).unwrap();
        assert_eq!(y.as_slice(), "hello");

        let z: int = table.get(1i).unwrap();
        assert_eq!(z, 9);
    }

    #[test]
    fn metatable() {
        let mut lua = Lua::new();

        let _:() = lua.execute("a = { 9, 8, 7 }").unwrap();

        {
            let table: LuaTable = lua.get("a").unwrap();

            let mut metatable = table.get_or_create_metatable();
            fn handler() -> int { 5 };
            metatable.set("__add".to_string(), handler);
        }

        let r: int = lua.execute("return a + a").unwrap();
        assert_eq!(r, 5);
    }
}
