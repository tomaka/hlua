use ffi;
use Lua;
use HasLua;
use CopyRead;
use ConsumeRead;
use Push;
use LuaTable;
use std::intrinsics::TypeId;

extern fn destructor_wrapper(lua: *mut ffi::lua_State) -> ::libc::c_int {
    use std::mem;

    let impl_raw = unsafe { ffi::lua_touserdata(lua, ffi::lua_upvalueindex(1)) };
    let imp: fn(*mut ffi::lua_State)->::libc::c_int = unsafe { mem::transmute(impl_raw) };

    imp(lua)
}

fn destructor_impl<T>(lua: *mut ffi::lua_State) -> ::libc::c_int {
    use std::mem;

    let obj = unsafe { ffi::lua_touserdata(lua, -1) };
    let obj: &mut T = unsafe { mem::transmute(obj) };
    mem::replace(obj, unsafe { mem::uninitialized() });

    0
}


/// Pushes an object as a user data.
/// 
/// # Arguments
///  * metatable: Function that fills the metatable of the object.
// TODO: the type must be Send because the Lua context is Send, but this conflicts with &str
#[experimental]
pub fn push_userdata<'a, 'lua, T: 'static>(data: T, lua: &'a mut Lua<'lua>, metatable: |&mut LuaTable<'a, Lua<'lua>>|) -> uint {
    use std::mem;

    let typeid = format!("{}", TypeId::of::<T>());

    let luaDataRaw = unsafe { ffi::lua_newuserdata(lua.lua, mem::size_of_val(&data) as ::libc::size_t) };
    let luaData: *mut T = unsafe { mem::transmute(luaDataRaw) };
    unsafe { use std::ptr; ptr::write(luaData, data) };

    let lua_raw = lua.use_lua();

    // creating a metatable
    unsafe {
        ffi::lua_newtable(lua.lua);

        // index "__typeid" corresponds to the hash of the TypeId of T
        "__typeid".push_to_lua(lua);
        typeid.push_to_lua(lua);
        ffi::lua_settable(lua.lua, -3);

        // index "__gc" call the object's destructor
        {
            "__gc".push_to_lua(lua);

            // pushing destructor_impl as a lightuserdata
            let destructor_impl: fn(*mut ffi::lua_State)->::libc::c_int = destructor_impl::<T>;
            ffi::lua_pushlightuserdata(lua.use_lua(), mem::transmute(destructor_impl));

            // pushing destructor_wrapper as a closure
            ffi::lua_pushcclosure(lua.use_lua(), mem::transmute(destructor_wrapper), 1);

            ffi::lua_settable(lua.lua, -3);
        }

        {
            let mut table = ConsumeRead::read_from_variable(::LoadedVariable { lua: lua, size: 1 }).ok().unwrap();
            metatable(&mut table);
            mem::forget(table);
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
