use ast::ir::*;
use ast::ptr::*;

use utils::LinkedHashMap;
use utils::POINTER_SIZE;

type SlotID = usize;

// | previous frame ...
// |---------------
// | return address
// | old RBP        <- RBP
// | func ID
// | callee saved
// | spilled
// |---------------
// | alloca area

pub struct Frame {
    cur_slot_id: SlotID,
    cur_offset: isize, // offset to rbp
    
    allocated: LinkedHashMap<SlotID, FrameSlot>,
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            cur_slot_id: 0,
            cur_offset: -POINTER_SIZE * 2, // reserve for old RBP and func ID
            allocated: LinkedHashMap::new()
        }
    }
    
    pub fn alloc_slot_for_callee_saved_reg(&mut self, reg: P<Value>, vm: &VM) -> P<Value> {
        let slot = {
            let ret = FrameSlot {
                id: cur_slot_id,
                offset: cur_offset,
                value: reg.clone()
            };
            cur_slot_id += 1;
            offset -= vm.get_type_size(reg.id());
            ret
        };
        
        slot.make_memory_op(vm)
    }
    
    pub fn alloc_slot_for_spilling(&mut self, reg: P<Value>) -> P<Value> {
        unimplemented!()
    }
}

struct FrameSlot {
    id: SlotID,
    offset: isize,
    
    value: P<Value>
}

impl FrameSlot {
    #[cfg(target_arch = "x86_64")]
    pub fn make_memory_op(&self, vm: &VM) -> P<Value> {
        use compiler::backend::x86_64;
        unimplemented!()
//        P(Value{
//            hdr: MuEntityHeader::unnamed(vm.next_id()),
//            ty: reg.ty.clone(),
//            v: Value_::Memory(
//                MemoryLocation::Address{
//                    base: x86_64::RBP.clone(),
//                    offset: 
//                }
//            )
//        })
    }
}