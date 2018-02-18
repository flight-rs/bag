#![feature(const_fn, conservative_impl_trait)]

extern crate spin;
pub extern crate failure as fail;

pub mod bags;
pub mod ops;

use std::ops::Deref;

pub trait Bag<T: ?Sized>: TryBag<T> {
    fn get(&self) -> &T;
}

impl<T: ?Sized> AsRef<T> for Bag<T> {
    fn as_ref(&self) -> &T { self.get() }
}

impl<T: ?Sized> Deref for Bag<T> {
    type Target = T;
    fn deref(&self) -> &T { self.get() }
}

pub trait TryBag<T: ?Sized> {
    fn try_get(&self) -> Result<&T, &fail::Error>;
}

pub trait Unbag<T>: TryUnbag<T> {
    fn unbag(self) -> T;
}

pub trait TryUnbag<T> {
    fn try_unbag(self) -> Result<T, fail::Error>;
}
