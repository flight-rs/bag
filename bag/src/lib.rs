#![feature(const_fn, conservative_impl_trait)]

extern crate spin;
pub extern crate failure as fail;

pub mod bags;
pub mod ops;

use std::ops::Deref;

/// Trait for types that provide access to some data.
pub trait Bag<T: ?Sized>: TryBag<T> {
    /// Get an immutable reference to the stored data.
    fn get(&self) -> &T;
}

impl<T: ?Sized> AsRef<T> for Bag<T> {
    fn as_ref(&self) -> &T { self.get() }
}

impl<T: ?Sized> Deref for Bag<T> {
    type Target = T;
    fn deref(&self) -> &T { self.get() }
}

/// Trait for types that might provide access to some data.
pub trait TryBag<T: ?Sized> {
    /// Get an immutable reference to the stored data, or an error if the data failed to load.
    fn try_get(&self) -> Result<&T, &fail::Error>;
}

/// Trait for types that wrap some data.
pub trait Unbag<T>: TryUnbag<T> {
    /// Unwrap the stored data.
    fn unbag(self) -> T;
}

/// Trait for types that might wrap some data.
pub trait TryUnbag<T> {
    /// Attempt to unwrap the stored data, or an error if the data failed to load.
    fn try_unbag(self) -> Result<T, fail::Error>;
}

/// Trait for types that can be loaded into a `Bag`. Used primarily for
/// `bag_derive` magic.
pub trait InitBag: Sized {
    type Bag: /*Bag<Self> + */Unbag<Self>;
    fn init() -> Self::Bag;
}

/// Trait for types that can be loaded into a `TryBag`. Used primarily for
/// `bag_derive` magic.
pub trait InitTryBag: Sized {
    type Bag: /*TryBag<Self> + */TryUnbag<Self>;
    fn init() -> Self::Bag;
}
