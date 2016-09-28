mod api_c;
mod api_bridge;
pub mod api_impl;

mod deps { 

    // should import from ast/src/ir.rs
    pub type WPID  = usize;
    pub type MuID  = usize;
    pub type MuName = String;
    pub type CName  = MuName;

    pub struct APIMuValue {
        // stub
    }

}
