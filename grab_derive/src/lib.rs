#![crate_type = "proc-macro"]

//! DO NOT USE THIS DIRECTLY!!!

#[macro_use]
extern crate quote;
extern crate syn;
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::visit::{self, Visit};

#[proc_macro_derive(GrabProc, attributes(grabber))]
pub fn derive_grab_proc(input: TokenStream) -> TokenStream {
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

    struct FindType {
        pub ty: Option<syn::Type>,
    }

    impl<'ast> Visit<'ast> for FindType {
         fn visit_type(&mut self, ty: &'ast syn::Type) {
            match *ty {
                syn::Type::Ptr(ref ptr) => self.ty = Some((&*ptr.elem).clone()),
                ref ty => self.ty = Some(ty.clone()),
            }
        }
    }

    struct Metadata {
        pub uri: Option<String>,
        pub target: Option<syn::Type>,
    }

    impl<'ast> Visit<'ast> for Metadata {
        fn visit_meta_name_value(&mut self, nv: &syn::MetaNameValue) {
            let name = nv.ident.as_ref().to_owned();

            match name.as_str() {
                "uri" => self.uri = match nv.lit {
                    syn::Lit::Str(ref s) => Some(s.value()),
                    _ => None,
                },
                _ => (),
            }
        }

        fn visit_meta_list(&mut self, _ml: &'ast syn::MetaList) {
            // let name = ml.ident.as_ref().to_owned();
            // match name.as_str() {
            //     "require" => meta_to_flags(ml, &mut self.require),
            //     "forbid" => meta_to_flags(ml, &mut self.forbid),
            //     "arg" => meta_to_map(ml, &mut self.args),
            //     k => panic!("unknown grabber key \"{}\"", k),
            // }
        }

        fn visit_field(&mut self, field: &'ast syn::Field) {
            if field.ident.as_ref().map(AsRef::as_ref) == Some("_into") {
                let mut ft = FindType {
                    ty: None,
                };
                visit::visit_field(&mut ft, field);
                self.target = ft.ty
            }
        }
    }

    let mut meta = Metadata {
        uri: None,
        target: None,
    };
    for m in input.attrs.iter().flat_map(|a| filter_meta(a, "grabber")) {
        visit::visit_nested_meta(&mut meta, &m);
    }
    visit::visit_data(&mut meta, &input.data);

    let ident = input.ident;
    let target = meta.target.expect("no target type given");

    let expanded = quote! {
        impl GrabProc for #ident {
            type Output = #target;
            fn init<P: Pack>(pack: &mut P) -> Self::Output { unsafe {
                pack.load::<Self::Output>(Uid(0, 0))
            } }
        }
    };

    expanded.into()
}
