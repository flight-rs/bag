use ::{Bag, TryBag, fail};
use std::path::Path;
use std::io::Read;
use std::fs::File;
use std::borrow::Borrow;
use std::ptr;
use spin::{Once, Mutex};

pub struct StaticBag<T: ?Sized>(pub T);
impl<U: ?Sized, T: ?Sized + Borrow<U>> Bag<U> for StaticBag<T> { 
    fn get(&self) -> &U { self.0.borrow() } 
}
impl<U: ?Sized, T: ?Sized + Borrow<U>> TryBag<U> for StaticBag<T> { 
    fn try_get(&self) -> Result<&U, &fail::Error> {
        Ok(self.get())
    }
}

pub struct StaticTryBag<T>(pub Result<T, fail::Error>);
impl<U: ?Sized, T: Borrow<U>> TryBag<U> for StaticTryBag<T> { 
    fn try_get(&self) -> Result<&U, &fail::Error> {
        self.0.as_ref().map(Borrow::borrow)
    }
}

enum LazyState<T, U> {
    Pre(T),
    Post(U),
}

pub struct LazyMapBag<T, U, F: Fn(T) -> U> {
    state: Mutex<LazyState<(T, F), U>>,
}

impl<T, U, F: Fn(T) -> U> LazyMapBag<T, U, F> {
    pub const fn new(data: T, map: F) -> LazyMapBag<T, U, F> {
        LazyMapBag { 
            state: Mutex::new(LazyState::Pre((data, map))),
        }
    }

    pub fn apply(&self) -> &U {
        // TODO: This really bloody sucks
        // remove all unsafeness you idiot
        
        let mut state = self.state.lock();
        let state = &mut *state;

        use self::LazyState::*;
        unsafe {
            let pre = match state {
                &mut Pre(ref c) => Some(ptr::read(c)),
                _ => None,
            };

            if let Some((data, map)) = pre {
                ptr::write(state, Post(map(data)));
            }
        }
        
        if let &mut Post(ref v) = state { 
            unsafe { &*(v as *const _) }
        } else { unreachable!() }
    }
}

impl<T, V: ?Sized, U: Bag<V>, F: Fn(T) -> U> Bag<V> for LazyMapBag<T, U, F> {
    fn get(&self) -> &V { self.apply().get() }
}

impl<T, V: ?Sized, U: TryBag<V>, F: Fn(T) -> U> TryBag<V> for LazyMapBag<T, U, F> {
    fn try_get(&self) -> Result<&V, &fail::Error> { self.apply().try_get() }
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
