use ::{Bag, TryBag, fail};
use std::path::Path;
use std::io::Read;
use std::fs::File;
use spin::Once;

pub struct RefBag<'a, T: ?Sized + 'a>(pub &'a T);
impl<'a, T: ?Sized + 'a> Bag<T> for RefBag<'a, T> { 
    fn get(&self) -> &T { self.0 } 
}
impl<'a, T: ?Sized + 'a> TryBag<T> for RefBag<'a, T> { 
    fn try_get(&self) -> Result<&T, &fail::Error> {
        Ok(self.get())
    }
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
