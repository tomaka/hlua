use std::any::TypeId;
use std::ops::{Deref, DerefMut};
use std::mem;
use std::ptr;

use ffi;
use libc;

use AsLua;
use AsMutLua;
use Push;
use PushGuard;
use LuaRead;

use LuaTable;

extern fn destructor_wrapper(lua: *mut ffi::lua_State) -> libc::c_int {
    let impl_raw = unsafe { ffi::lua_touserdata(lua, ffi::lua_upvalueindex(1)) };
    let imp: fn(*mut ffi::lua_State)->::libc::c_int = unsafe { mem::transmute(impl_raw) };

    imp(lua)
}

fn destructor_impl<T>(lua: *mut ffi::lua_State) -> libc::c_int {
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
pub fn push_userdata<L, T, F>(data: T, mut lua: L, mut metatable: F) -> PushGuard<L>
                              where F: FnMut(LuaTable<PushGuard<&mut L>>), L: AsMutLua,
                                    T: Send + 'static
{
    let typeid = format!("{:?}", TypeId::of::<T>());

    let lua_data_raw = unsafe { ffi::lua_newuserdata(lua.as_mut_lua().0,
                                                     mem::size_of_val(&data) as libc::size_t) };
    let lua_data: *mut T = unsafe { mem::transmute(lua_data_raw) };
    unsafe { ptr::write(lua_data, data) };

    let lua_raw = lua.as_mut_lua();

    // creating a metatable
    unsafe {
        ffi::lua_newtable(lua.as_mut_lua().0);

        // index "__typeid" corresponds to the hash of the TypeId of T
        "__typeid".push_to_lua(&mut lua);
        typeid.push_to_lua(&mut lua);
        ffi::lua_settable(lua.as_mut_lua().0, -3);

        // index "__gc" call the object's destructor
        {
            "__gc".push_to_lua(&mut lua);

            // pushing destructor_impl as a lightuserdata
            let destructor_impl: fn(*mut ffi::lua_State) -> libc::c_int = destructor_impl::<T>;
            ffi::lua_pushlightuserdata(lua.as_mut_lua().0, mem::transmute(destructor_impl));

            // pushing destructor_wrapper as a closure
            ffi::lua_pushcclosure(lua.as_mut_lua().0, mem::transmute(destructor_wrapper), 1);

            ffi::lua_settable(lua.as_mut_lua().0, -3);
        }

        // calling the metatable closure
        {
            let table = LuaRead::lua_read(PushGuard { lua: &mut lua, size: 1 }).unwrap();
            metatable(table);
        }

        ffi::lua_setmetatable(lua_raw.0, -2);
    }

    PushGuard { lua: lua, size: 1 }
}
/*
pub fn read_consume_userdata<'a, L: AsLua, T: 'static>(mut var: LoadedVariable<'a, L>)
    -> Result<UserdataOnStack<'a, L, T>, LoadedVariable<'a, L>>
{
    unsafe {
        let expected_typeid = format!("{}", TypeId::of::<T>());

        let data_ptr = ffi::lua_touserdata(var.as_lua(), -1);
        if data_ptr.is_null() {
            return Err(var);
        }

        if ffi::lua_getmetatable(var.as_lua(), -1) == 0 {
            return Err(var);
        }

        "__typeid".push_to_lua(&mut var);
        ffi::lua_gettable(var.as_lua(), -2);
        if CopyRead::read_from_lua(&mut var, -1) != Some(expected_typeid) {
            return Err(var);
        }
        ffi::lua_pop(var.as_lua(), -2);

        Ok(UserdataOnStack { variable: var })
    }
}

pub struct UserdataOnStack<'a, L: 'a, T> {
    variable: LoadedVariable<'a, L>,
}

impl<'a, L: AsLua, T> Deref<T> for UserdataOnStack<'a, L, T> {
    fn deref(&self) -> &T {
        use std::mem;
        let me: &mut UserdataOnStack<L, T> = unsafe { mem::transmute(self) };
        let data = unsafe { ffi::lua_touserdata(me.variable.as_lua(), -1) };
        let data: &T = unsafe { mem::transmute(data) };
        data
    }
}

impl<'a, L: AsLua, T> DerefMut<T> for UserdataOnStack<'a, L, T> {
    fn deref_mut(&mut self) -> &mut T {
        use std::mem;
        let data = unsafe { ffi::lua_touserdata(self.variable.as_lua(), -1) };
        let data: &mut T = unsafe { mem::transmute(data) };
        data
    }
}
*/