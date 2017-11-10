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
/// A Dominator Tree pass for machine code.
pub mod mc_loopanalysis;
/// A register allocation pass. Graph coloring.
pub mod reg_alloc;
/// A peephole optimization pass after register allocation.
pub mod peephole_opt;
/// Code emission pass. May as well emit dot graph for IR and generated code.
pub mod code_emission;

use std;
use utils::*;
use utils::math::align_up;
use runtime::mm::*;
use num::integer::lcm;

/// for ahead-of-time compilation (boot image making), the file contains a persisted VM,
/// a persisted heap, constants. This allows the VM to resume execution with
/// the same status as before persisting.
#[cfg(feature = "aot")]
pub const AOT_EMIT_CONTEXT_FILE: &'static str = "context.S";

pub const AOT_EMIT_SYM_TABLE_FILE: &'static str = "mu_sym_table.S";

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
pub use compiler::backend::x86_64::number_of_usable_regs_in_group;
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
/// number of callee saved registers
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::CALLEE_SAVED_COUNT;
/// gets offset for callee saved registers (used for exception table)
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::get_callee_saved_offset;
/// gets frame pointer for previous frame
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::get_previous_frame_pointer;
/// gets return address for current frame
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::get_return_address;
/// sets frame pointer for previous frame
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::set_previous_frame_pointer;
/// sets return address for current frame
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::set_return_address;
/// gets staci pointer for previous frame
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::get_previous_stack_pointer;
/// emits code for a function version (the function needs to be compiled first)
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
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::ARGUMENT_GPRS;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::ARGUMENT_FPRS;
#[cfg(target_arch = "x86_64")]
pub use compiler::backend::x86_64::call_stack_size;

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
pub use compiler::backend::aarch64::number_of_usable_regs_in_group;
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
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::CALLEE_SAVED_COUNT;
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
/// emits code for a function version (the function needs to be compiled first)
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
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::ARGUMENT_GPRS;
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::ARGUMENT_FPRS;
#[cfg(target_arch = "aarch64")]
pub use compiler::backend::aarch64::call_stack_size;

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
#[derive(Clone, Debug)]
pub struct BackendType {
    pub ty: P<MuType>,
    pub size: ByteSize,
    pub alignment: ByteSize,
    /// struct layout of the type, None if this is not a struct/hybrid type
    pub struct_layout: Option<Vec<ByteSize>>,
    /// element size for hybrid/array type
    pub elem_size: Option<ByteSize>,
    /// GC type, containing information for GC
    pub gc_type: Option<TypeEncode>,
    /// GC type for full encoding of hybrid types, as hybrid types can be arbitrarily long,
    /// and we need a full encoding
    pub gc_type_hybrid_full: Option<TypeEncode>
}

