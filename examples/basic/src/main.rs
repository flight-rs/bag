#[macro_use]
extern crate bag_derive;
#[macro_use]
extern crate bag;

use bag::Unbag;

fn main() {
    let value = bag!("test.txt" => Bag<Vec<u8>>).unbag();
    println!("{:?}", value);
}
