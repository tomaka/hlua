extern crate lua = "rust-hl-lua";

#[test]
fn readwrite() {
    #[deriving(Clone)]
    struct Foo;
    impl<'a> lua::Pushable<'a> for Foo {
        fn push_to_lua(self, lua: &mut lua::Lua<'a>) -> uint {
            lua::userdata::push_userdata(self, lua, |_|{})
        }
    }
    impl lua::CopyReadable for Foo {}

    let mut lua = lua::Lua::new();

    lua.set("a", Foo);
   // let x: Foo = lua.get("a").unwrap();
}

#[test]
fn destructor_called() {
    // TODO: 
    /*let called = ::std::sync::Arc::new(::std::sync::Mutex::new(false));

    struct Foo {
        called: ::std::sync::Arc<::std::sync::Mutex<bool>>
    }

    impl Drop for Foo {
        fn drop(&mut self) {
            let mut called = self.called.lock();
            (*called) = true;
        }
    }

    impl<'a> ::Pushable<'a> for Foo {}

    {
        let mut lua = Lua::new();
        lua.set("a", Foo{called: called.clone()});
    }

    let locked = called.lock();
    assert!(*locked);*/
}

#[test]
fn type_check() {
    #[deriving(Clone)]
    struct Foo;
    impl<'a> lua::Pushable<'a> for Foo {
        fn push_to_lua(self, lua: &mut lua::Lua<'a>) -> uint {
            lua::userdata::push_userdata(self, lua, |_|{})
        }
    }
    impl lua::CopyReadable for Foo {}

    #[deriving(Clone)]
    struct Bar;
    impl<'a> lua::Pushable<'a> for Bar {
        fn push_to_lua(self, lua: &mut lua::Lua<'a>) -> uint {
            lua::userdata::push_userdata(self, lua, |_|{})
        }
    }
    impl lua::CopyReadable for Bar {}

    let mut lua = lua::Lua::new();

    lua.set("a", Foo);
    
    /*let x: Option<Bar> = lua.get("a");
    assert!(x.is_none())*/
}

#[test]
fn metatables() {
    #[deriving(Clone)]
    struct Foo;
    impl<'a> lua::Pushable<'a> for Foo {
        fn push_to_lua(self, lua: &mut lua::Lua<'a>) -> uint {
            lua::userdata::push_userdata(self, lua, |table| {
                table.set("__index".to_string(), vec!(
                    ("test".to_string(), || 5i)
                ));
            })
        }
    }

    let mut lua = lua::Lua::new();

    lua.set("a", Foo);

    let x: int = lua.execute("return a.test()").unwrap();
    assert_eq!(x, 5);
}
