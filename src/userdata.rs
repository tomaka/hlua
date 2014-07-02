use ffi;
use Lua;
use Pushable;

/*fn destructor<T>(_: T) {

}*/

// TODO: the type must be Send because the Lua context is Send, but this conflicts with &str
pub fn push_userdata<T: ::std::any::Any>(data: T, lua: &mut Lua) -> uint {
    let typeid = format!("{}", data.get_type_id());

    let luaDataRaw = unsafe { ffi::lua_newuserdata(lua.lua, ::std::mem::size_of_val(&data) as ::libc::size_t) };
    let luaData: &mut T = unsafe { ::std::mem::transmute(luaDataRaw) };
    (*luaData) = data;

    // creating a metatable
    unsafe {
        ffi::lua_newtable(lua.lua);

        // index "__typeid" corresponds to the hash of the TypeId of T
        "__typeid".push_to_lua(lua);
        typeid.push_to_lua(lua);
        ffi::lua_settable(lua.lua, -3);

        // index "__gc" call the object's destructor
        // TODO: 
        /*"__gc".push_to_lua(lua);
        destructor::<T>.push_to_lua(lua);
        ffi::lua_settable(lua.lua, -3);*/

        ffi::lua_setmetatable(lua.lua, -2);
    }

    1
}

/*impl<T: Clone + 'static> CopyReadable for UserData<T> {
    fn read_from_lua(lua: &mut Lua, index: i32) -> Option<UserData<T>> {
        unsafe {
            let expectedTypeid = format!("{}", TypeId::of::<T>());

            let dataPtr = ffi::lua_touserdata(lua.lua, index);
            if dataPtr.is_null() {
                return None;
            }

            if ffi::lua_getmetatable(lua.lua, -1) == 0 {
                return None;
            }

            "__typeid".push_to_lua(lua);
            ffi::lua_gettable(lua.lua, -2);
            if CopyReadable::read_from_lua(lua, -1) != Some(expectedTypeid) {
                return None;
            }
            ffi::lua_pop(lua.lua, -2);

            let data: &T = ::std::mem::transmute(dataPtr);
            Some(UserData{value: data.clone()})
        }
    }
}*/

#[cfg(test)]
mod tests {
    use Lua;

    #[test]
    fn readwrite() {
        #[deriving(Clone)]
        struct Foo;
        impl ::Pushable for Foo {}

        let mut lua = Lua::new();

        lua.set("a", Foo);
        //let x: Foo = lua.get("a").unwrap();       // TODO: 
    }

    #[test]
    fn destructor_called() {
        // TODO: how to test this?
    }

    #[test]
    fn type_check() {
        #[deriving(Clone)]
        struct Foo;
        impl ::Pushable for Foo {}
        #[deriving(Clone)]
        struct Bar;
        impl ::Pushable for Bar {}

        let mut lua = Lua::new();

        lua.set("a", Foo);
        
        // TODO: 
        /*let x: Option<Bar> = lua.get("a");
        assert!(x.is_none())*/
    }
}
