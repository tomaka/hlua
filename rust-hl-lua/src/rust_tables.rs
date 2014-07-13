use super::ffi;
use super::Lua;
use super::Pushable;

fn push_iter<'lua, V: Pushable<'lua>, I: Iterator<V>>(lua: &mut Lua<'lua>, iterator: I) -> uint
{
    // creating empty table
    unsafe { ffi::lua_newtable(lua.lua) };

    for (elem, index) in iterator.zip(::std::iter::count(1u, 1u)) {
        let pushedCnt = elem.push_to_lua(lua);

        match pushedCnt {
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

impl<'lua, T: Pushable<'lua>> Pushable<'lua> for Vec<T> {
    fn push_to_lua(self, lua: &mut Lua<'lua>) -> uint {
        push_iter(lua, self.move_iter())
    }
}

impl<'a, 'lua, T: Pushable<'lua> + Clone> Pushable<'lua> for &'a [T] {
    fn push_to_lua(self, lua: &mut Lua<'lua>) -> uint {
        push_iter(lua, self.iter().map(|e| e.clone()))
    }
}
