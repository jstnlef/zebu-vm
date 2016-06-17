pub mod inst_sel;
pub mod reg_alloc;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
mod x86_64;

#[cfg(target_arch = "arm")]
#[path = "arch/arm/mod.rs"]
mod arm;

#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::get_name_for_value;

#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::GPR_COUNT;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::FPR_COUNT;

#[cfg(target_arch = "arm")]
pub use compiler::backend::arm::GPR_COUNT;
#[cfg(target_arch = "arm")]
pub use compiler::backend::arm::FPR_COUNT;