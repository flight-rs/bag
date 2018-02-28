#[macro_use]
extern crate quote;
extern crate syn;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;

pub mod uri;
pub mod solver;
//pub mod tyu;

pub struct Bagger {
    
}

impl Bagger {
    pub fn new() -> Bagger {
        Bagger { }
    }
}
