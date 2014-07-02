use super::ffi;
use super::Lua;
use super::Pushable;
use super::{ ConsumeReadable, CopyReadable, LoadedVariable };
use std::intrinsics::TypeId;

pub struct UserData<T> {
    value: T
}

impl<T: Clone> UserData<T> {
    pub fn new(val: T) -> UserData<T> {
        UserData{value: val}
    }
}

impl<T> Deref<T> for UserData<T> {
    fn deref<'a>(&'a self)
        -> &'a T
    {
        &self.value
    }
}

impl<T> DerefMut<T> for UserData<T> {
    fn deref_mut<'a>(&'a mut self)
        -> &'a mut T
    {
        &mut self.value
    }
}

impl<T: Clone> Clone for UserData<T> {
    fn clone(&self) -> UserData<T> {
        UserData { value: self.value.clone() }
    }
}

fn destructor<T>(_: UserData<T>) {

}

impl<T: Clone + 'static> Pushable for UserData<T> {
    fn push_to_lua(self, lua: &mut Lua) -> uint {
        let dataRaw = unsafe { ffi::lua_newuserdata(lua.lua, ::std::mem::size_of_val(&self.value) as ::libc::size_t) };
        let data: &mut T = unsafe { ::std::mem::transmute(dataRaw) };
        (*data) = self.value.clone();

        // creating a metatable
        let typeid = format!("{}", TypeId::of::<T>());
        unsafe {
            ffi::lua_newtable(lua.lua);

            // index "__typeid" corresponds to the hash of the TypeId of T
            "__typeid".push_to_lua(lua);
            typeid.push_to_lua(lua);
            ffi::lua_settable(lua.lua, -3);

            // index "__gc" call the object's destructor
            "__gc".push_to_lua(lua);
            destructor::<T>.push_to_lua(lua);
            ffi::lua_settable(lua.lua, -3);

            ffi::lua_setmetatable(lua.lua, -2);
        }

        1
    }
}

impl<T: Clone + 'static> CopyReadable for UserData<T> {
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
}

impl<'a, T:Clone> ConsumeReadable<'a> for UserData<T> {
    fn read_from_variable(var: LoadedVariable<'a>) -> Result<UserData<T>, LoadedVariable<'a>> {
        match CopyReadable::read_from_lua(var.lua, -1) {
            None => Err(var),
            Some(a) => Ok(a)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn readwrite() {
        let mut lua = super::super::Lua::new();
        let d = super::UserData::new(2i);

        lua.set("a", d);
        let x: super::UserData<int> = lua.get("a").unwrap();
        assert_eq!(x.value, 2)
    }

    #[test]
    fn type_check() {
        #[deriving(Clone)]
        struct Foo;
        #[deriving(Clone)]
        struct Bar;

        let mut lua = super::super::Lua::new();

        lua.set("a", super::UserData::new(Foo));

        let x: Option<super::UserData<Bar>> = lua.get("a");
        assert!(x.is_none())
    }
}
