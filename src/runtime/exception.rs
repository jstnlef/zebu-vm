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
use std::collections::HashMap;
use std::ops::Deref;
use compiler::machine_code::CompiledCallsite;
use runtime::*;
use log;

/// runtime function to deal with exception (unwind stack, find catch block, and restore)
/// This function is called by muentry_throw_exception() which gets emitted for THROW instruction
/// With the first argument being the address of the exception object, and the second argument
/// should be point to the base of the call frame of muentry_throw_exception,
/// which saves every callee saved register (note this frame will be modified by this function).
/// e.g. on aarch64 (where the values are the value of the registers immediately before the first
/// instruction in muentry_throw_exception is executed):
///                  Return Address              (value of X30)
/// frame_cursor --> Frame Pointer               (value of X29)
///                  First Callee Saved Register (value of X19)
///                  .........
///                  Last Callee Saved Register  (value of D15)
/// The actual offsets of the callee saved registers is determined by get_callee_saved_offset
/// (relative to frame_cursor)
/// The location of Frame Pointer and Return address is architecture dependent
/// (and are accessed by get/set_return_address and get/set_previous_frame and may be passed
/// real frame pointers or the frame cursor)
#[no_mangle]
pub extern "C" fn throw_exception_internal(exception_obj: Address, frame_cursor: Address) -> ! {
    debug!("throwing exception: {}", exception_obj);

    if cfg!(debug_assertions) {
        trace!("Initial Frame: ");
        print_frame(frame_cursor);
    }

    let ref mut cur_thread = thread::MuThread::current_mut();

    // set exception object (the catch block will have a landing pad to fetch this object)
    cur_thread.exception_obj = exception_obj;

    let ref vm = cur_thread.vm;
    // this will be 16 bytes bellow the bottom of the previous frame
    let mut current_frame_pointer = frame_cursor;
    let mut callsite = get_return_address(current_frame_pointer);
    // thrower's fp, the starting point of the previous frame
    let mut previous_frame_pointer = get_previous_frame_pointer(current_frame_pointer);
    // the address of the catch block
    let catch_address;
    // the stack pointer to restore to
    let sp;
    {
        // acquire lock for exception table
        let compiled_callsite_table = vm.compiled_callsite_table().read().unwrap();

        print_backtrace(frame_cursor, compiled_callsite_table.deref());
        loop {

            // Lookup the table for the callsite
            trace!("Callsite: 0x{:x}", callsite);
            trace!("\tprevious_frame_pointer: 0x{:x}", previous_frame_pointer);
            trace!("\tcurrent_frame_pointer: 0x{:x}", current_frame_pointer);

            let callsite_info = {
                let table_entry = compiled_callsite_table.get(&callsite);

                if table_entry.is_none() {
                    // we are not dealing with native frames for unwinding stack
                    // See Issue #42
                    error!(
                        "Cannot find Mu callsite (i.e. we have reached a native frame), \
                         either there isn't a catch block to catch the exception or \
                         your catch block is above a native function call"
                    );
                    panic!("Uncaught Mu Exception");
                }
                table_entry.unwrap()
            };

            // Check for a catch block at this callsite
            if callsite_info.exceptional_destination.is_some() {
                catch_address = callsite_info.exceptional_destination.unwrap();
                debug!("Found catch block: 0x{:x} - {}", catch_address,
                    get_symbol_name(catch_address));
                sp = get_previous_stack_pointer(
                    current_frame_pointer,
                    callsite_info.stack_args_size
                );
                trace!("\tRestoring SP to: 0x{:x}", sp);

                if cfg!(debug_assertions) {
                    trace!("Restoring frame: ");
                    print_frame(frame_cursor);
                }

                break; // Found a catch block
            }

            // Restore callee saved registers
            unsafe {
                for (target_offset, source_offset) in callsite_info.callee_saved_registers.iter() {
                    // *(frame_cursor + target_offset) = *(frame_pointer + source_offset)
                    let val = (previous_frame_pointer + *source_offset).load::<Address>();
                    (frame_cursor + *target_offset).store::<Address>(val);
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
    // The above loop will only exit when a catch block is found, so restore to it
    unsafe {
        thread::exception_restore(catch_address, frame_cursor.to_ptr(), sp);
    }
}

/// prints current frame cursor
fn print_frame(cursor: Address) {
    let top = 2;
    let bottom = -(CALLEE_SAVED_COUNT as isize);
    for i in (bottom..top).rev() {
        unsafe {
            let addr = cursor + (i * POINTER_SIZE as isize);
            let val = addr.load::<Word>();
            trace!("\taddr: 0x{:x} | val: 0x{:x} {}", addr, val, {
                if addr == cursor {
                    "<- cursor"
                } else {
                    ""
                }
            });
        }

    }
}

/// This function may segfault or panic when it reaches the bottom of the stack
//  TODO: Determine where the bottom is without segfaulting
fn print_backtrace(base: Address, compiled_callsite_table: &HashMap<Address, CompiledCallsite>) {
    if log::max_log_level() < log::LogLevelFilter::Debug {
        return;
    }
	
    debug!("BACKTRACE: ");
    let cur_thread = thread::MuThread::current();
    let ref vm = cur_thread.vm;
    // compiled_funcs: RwLock<HashMap<MuID, RwLock<CompiledFunction>>>;
    let compiled_funcs = vm.compiled_funcs().read().unwrap();
    let mut frame_pointer = base;
    let mut frame_count = 0;

    loop {
        let callsite = get_return_address(frame_pointer);
        frame_pointer = get_previous_frame_pointer(frame_pointer);
        if frame_pointer.is_zero() {
            return;
        }

        if compiled_callsite_table.contains_key(&callsite) {
            let function_version = compiled_callsite_table
                .get(&callsite)
                .unwrap()
                .function_version;
            let compiled_func = compiled_funcs
                .get(&function_version)
                .unwrap()
                .read()
                .unwrap();

            debug!(
                "\tframe {:2}: 0x{:x} - {} (fid: #{}, fvid: #{}) at 0x{:x} - {}",
                frame_count,
                compiled_func.start.to_address(),
                vm.get_name_for_func(compiled_func.func_id),
                compiled_func.func_id,
                compiled_func.func_ver_id,
                callsite,
                get_symbol_name(callsite)
            );
        } else {
            let (func_name, func_start) = get_function_info(callsite);
            debug!(
                "\tframe {:2}: 0x{:x} - {} at 0x{:x}",
                frame_count,
                func_start,
                func_name,
                callsite
            );
	    debug!("...");
            break;
        }

        frame_count += 1;
    }
}
