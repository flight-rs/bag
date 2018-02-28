use uri::Uri;

use quote::Tokens;
use syn::Type;
use mime::Mime;

use std::path::PathBuf;

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

pub struct GenericProducer;
impl Node for GenericProducer {
    type Target = Box<Fn(Type) -> Tokens>;
}
