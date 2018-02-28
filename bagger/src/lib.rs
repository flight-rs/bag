#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;
extern crate mime;
extern crate mime_guess;

pub mod uri;
pub mod solver;
mod flag;
pub mod nodes;
mod builtins;
//pub mod tyu;

pub use solver::{NodeInput, EdgeBuilder};
pub use flag::{Flag, FlagSet, FlagMap};
pub use nodes::Node;

pub struct BagRequest {
    pub uri: uri::Uri,
    pub target: syn::Type,
    pub required: FlagSet,
    pub args: FlagMap<String>,
}

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
