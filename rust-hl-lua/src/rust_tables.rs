use super::ffi;
use super::Lua;
use super::Push;

use std::collections::hashmap::{HashMap, HashSet};
use std::iter::Repeat;
use collections::hash::Hash;

fn push_iter<'lua, V: Push<'lua>, I: Iterator<V>>(lua: &mut Lua<'lua>, iterator: I) -> uint
{
    // creating empty table
    unsafe { ffi::lua_newtable(lua.lua) };

    for (elem, index) in iterator.zip(::std::iter::count(1u, 1u)) {
        let pushed_cnt = elem.push_to_lua(lua);

        match pushed_cnt {
            0 => continue,
            1 => {
                index.push_to_lua(lua);
                unsafe { ffi::lua_insert(lua.lua, -2) }
                unsafe { ffi::lua_settable(lua.lua, -3) }
            },
            2 => unsafe { ffi::lua_settable(lua.lua, -3) },
            _ => fail!()
        }
    }

    1
}

fn push_rec_iter<'lua, V: Push<'lua>, I: Iterator<V>>(lua: &mut Lua<'lua>, mut iterator: I)
                                                      -> uint
{
    let (nrec, _) = iterator.size_hint();

    // creating empty table with pre-allocated non-array elements
    unsafe { ffi::lua_createtable(lua.lua, 0, nrec as i32) };

    for elem in iterator {
        let pushed_cnt = elem.push_to_lua(lua);

        match pushed_cnt {
            0 => continue,
            2 => unsafe { ffi::lua_settable(lua.lua, -3) },
            _ => fail!()
        }
    }

    1
}

impl<'lua, T: Push<'lua>> Push<'lua> for Vec<T> {
    fn push_to_lua(self, lua: &mut Lua<'lua>) -> uint {
        push_iter(lua, self.move_iter())
    }
}

impl<'a, 'lua, T: Push<'lua> + Clone> Push<'lua> for &'a [T] {
    fn push_to_lua(self, lua: &mut Lua<'lua>) -> uint {
        push_iter(lua, self.iter().map(|e| e.clone()))
    }
}

impl<'lua, K: Push<'lua> + Eq + Hash, V: Push<'lua>> Push<'lua> for HashMap<K, V> {
    fn push_to_lua(self, lua: &mut Lua<'lua>) -> uint {
        push_rec_iter(lua, self.move_iter())
    }
}

impl<'lua, K: Push<'lua> + Eq + Hash> Push<'lua> for HashSet<K> {
    fn push_to_lua(self, lua: &mut Lua<'lua>) -> uint {
        push_rec_iter(lua, self.move_iter().zip(Repeat::new(true)))
    }
}
