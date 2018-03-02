#![crate_type = "proc-macro"]
// The `quote!` macro requires deep recursion.
#![recursion_limit = "256"]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;
extern crate proc_macro2;

extern crate bagger;
#[macro_use]
extern crate lazy_static;

use proc_macro::TokenStream;
use syn::{visit};
use std::collections::HashMap;

#[proc_macro_derive(InitBag, attributes(bagger))]
pub fn derive_init_bag(input: TokenStream) -> TokenStream {
    unimplemented!()
}

fn bagger_attr_meta(attr: &syn::Attribute, only: &syn::Ident) -> Option<syn::Meta> {
    match attr.interpret_meta() {
        Some(meta) => if meta.name() == only { Some(meta) } else { None },
        _ => None,
    }
}

#[proc_macro_derive(InitTryBag, attributes(bagger))]
pub fn derive_init_try_bag(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse(input).unwrap();    
    let bggr = Bagger::new();

    struct FieldData {
        pub ty: syn::Type,
        pub ident: Option<syn::Ident>,
        pub args: Vec<syn::Lit>,
        pub kwargs: HashMap<syn::Ident, syn::Lit>, 
    }

    impl<'ast> visit::Visit<'ast> for FieldData {
        fn visit_meta_name_value(&mut self, nv: &syn::MetaNameValue) {
            self.kwargs.insert(nv.ident, nv.lit.clone());
        }

        fn visit_lit(&mut self, lit: &syn::Lit) {
            self.args.push(lit.clone());
        }
    }

    enum StructVari {
        NAMED,
        UNNAMED,
        UNIT,
    }

    struct InputData {
        pub vari: Option<StructVari>,
        pub fds: Vec<FieldData>,
    }

    impl<'ast> visit::Visit<'ast> for InputData {
        fn visit_data_struct(&mut self, item: &syn::DataStruct) {
            self.vari = Some(match item.fields {
                syn::Fields::Named(_) => StructVari::NAMED,
                syn::Fields::Unnamed(_) => StructVari::UNNAMED,
                syn::Fields::Unit => StructVari::UNIT,
            });
            visit::visit_data_struct(self, item);
        }

        fn visit_field(&mut self, field: &syn::Field) {
            let mut fd = FieldData {
                ty: field.ty.clone(),
                ident: field.ident,
                args: Vec::new(),
                kwargs: HashMap::new(),
            };

            let bggr_ident = syn::Ident::from("bagger");
            for meta in field.attrs.iter()
                .filter_map(|a| bagger_attr_meta(a, &bggr_ident)) 
            {
                visit::visit_meta(&mut fd, &meta);
            }
        }
    }

    let ident = input.ident;
    let tuple_ty = quote! { (::bag::bags::TryStatic<String>,) };
    let expanded = quote! {
        impl ::bag::InitTryBag for #ident {
            type Bag = ::bag::bags::TryLazyMap<#tuple_ty, Self, fn(#tuple_ty) -> Result<Self, ::bag::fail::Error>>;
            fn init() -> Self::Bag {
                use ::bag::TryUnbag;
                ::bag::bags::TryLazyMap::new((
                    ::bag::bags::TryStatic(Ok("hello".to_owned())),
                ), |init| {
                    Ok(#ident {
                        test: init.0.try_unbag()?,
                    })
                })
            }
        }
    };

    expanded.into()
}
