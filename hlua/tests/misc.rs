extern crate hlua;

use hlua::Lua;
use std::fs::File;

#[test]
#[should_fail]
fn readerrors() {
    let mut lua = Lua::new();
    let file = File::open(&Path::new("/path/to/unexisting/file")).unwrap();
    let _res: () = lua.execute_from_reader(file).unwrap();
}
