use ast::ir::*;
use compiler::machine_code::CompiledFunction;
use compiler::backend::aarch64;
use utils::Address;
use utils::Word;
use utils::POINTER_SIZE;
use runtime::thread;

use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::collections::HashMap;
use std::fmt;

// muentry_throw_exception in swap_stack_aarch64_sysv.S
// is like a special calling convention to throw_exception_internal
// in order to save all the callee saved registers at a known location

// normal calling convention:
// ---code---                                        ---stack---
// push caller saved                                 caller saved
// call
//          -> (in callee) push LR, FP               LR, old FP
//                         MOV  SP -> FP            callee saved
//                         push callee saved

// this function's calling convention
// ---code---                                        ---stack---
// push caller saved                                 caller saved
// call                                              LR, old FP
//          -> (in asm)  push callee saved           all callee saved <- 2nd arg
//             (in rust) push LR, FP                 (by rust) LR, old FP
//                       mov  SP -> FP             (by rust) callee saved
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
    inspect_higher_address(last_frame_callee_saved, 20);

    let mut cur_thread = thread::MuThread::current_mut();
    // set exception object
    cur_thread.exception_obj = exception_obj;

    let cf_lock   = cur_thread.vm.compiled_funcs().read().unwrap();
    let func_lock = cur_thread.vm.funcs().read().unwrap();

    let rust_frame_return_addr = unsafe {last_frame_callee_saved.plus(POINTER_SIZE * 19).load::<Address>()};
    trace!("return address   : 0x{:x} - throw instruction", rust_frame_return_addr);

    // the return address is within throwing frame
    let throw_frame_callsite = rust_frame_return_addr;
    let (throw_func, throw_fv) = find_func_for_address(&cf_lock, &func_lock, throw_frame_callsite).unwrap();
    trace!("throwing fucntion: {}", throw_func);

    // skip to previous frame
    // this is the frame that throws the exception
    let previous_frame_fp_loc = last_frame_callee_saved.plus(POINTER_SIZE * 18);
    let fp = unsafe {previous_frame_fp_loc.load::<Address>()};
    trace!("FP of previous frame is {} (last_frame_callee_saved {} + 144)", fp, last_frame_callee_saved);

    // set cursor to throwing frame
    let mut cursor = FrameCursor {
        fp: fp,
        return_addr: unsafe {fp.plus(POINTER_SIZE).load::<Address>()},
        func_id: throw_func,
        func_ver_id: throw_fv,
        callee_saved_locs: hashmap!{
            //aarch64::LR.id() => last_frame_callee_saved.plus(POINTER_SIZE * 19),
            //aarch64::FP.id() => last_frame_callee_saved.plus(POINTER_SIZE * 18),

            aarch64::D8.id() => last_frame_callee_saved.plus(POINTER_SIZE * 17),
            aarch64::D9.id() => last_frame_callee_saved.plus(POINTER_SIZE * 16),
            aarch64::D10.id() => last_frame_callee_saved.plus(POINTER_SIZE * 15),
            aarch64::D11.id() => last_frame_callee_saved.plus(POINTER_SIZE * 14),
            aarch64::D12.id() => last_frame_callee_saved.plus(POINTER_SIZE * 13),
            aarch64::D13.id() => last_frame_callee_saved.plus(POINTER_SIZE * 12),
            aarch64::D14.id() => last_frame_callee_saved.plus(POINTER_SIZE * 11),
            aarch64::D15.id() => last_frame_callee_saved.plus(POINTER_SIZE * 10),

            aarch64::X19.id() => last_frame_callee_saved.plus(POINTER_SIZE * 9),
            aarch64::X20.id() => last_frame_callee_saved.plus(POINTER_SIZE * 8),
            aarch64::X21.id() => last_frame_callee_saved.plus(POINTER_SIZE * 7),
            aarch64::X22.id() => last_frame_callee_saved.plus(POINTER_SIZE * 6),
            aarch64::X23.id() => last_frame_callee_saved.plus(POINTER_SIZE * 5),
            aarch64::X24.id() => last_frame_callee_saved.plus(POINTER_SIZE * 4),
            aarch64::X25.id() => last_frame_callee_saved.plus(POINTER_SIZE * 3),
            aarch64::X26.id() => last_frame_callee_saved.plus(POINTER_SIZE * 2),
            aarch64::X27.id() => last_frame_callee_saved.plus(POINTER_SIZE * 1),
            aarch64::X28.id() => last_frame_callee_saved.plus(POINTER_SIZE * 0),
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

                let x28 = unpack_callee_saved_from_cursor!(aarch64::X28);
                let x27 = unpack_callee_saved_from_cursor!(aarch64::X27);
                let x26 = unpack_callee_saved_from_cursor!(aarch64::X26);
                let x25 = unpack_callee_saved_from_cursor!(aarch64::X25);
                let x24 = unpack_callee_saved_from_cursor!(aarch64::X24);
                let x23 = unpack_callee_saved_from_cursor!(aarch64::X23);
                let x22 = unpack_callee_saved_from_cursor!(aarch64::X22);
                let x21 = unpack_callee_saved_from_cursor!(aarch64::X21);
                let x20 = unpack_callee_saved_from_cursor!(aarch64::X20);
                let x19 = unpack_callee_saved_from_cursor!(aarch64::X19);
                let d15 = unpack_callee_saved_from_cursor!(aarch64::D15);
                let d14 = unpack_callee_saved_from_cursor!(aarch64::D14);
                let d13 = unpack_callee_saved_from_cursor!(aarch64::D13);
                let d12 = unpack_callee_saved_from_cursor!(aarch64::D12);
                let d11 = unpack_callee_saved_from_cursor!(aarch64::D11);
                let d10 = unpack_callee_saved_from_cursor!(aarch64::D10);
                let d9 = unpack_callee_saved_from_cursor!(aarch64::D9);
                let d8 = unpack_callee_saved_from_cursor!(aarch64::D8);
                let fp = cursor.fp.as_usize() as Word;
                let lr = cursor.return_addr.as_usize() as Word;
                let array = vec![x28, x27, x26, x25, x24, x23, x22, x21, x20, x19, d15, d14, d13, d12, d11, d10, d9, d8, fp, lr];

                let sp = cursor.fp.offset(- (frame.cur_size() as isize));

                info!("going to restore thread to {} with SP {}", dest_addr, sp);
                unsafe {thread::exception_restore(dest_addr, array.as_ptr(), sp)};

                unreachable!()
            }
        }
        trace!("didnt find a catch block");

        // update callee saved register location
        for reg in aarch64::CALLEE_SAVED_REGS.iter() {
            let reg_id = reg.id();
            trace!("update callee saved register {}", reg.name().unwrap());
            if frame.allocated.contains_key(&reg_id) {
                let offset_from_fp = frame.allocated.get(&reg_id).unwrap().offset;
                let reg_restore_addr = cursor.fp.offset(offset_from_fp);

                trace!("update callee saved register {} with loc 0x{:x}", reg.name().unwrap(), reg_restore_addr);
                cursor.callee_saved_locs.insert(reg_id, reg_restore_addr);
            } else {
                info!("failed to find an entry for {} in current frame", reg.name().unwrap());
            }
        }

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
        let func_name = cur_thread.vm.name_of(cursor.func_id);

        info!("frame {:2}: 0x{:x} - {} (fid: #{}, fvid: #{}) at 0x{:x}", frame_count, func_start, func_name, cursor.func_id, cursor.func_ver_id, callsite);

        if cursor.has_previous_frame(&cf_lock, &func_lock) {
            frame_count += 1;
            callsite = cursor.return_addr;

            cursor.to_previous_frame(&cf_lock, &func_lock);
        } else {
            break;
        }
    }

    info!("backtrace done.");
}

