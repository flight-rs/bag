//! # Bag
//!
//! The core of the bag ecosystem is the `bag` crate, which defines the family of
//! `Bag` traits. A Bag is a run-time object which encapsulates the loading and
//! storage of some asset data. `Bag` instances can be declared using the `bag!`
//! macro, which is implemented through the `bag_derive` crate. Build-time reasoning about 
//! `Bag` declarations is available through the `bagger` crate. Bagger understands
//! all non-trivial asset types (i.e. images, JSON documents, scripts, dlls)
//! through plugins (i.e. `bagger_image`, `bagger_json`, `bagger_js`, `bagger_dll`).
//! Most of these plugins do have a runtime dependency that must also be imported (`image`, `serde_json`, `jsbag`, `sharedlib`). 
//! Bagger will warn if any of these are missing.

////////////////////////////////////////////////////////////////////////////////

extern crate spin;
pub extern crate failure as fail;

pub mod bags;
pub mod ops;
pub mod macros;

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

/// Trait for types that can instantiate some Bag. Only used by the `bag!` macro
/// to access `bag_derive`, pending real proc macros.
#[deprecated(since="0.0.0", note="only used by the bag macro")]
pub trait InitBag {
    type Bag;
    fn init() -> Self::Bag;
}
