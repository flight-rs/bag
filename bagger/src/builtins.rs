use ::{Bagger, NodeInput, EdgeBuilder, Flag};
use nodes::*;
use expr::*;

use failure::err_msg;
use mime::Mime;

use std::str::FromStr;
use std::io;

fn is_text(mime: &Mime) -> bool {
    use mime::TopLevel;
    match mime {
        &Mime(TopLevel::Text, ..) => true,
        _ => false,
    }
}

fn get_mime(node: &NodeInput<LocalPath>) -> Mime {
    use mime_guess::guess_mime_type;

    if let Some(mime_text) = node.arg("content") {
        if let Ok(m) = Mime::from_str(mime_text) {
            return m
        }
    }
    guess_mime_type(&node.node.0)
}

pub fn register_builtins(bggr: &mut Bagger) {
    let static_flag = Flag::from_str("static");
    let include_flag = Flag::from_str("include");

    // Request -> LocalPath
    bggr.transform(|mut n: NodeInput<Request>| {
        let uri = &n.node.0;
        let path = uri.path.clone();
        let mut edge = EdgeBuilder::new();

        match uri.scheme.as_ref().map(String::as_str) {
            None |
            Some("file") |
            Some("files") => (),
            _ => edge.stop(err_msg("scheme does not reference the file system")),
        }

        n.edges.add(LocalPath(path), edge);
    });

    // LocalPath -> LocalRead
    bggr.transform(|mut n: NodeInput<LocalPath>| {
        use std::fs::File;

        // build edge
        let mut edge = EdgeBuilder::new();

        // read file
        let path = n.node.0.clone();
        edge.value(move |()| {
            Ok(Box::new(File::open(&path)?) as _)
        });

        // append edge
        let mime = get_mime(&n);
        n.edges.add(LocalRead(mime), edge);
    });

    // LocalPath -> Producer<[u8]>, Producer<str>
    // uses include_*!
    bggr.transform(move |mut n: NodeInput<LocalPath>| {
        let flags = &[static_flag, include_flag];
        let span = n.span;

        let bytes_expr_type = ExprType::of(parse_quote!(&'static [u8]));
        let bytes_info = BagInfo::from_quote(parse_quote!(
            Bag<[u8]> + Unbag<&'static [u8]>
        )).unwrap();

        let mut bytes_edge = EdgeBuilder::new();
        bytes_edge.satisfies_flags(flags);

        let str_expr_type = ExprType::of(parse_quote!(&'static str));
        let str_info = BagInfo::from_quote(parse_quote!(
            Bag<str> + Unbag<&'static str>
        )).unwrap();

        let mut str_edge = EdgeBuilder::new();
        if !is_text(&get_mime(&n)) {
            str_edge.stop(err_msg("file content is not text"));
        }
        str_edge.satisfies_flags(flags);

        if let Some(path) = n.node.0.to_str().map(ToOwned::to_owned) {
            let bytes_path = path.clone();
            bytes_edge.value(move |_| Ok(Expr::from_quote(
                quote_spanned! { span => include_bytes!(#bytes_path) },
                bytes_expr_type.clone(),
            ).bag_static()));

            str_edge.value(move |_| Ok(Expr::from_quote(
                quote_spanned! { span => include_str!(#path) },
                str_expr_type.clone(),
            ).bag_static()));
        } else {
            bytes_edge.stop(err_msg("path not utf-8"));
            str_edge.stop(err_msg("path not utf-8"));
        }

        n.edges.add(Producer(bytes_info), bytes_edge);
        n.edges.add(Producer(str_info), str_edge);
    });

    // LocalRead -> Producer<[u8]>, Producer<str>
    bggr.transform(move |mut n: NodeInput<LocalRead>| {
        use syn::LitByteStr;

        let flags = &[static_flag];
        let span = n.span;

        let bytes_expr_type = ExprType::of(parse_quote!(&'static [u8]));
        let bytes_info = BagInfo::from_quote(parse_quote!(
            Bag<[u8]> + Unbag<&'static [u8]>
        )).unwrap();

        // include byte string
        let mut edge = EdgeBuilder::new();
        edge.satisfies_flags(flags);
        edge.value(move |mut read: Box<io::Read>| {
            let mut bytes = Vec::new();
            read.read_to_end(&mut bytes)?;
            Ok(Expr::from_quote(
                LitByteStr::new(&bytes, span),
                bytes_expr_type.clone(),
            ).bag_static())
        });
        n.edges.add(Producer(bytes_info), edge);

        let str_expr_type = ExprType::of(parse_quote!(&'static str));
        let str_info = BagInfo::from_quote(parse_quote!(
            Bag<str> + Unbag<&'static str>
        )).unwrap();

        let mut edge = EdgeBuilder::new();
        edge.satisfies_flags(flags);
        edge.value(move |mut read: Box<io::Read>| {
            let mut string = String::new();
            read.read_to_string(&mut string)?;
            Ok(Expr::from_quote(
                string,
                str_expr_type.clone(),
            ).bag_static())
        });
        if !is_text(&n.node.0) {
            edge.stop(err_msg("read content is not text"));
        }
        n.edges.add(Producer(str_info), edge);
    });

    // Producer<str> -> Producer<String>
    bggr.transform(move |mut n: NodeInput<Producer>| {
        use syn::{Ident, Type};

        let slice_info = BagInfo::from_quote(parse_quote!(Bag<str>)).unwrap();
        let owned_ty: Type = parse_quote!(String);
        let owned_info = BagInfo::simple(owned_ty.clone(), Some(owned_ty.clone()));
        let chars = Ident::from("chars");

        // include byte string
        if n.node.0.satisfies(&slice_info) {
            let mut edge = EdgeBuilder::new();
            edge.value(move |chars_bag: BagExpr| {
                let mut expr = Expr::from_quote(
                    quote! { ::bag::Bag::<str>::get(&#chars).to_owned() },
                    ExprType::of(owned_ty.clone()),
                );
                expr.inputs.push((chars, chars_bag.to_expr()));
                Ok(expr.bag_lazy_map())
            });
            n.edges.add(Producer(owned_info), edge);
        }
    });

     // Producer<[T]> -> Producer<Vec<T>>
    bggr.transform(move |mut n: NodeInput<Producer>| {
        use syn::{Ident, Type};
        use ::expr::BagTrait;

        for (tr, slice_ty) in &n.node.0.impls {
            match tr {
                BagTrait::Simple => (),
                _ => continue,
            }

            let ety = match slice_ty {
                Type::Slice(ref t) => (&*t.elem).clone(),
                _ => continue,
            };

            let vec_ty: Type = parse_quote!(Vec<#ety>);
            let vec_info = BagInfo::simple(vec_ty.clone(), Some(vec_ty.clone()));
            let elems = Ident::from("elems");

            let mut edge = EdgeBuilder::new();
            edge.value(move |elems_bag: BagExpr| {
                let mut expr = Expr::from_quote(
                    quote! { ::bag::Bag::<[_]>::get(&#elems).to_owned() },
                    ExprType::of(vec_ty.clone()),
                );
                expr.inputs.push((elems, elems_bag.to_expr()));
                Ok(expr.bag_lazy_map())
            });
            n.edges.add(Producer(vec_info), edge);
        }
    });
}
