// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use utils::Address;
use mu::runtime::mm;
use mu::runtime::thread;
use mu::runtime::thread::MuThread;
use mu::vm::VM;

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
    let allocator_addr = tl_addr + *thread::ALLOCATOR_OFFSET;
    assert_eq!(allocator_addr, Address::from_ptr(allocator_ptr));

    let native_sp_ptr  = &tl.native_sp_loc as *const Address;
    let native_sp_addr = tl_addr + *thread::NATIVE_SP_LOC_OFFSET;
    assert_eq!(native_sp_addr, Address::from_ptr(native_sp_ptr));

    let user_tls_ptr   = &tl.user_tls as *const Address;
    let user_tls_addr  = tl_addr + *thread::USER_TLS_OFFSET;
    assert_eq!(user_tls_addr, Address::from_ptr(user_tls_ptr));

    let exc_obj_ptr    = &tl.exception_obj as *const Address;
    let exc_obj_addr   = tl_addr + *thread::EXCEPTION_OBJ_OFFSET;
    assert_eq!(exc_obj_addr, Address::from_ptr(exc_obj_ptr));
}