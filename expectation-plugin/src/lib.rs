//! This crate provides the `#[expectation]` attribute. Its use is
//! documented in the `expectation` crate.

#![crate_name = "expectation_plugin"]
#![crate_type = "dylib"]
#![feature(plugin_registrar, rustc_private)]

extern crate rustc_plugin;
extern crate syntax;

use syntax::ast;
use syntax::Span;
use syntax::ast::{Ident, ItemKind, PatKind, Stmt, StmtKind, TyKind};
use syntax::ext::base::{Annotatable, ExtCtxt, MultiModifier};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;
use syntax::symbol::Symbol;

use rustc_plugin::Registry;

/// For the `#[expectation]` attribute. Do not use.
#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        Symbol::intern("expectation"),
        MultiModifier(Box::new(expand_meta_expectation)),
    );
}

fn expand_meta_expectation(
    cx: &mut ExtCtxt,
    span: Span,
    _: &ast::MetaItem,
    annot_item: Annotatable,
) -> Annotatable {
    let item = annot_item.expect_item();
    match item.node {
        ItemKind::Fn(ref decl, header, _, _) => {
            let prop_ident = cx.expr_ident(span, item.ident);
            let prop_ty = cx.ty(
                span,
                TyKind::BareFn(P(ast::BareFnTy {
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
                })),
            );
            let inner_ident = cx.expr_cast(span, prop_ident, prop_ty);
            return wrap_item(cx, span, &*item, inner_ident);
        }
        _ => {
            cx.span_err(span, "#[expectation] only supported on functions");
        }
    }
    Annotatable::Item(item)
}

fn wrap_item(
    cx: &mut ExtCtxt,
    span: Span,
    item: &ast::Item,
    inner_ident: P<ast::Expr>,
) -> Annotatable {
    let new_name_str = "expectation_test_".to_string() + &item.ident.as_str();
    let new_name = ast::Ident::from_str(&new_name_str);

    // Copy original function without attributes
    let prop = P(ast::Item {
        attrs: Vec::new(),
        ..item.clone()
    });
    // ::expectation::expectation
    let crate_name = Ident::from_str("expectation");
    let function_name = Ident::from_str("expect");
    let check_path = vec![crate_name, function_name];
    // Wrap original function in new outer function,
    // calling ::expectation::expectation()
    let fn_decl = Stmt {
        id: ast::DUMMY_NODE_ID,
        node: StmtKind::Item(prop),
        span: span,
    };
    let name_str = cx.expr_str(span, Symbol::intern(&new_name_str));
    let check_call = Stmt {
        id: ast::DUMMY_NODE_ID,
        node: StmtKind::Expr(cx.expr_call_global(span, check_path, vec![name_str, inner_ident])),
        span: span,
    };
    let body = cx.block(span, vec![fn_decl, check_call]);
    let test = cx.item_fn(
        span,
        new_name,
        vec![],
        cx.ty(span, TyKind::Tup(vec![])),
        body,
    );

    // Copy attributes from original function
    let mut attrs = item.attrs.clone();
    // Add #[test] attribute
    attrs.push(cx.attribute(span, cx.meta_word(span, Symbol::intern("test"))));
    // Attach the attributes to the outer function
    Annotatable::Item(P(ast::Item {
        attrs: attrs,
        ..(*test).clone()
    }))
}
