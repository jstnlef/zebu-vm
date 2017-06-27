// Copyright 2017 The Australian National University
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// A instruction selection pass. Uses simple tree pattern matching.
pub mod inst_sel;
/// A register allocation pass. Graph coloring.
pub mod reg_alloc;
/// A peephole optimization pass after register allocation.
pub mod peephole_opt;
/// Code emission pass. May as well emit dot graph for IR and generated code.
pub mod code_emission;

use utils::ByteSize;
use utils::math::align_up;
use runtime::mm;
use runtime::mm::common::gctype::{GCType, GCTYPE_INIT_ID, RefPattern};

/// for ahead-of-time compilation (boot image making), the file contains a persisted VM, a persisted
/// heap, constants. This allows the VM to resume execution with the same status as before persisting.
#[cfg(feature = "aot")]
pub const AOT_EMIT_CONTEXT_FILE : &'static str = "context.s";

/// name for prologue (this is not full name, but prologue name is generated from this)
pub const PROLOGUE_BLOCK_NAME: &'static str = "prologue";
/// name for epilogue (this is not full name, but epilogue name is generated from this)
pub const EPILOGUE_BLOCK_NAME: &'static str = "epilogue";

// type alias to make backend code more readable
pub type Reg<'a> = &'a P<Value>;
pub type Mem<'a> = &'a P<Value>;

// re-export some common backend functions to allow target independent code

/// --- X86_64 backend ---
#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
pub mod x86_64;

/// estimates how many machine instructions are needed for a Mu instruction
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::estimate_insts_for_ir;

/// initializes machine registers in the function context
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::init_machine_regs_for_func;

/// checks if two machine registers are alias (the same register)
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::is_aliased;

/// gets color for a machine register (e.g. AH, AX, EAX all have color of RAX)
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::get_color_for_precolored;

/// returns the number of registers in a given RegGroup
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::number_of_regs_in_group;

/// returns the number of all machine registers
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::number_of_all_regs;

/// returns a hashmap of all the machine registers
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::all_regs;

/// returns all usable registers (machine registers that can be assigned to temporaries)
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::all_usable_regs;

/// returns RegGroup for a machine register
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::pick_group_for_reg;

/// checks if a register is callee saved
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::is_callee_saved;

/// emits code for a function version (the function needs to be compiled first)
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::CALLEE_SAVED_COUNT;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::get_callee_saved_offset;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::get_previous_frame_pointer;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::get_return_address;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::set_previous_frame_pointer;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::set_return_address;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::get_previous_stack_pointer;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::emit_code;

/// emits context (persisted VM/heap/etc), should only be called after
/// finishing compilation for all functions
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::emit_context;

/// emits context with consideration of relocation info
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::emit_context_with_reloc;

/// rewrites a compiled Mu function with given spilling info
/// (inserting load/store for spilled temporaries)
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::spill_rewrite;

/// --- aarch64 backend ---
#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
pub mod aarch64;

/// estimates how many machine instructions are needed for a Mu instruction
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::estimate_insts_for_ir;

/// initializes machine registers in the function context
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::init_machine_regs_for_func;

/// checks if two machine registers are alias (the same register)
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::is_aliased;

/// gets color for a machine register
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::get_color_for_precolored;

/// returns the number of registers in a given RegGroup
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::number_of_regs_in_group;

/// returns the number of all machine registers
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::number_of_all_regs;

/// returns a hashmap of all the machine registers
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::all_regs;

/// returns all usable registers (machine registers that can be assigned to temporaries)
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::all_usable_regs;

/// returns RegGroup for a machine register
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::pick_group_for_reg;

/// checks if a register is callee saved
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::is_callee_saved;

/// emits code for a function version (the function needs to be compiled first)
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::CALLEE_SAVED_COUNT ;
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::get_callee_saved_offset;
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::get_previous_frame_pointer;
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::get_return_address;
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::get_previous_stack_pointer;
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::set_previous_frame_pointer;
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::set_return_address;
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::emit_code;

/// emits context (persisted VM/heap/etc), should only be called after
/// finishing compilation for all functions
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::emit_context;

/// emits context with consideration of relocation info
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::emit_context_with_reloc;

/// rewrites a compiled Mu function with given spilling info
/// (inserting load/store for spilled temporaries)
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::spill_rewrite;

use vm::VM;
use ast::types::*;
use ast::ptr::*;
use ast::ir::*;

