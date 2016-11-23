pub mod inst_sel;
pub mod reg_alloc;
pub mod peephole_opt;
pub mod code_emission;

use utils::ByteSize;

pub type Word = usize;
pub const WORD_SIZE : ByteSize = 8;

pub const AOT_EMIT_CONTEXT_FILE : &'static str = "context.s";

// this is not full name, but pro/epilogue name is generated from this
pub const PROLOGUE_BLOCK_NAME: &'static str = "prologue";
pub const EPILOGUE_BLOCK_NAME: &'static str = "epilogue";

pub type Reg<'a> = &'a P<Value>;
pub type Mem<'a> = &'a P<Value>;

// X86_64

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::init_machine_regs_for_func;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::is_aliased;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::get_color_for_precolroed;
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
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::spill_rewrite;

// ARM

#[cfg(target_arch = "arm")]
#[path = "arch/arm/mod.rs"]
mod arm;

// common data structure with target specific info

use vm::VM;
use ast::types::*;
use ast::ptr::*;
use ast::ir::*;
pub fn resolve_backend_type_info (ty: &MuType, vm: &VM) -> BackendTypeInfo {
    match ty.v {
        // integral
        MuType_::Int(size_in_bit) => {
            match size_in_bit {
                1  => BackendTypeInfo{size: 1, alignment: 1, struct_layout: None},
                8  => BackendTypeInfo{size: 1, alignment: 1, struct_layout: None},
                16 => BackendTypeInfo{size: 2, alignment: 2, struct_layout: None},
                32 => BackendTypeInfo{size: 4, alignment: 4, struct_layout: None},
                64 => BackendTypeInfo{size: 8, alignment: 8, struct_layout: None},
                _ => unimplemented!()
            }
        },
        // pointer of any type
        MuType_::Ref(_)
        | MuType_::IRef(_)
        | MuType_::WeakRef(_)
        | MuType_::UPtr(_)
        | MuType_::FuncRef(_)
        | MuType_::UFuncPtr(_)
        | MuType_::Tagref64
        | MuType_::ThreadRef
        | MuType_::StackRef => BackendTypeInfo{size: 8, alignment: 8, struct_layout: None},
        // floating point
        MuType_::Float => BackendTypeInfo{size: 4, alignment: 4, struct_layout: None},
        MuType_::Double => BackendTypeInfo{size: 8, alignment: 8, struct_layout: None},
        // array
        MuType_::Array(ref ty, len) => {
            let ele_ty = vm.get_backend_type_info(ty.id());
            
            BackendTypeInfo{size: ele_ty.size * len, alignment: ele_ty.alignment, struct_layout: None}
        }
        // struct
        MuType_::Struct(ref name) => {
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
        MuType_::Hybrid(ref name) => {
            let read_lock = HYBRID_TAG_MAP.read().unwrap();
            let hybrid = read_lock.get(name).unwrap();

            let fix_tys = hybrid.get_fix_tys();
            let var_ty  = hybrid.get_var_ty();

            // treat fix_tys as struct
            let mut ret = layout_struct(fix_tys, vm);
            
            // treat var_ty as array (getting its alignment)
            let var_align = vm.get_backend_type_info(var_ty.id()).alignment;
            
            if ret.alignment < var_align {
                ret.alignment = var_align;
            }
            
            ret
        }
        // void
        MuType_::Void => BackendTypeInfo{size: 0, alignment: 8, struct_layout: None},
        // vector
        MuType_::Vector(_, _) => unimplemented!()
    }
}

fn layout_struct(tys: &Vec<P<MuType>>, vm: &VM) -> BackendTypeInfo {
    let mut offsets : Vec<ByteSize> = vec![];
    let mut cur : ByteSize = 0;
    let mut struct_align : ByteSize = 1;
    
    for ty in tys.iter() {
        let ty_info = vm.get_backend_type_info(ty.id());
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

pub fn sequetial_layout(tys: &Vec<P<MuType>>, vm: &VM) -> (ByteSize, ByteSize, Vec<ByteSize>) {
    let ret = layout_struct(tys, vm);
    
    (ret.size, ret.alignment, ret.struct_layout.unwrap())
} 

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct BackendTypeInfo {
    pub size: ByteSize,
    pub alignment: ByteSize,
    pub struct_layout: Option<Vec<ByteSize>>
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, RustcEncodable, RustcDecodable)]
pub enum RegGroup {GPR, FPR}

impl RegGroup {
    pub fn get(ty: &P<MuType>) -> RegGroup {
        match ty.v {
            // for now, only use 64bits registers
            MuType_::Int(len) if len == 1  => RegGroup::GPR,
            MuType_::Int(len) if len == 8  => RegGroup::GPR,
            MuType_::Int(len) if len == 16 => RegGroup::GPR,
            MuType_::Int(len) if len == 32 => RegGroup::GPR,
            MuType_::Int(len) if len == 64 => RegGroup::GPR,

            MuType_::Ref(_)
            | MuType_::IRef(_)
            | MuType_::WeakRef(_)
            | MuType_::UPtr(_)
            | MuType_::ThreadRef
            | MuType_::StackRef
            | MuType_::Tagref64
            | MuType_::FuncRef(_)
            | MuType_::UFuncPtr(_)         => RegGroup::GPR,

            MuType_::Float                 => RegGroup::FPR,
            MuType_::Double                => RegGroup::FPR,

            _ => unimplemented!()
        }
    }
}