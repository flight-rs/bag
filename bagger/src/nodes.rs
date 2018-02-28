use uri::Uri;
use syn::{Expr, Type};
use mime::Mime;

use std::path::PathBuf;

/// Type that defines a node.
pub trait Node: Send + 'static {
    // waiting for downcast_ref to allow ?Sized
    type Target: /*?Sized +*/ 'static;
    type Meta: 'static;
}

/// The starting node, a basic asset request.
pub struct Request;
impl Node for Request {
    type Target = ();
    type Meta = Uri;
}

pub struct LocalPath;
impl Node for LocalPath {
    type Target = ();
    type Meta = PathBuf;
}

pub struct ByteData;
impl Node for ByteData {
    type Target = Vec<u8>;
    type Meta = Mime;
}

pub struct StrData;
impl Node for StrData {
    type Target = String;
    type Meta = Mime;
}

pub struct Producer;
impl Node for Producer {
    type Target = Expr;
    type Meta = Type;
}

pub struct GenericProducer;
impl Node for GenericProducer {
    type Target = Box<Fn(Type) -> Expr>;
    type Meta = ();
}