/// BackendType describes storage type info for a MuType, including
/// size, alignment, struct layout, array element padded size, GC type.
///
/// We are compatible with C ABI, so that Mu objects can be accessed from
/// native without extra steps (though they need to be pinned first)
///
//  GCType is a temporary design, we will rewrite GC (Issue#12)
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct BackendType {
    pub size: ByteSize,
    pub alignment: ByteSize,
    /// struct layout of the type, None if this is not a struct/hybrid type
    pub struct_layout: Option<Vec<ByteSize>>,
    /// element padded size for hybrid/array type
    /// for hybrid/array, every element needs to be properly aligned
    /// thus it may take more space than it actually needs
    pub elem_padded_size: Option<ByteSize>,
    /// GC type, containing information for GC (this is a temporary design)
    /// See Issue#12
    pub gc_type: P<GCType>
}

impl BackendType {
    /// gets field offset of a struct/hybrid type. Panics if this is not struct/hybrid type
    pub fn get_field_offset(&self, index: usize) -> ByteSize {
        if self.struct_layout.is_some() {
            let layout = self.struct_layout.as_ref().unwrap();
            layout[index]
        } else {
            panic!("trying to get field offset on a non-struct type")
        }
    }

    /// resolves a MuType to a BackendType
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    pub fn resolve(ty: &MuType, vm: &VM) -> BackendType {
        match ty.v {
            // integer
            MuType_::Int(size_in_bit) => {
                match size_in_bit {
                    1  => BackendType{
                        size: 1, alignment: 1, struct_layout: None, elem_padded_size: None,
                        gc_type: mm::add_gc_type(GCType::new_noreftype(1, 1))
                    },
                    8  => BackendType{
                        size: 1, alignment: 1, struct_layout: None, elem_padded_size: None,
                        gc_type: mm::add_gc_type(GCType::new_noreftype(1, 1))
                    },
                    16 => BackendType{
                        size: 2, alignment: 2, struct_layout: None, elem_padded_size: None,
                        gc_type: mm::add_gc_type(GCType::new_noreftype(2, 2))
                    },
                    32 => BackendType{
                        size: 4, alignment: 4, struct_layout: None, elem_padded_size: None,
                        gc_type: mm::add_gc_type(GCType::new_noreftype(4, 4))
                    },
                    64 => BackendType{
                        size: 8, alignment: 8, struct_layout: None, elem_padded_size: None,
                        gc_type: mm::add_gc_type(GCType::new_noreftype(8, 8))
                    },
                    128 => BackendType {
                        size: 16, alignment: 16, struct_layout: None, elem_padded_size: None,
                        gc_type: mm::add_gc_type(GCType::new_noreftype(16, 16))
                    },
                    _ => unimplemented!()
                }
            },
            // reference of any type
            MuType_::Ref(_)
            | MuType_::IRef(_)
            | MuType_::WeakRef(_) => BackendType{
                size: 8, alignment: 8, struct_layout: None, elem_padded_size: None,
                gc_type: mm::add_gc_type(GCType::new_reftype())
            },
            // pointer/opque ref
            MuType_::UPtr(_)
            | MuType_::UFuncPtr(_)
            | MuType_::FuncRef(_)
            | MuType_::ThreadRef
            | MuType_::StackRef => BackendType{
                size: 8, alignment: 8, struct_layout: None, elem_padded_size: None,
                gc_type: mm::add_gc_type(GCType::new_noreftype(8, 8))
            },
            // tagref
            MuType_::Tagref64 => BackendType {
                size: 8, alignment: 8, struct_layout: None, elem_padded_size: None,
                gc_type: mm::add_gc_type(GCType::new_reftype())
            },
            // floating point
            MuType_::Float => BackendType{
                size: 4, alignment: 4, struct_layout: None, elem_padded_size: None,
                gc_type: mm::add_gc_type(GCType::new_noreftype(4, 4))
            },
            MuType_::Double => BackendType {
                size: 8, alignment: 8, struct_layout: None, elem_padded_size: None,
                gc_type: mm::add_gc_type(GCType::new_noreftype(8, 8))
            },
            // array
            MuType_::Array(ref ty, len) => {
                let ele_ty = vm.get_backend_type_info(ty.id());
                let ele_padded_size = align_up(ele_ty.size, ele_ty.alignment);

                BackendType{
                    size         : ele_padded_size * len,
                    alignment    : ele_ty.alignment,
                    struct_layout: None,
                    elem_padded_size : Some(ele_padded_size),
                    gc_type      : mm::add_gc_type(GCType::new_fix(GCTYPE_INIT_ID,
                                                                   ele_padded_size * len,
                                                                   ele_ty.alignment,
                                                                   Some(RefPattern::Repeat{
                                                                       pattern: Box::new(RefPattern::NestedType(vec![ele_ty.gc_type])),
                                                                       count  : len
                                                                   })
                    ))
                }
            }
            // struct
            MuType_::Struct(ref name) => {
                let read_lock = STRUCT_TAG_MAP.read().unwrap();
                let struc = read_lock.get(name).unwrap();
                let tys = struc.get_tys();

                trace!("layout struct: {}", struc);
                BackendType::layout_struct(tys, vm)
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
                let mut ret = BackendType::layout_struct(fix_tys, vm);

                // treat var_ty as array (getting its alignment)
                let var_ele_ty = vm.get_backend_type_info(var_ty.id());
                let var_align = var_ele_ty.alignment;
                let var_padded_size = align_up(var_ele_ty.size, var_ele_ty.alignment);
                ret.elem_padded_size = Some(var_padded_size);

                // fix type info as hybrid
                // 1. check alignment
                if ret.alignment < var_align {
                    ret.alignment = var_align;
                }
                // 2. fix gc type
                let mut gctype = ret.gc_type.as_ref().clone();
                gctype.var_refs = Some(RefPattern::NestedType(vec![var_ele_ty.gc_type.clone()]));
                gctype.var_size = Some(var_padded_size);
                ret.gc_type = mm::add_gc_type(gctype);

                ret
            }
            // void
            MuType_::Void => BackendType{
                size: 0, alignment: 8, struct_layout: None, elem_padded_size: None,
                gc_type: mm::add_gc_type(GCType::new_noreftype(0, 8))
            },
            // vector
            MuType_::Vector(_, _) => unimplemented!()
        }
    }

