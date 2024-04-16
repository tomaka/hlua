#[macro_export]
macro_rules! implement_lua_push {
    ($ty:ty, $cb:expr) => {
        impl<'lua, L> $crate::Push<L> for $ty
        where
            L: $crate::AsMutLua<'lua>,
        {
            type Err = $crate::Void; // TODO: use ! instead
            #[inline]
            fn push_to_lua(self, lua: L) -> Result<$crate::PushGuard<L>, ($crate::Void, L)> {
                Ok($crate::push_userdata(self, lua, $cb))
            }
        }

        impl<'lua, L> $crate::PushOne<L> for $ty where L: $crate::AsMutLua<'lua> {}
    };
}

#[macro_export]
macro_rules! implement_lua_read {
    ($ty:ty) => {
        impl<'s, 'c> hlua::LuaRead<&'c mut hlua::InsideCallback> for &'s mut $ty {
            #[inline]
            fn lua_read_at_position(
                lua: &'c mut hlua::InsideCallback,
                index: i32,
            ) -> Result<&'s mut $ty, &'c mut hlua::InsideCallback> {
                // FIXME:
                unsafe { ::std::mem::transmute($crate::read_userdata::<$ty>(lua, index)) }
            }
        }

        impl<'s, 'c> hlua::LuaRead<&'c mut hlua::InsideCallback> for &'s $ty {
            #[inline]
            fn lua_read_at_position(
                lua: &'c mut hlua::InsideCallback,
                index: i32,
            ) -> Result<&'s $ty, &'c mut hlua::InsideCallback> {
                // FIXME:
                unsafe { ::std::mem::transmute($crate::read_userdata::<$ty>(lua, index)) }
            }
        }

        impl<'s, 'b, 'c> hlua::LuaRead<&'b mut &'c mut hlua::InsideCallback> for &'s mut $ty {
            #[inline]
            fn lua_read_at_position(
                lua: &'b mut &'c mut hlua::InsideCallback,
                index: i32,
            ) -> Result<&'s mut $ty, &'b mut &'c mut hlua::InsideCallback> {
                let ptr_lua = lua as *mut &mut hlua::InsideCallback;
                let deref_lua = unsafe { ::std::ptr::read(ptr_lua) };
                let res = Self::lua_read_at_position(deref_lua, index);
                match res {
                    Ok(x) => Ok(x),
                    _ => Err(lua),
                }
            }
        }

        impl<'s, 'b, 'c> hlua::LuaRead<&'b mut &'c mut hlua::InsideCallback> for &'s $ty {
            #[inline]
            fn lua_read_at_position(
                lua: &'b mut &'c mut hlua::InsideCallback,
                index: i32,
            ) -> Result<&'s $ty, &'b mut &'c mut hlua::InsideCallback> {
                let ptr_lua = lua as *mut &mut hlua::InsideCallback;
                let deref_lua = unsafe { ::std::ptr::read(ptr_lua) };
                let res = Self::lua_read_at_position(deref_lua, index);
                match res {
                    Ok(x) => Ok(x),
                    _ => Err(lua),
                }
            }
        }
    };
}
