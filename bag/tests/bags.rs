extern crate bag;
use bag::{Bag, TryBag, Unbag, TryUnbag};
use bag::bags::*;
use bag::ops::*;

#[macro_use]
extern crate failure;
use failure::Error;

pub static FILE_TEXT: TryLazyMap<&str, String, fn(&'static str) -> Result<String, Error>> = file_contents("./tests/hello.txt");
pub static STATIC_TEXT: Static<&str> = Static(HELLO);
pub const HELLO: &str = "Hello, world!";

#[test]
fn static_file_text() {
    assert_eq!(TryBag::<str>::try_get(&FILE_TEXT).unwrap(), HELLO);
}

#[test]
fn static_text() {
    assert_eq!(Bag::<str>::get(&STATIC_TEXT), HELLO);
    assert_eq!(TryBag::<str>::try_get(&STATIC_TEXT).unwrap(), HELLO);
}

#[test]
fn static_unbag() {
    let bag = Static(5);
    assert_eq!(bag.unbag(), 5);
    assert_eq!(bag.try_unbag().unwrap(), 5);
}

#[test]
fn try_unbag() {
    let bag = TryStatic(Ok(5));
    assert_eq!(bag.try_unbag().unwrap(), 5);
}

#[test]
fn lazy_map() {
    let bag = map(Static(99), |x| format!("{} bottles", x));
    assert_eq!(Bag::<str>::get(&bag), "99 bottles");
}


#[test]
fn try_lazy_map() {
    use std::str::FromStr;

    let bag = try_map(Static("hello"), |s| Ok(u32::from_str(s)?));
    assert!(TryBag::<u32>::try_get(&bag).is_err());

    let bag = try_map(TryStatic(Err(format_err!("Boom!"))), |s: u32| Ok(s));
    assert!(TryBag::<u32>::try_get(&bag).is_err());

    let bag = try_map(Static("42"), |s| Ok(u32::from_str(s)?));
    assert_eq!(*TryBag::<u32>::try_get(&bag).unwrap(), 42);
}

#[test]
fn lazy_map_is_lazy() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let atom = Arc::new(AtomicBool::new(false));
    let bag_atom = atom.clone();
    let bag = map(Static(()), move |()| {
        bag_atom.swap(true, Ordering::SeqCst);
    });

    assert!(!atom.load(Ordering::SeqCst));
    Bag::<()>::get(&bag);
    assert!(atom.load(Ordering::SeqCst));
}