    /// layouts struct fields
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn layout_struct(tys: &Vec<P<MuType>>, vm: &VM) -> BackendType {
        let mut offsets : Vec<ByteSize> = vec![];
        let mut cur : ByteSize = 0;
        let mut struct_align : ByteSize = 1;

        // for gc type
        let mut use_ref_offsets = true;
        let mut ref_offsets = vec![];
        let mut gc_types    = vec![];

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

            // for convenience, if the struct contains other struct/array
            // we do not use reference map
            if ty.is_aggregate() {
                use_ref_offsets = false;
            }

            // if this type is reference type, we store its offsets
            // we may not use this ref map though
            if ty.is_heap_reference() {
                ref_offsets.push(cur);
            }
            // always store its gc type (we may not use it as well)
            gc_types.push(ty_info.gc_type.clone());

            cur += ty_info.size;
        }

        // if we need padding at the end
        let size = if cur % struct_align != 0 {
            (cur / struct_align + 1) * struct_align
        } else {
            cur
        };

        BackendType {
            size         : size,
            alignment    : struct_align,
            struct_layout: Some(offsets),
            elem_padded_size: None,
            gc_type      : mm::add_gc_type(GCType::new_fix(GCTYPE_INIT_ID,
                                                           size,
                                                           struct_align,
                                                           Some(if use_ref_offsets {
                                                               RefPattern::Map {
                                                                   offsets: ref_offsets,
                                                                   size: size
                                                               }
                                                           } else {
                                                               RefPattern::NestedType(gc_types)
                                                           })))
        }
    }

    /// sequentially layout a few Mu types as if they are fields in a struct.
    /// Returns a triple of (size, alignment, offsets of each type)
    /// (when dealing with call convention, we use this function to layout stack arguments)
    pub fn sequential_layout(tys: &Vec<P<MuType>>, vm: &VM) -> (ByteSize, ByteSize, Vec<ByteSize>) {
        let ret = BackendType::layout_struct(tys, vm);

        (ret.size, ret.alignment, ret.struct_layout.unwrap())
    }
}

use std::fmt;
impl fmt::Display for BackendType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} bytes ({} bytes aligned), ", self.size, self.alignment).unwrap();
        if self.struct_layout.is_some() {
            use utils::vec_utils;

            let layout = self.struct_layout.as_ref().unwrap();
            write!(f, "field offsets: ({})", vec_utils::as_str(layout)).unwrap();
        }

        Ok(())
    }
}

/// RegGroup describes register class
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, RustcEncodable, RustcDecodable)]
pub enum RegGroup {
    /// general purpose register
    GPR,
    /// requires two general purpose register
    GPREX,
    /// floating point register
    FPR
}

impl RegGroup {
    /// gets RegGroup from a MuType
    pub fn get_from_ty(ty: &P<MuType>) -> RegGroup {
        match ty.v {
            // for now, only use 64bits registers
            MuType_::Int(len) if len == 1  => RegGroup::GPR,
            MuType_::Int(len) if len == 8  => RegGroup::GPR,
            MuType_::Int(len) if len == 16 => RegGroup::GPR,
            MuType_::Int(len) if len == 32 => RegGroup::GPR,
            MuType_::Int(len) if len == 64 => RegGroup::GPR,
            MuType_::Int(len) if len == 128=> RegGroup::GPREX,

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

    /// gets RegGroup from a Mu Value
    pub fn get_from_value(val: &P<Value>) -> RegGroup {
        RegGroup::get_from_ty(&val.ty)
    }
}