fn inspect_higher_address(base: Address, n: isize) {
    let mut i = n;
    while i >= 0 {
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
    fp: Address,
    return_addr: Address,
    func_id: MuID,
    func_ver_id: MuID,
    callee_saved_locs: HashMap<MuID, Address>
}

impl fmt::Display for FrameCursor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\nFrameCursor{{").unwrap();
        writeln!(f, "  FP=0x{:x}, return_addr=0x{:x}, func_id={}, func_version_id={}", self.fp, self.return_addr, self.func_id, self.func_ver_id).unwrap();
        writeln!(f, "  callee_saved:").unwrap();
        for (reg, addr) in self.callee_saved_locs.iter() {
            let val = unsafe {addr.load::<u64>()};
            writeln!(f, "    {} at 0x{:x} (value=0x{:x})", aarch64::get_register_from_id(*reg), addr, val).unwrap()
        }
        writeln!(f, "}}")
    }
}

impl FrameCursor {
    fn has_previous_frame(&self, cf: &RwLockReadGuard<HashMap<MuID, RwLock<CompiledFunction>>>,
                          funcs: &RwLockReadGuard<HashMap<MuID, RwLock<MuFunction>>>) -> bool {
        !self.return_addr.is_zero() && find_func_for_address(cf, funcs, self.return_addr).is_some()
    }

    fn to_previous_frame(&mut self, cf: &RwLockReadGuard<HashMap<MuID, RwLock<CompiledFunction>>>, funcs: &RwLockReadGuard<HashMap<MuID, RwLock<MuFunction>>>) {
        // check if return_addr is valid
        // FIXME: should use a sentinel value here
        if self.return_addr.is_zero() {
            panic!("cannot go to previous frame (return address is zero)");
        }

        let previous_fp = unsafe {self.fp.load::<Address>()};
        let previous_return_addr = unsafe {previous_fp.plus(POINTER_SIZE).load::<Address>()};
        let (previous_func, previous_fv_id) = find_func_for_address(cf, funcs, self.return_addr).unwrap();

        self.fp = previous_fp;
        self.return_addr = previous_return_addr;
        self.func_id = previous_func;
        self.func_ver_id = previous_fv_id;
    }
}

const TRACE_FIND_FUNC : bool = false;

#[allow(unused_imports)]
fn find_func_for_address (cf: &RwLockReadGuard<HashMap<MuID, RwLock<CompiledFunction>>>,
                          funcs: &RwLockReadGuard<HashMap<MuID, RwLock<MuFunction>>>,
                          pc_addr: Address) -> Option<(MuID, MuID)> {
    use std::ops::Deref;

    if TRACE_FIND_FUNC {
        trace!("trying to find FuncVersion for address 0x{:x}", pc_addr);
    }
    for (_, func) in cf.iter() {
        let func = func.read().unwrap();

        let start = func.start.to_address();
        let end = func.end.to_address();

        if TRACE_FIND_FUNC {
            let f = match funcs.get(&func.func_id) {
                Some(f) => f,
                None => panic!("failed to find func #{}", func.func_id)
            };
            let f_lock = f.read().unwrap();
            trace!("CompiledFunction: func={}, fv_id={}, start=0x{:x}, end=0x{:x}", f_lock.deref(), func.func_ver_id, start, end);
        }

        // pc won't be the start of a function, but could be the end
        if pc_addr > start && pc_addr <= end {
            if TRACE_FIND_FUNC {
                trace!("Found CompiledFunction: func_id={}, fv_id={}", func.func_id, func.func_ver_id);
            }
            return Some((func.func_id, func.func_ver_id));
        }
    }

    None
}
