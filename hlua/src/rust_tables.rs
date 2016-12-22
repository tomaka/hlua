use ffi;

use Push;
use PushGuard;
use AsMutLua;

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[inline]
fn push_iter<'lua, L, V, I>(mut lua: L, iterator: I) -> PushGuard<L>
    where L: AsMutLua<'lua>,
          V: for<'b> Push<&'b mut L>,
          I: Iterator<Item = V>
{
    // creating empty table
    unsafe { ffi::lua_newtable(lua.as_mut_lua().0) };

    for (elem, index) in iterator.zip((1..)) {
        let size = elem.push_to_lua(&mut lua).forget();

        match size {
            0 => continue,
            1 => {
                let index = index as u32;
                index.push_to_lua(&mut lua).forget();
                unsafe { ffi::lua_insert(lua.as_mut_lua().0, -2) }
                unsafe { ffi::lua_settable(lua.as_mut_lua().0, -3) }
            }
            2 => unsafe { ffi::lua_settable(lua.as_mut_lua().0, -3) },
            _ => unreachable!(),
        }
    }

    let raw_lua = lua.as_lua();
    PushGuard {
        lua: lua,
        size: 1,
        raw_lua: raw_lua,
    }
}

#[inline]
fn push_rec_iter<'lua, L, V, I>(mut lua: L, iterator: I) -> PushGuard<L>
    where L: AsMutLua<'lua>,
          V: for<'a> Push<&'a mut L>,
          I: Iterator<Item = V>
{
    let (nrec, _) = iterator.size_hint();

    // creating empty table with pre-allocated non-array elements
    unsafe { ffi::lua_createtable(lua.as_mut_lua().0, 0, nrec as i32) };

    for elem in iterator {
        let size = elem.push_to_lua(&mut lua).forget();

        match size {
            0 => continue,
            2 => unsafe { ffi::lua_settable(lua.as_mut_lua().0, -3) },
            _ => unreachable!(),
        }
    }

    let raw_lua = lua.as_lua();
    PushGuard {
        lua: lua,
        size: 1,
        raw_lua: raw_lua,
    }
}

impl<'lua, L, T> Push<L> for Vec<T>
    where L: AsMutLua<'lua>,
          T: for<'a> Push<&'a mut L>
{
    #[inline]
    fn push_to_lua(self, lua: L) -> PushGuard<L> {
        push_iter(lua, self.into_iter())
    }
}

impl<'a, 'lua, L, T> Push<L> for &'a [T]
    where L: AsMutLua<'lua>,
          T: Clone + for<'b> Push<&'b mut L>
{
    #[inline]
    fn push_to_lua(self, lua: L) -> PushGuard<L> {
        push_iter(lua, self.iter().map(|e| e.clone()))
    }
}

impl<'lua, L, K, V> Push<L> for HashMap<K, V>
    where L: AsMutLua<'lua>,
          K: for<'a, 'b> Push<&'a mut &'b mut L> + Eq + Hash,
          V: for<'a, 'b> Push<&'a mut &'b mut L>
{
    #[inline]
    fn push_to_lua(self, lua: L) -> PushGuard<L> {
        push_rec_iter(lua, self.into_iter())
    }
}

impl<'lua, L, K> Push<L> for HashSet<K>
    where L: AsMutLua<'lua>,
          K: for<'a, 'b> Push<&'a mut &'b mut L> + Eq + Hash
{
    #[inline]
    fn push_to_lua(self, lua: L) -> PushGuard<L> {
        use std::iter;
        push_rec_iter(lua, self.into_iter().zip(iter::repeat(true)))
    }
}
