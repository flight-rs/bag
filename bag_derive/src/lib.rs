#![crate_type = "proc-macro"]

#[macro_use]
extern crate quote;
extern crate syn;
extern crate proc_macro;
extern crate proc_macro2;

extern crate bagger;

use proc_macro::TokenStream;
use syn::{visit};
use bagger::{Bagger, BagRequest};
use bagger::uri::Uri;

use std::str::FromStr;

#[proc_macro_derive(InitBag, attributes(bagger))]
pub fn derive_init_bag(_input: TokenStream) -> TokenStream {
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

    struct FieldData {
        pub ty: syn::Type,
        pub ident: Option<syn::Ident>,
        pub uri: Option<Uri>,
        pub args: Vec<syn::Lit>,
        pub kwargs: Vec<(String, syn::Lit)>, 
    }

    impl<'ast> visit::Visit<'ast> for FieldData {
        fn visit_meta_name_value(&mut self, nv: &syn::MetaNameValue) {
            let name = nv.ident.as_ref().to_owned();
            if name.as_str() == "uri" {
                if let syn::Lit::Str(ref uri) = nv.lit {
                    self.uri = Some(Uri::from_str(&uri.value())
                        .expect("URI is not valid"))
                } else {
                    panic!("URI decorator is not string")
                }
            }
            self.kwargs.push((name, nv.lit.clone()));
        }

        fn visit_lit(&mut self, lit: &syn::Lit) {
            self.args.push(lit.clone());
        }
    }

    struct InputData {
        pub fds: Vec<FieldData>,
    }

    impl<'ast> visit::Visit<'ast> for InputData {
        fn visit_field(&mut self, field: &syn::Field) {
            let mut fd = FieldData {
                ty: field.ty.clone(),
                ident: field.ident,
                uri: None,
                args: Vec::new(),
                kwargs: Vec::new(),
            };

            let bggr_ident = syn::Ident::from("bagger");
            for meta in field.attrs.iter()
                .filter_map(|a| bagger_attr_meta(a, &bggr_ident)) 
            {
                visit::visit_meta(&mut fd, &meta);
            }

            self.fds.push(fd);
        }
    }

    let mut indat = InputData {
        fds: Vec::new(),
    };
    visit::visit_derive_input(&mut indat, &input);
    let mut fds = indat.fds.into_iter();
    
    let contains = fds.next().expect("InitBag derives require at least one field");
    if fds.next().is_some() {
        panic!("InitBag derives can use at most one field")
    }

    let uri = contains.uri.expect("missing field URI decorator");
    let mut ty = contains.ty;
    ty = match ty {
        syn::Type::Reference(r) => *r.elem,
        t => t,
    };

    let bggr = Bagger::new();
    let mut req = BagRequest::new(uri, ty);

    for (key, arg) in contains.kwargs.into_iter().filter_map(|(k, a)| match a {
        syn::Lit::Str(v) => Some((k, v.value())),
        _ => None,
    }) {
        match key.as_str() {
            "forbid" =>  req.forbid(&arg),
            "require" => req.require(&arg),
            _ => (),
        }
    }

    let sol = bggr.solve(req).expect("could not bag");

    let ident = input.ident;
    let bag_type = sol.bag_expr.returns.full;
    let inside_type = sol.bag_expr.returns.info.contains;
    let bag_expr = sol.bag_expr.expr;

    let expanded = quote! {
        impl ::bag::InitTryBag for #ident {
            type Inside = #inside_type;
            type Bag = #bag_type;
            fn init() -> Self::Bag { #bag_expr }
        }
    };

    expanded.into()
}
