#[macro_export]
macro_rules! implement_lua_push {
    ($ty:ty, $cb:expr) => {
        impl<L> $crate::Push<L> for Sound where L: $crate::AsMutLua {
            fn push_to_lua(self, lua: L) -> $crate::PushGuard<L> {
                $crate::userdata::push_userdata(self, lua, $cb)
            }
        }
    };
}

#[macro_export]
macro_rules! implement_lua_read {
    ($ty:ty) => {
        /*impl<'c: 's, 's> hlua::LuaRead<&'c mut hlua::InsideCallback> for &'s mut Sound {
            fn lua_read_at_position(lua: &'c mut hlua::InsideCallback, index: i32) -> Result<&'s mut Sound, &'c mut hlua::InsideCallback> {
                hlua::userdata::read_userdata_ref(lua, index)
            }
        }*/
    };
}
