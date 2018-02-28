#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
extern crate bagger;
use bagger::{Bagger, BagRequest, Flag};
use bagger::uri::Uri;
use std::str::FromStr;

#[test]
pub fn solve_static_str() {
    let mut bggr = Bagger::new();
    let mut req = BagRequest::new(
        Uri::from_str("./tests/hello.txt").unwrap(),
        parse_quote!(str));
    req.required.insert(Flag::new("static"));
    println!("{:?}", bggr.solve(req))
}
