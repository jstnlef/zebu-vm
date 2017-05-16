#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::inst_sel::*;

#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::inst_sel::*;
