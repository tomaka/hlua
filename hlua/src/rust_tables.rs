use ffi;
use any::{AnyLuaValue, AnyHashableLuaValue};

use Push;
use PushGuard;
use PushOne;
use AsMutLua;
use TuplePushError;
use LuaRead;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::Hash;
use std::iter;

#[inline]
fn push_iter<'lua, L, V, I, E>(mut lua: L, iterator: I) -> Result<PushGuard<L>, (E, L)>
    where L: AsMutLua<'lua>,
          V: for<'b> Push<&'b mut L, Err = E>,
          I: Iterator<Item = V>
{
    // creating empty table
    unsafe { ffi::lua_newtable(lua.as_mut_lua().0) };

    for (elem, index) in iterator.zip(1..) {
        let size = match elem.push_to_lua(&mut lua) {
            Ok(pushed) => pushed.forget_internal(),
            Err((_err, _lua)) => panic!(),     // TODO: wrong   return Err((err, lua)),      // FIXME: destroy the temporary table
        };

        match size {
            0 => continue,
            1 => {
                let index = index as u32;
                match index.push_to_lua(&mut lua) {
                    Ok(pushed) => pushed.forget_internal(),
                    Err(_) => unreachable!(),
                };
                unsafe { ffi::lua_insert(lua.as_mut_lua().0, -2) }
                unsafe { ffi::lua_settable(lua.as_mut_lua().0, -3) }
            }
            2 => unsafe { ffi::lua_settable(lua.as_mut_lua().0, -3) },
            _ => unreachable!(),
        }
    }

    let raw_lua = lua.as_lua();
    Ok(PushGuard {
        lua: lua,
        size: 1,
        raw_lua: raw_lua,
    })
}

#[inline]
fn push_rec_iter<'lua, L, V, I, E>(mut lua: L, iterator: I) -> Result<PushGuard<L>, (E, L)>
    where L: AsMutLua<'lua>,
          V: for<'a> Push<&'a mut L, Err = E>,
          I: Iterator<Item = V>
{
    let (nrec, _) = iterator.size_hint();

    // creating empty table with pre-allocated non-array elements
    unsafe { ffi::lua_createtable(lua.as_mut_lua().0, 0, nrec as i32) };

    for elem in iterator {
        let size = match elem.push_to_lua(&mut lua) {
            Ok(pushed) => pushed.forget_internal(),
            Err((_err, _lua)) => panic!(),     // TODO: wrong   return Err((err, lua)),      // FIXME: destroy the temporary table
        };

        match size {
            0 => continue,
            2 => unsafe { ffi::lua_settable(lua.as_mut_lua().0, -3) },
            _ => unreachable!(),
        }
    }

    let raw_lua = lua.as_lua();
    Ok(PushGuard {
        lua: lua,
        size: 1,
        raw_lua: raw_lua,
    })
}

impl<'lua, L, T, E> Push<L> for Vec<T>
    where L: AsMutLua<'lua>,
          T: for<'a> Push<&'a mut L, Err = E>
{
    type Err = E;

    #[inline]
    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (E, L)> {
        push_iter(lua, self.into_iter())
    }
}

impl<'lua, L, T, E> PushOne<L> for Vec<T>
    where L: AsMutLua<'lua>,
          T: for<'a> Push<&'a mut L, Err = E>
{
}

impl<'lua, L> LuaRead<L> for Vec<AnyLuaValue>
    where L: AsMutLua<'lua>
{
    fn lua_read_at_position(lua: L, index: i32) -> Result<Self, L> {
        // We need this as iteration order isn't guaranteed to match order of
        // keys, even if they're numeric
        // https://www.lua.org/manual/5.2/manual.html#pdf-next
        let mut dict: BTreeMap<i32, AnyLuaValue> = BTreeMap::new();

        let mut me = lua;
        unsafe { ffi::lua_pushnil(me.as_mut_lua().0) };
        let index = index - 1;

        loop {
            if unsafe { ffi::lua_next(me.as_mut_lua().0, index) } == 0 {
                break;
            }

            let key = {
                let maybe_key: Option<i32> =
                    LuaRead::lua_read_at_position(&mut me, -2).ok();
                match maybe_key {
                    None => {
                        // Cleaning up after ourselves
                        unsafe { ffi::lua_pop(me.as_mut_lua().0, 2) };
                        return Err(me)
                    }
                    Some(k) => k,
                }
            };

            let value: AnyLuaValue =
                LuaRead::lua_read_at_position(&mut me, -1).ok().unwrap();

            unsafe { ffi::lua_pop(me.as_mut_lua().0, 1) };

            dict.insert(key, value);
        }

        let (maximum_key, minimum_key) =
            (*dict.keys().max().unwrap_or(&1), *dict.keys().min().unwrap_or(&1));

        if minimum_key != 1 {
            // Rust doesn't support sparse arrays or arrays with negative
            // indices
            return Err(me);
        }

        let mut result =
            Vec::with_capacity(maximum_key as usize);

        // We expect to start with first element of table and have this
        // be smaller that first key by one
        let mut previous_key = 0;

        // By this point, we actually iterate the map to move values to Vec
        // and check that table represented non-sparse 1-indexed array
        for (k, v) in dict {
            if previous_key + 1 != k {
                return Err(me)
            } else {
                // We just push, thus converting Lua 1-based indexing
                // to Rust 0-based indexing
                result.push(v);
                previous_key = k;
            }
        }

        Ok(result)
    }
}

