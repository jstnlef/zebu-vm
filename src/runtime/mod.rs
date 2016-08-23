pub mod mm;
pub mod thread;

pub use runtime::mm::common::Address;
pub use runtime::mm::common::ObjectReference;

use utils;
use ast::ir::*;
use compiler::backend::Word;
use compiler::backend::RegGroup;

#[derive(Clone, Debug)]
pub enum ValueLocation {
    Register(RegGroup, MuID),
    Direct(RegGroup, Address),
    Indirect(RegGroup, Address),
    Constant(RegGroup, Word),
    
    Relocatable(RegGroup, MuName)
}

impl ValueLocation {
    pub fn load_value(&self) -> (RegGroup, Word) {
        match self {
            &ValueLocation::Register(_, _)
            | &ValueLocation::Direct(_, _)
            | &ValueLocation::Indirect(_, _) => unimplemented!(),
            
            &ValueLocation::Constant(group, word) => {
                (group, word)
            }
            &ValueLocation::Relocatable(_, _) => panic!("expect a runtime value")
        }
    }
    
    #[allow(unused_variables)]
    pub fn from_constant(c: Constant) -> ValueLocation {
        match c {
            Constant::Int(int_val) => ValueLocation::Constant(RegGroup::GPR, utils::mem::u64_to_raw(int_val)),
            Constant::Float(f32_val) => ValueLocation::Constant(RegGroup::FPR, utils::mem::f32_to_raw(f32_val)),
            Constant::Double(f64_val) => ValueLocation::Constant(RegGroup::FPR, utils::mem::f64_to_raw(f64_val)),
            
            _ => unimplemented!()
        }
    }
}