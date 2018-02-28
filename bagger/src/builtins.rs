use ::{Bagger, NodeInput, EdgeBuilder, Flag};
use nodes::*;

use failure::err_msg;
use syn;
use mime::Mime;

use std::str::FromStr;

fn is_text(mime: &Mime) -> bool {
    use mime::TopLevel;
    match mime {
        &Mime(TopLevel::Text, ..) => true,
        _ => false,
    }
}

pub fn register_builtins(bggr: &mut Bagger) {
    let static_flag = Flag::new("static");
    let include_flag = Flag::new("include");

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

    // LocalPath -> StrData, ByteData
    bggr.transform(|mut n: NodeInput<LocalPath>| {
        use std::fs::File;
        use std::io::prelude::*;
        use mime_guess::guess_mime_type;

        let path = n.node.0.clone();
        
        // get MIME type of file
        let mut mime = guess_mime_type(&path);
        if let Some(mime_text) = n.arg("content") {
            if let Ok(m) = Mime::from_str(mime_text) {
                mime = m
            }
        }
        // build StrData edge
        let mut text_edge = EdgeBuilder::new();
        // read file
        let text_path = path.clone();
        text_edge.value(move |()| {
            let mut string = String::new();
            File::open(&text_path)?.read_to_string(&mut string)?;
            Ok(string)
        });
        // edge does not exist if MIME type is not parseable text
        if !is_text(&mime) { text_edge.stop(err_msg("file type is not text")) }
        n.edges.add(StrData(mime.clone()), text_edge);

        // build ByteData edge
        let mut bytes_edge = EdgeBuilder::new();
        // read file
        bytes_edge.value(move |()| {
            let mut bytes = Vec::new();
            File::open(&path)?.read_to_end(&mut bytes)?;
            Ok(bytes)
        });
        n.edges.add(ByteData(mime), bytes_edge);        
    });

    // LocalPath -> Producer<[u8]>, Producer<str>
    bggr.transform(move |mut n: NodeInput<LocalPath>| {
        let path = n.node.0.clone();
        let flags = &[include_flag];

        let mut bytes_edge = EdgeBuilder::new();
        bytes_edge.satisfies_flags(flags);
        let bytes_ty: syn::Type = parse_quote!([u8]);

        let mut str_edge = EdgeBuilder::new();
        str_edge.satisfies_flags(flags);
        let str_ty: syn::Type = parse_quote!(str);

        if let Some(path) = path.to_str().map(ToOwned::to_owned) {
            let bytes_path = path.clone();
            bytes_edge.value(move |_| Ok(quote! {
                ::bag::bags::Static(include_bytes!(#bytes_path))
            }));
            str_edge.value(move |_| Ok(quote! {
                ::bag::bags::Static(include_str!(#path))
            }));
        } else {
            bytes_edge.stop(err_msg("path not utf-8"));
            str_edge.stop(err_msg("path not utf-8"));
        }

        n.edges.add(Producer(bytes_ty.clone()), bytes_edge);
        n.edges.add(Producer(str_ty.clone()), str_edge);
    });

    // ByteData -> Producer<[u8]>, StrData
    bggr.transform(move |mut n: NodeInput<ByteData>| {
        let flags = &[static_flag];
        let mut edge = EdgeBuilder::new();
        edge.satisfies_flags(flags);
        let ty: syn::Type = parse_quote!([u8]);
        edge.value(move |bytes: Vec<u8>| Ok(quote! {
            &'static [#(#bytes),*]
        }));
        n.edges.add(Producer(ty), edge);

        let mut edge = EdgeBuilder::new();
        edge.value(|bytes: Vec<u8>| Ok(String::from_utf8(bytes)?));
        if !is_text(&n.node.0) {
            edge.stop(err_msg("data type is not text"))
        }
         n.edges.add(StrData(n.node.0.clone()), edge);
    });

    // StrData -> Producer<str>
    bggr.transform(move |mut n: NodeInput<StrData>| {
        let flags = &[static_flag];
        let mut edge = EdgeBuilder::new();
        edge.satisfies_flags(flags);
        let ty: syn::Type = parse_quote!(str);
        edge.value(move |string: String| Ok(quote! { #string }));
        n.edges.add(Producer(ty.clone()), edge);
    });
}
