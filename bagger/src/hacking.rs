/// prototyping stuff

use syn::{Ident, Type};
use quote::Tokens;

pub struct Bagger {
    
}

impl Bagger {
    pub fn new() -> Bagger {
        Bagger { }
    }

    pub fn bag(&mut self, info: BagInfo) -> BagSolution {
        assert_eq!(info.cap, BagTrait::TRY_BAG);
        BagSolution {
            file_path: info.path,
            buf_ty: info.ty,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BagTrait {
    BAG,
    TRY_BAG,
}

pub struct BagInfo {
    pub path: String,
    pub ty: Type,
    pub cap: BagTrait,
}

pub struct BagSolution {
    file_path: String,
    buf_ty: Type,
}

pub struct Artifact {

}

impl BagSolution {
    pub fn artifacts(&self) -> Vec<Artifact> {
        unimplemented!()
    }

    pub fn pre_type(&self) -> Type {

    }

    pub fn pre_expr(&self) -> Tokens {

    }

    pub fn data_expr(&self, pre_ref: Tokens) -> Tokens {
        quote! {
            ::bag::ops::file_contents::<_, #buf_ty>(#pre_ref)
        }
    }

    pub fn bag_expr(self) -> Tokens {
        let BagSolution {
            file_path,
            buf_ty,
        } = self;
        
    }
}
