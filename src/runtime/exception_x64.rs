use ast::ir::*;
use compiler::machine_code::CompiledFunction;
use compiler::backend::x86_64;
use utils::Address;
use utils::Word;
use utils::POINTER_SIZE;
use runtime::thread;

use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::collections::HashMap;
use std::fmt;

#[no_mangle]
#[allow(unreachable_code)]
pub extern fn muentry_throw_exception(exception_obj: Address) {
    trace!("throwing exception: {}", exception_obj);
    
    let mut cur_thread = thread::MuThread::current_mut();
    // set exception object
    cur_thread.exception_obj = exception_obj;
    
    let cf_lock = cur_thread.vm.compiled_funcs().read().unwrap(); 
    
    // rbp of current frame (mu_throw_exception(), Rust frame)
    let rust_frame_rbp = unsafe {thread::get_current_frame_rbp()};
    trace!("current frame RBP: 0x{:x}", rust_frame_rbp);
    inspect_nearby_address(rust_frame_rbp, 5);    
    let rust_frame_return_addr = unsafe {rust_frame_rbp.plus(POINTER_SIZE).load::<Address>()};
    trace!("return address   : 0x{:x} - throw instruction", rust_frame_return_addr);
    
    // the return address is within throwing frame
    let throw_frame_callsite = rust_frame_return_addr;
    let (throw_func, throw_fv) = find_func_for_address(&cf_lock, throw_frame_callsite);
    trace!("throwing fucntion: {}", throw_func);
    
    // skip to previous frame
    // this is the frame that throws the exception
    let rbp = unsafe {rust_frame_rbp.load::<Address>()};
    
    // set cursor to throwing frame
    let mut cursor = FrameCursor {
        rbp: rbp,
        return_addr: unsafe {rbp.plus(POINTER_SIZE).load::<Address>()},
        func_id: throw_func,
        func_ver_id: throw_fv,
        callee_saved_locs: HashMap::new()
    };
    trace!("cursor at first Mu frame: {}", cursor);
    
    let mut callsite = rust_frame_return_addr;
    
    trace!("Stack Unwinding starts");
    loop {
        trace!("frame cursor: {}", cursor);
        // get return address (the slot above RBP slot)
//        let return_addr = unsafe {rbp.plus(POINTER_SIZE).load::<Address>()};
        
        // release the locks, and keep a clone of the frame
        // because we may improperly leave this function
        // FIXME: consider using Rust () -> ! to tell Rust compiler that we may not finish the func
        let frame = {
            let rwlock_cf = match cf_lock.get(&cursor.func_ver_id) {
                Some(ret) => ret,
                None => panic!("cannot find compiled func with func_id {}, possibly didnt find the right frame for return address", cursor.func_id)
            };
            let rwlock_cf = rwlock_cf.read().unwrap();
            rwlock_cf.frame.clone()
        };
        trace!("frame info: {}", frame);
        
        // update callee saved register location
        for reg in x86_64::CALLEE_SAVED_GPRs.iter() {
            let reg_id = reg.id();
            trace!("update callee saved register {}", reg_id);
            if frame.allocated.contains_key(&reg_id) {
                let offset_from_rbp = frame.allocated.get(&reg_id).unwrap().offset;
                let reg_restore_addr = cursor.rbp.offset(offset_from_rbp);
                
                trace!("update callee saved register {} with loc 0x{:x}", reg_id, reg_restore_addr);
                cursor.callee_saved_locs.insert(reg_id, reg_restore_addr);
            } else {
                // rbp won't find a location
                if reg_id == x86_64::RBP.id() {
                    
                } else {
                    warn!("failed to find an entry for it in current frame");
                }
            }
        }
        
        // find exception block - comparing callsite with frame info
        trace!("checking catch block: looking for callsite 0x{:x}", callsite);
        let exception_callsites = frame.get_exception_callsites();
        for &(ref possible_callsite, ref dest) in exception_callsites.iter() {
            let possible_callsite_addr = possible_callsite.to_address();
            trace!("..check {} at 0x{:x}", possible_callsite, possible_callsite_addr);
            
            if callsite == possible_callsite_addr {
                trace!("found catch block at {}", dest);
                // found an exception block
                let dest_addr = dest.to_address();
                
                // restore callee saved register and jump to dest_addr
                
                // prepare a plain array [rbx, rbp, r12, r13, r14, r15]
                macro_rules! unpack_callee_saved_from_cursor {
                    ($reg: expr) => {
                        match cursor.callee_saved_locs.get(&$reg.id()) {
                            Some(addr) => unsafe {addr.load::<Word>()},
                            None => {
                                warn!("no {} value was saved along unwinding, please check", $reg.name().unwrap());
                                0
                            }
                        }
                    }
                };
                
                let rbx = unpack_callee_saved_from_cursor!(x86_64::RBX);
                let r12 = unpack_callee_saved_from_cursor!(x86_64::R12);
                let r13 = unpack_callee_saved_from_cursor!(x86_64::R13);
                let r14 = unpack_callee_saved_from_cursor!(x86_64::R14);
                let r15 = unpack_callee_saved_from_cursor!(x86_64::R15);
                let rbp = cursor.rbp.as_usize() as Word;
                let array = vec![rbx, rbp, r12, r13, r14, r15];
                
                let rsp = cursor.rbp.offset(frame.cur_offset());
                
                unsafe {thread::exception_restore(dest_addr, array.as_ptr(), rsp)};
                
                unreachable!()
            }
        }
        trace!("didnt find a catch block");
        
        // keep unwinding
        callsite = cursor.return_addr;
        cursor.to_previous_frame(&cf_lock);
        trace!("cursor unwinds to previous frame: {}", cursor);        
    }
}

