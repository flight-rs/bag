#[macro_export]
macro_rules! bag {
    (
        $(+$r:ident)*
        $(?$f:ident)*
        $(%$a:ident=($e:expr))*
        $uri:expr
        => 
        $($bag:ident<$contains:ty>$(+)*)+
    ) => {{
        #[derive(InitBag)]
        #[bagger(uri=$uri)]
        #[bagger(require($($r),*))]
        #[bagger(forbid($($f),*))]
        $(#[bagger(arg($a=$e))])*
        struct MkBag(
            $(
            #[bagger($bag)]
            *const $contains
            ),+
        );
        
        #[allow(deprecated)]
        <MkBag as ::bag::InitBag>::init()
    }}
}
