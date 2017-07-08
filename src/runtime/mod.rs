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

use utils;
use ast::ir::*;
use vm::VM;
use compiler::backend::Word;
use compiler::backend::RegGroup;
use utils::Address;

use libc::*;
use std;
use std::fmt;
use std::ffi::CString;
use std::ffi::CStr;
use std::sync::Arc;
use rodal;

pub mod mm;
pub mod thread;
pub mod math;
pub mod entrypoints;

pub mod exception;


// TODO: this actually returns the name and address of the nearest symbol (of any type)
// that starts before function_addr (instead we want the nearest function symbol)
pub fn get_function_info(function_addr: Address) -> (CName, Address) {
    // dladdr will initialise this for us
    let mut info = unsafe{std::mem::uninitialized::<Dl_info>()};

    unsafe {dladdr(function_addr.to_ptr_mut::<c_void>(), &mut info)};

    let error = unsafe {dlerror()};
    if !error.is_null() {
        let cstr = unsafe {CStr::from_ptr(error)};
        error!("cannot find function address: {}", function_addr);
        error!("{}", cstr.to_str().unwrap());

        panic!("failed to resolve function address");
    }
    if !info.dli_sname.is_null() {
        (unsafe {CStr::from_ptr(info.dli_sname)}.to_str().unwrap().to_string(), Address::from_ptr(info.dli_saddr))
    } else {
        ("UNKOWN".to_string(), Address::from_ptr(info.dli_saddr))
    }

}

pub fn resolve_symbol(symbol: String) -> Address {
    use std::ptr;

    let c_symbol = CString::new(mangle_name(symbol.clone())).unwrap();
    
    let rtld_default = unsafe {dlopen(ptr::null(), 0)};
    let ret = unsafe {dlsym(rtld_default, c_symbol.as_ptr())};

    let error = unsafe {dlerror()};
    if !error.is_null() {
        let cstr = unsafe {CStr::from_ptr(error)};
        panic!("failed to resolve symbol: {} ({})", symbol, cstr.to_str().unwrap());
    }
    
    Address::from_ptr(ret)
}

rodal_enum!(ValueLocation{(Register: group, id), (Constant: group, word), (Relocatable: group, name)});
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ValueLocation {
    Register(RegGroup, MuID),
    Constant(RegGroup, Word),
    Relocatable(RegGroup, MuName),
    
    Direct(RegGroup, Address),    // Not dumped
    Indirect(RegGroup, Address),  // Not dumped
}

impl fmt::Display for ValueLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ValueLocation::Register(_, id) => write!(f, "VL_Reg: {}", id),
            &ValueLocation::Constant(_, val) => write!(f, "VL_Const: {}", val),
            &ValueLocation::Relocatable(_, ref name) => write!(f, "VL_Reloc: {}", name),
            &ValueLocation::Direct(_, addr) => write!(f, "VL_Direct: 0x{:x}", addr),
            &ValueLocation::Indirect(_, addr) => write!(f, "VL_Indirect: 0x{:x}", addr)
        }
    }
}

impl ValueLocation {
    pub fn load_value(&self) -> (RegGroup, Word) {
        match self {
            &ValueLocation::Register(_, _)
            | &ValueLocation::Direct(_, _)
            | &ValueLocation::Indirect(_, _) => unimplemented!(),
            
            &ValueLocation::Constant(group, word) => {
                (group, word)
            }
            &ValueLocation::Relocatable(_, _) => panic!("expect a runtime value")
        }
    }
    
    #[allow(unused_variables)]
    pub fn from_constant(c: Constant) -> ValueLocation {
        match c {
            Constant::Int(int_val) => ValueLocation::Constant(RegGroup::GPR, utils::mem::u64_to_raw(int_val)),
            Constant::Float(f32_val) => ValueLocation::Constant(RegGroup::FPR, utils::mem::f32_to_raw(f32_val)),
            Constant::Double(f64_val) => ValueLocation::Constant(RegGroup::FPR, utils::mem::f64_to_raw(f64_val)),
            
            _ => unimplemented!()
        }
    }
    
    pub fn to_address(&self) -> Address {
        match self {
            &ValueLocation::Register(_, _)
            | &ValueLocation::Constant(_, _) => panic!("a register/constant cannot be turned into address"),
            &ValueLocation::Direct(_, addr) => addr, 
            &ValueLocation::Indirect(_, addr) => unsafe {addr.load::<Address>()},
            &ValueLocation::Relocatable(_, ref symbol) => resolve_symbol(symbol.clone())
        }
    }

    pub fn to_relocatable(&self) -> MuName {
        match self {
            &ValueLocation::Relocatable(_, ref name) => name.clone(),
            _ => panic!("expecting Relocatable location, found {}", self)
        }
    }
}

pub const PRIMORDIAL_ENTRY : &'static str = "src/runtime/main.c";

#[no_mangle]
pub extern fn mu_trace_level_log() {
    VM::start_logging_trace();
}

#[no_mangle]
pub static mut LAST_TIME: c_ulong = 0;

#[no_mangle]
pub extern fn mu_main(edata: *const(), dumped_vm : *mut Arc<VM>, argc: c_int, argv: *const *const c_char) {
    VM::start_logging_env();
    debug!("mu_main() started...");

    unsafe{rodal::load_asm_bounds(rodal::Address::from_ptr(dumped_vm), rodal::Address::from_ptr(edata))};
    let vm = VM::resume_vm(dumped_vm);

    let primordial = vm.primordial.read().unwrap();
    if primordial.is_none() {
        panic!("no primordial thread/stack/function. Client should provide an entry point");
    } else {
        let primordial = primordial.as_ref().unwrap();
        
        // create mu stack
        let stack = vm.new_stack(primordial.func_id);

        // if the primordial named some const arguments, we use the const args
        // otherwise we push 'argc' and 'argv' to new stack
        let args : Vec<ValueLocation> = if primordial.has_const_args {
            primordial.args.iter().map(|arg| ValueLocation::from_constant(arg.clone())).collect()
        } else {
            let mut args = vec![];

            // 1st arg: argc
            args.push(ValueLocation::from_constant(Constant::Int(argc as u64)));

            // 2nd arg: argv
            args.push(ValueLocation::from_constant(Constant::Int(argv as u64)));

            args
        };
        
        // FIXME: currently assumes no user defined thread local
        // will need to fix this after we can serialize heap object
        let thread = thread::MuThread::new_thread_normal(stack, unsafe{Address::zero()}, args, vm.clone());
        
        thread.join().unwrap();
    }
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern fn muentry_print_hex(x: u64) {
    println!("PRINTHEX: 0x{:x}", x);
}