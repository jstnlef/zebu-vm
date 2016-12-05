extern crate log;
extern crate libloading;
extern crate mu;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::vm::*;
use self::mu::compiler::*;
use self::mu::runtime::thread::MuThread;
use self::mu::utils::Address;
use self::mu::utils::LinkedHashMap;

use std::sync::Arc;
use std::sync::RwLock;
use self::mu::testutil;
use self::mu::testutil::aot;

#[test]
fn test_instruction_new() {
    VM::start_logging_trace();
    
    let vm = Arc::new(alloc_new());
    
    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());
    
    let func_id = vm.id_of("alloc_new");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
        
        compiler.compile(&mut func_ver);
    }
    
    vm.make_primordial_thread(func_id, vec![]);
    backend::emit_context(&vm);
    
    let executable = aot::link_primordial(vec!["alloc_new".to_string()], "alloc_new_test", &vm);
    aot::execute(executable);
}

#[allow(dead_code)]
//#[test]
// The test won't work, since the generated dylib wants to use 'alloc_slow'.
// but in current process, there is no 'alloc_slow' (rust mangles it)
// The solution would be starting mu vm with libmu.so, then create IR from there.
// test_jit should contains a test for it. So I do not test it here
fn test_instruction_new_on_cur_thread() {
    VM::start_logging_trace();

    // compile
    let vm = Arc::new(alloc_new());
    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());
    let func_id = vm.id_of("alloc_new");
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    backend::emit_context(&vm);

    // link
    let libname = &testutil::get_dylib_name("alloc_new_on_cur_thread");
    let dylib = aot::link_dylib(vec![Mu("alloc_new")], libname, &vm);
    let lib = libloading::Library::new(dylib.as_os_str()).unwrap();

    unsafe {
        MuThread::current_thread_as_mu_thread(Address::zero(), vm.clone());
        let func : libloading::Symbol<unsafe extern fn() -> ()> = lib.get(b"alloc_new").unwrap();

        func();
    }
}

#[allow(unused_variables)]
pub fn alloc_new() -> VM {
    let vm = VM::new();
    
    // .typedef @int64 = int<64>
    // .typedef @iref_int64 = iref<int<64>>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), "int64".to_string());
    let type_def_iref_int64 = vm.declare_type(vm.next_id(), MuType_::iref(type_def_int64.clone()));
    vm.set_name(type_def_iref_int64.as_entity(), "iref_int64".to_string());
    let type_def_ref_int64 = vm.declare_type(vm.next_id(), MuType_::muref(type_def_int64.clone()));
    vm.set_name(type_def_ref_int64.as_entity(), "ref_int64".to_string());
    
    // .const @int_64_0 <@int_64> = 0
    // .const @int_64_1 <@int_64> = 1
    let const_def_int64_0 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(0));
    vm.set_name(const_def_int64_0.as_entity(), "int64_0".to_string());
    let const_def_int64_1 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(1));
    vm.set_name(const_def_int64_1.as_entity(), "int64_1".to_string());
    
    // .funcsig @alloc_new_sig = () -> (@int64)
    let func_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_int64.clone()], vec![]);
    vm.set_name(func_sig.as_entity(), "alloc_new_sig".to_string());

    // .funcdecl @alloc_new <@alloc_new_sig>
    let func = MuFunction::new(vm.next_id(), func_sig.clone());
    vm.set_name(func.as_entity(), "alloc_new".to_string());
    let func_id = func.id();
    vm.declare_func(func);
    
    // .funcdef @alloc VERSION @v1 <@alloc_new_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, func_sig.clone());
    
    // %blk_0():
    let mut blk_0 = Block::new(vm.next_id());
    vm.set_name(blk_0.as_entity(), "blk_0".to_string());
    
    // %a = NEW <@int64_t>
    let blk_0_a = func_ver.new_ssa(vm.next_id(), type_def_ref_int64.clone());
    vm.set_name(blk_0_a.as_entity(), "blk_0_a".to_string());
    let blk_0_inst0 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_0_a.clone_value()]),
        ops: RwLock::new(vec![]),
        v: Instruction_::New(type_def_int64.clone())
    });
    
    // %a_iref = GETIREF <@int_64> @a
    let blk_0_a_iref = func_ver.new_ssa(vm.next_id(), type_def_iref_int64.clone());
    vm.set_name(blk_0_a.as_entity(), "blk_0_a_iref".to_string());
    let blk_0_inst1 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_0_a_iref.clone_value()]),
        ops: RwLock::new(vec![blk_0_a.clone()]),
        v: Instruction_::GetIRef(0)
    });
    
    // STORE <@int_64> @a_iref @int_64_1
    let blk_0_const_int64_1 = func_ver.new_constant(const_def_int64_1.clone());
    let blk_0_inst2 = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_0_a_iref.clone(), blk_0_const_int64_1.clone()]),
        v: Instruction_::Store{
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0,
            value: 1
        }
    });    
        
//    // %x = LOAD <@int_64> @a_iref
//    let blk_0_x = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
//    vm.set_name(blk_0_x.as_entity(), "blk_0_x".to_string());
//    let blk_0_inst3 = func_ver.new_inst(vm.next_id(), Instruction{
//        value: Some(vec![blk_0_x.clone_value()]),
//        ops: RwLock::new(vec![blk_0_a_iref.clone()]),
//        v: Instruction_::Load{
//            is_ptr: false,
//            order: MemoryOrder::Relaxed,
//            mem_loc: 0
//        }
//    });
    
    let blk_0_term = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![]),
        v: Instruction_::ThreadExit
    });
    
    let blk_0_content = BlockContent {
        args: vec![],
        exn_arg: None,
        body: vec![blk_0_inst0, blk_0_inst1, blk_0_inst2, blk_0_term],
        keepalives: None
    };
    blk_0.content = Some(blk_0_content);
    
    func_ver.define(FunctionContent{
        entry: blk_0.id(),
        blocks: {
            let mut ret = LinkedHashMap::new();
            ret.insert(blk_0.id(), blk_0);
            ret
        }
    });
    
    vm.define_func_version(func_ver);
    
    vm
}