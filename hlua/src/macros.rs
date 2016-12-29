#[macro_export]
macro_rules! implement_lua_push {
    ($ty:ty, $cb:expr) => {
        impl<'lua, L> $crate::Push<L> for $ty where L: $crate::AsMutLua<'lua> {
            type Err = $crate::Void;      // TODO: use ! instead
            #[inline]
            fn push_to_lua(self, lua: L) -> Result<$crate::PushGuard<L>, ($crate::Void, L)> {
                Ok($crate::push_userdata(self, lua, $cb))
            }
        }
        
        impl<'lua, L> $crate::PushOne<L> for $ty where L: $crate::AsMutLua<'lua> {
        }
    };
}

#[macro_export]
macro_rules! implement_lua_read {
    ($ty:ty) => {
        impl<'l, 'lua, L> hlua::LuaRead<&'l L> for &'l $ty
            where L: hlua::AsLua<'lua>
        {
            #[inline]
            fn lua_read_at_position(lua: &'l L, index: i32) -> Result<&'l $ty, &'l L> {
                hlua::read_userdata(lua, index)
            }
        }

        /*impl<'l, 'lua, L> hlua::LuaRead<&'l mut L> for &'l $ty
            where L: hlua::AsMutLua<'lua>
        {
            #[inline]
            fn lua_read_at_position(lua: &'l mut L, index: i32) -> Result<&'l $ty, &'l mut L> {
                hlua::read_userdata(lua, index)
            }
        }*/

        impl<'l, 'lua, L> hlua::LuaRead<&'l mut L> for &'l mut $ty
            where L: hlua::AsMutLua<'lua>
        {
            #[inline]
            fn lua_read_at_position(lua: &'l mut L, index: i32) -> Result<&'l mut $ty, &'l mut L> {
                hlua::read_mut_userdata(lua, index)
            }
        }
    };
}
