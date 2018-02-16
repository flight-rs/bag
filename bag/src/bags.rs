use ::{Bag, TryBag, fail};
use std::path::Path;
use std::io::Read;
use std::fs::File;
use spin::Once;

pub struct ByteBag<'a>(&'a [u8]);
impl<'a> Bag<[u8]> for ByteBag<'a> { 
    fn get(&self) -> &[u8] { self.0 } 
}
impl<'a> TryBag<[u8]> for ByteBag<'a> { 
    fn try_get(&self) -> Result<&[u8], &fail::Error> {
        Ok(self.get())
    }
}

pub struct StrBag<'a>(&'a str);
impl<'a> Bag<str> for StrBag<'a> { 
    fn get(&self) -> &str { self.0 } 
}
impl<'a> TryBag<str> for StrBag<'a> { 
    fn try_get(&self) -> Result<&str, &fail::Error> {
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

pub struct FileContentsBag<P: AsRef<Path>, T: ReadTarget> {
    pub path: P,
    buf: Once<Result<T::Buf, fail::Error>>,
}

impl<P: AsRef<Path>, T: ReadTarget> FileContentsBag<P, T> {
    pub const fn new(path: P) -> Self {
        FileContentsBag { path, buf: Once::new() }
    }
}

impl<P: AsRef<Path>, T: ReadTarget> TryBag<T> for FileContentsBag<P, T> {
    fn try_get(&self) -> Result<&T, &fail::Error> {
        self.buf.call_once(|| T::consume(File::open(&self.path)?))
            .as_ref()
            .map(AsRef::as_ref)
    }
}
