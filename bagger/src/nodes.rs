use uri::Uri;
use solver::{Terminal, Working, NodeInstance, Solution};

use quote::Tokens;
use syn::Type;
use mime::Mime;

use std::path::PathBuf;
use std::any::Any;

/// Type that defines a node.
pub trait Node: 'static {
    // waiting for downcast_ref to allow ?Sized
    type Target: /*?Sized +*/ 'static;
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

pub struct ByteData(pub Mime);
impl Node for ByteData {
    type Target = Vec<u8>;
}

pub struct StrData(pub Mime);
impl Node for StrData {
    type Target = String;
}

pub struct Producer(pub Type);
impl Node for Producer {
    type Target = Tokens;
}

pub struct EndOnProducer;
impl Terminal for EndOnProducer {
    fn terminate(&self, w: &Working, n: &NodeInstance) -> bool {
        match n.downcast_ref::<Producer>() {
            Some(&Producer(ref ty)) => ty == &w.target,
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
    type Target = Box<Fn(Type) -> Tokens>;
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
                (w.target),
        }
    }
}


pub struct Terminate;
impl Node for Terminate {
    type Target = Tokens;
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
