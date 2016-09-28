use utils::Address;
use runtime::thread;

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub extern fn mu_throw_exception(exception_obj: Address) {
    trace!("throwing exception: {}", exception_obj);
    
    thread::MuThread::current_mut().exception_obj = exception_obj;
    
    
}