impl<'a, 'lua, L, T, E> Push<L> for &'a [T]
    where L: AsMutLua<'lua>,
          T: Clone + for<'b> Push<&'b mut L, Err = E>
{
    type Err = E;

    #[inline]
    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (E, L)> {
        push_iter(lua, self.iter().map(|e| e.clone()))
    }
}

impl<'a, 'lua, L, T, E> PushOne<L> for &'a [T]
    where L: AsMutLua<'lua>,
          T: Clone + for<'b> Push<&'b mut L, Err = E>
{
}

impl<'lua, L> LuaRead<L> for HashMap<AnyHashableLuaValue, AnyLuaValue>
    where L: AsMutLua<'lua>
{
    // TODO: this should be implemented using the LuaTable API instead of raw Lua calls.
    fn lua_read_at_position(lua: L, index: i32) -> Result<Self, L> {
        let mut me = lua;
        unsafe { ffi::lua_pushnil(me.as_mut_lua().0) };
        let index = index - 1;
        let mut result = HashMap::new();

        loop {
            if unsafe { ffi::lua_next(me.as_mut_lua().0, index) } == 0 {
                break;
            }

            let key = {
                let maybe_key: Option<AnyHashableLuaValue> =
                    LuaRead::lua_read_at_position(&mut me, -2).ok();
                match maybe_key {
                    None => {
                        // Cleaning up after ourselves
                        unsafe { ffi::lua_pop(me.as_mut_lua().0, 2) };
                        return Err(me)
                    }
                    Some(k) => k,
                }
            };

            let value: AnyLuaValue =
                LuaRead::lua_read_at_position(&mut me, -1).ok().unwrap();

            unsafe { ffi::lua_pop(me.as_mut_lua().0, 1) };

            result.insert(key, value);
        }

        Ok(result)
    }
}

// TODO: use an enum for the error to allow different error types for K and V
impl<'lua, L, K, V, E> Push<L> for HashMap<K, V>
    where L: AsMutLua<'lua>,
          K: for<'a, 'b> PushOne<&'a mut &'b mut L, Err = E> + Eq + Hash,
          V: for<'a, 'b> PushOne<&'a mut &'b mut L, Err = E>
{
    type Err = E;

    #[inline]
    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (E, L)> {
        match push_rec_iter(lua, self.into_iter()) {
            Ok(g) => Ok(g),
            Err((TuplePushError::First(err), lua)) => Err((err, lua)),
            Err((TuplePushError::Other(err), lua)) => Err((err, lua)),
        }
    }
}

impl<'lua, L, K, V, E> PushOne<L> for HashMap<K, V>
    where L: AsMutLua<'lua>,
          K: for<'a, 'b> PushOne<&'a mut &'b mut L, Err = E> + Eq + Hash,
          V: for<'a, 'b> PushOne<&'a mut &'b mut L, Err = E>
{
}

impl<'lua, L, K, E> Push<L> for HashSet<K>
    where L: AsMutLua<'lua>,
          K: for<'a, 'b> PushOne<&'a mut &'b mut L, Err = E> + Eq + Hash
{
    type Err = E;

    #[inline]
    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (E, L)> {
        match push_rec_iter(lua, self.into_iter().zip(iter::repeat(true))) {
            Ok(g) => Ok(g),
            Err((TuplePushError::First(err), lua)) => Err((err, lua)),
            Err((TuplePushError::Other(_), _)) => unreachable!(),
        }
    }
}

