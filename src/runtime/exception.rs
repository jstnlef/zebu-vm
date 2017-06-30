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

use compiler::backend::*;
use utils::Address;
use utils::POINTER_SIZE;
use runtime::*;

// muentry_throw_exception should call this function,
// With the first argument being the address of the exception object,
// And the second argument should be point to the base of the call frame of muentry_throw_exception,
// which saves every callee saved register (note this frame will be modified by this function).
// e.g. on aarch64 (where the values are the value of the registers immediatly before the first instruction in muentry_throw_exception is executed):
//                  Return Address              (value of X30)
// frame_cursor --> Frame Pointer               (value of X29)
//                  First Callee Saved Register (value of X19)
//                  .........
//                  Last Callee Saved Register  (value of D15)
// The actual offsets of the callee saved registers is determined by get_callee_saved_offset (relative to frame_cursor)
// The location of Frame Pointer and Return address is architecture dependent
// (and are accesed by get/set_return_address and get/set_previous_frame and may be passed real frame pointers or the frame cursor)

#[no_mangle]
pub extern fn throw_exception_internal(exception_obj: Address, frame_cursor: Address) -> !  {
    trace!("throwing exception: {}", exception_obj);

    if cfg!(debug_assertions) {
        trace!("Initial Frame: ");
        print_frame(frame_cursor);
    }

    let ref mut cur_thread = thread::MuThread::current_mut();
    // set exception object
    cur_thread.exception_obj = exception_obj;
    let ref vm = cur_thread.vm;

    let mut current_frame_pointer = frame_cursor; // this will be 16 bytes bellow the bottom of the previous frame
    let mut callsite = get_return_address(current_frame_pointer);
    let mut previous_frame_pointer = get_previous_frame_pointer(current_frame_pointer); // thrower::fp, the starting point of the previous frame

    // acquire lock for exception table
    let compiled_exception_table = vm.compiled_exception_table.read().unwrap();

    loop {
        // Lookup the table for the callsite
        trace!("Callsite: 0x{:x}", callsite);
        trace!("\tprevious_frame_pointer: 0x{:x}", previous_frame_pointer);
        trace!("\tcurrent_frame_pointer: 0x{:x}", current_frame_pointer);

        let &(catch_address, compiled_func) = {
            let table_entry = compiled_exception_table.get(&callsite);

            if table_entry.is_none() {
                error!("Cannot find Mu callsite (i.e. we have reached a native frame), either there isn't a catch block to catch the exception or your catch block is above a native function call");
                print_backtrace(frame_cursor);
                // The above function will not return
            }

            table_entry.unwrap()
        };

        // Check for a catch block at this callsite (there won't be one on the first iteration of this loop)
        if !catch_address.is_zero() {
            trace!("Found catch block: 0x{:x}", catch_address);
            let sp = get_previous_stack_pointer(current_frame_pointer);
            trace!("\tRestoring SP to: 0x{:x}", sp);

            if cfg!(debug_assertions) {
                trace!("Restoring frame: ");
                print_frame(frame_cursor);
            }

            // Found a catch block, branch to it
            drop(compiled_exception_table);    // drop the lock first
            unsafe { thread::exception_restore(catch_address, frame_cursor.to_ptr(), sp); }
        }

        // Restore callee saved registers
        unsafe {
            let ref cf = *compiled_func;
            let ref callee_saved = cf.frame.callee_saved;
            for (target_offset, source_offset) in callee_saved {
                // *(frame_cursor + target_offset) = *(frame_pointer + source_offset)
                let val = previous_frame_pointer.offset(*source_offset).load::<Address>();
                frame_cursor.offset(*target_offset).store::<Address>(val);
            }
        }

        // Move up to the previous frame
        current_frame_pointer = previous_frame_pointer;
        previous_frame_pointer = get_previous_frame_pointer(current_frame_pointer);

        // Restore the callsite
        callsite = get_return_address(current_frame_pointer);
        set_return_address(frame_cursor, callsite);
        set_previous_frame_pointer(frame_cursor, previous_frame_pointer);
    }
}

fn print_frame(base: Address) {
    let top = 2;
    let bottom = -(CALLEE_SAVED_COUNT as isize);
    for i in (bottom .. top).rev() {
        unsafe {
            let addr = base.offset(i * POINTER_SIZE as isize);
            let val  = addr.load::<Word>();
            trace!("\taddr: 0x{:x} | val: 0x{:x} {}", addr, val, {if addr == base {"<- base"} else {""}});
        }

    }
}

// This function may segfault or panic when it reaches the bottom of the stack
// (TODO: Determine where the bottom is without segfaulting)
fn print_backtrace(base: Address) -> !{
    error!("BACKTRACE: ");

    let cur_thread = thread::MuThread::current();
    let ref vm = cur_thread.vm;

    let mut frame_pointer = base;
    let mut frame_count = 0;

    let compiled_exception_table = vm.compiled_exception_table.read().unwrap();

    loop {
        let callsite = get_return_address(frame_pointer);

        if compiled_exception_table.contains_key(&callsite) {
            let &(_, compiled_func_ptr) = compiled_exception_table.get(&callsite).unwrap();

            unsafe {
                let ref compiled_func = *compiled_func_ptr;

                error!("\tframe {:2}: 0x{:x} - {} (fid: #{}, fvid: #{}) at 0x{:x}", frame_count,
                compiled_func.start.to_address(), vm.name_of(compiled_func.func_id),
                compiled_func.func_id, compiled_func.func_ver_id, callsite);
            }
        } else {
            let (func_name, func_start) = get_function_info(callsite);
            error!("\tframe {:2}: 0x{:x} - {} at 0x{:x}", frame_count, func_start, func_name, callsite);
        }

        frame_pointer = get_previous_frame_pointer(frame_pointer);
        if frame_pointer.is_zero() {
            panic!("Uncaught Mu Exception");
        }
        frame_count += 1;
    }
}