use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

use ffi;
use libc;

use AsLua;
use AsMutLua;
use LuaContext;
use LuaRead;
use Push;
use PushGuard;

use InsideCallback;
use LuaTable;

// Called when an object inside Lua is being dropped.
#[inline]
extern "C" fn destructor_wrapper<T>(lua: *mut ffi::lua_State) -> libc::c_int {
    unsafe {
        let obj = ffi::lua_touserdata(lua, -1);
        ptr::drop_in_place(obj as *mut TypeId);
        ptr::drop_in_place((obj as *mut u8).offset(mem::size_of::<TypeId>() as isize) as *mut T);
        0
    }
}

/// Pushes an object as a user data.
///
/// In Lua, a user data is anything that is not recognized by Lua. When the script attempts to
/// copy a user data, instead only a reference to the data is copied.
///
/// The way a Lua script can use the user data depends on the content of the **metatable**, which
/// is a Lua table linked to the object.
///
/// [See this link for more infos.](http://www.lua.org/manual/5.2/manual.html#2.4)
///
/// # About the Drop trait
///
/// When the Lua context detects that a userdata is no longer needed it calls the function at the
/// `__gc` index in the userdata's metatable, if any. The hlua library will automatically fill this
/// index with a function that invokes the `Drop` trait of the userdata.
///
/// You can replace the function if you wish so, although you are strongly discouraged to do it.
/// It is no unsafe to leak data in Rust, so there is no safety issue in doing so.
///
/// # Arguments
///
///  - `metatable`: Function that fills the metatable of the object.
///
#[inline]
pub fn push_userdata<'lua, L, T, F>(data: T, mut lua: L, metatable: F) -> PushGuard<L>
where
    F: FnOnce(LuaTable<&mut PushGuard<&mut L>>),
    L: AsMutLua<'lua>,
    T: Send + 'static + Any,
{
    unsafe {
        let typeid = TypeId::of::<T>();

        let lua_data = {
            let tot_size = mem::size_of_val(&typeid) + mem::size_of_val(&data);
            ffi::lua_newuserdata(lua.as_mut_lua().0, tot_size as libc::size_t)
        };

        // We check the alignment requirements.
        debug_assert_eq!(lua_data as usize % mem::align_of_val(&data), 0);
        // Since the size of a `TypeId` should always be a usize, this assert should pass every
        // time as well.
        debug_assert_eq!(mem::size_of_val(&typeid) % mem::align_of_val(&data), 0);

        // We write the `TypeId` first, and the data right next to it.
        ptr::write(lua_data as *mut TypeId, typeid);
        let data_loc = (lua_data as *const u8).offset(mem::size_of_val(&typeid) as isize);
        ptr::write(data_loc as *mut _, data);

        let lua_raw = lua.as_mut_lua();

        // Creating a metatable.
        ffi::lua_newtable(lua.as_mut_lua().0);

        // Index "__gc" in the metatable calls the object's destructor.

        // TODO: Could use std::intrinsics::needs_drop to avoid that if not needed.
        // After some discussion on IRC, it would be acceptable to add a reexport in libcore
        // without going through the RFC process.
        {
            match "__gc".push_to_lua(&mut lua) {
                Ok(p) => p.forget(),
                Err(_) => unreachable!(),
            };

            ffi::lua_pushcfunction(lua.as_mut_lua().0, destructor_wrapper::<T>);
            ffi::lua_settable(lua.as_mut_lua().0, -3);
        }

        // Calling the metatable closure.
        {
            let raw_lua = lua.as_lua();
            let mut guard = PushGuard {
                lua: &mut lua,
                size: 1,
                raw_lua: raw_lua,
            };
            metatable(LuaRead::lua_read(&mut guard).ok().unwrap());
            guard.forget();
        }

        ffi::lua_setmetatable(lua_raw.0, -2);
    }

    let raw_lua = lua.as_lua();
    PushGuard {
        lua: lua,
        size: 1,
        raw_lua: raw_lua,
    }
}

///
#[inline]
pub fn read_userdata<'t, 'c, T>(
    lua: &'c mut InsideCallback,
    index: i32,
) -> Result<&'t mut T, &'c mut InsideCallback>
where
    T: 'static + Any,
{
    unsafe {
        let data_ptr = ffi::lua_touserdata(lua.as_lua().0, index);
        if data_ptr.is_null() {
            return Err(lua);
        }

        let actual_typeid = data_ptr as *const TypeId;
        if *actual_typeid != TypeId::of::<T>() {
            return Err(lua);
        }

        let data = (data_ptr as *const u8).offset(mem::size_of::<TypeId>() as isize);
        Ok(&mut *(data as *mut T))
    }
}

/// Represents a user data located inside the Lua context.
#[derive(Debug)]
pub struct UserdataOnStack<T, L> {
    variable: L,
    index: i32,
    marker: PhantomData<T>,
}

impl<'lua, T, L> LuaRead<L> for UserdataOnStack<T, L>
where
    L: AsMutLua<'lua>,
    T: 'lua + Any,
{
    #[inline]
    fn lua_read_at_position(lua: L, index: i32) -> Result<UserdataOnStack<T, L>, L> {
        unsafe {
            let data_ptr = ffi::lua_touserdata(lua.as_lua().0, index);
            if data_ptr.is_null() {
                return Err(lua);
            }

            let actual_typeid = data_ptr as *const TypeId;
            if *actual_typeid != TypeId::of::<T>() {
                return Err(lua);
            }

            Ok(UserdataOnStack {
                variable: lua,
                index: index,
                marker: PhantomData,
            })
        }
    }
}

unsafe impl<'lua, T, L> AsLua<'lua> for UserdataOnStack<T, L>
where
    L: AsLua<'lua>,
    T: 'lua + Any,
{
    #[inline]
    fn as_lua(&self) -> LuaContext {
        self.variable.as_lua()
    }
}

unsafe impl<'lua, T, L> AsMutLua<'lua> for UserdataOnStack<T, L>
where
    L: AsMutLua<'lua>,
    T: 'lua + Any,
{
    #[inline]
    fn as_mut_lua(&mut self) -> LuaContext {
        self.variable.as_mut_lua()
    }
}

impl<'lua, T, L> Deref for UserdataOnStack<T, L>
where
    L: AsLua<'lua>,
    T: 'lua + Any,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe {
            let base = ffi::lua_touserdata(self.variable.as_lua().0, self.index);
            let data = (base as *const u8).offset(mem::size_of::<TypeId>() as isize);
            &*(data as *const T)
        }
    }
}

impl<'lua, T, L> DerefMut for UserdataOnStack<T, L>
where
    L: AsMutLua<'lua>,
    T: 'lua + Any,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            let base = ffi::lua_touserdata(self.variable.as_mut_lua().0, self.index);
            let data = (base as *const u8).offset(mem::size_of::<TypeId>() as isize);
            &mut *(data as *mut T)
        }
    }
}
