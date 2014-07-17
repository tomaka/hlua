use super::ffi;
use super::Push;
use HasLua;

fn push_iter<L: HasLua, V: Push<L>, I: Iterator<V>>(lua: &mut L, iterator: I) -> uint
{
    // creating empty table
    unsafe { ffi::lua_newtable(lua.use_lua()) };

    for (elem, index) in iterator.zip(::std::iter::count(1u, 1u)) {
        let pushedCnt = elem.push_to_lua(lua);

        match pushedCnt {
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
