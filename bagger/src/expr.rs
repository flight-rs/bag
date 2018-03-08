use syn::{self, Ident, Type};
use syn::punctuated::Punctuated;
use syn::token;
use quote::{Tokens, ToTokens};
use std::collections::HashSet;

use failure::{Error, err_msg};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BagInfo {
    pub impls: HashSet<(BagTrait, Type)>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum BagTrait {
    Simple,
    Try,
    Unbag,
    TryUnbag,
    Async,
}

impl BagTrait {
    /// Converts this bag trait to the equivalent failable bag. For example, 
    /// `Bag` is converted into `TryBag`.
    pub fn failable(&mut self) {
        use self::BagTrait::*;

        *self = match *self {
            Simple => Try,
            Try => Try,
            Unbag => TryUnbag,
            TryUnbag => TryUnbag,
            Async => Async,
        };
    }

    pub fn from_ident(i: &syn::Ident) -> Result<BagTrait, Error> {
        use self::BagTrait::*;

        let tr_name = i.as_ref();
        Ok(match tr_name {
            "Bag" => Simple,
            "TryBag" => Try,
            "Unbag" => Unbag,
            "TryUnbag" => TryUnbag,
            "AsyncBag" => Async,
            _ => bail!("trait \"{}\" is not a bag", tr_name),
        })
    }

    pub fn from_quote(q: syn::Path) -> Result<(BagTrait, Type), Error> {
        // TODO: check rest of path
        let seg = q.segments.into_iter()
            .last()
            .ok_or(err_msg("bag trait is missing"))?;
        let tr = BagTrait::from_ident(&seg.ident)?;
        let tr_name = seg.ident.as_ref();

        let no_params_err = format_err!("\"{}\" must take have a type parameter", tr_name);
        let ty = if let syn::PathArguments::AngleBracketed(args) = seg.arguments {
            if args.args.len() > 1 { bail!("\"{}\" has too many type parameters", tr_name) };
            let arg = args.args.into_iter().next()
                .ok_or(no_params_err)?;
            match arg {
                syn::GenericArgument::Type(ty) => ty,
                arg =>  bail!("parameter \"{:?}\" on \"{}\" is not a type", arg, tr_name)
            }
        } else { Err(no_params_err)? };

        Ok((tr, ty))
    }
}

impl BagInfo {
    /// Create a target with no bounds
    pub fn empty() -> BagInfo {
        BagInfo { impls: HashSet::new() }
    }

    /// Create a simple bag target of  the form `Bag<A> + Unbag<B>`.
    pub fn simple(bag: Type, unbag: Option<Type>) -> BagInfo {
        let mut info = BagInfo::empty();
        info.impls.insert((BagTrait::Simple, bag));
        if let Some(unbag) = unbag {
            info.impls.insert((BagTrait::Unbag, unbag));
        }
        info
    }

    /// Create a simple bag target of  the form `TryBag<A> + TryUnbag<B>`.
    pub fn simple_try(bag: Type, unbag: Option<Type>) -> BagInfo {
        let mut info = BagInfo::empty();
        info.impls.insert((BagTrait::Try, bag));
        if let Some(unbag) = unbag {
            info.impls.insert((BagTrait::TryUnbag, unbag));
        }
        info
    }

    fn add_quote_bounds(
        &mut self,
        q: Punctuated<syn::TypeParamBound, token::Add>,
    ) 
        -> Result<(), Error> 
    {
        for b in q {
            match b {
                syn::TypeParamBound::Trait(tb) => {
                    self.impls.insert(BagTrait::from_quote(tb.path)?);
                },
                _ => (),
            }
        }
        Ok(())
    }

    /// Parse the bag target from rust syntax such as
    /// `Bag<str> + Unbag<String> + Unbag<&'static str>`
    pub fn from_quote(q: Type) -> Result<BagInfo, Error> {
        let mut info = BagInfo::empty();
        match q {
            Type::Path(p) => {
                info.impls.insert(BagTrait::from_quote(p.path)?);
            }
            Type::TraitObject(b) => info.add_quote_bounds(b.bounds)?,
            Type::ImplTrait(b) => info.add_quote_bounds(b.bounds)?,
            _ => bail!("target is not a plus-deliniated list of traits"),
        }
        info.simplify();
        Ok(info)
    }

    /// Remove redundant trait bounds. For example, the existence of `Bag<T>`
    /// implicitly guarantees the existence of `TryBag<T>`.
    pub fn simplify(&mut self) {
        let mut simp = HashSet::with_capacity(self.impls.len());
        for (b, t) in self.impls.drain() {
            use self::BagTrait::*;
            let mut x = (Simple, t);
            match b {
                Simple => {
                    x.0 = Try;
                    simp.remove(&x);
                    x.0 = Simple;
                    simp.insert(x);
                },
                Try => {
                    x.0 = Simple;
                    if !simp.contains(&x) {
                        x.0 = Try;
                        simp.insert(x);
                    }
                },
                Unbag => {
                    x.0 = TryUnbag;
                    simp.remove(&x);
                    x.0 = Unbag;
                    simp.insert(x);
                },
                TryUnbag => {
                    x.0 = Unbag;
                    if !simp.contains(&x) {
                        x.0 = TryUnbag;
                        simp.insert(x);
                    }
                },
                Async => {
                    x.0 = Async;
                    simp.insert(x);
                },
            }
        }
        self.impls = simp;
    }

    /// Do the bounds in `self` satisfy all the bounds in `other`?
    pub fn satisfies(&self, other: &BagInfo) -> bool {
        // other contains no bounds that are not also in self
        other.impls.difference(&self.impls).next().is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BagType {
    pub full: Type,
    pub info: BagInfo,
}

impl BagType {
    pub fn simple(full: Type, view: Type, unbag: Option<Type>) -> BagType {
        BagType {
            full,
            info: BagInfo::simple(view, unbag),
        }
    }

    pub fn simple_try(full: Type, view: Type, unbag: Option<Type>) -> BagType {
        BagType {
            full,
            info: BagInfo::simple_try(view, unbag),
        }
    }
}

#[derive(Debug)]
pub struct BagExpr {
    pub expr: Tokens,
    pub returns: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprType {
    pub ok_type: Type,
    pub is_result: bool,
}

impl ExprType {
    pub fn of(ty: Type) -> ExprType {
        ExprType { ok_type: ty, is_result: false }
    }

    pub fn of_result(ty: Type) -> ExprType {
        ExprType { ok_type: ty, is_result: true }
    }

    pub fn full(self) -> Type {
        let t = self.ok_type;
        if self.is_result {
            parse_quote! { Result<#t, ::failure::Error> }
        } else {
            t
        }
    }
}

#[derive(Debug)]
pub struct Expr {
    pub inputs: Vec<(Ident, Expr)>,
    pub expr: Tokens,
    pub returns: ExprType,
}

impl Expr {
    pub fn from_quote<T: ToTokens>(toks: T, returns: ExprType) -> Expr {
        Expr { inputs: Vec::new(), expr: toks.into_tokens(), returns }
    }

    pub fn flatten(self) -> FlatExpr {
        let expr = self.expr;
        let inputs = self.inputs.into_iter().map(|(v, e)| {
            let fe = e.flatten();
            let t = fe.returns.full();
            let e = fe.expr;
            quote! { let #v: #t = #e; }
        });
        FlatExpr {
            expr: quote! {{ #(#inputs)* #expr }},
            returns: self.returns,
        }
    }

    pub fn bag_static(self) -> BagExpr {
        let FlatExpr { 
            expr,
            returns: ExprType { 
                ok_type, 
                is_result,
        } } = self.flatten();

        BagExpr {
            expr: if is_result {
                quote!( ::bag::bags::TryStatic::<#ok_type>(#expr) )
            } else {
                quote!( ::bag::bags::Static::<#ok_type>(#expr) )
            },
            returns: if is_result {
                parse_quote!( ::bag::bags::TryStatic<#ok_type> )
            } else {
                parse_quote!( ::bag::bags::Static<#ok_type> )
            },
        }
    }

    pub fn bag_lazy_map(self) -> BagExpr {
        let inputs: Vec<_> = self.inputs.into_iter()
            .map(|(v, e)| (v, e.flatten()))
            .collect();
        let input_names: Vec<_> = inputs.iter().map(|&(x, _)| x).collect();
        let (input_types, input_exprs): (Vec<_>, Vec<_>) = inputs.into_iter()
            .map(|(_, x)| (x.returns.full(), x.expr))
            .unzip();

        let a_type = quote! { (#(#input_types,)*) };
        let b_type = self.returns.ok_type;
        let b_expr = self.expr;

        let bag_name = Ident::from(if self.returns.is_result {
            "TryLazyMap"
        } else {
            "LazyMap"
        });

        BagExpr {
            expr: quote! {
                ::bag::bags::#bag_name::<
                    #a_type,
                    #b_type,
                    fn(#a_type) -> #b_type
                >::new(
                    (#(#input_exprs,)*),
                    |(#(#input_names,)*)| #b_expr
                )
            },
            returns: parse_quote! {
                ::bag::bags::#bag_name<
                    #a_type,
                    #b_type,
                    fn(#a_type) -> #b_type
                >
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlatExpr {
    pub expr: Tokens,
    pub returns: ExprType,
}

pub trait GenericExpr {
    fn eval_to(&self, returns: &ExprType) -> Expr;
}

pub trait GenericBagExpr {
    fn eval_to_bag(&self, satisfy: &BagInfo) -> BagExpr;
}

