use ffi;

use Push;
use PushGuard;
use AsMutLua;

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter;
use std::mem;

fn push_iter<L, V, I>(mut lua: L, iterator: I) -> PushGuard<L>
                      where L: AsMutLua, V: for<'b> Push<&'b mut L>, I: Iterator<Item=V>
{
    // creating empty table
    unsafe { ffi::lua_newtable(lua.as_mut_lua().0) };

    for (elem, index) in iterator.zip(iter::count(1u, 1u)) {
        let size = {
            let pushed_cnt = elem.push_to_lua(&mut lua);

            let size = pushed_cnt.size;
            unsafe { mem::forget(pushed_cnt) };
            size
        };

        match size {
            0 => continue,
            1 => {
                let index = index as u32;
                index.push_to_lua(&mut lua);
                unsafe { ffi::lua_insert(lua.as_mut_lua().0, -2) }
                unsafe { ffi::lua_settable(lua.as_mut_lua().0, -3) }
            },
            2 => unsafe { ffi::lua_settable(lua.as_mut_lua().0, -3) },
            _ => panic!()
        }
    }

    PushGuard { lua: lua, size: 1 }
}

/*fn push_rec_iter<L: AsLua, V: Push<L>, I: Iterator<V>>(lua: &mut L, mut iterator: I)
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
}*/

impl<L, T> Push<L> for Vec<T> where L: AsMutLua, T: for<'a> Push<&'a mut L> {
    fn push_to_lua(self, lua: L) -> PushGuard<L> {
        push_iter(lua, self.into_iter())
    }
}

impl<'a, L, T> Push<L> for &'a [T] where L: AsMutLua, T: Clone + for<'a> Push<&'a mut L> {
    fn push_to_lua(self, lua: L) -> PushGuard<L> {
        push_iter(lua, self.iter().map(|e| e.clone()))
    }
}

/*impl<L: AsLua, K: Push<L> + Eq + Hash, V: Push<L>> Push<L> for HashMap<K, V> {
    fn push_to_lua(self, lua: &mut L) -> uint {
        push_rec_iter(lua, self.into_iter())
    }
}

impl<L: AsLua, K: Push<L> + Eq + Hash> Push<L> for HashSet<K> {
    fn push_to_lua(self, lua: &mut L) -> uint {
        use std::iter;
        push_rec_iter(lua, self.into_iter().zip(iter::repeat(true)))
    }
}*/
