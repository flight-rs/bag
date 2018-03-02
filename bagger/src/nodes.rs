use uri::Uri;
use solver::{Terminal, Working, NodeInstance, Solution};
use expr::{Expr, ExprType, BagExpr, BagType, GenericBagExpr};

use syn;
use mime::Mime;

use std::path::PathBuf;
use std::any::Any;
use std::io;

/// Type that defines a node.
pub trait Node: 'static {
    type Target: 'static;
}

/// The starting node, a basic asset request.
pub struct Request(pub Uri);
impl Node for Request {
    type Target = ();
}

pub struct LocalPath(pub PathBuf);
impl Node for LocalPath {
    type Target = ();
}

pub struct LocalRead(pub Mime);
impl Node for LocalRead {
    type Target = Box<io::Read>;
}

pub struct ReadData(pub Mime);
impl Node for ReadData {
    type Target = Expr;
}

pub struct Data(pub ExprType);
impl Node for Data {
    type Target = Expr;
}

pub struct Producer(pub BagType);
impl Node for Producer {
    type Target = BagExpr;
}

impl Producer {
    pub fn holds(ty: syn::Type) -> Producer {
        Producer(BagType::holds(ty))
    }

    pub fn holds_result(ty: syn::Type) -> Producer {
        Producer(BagType::holds_result(ty))
    }
}

pub struct EndOnProducer;
impl Terminal for EndOnProducer {
    fn terminate(&self, w: &Working, n: &NodeInstance) -> bool {
        match n.downcast_ref::<Producer>() {
            Some(&Producer(ref ty)) => &ty.contains == &w.target,
            _ => false,
        }
    }

    fn extract(&self, _: Working, n: Box<Any>) -> Solution {
        Solution { 
            bag_expr: *n.downcast::<<Producer as Node>::Target>()
                .unwrap(),
        }
    }
}

pub struct GenericProducer;
impl Node for GenericProducer {
    type Target = Box<GenericBagExpr>;
}

pub struct EndOnGenericProducer;
impl Terminal for EndOnGenericProducer {
    fn terminate(&self, _: &Working, n: &NodeInstance) -> bool {
        n.is::<GenericProducer>()
    }

    fn extract(&self, w: Working, n: Box<Any>) -> Solution {
        Solution { 
            bag_expr: n.downcast::<<GenericProducer as Node>::Target>()
                .unwrap()
                .eval_to_bag(&w.target),
        }
    }
}


pub struct Terminate;
impl Node for Terminate {
    type Target = BagExpr;
}

pub struct EndOnTerminate;
impl Terminal for EndOnTerminate {
    fn terminate(&self, _: &Working, n: &NodeInstance) -> bool {
        n.is::<Terminate>()
    }

    fn extract(&self, _: Working, n: Box<Any>) -> Solution {
        Solution { 
            bag_expr: *n.downcast::<<Terminate as Node>::Target>()
                .unwrap(),
        }
    }
}