fn inspect_nearby_address(base: Address, n: isize) {
    let mut i = n;
    while i >= -n {
        unsafe {
            let addr = base.offset(i * POINTER_SIZE as isize);
            let val  = addr.load::<Word>();
            trace!("addr: 0x{:x} | val: 0x{:x}", addr, val);
        }
        i -= 1;
    }
}

struct FrameCursor {
    rbp: Address,
    return_addr: Address,
    func_id: MuID,
    func_ver_id: MuID,
    callee_saved_locs: HashMap<MuID, Address>
}

impl fmt::Display for FrameCursor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "FrameCursor{{").unwrap();
        writeln!(f, "  rbp=0x{:x}, return_addr=0x{:x}, func_id={}, func_version_id={}", self.rbp, self.return_addr, self.func_id, self.func_ver_id).unwrap();
        writeln!(f, "  callee_saved:").unwrap();
        for (reg, addr) in self.callee_saved_locs.iter() {
            writeln!(f, "    #{} at 0x{:x}", reg, addr).unwrap()
        }
        writeln!(f, "}}")
    }
}

impl FrameCursor {
    fn to_previous_frame(&mut self, cf: &RwLockReadGuard<HashMap<MuID, RwLock<CompiledFunction>>>) {
        // check if return_addr is valid
        // FIXME: should use a sentinel value here
        if self.return_addr.is_zero() {
            panic!("cannot go to previous frame (return address is zero)");
        }
        
        let previous_rbp = unsafe {self.rbp.load::<Address>()};
        let previous_return_addr = unsafe {previous_rbp.plus(POINTER_SIZE).load::<Address>()};
        let (previous_func, previous_fv_id) = find_func_for_address(cf, self.return_addr);
        
        self.rbp = previous_rbp;
        self.return_addr = previous_return_addr;
        self.func_id = previous_func;
        self.func_ver_id = previous_fv_id;
    }
}

fn find_func_for_address (cf: &RwLockReadGuard<HashMap<MuID, RwLock<CompiledFunction>>>, pc_addr: Address) -> (MuID, MuID) {
    trace!("trying to find FuncVersion for address 0x{:x}", pc_addr);
    for (_, func) in cf.iter() {
        let func = func.read().unwrap();
        
        let start = func.start.to_address();
        let end = func.end.to_address();
        trace!("CompiledFunction: func_id={}, fv_id={}, start=0x{:x}, end=0x{:x}", func.func_id, func.func_ver_id, start, end);
        
        // pc won't be the start of a function, but could be the end
        if pc_addr > start && pc_addr <= end {
            trace!("Found CompiledFunction: func_id={}, fv_id={}", func.func_id, func.func_ver_id);
            return (func.func_id, func.func_ver_id);
        }
    }
    
    panic!("cannot find compiled function for pc 0x{:x}", pc_addr);
}
