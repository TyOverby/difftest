//! This crate provides the `#[difftest]` attribute. Its use is
//! documented in the `difftest` crate.

#![crate_name = "difftest_macros"]
#![crate_type = "dylib"]
#![doc(html_root_url = "http://burntsushi.net/rustdoc/difftest")]

#![feature(plugin_registrar, rustc_private)]

extern crate syntax;
extern crate rustc_plugin;

use syntax::ast;
use syntax::ast::{Ident, ItemKind, PatKind, StmtKind, Stmt, TyKind};
use syntax::codemap;
use syntax::ext::base::{ExtCtxt, MultiModifier, Annotatable};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;
use syntax::symbol::Symbol;

use rustc_plugin::Registry;

/// For the `#[difftest]` attribute. Do not use.
#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(Symbol::intern("difftest"),
                                  MultiModifier(Box::new(expand_meta_difftest)));
}

/// Expands the `#[difftest]` attribute.
///
/// Expands:
/// ```
/// #[difftest]
/// fn check_something(_: usize) -> bool {
///     true
/// }
/// ```
/// to:
/// ```
/// #[test]
/// fn __difftest_check_something() {
///     fn check_something(_: usize) -> bool {
///         true
///     }
///     ::difftest::difftest(check_something as fn(usize) -> bool)
/// }
/// ```
fn expand_meta_difftest(cx: &mut ExtCtxt,
                          span: codemap::Span,
                          _: &ast::MetaItem,
                          annot_item: Annotatable) -> Annotatable {
    let item = annot_item.expect_item();
    match item.node {
        ItemKind::Fn(ref decl, header, _, _) => {
            let prop_ident = cx.expr_ident(span, item.ident);
            let prop_ty = cx.ty(span, TyKind::BareFn(P(ast::BareFnTy {
                unsafety: header.unsafety,
                abi: header.abi,
                generic_params: vec![],
                decl: decl.clone().map(|mut decl| {
                    for arg in decl.inputs.iter_mut() {
                        arg.pat = arg.pat.clone().map(|mut pat| {
                            pat.node = PatKind::Wild;
                            pat
                        });
                    }
                    decl
                }),
            })));
            let inner_ident = cx.expr_cast(span, prop_ident, prop_ty);
            return wrap_item(cx, span, &*item, inner_ident);
        },
        _ => {
            cx.span_err(
                span, "#[difftest] only supported on statics and functions");
        }
    }
    Annotatable::Item(item)
}

fn wrap_item(cx: &mut ExtCtxt,
             span: codemap::Span,
             item: &ast::Item,
             inner_ident: P<ast::Expr>) -> Annotatable {
    // Copy original function without attributes
    let prop = P(ast::Item {attrs: Vec::new(), ..item.clone()});
    // ::difftest::difftest
    let check_ident = Ident::from_str("difftest");
    let check_path = vec!(check_ident, check_ident);
    // Wrap original function in new outer function,
    // calling ::difftest::difftest()
    let fn_decl = Stmt {
        id: ast::DUMMY_NODE_ID,
        node: StmtKind::Item(prop),
        span: span,
    };
    let check_call = Stmt {
        id: ast::DUMMY_NODE_ID,
        node: StmtKind::Expr(cx.expr_call_global(span, check_path, vec![inner_ident])),
        span: span,
    };
    let body = cx.block(span, vec![fn_decl, check_call]);
    let test = cx.item_fn(span, item.ident, vec![], cx.ty(span, TyKind::Tup(vec![])), body);

    // Copy attributes from original function
    let mut attrs = item.attrs.clone();
    // Add #[test] attribute
    attrs.push(cx.attribute(
        span, cx.meta_word(span, Symbol::intern("test"))));
    // Attach the attributes to the outer function
    Annotatable::Item(P(ast::Item {attrs: attrs, ..(*test).clone()}))
}
