#![crate_type = "proc-macro"]

#[macro_use]
extern crate quote;
extern crate syn;
extern crate proc_macro;
extern crate proc_macro2;

extern crate bagger;
use bagger::{Bagger, BagRequest};
use bagger::Uri;
use bagger::flag::{Flag, FlagSet, FlagMap};
use bagger::expr::BagInfo;

use proc_macro2::{TokenStream, TokenNode, Span};
use proc_macro::TokenStream as TokenStream1;

use std::str::FromStr;

#[proc_macro_derive(InitBag, attributes(bagger))]
pub fn derive_init_try_bag(input: TokenStream1) -> TokenStream1 {
    let input: syn::DeriveInput = syn::parse(input).unwrap();

    fn tks_to_uri(tks: TokenStream) -> Uri {
        match syn::parse2::<syn::Lit>(tks).expect("URI not a literal") {
            syn::Lit::Str(s) => Uri::from_str(&s.value()).expect("URI is not valid"),
            _ => panic!("URI is not a string expression"),
        }
    }

    fn tks_to_target(tks: TokenStream) -> BagInfo {
        let syntax_err = "target is not a plus-deliniated list of traits";
        BagInfo::from_quote(syn::parse2::<syn::Type>(tks).
            expect(syntax_err))
            .expect("target is not understood")
            
    }

    fn tks_to_flag(tks: TokenStream) -> Flag {
        Flag::from_str(syn::parse2::<syn::Ident>(tks)
            .expect("flag is not an ident")
            .as_ref())
    }

    fn tks_to_value(tks: TokenStream) -> String {
        match syn::parse2::<syn::Lit>(tks).expect("value not a literal") {
            syn::Lit::Str(s) => s.value(),
            _ => panic!("value is not a string expression"),
        }
    }

    fn is_path_bagger(p: syn::Path) -> bool {
        let mut segs = p.segments.into_iter();
        (match segs.next() {
            Some(s) => s.ident.as_ref() == "bagger",
            None => false,
        }) && segs.next().is_none()
    }

    struct Metadata {
        pub uri: Option<Uri>,
        pub target: Option<BagInfo>,
        pub require: FlagSet,
        pub forbid: FlagSet,
        pub args: FlagMap<String>,
    }

    impl Metadata {
        fn tts(&mut self, tok: TokenStream) {
            let mut ts = tok.into_iter();
            let com = match ts.next().expect("bagger command not given").kind {
                TokenNode::Term(term) => term.as_str().to_owned(),
                _ => panic!("bagger command is not an ident"),
            };
            let mut args = ts.map(|data| match data.kind {
                TokenNode::Group(_, tts) => tts,
                _ => panic!("bagger argument is not a group"),
            });

            let arg1 = args.next().expect("must provide at least one argument");

            match com.as_str() {
                "uri" => self.uri = Some(tks_to_uri(arg1)),
                "target" => self.target = Some(tks_to_target(arg1)),
                "require" => { self.require.insert(tks_to_flag(arg1)); },
                "forbid" => { self.forbid.insert(tks_to_flag(arg1)); },
                "arg" => { self.args.insert(
                    tks_to_flag(arg1),
                    tks_to_value(args.next().expect("arg command takes two params"))
                ); },
                n => panic!("unknown bagger command \"{}\"", n),
            }
        }
    }

    let mut meta = Metadata {
        uri: None,
        target: None,
        require: FlagSet::new(),
        forbid: FlagSet::new(),
        args: FlagMap::new(),
    };

    for attr in input.attrs.into_iter() {
        if is_path_bagger(attr.path) {
            meta.tts(attr.tts);
        }
    }

    let req = BagRequest {
        uri: meta.uri.expect("URI not provided"),
        target: meta.target.expect("target not provided"),
        required: meta.require,
        forbidden: meta.forbid,
        args: meta.args,
        span: Span::call_site(),
    };

    let bggr = Bagger::new();
    let sol = bggr.solve(req).expect("could not bag");

    let ident = input.ident;
    let bag_type = sol.bag_expr.returns;
    let bag_expr = sol.bag_expr.expr;

    let expanded = quote! {
        #[allow(deprecated)]
        impl ::bag::InitBag for #ident {
            type Bag = #bag_type;
            fn init() -> Self::Bag { #bag_expr }
        }
    };

    expanded.into()
}
