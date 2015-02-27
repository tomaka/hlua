use ffi;
use libc;

use AsLua;
use AsMutLua;
use LuaContext;
use LuaRead;
use Push;
use PushGuard;

use std::marker::PhantomData;
use std::fmt::Debug;
use std::mem;
use std::ptr;

/// 
pub fn function<F, P>(f: F) -> Function<F, P, <F as FnMut<P>>::Output> where F: FnMut<P> {
    Function {
        function: f,
        marker: PhantomData,
    }
}

pub struct Function<F, P, R> {
    function: F,
    marker: PhantomData<(P, R)>,
}

// this function is the main entry point when Lua wants to call one of our functions
extern fn wrapper1(lua: *mut ffi::lua_State) -> ::libc::c_int {
    // we load the pointer to the wrapper2 function from an upvalue (an upvalue is a value that was pushed alongside our function)
    let wrapper2raw = unsafe { ffi::lua_touserdata(lua, ffi::lua_upvalueindex(2)) };
    let wrapper2: fn(*mut ffi::lua_State)->::libc::c_int = unsafe { ::std::mem::transmute(wrapper2raw) };

    wrapper2(lua)
}

// this function is called when Lua wants to call one of our functions
fn wrapper2<T, P, R>(lua: *mut ffi::lua_State) -> libc::c_int
                     where T: FnMut(P) -> R,
                           P: for<'p> LuaRead<&'p mut InsideCallback>,
                           R: for<'p> Push<&'p mut InsideCallback>
{
    // loading the object that we want to call from the Lua context
    let data_raw = unsafe { ffi::lua_touserdata(lua, ffi::lua_upvalueindex(1)) };
    let data: &mut T = unsafe { mem::transmute(data_raw) };

    // creating a temporary Lua context in order to pass it to push & read functions
    let mut tmp_lua = InsideCallback { lua: LuaContext(lua) };

    // trying to read the arguments
    let arguments_count = unsafe { ffi::lua_gettop(lua) } as i32;
    let args = match LuaRead::lua_read_at_position(&mut tmp_lua, -arguments_count as libc::c_int) {      // TODO: what if the user has the wrong params?
        None => {
            let err_msg = format!("wrong parameter types for callback function");
            err_msg.push_to_lua(&mut tmp_lua);
            unsafe { ffi::lua_error(lua); }
            unreachable!()
        },
        Some(a) => a
    };

    let ret_value = data(args);

    // pushing back the result of the function on the stack
    let nb = {
        let guard = ret_value.push_to_lua(&mut tmp_lua);
        let nb = guard.size;
        unsafe { mem::forget(guard) };
        nb
    };

    nb as libc::c_int
}

// this trait should be implemented on objects that are pushed to be callbacks
trait AnyCallable {
    fn load_args_and_call(&mut self, lua: *mut ffi::lua_State) -> ::libc::c_int;
}

// lua context used inside callbacks
#[doc(hidden)]
pub struct InsideCallback {
    lua: LuaContext,
}

unsafe impl<'a> AsLua for &'a InsideCallback {
    fn as_lua(&self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'a> AsLua for &'a mut InsideCallback {
    fn as_lua(&self) -> LuaContext {
        self.lua
    }
}

unsafe impl<'a> AsMutLua for &'a mut InsideCallback {
    fn as_mut_lua(&mut self) -> LuaContext {
        self.lua
    }
}

impl<L, F, P, R> Push<L> for Function<F, P, R>
        where L: AsMutLua, P: 'static,
              P: for<'p> LuaRead<&'p mut InsideCallback>,
              R: for<'a> Push<&'a mut InsideCallback> + 'static
{
    fn push_to_lua(self, mut lua: L) -> PushGuard<L> {
        // pushing the function pointer as a userdata
        let lua_data_raw = unsafe {
            ffi::lua_newuserdata(lua.as_mut_lua().0, mem::size_of::<F>() as libc::size_t)
        };

        let lua_data: *mut F = unsafe { mem::transmute(lua_data_raw) };
        unsafe { ptr::write(lua_data, self.function) };

        // pushing wrapper2 as a lightuserdata
        let wrapper2: fn(*mut ffi::lua_State) -> libc::c_int = wrapper2::<fn(P) -> R, _, _>;
        unsafe { ffi::lua_pushlightuserdata(lua.as_mut_lua().0, mem::transmute(wrapper2)) };

        // pushing wrapper1 as a closure
        unsafe { ffi::lua_pushcclosure(lua.as_mut_lua().0, mem::transmute(wrapper1), 2) };

        PushGuard { lua: lua, size: 1 }
    }
}   

impl<'a, T, E> Push<&'a mut InsideCallback> for Result<T, E>
                where T: Push<&'a mut InsideCallback> + for<'b> Push<&'b mut &'a mut InsideCallback>, E: Debug
{
    fn push_to_lua(self, mut lua: &'a mut InsideCallback) -> PushGuard<&'a mut InsideCallback> {
        match self {
            Ok(val) => val.push_to_lua(lua),
            Err(val) => {
                let msg = format!("{:?}", val);
                msg.push_to_lua(&mut lua);
                unsafe { ffi::lua_error(lua.as_mut_lua().0); }
                unreachable!()
            }
        }
    }
}
