use ::{Bag, TryBag, Unbag, TryUnbag, fail};
use std::borrow::Borrow;

mod map;
pub use self::map::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Static<U: ?Sized>(pub U);
impl<T: ?Sized, U: ?Sized + Borrow<T>> Bag<T> for Static<U> { 
    fn get(&self) -> &T { self.0.borrow() } 
}
impl<T: ?Sized, U: ?Sized + Borrow<T>> TryBag<T> for Static<U> { 
    fn try_get(&self) -> Result<&T, &fail::Error> {
        Ok(self.get())
    }
}
impl<U> Unbag<U> for Static<U> {
    fn unbag(self) -> U { self.0 }
}
impl<U> TryUnbag<U> for Static<U> {
    fn try_unbag(self) -> Result<U, fail::Error> { Ok(self.0) }
}

#[derive(Debug)]
pub struct TryStatic<U>(pub Result<U, fail::Error>);
impl<U: ?Sized, T: Borrow<U>> TryBag<U> for TryStatic<T> { 
    fn try_get(&self) -> Result<&U, &fail::Error> {
        self.0.as_ref().map(Borrow::borrow)
    }
}
impl<U> TryUnbag<U> for TryStatic<U> {
    fn try_unbag(self) -> Result<U, fail::Error> { self.0 }
}
