use ::{Bag, TryBag, Unbag, TryUnbag, fail};
use spin::Mutex;
use std::borrow::Borrow;

enum MapState<A, B, F> {
    Unapplied {
        func: F,
        data: A,
    },
    Temp, // prevent double free in the event of unwind
    Applied {
        data: B,
    },
}

impl<A, B, F: FnOnce(A) -> B> MapState<A, B, F> {
    fn apply(&mut self) {      
        // guarentee no mutation once applied!!
        if let &mut MapState::Applied { .. } = self { return }

        // not yet applied
        use std::mem::replace;
        let applied = match replace(self, MapState::Temp) {
            MapState::Unapplied { func, data } => MapState::Applied { data: func(data) },
            _ => unreachable!(),
        };
        replace(self, applied);
    }

    fn get(mut self) -> B {
        match self {
            MapState::Applied { data } => data,
            MapState::Unapplied { func, data } => func(data),
            _ => unreachable!(),
        }
    }

    fn get_ptr(&mut self) -> *const B {
        self.apply();
        match self {
            &mut MapState::Applied { ref data } => data,
            _ => unreachable!(),
        }
    }
}

pub struct LazyMap<A, B, F: FnOnce(A) -> B> {
    state: Mutex<MapState<A, B, F>>,
}
impl<A, B, F: FnOnce(A) -> B> LazyMap<A, B, F> {
    pub const fn new(data: A, func: F) -> Self {
        LazyMap { state: Mutex::new(MapState::Unapplied { data, func }) }
    }
}
impl<A, B, F: FnOnce(A) -> B, T> Bag<T> for LazyMap<A, B, F> 
    where T: ?Sized, B: Borrow<T> 
{
    fn get(&self) -> &T {
        // Applied state will never be mutated, so we can borrow freely 
        unsafe { &*self.state.lock().get_ptr() }.borrow()
    }
}
impl<A, B, F: FnOnce(A) -> B, T> TryBag<T> for LazyMap<A, B, F> 
    where T: ?Sized, B: Borrow<T> 
{
    fn try_get(&self) -> Result<&T, &fail::Error> {
        Ok(self.get())
    }
}
impl<A, B, F: FnOnce(A) -> B> Unbag<B> for LazyMap<A, B, F> {
    fn unbag(self) -> B { self.state.into_inner().get() }
}
impl<A, B, F: FnOnce(A) -> B> TryUnbag<B> for LazyMap<A, B, F> {
    fn try_unbag(self) -> Result<B, fail::Error> { Ok(self.unbag()) }
}

pub struct TryLazyMap<A, B, F: FnOnce(A) -> Result<B, fail::Error>> {
    state: Mutex<MapState<
        A,
        Result<B, fail::Error>,
        F
    >>,
}
impl<A, B, F: FnOnce(A) -> Result<B, fail::Error>> TryLazyMap<A, B, F> {
    pub const fn new(data: A, func: F) -> Self {
        TryLazyMap { state: Mutex::new(MapState::Unapplied { data, func }) }
    }
}
impl<A, B, F: FnOnce(A) -> Result<B, fail::Error>, T> TryBag<T> for TryLazyMap<A, B, F> 
    where T: ?Sized, B: Borrow<T> 
{
    fn try_get(&self) -> Result<&T, &fail::Error> {
        unsafe { &*self.state.lock().get_ptr() }.as_ref().map(Borrow::borrow)
    }
}
impl<A, B, F: FnOnce(A) -> Result<B, fail::Error>> TryUnbag<B> for TryLazyMap<A, B, F> {
    fn try_unbag(self) -> Result<B, fail::Error> {
        self.state.into_inner().get()
    }
}
