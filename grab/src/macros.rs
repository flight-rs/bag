#[macro_export]
macro_rules! grab {
    (
        $load:expr,
        {
            $(+$r:expr),*
            $(,-$f:expr)*
            $(,$a:ident = $e:expr)*
            $(,)*
        },
        $uri:expr
        =>
        $into:ty
    ) => {{
        use $crate::internal::{Pack, GrabProc, Uid};

        #[derive(GrabProc)]
        #[grabber(uri=$uri)]
        $(#[grabber(require=$r)])*
        $(#[grabber(forbid=$f)])*
        $(#[grabber(arg($a=$e))])*
        struct Grabbed {
            _into: *const $into,
        };

        <Grabbed as GrabProc>::init($load)
    }};
    (
        $load:expr,
        $cat:expr,
        $uri:expr
        =>
        $into:ty
    ) => { grab!($load, { +$cat }, $uri => $into) };
    (
        $load:expr,
        $uri:expr
        =>
        $into:ty
    ) => { grab!($load, {}, $uri => $into) };
}

#[test]
fn test_macro() {
    struct Test;
    impl ::internal::Pack for Test {
        unsafe fn load<T>(&mut self, _index: ::internal::Uid) -> T {
            ::std::mem::zeroed()
        }
    }
    let mut pack = Test;

    grab!(&mut pack, "test/img.png" => Vec<u8>);
    grab!(&mut pack, "bytes", "test/compressed.bzip" => Vec<u8>);
    grab!(&mut pack, { +"text", -"img", size = "120px" }, "test/text.txt" => String);
}
