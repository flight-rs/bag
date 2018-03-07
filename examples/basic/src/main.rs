#[macro_use]
extern crate bag_derive;
extern crate bag;

use bag::{InitTryBag, TryBag};

#[derive(InitTryBag)]
pub struct TestInit {
    #[bagger(uri="test.txt")]
    #[bagger(require="include")]
    pub test: str,
}

fn main() {
    let bag = TestInit::init();
    println!("{}", TryBag::<str>::try_get(&bag).unwrap());
}
