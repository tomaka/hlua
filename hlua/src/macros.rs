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
        impl<'s, 'c> hlua::LuaRead<&'c mut hlua::InsideCallback> for &'s mut Sound {
            fn lua_read_at_position(lua: &'c mut hlua::InsideCallback, index: i32) -> Result<&'s mut Sound, &'c mut hlua::InsideCallback> {
                // FIXME: 
                unsafe { ::std::mem::transmute($crate::userdata::read_userdata::<$ty>(lua, index)) }
            }
        }

        impl<'s, 'c> hlua::LuaRead<&'c mut hlua::InsideCallback> for &'s Sound {
            fn lua_read_at_position(lua: &'c mut hlua::InsideCallback, index: i32) -> Result<&'s Sound, &'c mut hlua::InsideCallback> {
                // FIXME: 
                unsafe { ::std::mem::transmute($crate::userdata::read_userdata::<$ty>(lua, index)) }
            }
        }
    };
}
