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

// muentry_throw_exception in swap_stack_x64_sysV.S
// is like a special calling convention to throw_exception_internal
// in order to save all the callee saved registers at a known location

// normal calling convention:
// ---code---                                        ---stack---
// push caller saved                                 caller saved
// call                                              return addr
//          -> (in callee) push rbp                  old rbp
//                         mov  rsp -> rbp           callee saved
//                         push callee saved

// this function's calling convention
// ---code---                                        ---stack---
// push caller saved                                 caller saved
// call                                              return addr
//          -> (in asm)  push callee saved           all callee saved <- 2nd arg
//             (in rust) push rbp                    (by rust) old rbp
//                       mov  rsp -> rbp             (by rust) callee saved
//                       push callee saved

// we do not want to make any assumptionon  where rust saves rbp or callee saved
// so we save them by ourselves in assembly, and pass a pointer as 2nd argument

#[no_mangle]
#[allow(unreachable_code)]
// last_frame_callee_saved: a pointer passed from assembly, values of 6 callee_saved
// registers are layed out as rbx, rbp, r12-r15 (from low address to high address)
// and return address is put after 6 callee saved regsiters
pub extern fn throw_exception_internal(exception_obj: Address, last_frame_callee_saved: Address) -> ! {
    trace!("throwing exception: {}", exception_obj);

    trace!("callee saved registers of last frame is saved at {}", last_frame_callee_saved);
    inspect_nearby_address(last_frame_callee_saved, 8);
    
    let mut cur_thread = thread::MuThread::current_mut();
    // set exception object
    cur_thread.exception_obj = exception_obj;
    
    let cf_lock   = cur_thread.vm.compiled_funcs().read().unwrap();
    let func_lock = cur_thread.vm.funcs().read().unwrap();

    let rust_frame_return_addr = unsafe {last_frame_callee_saved.plus(POINTER_SIZE * x86_64::CALLEE_SAVED_GPRs.len()).load::<Address>()};
    trace!("return address   : 0x{:x} - throw instruction", rust_frame_return_addr);
    
    // the return address is within throwing frame
    let throw_frame_callsite = rust_frame_return_addr;
    let (throw_func, throw_fv) = find_func_for_address(&cf_lock, &func_lock, throw_frame_callsite);
    trace!("throwing fucntion: {}", throw_func);
    
    // skip to previous frame
    // this is the frame that throws the exception
    let previous_frame_rbp_loc = last_frame_callee_saved.plus(POINTER_SIZE);
    let rbp = unsafe {previous_frame_rbp_loc.load::<Address>()};
    trace!("rbp of previous frame is {} (last_frame_callee_saved {} + 8)", rbp, last_frame_callee_saved);
    
    // set cursor to throwing frame
    let mut cursor = FrameCursor {
        rbp: rbp,
        return_addr: unsafe {rbp.plus(POINTER_SIZE).load::<Address>()},
        func_id: throw_func,
        func_ver_id: throw_fv,
        callee_saved_locs: hashmap!{
            x86_64::RBX.id() => last_frame_callee_saved,
            x86_64::RBP.id() => previous_frame_rbp_loc,
            x86_64::R12.id() => last_frame_callee_saved.plus(POINTER_SIZE * 2),
            x86_64::R13.id() => last_frame_callee_saved.plus(POINTER_SIZE * 3),
            x86_64::R14.id() => last_frame_callee_saved.plus(POINTER_SIZE * 4),
            x86_64::R15.id() => last_frame_callee_saved.plus(POINTER_SIZE * 5),
        }
    };

    print_backtrace(throw_frame_callsite, cursor.clone());
    
    let mut callsite = rust_frame_return_addr;
    
    trace!("Stack Unwinding starts");
    loop {
        trace!("frame cursor: {}", cursor);

        // release the locks, and keep a clone of the frame
        // because we may improperly leave this function
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
                    info!("failed to find an entry for {} in current frame", reg_id);
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
                                info!("no {} value was saved along unwinding", $reg.name().unwrap());
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

                info!("going to restore thread to {} with RSP {}", dest_addr, rsp);
                unsafe {thread::exception_restore(dest_addr, array.as_ptr(), rsp)};
                
                unreachable!()
            }
        }
        trace!("didnt find a catch block");
        
        // keep unwinding
        callsite = cursor.return_addr;
        cursor.to_previous_frame(&cf_lock, &func_lock);
        trace!("cursor unwinds to previous frame: {}", cursor);        
    }
}

