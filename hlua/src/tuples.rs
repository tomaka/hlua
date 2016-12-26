use AsMutLua;
use AsLua;

use Push;
use PushGuard;
use LuaRead;
use Void;

macro_rules! tuple_impl {
    ($ty:ident) => (
        impl<'lua, LU, $ty> Push<LU> for ($ty,) where LU: AsMutLua<'lua>, $ty: Push<LU> {
            type Err = <$ty as Push<LU>>::Err;

            #[inline]
            fn push_to_lua(self, lua: LU) -> Result<PushGuard<LU>, (Self::Err, LU)> {
                self.0.push_to_lua(lua)
            }
        }

        impl<'lua, LU, $ty> LuaRead<LU> for ($ty,) where LU: AsMutLua<'lua>, $ty: LuaRead<LU> {
            #[inline]
            fn lua_read_at_position(lua: LU, index: i32) -> Result<($ty,), LU> {
                LuaRead::lua_read_at_position(lua, index).map(|v| (v,))
            }
        }
    );

    ($first:ident, $($other:ident),+) => (
        #[allow(non_snake_case)]
        impl<'lua, LU, $first: for<'a> Push<&'a mut LU>, $($other: for<'a> Push<&'a mut LU>),+>
            Push<LU> for ($first, $($other),+) where LU: AsMutLua<'lua>
        {
            type Err = Void;      // TODO: wrong

            #[inline]
            fn push_to_lua(self, mut lua: LU) -> Result<PushGuard<LU>, (Self::Err, LU)> {
                match self {
                    ($first, $($other),+) => {
                        let mut total = match $first.push_to_lua(&mut lua) {
                            Ok(pushed) => pushed.forget(),
                            Err(_) => panic!()      // TODO: wrong
                        };

                        $(
                            total += match $other.push_to_lua(&mut lua) {
                                Ok(pushed) => pushed.forget(),
                                Err(_) => panic!()      // TODO: wrong
                            };
                        )+

                        let raw_lua = lua.as_lua();
                        Ok(PushGuard { lua: lua, size: total, raw_lua: raw_lua })
                    }
                }
            }
        }

        // TODO: what if T or U are also tuples? indices won't match
        #[allow(unused_assignments)]
        #[allow(non_snake_case)]
        impl<'lua, LU, $first: for<'a> LuaRead<&'a mut LU>, $($other: for<'a> LuaRead<&'a mut LU>),+>
            LuaRead<LU> for ($first, $($other),+) where LU: AsLua<'lua>
        {
            #[inline]
            fn lua_read_at_position(mut lua: LU, index: i32) -> Result<($first, $($other),+), LU> {
                let mut i = index;

                let $first: $first = match LuaRead::lua_read_at_position(&mut lua, i) {
                    Ok(v) => v,
                    Err(_) => return Err(lua)
                };

                i += 1;

                $(
                    let $other: $other = match LuaRead::lua_read_at_position(&mut lua, i) {
                        Ok(v) => v,
                        Err(_) => return Err(lua)
                    };
                    i += 1;
                )+

                Ok(($first, $($other),+))

            }
        }

        tuple_impl!($($other),+);
    );
}

tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M);
