#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

extern crate libc;

use libc::c_int;
use std::mem;
use std::ptr;


/*const LUA_VERSION_MAJOR	"5"
const LUA_VERSION_MINOR	"3"
const LUA_VERSION_NUM		503
const LUA_VERSION_RELEASE	"3"

const LUA_VERSION	"Lua " LUA_VERSION_MAJOR "." LUA_VERSION_MINOR
const LUA_RELEASE	LUA_VERSION "." LUA_VERSION_RELEASE
const LUA_COPYRIGHT	LUA_RELEASE "  Copyright (C) 1994-2016 Lua.org, PUC-Rio"
const LUA_AUTHORS	"R. Ierusalimschy, L. H. de Figueiredo, W. Celes"*/


/* mark for precompiled code ('<esc>Lua') */
pub const LUA_SIGNATURE: &'static str = "\x1bLua";

/* option for multiple returns in 'lua_pcall' and 'lua_call' */
pub const LUA_MULTRET: c_int = -1;


/*
** Pseudo-indices
** (-LUAI_MAXSTACK is the minimum valid index; we keep some free empty
** space after that to help overflow detection)
*/
pub const LUAI_MAXSTACK: c_int = 1000000;  // or 15000 with 32b        // TODO:
pub const LUA_REGISTRYINDEX: c_int = (-LUAI_MAXSTACK - 1000);
#[inline(always)]
pub fn lua_upvalueindex(i: c_int) -> c_int {
    LUA_REGISTRYINDEX - i
}


/* thread status */
pub const LUA_OK: c_int = 0;
pub const LUA_YIELD: c_int = 1;
pub const LUA_ERRRUN: c_int = 2;
pub const LUA_ERRSYNTAX: c_int = 3;
pub const LUA_ERRMEM: c_int = 4;
pub const LUA_ERRGCMM: c_int = 5;
pub const LUA_ERRERR: c_int = 6;

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct lua_State;


/*
** basic types
*/
pub const LUA_TNONE: c_int = -1;

pub const LUA_TNIL: c_int = 0;
pub const LUA_TBOOLEAN: c_int = 1;
pub const LUA_TLIGHTUSERDATA: c_int = 2;
pub const LUA_TNUMBER: c_int = 3;
pub const LUA_TSTRING: c_int = 4;
pub const LUA_TTABLE: c_int = 5;
pub const LUA_TFUNCTION: c_int = 6;
pub const LUA_TUSERDATA: c_int = 7;
pub const LUA_TTHREAD: c_int = 8;

pub const LUA_NUMTAGS: c_int = 9;



/* minimum Lua stack available to a C function */
pub const LUA_MINSTACK: c_int = 20;


/* predefined values in the registry */
pub const LUA_RIDX_MAINTHREAD: lua_Integer = 1;
pub const LUA_RIDX_GLOBALS: lua_Integer = 2;
pub const LUA_RIDX_LAST: lua_Integer = LUA_RIDX_GLOBALS;

// Warning: the type of lua_Number can be configured in the build.
pub type lua_Number = libc::c_double;
pub type lua_Integer = i64;
pub type lua_Unsigned = u64;
pub type lua_KContext = isize;


/*
** Type for C functions registered with Lua
*/
pub type lua_CFunction = extern "C" fn(L: *mut lua_State) -> c_int;

/*
** Type for continuation functions
*/
pub type lua_KFunction = extern "C" fn(L: *mut lua_State, status: c_int, ctx: lua_KContext) -> c_int;


/*
** Type for functions that read/write blocks when loading/dumping Lua chunks
*/
pub type lua_Reader = extern "C" fn(L: *mut lua_State, ud: *mut libc::c_void, sz: *mut libc::size_t) -> *const libc::c_char;
pub type lua_Writer = extern "C" fn(L: *mut lua_State, p: *const libc::c_void, sz: libc::size_t, ud: *mut libc::c_void) -> libc::c_int;


