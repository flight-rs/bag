#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
extern crate bagger;

use bagger::{Bagger, BagRequest};
use bagger::uri::Uri;
use bagger::expr::BagType;

use std::str::FromStr;

#[test]
pub fn solve_static_str() {
    let bggr = Bagger::new();
    let mut req = BagRequest::new(
        Uri::from_str("./tests/hello.txt").unwrap(),
        parse_quote!(str));
    req.require("static");
    req.forbid("include");

    let sol = bggr.solve(req).unwrap();
    assert_eq!(
        sol.bag_expr.expr,
        quote! { ::bag::bags::Static::<&'static str>({ "Hello, World!\n" }) },
    );
    assert_eq!(
        sol.bag_expr.returns,
        BagType::holds(
            parse_quote!(::bag::bags::Static<&'static str>),
            parse_quote!(str),
            Some(parse_quote!(&'static str))),
    );
}

#[test]
pub fn solve_include_str() {
    let bggr = Bagger::new();
    let mut req = BagRequest::new(
        Uri::from_str("./tests/hello.txt").unwrap(),
        parse_quote!(str));
    req.require("include");

    let sol = bggr.solve(req).unwrap();
    assert_eq!(
        sol.bag_expr.expr,
        quote! { ::bag::bags::Static::<&'static str>({ include_str!("./tests/hello.txt") }) },
    );
    assert_eq!(
        sol.bag_expr.returns,
        BagType::holds(
            parse_quote!(::bag::bags::Static<&'static str>),
            parse_quote!(str),
            Some(parse_quote!(&'static str))),
    );
}

#[test]
pub fn solve_include_bytes() {
    let bggr = Bagger::new();
    let mut req = BagRequest::new(
        Uri::from_str("./tests/tiny.png").unwrap(),
        parse_quote!([u8]));
    req.require("include");

    let sol = bggr.solve(req).unwrap();
    assert_eq!(
        sol.bag_expr.expr,
        quote! { ::bag::bags::Static::<&'static [u8]>({ include_bytes!("./tests/tiny.png") }) },
    );
    assert_eq!(
        sol.bag_expr.returns,
        BagType::holds(
            parse_quote!(::bag::bags::Static<&'static [u8]>),
            parse_quote!([u8]),
            Some(parse_quote!(&'static [u8]))),
    );
}

#[test]
pub fn solve_static_png_as_str() {
    let bggr = Bagger::new();
    let ty: syn::Type = parse_quote!(str);
    let uri = Uri::from_str("./tests/tiny.png").unwrap();

    let mut req = BagRequest::new(uri.clone(), ty.clone());
    req.require("static");
    req.forbid("include");

    assert!(bggr.solve(req).is_err());

    let mut req = BagRequest::new(uri.clone(), ty.clone());
    req.require("static");
    req.forbid("include");
    req.arg("content", "text/plain");

    assert!(bggr.solve(req).is_err());
}

#[test]
pub fn solve_include_png_as_str() {
    let bggr = Bagger::new();
    let ty: syn::Type = parse_quote!(str);
    let uri = Uri::from_str("./tests/tiny.png").unwrap();

    let mut req = BagRequest::new(uri.clone(), ty.clone());
    req.require("include");

    assert!(bggr.solve(req).is_err());

    let mut req = BagRequest::new(uri.clone(), ty.clone());
    req.require("include");
    req.arg("content", "text/plain");

    assert!(bggr.solve(req).is_ok());
}
