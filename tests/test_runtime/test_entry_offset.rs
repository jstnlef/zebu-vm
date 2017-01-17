use utils::Address;
use mu::runtime::mm;
use mu::runtime::thread;
use mu::runtime::thread::MuThread;
use mu::vm::VM;

use std::usize;
use std::sync::Arc;

#[test]
fn test_muthread_entry_offset() {
    let vm = Arc::new(VM::new());

    unsafe {
        MuThread::current_thread_as_mu_thread(Address::max(), vm.clone());
    }

    let tl : &MuThread = MuThread::current();

    let tl_ptr  = tl as *const MuThread;
    let tl_addr = unsafe {thread::muentry_get_thread_local()};
    assert_eq!(tl_addr, Address::from_ptr(tl_ptr));

    let allocator_ptr  = &tl.allocator as *const mm::Mutator;
    let allocator_addr = tl_addr.plus(*thread::ALLOCATOR_OFFSET);
    assert_eq!(allocator_addr, Address::from_ptr(allocator_ptr));

    let native_sp_ptr  = &tl.native_sp_loc as *const Address;
    let native_sp_addr = tl_addr.plus(*thread::NATIVE_SP_LOC_OFFSET);
    assert_eq!(native_sp_addr, Address::from_ptr(native_sp_ptr));

    let user_tls_ptr   = &tl.user_tls as *const Address;
    let user_tls_addr  = tl_addr.plus(*thread::USER_TLS_OFFSET);
    assert_eq!(user_tls_addr, Address::from_ptr(user_tls_ptr));

    let exc_obj_ptr    = &tl.exception_obj as *const Address;
    let exc_obj_addr   = tl_addr.plus(*thread::EXCEPTION_OBJ_OFFSET);
    assert_eq!(exc_obj_addr, Address::from_ptr(exc_obj_ptr));
}