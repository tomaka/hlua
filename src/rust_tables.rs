extern crate libc;
extern crate std;

use super::liblua;
use super::Lua;
use super::Pushable;

fn push_iter<'a, V: Pushable, I: Iterator<&'a V>>(lua: &mut Lua, iterator: I) -> uint
{
    // creating empty table
    unsafe { liblua::lua_newtable(lua.lua) };

    for (elem, index) in iterator.zip(std::iter::count(1u, 1u)) {
        let pushedCnt = elem.push_to_lua(lua);

        match pushedCnt {
            0 => continue,
            1 => {
                index.push_to_lua(lua);
                unsafe { liblua::lua_insert(lua.lua, -2) }
                unsafe { liblua::lua_settable(lua.lua, -3) }
            },
            2 => unsafe { liblua::lua_settable(lua.lua, -3) },
            _ => fail!()
        }
    }

    1
}

impl<T: Pushable> Pushable for Vec<T> {
    fn push_to_lua(&self, lua: &mut Lua) -> uint {
        push_iter(lua, self.iter())
    }
}

impl<'a, T: Pushable> Pushable for &'a [T] {
    fn push_to_lua(&self, lua: &mut Lua) -> uint {
        push_iter(lua, self.iter())
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
