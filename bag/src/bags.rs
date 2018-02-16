use ::{Bag, TryBag, fail};
use std::path::Path;
use std::io::Read;
use std::fs::File;
use spin::Once;

pub struct ByteBag<'a>(&'a [u8]);
impl<'a> Bag<[u8]> for ByteBag<'a> { fn get(&self) -> &[u8] { self.0 } }

pub struct StrBag<'a>(&'a str);
impl<'a> Bag<str> for StrBag<'a> { fn get(&self) -> &str { self.0 } }

pub trait FsTarget {
    type Buf: AsRef<Self>;
    fn consume<R: Read>(R) -> Result<Self::Buf, fail::Error>;
}

pub struct FsBag<P: AsRef<Path>, T: FsTarget> {
    pub path: P,
    buf: Once<Result<T::Buf, fail::Error>>,
}

impl<P, T> TryBag<T> for FsBag<P, T> {
    fn try_get(&self) -> Result<&T, &fail::Error> {
        self.buf.call_once(|| T::consume(File::open(self.path)?))
            .as_ref()
            .map(|v| v.as_ref())
    }
}
