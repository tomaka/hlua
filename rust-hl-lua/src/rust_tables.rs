use ffi;

use Push;
use AsLua;
use AsMutLua;

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

fn push_iter<L: AsLua, V: Push<L>, I: Iterator<V>>(lua: &mut L, iterator: I) -> uint {
    // creating empty table
    unsafe { ffi::lua_newtable(lua.as_lua()) };

    for (elem, index) in iterator.zip(::std::iter::count(1u, 1u)) {
        let pushed_cnt = elem.push_to_lua(lua);

        match pushed_cnt {
            0 => continue,
            1 => {
                index.push_to_lua(lua);
                unsafe { ffi::lua_insert(lua.as_lua(), -2) }
                unsafe { ffi::lua_settable(lua.as_lua(), -3) }
            },
            2 => unsafe { ffi::lua_settable(lua.as_lua(), -3) },
            _ => panic!()
        }
    }

    1
}

fn push_rec_iter<L: AsLua, V: Push<L>, I: Iterator<V>>(lua: &mut L, mut iterator: I)
                                                                   -> uint
{
    let (nrec, _) = iterator.size_hint();

    // creating empty table with pre-allocated non-array elements
    unsafe { ffi::lua_createtable(lua.as_lua(), 0, nrec as i32) };

    for elem in iterator {
        let pushed_cnt = elem.push_to_lua(lua);

        match pushed_cnt {
            0 => continue,
            2 => unsafe { ffi::lua_settable(lua.as_lua(), -3) },
            _ => panic!()
        }
    }

    1
}

impl<L: AsLua, T: Push<L>> Push<L> for Vec<T> {
    fn push_to_lua(self, lua: &mut L) -> uint {
        push_iter(lua, self.into_iter())
    }
}

impl<'a, L: AsLua, T: Push<L> + Clone> Push<L> for &'a [T] {
    fn push_to_lua(self, lua: &mut L) -> uint {
        push_iter(lua, self.iter().map(|e| e.clone()))
    }
}

impl<L: AsLua, K: Push<L> + Eq + Hash, V: Push<L>> Push<L> for HashMap<K, V> {
    fn push_to_lua(self, lua: &mut L) -> uint {
        push_rec_iter(lua, self.into_iter())
    }
}

impl<L: AsLua, K: Push<L> + Eq + Hash> Push<L> for HashSet<K> {
    fn push_to_lua(self, lua: &mut L) -> uint {
        use std::iter;
        push_rec_iter(lua, self.into_iter().zip(iter::repeat(true)))
    }
}
