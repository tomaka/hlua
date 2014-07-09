#![feature(plugin_registrar)]

extern crate rustc;
extern crate syntax;

use std::gc::{GC, Gc};
use syntax::parse::token;
use syntax::ast;
use syntax::ext::base;
use syntax::ext::quote::rt::ToSource;
use syntax::codemap;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut ::rustc::plugin::Registry) {
    reg.register_syntax_extension(token::intern("export_lua_module"), base::ItemModifier(expand_lua_module));
}

// handler for export_lua_module
pub fn expand_lua_module(ecx: &mut base::ExtCtxt, span: codemap::Span, meta_item: Gc<ast::MetaItem>, input_item: Gc<ast::Item>)
    -> Gc<ast::Item>
{
    // checking that the input item is a module
    let module = match input_item.node {
        ast::ItemMod(ref module) => module,
        _ => {
            ecx.span_err(input_item.span, "export_lua_module extension is only allowed on modules");
            return input_item
        }
    };

    // creating the new item that will be returned by the function
    // it is just a clone of the input with more elements added to it
    let mut newItem = input_item.deref().clone();
    newItem.vis = ast::Public;
    if input_item.vis != ast::Public {
        ecx.span_warn(input_item.span, "export_lua_module will turn the module into a public module");
    }

    // creating an array of the lines of code to add to the main Lua entry point
    let moduleHandlerBody: Vec<String> = {
        let mut moduleHandlerBody = Vec::new();

        for moditem in module.items.iter() {
            let moditem_name = moditem.ident.to_source();

            match moditem.node {
                ast::ItemFn(..) | ast::ItemStatic(..) => 
                    moduleHandlerBody.push(format!(r#"
                        table.set("{0}".to_string(), {0});
                    "#, moditem_name)),

                _ => {
                    ecx.span_warn(moditem.span, format!("item `{}` is neiter a function nor a static and will thus be ignored by `export_lua_module`", moditem_name).as_slice());
                    continue
                }
            };

            // adding a line to the content 
        }

        moduleHandlerBody
    };

    // adding extern crate declarations
    {
        let generatedCodeContent = format!(r#"
            mod x {{
                extern crate rust_hl_lua;
                extern crate libc;
            }}
        "#);

        // creating a new Rust parser from this
        let mut parser = ::syntax::parse::new_parser_from_source_str(ecx.parse_sess(), ecx.cfg(), "".to_string(), generatedCodeContent);

        // getting all the items defined inside "generateCodeContent"
        match parser.parse_item_with_outer_attributes() {
            None => (),
            Some(m) => {
                let mut m = match m.node {
                    ast::ItemMod(ref m) => m, _ => { ecx.span_err(span, "internal error in the library"); return input_item; }
                };

                let ref mut mutNewItem = match &mut newItem.node {
                    &ast::ItemMod(ref mut m) => m,
                    _ => { ecx.span_err(span, "internal error in the library"); return input_item; }
                };

                for i in m.view_items.iter() { mutNewItem.view_items.unshift(i.clone()) }
            }
        }

        if !parser.eat(&token::EOF) {
            ecx.span_err(input_item.span, "the rust parser failed to compile the module, there is an internal bug in this library");
            return input_item;
        }
    }

    // generating the source code that we will add inside the module
    {
        let generatedCodeContent = format!(r#"
            #[no_mangle]
            pub extern "C" fn luaopen_{0}(lua: *mut self::libc::c_void) -> self::libc::c_int {{
                unsafe {{
                    let mut lua = self::rust_hl_lua::Lua::from_existing_state(lua, false);
                    let mut table = lua.load_new_table();

                    {1}

                    ::std::mem::forget(table);
                    1
               }}
            }}
        "#, input_item.ident.to_source(), moduleHandlerBody.connect("\n"));

        // creating a new Rust parser from this
        let mut parser = ::syntax::parse::new_parser_from_source_str(ecx.parse_sess(), ecx.cfg(), "".to_string(), generatedCodeContent);

        // getting all the items defined inside "generateCodeContent"
        loop {
            match parser.parse_item_with_outer_attributes() {
                None => break,
                Some(i) => match &mut newItem.node {
                    &ast::ItemMod(ref mut m) => m.items.push(i),
                    _ => { ecx.span_err(span, "internal error in the library"); return input_item; }
                }
            }
        }

        if !parser.eat(&token::EOF) {
            ecx.span_err(input_item.span, "the rust parser failed to compile the module, there is an internal bug in this library");
            return input_item;
        }
    }

    // returning
    box(GC) newItem
}
