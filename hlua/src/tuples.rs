use std::mem;

use AsMutLua;
use AsLua;

use Push;
use PushGuard;
use LuaRead;

macro_rules! tuple_impl {
    ($ty:ident) => (

    );

    ($first:ident, $($other:ident),+) => (
        #[allow(non_snake_case)]
        impl<LU, $first: for<'a> Push<&'a mut LU>, $($other: for<'a> Push<&'a mut LU>),+> Push<LU> for ($first, $($other),+)
                                                                          where LU: AsMutLua
        {
            fn push_to_lua(self, mut lua: LU) -> PushGuard<LU> {
                match self {
                    ($first, $($other),+) => {
                        let mut total = 0;

                        {
                            let guard = $first.push_to_lua(&mut lua);
                            total += guard.size;
                            unsafe { mem::forget(guard) };
                        }

                        $(
                            {
                                let guard = $other.push_to_lua(&mut lua);
                                total += guard.size;
                                unsafe { mem::forget(guard) };
                            }
                        )+
                        PushGuard { lua: lua, size: total }
                    }
                }
            }
        }

        /*// TODO: what if T or U are also tuples? indices won't match
        #[allow(unused_assignments)]
        impl<LU: AsLua, $($ty: CopyRead<LU>),+> CopyRead<LU> for ($($ty),+) {
            fn read_from_lua(lua: &mut LU, index: i32) -> Option<($($ty),+)> {

                let mut i = index;
                $(
                    let $nb: Option<$ty> = CopyRead::read_from_lua(lua, i);
                    i += 1;
                )+

                if $($nb.is_none())||+ {
                    return None;
                }

                Some(($($nb.unwrap()),+))

            }
        }*/

        tuple_impl!($($other),+);
    );
}

tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M);
