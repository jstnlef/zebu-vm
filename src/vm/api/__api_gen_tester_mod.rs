// Invoke "python3 muapi2rustapi.py", and then
// invoke "rustc --test __api_gen_tester_mod.rs -o /tmp/api_gen_tester_junk"
// to test whether the generated code compiles.

mod api_c;
mod api_bridge;
mod __api_impl_stubs;
mod api_impl {
    pub use __api_impl_stubs::*;
}

/// This is for testing. In the productional setting, replace them with the definitions from
/// `src/ast/src/ir.rs` and `src/ast/src/bundle.rs`
mod deps { 

    // should import from ast/src/ir.rs
    pub type WPID  = usize;
    pub type MuID  = usize;
    pub type MuName = String;
    pub type CName  = MuName;

    pub struct APIHandleKey {
        // stub
    }

}
