pub use self::arch_specific::*;

#[cfg(target_arch = "x86_64")]
#[path="x86_64/inst_sel.rs"]
mod arch_specific;

#[cfg(target_arch = "arm")]
#[path="arm/inst_sel.rs"]
mod arch_specific;