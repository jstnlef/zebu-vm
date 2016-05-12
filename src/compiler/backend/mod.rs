pub mod inst_sel;

mod temp;
pub use compiler::backend::temp::Temporary;

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "arm")]
mod arm;