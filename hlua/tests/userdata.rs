extern crate hlua;

#[test]
fn readwrite() {
    #[derive(Clone)]
    struct Foo;
    impl<L> hlua::Push<L> for Foo where L: hlua::AsMutLua {
        fn push_to_lua(self, lua: L) -> hlua::PushGuard<L> {
            hlua::userdata::push_userdata(self, lua, |_|{})
        }
    }
    impl<L> hlua::LuaRead<L> for Foo where L: hlua::AsMutLua {
        fn lua_read_at_position(lua: L, index: i32) -> Result<Foo, L> {
            let val: Result<hlua::userdata::UserdataOnStack<Foo, _>, _> = hlua::LuaRead::lua_read_at_position(lua, index);
            val.map(|d| d.clone())
        }
    }

    let mut lua = hlua::Lua::new();

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

    impl<L> hlua::Push<L> for Foo where L: hlua::AsMutLua {
        fn push_to_lua(self, lua: L) -> hlua::PushGuard<L> {
            hlua::userdata::push_userdata(self, lua, |_|{})
        }
    }

    {
        let mut lua = hlua::Lua::new();
        lua.set("a", Foo{called: called.clone()});
    }

    let locked = called.lock().unwrap();
    assert!(*locked);
}

#[test]
fn type_check() {
    #[derive(Clone)]
    struct Foo;
    impl<L> hlua::Push<L> for Foo where L: hlua::AsMutLua {
        fn push_to_lua(self, lua: L) -> hlua::PushGuard<L> {
            hlua::userdata::push_userdata(self, lua, |_|{})
        }
    }
    impl<L> hlua::LuaRead<L> for Foo where L: hlua::AsMutLua {
        fn lua_read_at_position(lua: L, index: i32) -> Result<Foo, L> {
            let val: Result<hlua::userdata::UserdataOnStack<Foo, _>, _> = hlua::LuaRead::lua_read_at_position(lua, index);
            val.map(|d| d.clone())
        }
    }

    #[derive(Clone)]
    struct Bar;
    impl<L> hlua::Push<L> for Bar where L: hlua::AsMutLua {
        fn push_to_lua(self, lua: L) -> hlua::PushGuard<L> {
            hlua::userdata::push_userdata(self, lua, |_|{})
        }
    }
    impl<L> hlua::LuaRead<L> for Bar where L: hlua::AsMutLua {
        fn lua_read_at_position(lua: L, index: i32) -> Result<Bar, L> {
            let val: Result<hlua::userdata::UserdataOnStack<Bar, _>, _> = hlua::LuaRead::lua_read_at_position(lua, index);
            val.map(|d| d.clone())
        }
    }

    let mut lua = hlua::Lua::new();

    lua.set("a", Foo);
    
    let x: Option<Bar> = lua.get("a");
    assert!(x.is_none())
}

#[test]
fn metatables() {
    #[derive(Clone)]
    struct Foo;
    impl<L> hlua::Push<L> for Foo where L: hlua::AsMutLua {
        fn push_to_lua(self, lua: L) -> hlua::PushGuard<L> {
            hlua::userdata::push_userdata(self, lua, |mut table| {
                table.set("__index".to_string(), vec![
                    ("test".to_string(), hlua::function(|| 5))
                ]);
            })
        }
    }

    let mut lua = hlua::Lua::new();

    lua.set("a", Foo);

    let x: i32 = lua.execute("return a.test()").unwrap();
    assert_eq!(x, 5);
}
