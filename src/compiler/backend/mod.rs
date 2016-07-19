pub mod inst_sel;
pub mod reg_alloc;
pub mod peephole_opt;
pub mod code_emission;

pub type ByteSize = usize;

// X86_64

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
pub use compiler::backend::x86_64::all_usable_regs;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::pick_group_for_reg;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::is_callee_saved;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::emit_code;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::emit_context;

// ARM

#[cfg(target_arch = "arm")]
#[path = "arch/arm/mod.rs"]
mod arm;

// common data structure with target specific info

use vm::context::VMContext;
use ast::types::*;
use ast::ptr::*;
pub fn resolve_backend_type_info (ty: &MuType, vm: &VMContext) -> BackendTypeInfo {
    match ty {
        // integral
        &MuType_::Int(size_in_bit) => {
            match size_in_bit {
                8  => BackendTypeInfo{size: 1, alignment: 1, struct_layout: None},
                16 => BackendTypeInfo{size: 2, alignment: 2, struct_layout: None},
                32 => BackendTypeInfo{size: 4, alignment: 4, struct_layout: None},
                64 => BackendTypeInfo{size: 8, alignment: 8, struct_layout: None},
                _ => unimplemented!()
            }
        },
        // pointer of any type
        &MuType_::Ref(_)
        | &MuType_::IRef(_)
        | &MuType_::WeakRef(_)
        | &MuType_::UPtr(_)
        | &MuType_::FuncRef(_)
        | &MuType_::UFuncPtr(_)
        | &MuType_::Tagref64
        | &MuType_::ThreadRef
        | &MuType_::StackRef => BackendTypeInfo{size: 8, alignment: 8, struct_layout: None},
        // floating point
        &MuType_::Float => BackendTypeInfo{size: 4, alignment: 4, struct_layout: None},
        &MuType_::Double => BackendTypeInfo{size: 8, alignment: 8, struct_layout: None},
        // array
        &MuType_::Array(ref ty, len) => {
            let ele_ty = vm.get_backend_type_info(ty);
            
            BackendTypeInfo{size: ele_ty.size * len, alignment: ele_ty.alignment, struct_layout: None}
        }
        // struct
        &MuType_::Struct(name) => {
            let read_lock = STRUCT_TAG_MAP.read().unwrap();
            let struc = read_lock.get(name).unwrap();
            let tys = struc.get_tys();            
            
            trace!("layout struct: {}", struc);
            layout_struct(tys, vm)
        }
        // hybrid 
        // - align is the most strict aligned element (from all fix tys and var ty)
        // - size is fixed tys size
        // - layout is fixed tys layout
        &MuType_::Hybrid(ref fix_tys, ref var_ty) => {
            // treat fix_tys as struct
            let mut ret = layout_struct(fix_tys, vm);
            
            // treat var_ty as array (getting its alignment)
            let var_align = vm.get_backend_type_info(var_ty).alignment;
            
            if ret.alignment < var_align {
                ret.alignment = var_align;
            }
            
            ret
        }
        // void
        &MuType_::Void => BackendTypeInfo{size: 0, alignment: 8, struct_layout: None},
        // vector
        &MuType_::Vector(_, _) => unimplemented!()
    }
}

fn layout_struct(tys: &Vec<P<MuType_>>, vm: &VMContext) -> BackendTypeInfo {
    let mut offsets : Vec<ByteSize> = vec![];
    let mut cur : ByteSize = 0;
    let mut struct_align : ByteSize = 0;
    
    for ty in tys.iter() {
        let ty_info = vm.get_backend_type_info(ty);
        trace!("examining field: {}, {:?}", ty, ty_info);
        
        let align = ty_info.alignment;
        if struct_align < align {
            struct_align = align;
        }
        
        if cur % align != 0 {
            // move cursor to next aligned offset
            cur = (cur / align + 1) * align;
        }
        
        offsets.push(cur);
        trace!("aligned to {}", cur);
        
        cur += ty_info.size;
    }
    
    // if we need padding at the end
    if cur % struct_align != 0 {
        cur = (cur / struct_align + 1) * struct_align;
    }
    
    BackendTypeInfo {
        size: cur,
        alignment: struct_align,
        struct_layout: Some(offsets)
    }
}

#[derive(Clone, Debug)]
pub struct BackendTypeInfo {
    size: ByteSize,
    alignment: ByteSize,
    struct_layout: Option<Vec<ByteSize>>
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RegGroup {GPR, FPR}