/*
** Type for memory-allocation functions
*/
pub type lua_Alloc = extern "C" fn(ud: *mut libc::c_void, ptr: *mut libc::c_void, osize: libc::size_t, nsize: libc::size_t) -> *mut libc::c_void;

/* Functions to be called by the debugger in specific events */
pub type lua_Hook = extern "C" fn(L: *mut lua_State, ar: *mut lua_Debug);


/*
** RCS ident string
*/
//extern const char lua_ident[];


extern "C" {
    pub fn lua_newstate(f: lua_Alloc, ud: *mut libc::c_void) -> *mut lua_State;
    pub fn lua_close(L: *mut lua_State);
    pub fn lua_newthread(L: *mut lua_State) -> *mut lua_State;
    pub fn lua_atpanic(L: *mut lua_State, panicf: lua_CFunction) -> lua_CFunction;
    pub fn lua_version(L: *mut lua_State) -> *const lua_Number;
    pub fn lua_absindex(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_gettop(L: *mut lua_State) -> c_int;
    pub fn lua_settop(L: *mut lua_State, idx: c_int);
    pub fn lua_pushvalue(L: *mut lua_State, idx: c_int);
    pub fn lua_rotate(L: *mut lua_State, idx: c_int, n: c_int);
    pub fn lua_copy(L: *mut lua_State, fromidx: c_int, toidx: c_int);
    pub fn lua_checkstack(L: *mut lua_State, n: c_int) -> c_int;
    pub fn lua_xmove(from: *mut lua_State, to: *mut lua_State, n: c_int);
    pub fn lua_isnumber(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_isstring(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_iscfunction(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_isinteger(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_isuserdata(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_type(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_typename(L: *mut lua_State, tp: c_int) -> *const libc::c_char;
    pub fn lua_tonumberx(L: *mut lua_State, idx: c_int, isnum: *mut c_int) -> lua_Number;
    pub fn lua_tointegerx(L: *mut lua_State, idx: c_int, isnum: *mut c_int) -> lua_Integer;
    pub fn lua_toboolean(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_tolstring(L: *mut lua_State, idx: c_int, len: *mut libc::size_t) -> *const libc::c_char;
    pub fn lua_rawlen(L: *mut lua_State, idx: c_int) -> libc::size_t;
    pub fn lua_tocfunction(L: *mut lua_State, idx: c_int) -> lua_CFunction;
    pub fn lua_touserdata(L: *mut lua_State, idx: c_int) -> *mut libc::c_void;
    pub fn lua_tothread(L: *mut lua_State, idx: c_int) -> *mut lua_State;
    pub fn lua_topointer(L: *mut lua_State, idx: c_int) -> *const libc::c_void;
    pub fn lua_arith(L: *mut lua_State, op: c_int);
    pub fn lua_rawequal(L: *mut lua_State, idx1: c_int, idx2: c_int) -> c_int;
    pub fn lua_compare(L: *mut lua_State, idx1: c_int, idx2: c_int, op: c_int) -> c_int;
    pub fn lua_pushnil(L: *mut lua_State);
    pub fn lua_pushnumber(L: *mut lua_State, n: lua_Number);
    pub fn lua_pushinteger(L: *mut lua_State, n: lua_Integer);
    pub fn lua_pushlstring(L: *mut lua_State, s: *const libc::c_char, len: libc::size_t) -> *const libc::c_char;
    pub fn lua_pushstring(L: *mut lua_State, s: *const libc::c_char) -> *const libc::c_char;
    //pub fn lua_pushvfstring(L: *mut lua_State, fmt: *const libc::c_char, va_list argp) -> *const libc::c_char;
    //pub fn lua_pushfstring(L: *mut lua_State, fmt: *const libc::c_char, ...) -> *const libc::c_char;
    pub fn lua_pushcclosure(L: *mut lua_State, f: lua_CFunction, n: c_int);
    pub fn lua_pushboolean(L: *mut lua_State, b: c_int);
    pub fn lua_pushlightuserdata(L: *mut lua_State, p: *mut libc::c_void);
    pub fn lua_pushthread(L: *mut lua_State) -> c_int;
    pub fn lua_getglobal(L: *mut lua_State, name: *const libc::c_char) -> c_int;
    pub fn lua_gettable(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_getfield(L: *mut lua_State, idx: c_int, k: *const libc::c_char) -> c_int;
    pub fn lua_geti(L: *mut lua_State, idx: c_int, n: lua_Integer) -> c_int;
    pub fn lua_rawget(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_rawgeti(L: *mut lua_State, idx: c_int, n: lua_Integer) -> c_int;
    pub fn lua_rawgetp(L: *mut lua_State, idx: c_int, p: *const libc::c_void) -> c_int;
    pub fn lua_createtable(L: *mut lua_State, narr: c_int, nrec: c_int);
    pub fn lua_newuserdata(L: *mut lua_State, sz: libc::size_t) -> *mut libc::c_void;
    pub fn lua_getmetatable(L: *mut lua_State, objindex: c_int) -> c_int;
    pub fn lua_getuservalue(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_setglobal(L: *mut lua_State, name: *const libc::c_char);
    pub fn lua_settable(L: *mut lua_State, idx: c_int);
    pub fn lua_setfield(L: *mut lua_State, idx: c_int, k: *const libc::c_char);
    pub fn lua_seti(L: *mut lua_State, idx: c_int, n: lua_Integer);
    pub fn lua_rawset(L: *mut lua_State, idx: c_int);
    pub fn lua_rawseti(L: *mut lua_State, idx: c_int, n: lua_Integer);
    pub fn lua_rawsetp(L: *mut lua_State, idx: c_int, p: *const libc::c_void);
    pub fn lua_setmetatable(L: *mut lua_State, objindex: c_int) -> c_int;
    pub fn lua_setuservalue(L: *mut lua_State, idx: c_int);
    pub fn lua_callk(L: *mut lua_State, nargs: c_int, nresults: c_int, ctx: lua_KContext, k: lua_KFunction);
    pub fn lua_pcallk(L: *mut lua_State, nargs: c_int, nresults: c_int, errfunc: c_int, ctx: lua_KContext, k: lua_KFunction) -> c_int;
    pub fn lua_load(L: *mut lua_State, reader: lua_Reader, dt: *mut libc::c_void, chunkname: *const libc::c_char, mode: *const libc::c_char) -> c_int;
    pub fn lua_dump(L: *mut lua_State, writer: lua_Writer, data: *mut libc::c_void, strip: c_int) -> c_int;
    pub fn lua_yieldk(L: *mut lua_State, nresults: c_int, ctx: lua_KContext, k: lua_KFunction) -> c_int;
    pub fn lua_resume(L: *mut lua_State, from: *mut lua_State, narg: c_int) -> c_int;
    pub fn lua_status(L: *mut lua_State) -> c_int;
    pub fn lua_isyieldable(L: *mut lua_State) -> c_int;
    pub fn lua_gc(L: *mut lua_State, what: c_int, data: c_int) -> c_int;
    pub fn lua_error(L: *mut lua_State) -> c_int;
    pub fn lua_next(L: *mut lua_State, idx: c_int) -> c_int;
    pub fn lua_concat(L: *mut lua_State, n: c_int);
    pub fn lua_len(L: *mut lua_State, idx: c_int);
    pub fn lua_stringtonumber(L: *mut lua_State, s: *const libc::c_char) -> libc::size_t;
    pub fn lua_getallocf(L: *mut lua_State, ud: *mut *mut libc::c_void) -> lua_Alloc;
    pub fn lua_setallocf(L: *mut lua_State, f: lua_Alloc, ud: *mut libc::c_void);
    pub fn lua_getstack(L: *mut lua_State, level: c_int, ar: *mut lua_Debug) -> c_int;
    pub fn lua_getinfo(L: *mut lua_State, what: *const libc::c_char, ar: *mut lua_Debug) -> c_int;
    pub fn lua_getlocal(L: *mut lua_State, ar: *const lua_Debug, n: c_int) -> *const libc::c_char;
    pub fn lua_setlocal(L: *mut lua_State, ar: *const lua_Debug, n: c_int) -> *const libc::c_char;
    pub fn lua_getupvalue(L: *mut lua_State, funcindex: c_int, n: c_int) -> *const libc::c_char;
    pub fn lua_setupvalue(L: *mut lua_State, funcindex: c_int, n: c_int) -> *const libc::c_char;
    pub fn lua_upvalueid(L: *mut lua_State, fidx: c_int, n: c_int) -> *mut libc::c_void;
    pub fn lua_upvaluejoin(L: *mut lua_State, fidx1: c_int, n1: c_int, fidx2: c_int, n2: c_int);
    pub fn lua_sethook(L: *mut lua_State, func: lua_Hook, mask: c_int, count: c_int);
    pub fn lua_gethook(L: *mut lua_State) -> lua_Hook;
    pub fn lua_gethookmask(L: *mut lua_State) -> c_int;
    pub fn lua_gethookcount(L: *mut lua_State) -> c_int;

    pub fn luaL_openlibs(L: *mut lua_State);
}

pub const LUA_OPADD: c_int = 0;
pub const LUA_OPSUB: c_int = 1;
pub const LUA_OPMUL: c_int = 2;
pub const LUA_OPMOD: c_int = 3;
pub const LUA_OPPOW: c_int = 4;
pub const LUA_OPDIV: c_int = 5;
pub const LUA_OPIDIV: c_int = 6;
pub const LUA_OPBAND: c_int = 7;
pub const LUA_OPBOR: c_int = 8;
pub const LUA_OPBXOR: c_int = 9;
pub const LUA_OPSHL: c_int = 10;
pub const LUA_OPSHR: c_int = 11;
pub const LUA_OPUNM: c_int = 12;
pub const LUA_OPBNOT: c_int = 13;
pub const LUA_OPEQ: c_int = 0;
pub const LUA_OPLT: c_int = 1;
pub const LUA_OPLE: c_int = 2;
pub const LUA_GCSTOP: c_int = 0;
pub const LUA_GCRESTART: c_int = 1;
pub const LUA_GCCOLLECT: c_int = 2;
pub const LUA_GCCOUNT: c_int = 3;
pub const LUA_GCCOUNTB: c_int = 4;
pub const LUA_GCSTEP: c_int = 5;
pub const LUA_GCSETPAUSE: c_int = 6;
pub const LUA_GCSETSTEPMUL: c_int = 7;
pub const LUA_GCISRUNNING: c_int = 9;
pub const LUA_HOOKCALL: c_int = 0;
pub const LUA_HOOKRET: c_int = 1;
pub const LUA_HOOKLINE: c_int = 2;
pub const LUA_HOOKCOUNT: c_int = 3;
pub const LUA_HOOKTAILCALL: c_int = 4;
pub const LUA_MASKCALL: c_int = 1 << LUA_HOOKCALL as usize;
pub const LUA_MASKRET: c_int = 1 << LUA_HOOKRET as usize;
pub const LUA_MASKLINE: c_int = 1 << LUA_HOOKLINE as usize;
pub const LUA_MASKCOUNT: c_int = 1 << LUA_HOOKCOUNT as usize;


#[inline(always)]
pub unsafe fn lua_call(L: *mut lua_State, nargs: c_int, nresults: c_int) {
    lua_callk(L, nargs, nresults, 0, mem::zeroed())
}

#[inline(always)]
pub unsafe fn lua_pcall(L: *mut lua_State, nargs: c_int, nresults: c_int, errfunc: c_int) -> c_int {
    lua_pcallk(L, nargs, nresults, errfunc, 0, mem::zeroed())
}

#[inline(always)]
pub unsafe fn lua_yield(L: *mut lua_State, nresults: c_int) -> c_int {
    lua_yieldk(L, nresults, 0, mem::zeroed())
}

//const lua_getextraspace(L)	(()((: *mut libc::c_voidchar *)(L) - LUA_EXTRASPACE))

#[inline(always)]
pub unsafe fn lua_tonumber(L: *mut lua_State, idx: c_int) -> lua_Number {
    lua_tonumberx(L, idx, ptr::null_mut())
}

#[inline(always)]
pub unsafe fn lua_tointeger(L: *mut lua_State, idx: c_int) -> lua_Integer {
    lua_tointegerx(L, idx, ptr::null_mut())
}

#[inline(always)]
pub unsafe fn lua_pop(L: *mut lua_State, n: c_int) {
    lua_settop(L, -n-1)
}

#[inline(always)]
pub unsafe fn lua_newtable(L: *mut lua_State) {
    lua_createtable(L, 0, 0)
}

#[inline(always)]
pub unsafe fn lua_register(L: *mut lua_State, name: *const libc::c_char, f: lua_CFunction) {
    lua_pushcfunction(L, f);
    lua_setglobal(L, name)
}

#[inline(always)]
pub unsafe fn lua_pushcfunction(L: *mut lua_State, f: lua_CFunction) {
    lua_pushcclosure(L, f, 0)
}

#[inline(always)]
pub unsafe fn lua_isfunction(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TFUNCTION
}

#[inline(always)]
pub unsafe fn lua_istable(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TTABLE
}

#[inline(always)]
pub unsafe fn lua_islightuserdata(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TLIGHTUSERDATA
}

#[inline(always)]
pub unsafe fn lua_isnil(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TNIL
}

#[inline(always)]
pub unsafe fn lua_isboolean(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TBOOLEAN
}

#[inline(always)]
pub unsafe fn lua_isthread(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TTHREAD
}

#[inline(always)]
pub unsafe fn lua_isnone(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) == LUA_TNONE
}

#[inline(always)]
pub unsafe fn lua_isnoneornil(L: *mut lua_State, idx: c_int) -> bool {
    lua_type(L, idx) <= 0
}

// const lua_pushliteral(L, s)	lua_pushstring(L, "" s)

#[inline(always)]
pub unsafe fn lua_pushglobaltable(L: *mut lua_State) {
    lua_rawgeti(L, LUA_REGISTRYINDEX, LUA_RIDX_GLOBALS);
}

#[inline(always)]
pub unsafe fn lua_tostring(L: *mut lua_State, i: c_int) -> *const libc::c_char {
    lua_tolstring(L, i, ptr::null_mut())
}

#[inline(always)]
pub unsafe fn lua_insert(L: *mut lua_State, i: c_int) {
    lua_rotate(L, i, 1)
}

#[inline(always)]
pub unsafe fn lua_remove(L: *mut lua_State, i: c_int) {
    lua_rotate(L, i, -1);
    lua_pop(L, 1);
}

#[inline(always)]
pub unsafe fn lua_replace(L: *mut lua_State, i: c_int) {
    lua_copy(L, -1, i);
    lua_pop(L, 1);
}

#[inline(always)]
pub unsafe fn lua_pushunsigned(L: *mut lua_State, i: lua_Unsigned) {
    lua_pushinteger(L, i as lua_Integer)
}

#[inline(always)]
pub unsafe fn lua_tounsignedx(L: *mut lua_State, i: c_int, isnum: *mut c_int) -> lua_Unsigned {
    lua_tointegerx(L, i, isnum) as lua_Unsigned
}

#[inline(always)]
pub unsafe fn lua_tounsigned(L: *mut lua_State, i: c_int) -> lua_Unsigned {
    lua_tounsignedx(L, i, ptr::null_mut())
}

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct lua_Debug {
    pub event: c_int,
    pub name: *const libc::c_char,
    pub namewhat: *const libc::c_char,
    pub what: *const libc::c_char,
    pub source: *const libc::c_char,
    pub currentline: c_int,
    pub linedefined: c_int,
    pub lastlinedefined: c_int,
    pub nups: libc::c_uchar,
    pub nparams: libc::c_uchar,
    pub isvararg: libc::c_char,
    pub istailcall: libc::c_char,
    pub short_src: [libc::c_char ; 60],
    //i_ci: *CallInfo
}
