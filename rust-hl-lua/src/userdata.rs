use {HasLua, CopyRead, ConsumeRead, Push, LoadedVariable, LuaTable};
use std::intrinsics::TypeId;
use ffi;

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
/// In Lua, a user data is anything that is not recognized by Lua. When the script attempts to
///  copy a user data, instead only a reference to the data is copied.
///
/// The way a Lua script can use the user data depends on the content of the **metatable**, which
///  is a Lua table linked to the object.
/// 
/// [See this link for more infos.](http://www.lua.org/manual/5.2/manual.html#2.4)
/// 
/// # Arguments
///  * metatable: Function that fills the metatable of the object.
#[experimental]
pub fn push_userdata<L: HasLua, T: 'static + Send>(data: T, lua: &mut L,
    metatable: |&mut LuaTable<L>|) -> uint
{
    use std::mem;

    let typeid = format!("{}", TypeId::of::<T>());

    let lua_data_raw = unsafe { ffi::lua_newuserdata(lua.use_lua(),
        mem::size_of_val(&data) as ::libc::size_t) };
    let lua_data: *mut T = unsafe { mem::transmute(lua_data_raw) };
    unsafe { use std::ptr; ptr::write(lua_data, data) };

    let lua_raw = lua.use_lua();

    // creating a metatable
    unsafe {
        ffi::lua_newtable(lua.use_lua());

        // index "__typeid" corresponds to the hash of the TypeId of T
        "__typeid".push_to_lua(lua);
        typeid.push_to_lua(lua);
        ffi::lua_settable(lua.use_lua(), -3);

        // index "__gc" call the object's destructor
        {
            "__gc".push_to_lua(lua);

            // pushing destructor_impl as a lightuserdata
            let destructor_impl: fn(*mut ffi::lua_State)->::libc::c_int = destructor_impl::<T>;
            ffi::lua_pushlightuserdata(lua.use_lua(), mem::transmute(destructor_impl));

            // pushing destructor_wrapper as a closure
            ffi::lua_pushcclosure(lua.use_lua(), mem::transmute(destructor_wrapper), 1);

            ffi::lua_settable(lua.use_lua(), -3);
        }

        // calling the metatable closure
        {
            use LoadedVariable;
            let mut table = ConsumeRead::read_from_variable(LoadedVariable{ lua: lua, size: 1 })
                .ok().unwrap();
            metatable(&mut table);
            mem::forget(table);
        }

        ffi::lua_setmetatable(lua_raw, -2);
    }

    1
}

#[experimental]
pub fn read_consume_userdata<'a, L: HasLua, T: 'static>(mut var: LoadedVariable<'a, L>)
    -> Result<UserdataOnStack<'a, L, T>, LoadedVariable<'a, L>> 
{
    unsafe {
        let expectedTypeid = format!("{}", TypeId::of::<T>());

        let dataPtr = ffi::lua_touserdata(var.use_lua(), -1);
        if dataPtr.is_null() {
            return Err(var);
        }

        if ffi::lua_getmetatable(var.use_lua(), -1) == 0 {
            return Err(var);
        }

        "__typeid".push_to_lua(&mut var);
        ffi::lua_gettable(var.use_lua(), -2);
        if CopyRead::read_from_lua(&mut var, -1) != Some(expectedTypeid) {
            return Err(var);
        }
        ffi::lua_pop(var.use_lua(), -2);

        Ok(UserdataOnStack { variable: var })
    }
}

#[experimental]
pub fn read_copy_userdata<L: HasLua, T: Clone + 'static>(lua: &mut L, index: ::libc::c_int) -> Option<T> {
    unsafe {
        let expectedTypeid = format!("{}", TypeId::of::<T>());

        let dataPtr = ffi::lua_touserdata(lua.use_lua(), index);
        if dataPtr.is_null() {
            return None;
        }

        if ffi::lua_getmetatable(lua.use_lua(), -1) == 0 {
            return None;
        }

        "__typeid".push_to_lua(lua);
        ffi::lua_gettable(lua.use_lua(), -2);
        if CopyRead::read_from_lua(lua, -1) != Some(expectedTypeid) {
            return None;
        }
        ffi::lua_pop(lua.use_lua(), -2);

        let data: &T = ::std::mem::transmute(dataPtr);
        Some(data.clone())
    }
}

pub struct UserdataOnStack<'a, L, T> {
    variable: LoadedVariable<'a, L>,
}

impl<'a, L: HasLua, T> Deref<T> for UserdataOnStack<'a, L, T> {
    fn deref(&self) -> &T {
        use std::mem;
        let me: &mut UserdataOnStack<L, T> = unsafe { mem::transmute(self) };
        let data = unsafe { ffi::lua_touserdata(me.variable.use_lua(), -1) };
        let data: &T = unsafe { mem::transmute(data) };
        data
    }
}

impl<'a, L: HasLua, T> DerefMut<T> for UserdataOnStack<'a, L, T> {
    fn deref_mut(&mut self) -> &mut T {
        use std::mem;
        let data = unsafe { ffi::lua_touserdata(self.variable.use_lua(), -1) };
        let data: &mut T = unsafe { mem::transmute(data) };
        data
    }
}
