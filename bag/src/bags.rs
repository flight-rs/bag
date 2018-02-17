use ::{Bag, TryBag, Unbag, TryUnbag, fail};
use std::path::Path;
use std::io::Read;
use std::fs::File;
use std::borrow::Borrow;
use spin::{Once};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StaticBag<U: ?Sized>(pub U);
impl<T: ?Sized, U: ?Sized + Borrow<T>> Bag<T> for StaticBag<U> { 
    fn get(&self) -> &T { self.0.borrow() } 
}
impl<T: ?Sized, U: ?Sized + Borrow<T>> TryBag<T> for StaticBag<U> { 
    fn try_get(&self) -> Result<&T, &fail::Error> {
        Ok(self.get())
    }
}
impl<U> Unbag<U> for StaticBag<U> {
    fn unbag(self) -> U { self.0 }
}
impl<U> TryUnbag<U> for StaticBag<U> {
    fn try_unbag(self) -> Result<U, fail::Error> { Ok(self.0) }
}

#[derive(Debug)]
pub struct StaticTryBag<U>(pub Result<U, fail::Error>);
impl<U: ?Sized, T: Borrow<U>> TryBag<U> for StaticTryBag<T> { 
    fn try_get(&self) -> Result<&U, &fail::Error> {
        self.0.as_ref().map(Borrow::borrow)
    }
}
impl<U> TryUnbag<U> for StaticTryBag<U> {
    fn try_unbag(self) -> Result<U, fail::Error> { self.0 }
}

pub trait ReadTarget {
    type Buf: AsRef<Self>;
    fn consume<R: Read>(R) -> Result<Self::Buf, fail::Error>;
}

impl ReadTarget for str {
    type Buf = String;
    fn consume<R: Read>(mut r: R) -> Result<Self::Buf, fail::Error> {
        let mut buf = String::new();
        r.read_to_string(&mut buf)?;
        Ok(buf)
    }
}

impl ReadTarget for [u8] {
    type Buf = Vec<u8>;
    fn consume<R: Read>(mut r: R) -> Result<Self::Buf, fail::Error> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

pub struct FileContentsBag<P: AsRef<Path>, T: ReadTarget + ?Sized> {
    pub path: P,
    buf: Once<Result<T::Buf, fail::Error>>,
}

impl<P: AsRef<Path>, T: ReadTarget + ?Sized> FileContentsBag<P, T> {
    pub const fn new(path: P) -> Self {
        FileContentsBag { path, buf: Once::new() }
    }
}

impl<P: AsRef<Path>, T: ReadTarget + ?Sized> TryBag<T> for FileContentsBag<P, T> {
    fn try_get(&self) -> Result<&T, &fail::Error> {
        self.buf.call_once(|| T::consume(File::open(&self.path)?))
            .as_ref()
            .map(AsRef::as_ref)
    }
}
