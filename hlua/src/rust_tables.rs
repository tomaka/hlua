use ffi;

use Push;
use PushGuard;
use AsMutLua;
use Void;

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
            Err((err, lua)) => panic!(),     // TODO: wrong   return Err((err, lua)),      // FIXME: destroy the temporary table
        };

        match size {
            0 => continue,
            1 => {
                let index = index as u32;
                match index.push_to_lua(&mut lua) {
                    Ok(pushed) => pushed.forget(),
                    Err(_) => unreachable!()
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
            Err((err, lua)) => panic!()     // TODO: wrong   return Err((err, lua)),      // FIXME: destroy the temporary table
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
    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (E, L)>  {
        push_iter(lua, self.into_iter())
    }
}

impl<'a, 'lua, L, T, E> Push<L> for &'a [T]
    where L: AsMutLua<'lua>,
          T: Clone + for<'b> Push<&'b mut L, Err = E>
{
    type Err = E;

    #[inline]
    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (E, L)>  {
        push_iter(lua, self.iter().map(|e| e.clone()))
    }
}

// TODO: use an enum for the error to allow different error types for K and V
impl<'lua, L, K, V, E> Push<L> for HashMap<K, V>
    where L: AsMutLua<'lua>,
          K: for<'a, 'b> Push<&'a mut &'b mut L, Err = E> + Eq + Hash,
          V: for<'a, 'b> Push<&'a mut &'b mut L, Err = E>
{
    type Err = Void;      // TODO: can't use E because pushing tuples

    #[inline]
    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (Void, L)>  {
        push_rec_iter(lua, self.into_iter())
    }
}

impl<'lua, L, K, E> Push<L> for HashSet<K>
    where L: AsMutLua<'lua>,
          K: for<'a, 'b> Push<&'a mut &'b mut L, Err = E> + Eq + Hash
{
    type Err = Void;      // TODO: can't use E because pushing tuples

    #[inline]
    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (Void, L)> {
        push_rec_iter(lua, self.into_iter().zip(iter::repeat(true)))
    }
}
