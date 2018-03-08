#![crate_type = "proc-macro"]

//! DO NOT USE THIS DIRECTLY!!!
//!
//! The custom derives available in this crate exist purely to fake out
//! the `bag!` macro until `proc_macro` is stabilized.
//!
//! Again, `#[derive(InitBag)]` is built to untangle a very specific and
//! nonsensical struct definition built by macros in the `bag` crate. Just
//! pretend it doesn't exist.

#[macro_use]
extern crate quote;
extern crate syn;
extern crate proc_macro;
extern crate proc_macro2;

extern crate bagger;
use bagger::{Bagger, BagRequest};
use bagger::Uri;
use bagger::flag::{Flag, FlagSet, FlagMap};
use bagger::expr::{BagInfo, BagTrait};

use proc_macro2::Span;
use proc_macro::TokenStream;
use syn::visit::{self, Visit};

use std::str::FromStr;

#[proc_macro_derive(InitBag, attributes(bagger))]
pub fn derive_init_try_bag(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse(input).unwrap();

    fn filter_meta(attr: &syn::Attribute, only: &str) -> Vec<syn::NestedMeta> {
        match attr.interpret_meta() {
            Some(syn::Meta::List(nv)) => if nv.ident.as_ref() == only {
                return nv.nested.into_iter().collect();
            } else { () },
            _ => ()
        }
        Vec::new()
    }

    fn meta_to_flags(meta: &syn::MetaList, flags: &mut FlagSet) {
        for meta in &meta.nested {
            match meta {
                &syn::NestedMeta::Meta(syn::Meta::Word(ref word)) => {
                    flags.insert(Flag::new(word.to_string()));
                }
                _ => panic!("item is not flag"),
            }
        }
    }

    fn meta_to_map(meta: &syn::MetaList, flags: &mut FlagMap<String>) {
        for meta in &meta.nested {
            match meta {
                &syn::NestedMeta::Meta(syn::Meta::NameValue(ref nv)) => {
                    flags.insert(
                        Flag::new(nv.ident.to_string()),
                        match nv.lit {
                            syn::Lit::Str(ref s) => s.value(),
                            _ => panic!("arg param is not string")
                        }
                    );
                }
                _ => panic!("item is not assoc"),
            }
        }
    }

    struct BagBound {
        pub bag: Option<BagTrait>,
        pub ty: Option<syn::Type>,
    }

    impl<'ast> Visit<'ast> for BagBound {
        fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
            let mut nested = filter_meta(attr, "bagger");

            let err_msg = "field does not correctly define bag trait";
            if nested.len() != 1 {
                panic!(err_msg);
            }

            match nested.pop() {
                Some(syn::NestedMeta::Meta(syn::Meta::Word(ident))) =>
                    self.bag = Some(BagTrait::from_ident(&ident).expect("unknown bag")),
                _ => panic!(err_msg),
            }
        }

        fn visit_type(&mut self, ty: &'ast syn::Type) {
            match *ty {
                syn::Type::Ptr(ref ptr) => self.ty = Some((&*ptr.elem).clone()),
                ref ty => self.ty = Some(ty.clone()),
            }
        }
    }

    struct Metadata {
        pub uri: Option<Uri>,
        pub target: BagInfo,
        pub require: FlagSet,
        pub forbid: FlagSet,
        pub args: FlagMap<String>,
    }

    impl<'ast> Visit<'ast> for Metadata {
        fn visit_meta_name_value(&mut self, nv: &syn::MetaNameValue) {
            let name = nv.ident.as_ref().to_owned(); 

            match name.as_str() {
                "uri" => self.uri = Some(Uri::from_str(&match nv.lit {
                    syn::Lit::Str(ref s) => s.value(),
                    _ => panic!("literal is not a string"),
                }).expect("URI is not valid")),
                k => panic!("unknown bagger key \"{}\"", k),
            }
        }

        fn visit_meta_list(&mut self, ml: &'ast syn::MetaList) {
            let name = ml.ident.as_ref().to_owned();
            
            match name.as_str() {
                "require" => meta_to_flags(ml, &mut self.require),
                "forbid" => meta_to_flags(ml, &mut self.forbid),
                "arg" => meta_to_map(ml, &mut self.args),
                k => panic!("unknown bagger key \"{}\"", k),
            }
        }

        fn visit_field(&mut self, field: &'ast syn::Field) {
            let mut bag = BagBound {
                bag: None,
                ty: None,
            };
            visit::visit_field(&mut bag, field);

            self.target.impls.insert((
                bag.bag.expect("bag trait not given"),
                bag.ty.expect("bag trait not given"),
            ));
        }
    }

    let mut meta = Metadata {
        uri: None,
        target: BagInfo::empty(),
        require: FlagSet::new(),
        forbid: FlagSet::new(),
        args: FlagMap::new(),
    };
    for m in input.attrs.iter().flat_map(|a| filter_meta(a, "bagger")) {
        visit::visit_nested_meta(&mut meta, &m);
    }
    visit::visit_data(&mut meta, &input.data);
    meta.target.simplify();

    let req = BagRequest {
        uri: meta.uri.expect("URI not provided"),
        target: meta.target,
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
