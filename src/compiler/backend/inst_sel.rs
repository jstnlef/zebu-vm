#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::inst_sel::*;

#[cfg(target_arch = "arm")]
pub use compiler::backend::arm::inst_sel::*;