pub trait GrabProc {
    type Output;
    fn init<P: Pack>(pack: &mut P) -> Self::Output;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uid(pub usize, pub usize);

pub trait Pack {
    unsafe fn load<T>(&mut self, index: Uid) -> T;
}
