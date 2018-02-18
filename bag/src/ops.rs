use ::{Unbag, TryUnbag, fail};
use ::bags::{LazyMap, TryLazyMap};
use std::path::Path;
use std::io::Read;
use std::fs::File;

pub const fn map<A, B, T: Unbag<A>, F: FnOnce(A) -> B>(bag: T, func: F)
        -> LazyMap<(T, F), B, fn((T, F))->B>
{
    LazyMap::new((bag, func), |(bag, func)| func(bag.unbag()))
}

pub const fn try_map<A, B, T: TryUnbag<A>, F: FnOnce(A) -> Result<B, fail::Error>>(bag: T, func: F)
        -> TryLazyMap<(T, F), B, fn((T, F))->Result<B, fail::Error>>
{
    TryLazyMap::new((bag, func), |(bag, func)| func(bag.try_unbag()?))
}

pub trait ReadTarget: Sized {
    fn consume<R: Read>(R) -> Result<Self, fail::Error>;
}

impl ReadTarget for String {
    fn consume<R: Read>(mut r: R) -> Result<Self, fail::Error> {
        let mut buf = String::new();
        r.read_to_string(&mut buf)?;
        Ok(buf)
    }
}

impl ReadTarget for Vec<u8> {
    fn consume<R: Read>(mut r: R) -> Result<Self, fail::Error> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

pub const fn file_contents<P, T>(path: P)
    -> TryLazyMap<P, T, fn(P) -> Result<T, fail::Error>>
    where P: AsRef<Path>, T: ReadTarget
{
    TryLazyMap::new(path, |path| {
        let file = File::open(path)?;
        T::consume(file)
    })
}