impl<'lua, L, K, E> PushOne<L> for HashSet<K>
    where L: AsMutLua<'lua>,
          K: for<'a, 'b> PushOne<&'a mut &'b mut L, Err = E> + Eq + Hash
{
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet, BTreeMap};
    use Lua;
    use LuaTable;
    use AnyLuaValue;
    use AnyHashableLuaValue;

    #[test]
    fn write() {
        let mut lua = Lua::new();

        lua.set("a", vec![9, 8, 7]);

        let mut table: LuaTable<_> = lua.get("a").unwrap();

        let values: Vec<(i32, i32)> = table.iter().filter_map(|e| e).collect();
        assert_eq!(values, vec![(1, 9), (2, 8), (3, 7)]);
    }

    #[test]
    fn write_map() {
        let mut lua = Lua::new();

        let mut map = HashMap::new();
        map.insert(5, 8);
        map.insert(13, 21);
        map.insert(34, 55);

        lua.set("a", map.clone());

        let mut table: LuaTable<_> = lua.get("a").unwrap();

        let values: HashMap<i32, i32> = table.iter().filter_map(|e| e).collect();
        assert_eq!(values, map);
    }

    #[test]
    fn write_set() {
        let mut lua = Lua::new();

        let mut set = HashSet::new();
        set.insert(5);
        set.insert(8);
        set.insert(13);
        set.insert(21);
        set.insert(34);
        set.insert(55);

        lua.set("a", set.clone());

        let mut table: LuaTable<_> = lua.get("a").unwrap();

        let values: HashSet<i32> = table.iter()
            .filter_map(|e| e)
            .map(|(elem, set): (i32, bool)| {
                assert!(set);
                elem
            })
            .collect();

        assert_eq!(values, set);
    }

    #[test]
    fn globals_table() {
        let mut lua = Lua::new();

        lua.globals_table().set("a", 12);

        let val: i32 = lua.get("a").unwrap();
        assert_eq!(val, 12);
    }

    #[test]
    fn reading_vec_works() {
        let mut lua = Lua::new();

        let orig = [1., 2., 3.];

        lua.set("v", &orig[..]);

        let read: Vec<_> = lua.get("v").unwrap();
        for (o, r) in orig.iter().zip(read.iter()) {
            if let AnyLuaValue::LuaNumber(ref n) = *r {
                assert_eq!(o, n);
            } else {
                panic!("Unexpected variant");
            }
        }
    }

    #[test]
    fn reading_vec_from_sparse_table_doesnt_work() {
        let mut lua = Lua::new();

        lua.execute::<()>(r#"v = { [-1] = -1, [2] = 2, [42] = 42 }"#).unwrap();

        let read: Option<Vec<_>> = lua.get("v");
        if read.is_some() {
            panic!("Unexpected success");
        }
    }

    #[test]
    fn reading_vec_with_empty_table_works() {
        let mut lua = Lua::new();

        lua.execute::<()>(r#"v = { }"#).unwrap();

        let read: Vec<_> = lua.get("v").unwrap();
        assert_eq!(read.len(), 0);
    }

    #[test]
    fn reading_vec_with_complex_indexes_doesnt_work() {
        let mut lua = Lua::new();

        lua.execute::<()>(r#"v = { [-1] = -1, ["foo"] = 2, [{}] = 42 }"#).unwrap();

        let read: Option<Vec<_>> = lua.get("v");
        if read.is_some() {
            panic!("Unexpected success");
        }
    }

    #[test]
    fn reading_heterogenous_vec_works() {
        let mut lua = Lua::new();

        let orig = [
            AnyLuaValue::LuaNumber(1.),
            AnyLuaValue::LuaBoolean(false),
            AnyLuaValue::LuaNumber(3.),
            // Pushing String to and reading it from makes it a number
            //AnyLuaValue::LuaString(String::from("3"))
        ];

        lua.set("v", &orig[..]);

        let read: Vec<_> = lua.get("v").unwrap();
        assert_eq!(read, orig);
    }

    #[test]
    fn reading_vec_set_from_lua_works() {
        let mut lua = Lua::new();

        lua.execute::<()>(r#"v = { 1, 2, 3 }"#).unwrap();

        let read: Vec<_> = lua.get("v").unwrap();
        assert_eq!(
            read,
            [1., 2., 3.].iter()
                .map(|x| AnyLuaValue::LuaNumber(*x)).collect::<Vec<_>>());
    }

    #[test]
    fn reading_hashmap_works() {
        let mut lua = Lua::new();

        let orig: HashMap<i32, f64> = [1., 2., 3.].iter().enumerate().map(|(k, v)| (k as i32, *v as f64)).collect();
        let orig_copy = orig.clone();
        // Collect to BTreeMap so that iterator yields values in order
        let orig_btree: BTreeMap<_, _> = orig_copy.into_iter().collect();

        lua.set("v", orig);

        let read: HashMap<AnyHashableLuaValue, AnyLuaValue> = lua.get("v").unwrap();
        // Same as above
        let read_btree: BTreeMap<_, _> = read.into_iter().collect();
        for (o, r) in orig_btree.iter().zip(read_btree.iter()) {
            if let (&AnyHashableLuaValue::LuaNumber(i), &AnyLuaValue::LuaNumber(n)) = r {
                let (&o_i, &o_n) = o;
                assert_eq!(o_i, i);
                assert_eq!(o_n, n);
            } else {
                panic!("Unexpected variant");
            }
        }
    }

    #[test]
    fn reading_hashmap_from_sparse_table_works() {
        let mut lua = Lua::new();

        lua.execute::<()>(r#"v = { [-1] = -1, [2] = 2, [42] = 42 }"#).unwrap();

        let read: HashMap<_, _> = lua.get("v").unwrap();
        assert_eq!(read[&AnyHashableLuaValue::LuaNumber(-1)], AnyLuaValue::LuaNumber(-1.));
        assert_eq!(read[&AnyHashableLuaValue::LuaNumber(2)], AnyLuaValue::LuaNumber(2.));
        assert_eq!(read[&AnyHashableLuaValue::LuaNumber(42)], AnyLuaValue::LuaNumber(42.));
        assert_eq!(read.len(), 3);
    }

    #[test]
    fn reading_hashmap_with_empty_table_works() {
        let mut lua = Lua::new();

        lua.execute::<()>(r#"v = { }"#).unwrap();

        let read: HashMap<_, _> = lua.get("v").unwrap();
        assert_eq!(read.len(), 0);
    }

    #[test]
    fn reading_hashmap_with_complex_indexes_works() {
        let mut lua = Lua::new();

        lua.execute::<()>(r#"v = { [-1] = -1, ["foo"] = 2, [2.] = 42 }"#).unwrap();

        let read: HashMap<_, _> = lua.get("v").unwrap();
        assert_eq!(read[&AnyHashableLuaValue::LuaNumber(-1)], AnyLuaValue::LuaNumber(-1.));
        assert_eq!(read[&AnyHashableLuaValue::LuaString("foo".to_owned())], AnyLuaValue::LuaNumber(2.));
        assert_eq!(read[&AnyHashableLuaValue::LuaNumber(2)], AnyLuaValue::LuaNumber(42.));
        assert_eq!(read.len(), 3);
    }

    #[test]
    fn reading_hashmap_with_floating_indexes_works() {
        let mut lua = Lua::new();

        lua.execute::<()>(r#"v = { [-1.25] = -1, [2.5] = 42 }"#).unwrap();

        let read: HashMap<_, _> = lua.get("v").unwrap();
        // It works by truncating integers in some unspecified way
        // https://www.lua.org/manual/5.2/manual.html#lua_tointegerx
        assert_eq!(read[&AnyHashableLuaValue::LuaNumber(-1)], AnyLuaValue::LuaNumber(-1.));
        assert_eq!(read[&AnyHashableLuaValue::LuaNumber(2)], AnyLuaValue::LuaNumber(42.));
        assert_eq!(read.len(), 2);
    }

    #[test]
    fn reading_heterogenous_hashmap_works() {
        let mut lua = Lua::new();

        let mut orig = HashMap::new();
        orig.insert(AnyHashableLuaValue::LuaNumber(42), AnyLuaValue::LuaNumber(42.));
        orig.insert(AnyHashableLuaValue::LuaString("foo".to_owned()), AnyLuaValue::LuaString("foo".to_owned()));
        orig.insert(AnyHashableLuaValue::LuaBoolean(true), AnyLuaValue::LuaBoolean(true));

        let orig_clone = orig.clone();
        lua.set("v", orig);

        let read: HashMap<_, _> = lua.get("v").unwrap();
        assert_eq!(read, orig_clone);
    }

    #[test]
    fn reading_hashmap_set_from_lua_works() {
        let mut lua = Lua::new();

        lua.execute::<()>(r#"v = { [1] = 2, [2] = 3, [3] = 4 }"#).unwrap();

        let read: HashMap<_, _> = lua.get("v").unwrap();
        assert_eq!(
            read,
            [2., 3., 4.].iter().enumerate()
                .map(|(k, v)| (AnyHashableLuaValue::LuaNumber((k + 1) as i32), AnyLuaValue::LuaNumber(*v))).collect::<HashMap<_, _>>());
    }
}
