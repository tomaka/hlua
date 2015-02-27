extern crate "hlua" as lua;

#[test]
fn readwrite() {
    #[derive(Clone)]
    struct Foo;
    impl<L> lua::Push<L> for Foo where L: lua::AsMutLua {
        fn push_to_lua(self, lua: L) -> lua::PushGuard<L> {
            lua::userdata::push_userdata(self, lua, |_|{})
        }
    }
    impl<L> lua::LuaRead<L> for Foo where L: lua::AsMutLua {
        fn lua_read_at_position(lua: L, index: i32) -> Option<Foo> {
            lua::userdata::read_userdata::<Foo, _>(lua, index).map(|d| d.clone())
        }
    }

    let mut lua = lua::Lua::new();

    lua.set("a", Foo);
    let _: Foo = lua.get("a").unwrap();
}

#[test]
fn destructor_called() {
    use std::sync::{Arc, Mutex};

    let called = Arc::new(Mutex::new(false));

    struct Foo {
        called: Arc<Mutex<bool>>
    }

    impl Drop for Foo {
        fn drop(&mut self) {
            let mut called = self.called.lock().unwrap();
            (*called) = true;
        }
    }

    impl<L> lua::Push<L> for Foo where L: lua::AsMutLua {
        fn push_to_lua(self, lua: L) -> lua::PushGuard<L> {
            lua::userdata::push_userdata(self, lua, |_|{})
        }
    }

    {
        let mut lua = lua::Lua::new();
        lua.set("a", Foo{called: called.clone()});
    }

    let locked = called.lock().unwrap();
    assert!(*locked);
}

#[test]
fn type_check() {
    #[derive(Clone)]
    struct Foo;
    impl<L> lua::Push<L> for Foo where L: lua::AsMutLua {
        fn push_to_lua(self, lua: L) -> lua::PushGuard<L> {
            lua::userdata::push_userdata(self, lua, |_|{})
        }
    }
    impl<L> lua::LuaRead<L> for Foo where L: lua::AsMutLua {
        fn lua_read_at_position(lua: L, index: i32) -> Option<Foo> {
            lua::userdata::read_userdata::<Foo, _>(lua, index).map(|d| d.clone())
        }
    }

    #[derive(Clone)]
    struct Bar;
    impl<L> lua::Push<L> for Bar where L: lua::AsMutLua {
        fn push_to_lua(self, lua: L) -> lua::PushGuard<L> {
            lua::userdata::push_userdata(self, lua, |_|{})
        }
    }
    impl<L> lua::LuaRead<L> for Bar where L: lua::AsMutLua {
        fn lua_read_at_position(lua: L, index: i32) -> Option<Bar> {
            lua::userdata::read_userdata::<Bar, _>(lua, index).map(|d| d.clone())
        }
    }

    let mut lua = lua::Lua::new();

    lua.set("a", Foo);
    
    let x: Option<Bar> = lua.get("a");
    assert!(x.is_none())
}

#[test]
fn metatables() {
    #[derive(Clone)]
    struct Foo;
    impl<L> lua::Push<L> for Foo where L: lua::AsMutLua {
        fn push_to_lua(self, lua: L) -> lua::PushGuard<L> {
            lua::userdata::push_userdata(self, lua, |table| {
                table.set("__index".to_string(), vec!(
                    ("test".to_string(), || 5)
                ));
            })
        }
    }

    let mut lua = lua::Lua::new();

    lua.set("a", Foo);

    let x: i32 = lua.execute("return a.test()").unwrap();
    assert_eq!(x, 5);
}
