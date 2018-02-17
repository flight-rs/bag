extern crate bag;
use bag::{Bag, TryBag, Unbag, TryUnbag};
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

#[test]
fn static_unbag() {
    let bag = StaticBag(5);
    assert_eq!(bag.unbag(), 5);
    assert_eq!(bag.try_unbag().unwrap(), 5);
}

#[test]
fn static_try_unbag() {
    let bag = StaticTryBag(Ok(5));
    assert_eq!(bag.try_unbag().unwrap(), 5);
}
