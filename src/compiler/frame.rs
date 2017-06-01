use ast::ir::*;
use ast::ptr::*;
use ast::types::*;
use runtime::ValueLocation;

use std::fmt;
use std::collections::HashMap;
use vm::VM;

/// Frame serves two purposes:
/// * it manages stack allocation that are known statically (such as callee saved,
///   spilled registers)
/// * it also stores exception table for a given function, used for exception handling at runtime

/// Mu frame layout is compatible with C ABI

/// on x64
/// | previous frame ...
/// |---------------
/// | return address
/// | old RBP        <- RBP
/// | callee saved
/// | spilled
/// |---------------
/// | alloca area (not implemented)

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct Frame {
    /// function version for this frame
    func_ver_id: MuID,
    /// current offset to frame base pointer
    cur_offset: isize,
    /// arguments passed to this function by registers (used for validating register allocation)
    pub argument_by_reg: HashMap<MuID, P<Value>>,
    /// arguments passed to this function by stack (used for validating register allocation)
    pub argument_by_stack: HashMap<MuID, P<Value>>,
    /// allocated frame location for Mu Values
    pub allocated: HashMap<MuID, FrameSlot>,
    /// all the exception callsites in this frame as pairs of (callsite, destination address)
    exception_callsites: Vec<(ValueLocation, ValueLocation)>
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\nFrame for FuncVer {} {{", self.func_ver_id).unwrap();
        writeln!(f, "  allocated slots:").unwrap();
        for slot in self.allocated.values() {
            writeln!(f, "    {}", slot).unwrap();
        }
        writeln!(f, "  exception callsites:").unwrap();
        for &(ref callsite, ref dest) in self.exception_callsites.iter() {
            writeln!(f, "    callsite: {} -> {}", callsite, dest).unwrap()
        }
        writeln!(f, "  cur offset: {}", self.cur_offset).unwrap();
        writeln!(f, "}}")
    }
}

impl Frame {
    /// creates a new Frame
    pub fn new(func_ver_id: MuID) -> Frame {
        Frame {
            func_ver_id: func_ver_id,
            cur_offset: 0,
            argument_by_reg: HashMap::new(),
            argument_by_stack: HashMap::new(),

            allocated: HashMap::new(),
            exception_callsites: vec![]
        }
    }

    /// returns current size,
    /// which is always a multiple of 16 bytes for x64/aarch64 (alignment requirement)
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    pub fn cur_size(&self) -> usize {
        // frame size is a multiple of 16 bytes
        let size = self.cur_offset.abs() as usize;

        // align size to a multiple of 16 bytes
        let size = (size + 16 - 1) & !(16 - 1);

        debug_assert!(size % 16 == 0);

        size
    }

    /// adds a record of a Mu value argument passed in a certain register
    pub fn add_argument_by_reg(&mut self, temp: MuID, reg: P<Value>) {
        self.argument_by_reg.insert(temp, reg);
    }

    /// adds a record of a Mu value argumetn passed on stack
    pub fn add_argument_by_stack(&mut self, temp: MuID, stack_slot: P<Value>) {
        self.argument_by_stack.insert(temp, stack_slot);
    }

    /// allocates next stack slot for a callee saved register, and returns
    /// a memory operand representing the stack slot
    pub fn alloc_slot_for_callee_saved_reg(&mut self, reg: P<Value>, vm: &VM) -> P<Value> {
        let slot = self.alloc_slot(&reg, vm);

        slot.make_memory_op(reg.ty.clone(), vm)
    }

    /// removes the record for a callee saved register
    /// We allocate stack slots for all the callee saved regsiter, and later
    /// remove slots for those registers that are not actually used
    pub fn remove_record_for_callee_saved_reg(&mut self, reg: MuID) {
        self.allocated.remove(&reg);
    }

    /// allocates next stack slot for a spilled register, and returns
    /// a memory operand representing the stack slot
    pub fn alloc_slot_for_spilling(&mut self, reg: P<Value>, vm: &VM) -> P<Value> {
        let slot = self.alloc_slot(&reg, vm);
        slot.make_memory_op(reg.ty.clone(), vm)
    }

    /// gets exception callsites for this frame
    pub fn get_exception_callsites(&self) -> &Vec<(ValueLocation, ValueLocation)> {
        &self.exception_callsites
    }

    /// adds an exception callsite for this frame
    pub fn add_exception_callsite(&mut self, callsite: ValueLocation, dest: ValueLocation) {
        trace!("add exception callsite: {} to dest {}", callsite, dest);
        self.exception_callsites.push((callsite, dest));
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn alloc_slot(&mut self, val: &P<Value>, vm: &VM) -> &FrameSlot {
        // base pointer is 16 bytes aligned, we are offsetting from base pointer
        // every value should be properly aligned

        let backendty = vm.get_backend_type_info(val.ty.id());
        // asserting that the alignment is no larger than 16 bytes, otherwise
        // we need to adjust offset in a different way
        if backendty.alignment > 16 {
            unimplemented!()
        }

        self.cur_offset -= backendty.size as isize;

        {
            // if alignment doesnt satisfy, make adjustment
            let abs_offset = self.cur_offset.abs() as usize;
            if abs_offset % backendty.alignment != 0 {
                use utils::math;
                let abs_offset = math::align_up(abs_offset, backendty.alignment);

                self.cur_offset = -(abs_offset as isize);
            }
        }

        let id = val.id();
        let ret = FrameSlot {
            offset: self.cur_offset,
            value: val.clone(),
        };

        self.allocated.insert(id, ret);
        self.allocated.get(&id).unwrap()
    }
}

/// FrameSlot presents a Value stored in a certain frame location
#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct FrameSlot {
    /// location offset from current base pointer
    pub offset: isize,
    /// Mu value that resides in this location
    pub value: P<Value>,
}

impl fmt::Display for FrameSlot {
    #[cfg(target_arch = "x86_64")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(RBP): {}", self.offset, self.value)
    }

    #[cfg(target_arch = "aarch64")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[FP, #{}]: {}", self.offset, self.value)
    }
}

impl FrameSlot {
    /// generates a memory operand for this frame slot
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
    /// generates a memory operand for this frame slot
    #[cfg(target_arch = "aarch64")]
    pub fn make_memory_op(&self, ty: P<MuType>, vm: &VM) -> P<Value> {
        use compiler::backend::aarch64;

        P(Value{
            hdr: MuEntityHeader::unnamed(vm.next_id()),
            ty: ty.clone(),
            v: Value_::Memory(
                MemoryLocation::VirtualAddress{
                    base: aarch64::FP.clone(),
                    offset: Some(Value::make_int_const(vm.next_id(), self.offset as u64)),
                    scale: 1,
                    signed: false
                }
            )
        })
    }
}