rodal_struct!(BackendType {
    ty,
    size,
    alignment,
    struct_layout,
    elem_size,
    gc_type,
    gc_type_hybrid_full
});

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

    pub fn resolve_gc_type(&mut self, vm: &VM) {
        // if we have resolved gc type for this type, just return
        debug_assert!(self.gc_type.is_none());

        let mut hybrid_full_encode = None;
        let encode = match self.ty.v {
            // integer
            MuType_::Int(size_in_bit) => {
                match size_in_bit {
                    1...64 => TypeEncode::short_noref(MINIMAL_ALIGNMENT, 1),
                    128 => TypeEncode::short_noref(MINIMAL_ALIGNMENT, 2),
                    _ => unimplemented!()
                }
            }
            // ref
            MuType_::Ref(_) | MuType_::IRef(_) => TypeEncode::short_ref(),
            // weakref
            MuType_::WeakRef(_) => TypeEncode::short_weakref(),
            // native pointer or opaque ref
            MuType_::UPtr(_) |
            MuType_::UFuncPtr(_) |
            MuType_::FuncRef(_) |
            MuType_::ThreadRef |
            MuType_::StackRef => TypeEncode::short_noref(MINIMAL_ALIGNMENT, 1),
            // tag ref
            MuType_::Tagref64 => TypeEncode::short_tagref(),
            // floating point
            MuType_::Float | MuType_::Double => TypeEncode::short_noref(MINIMAL_ALIGNMENT, 1),
            // struct and array
            MuType_::Struct(_) | MuType_::Array(_, _) => {
                let mut word_tys = vec![];
                BackendType::append_word_ty(&mut word_tys, 0, &self.ty, vm);
                debug_assert_eq!(
                    math::align_up(self.size, POINTER_SIZE) / POINTER_SIZE,
                    word_tys.len()
                );
                if self.size > MAX_MEDIUM_OBJECT {
                    TypeEncode::full(check_alignment(self.alignment), word_tys, vec![])
                } else {
                    TypeEncode::short_aggregate_fix(check_alignment(self.alignment), word_tys)
                }
            }
            // hybrid
            MuType_::Hybrid(ref name) => {
                let lock = HYBRID_TAG_MAP.read().unwrap();
                let hybrid = lock.get(name).unwrap();

                // for fix types
                let fix_tys = hybrid.get_fix_tys();
                let fix_ty_offsets = self.struct_layout.as_ref().unwrap();
                let mut fix_word_tys = vec![];
                for i in 0..fix_tys.len() {
                    let next_offset = BackendType::append_word_ty(
                        &mut fix_word_tys,
                        fix_ty_offsets[i],
                        &fix_tys[i],
                        vm
                    );

                    if cfg!(debug_assertions) && i != fix_tys.len() - 1 {
                        assert!(next_offset <= fix_ty_offsets[i + 1]);
                    }
                }

                let var_ty = hybrid.get_var_ty();
                let mut var_word_tys = vec![];
                BackendType::append_word_ty(&mut var_word_tys, self.size, &var_ty, vm);

                hybrid_full_encode = Some(TypeEncode::full(
                    check_alignment(self.alignment),
                    fix_word_tys.clone(),
                    var_word_tys.clone()
                ));
                TypeEncode::short_hybrid(
                    check_alignment(self.alignment),
                    fix_word_tys,
                    var_word_tys
                )
            }
            _ => unimplemented!()
        };

        self.gc_type = Some(encode);
        if hybrid_full_encode.is_some() {
            self.gc_type_hybrid_full = hybrid_full_encode;
        }
    }

    /// finish current type, and returns new offset
    fn append_word_ty(
        res: &mut Vec<WordType>,
        cur_offset: ByteSize,
        cur_ty: &P<MuType>,
        vm: &VM
    ) -> ByteSize {
        debug!(
            "append_word_ty(): cur_offset={}, cur_ty={}",
            cur_offset,
            cur_ty
        );
        let cur_backend_ty = BackendType::resolve(cur_ty, vm);
        let cur_offset = math::align_up(cur_offset, cur_backend_ty.alignment);
        let pointer_aligned = cur_offset % POINTER_SIZE == 0;
        match cur_ty.v {
            MuType_::Int(_) => {
                if pointer_aligned {
                    res.push(WordType::NonRef);
                }
            }
            MuType_::Ref(_) | MuType_::IRef(_) => {
                debug_assert!(pointer_aligned);
                res.push(WordType::Ref);
            }
            MuType_::WeakRef(_) => {
                debug_assert!(pointer_aligned);
                res.push(WordType::WeakRef);
            }
            MuType_::UPtr(_) |
            MuType_::UFuncPtr(_) |
            MuType_::FuncRef(_) |
            MuType_::ThreadRef |
            MuType_::StackRef => {
                debug_assert!(pointer_aligned);
                res.push(WordType::NonRef);
            }
            MuType_::Tagref64 => {
                debug_assert!(pointer_aligned);
                res.push(WordType::TaggedRef);
            }
            MuType_::Float => {
                if pointer_aligned {
                    res.push(WordType::NonRef);
                }
            }
            MuType_::Double => {
                debug_assert!(pointer_aligned);
                res.push(WordType::NonRef);
            }
            MuType_::Struct(ref name) => {
                let struct_tys = {
                    let lock = STRUCT_TAG_MAP.read().unwrap();
                    let struc = lock.get(name).unwrap();
                    struc.get_tys().to_vec()
                };
                debug_assert!(cur_backend_ty.struct_layout.is_some());
                let struct_ty_offsets = cur_backend_ty.struct_layout.as_ref().unwrap();
                debug_assert_eq!(struct_tys.len(), struct_ty_offsets.len());
                for i in 0..struct_tys.len() {
                    let next_offset =
                        BackendType::append_word_ty(res, struct_ty_offsets[i], &struct_tys[i], vm);

                    if cfg!(debug_assertions) && i != struct_tys.len() - 1 {
                        assert!(next_offset <= struct_ty_offsets[i + 1]);
                    }
                }
            }
            MuType_::Array(ref ty, len) => {
                let backend_ty = BackendType::resolve(cur_ty, vm);
                for i in 0..len {
                    let offset = backend_ty.elem_size.unwrap() * i;
                    let next_offset = BackendType::append_word_ty(res, offset, ty, vm);
                    debug_assert!((next_offset - offset) <= backend_ty.elem_size.unwrap());
                }
            }
            MuType_::Hybrid(_) => unreachable!(),
            _ => unimplemented!()
        }
        cur_offset + cur_backend_ty.size
    }

    /// resolves a MuType to a BackendType
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    pub fn resolve(ty: &P<MuType>, vm: &VM) -> BackendType {
        match ty.v {
            // integer
            MuType_::Int(size_in_bit) => {
                match size_in_bit {
                    1...8 => BackendType {
                        ty: ty.clone(),
                        size: 1,
                        alignment: 1,
                        struct_layout: None,
                        elem_size: None,
                        gc_type: None,
                        gc_type_hybrid_full: None
                    },
                    9...16 => BackendType {
                        ty: ty.clone(),
                        size: 2,
                        alignment: 2,
                        struct_layout: None,
                        elem_size: None,
                        gc_type: None,
                        gc_type_hybrid_full: None
                    },
                    17...32 => BackendType {
                        ty: ty.clone(),
                        size: 4,
                        alignment: 4,
                        struct_layout: None,
                        elem_size: None,
                        gc_type: None,
                        gc_type_hybrid_full: None
                    },
                    33...64 => BackendType {
                        ty: ty.clone(),
                        size: 8,
                        alignment: 8,
                        struct_layout: None,
                        elem_size: None,
                        gc_type: None,
                        gc_type_hybrid_full: None
                    },
                    128 => BackendType {
                        ty: ty.clone(),
                        size: 16,
                        alignment: 16,
                        struct_layout: None,
                        elem_size: None,
                        gc_type: None,
                        gc_type_hybrid_full: None
                    },
                    _ => unimplemented!()
                }
            }
            // reference of any type
            MuType_::Ref(_) | MuType_::IRef(_) | MuType_::WeakRef(_) => BackendType {
                ty: ty.clone(),
                size: 8,
                alignment: 8,
                struct_layout: None,
                elem_size: None,
                gc_type: None,
                gc_type_hybrid_full: None
            },
            // pointer/opque ref
            MuType_::UPtr(_) |
            MuType_::UFuncPtr(_) |
            MuType_::FuncRef(_) |
            MuType_::ThreadRef |
            MuType_::StackRef => BackendType {
                ty: ty.clone(),
                size: 8,
                alignment: 8,
                struct_layout: None,
                elem_size: None,
                gc_type: None,
                gc_type_hybrid_full: None
            },
            // tagref
            MuType_::Tagref64 => BackendType {
                ty: ty.clone(),
                size: 8,
                alignment: 8,
                struct_layout: None,
                elem_size: None,
                gc_type: None,
                gc_type_hybrid_full: None
            },
            // floating point
            MuType_::Float => BackendType {
                ty: ty.clone(),
                size: 4,
                alignment: 4,
                struct_layout: None,
                elem_size: None,
                gc_type: None,
                gc_type_hybrid_full: None
            },
            MuType_::Double => BackendType {
                ty: ty.clone(),
                size: 8,
                alignment: 8,
                struct_layout: None,
                elem_size: None,
                gc_type: None,
                gc_type_hybrid_full: None
            },
            // array
            MuType_::Array(ref ele_ty, len) => {
                let ele_backend_ty = vm.get_backend_type_info(ele_ty.id());
                let elem_size = ele_backend_ty.size;
                let size = ele_backend_ty.size * len;
                let align = ele_backend_ty.alignment;

                // Acording to the AMD64 SYSV ABI Version 0.99.8,
                // a 'local or global array variable of at least 16 bytes ... always has
                // alignment of at least 16 bytes' However, if we apply this rule,
                // it will break 'Mu's array rule, hopefully C programs
                // won't care if we allocate a local or global which is incorrectly alligned
                // (A c function can't be sure a pointer to array that is passed to it is
                // a local or global so this is unlikely to break anything).

                BackendType {
                    ty: ty.clone(),
                    size,
                    alignment: align,
                    struct_layout: None,
                    elem_size: Some(elem_size),
                    gc_type: None,
                    gc_type_hybrid_full: None
                }
            }
            // struct
            MuType_::Struct(ref name) => {
                let read_lock = STRUCT_TAG_MAP.read().unwrap();
                let struc = read_lock.get(name).unwrap();
                let tys = struc.get_tys();

                trace!("layout struct: {}", struc);
                BackendType::layout_struct(ty, tys, vm)
            }
            // hybrid
            // - align is the most strict aligned element (from all fix tys and var ty)
            // - size is fixed tys size
            // - layout is fixed tys layout
            MuType_::Hybrid(ref name) => {
                let read_lock = HYBRID_TAG_MAP.read().unwrap();
                let hybrid = read_lock.get(name).unwrap();

                let fix_tys = hybrid.get_fix_tys();
                let var_ty = hybrid.get_var_ty();

                // treat fix_tys as struct
                let mut ret = BackendType::layout_struct(ty, fix_tys, vm);

                // treat var_ty as array (getting its alignment)
                let var_ele_ty = vm.get_backend_type_info(var_ty.id());
                let var_size = var_ele_ty.size;
                let var_align = var_ele_ty.alignment;
                ret.elem_size = Some(var_size);

                ret.alignment = lcm(ret.alignment, var_align);
                ret.size = align_up(ret.size, ret.alignment);

                ret
            }
            // void
            MuType_::Void => BackendType {
                ty: ty.clone(),
                size: 0,
                alignment: 1,
                struct_layout: None,
                elem_size: None,
                gc_type: None,
                gc_type_hybrid_full: None
            },
            // vector
            MuType_::Vector(_, _) => unimplemented!()
        }
    }

    /// layouts struct fields
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn layout_struct(main_ty: &P<MuType>, tys: &Vec<P<MuType>>, vm: &VM) -> BackendType {
        let (size, alignment, offsets) = BackendType::sequential_layout(tys, vm);

        BackendType {
            ty: main_ty.clone(),
            size,
            alignment,
            struct_layout: Some(offsets),
            elem_size: None,
            gc_type: None,
            gc_type_hybrid_full: None
        }
    }

    /// sequentially layout a few Mu types as if they are fields in a struct.
    /// Returns a triple of (size, alignment, offsets of each type)
    /// (when dealing with call convention, we use this function to layout stack arguments)
    pub fn sequential_layout(tys: &Vec<P<MuType>>, vm: &VM) -> (ByteSize, ByteSize, Vec<ByteSize>) {
        let mut offsets: Vec<ByteSize> = vec![];
        let mut cur: ByteSize = 0;
        let mut struct_align: ByteSize = 1;

        for ty in tys.iter() {
            let ty_info = vm.get_backend_type_info(ty.id());
            trace!("examining field: {}, {:?}", ty, ty_info);

            let align = ty_info.alignment;
            struct_align = lcm(struct_align, align);
            cur = align_up(cur, align);
            offsets.push(cur);
            trace!("aligned to {}", cur);

            cur += ty_info.size;
        }

        // if we need padding at the end
        let size = align_up(cur, struct_align);

        (size, struct_align, offsets)
    }
}

