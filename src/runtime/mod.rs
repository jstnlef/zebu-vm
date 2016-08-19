pub mod mem;
pub mod thread;

pub use runtime::mem::common::Address;
pub use runtime::mem::common::ObjectReference;


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
            | &ValueLocation::Indirect(_, _)
            | &ValueLocation::Constant(_, _) => unimplemented!(),
            &ValueLocation::Relocatable(_, _) => panic!("expect a runtime value")
        }
    }
    
    #[allow(unused_variables)]
    pub fn from_constant(c: Constant) -> ValueLocation {
        unimplemented!()
    }
}