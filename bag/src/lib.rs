#![feature(const_fn)]

extern crate spin;
pub extern crate failure as fail;
pub mod bags;

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
