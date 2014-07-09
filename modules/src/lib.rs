#![feature(plugin_registrar)]

extern crate libc;
extern crate rustc;
extern crate syntax;

use syntax::parse::token;
use syntax::ast::{ Expr, Item, TokenTree };
use syntax::ext::base::{expr_to_str, get_exprs_from_tts, DummyResult, ExtCtxt, MacResult};
use syntax::codemap::Span;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut ::rustc::plugin::Registry) {
    reg.register_macro("lua_module", macro_handler);
}

// this is the object that we will return from the macro expansion
struct MacroResult {
    content: Vec<::std::gc::Gc<Item>>
}
impl MacResult for MacroResult {
    fn make_def(&self) -> Option<::syntax::ext::base::MacroDef> { None }
    fn make_expr(&self) -> Option<::std::gc::Gc<::syntax::ast::Expr>> { None }
    fn make_pat(&self) -> Option<::std::gc::Gc<::syntax::ast::Pat>> { None }
    fn make_stmt(&self) -> Option<::std::gc::Gc<::syntax::ast::Stmt>> { None }

    fn make_items(&self) -> Option<::syntax::util::small_vector::SmallVector<::std::gc::Gc<Item>>> {
        Some(::syntax::util::small_vector::SmallVector::many(self.content.clone()))
    }
}

// handler for generate_gl_bindings!
pub fn macro_handler(ecx: &mut ExtCtxt, span: Span, token_tree: &[TokenTree]) -> Box<MacResult> {
    // getting the arguments from the macro
    let (moduleName, _) = match parse_macro_arguments(ecx, span.clone(), token_tree) {
        Some(t) => t,
        None => return DummyResult::any(span)
    };

    // generating the source code
    let content = format!(r#"
        mod ffi {{
            pub struct lua_State;
            #[link(name = "lua5.2")]
            extern {{
                pub fn lua_createtable(L: *mut lua_State, narr: ::libc::c_int, nrec: ::libc::c_int);
            }}
        }}

        #[no_mangle]
        pub extern "C" fn luaopen_{}(lua: *mut ffi::lua_State) -> ::libc::c_int {{
            unsafe {{ ffi::lua_createtable(lua, 0, 0); }}
            1
        }}
    "#, moduleName);

    // creating a new Rust parser from this
    let mut parser = ::syntax::parse::new_parser_from_source_str(ecx.parse_sess(), ecx.cfg(), "".to_string(), content);

    // getting all the items defined by the bindings
    let mut items = Vec::new();
    loop {
        match parser.parse_item_with_outer_attributes() {
            None => break,
            Some(i) => items.push(i)
        }
    }
    if !parser.eat(&token::EOF) {
        ecx.span_err(span, "the rust parser failed to compile all the generated bindings (meaning there is a bug in this library!)");
        return DummyResult::any(span)
    }
    box MacroResult { content: items } as Box<MacResult>
}

fn parse_macro_arguments(ecx: &mut ExtCtxt, span: Span, tts: &[TokenTree]) -> Option<(String, Vec<(String, Expr)>)> {
    let values = match get_exprs_from_tts(ecx, span, tts) {
        Some(v) => v,
        None => return None
    };

    if values.len() != 2 {
        ecx.span_err(span, format!("expected 2 arguments but got {}", values.len()).as_slice());
        return None;
    }

    let name = expr_to_str(ecx, values.get(0).clone(), "expected string literal").map(|e| match e { (s, _) => s.get().to_string() }).unwrap();

    Some((name, Vec::new()))
}
