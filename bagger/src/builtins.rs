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

        let mut bytes_edge = EdgeBuilder::new();
        let bytes_type = BagType::holds(parse_quote!([u8]));
        bytes_edge.satisfies_flags(flags);

        let mut str_edge = EdgeBuilder::new();
        let str_type = BagType::holds(parse_quote!(str));
        if !is_text(&get_mime(&n)) {
            str_edge.stop(err_msg("file content is not text"));
        }
        str_edge.satisfies_flags(flags);

        if let Some(path) = n.node.0.to_str().map(ToOwned::to_owned) {
            let bytes_path = path.clone();
            let bytes_type = bytes_type.clone();
            bytes_edge.value(move |_| Ok(BagExpr {
                expr: quote_spanned! {
                    span => ::bag::bags::Static(include_bytes!(#bytes_path))
                },
                returns: bytes_type.clone(),
            }));

            let str_type = str_type.clone();
            str_edge.value(move |_| Ok(BagExpr {
                expr: quote_spanned! { 
                    span => ::bag::bags::Static(include_str!(#path))
                },
                returns: str_type.clone(),
            }));
        } else {
            bytes_edge.stop(err_msg("path not utf-8"));
            str_edge.stop(err_msg("path not utf-8"));
        }

        n.edges.add(Producer(bytes_type), bytes_edge);
        n.edges.add(Producer(str_type), str_edge);
    });

    // LocalRead -> Producer<[u8]>, Producer<str>
    bggr.transform(move |mut n: NodeInput<LocalRead>| {
        use syn::LitByteStr;

        let flags = &[static_flag];
        let span = n.span;

        // include byte string
        let mut edge = EdgeBuilder::new();
        edge.satisfies_flags(flags);
        let ty = BagType::holds(parse_quote!([u8]));
        let returns = ty.clone();
        edge.value(move |mut read: Box<io::Read>| {
            let mut bytes = Vec::new();
            read.read_to_end(&mut bytes)?;
            let bstr = LitByteStr::new(&bytes, span);
            Ok(BagExpr {
                expr: quote_spanned! { span => ::bag::bags::Static(#bstr) },
                returns: returns.clone(),
            })
        });
        n.edges.add(Producer(ty), edge);

        let mut edge = EdgeBuilder::new();
        edge.satisfies_flags(flags);
        let ty = BagType::holds(parse_quote!(str));
        let returns = ty.clone();
        edge.value(move |mut read: Box<io::Read>| {
            let mut string = String::new();
            read.read_to_string(&mut string)?;
            Ok(BagExpr {
                expr: quote_spanned! { span => ::bag::bags::Static(#string)},
                returns: returns.clone(),
            })
        });
        if !is_text(&n.node.0) {
            edge.stop(err_msg("read content is not text"));
        }
        n.edges.add(Producer(ty), edge);
    });
}
