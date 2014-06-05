extern crate libc;
extern crate std;

use super::liblua;
use super::Index;
use super::Lua;
use super::Pushable;
use super::Readable;

impl<T:Pushable> Pushable for Vec<T> {
    fn push_to_lua(self, lua: &Lua) {
        unimplemented!()
    }
}
