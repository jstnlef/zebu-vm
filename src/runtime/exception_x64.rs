use ast::ir::*;
use compiler::machine_code::CompiledFunction;
use compiler::frame::*;
use compiler::backend::x86_64;
use utils::Address;
use utils::POINTER_SIZE;
use runtime::thread;

use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::collections::HashMap;

#[no_mangle]
pub extern fn mu_throw_exception(exception_obj: Address) {
    trace!("throwing exception: {}", exception_obj);
    
    let mut cur_thread = thread::MuThread::current_mut();
    // set exception object
    cur_thread.exception_obj = exception_obj;
    
    let cf_lock = cur_thread.vm.compiled_funcs().read().unwrap(); 
    
    // rbp of current frame (mu_throw_exception(), Rust frame)
    let rust_frame_rbp = unsafe {thread::get_current_frame_rbp()};
    let rust_frame_return_addr = unsafe {rust_frame_rbp.plus(POINTER_SIZE).load::<Address>()};
    
    // the return address is within throwing frame
    let throw_frame_callsite = rust_frame_return_addr;
    let throw_func_id = find_func_for_address(&cf_lock, throw_frame_callsite);
    
    // skip to previous frame
    // this is the frame that throws the exception
    let rbp = unsafe {rust_frame_rbp.load::<Address>()};
    
    // set cursor to throwing frame
    let mut cursor = FrameCursor {
        rbp: rbp,
        return_addr: unsafe {rbp.plus(POINTER_SIZE).load::<Address>()},
        func_id: throw_func_id,
        callee_saved_locs: HashMap::new()
    };
    
    loop {
        // get return address (the slot above RBP slot)
//        let return_addr = unsafe {rbp.plus(POINTER_SIZE).load::<Address>()};
        
        // check if return_addr is valid
        // FIXME: should use a sentinel value here
        if cursor.return_addr.is_zero() {
            panic!("cannot find exception catch block, throws by {}", throw_func_id);
        }
        
        let rwlock_cf = match cf_lock.get(&cursor.func_id) {
            Some(ret) => ret,
            None => panic!("cannot find compiled func with func_id {}, possibly didnt find the right frame for return address", cursor.func_id)
        };
        let rwlock_cf = rwlock_cf.read().unwrap();
        let ref frame = rwlock_cf.frame;
        
        // update callee saved register location
        for reg in x86_64::CALLEE_SAVED_GPRs.iter() {
            let reg_id = reg.id();
            if frame.allocated.contains_key(&reg_id) {
                let offset_from_rbp = frame.allocated.get(&reg_id).unwrap().offset;
                let reg_restore_addr = cursor.rbp.offset(offset_from_rbp);
                
                cursor.callee_saved_locs.insert(reg_id, reg_restore_addr);
            }
        }
        
        cursor.to_previous_frame(&cf_lock);
        
        // find exception block (if available)
    }
}

struct FrameCursor {
    rbp: Address,
    return_addr: Address,
    func_id: MuID,
    callee_saved_locs: HashMap<MuID, Address>
}

impl FrameCursor {
    fn to_previous_frame(&mut self, cf: &RwLockReadGuard<HashMap<MuID, RwLock<CompiledFunction>>>) {
        let previous_rbp = unsafe {self.rbp.load::<Address>()};
        let previous_return_addr = unsafe {previous_rbp.plus(POINTER_SIZE).load::<Address>()};
        let previous_func_id = find_func_for_address(cf, self.return_addr);
        
        self.rbp = previous_rbp;
        self.return_addr = previous_return_addr;
        self.func_id = previous_func_id;
    }
}

fn find_func_for_address (cf: &RwLockReadGuard<HashMap<MuID, RwLock<CompiledFunction>>>, pc_addr: Address) -> MuID {
    unimplemented!()
}