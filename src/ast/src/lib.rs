//! # MuIR AST crate
//!
//! This crate provides data structures to allow construct MuIR in Rust code, including:
//! * types
//! * ir
//!   * MuFunction
//!   * MuFunctionVersion
//!     * FunctionContent
//!       * Block
//!         * BlockContent
//!           * TreeNode
//!             * Value
//!             * Instruction
//! * inst
//! * op (operators)
//!
//! Client should not create MuIR via this crate, use API instead.

extern crate log;
extern crate simple_logger;
#[macro_use]
extern crate lazy_static;
extern crate rustc_serialize;
extern crate utils;

/// all data structures for MuIR is an *MuEntity*
/// which has a unique MuID, and an optional MuName
#[macro_export]
macro_rules! impl_mu_entity {
    ($entity: ty) => {
        impl MuEntity for $entity {
            #[inline(always)]
            fn id(&self) -> MuID {self.hdr.id()}
            #[inline(always)]
            fn name(&self) -> Option<MuName> {self.hdr.name()}
            fn as_entity(&self) -> &MuEntity {
                let ref_ty : &$entity = self;
                ref_ty as &MuEntity
            }
        }
    }
}

macro_rules! select_value {
    ($cond: expr, $res1 : expr, $res2 : expr) => {
        if $cond {
            $res1
        } else {
            $res2
        }
    }
}

#[macro_use]
pub mod ir;
pub mod inst;
pub mod types;
pub mod ptr;
pub mod op;
