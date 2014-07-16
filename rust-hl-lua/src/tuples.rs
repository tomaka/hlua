use { Lua, Push, CopyRead };

macro_rules! tuple_impl(
    ($($ty:ident | $nb:ident),+) => (
        impl<'lua, $($ty: Push<'lua>),+> Push<'lua> for ($($ty),+) {
            fn push_to_lua(self, lua: &mut Lua<'lua>) -> uint {
                match self {
                    ($($nb),+) => {
                        let mut total = 0;
                        $(total += $nb.push_to_lua(lua);)+
                        total
                    }
                }
            }
        }

        // TODO: what if T or U are also tuples? indices won't match
        #[allow(dead_assignment)]
        impl<$($ty: CopyRead),+> CopyRead for ($($ty),+) {
            fn read_from_lua<'lua>(lua: &mut Lua<'lua>, index: i32) -> Option<($($ty),+)> {

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
        }
    );
)

tuple_impl!(A | ref0, B | ref1)
tuple_impl!(A | ref0, B | ref1, C | ref2)
tuple_impl!(A | ref0, B | ref1, C | ref2, D | ref3)
tuple_impl!(A | ref0, B | ref1, C | ref2, D | ref3, E | ref4)
tuple_impl!(A | ref0, B | ref1, C | ref2, D | ref3, E | ref4, F | ref5)
tuple_impl!(A | ref0, B | ref1, C | ref2, D | ref3, E | ref4, F | ref5, G | ref6)
tuple_impl!(A | ref0, B | ref1, C | ref2, D | ref3, E | ref4, F | ref5, G | ref6, H | ref7)
tuple_impl!(A | ref0, B | ref1, C | ref2, D | ref3, E | ref4, F | ref5, G | ref6, H | ref7, I | ref8)
tuple_impl!(A | ref0, B | ref1, C | ref2, D | ref3, E | ref4, F | ref5, G | ref6, H | ref7, I | ref8, J | ref9)
tuple_impl!(A | ref0, B | ref1, C | ref2, D | ref3, E | ref4, F | ref5, G | ref6, H | ref7, I | ref8, J | ref9, K | ref10)
tuple_impl!(A | ref0, B | ref1, C | ref2, D | ref3, E | ref4, F | ref5, G | ref6, H | ref7, I | ref8, J | ref9, K | ref10, L | ref11)
