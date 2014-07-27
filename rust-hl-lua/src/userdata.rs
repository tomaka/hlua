use ffi;
use Lua;
use HasLua;
use CopyRead;
use ConsumeRead;
use Push;
use LuaTable;
use std::intrinsics::TypeId;

/*fn destructor<T>(_: T) {

}*/

/// Pushes an object as a user data.
/// 
/// # Arguments
///  * metatable: Function that fills the metatable of the object.
// TODO: the type must be Send because the Lua context is Send, but this conflicts with &str
#[experimental]
pub fn push_userdata<'a, 'lua, T: 'static>(data: T, lua: &'a mut Lua<'lua>, metatable: |&mut LuaTable<'a, Lua<'lua>>|) -> uint {
    let typeid = format!("{}", TypeId::of::<T>());

    let luaDataRaw = unsafe { ffi::lua_newuserdata(lua.lua, ::std::mem::size_of_val(&data) as ::libc::size_t) };
    let luaData: *mut T = unsafe { ::std::mem::transmute(luaDataRaw) };
    unsafe { ::std::ptr::write(luaData, data) };

    let lua_raw = lua.use_lua();

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

        ffi::lua_setmetatable(lua_raw, -2);
    }

    1
}

// TODO: the type must be Send because the Lua context is Send, but this conflicts with &str
#[experimental]
pub fn read_copy_userdata<T: Clone + 'static>(lua: &mut Lua, index: ::libc::c_int) -> Option<T> {
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
        if CopyRead::read_from_lua(lua, -1) != Some(expectedTypeid) {
            return None;
        }
        ffi::lua_pop(lua.lua, -2);

        let data: &T = ::std::mem::transmute(dataPtr);
        Some(data.clone())
    }
}
