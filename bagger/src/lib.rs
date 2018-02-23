#[macro_use]
extern crate quote;
extern crate syn;

// prototyping stuff
mod hacking;
pub use hacking::*;

use std::marker::PhantomData;
use std::hash::Hash;
use std::any::TypeId;

#[derive(Hash, PartialEq, Eq)]
pub enum NodeDesc {
    Uri,
    Source(TypeId),
    Jewel(syn::Type),
}

pub trait Node {
    type Data: ?Sized + 'static;

    fn desc(&self) -> NodeDesc;
}

pub struct BagUri;

impl Node for BagUri {
    type Data: str;

    fn desc(&self) -> NodeDesc { NodeDesc::Uri }
}

pub struct Source<T: ?Sized> {
    _data: PhantomData<T>,
}

impl<T: ?Sized> Source<T> {
    pub fn new() -> Source<T> {
        Source { _data: PhantomData }
    }
}

impl<T: 'static + ?Sized> Node for Source<T> {
    type Data = T;

    fn desc(&self) -> NodeDesc {
        NodeDesc::Source(TypeId::of::<T>())
    }
}

pub struct Jewel {
    pub ty: syn::Type,
}

impl Jewel {
    pub fn new(ty: syn::Type) -> Jewel {
        Jewel { ty }
    }

    pub fn ty_str(ty: &str) -> Jewel {
        Jewel { ty: syn::parse_str(ty).unwrap() }
    }

    pub fn ty_tokens(ty: quote::Tokens) -> Jewel {
        Jewel { ty: syn::parse(ty.into()).unwrap() }
    }
}

macro_rules! jewel {
    ($t:ty) => { $crate::Jewel::ty_tokens(quote!{$t}) };
}

pub struct JewelGen {
    pub expr: quote::Tokens,
}

impl Node for Jewel {
    type Data = JewelGen;

    fn desc(&self) -> NodeDesc {
        NodeDesc::Jewel(self.ty.clone())
    }
}

pub struct EdgeOutput<T: ?Sized> {
    pub data: Box<T>,
}

pub fn register<A: Node, B: Node, F>(_: A, _: B, _: F)
    where F: Fn(&mut Bagger, &A::Data) -> Option<EdgeOutput<B::Data>>
{
    unimplemented!()
}

fn prototype_register() {
    use std::io::Read;

    register(Source::<Path>::new(), jewel!(&'static [u8]), |_, data|
        if let Some(path) = data.to_str() {
            Some(JewelGen { expr: quote!{ include_bytes!(#path) }})
        } else { None }
    });

    register(Source::<Path>::new(), jewel!(&'static str), |_, data|
        if let Some(path) = data.to_str() {
            Some(JewelGen { expr: quote!{ include_str!(#path) }})
        } else { None }
    });
}
