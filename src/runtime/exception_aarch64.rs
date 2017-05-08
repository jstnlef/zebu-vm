#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

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
    unimplemented!();
}

fn print_backtrace(callsite: Address, mut cursor: FrameCursor) {
    unimplemented!();
}

fn inspect_nearby_address(base: Address, n: isize) {
    unimplemented!();
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
        unimplemented!();
    }
}

impl FrameCursor {
    fn has_previous_frame(&self, cf: &RwLockReadGuard<HashMap<MuID, RwLock<CompiledFunction>>>,
                          funcs: &RwLockReadGuard<HashMap<MuID, RwLock<MuFunction>>>) -> bool {
        unimplemented!();
    }

    fn to_previous_frame(&mut self, cf: &RwLockReadGuard<HashMap<MuID, RwLock<CompiledFunction>>>, funcs: &RwLockReadGuard<HashMap<MuID, RwLock<MuFunction>>>) {
        unimplemented!();
    }
}

const TRACE_FIND_FUNC : bool = false;

#[allow(unused_imports)]
fn find_func_for_address (cf: &RwLockReadGuard<HashMap<MuID, RwLock<CompiledFunction>>>,
                          funcs: &RwLockReadGuard<HashMap<MuID, RwLock<MuFunction>>>,
                          pc_addr: Address) -> Option<(MuID, MuID)> {

    unimplemented!();
}