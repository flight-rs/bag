use syn::Type;
use std::ops::{Deref};
use std::hash::{Hash, Hasher};
use std::cmp::{PartialEq, Eq};

#[derive(Eq)]
pub struct Typeu(pub Type);

impl PartialEq for Typeu {
    pub fn eq(&self, other: &Typeu) -> bool {
        types_eq(self, other)
    }
}

pub fn types_eq(a: &Type, b: &Type) -> bool {
    use self::Type::*;
    match (*a, *b) {
        (Slice(ref a), Slice(ref b)) => types_eq(a.elem, b.elem),
        (Array(ref a), Array(ref b)) => types_eq(a.elem, b.elem),
    }
}