fn print_backtrace(callsite: Address, mut cursor: FrameCursor) {
    info!("Mu backtrace:");

    let cur_thread = thread::MuThread::current();

    let cf_lock   = cur_thread.vm.compiled_funcs().read().unwrap();
    let func_lock = cur_thread.vm.funcs().read().unwrap();

    let mut frame_count = 0;
    let mut callsite = callsite;

    loop {
        let func_start = {
            match cf_lock.get(&cursor.func_ver_id) {
                Some(rwlock_cf) => {
                    rwlock_cf.read().unwrap().start.to_address()
                },
                None => unsafe {Address::zero()}
            }
        };
        let func_name = cur_thread.vm.name_of(cursor.func_ver_id);

        info!("frame {:2}: 0x{:x} - {} (fid: #{}, fvid: #{}) at 0x{:x}", frame_count, func_start, func_name, cursor.func_id, cursor.func_ver_id, callsite);

        if cursor.has_previous_frame() {
            frame_count += 1;
            callsite = cursor.return_addr;

            cursor.to_previous_frame(&cf_lock, &func_lock);
        } else {
            break;
        }
    }

    info!("backtrace done.");
}

fn inspect_nearby_address(base: Address, n: isize) {
    let mut i = n;
    while i >= -n {
        unsafe {
            let addr = base.offset(i * POINTER_SIZE as isize);
            let val  = addr.load::<Word>();
            trace!("addr: 0x{:x} | val: 0x{:x} {}", addr, val, {if addr == base {"<- base"} else {""}});
        }
        i -= 1;
    }
}

#[derive(Clone)]
struct FrameCursor {
    rbp: Address,
    return_addr: Address,
    func_id: MuID,
    func_ver_id: MuID,
    callee_saved_locs: HashMap<MuID, Address>
}

impl fmt::Display for FrameCursor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\nFrameCursor{{").unwrap();
        writeln!(f, "  rbp=0x{:x}, return_addr=0x{:x}, func_id={}, func_version_id={}", self.rbp, self.return_addr, self.func_id, self.func_ver_id).unwrap();
        writeln!(f, "  callee_saved:").unwrap();
        for (reg, addr) in self.callee_saved_locs.iter() {
            writeln!(f, "    #{} at 0x{:x}", reg, addr).unwrap()
        }
        writeln!(f, "}}")
    }
}

impl FrameCursor {
    fn has_previous_frame(&self) -> bool {
        !self.return_addr.is_zero()
    }

    fn to_previous_frame(&mut self, cf: &RwLockReadGuard<HashMap<MuID, RwLock<CompiledFunction>>>, funcs: &RwLockReadGuard<HashMap<MuID, RwLock<MuFunction>>>) {
        // check if return_addr is valid
        // FIXME: should use a sentinel value here
        if self.return_addr.is_zero() {
            panic!("cannot go to previous frame (return address is zero)");
        }
        
        let previous_rbp = unsafe {self.rbp.load::<Address>()};
        let previous_return_addr = unsafe {previous_rbp.plus(POINTER_SIZE).load::<Address>()};
        let (previous_func, previous_fv_id) = find_func_for_address(cf, funcs, self.return_addr);
        
        self.rbp = previous_rbp;
        self.return_addr = previous_return_addr;
        self.func_id = previous_func;
        self.func_ver_id = previous_fv_id;
    }
}

fn find_func_for_address (cf: &RwLockReadGuard<HashMap<MuID, RwLock<CompiledFunction>>>, funcs: &RwLockReadGuard<HashMap<MuID, RwLock<MuFunction>>>, pc_addr: Address) -> (MuID, MuID) {
//    use std::ops::Deref;

//    trace!("trying to find FuncVersion for address 0x{:x}", pc_addr);
    for (_, func) in cf.iter() {
        let func = func.read().unwrap();

        let f = match funcs.get(&func.func_id) {
            Some(f) => f,
            None => panic!("failed to find func #{}", func.func_id)
        };
//        let f_lock = f.read().unwrap();
        
        let start = func.start.to_address();
        let end = func.end.to_address();
//        trace!("CompiledFunction: func={}, fv_id={}, start=0x{:x}, end=0x{:x}", f_lock.deref(), func.func_ver_id, start, end);
        
        // pc won't be the start of a function, but could be the end
        if pc_addr > start && pc_addr <= end {
//            trace!("Found CompiledFunction: func_id={}, fv_id={}", func.func_id, func.func_ver_id);
            return (func.func_id, func.func_ver_id);
        }
    }
    
    panic!("cannot find compiled function for pc 0x{:x}", pc_addr);
}