use std::fmt;
impl fmt::Display for BackendType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} bytes ({} bytes aligned), ",
            self.size,
            self.alignment
        ).unwrap();
        if self.struct_layout.is_some() {
            use utils::vec_utils;

            let layout = self.struct_layout.as_ref().unwrap();
            write!(f, "field offsets: ({})", vec_utils::as_str(layout)).unwrap();
        }

        Ok(())
    }
}

/// RegGroup describes register class
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RegGroup {
    /// general purpose register
    GPR,
    /// requires two general purpose register
    GPREX,
    /// floating point register
    FPR
}

rodal_enum!(RegGroup { GPR, GPREX, FPR });

impl RegGroup {
    /// gets RegGroup from a MuType
    pub fn get_from_ty(ty: &P<MuType>) -> RegGroup {
        match ty.v {
            // for now, only use 64bits registers
            MuType_::Int(len) if len <= 64 => RegGroup::GPR,
            MuType_::Int(len) if len == 128 => RegGroup::GPREX,

            MuType_::Ref(_) |
            MuType_::IRef(_) |
            MuType_::WeakRef(_) |
            MuType_::UPtr(_) |
            MuType_::ThreadRef |
            MuType_::StackRef |
            MuType_::Tagref64 |
            MuType_::FuncRef(_) |
            MuType_::UFuncPtr(_) => RegGroup::GPR,

            MuType_::Float => RegGroup::FPR,
            MuType_::Double => RegGroup::FPR,

            _ => unimplemented!()
        }
    }

    /// gets RegGroup from a Mu Value
    pub fn get_from_value(val: &P<Value>) -> RegGroup {
        RegGroup::get_from_ty(&val.ty)
    }
}

fn make_block_name(inst: &MuName, label: &str) -> MuName {
    Arc::new(format!("{}:{}", inst, label))
}
