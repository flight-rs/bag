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

use proc_macro2::Span;
use proc_macro::TokenStream;
use syn::visit::{self, Visit};

use std::str::FromStr;

fn bagger_attr_meta(attr: &syn::Attribute, only: &syn::Ident) -> Option<syn::Meta> {
    match attr.interpret_meta() {
        Some(meta) => if meta.name() == only { Some(meta) } else { None },
        _ => None,
    }
}

#[proc_macro_derive(InitBag, attributes(bagger))]
pub fn derive_init_try_bag(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse(input).unwrap();  

    fn lit_str(lit: &syn::Lit) -> String {
        match *lit {
            syn::Lit::Str(ref s) => s.value(),
            _ => panic!("literal is not a string"),
        }
    }

    struct Metadata {
        pub uri: Option<Uri>,
        pub target: Option<BagInfo>,
        pub require: FlagSet,
        pub forbid: FlagSet,
        pub args: FlagMap<String>,
    }

    impl<'ast> Visit<'ast> for Metadata {
        fn visit_meta_name_value(&mut self, nv: &syn::MetaNameValue) {
            let name = nv.ident.as_ref().to_owned();
            let lit = &nv.lit;

            match name.as_str() {
                "uri" => self.uri = Some(Uri::from_str(&lit_str(lit))
                    .expect("URI is not valid")),
                "target" => self.target = Some(BagInfo::from_quote(
                    syn::parse_str(&lit_str(lit)).expect("target is not valid")
                ).expect("could not parse target")),
                "require" => { self.require.insert(Flag::new(lit_str(lit))); },
                "forbid" => { self.forbid.insert(Flag::new(lit_str(lit))); },
                n => panic!("unknown bagger input \"{}\"", n),
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

    let bagger_ident = syn::Ident::from("bagger");
    for m in input.attrs.iter().filter_map(|a| bagger_attr_meta(a, &bagger_ident)) {
        visit::visit_meta(&mut meta, &m);
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
