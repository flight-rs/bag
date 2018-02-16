extern crate spin;
pub extern crate failure as fail;
pub mod bags;

pub trait Bag<T: ?Sized>: TryBag<T> {
    fn get(&self) -> &T;
}

pub trait TryBag<T: ?Sized> {
    fn try_get(&self) -> Result<&T, &fail::Error>;
}

impl<B, T> TryBag<T> for B where B: Bag<T> {
    fn try_get(&self) -> Result<&T, &fail::Error> { Ok(Bag::get(self)) }
}
