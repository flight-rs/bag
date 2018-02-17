extern crate bag;
use bag::{Bag, TryBag};
use bag::bags::*;

pub static FILE_TEXT: FileContentsBag<&str, str> = FileContentsBag::new("./tests/hello.txt");
pub static STATIC_TEXT: StaticBag<&str> = StaticBag(HELLO);
pub const HELLO: &str = "Hello, world!";

#[test]
fn file_text() {
    assert_eq!(FILE_TEXT.try_get().unwrap(), HELLO);
}

#[test]
fn static_text() {
    assert_eq!(Bag::<str>::get(&STATIC_TEXT), HELLO);
    assert_eq!(TryBag::<str>::try_get(&STATIC_TEXT).unwrap(), HELLO);
}

use std::str::FromStr;

fn load_int(s: &str) -> StaticTryBag<Box<i32>> {
    StaticTryBag(match i32::from_str(s) {
        Ok(v) => Ok(Box::new(v)),
        Err(e) => Err(e.into()),
    })
}

pub static LAZY_BAG: LazyMapBag<&str, StaticTryBag<Box<i32>>, fn(&str) -> StaticTryBag<Box<i32>>>
    = LazyMapBag::new("5", load_int);

#[test]
fn lazy_map_bag() {
    let v: i32 = *LAZY_BAG.try_get().unwrap();
    assert_eq!(v, 5);
}
