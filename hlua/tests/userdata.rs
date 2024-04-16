extern crate hlua;

#[test]
fn readwrite() {
    #[derive(Clone)]
    struct Foo;
    impl<'lua, L> hlua::Push<L> for Foo
    where
        L: hlua::AsMutLua<'lua>,
    {
        type Err = hlua::Void;
        fn push_to_lua(self, lua: L) -> Result<hlua::PushGuard<L>, (hlua::Void, L)> {
            Ok(hlua::push_userdata(self, lua, |_| {}))
        }
    }
    impl<'lua, L> hlua::PushOne<L> for Foo where L: hlua::AsMutLua<'lua> {}
    impl<'lua, L> hlua::LuaRead<L> for Foo
    where
        L: hlua::AsMutLua<'lua>,
    {
        fn lua_read_at_position(lua: L, index: i32) -> Result<Foo, L> {
            let val: Result<hlua::UserdataOnStack<Foo, _>, _> =
                hlua::LuaRead::lua_read_at_position(lua, index);
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
        called: Arc<Mutex<bool>>,
    }

    impl Drop for Foo {
        fn drop(&mut self) {
            let mut called = self.called.lock().unwrap();
            (*called) = true;
        }
    }

    impl<'lua, L> hlua::Push<L> for Foo
    where
        L: hlua::AsMutLua<'lua>,
    {
        type Err = hlua::Void;
        fn push_to_lua(self, lua: L) -> Result<hlua::PushGuard<L>, (hlua::Void, L)> {
            Ok(hlua::push_userdata(self, lua, |_| {}))
        }
    }
    impl<'lua, L> hlua::PushOne<L> for Foo where L: hlua::AsMutLua<'lua> {}

    {
        let mut lua = hlua::Lua::new();
        lua.set(
            "a",
            Foo {
                called: called.clone(),
            },
        );
    }

    let locked = called.lock().unwrap();
    assert!(*locked);
}

#[test]
fn type_check() {
    #[derive(Clone)]
    struct Foo;
    impl<'lua, L> hlua::Push<L> for Foo
    where
        L: hlua::AsMutLua<'lua>,
    {
        type Err = hlua::Void;
        fn push_to_lua(self, lua: L) -> Result<hlua::PushGuard<L>, (hlua::Void, L)> {
            Ok(hlua::push_userdata(self, lua, |_| {}))
        }
    }
    impl<'lua, L> hlua::PushOne<L> for Foo where L: hlua::AsMutLua<'lua> {}
    impl<'lua, L> hlua::LuaRead<L> for Foo
    where
        L: hlua::AsMutLua<'lua>,
    {
        fn lua_read_at_position(lua: L, index: i32) -> Result<Foo, L> {
            let val: Result<hlua::UserdataOnStack<Foo, _>, _> =
                hlua::LuaRead::lua_read_at_position(lua, index);
            val.map(|d| d.clone())
        }
    }

    #[derive(Clone)]
    struct Bar;
    impl<'lua, L> hlua::Push<L> for Bar
    where
        L: hlua::AsMutLua<'lua>,
    {
        type Err = hlua::Void;
        fn push_to_lua(self, lua: L) -> Result<hlua::PushGuard<L>, (hlua::Void, L)> {
            Ok(hlua::push_userdata(self, lua, |_| {}))
        }
    }
    impl<'lua, L> hlua::PushOne<L> for Bar where L: hlua::AsMutLua<'lua> {}
    impl<'lua, L> hlua::LuaRead<L> for Bar
    where
        L: hlua::AsMutLua<'lua>,
    {
        fn lua_read_at_position(lua: L, index: i32) -> Result<Bar, L> {
            let val: Result<hlua::UserdataOnStack<Bar, _>, _> =
                hlua::LuaRead::lua_read_at_position(lua, index);
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
    impl<'lua, L> hlua::Push<L> for Foo
    where
        L: hlua::AsMutLua<'lua>,
    {
        type Err = hlua::Void;
        fn push_to_lua(self, lua: L) -> Result<hlua::PushGuard<L>, (hlua::Void, L)> {
            Ok(hlua::push_userdata(self, lua, |mut table| {
                table.set(
                    "__index".to_string(),
                    vec![("test".to_string(), hlua::function0(|| 5))],
                );
            }))
        }
    }
    impl<'lua, L> hlua::PushOne<L> for Foo where L: hlua::AsMutLua<'lua> {}

    let mut lua = hlua::Lua::new();

    lua.set("a", Foo);

    let x: i32 = lua.execute("return a.test()").unwrap();
    assert_eq!(x, 5);
}

#[test]
fn multiple_userdata() {
    #[derive(Clone)]
    struct Integer(u32);
    impl<'lua, L> hlua::Push<L> for Integer
    where
        L: hlua::AsMutLua<'lua>,
    {
        type Err = hlua::Void;
        fn push_to_lua(self, lua: L) -> Result<hlua::PushGuard<L>, (hlua::Void, L)> {
            Ok(hlua::push_userdata(self, lua, |_| {}))
        }
    }
    impl<'lua, L> hlua::PushOne<L> for Integer where L: hlua::AsMutLua<'lua> {}
    impl<'lua, L> hlua::LuaRead<L> for Integer
    where
        L: hlua::AsMutLua<'lua>,
    {
        fn lua_read_at_position(lua: L, index: i32) -> Result<Integer, L> {
            let val: Result<hlua::UserdataOnStack<Integer, _>, _> =
                hlua::LuaRead::lua_read_at_position(lua, index);
            val.map(|d| d.clone())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct BigInteger(u32, u32, u32, u32);
    impl<'lua, L> hlua::Push<L> for BigInteger
    where
        L: hlua::AsMutLua<'lua>,
    {
        type Err = hlua::Void;
        fn push_to_lua(self, lua: L) -> Result<hlua::PushGuard<L>, (hlua::Void, L)> {
            Ok(hlua::push_userdata(self, lua, |_| {}))
        }
    }
    impl<'lua, L> hlua::PushOne<L> for BigInteger where L: hlua::AsMutLua<'lua> {}
    impl<'lua, L> hlua::LuaRead<L> for BigInteger
    where
        L: hlua::AsMutLua<'lua>,
    {
        fn lua_read_at_position(lua: L, index: i32) -> Result<BigInteger, L> {
            let val: Result<hlua::UserdataOnStack<BigInteger, _>, _> =
                hlua::LuaRead::lua_read_at_position(lua, index);
            val.map(|d| d.clone())
        }
    }

    let axpy_float = |a: f64, x: Integer, y: Integer| a * x.0 as f64 + y.0 as f64;
    let axpy_float_2 = |a: f64, x: Integer, y: f64| a * x.0 as f64 + y;
    let broadcast_mul =
        |k: Integer, v: BigInteger| BigInteger(k.0 * v.0, k.0 * v.1, k.0 * v.2, k.0 * v.3);
    let collapse = |a: f32, k: Integer, v: BigInteger| {
        (k.0 * v.0) as f32 * a
            + (k.0 * v.1) as f32 * a
            + (k.0 * v.2) as f32 * a
            + (k.0 * v.3) as f32 * a
    };
    let mut lua = hlua::Lua::new();

    let big_integer = BigInteger(531, 246, 1, 953);
    lua.set("a", Integer(19));
    lua.set("b", Integer(114));
    lua.set("c", Integer(96));
    lua.set("d", Integer(313));
    lua.set("v", big_integer.clone());
    lua.set(
        "add",
        hlua::function2(|x: Integer, y: Integer| Integer(x.0 + y.0)),
    );
    lua.set(
        "axpy",
        hlua::function3(|a: Integer, x: Integer, y: Integer| Integer(a.0 * x.0 + y.0)),
    );
    lua.set("axpy_float", hlua::function3(&axpy_float));
    lua.set("axpy_float_2", hlua::function3(&axpy_float_2));
    lua.set("broadcast_mul", hlua::function2(&broadcast_mul));
    lua.set("collapse", hlua::function3(&collapse));

    assert_eq!(
        lua.execute::<Integer>("return add(a, b)").unwrap().0,
        19 + 114
    );
    assert_eq!(
        lua.execute::<Integer>("return add(b, c)").unwrap().0,
        114 + 96
    );
    assert_eq!(
        lua.execute::<Integer>("return add(c, d)").unwrap().0,
        96 + 313
    );
    assert_eq!(
        lua.execute::<Integer>("return axpy(a, b, c)").unwrap().0,
        19 * 114 + 96
    );
    assert_eq!(
        lua.execute::<Integer>("return axpy(b, c, d)").unwrap().0,
        114 * 96 + 313
    );
    assert_eq!(
        lua.execute::<f64>("return axpy_float(2.5, c, d)").unwrap(),
        axpy_float(2.5, Integer(96), Integer(313))
    );
    assert_eq!(
        lua.execute::<BigInteger>("return broadcast_mul(a, v)")
            .unwrap(),
        broadcast_mul(Integer(19), big_integer.clone())
    );
    assert_eq!(
        lua.execute::<BigInteger>("return broadcast_mul(b, v)")
            .unwrap(),
        broadcast_mul(Integer(114), big_integer.clone())
    );
    assert_eq!(
        lua.execute::<f32>("return collapse(19.25, c, v)").unwrap(),
        collapse(19.25, Integer(96), big_integer.clone())
    );
}
