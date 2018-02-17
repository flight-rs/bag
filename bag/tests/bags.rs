extern crate bag;
use bag::{Bag, TryBag};
use bag::bags::*;

pub static FILE_TEXT: FileContentsBag<&str, str> = FileContentsBag::new("./tests/hello.txt");
pub static STATIC_TEXT: RefBag<str> = RefBag("Hello, world!");

#[test]
fn file_text() {
    assert_eq!(FILE_TEXT.try_get().unwrap(), "Hello, world!");
}

#[test]
fn static_text() {
    assert_eq!(STATIC_TEXT.get(), "Hello, world!");
}
