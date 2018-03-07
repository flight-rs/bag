use syn::{Ident, Type};
use quote::{Tokens, ToTokens};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BagForm {
    Simple,
    Async,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BagInfo {
    pub form: BagForm,
    pub canfail: bool,
    pub unbag: Option<Type>,
    pub contains: Type,
}

impl BagInfo {
    pub fn holds(view: Type, unbag: Option<Type>) -> BagInfo {
        BagInfo {
            form: BagForm::Simple,
            canfail: false, 
            unbag: unbag,
            contains: view,
        }
    }

    pub fn holds_result(view: Type, unbag: Option<Type>) -> BagInfo {
        BagInfo {
            form: BagForm::Simple,
            canfail: true, 
            unbag: unbag,
            contains: view,
        }
    }

    pub fn satisfies(&self, other: &BagInfo) -> bool {
           self.form == other.form
        && self.canfail == other.canfail
        && (other.unbag.is_none() || self.unbag == other.unbag)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BagType {
    pub full: Type,
    pub info: BagInfo,
}

impl BagType {
    pub fn holds(full: Type, view: Type, unbag: Option<Type>) -> BagType {
        BagType {
            full,
            info: BagInfo::holds(view, unbag),
        }
    }

    pub fn holds_result(full: Type, view: Type, unbag: Option<Type>) -> BagType {
        BagType {
            full,
            info: BagInfo::holds_result(view, unbag),
        }
    }
}

#[derive(Debug)]
pub struct BagExpr {
    pub expr: Tokens,
    pub returns: BagType,
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

    pub fn bag_static(self, view: Type) -> BagExpr {
        let FlatExpr { 
            expr,
            returns: ExprType { 
                ok_type, 
                is_result,
        } } = self.flatten();
        let unbag = Some(ok_type.clone());

        BagExpr {
            expr: if is_result {
                quote!( ::bag::bags::TryStatic::<#ok_type>(#expr) )
            } else {
                quote!( ::bag::bags::Static::<#ok_type>(#expr) )
            },
            returns: BagType {
                full: if is_result {
                    parse_quote!( ::bag::bags::TryStatic<#ok_type> )
                } else {
                    parse_quote!( ::bag::bags::Static<#ok_type> )
                },
                info: BagInfo {
                    form: BagForm::Simple,
                    canfail: is_result,
                    unbag,
                    contains: view,
                },
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

        let contains = self.returns.ok_type.clone();
        let unbag = Some(self.returns.ok_type.clone());

        let a_type = quote! { (#(#input_types,)*) };
        let b_type = self.returns.ok_type;
        let b_expr = self.expr;

        let (bag_name, canfail): (Ident, bool) = if self.returns.is_result {
            (parse_quote!{ TryLazyMap }, true)
        } else {
            (parse_quote!{ LazyMap }, false)
        };

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
            returns: BagType {
                full: parse_quote! {
                    ::bag::bags::#bag_name<
                        #a_type,
                        #b_type,
                        fn(#a_type) -> #b_type
                    >
                },
                info: BagInfo {
                    form: BagForm::Simple,
                    canfail,
                    unbag,
                    contains,
                }
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
    fn eval_to(&self, returns: &Type) -> Expr;
}

pub trait GenericBagExpr {
    fn eval_to_bag(&self, contains: &Type) -> BagExpr;
}

