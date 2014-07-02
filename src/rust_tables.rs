use super::ffi;
use super::Lua;
use super::Pushable;

fn push_iter<V: Pushable, I: Iterator<V>>(lua: &mut Lua, iterator: I) -> uint
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

impl<T: Pushable> Pushable for Vec<T> {
    fn push_to_lua(self, lua: &mut Lua) -> uint {
        push_iter(lua, self.move_iter())
    }
}

impl<'a, T: Pushable + Clone> Pushable for &'a [T] {
    fn push_to_lua(self, lua: &mut Lua) -> uint {
        push_iter(lua, self.iter().map(|e| e.clone()))
    }
}

#[cfg(test)]
mod tests {
    use Lua;
    use LuaTable;

    #[test]
    fn write() {
        let mut lua = Lua::new();

        lua.set("a", vec!(9i, 8, 7));

        let mut table: LuaTable = lua.get("a").unwrap();

        let values: Vec<(int,int)> = table.iter().filter_map(|e| e).collect();
        assert_eq!(values, vec!( (1, 9), (2, 8), (3, 7) ));
    }
}
