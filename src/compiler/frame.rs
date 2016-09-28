use ast::ir::*;
use ast::ptr::*;
use ast::types::*;

use std::collections::HashMap;
use utils::POINTER_SIZE;
use vm::VM;

type SlotID = usize;

// | previous frame ...
// |---------------
// | return address
// | old RBP        <- RBP
// | callee saved
// | spilled
// |---------------
// | alloca area

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

#[derive(RustcEncodable, RustcDecodable)]
pub struct Frame {
    cur_slot_id: SlotID,
    cur_offset: isize, // offset to rbp
    
    allocated: HashMap<SlotID, FrameSlot>,
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            cur_slot_id: 0,
            cur_offset: - (POINTER_SIZE as isize * 1), // reserve for old RBP
            allocated: HashMap::new()
        }
    }
    
    pub fn alloc_slot_for_callee_saved_reg(&mut self, reg: P<Value>, vm: &VM) -> P<Value> {
        let slot = self.alloc_slot(&reg, vm);
        slot.make_memory_op(reg.ty.clone(), vm)
    }
    
    pub fn alloc_slot_for_spilling(&mut self, reg: P<Value>, vm: &VM) -> P<Value> {
        let slot = self.alloc_slot(&reg, vm);
        slot.make_memory_op(reg.ty.clone(), vm)
    }
    
    fn alloc_slot(&mut self, val: &P<Value>, vm: &VM) -> &FrameSlot {
        let id = self.cur_slot_id;
        let ret = FrameSlot {
            id: id,
            offset: self.cur_offset,
            value: val.clone()
        };
        
        self.cur_slot_id += 1;
        self.cur_offset -= vm.get_type_size(val.ty.id()) as isize;
        
        self.allocated.insert(id, ret);
        
        self.allocated.get(&id).unwrap()
    }
}

#[derive(RustcEncodable, RustcDecodable)]
struct FrameSlot {
    id: SlotID,
    offset: isize,
    
    value: P<Value>
}

impl FrameSlot {
    #[cfg(target_arch = "x86_64")]
    pub fn make_memory_op(&self, ty: P<MuType>, vm: &VM) -> P<Value> {
        use compiler::backend::x86_64;

        P(Value{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ty.clone(),
            v: Value_::Memory(
                MemoryLocation::Address{
                    base: x86_64::RBP.clone(),
                    offset: Some(Value::make_int_const(vm.next_id(), self.offset as u64)),
                    index: None,
                    scale: None
                }
            )
        })
    }
}