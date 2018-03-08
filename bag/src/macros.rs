#[macro_export]
macro_rules! bag {
    ($(+$r:ident)* $(?$f:ident)* $(%$a:ident=($e:expr))* $name:expr => $traits:ty) => {
        bag_internal!(
            [target $traits]
            [uri $name]
            $([require $r])*
            $([forbid $f])*
        )
    }
}

#[macro_export]
macro_rules! bag_internal {
    ($([$($v:tt)+])*) => {{
        #[derive(InitBag)]
        $(#[bagger($($v)*)])*
        struct MkBag;
        
        #[allow(deprecated)]
        <MkBag as ::bag::InitBag>::init()
    }}
}
