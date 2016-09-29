mod api_c;
mod api_bridge;
pub mod api_impl;

mod deps { 
    use std::cell::*;

    // should import from ast/src/ir.rs
    pub type WPID  = usize;
    pub type MuID  = usize;
    pub type MuName = String;
    pub type CName  = MuName;

    #[derive(Debug)]
    pub enum ValueBox {
        BoxInt(usize),
        BoxF32(f32),
        BoxF64(f64),
        BoxRef(Cell<usize>),    // so that GC can update the pointer
        BoxSeq(Vec<ValueBox>),
        BoxThread,
        BoxStack,
    }

    pub struct APIMuValue {
        pub ty: MuID,
        pub vb: ValueBox,
    }

}
