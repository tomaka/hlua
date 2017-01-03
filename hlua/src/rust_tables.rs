use ffi;

use Push;
use PushGuard;
use PushOne;
use AsMutLua;
use TuplePushError;

use std::collections::{HashMap, HashSet};
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

    for (elem, index) in iterator.zip((1..)) {
        let size = match elem.push_to_lua(&mut lua) {
            Ok(pushed) => pushed.forget(),
            Err((_err, _lua)) => panic!(),     // TODO: wrong   return Err((err, lua)),      // FIXME: destroy the temporary table
        };

        match size {
            0 => continue,
            1 => {
                let index = index as u32;
                match index.push_to_lua(&mut lua) {
                    Ok(pushed) => pushed.forget(),
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
            Ok(pushed) => pushed.forget(),
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
    use std::collections::HashMap;
    use std::collections::HashSet;
    use Lua;
    use LuaTable;

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
}
