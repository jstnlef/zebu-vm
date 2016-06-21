pub mod inst_sel;
pub mod reg_alloc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RegGroup {GPR, FPR}

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::init_machine_regs_for_func;

#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::number_of_regs_in_group;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::number_of_all_regs;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::all_regs;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::pick_group_for_reg;

#[cfg(target_arch = "arm")]
#[path = "arch/arm/mod.rs"]
mod arm;

#[cfg(target_arch = "arm")]
pub use compiler::backend::arm::GPR_COUNT;
#[cfg(target_arch = "arm")]
pub use compiler::backend::arm::FPR_COUNT;