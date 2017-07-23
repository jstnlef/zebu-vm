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

    assert_eq!(cur.exception_obj, unsafe { Address::zero() });

    // set exception obj using offset
    let tl_addr = unsafe { thread::muentry_get_thread_local() };
    let exc_obj_addr = tl_addr + *thread::EXCEPTION_OBJ_OFFSET;
    println!("storing exception obj Address::max() to {}", exc_obj_addr);
    unsafe { exc_obj_addr.store(usize::MAX) };

    println!("{}", cur);
    assert_eq!(cur.exception_obj, unsafe { Address::max() });
}
