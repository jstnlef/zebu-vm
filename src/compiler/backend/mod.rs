pub mod inst_sel;
pub mod reg_alloc;

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "arm")]
mod arm;