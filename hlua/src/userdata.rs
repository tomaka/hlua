use std::any::TypeId;
use std::marker::PhantomData;
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
                              where F: FnMut(LuaTable<&mut PushGuard<&mut L>>), L: AsMutLua,
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
        "__typeid".push_to_lua(&mut lua).forget();
        typeid.push_to_lua(&mut lua).forget();
        ffi::lua_settable(lua.as_mut_lua().0, -3);

        // index "__gc" call the object's destructor
        {
            "__gc".push_to_lua(&mut lua).forget();

            // pushing destructor_impl as a lightuserdata
            let destructor_impl: fn(*mut ffi::lua_State) -> libc::c_int = destructor_impl::<T>;
            ffi::lua_pushlightuserdata(lua.as_mut_lua().0, mem::transmute(destructor_impl));

            // pushing destructor_wrapper as a closure
            ffi::lua_pushcclosure(lua.as_mut_lua().0, mem::transmute(destructor_wrapper), 1);

            ffi::lua_settable(lua.as_mut_lua().0, -3);
        }

        // calling the metatable closure
        {
            let mut guard = PushGuard { lua: &mut lua, size: 1 };
            metatable(LuaRead::lua_read(&mut guard).unwrap());
            guard.forget();
        }

        ffi::lua_setmetatable(lua_raw.0, -2);
    }

    PushGuard { lua: lua, size: 1 }
}

pub fn read_userdata<T, L>(mut lua: L, index: i32) -> Option<UserdataOnStack<T, L>>
                           where L: AsMutLua, T: 'static
{
    assert!(index == -1);   // FIXME: 

    unsafe {
        let expected_typeid = format!("{:?}", TypeId::of::<T>());

        let data_ptr = ffi::lua_touserdata(lua.as_lua().0, -1);
        if data_ptr.is_null() {
            return None;
        }

        if ffi::lua_getmetatable(lua.as_lua().0, -1) == 0 {
            return None;
        }

        "__typeid".push_to_lua(&mut lua).forget();
        ffi::lua_gettable(lua.as_lua().0, -2);
        if LuaRead::lua_read(&mut lua) != Some(expected_typeid) {
            return None;
        }
        ffi::lua_pop(lua.as_lua().0, -2);

        Some(UserdataOnStack {
            variable: lua,
            marker: PhantomData,
        })
    }
}

pub struct UserdataOnStack<T, L> {
    variable: L,
    marker: PhantomData<T>,
}

impl<T, L> Deref for UserdataOnStack<T, L> where L: AsMutLua, T: 'static {
    type Target = T;

    fn deref(&self) -> &T {
        let me: &mut UserdataOnStack<T, L> = unsafe { mem::transmute(self) };       // FIXME: 
        let data = unsafe { ffi::lua_touserdata(me.variable.as_mut_lua().0, -1) };
        let data: &T = unsafe { mem::transmute(data) };
        data
    }
}

impl<T, L> DerefMut for UserdataOnStack<T, L> where L: AsMutLua, T: 'static {
    fn deref_mut(&mut self) -> &mut T {
        let data = unsafe { ffi::lua_touserdata(self.variable.as_mut_lua().0, -1) };
        let data: &mut T = unsafe { mem::transmute(data) };
        data
    }
}
