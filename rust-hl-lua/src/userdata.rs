use ffi;
use Lua;
use CopyRead;
use ConsumeRead;
use Push;
use LuaTable;
use std::any::Any;

/*fn destructor<T>(_: T) {

}*/

/// Pushes an object as a user data.
/// 
/// # Arguments
///  * metatable: Function that fills the metatable of the object.
// TODO: the type must be Send because the Lua context is Send, but this conflicts with &str
#[experimental]
pub fn push_userdata<T: ::std::any::Any>(data: T, lua: &mut Lua, metatable: |&mut LuaTable|) -> uint {
    let typeid = format!("{}", data.get_type_id());

    let luaDataRaw = unsafe { ffi::lua_newuserdata(lua.lua, ::std::mem::size_of_val(&data) as ::libc::size_t) };
    let luaData: *mut T = unsafe { ::std::mem::transmute(luaDataRaw) };
    unsafe { ::std::ptr::write(luaData, data) };

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

        {
            let mut table = ConsumeRead::read_from_variable(::LoadedVariable { lua: lua, size: 1 }).ok().unwrap();
            metatable(&mut table);
            ::std::mem::forget(table);
        }

        ffi::lua_setmetatable(lua.lua, -2);
    }

    1
}

// TODO: the type must be Send because the Lua context is Send, but this conflicts with &str
#[experimental]
pub fn read_copy_userdata<T: Clone + ::std::any::Any>(lua: &mut Lua, index: ::libc::c_int) -> Option<T> {
    unsafe {
        let dummyMe: &T = ::std::mem::uninitialized();      // TODO: this is very very hacky, I don't even know if it works
        let expectedTypeid = format!("{}", dummyMe.get_type_id());

        let dataPtr = ffi::lua_touserdata(lua.lua, index);
        if dataPtr.is_null() {
            return None;
        }

        if ffi::lua_getmetatable(lua.lua, -1) == 0 {
            return None;
        }

        "__typeid".push_to_lua(lua);
        ffi::lua_gettable(lua.lua, -2);
        if CopyRead::read_from_lua(lua, -1) != Some(expectedTypeid) {
            return None;
        }
        ffi::lua_pop(lua.lua, -2);

        let data: &T = ::std::mem::transmute(dataPtr);
        Some(data.clone())
    }
}
