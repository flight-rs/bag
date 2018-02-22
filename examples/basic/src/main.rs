#[macro_use]
extern crate bag_derive;
extern crate bag;

use bag::{InitTryBag, TryBag};

#[derive(InitTryBag)]
pub struct TestInit {
    #[bagger(path="test.txt")]
    pub test: String,
}

fn main() {
    println!("{}", &TestInit::init().try_get().unwrap().test);
}
