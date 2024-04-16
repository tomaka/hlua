extern crate cc;
extern crate pkg_config;

use std::env;

fn main() {
    match pkg_config::find_library("lua5.2") {
        Ok(_) => return,
        Err(..) => {}
    };

    let mut build = cc::Build::new();

    if env::var("CARGO_CFG_TARGET_OS") == Ok("linux".to_string()) {
        // Enable `io.popen` support
        build.define("LUA_USE_LINUX", None);
    }

    build
        .file("lua/src/lapi.c")
        .file("lua/src/lcode.c")
        .file("lua/src/lctype.c")
        .file("lua/src/ldebug.c")
        .file("lua/src/ldo.c")
        .file("lua/src/ldump.c")
        .file("lua/src/lfunc.c")
        .file("lua/src/lgc.c")
        .file("lua/src/llex.c")
        .file("lua/src/lmem.c")
        .file("lua/src/lobject.c")
        .file("lua/src/lopcodes.c")
        .file("lua/src/lparser.c")
        .file("lua/src/lstate.c")
        .file("lua/src/lstring.c")
        .file("lua/src/ltable.c")
        .file("lua/src/ltm.c")
        .file("lua/src/lundump.c")
        .file("lua/src/lvm.c")
        .file("lua/src/lzio.c")
        .file("lua/src/lauxlib.c")
        .file("lua/src/lbaselib.c")
        .file("lua/src/lbitlib.c")
        .file("lua/src/lcorolib.c")
        .file("lua/src/ldblib.c")
        .file("lua/src/liolib.c")
        .file("lua/src/lmathlib.c")
        .file("lua/src/loslib.c")
        .file("lua/src/lstrlib.c")
        .file("lua/src/ltablib.c")
        .file("lua/src/loadlib.c")
        .file("lua/src/linit.c")
        .define("LUA_COMPAT_ALL", None)
        .include("lua/src")
        .compile("liblua.a");
}
