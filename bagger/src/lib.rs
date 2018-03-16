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
extern crate proc_macro2;
extern crate easy_uri as uri;

pub mod expr;
pub mod solver;
pub mod flag;
pub mod nodes;
mod builtins;

pub use solver::{NodeInput, EdgeBuilder, Solution};
pub use flag::Flag;
pub use nodes::Node;
pub use uri::Uri;
pub use expr::BagInfo;

use flag::{FlagMap, FlagSet};
use proc_macro2::Span;

#[derive(Debug, Clone)]
pub struct BagRequest {
    pub uri: Uri,
    pub target: BagInfo,
    pub required: FlagSet,
    pub forbidden: FlagSet,
    pub args: FlagMap<String>,
    pub span: Span,
}

impl BagRequest {
    pub fn new(uri: Uri, target: BagInfo) -> BagRequest {
        BagRequest {
            uri,
            target,
            required: FlagSet::new(),
            forbidden: FlagSet::new(),
            args: FlagMap::new(),
            span: Span::call_site(),
        }
    }

    pub fn require_flag(&mut self, flag: Flag) {
        self.required.insert(flag);
        self.forbidden.remove(&flag);
    }

    pub fn require(&mut self, flag: &str) {
        self.require_flag(Flag::from_str(flag))
    }

    pub fn forbid_flag(&mut self, flag: Flag) {
        self.forbidden.insert(flag);
        self.required.remove(&flag);
    }

    pub fn forbid(&mut self, flag: &str) {
        self.forbid_flag(Flag::from_str(flag))
    }

    pub fn arg_flag(&mut self, flag: Flag, val: &str) {
        self.args.insert(flag, val.to_owned());
    }

    pub fn arg(&mut self, flag: &str, val: &str) {
        self.arg_flag(Flag::from_str(flag), val)
    }
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

    #[inline(always)]
    pub fn transform<N, F>(&mut self, trans: F)
        where N: Node, F: Fn(NodeInput<N>) + Send + 'static
    {
        let trans = Box::new(solver::FnTransform::new(trans));
        self.solver.transforms.push(trans as Box<solver::TransformInstance>);
    }

    pub fn terminal<T>(&mut self, term: T)
        where T: solver::Terminal + 'static
    {
        self.solver.terminals.push(Box::new(term) as _)
    }

    #[inline(always)]
    pub fn solve(&self, bag: BagRequest) -> Result<Solution, failure::Error> {
        self.solver.solve(bag)
    }
}
