use utils::Address;
use mu::runtime::thread;
use mu::runtime::thread::MuThread;
use mu::vm::VM;

use std::usize;
use std::sync::Arc;

#[test]
fn test_access_exception_obj() {
    let vm = Arc::new(VM::new());

    unsafe {
        MuThread::current_thread_as_mu_thread(Address::max(), vm.clone());
    }

    let cur = MuThread::current();
    println!("{}", cur);
    println!("reference = {:?}", cur as *const MuThread);

    assert_eq!(cur.exception_obj, unsafe {Address::zero()});

    // set exception obj using offset
    let tl_addr = unsafe {thread::muentry_get_thread_local()};
    let exc_obj_addr = tl_addr.plus(*thread::EXCEPTION_OBJ_OFFSET);
    println!("storing exception obj Address::max() to {}", exc_obj_addr);
    unsafe {exc_obj_addr.store(usize::MAX)};

    println!("{}", cur);
    assert_eq!(cur.exception_obj, unsafe {Address::max()});
}