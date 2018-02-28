#[macro_use]
extern crate quote;
extern crate syn;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;
extern crate mime;
extern crate mime_guess;

pub mod uri;
pub mod solver;
pub mod flag;
pub mod nodes;
mod builtins;
//pub mod tyu;

pub use solver::{NodeInput, EdgeBuilder};
pub use nodes::Node;

pub struct Bagger {
    solver: solver::Solver,
}

impl Bagger {
    pub fn new() -> Bagger {
        let mut bggr = Bagger {
            solver: solver::Solver::new(),
        };
        builtins::register_builtins(&mut bggr);
        bggr
    }

    pub fn transform<N, F>(&mut self, trans: F)
        where N: Node, F: Fn(NodeInput<N>) + Send + 'static
    {
        let trans = Box::new(solver::FnTransform::new(trans));
        self.solver.add_transform(trans as Box<solver::TransformInstance>);
    }
}
