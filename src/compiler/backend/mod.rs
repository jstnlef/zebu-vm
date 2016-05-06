pub mod inst_sel;

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "arm")]
mod arm;