#![crate_name = "rust-hl-lua-modules"]
#![feature(plugin_registrar)]
#![feature(quote)]

extern crate rustc;
extern crate syntax;

use std::gc::{GC, Gc};
use syntax::parse::token;
use syntax::ast;
use syntax::attr::AttrMetaMethods;
use syntax::ext::build::AstBuilder;
use syntax::ext::base;
use syntax::ext::quote::rt::ToSource;
use syntax::codemap;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut ::rustc::plugin::Registry) {
    reg.register_syntax_extension(token::intern("export_lua_module"),
        base::ItemModifier(expand_lua_module));
}

// handler for export_lua_module
pub fn expand_lua_module(ecx: &mut base::ExtCtxt, span: codemap::Span,
                        _: Gc<ast::MetaItem>, input_item: Gc<ast::Item>)
    -> Gc<ast::Item>
{
    let ecx: &base::ExtCtxt = &*ecx;

    // checking that the input item is a module
    let module = match input_item.node {
        ast::ItemMod(ref module) => module,
        _ => {
            ecx.span_err(input_item.span,
                "`export_lua_module` extension is only allowed on modules");
            return input_item
        }
    };

    // creating the new item that will be returned by the function
    // it is just a clone of the input with more elements added to it
    let mut new_item = input_item.deref().clone();
    new_item.vis = ast::Public;
    if input_item.vis != ast::Public {
        ecx.span_warn(input_item.span,
            "`export_lua_module` will turn the module into a public module");
    }

    // creating an array of the statements to add to the main Lua entry point
    let module_handler_body: Vec<Gc<ast::Stmt>> = {
        let mut module_handler_body = Vec::new();

        // iterating over elements inside the module
        for moditem in module.items.iter() {
            let moditem_name = moditem.ident.to_source();

            match moditem.node {
                ast::ItemFn(..) | ast::ItemStatic(..) => {
                    let moditem_name_slice = moditem_name.as_slice();
                    let moditem_name_item = ecx.ident_of(moditem_name_slice);
                    module_handler_body.push(
                        quote_stmt!(&*ecx,
                            table.set($moditem_name_slice.to_string(), $moditem_name_item);
                        )
                    )
                },

                _ => {
                    ecx.span_warn(moditem.span,
                        format!("item `{}` is neiter a function nor a static
                            and will thus be ignored by `export_lua_module`",
                            moditem_name).as_slice());
                    continue
                }
            };

            // handling lua_module_init
            if moditem.attrs.iter().find(|at| at.check_name("lua_module_init")).is_some() {
                let moditem_name_item = ecx.ident_of(moditem.ident.to_source().as_slice());
                module_handler_body.push(
                    quote_stmt!(ecx, $moditem_name_item())
                );
            }
        }

        module_handler_body
    };

    // adding extern crate declarations
    {
        let view_items = quote_item!(ecx, 
            mod x {
                extern crate lua = "rust-hl-lua";
                extern crate libc;
            }
        );

        let view_items = match view_items {
            Some(a) => a,
            None => {
                ecx.span_err(input_item.span,
                    "internal error in the library (could not parse view items)");
                return input_item;
            }
        };

        // getting all the items
        let view_items = match view_items.node {
            ast::ItemMod(ref m) => m,
            _ => { ecx.span_err(span, "internal error in the library"); return input_item; }
        };

        let ref mut mut_new_item = match &mut new_item.node {
            &ast::ItemMod(ref mut m) => m,
            _ => { ecx.span_err(span, "internal error in the library"); return input_item; }
        };

        for i in view_items.view_items.iter() {
            mut_new_item.view_items.insert(0, i.clone())
        }
    }

    // generating the function that we will add inside the module
    {
        let function_body = {
            let mut function_body = Vec::new();

            function_body.push(quote_stmt!(ecx,
                let mut lua = self::lua::Lua::from_existing_state(lua, false);
            ));

            function_body.push(quote_stmt!(ecx,
                let mut table = lua.load_new_table();
            ));

            function_body.push_all_move(module_handler_body);

            function_body.push(quote_stmt!(ecx,
                ::std::mem::forget(table);
            ));

            ecx.block(span.clone(), function_body, Some(
                quote_expr!(ecx, 1)
            ))
        };

        // identifier for "luaopen_mylib"
        let luaopen_id = ecx.ident_of(format!("luaopen_{}", input_item.ident.to_source())
            .as_slice());

        // building the function itself
        let function = quote_item!(ecx,
            #[no_mangle]
            pub extern "C" fn $luaopen_id(lua: *mut self::libc::c_void)
                                                    -> self::libc::c_int {
                unsafe {
                    $function_body
               }
            }
        );

        let function = match function {
            Some(f) => f,
            None => {
                ecx.span_err(input_item.span,
                    "internal error in the library (could not parse function body)");
                return input_item;
            }
        };

        // adding the function to the module
        match &mut new_item.node {
            &ast::ItemMod(ref mut m) => {
                m.items.push(function)
            },
            _ => { ecx.span_err(span, "internal error in the library"); return input_item; }
        };
    }

    // returning
    box(GC) new_item
}
