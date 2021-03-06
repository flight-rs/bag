#[macro_use]
extern crate bag_derive;
#[macro_use]
extern crate bag;

use bag::Bag;

fn main() {
    let bag = bag!(+include ?stuff %content=("text/plain") "test.txt" => Bag<str> + TryBag<str> + Unbag<String>);
    println!("{}", Bag::<str>::try_get(&bag).unwrap());
}
