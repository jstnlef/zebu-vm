use utils;
use ast::ir::*;
use vm::VM;
use compiler::backend::Word;
use compiler::backend::RegGroup;
use utils::Address;

use std::fmt;
use std::os::raw::c_int;
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::ffi::CString;
use std::ffi::CStr;
use std::sync::Arc;

pub mod mm;
pub mod thread;
pub mod math;
pub mod entrypoints;

#[cfg(target_arch = "x86_64")]
#[path = "exception_x64.rs"]
pub mod exception;

// consider using libloading crate instead of the raw c functions for dynalic libraries
// however i am not sure if libloading can load symbols from current process (not from an actual dylib)
// so here i use dlopen/dlsym from C
#[link(name="dl")]
extern "C" {
    fn dlopen(filename: *const c_char, flags: isize) -> *const c_void;
    fn dlsym(handle: *const c_void, symbol: *const c_char) -> *const c_void;
    fn dlerror() -> *const c_char;
}

pub fn resolve_symbol(symbol: String) -> Address {
    use std::ptr;

    let symbol = MuEntityHeader::name_check(symbol);
    
    let rtld_default = unsafe {dlopen(ptr::null(), 0)};
    let ret = unsafe {dlsym(rtld_default, CString::new(symbol.clone()).unwrap().as_ptr())};

    let error = unsafe {dlerror()};
    if !error.is_null() {
        let cstr = unsafe {CStr::from_ptr(error)};
        println!("cannot find symbol: {}", symbol);
        println!("{}", cstr.to_str().unwrap());

        panic!("failed to resolve symbol");
    }
    
    Address::from_ptr(ret)
}

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ValueLocation {
    Register(RegGroup, MuID),     // 0
    Constant(RegGroup, Word),     // 1    
    Relocatable(RegGroup, MuName),// 2
    
    Direct(RegGroup, Address),    // 3
    Indirect(RegGroup, Address),  // 4
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

impl Encodable for ValueLocation {
    fn encode<S: Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_enum("ValueLocation", |s| {
            match self {
                &ValueLocation::Register(grp, id) => {
                    s.emit_enum_variant("Register", 0, 2, |s| {
                        try!(s.emit_enum_variant_arg(0, |s| grp.encode(s)));
                        try!(s.emit_enum_variant_arg(1, |s| id.encode(s)));
                        Ok(())
                    })
                }
                &ValueLocation::Constant(grp, val) => {
                    s.emit_enum_variant("Constant", 1, 2, |s| {
                        try!(s.emit_enum_variant_arg(0, |s| grp.encode(s)));
                        try!(s.emit_enum_variant_arg(1, |s| val.encode(s)));
                        Ok(())
                    })    
                }                
                &ValueLocation::Relocatable(grp, ref name) => {
                    s.emit_enum_variant("Relocatable", 2, 2, |s| {
                        try!(s.emit_enum_variant_arg(0, |s| grp.encode(s)));
                        try!(s.emit_enum_variant_arg(1, |s| name.encode(s)));
                        Ok(())
                    })
                }
                &ValueLocation::Direct(_, _)
                | &ValueLocation::Indirect(_, _) => {
                    panic!("trying to encode an address location (not persistent)")
                }
            }
        })
    }
}

impl Decodable for ValueLocation {
    fn decode<D: Decoder>(d: &mut D) -> Result<ValueLocation, D::Error> {
        d.read_enum("ValueLocation", |d| {
            d.read_enum_variant(
                &vec!["Register", "Constant", "Relocatable"],
                |d, idx| {
                    match idx {
                        0 => {
                            // Register variant
                            let grp = try!(d.read_enum_variant_arg(0, |d| Decodable::decode(d)));
                            let id = try!(d.read_enum_variant_arg(1, |d| Decodable::decode(d)));
                            
                            Ok(ValueLocation::Register(grp, id))
                        }
                        1 => {
                            // Constant
                            let grp = try!(d.read_enum_variant_arg(0, |d| Decodable::decode(d)));
                            let val = try!(d.read_enum_variant_arg(1, |d| Decodable::decode(d)));
                            Ok(ValueLocation::Constant(grp, val))
                        }
                        2 => {
                            // Relocatable
                            let grp = try!(d.read_enum_variant_arg(0, |d| Decodable::decode(d)));
                            let name = try!(d.read_enum_variant_arg(1, |d| Decodable::decode(d)));
                            Ok(ValueLocation::Relocatable(grp, name))
                        }
                        _ => panic!("unexpected enum variant for ValueLocation: {}", idx)
                    }
                }
             ) 
        })
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
pub extern fn mu_main(serialized_vm : *const c_char, argc: c_int, argv: *const *const c_char) {
    debug!("mu_main() started...");
    
    let str_vm = unsafe{CStr::from_ptr(serialized_vm)}.to_str().unwrap();
    
    let vm : Arc<VM> = Arc::new(VM::resume_vm(str_vm));
    
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
    println!("0x{:x}", x);
}