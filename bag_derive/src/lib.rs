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
use syn::DeriveInput;

#[proc_macro_derive(InitBag, attributes(bagger))]
pub fn derive_init_bag(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    unimplemented!()
}

#[proc_macro_derive(InitTryBag, attributes(bagger))]
pub fn derive_init_try_bag(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    let ident = input.ident;
    
    /*if let syn::Data::Struct(strct) = input.data {
        if let syn::Fields::Named(fields) = strct.fields {

        } else { paic!("Fields are unnamed") }
    } else { paic!("Not a struct") }*/


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
