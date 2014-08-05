use super::ffi;
use super::Push;
use HasLua;

use std::collections::hashmap::{HashMap, HashSet};
use std::iter::Repeat;
use collections::hash::Hash;

fn push_iter<L: HasLua, V: Push<L>, I: Iterator<V>>(lua: &mut L, iterator: I) -> uint {
    // creating empty table
    unsafe { ffi::lua_newtable(lua.use_lua()) };

    for (elem, index) in iterator.zip(::std::iter::count(1u, 1u)) {
        let pushed_cnt = elem.push_to_lua(lua);

        match pushed_cnt {
            0 => continue,
            1 => {
                index.push_to_lua(lua);
                unsafe { ffi::lua_insert(lua.use_lua(), -2) }
                unsafe { ffi::lua_settable(lua.use_lua(), -3) }
            },
            2 => unsafe { ffi::lua_settable(lua.use_lua(), -3) },
            _ => fail!()
        }
    }

    1
}

fn push_rec_iter<L: HasLua, V: Push<L>, I: Iterator<V>>(lua: &mut L, mut iterator: I)
                                                                   -> uint
{
    let (nrec, _) = iterator.size_hint();

    // creating empty table with pre-allocated non-array elements
    unsafe { ffi::lua_createtable(lua.use_lua(), 0, nrec as i32) };

    for elem in iterator {
        let pushed_cnt = elem.push_to_lua(lua);

        match pushed_cnt {
            0 => continue,
            2 => unsafe { ffi::lua_settable(lua.use_lua(), -3) },
            _ => fail!()
        }
    }

    1
}

impl<L: HasLua, T: Push<L>> Push<L> for Vec<T> {
    fn push_to_lua(self, lua: &mut L) -> uint {
        push_iter(lua, self.move_iter())
    }
}

impl<'a, L: HasLua, T: Push<L> + Clone> Push<L> for &'a [T] {
    fn push_to_lua(self, lua: &mut L) -> uint {
        push_iter(lua, self.iter().map(|e| e.clone()))
    }
}

impl<L: HasLua, K: Push<L> + Eq + Hash, V: Push<L>> Push<L> for HashMap<K, V> {
    fn push_to_lua(self, lua: &mut L) -> uint {
        push_rec_iter(lua, self.move_iter())
    }
}

impl<L: HasLua, K: Push<L> + Eq + Hash> Push<L> for HashSet<K> {
    fn push_to_lua(self, lua: &mut L) -> uint {
        push_rec_iter(lua, self.move_iter().zip(Repeat::new(true)))
    }
}
