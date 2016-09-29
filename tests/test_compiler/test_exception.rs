extern crate log;
extern crate simple_logger;
extern crate libloading;
extern crate mu;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::vm::*;
use self::mu::compiler::*;
use self::mu::runtime::thread;
use self::mu::runtime::thread::MuThread;
use self::mu::runtime::mm;
use self::mu::utils::ByteSize;

use aot;

use std::sync::Arc;
use std::sync::RwLock;
use std::collections::HashMap;

#[test]
fn test_exception_simple_throw_catch() {
    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    let vm = Arc::new(simple_throw_catch());
    
    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());
    
    let func_throw = vm.id_of("throw_exception");
    let func_catch = vm.id_of("catch_exception");    
    {
        let funcs = vm.funcs().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        
        {
            let func = funcs.get(&func_throw).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
            compiler.compile(&mut func_ver);
        }
        {
            let func = funcs.get(&func_catch).unwrap().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
    
            compiler.compile(&mut func_ver);
        }
    }
    
    vm.make_primordial_thread(func_catch, vec![]);
    backend::emit_context(&vm);
    
    let executable = aot::link_primordial(vec![Mu("throw_exception"), Mu("catch_exception")], "simple_throw_catch_test");
    aot::execute(executable);
}

fn simple_throw_catch() -> VM {
    let vm = VM::new();
    
    // .typedef @int64 = int<64>
    // .typedef @ref_int64 = ref<int<64>>
    // .typedef @iref_int64 = iref<int<64>>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), "int64".to_string());
    let type_def_ref_int64 = vm.declare_type(vm.next_id(), MuType_::muref(type_def_int64.clone()));
    vm.set_name(type_def_ref_int64.as_entity(), "ref_int64".to_string());
    let type_def_iref_int64 = vm.declare_type(vm.next_id(), MuType_::iref(type_def_int64.clone()));
    vm.set_name(type_def_iref_int64.as_entity(), "iref_int64".to_string());    
    
    // .const @int_64_0 <@int_64> = 0
    // .const @int_64_1 <@int_64> = 1
    let const_def_int64_0 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(0));
    vm.set_name(const_def_int64_0.as_entity(), "int64_0".to_string());
    let const_def_int64_1 = vm.declare_const(vm.next_id(), type_def_int64.clone(), Constant::Int(1));
    vm.set_name(const_def_int64_1.as_entity(), "int64_1".to_string());    
    
    create_throw_exception_func(&vm);
    create_catch_exception_func(&vm);
    
    vm
}

fn create_catch_exception_func (vm: &VM) {
    // .typedef @funcref_throw_exception <@throw_exception_sig>
    let throw_exception_sig = vm.get_func_sig(vm.id_of("throw_exception_sig"));
    let throw_exception_id = vm.id_of("throw_exception");
    let type_funcref_throw_exception = vm.declare_type(vm.next_id(), MuType_::funcref(throw_exception_sig));
    // .const @throw_exception_func
    let const_funcref_throw_exception = vm.declare_const(vm.next_id(), type_funcref_throw_exception, Constant::FuncRef(throw_exception_id)); 
    
    // .funcsig @catch_exception_sig = () -> ()
    let func_sig = vm.declare_func_sig(vm.next_id(), vec![], vec![]);
    vm.set_name(func_sig.as_entity(), Mu("catch_exception_sig"));
    
    // .funcdecl @catch_exception <@catch_exception_sig>
    let func = MuFunction::new(vm.next_id(), func_sig.clone());
    vm.set_name(func.as_entity(), Mu("catch_exception"));
    let func_id = func.id();
    vm.declare_func(func);
    
    // .funcdef @catch_exception VERSION @v1 <@catch_exception_sig);
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, func_sig.clone());
    
    // %blk_0():
    let mut blk_0 = Block::new(vm.next_id());
    vm.set_name(blk_0.as_entity(), Mu("blk_0"));
    
    let blk_normal_cont_id = vm.next_id();
    let blk_exn_cont_id = vm.next_id();
    
    let blk_0_throw = func_ver.new_constant(vm.next_id(), const_funcref_throw_exception.clone());
    let blk_0_term = func_ver.new_inst(vm.next_id(), Instruction {
        value: None,
        ops: RwLock::new(vec![blk_0_throw]),
        v: Instruction_::Call {
            data: CallData {
                func: 0,
                args: vec![],
                convention: CallConvention::Mu
            },
            resume: ResumptionData {
                normal_dest: Destination {
                    target: blk_normal_cont_id,
                    args: vec![]
                },
                exn_dest: Destination {
                    target: blk_exn_cont_id,
                    args: vec![]
                }
            }
        }
    });
    
    let blk_0_content = BlockContent {
        args: vec![],
        exn_arg: None,
        body: vec![blk_0_term],
        keepalives: None
    };
    blk_0.content = Some(blk_0_content);
    
    // %blk_normal_cont():
    let mut blk_normal_cont = Block::new(blk_normal_cont_id);
    vm.set_name(blk_normal_cont.as_entity(), Mu("blk_normal_cont"));
    let blk_normal_cont_thread_exit = func_ver.new_inst(vm.next_id(), Instruction {
        value: None,
        ops: RwLock::new(vec![]),
        v: Instruction_::ThreadExit
    });
    blk_normal_cont.content = Some(BlockContent {
        args: vec![],
        exn_arg: None,
        body: vec![blk_normal_cont_thread_exit],
        keepalives: None
    });
    
    // %blk_exn_cont() %EXN:
    let mut blk_exn_cont = Block::new(blk_exn_cont_id);
    vm.set_name(blk_exn_cont.as_entity(), Mu("blk_exn_cont"));
    let type_ref_int64 = vm.get_type(vm.id_of("ref_int64"));    
    let blk_exn_cont_exception_arg = func_ver.new_ssa(vm.next_id(), type_ref_int64.clone());
    vm.set_name(blk_exn_cont_exception_arg.as_entity(), Mu("blk_0_exception_arg"));
    let blk_exn_cont_thread_exit = func_ver.new_inst(vm.next_id(), Instruction {
        value: None,
        ops: RwLock::new(vec![]),
        v: Instruction_::ThreadExit
    });
    blk_exn_cont.content = Some(BlockContent {
        args: vec![],
        exn_arg: Some(blk_exn_cont_exception_arg.clone_value()),
        body: vec![blk_exn_cont_thread_exit],
        keepalives: None
    });    
    
    func_ver.define(FunctionContent{
        entry: blk_0.id(),
        blocks: {
            let mut ret = HashMap::new();
            ret.insert(blk_0.id(), blk_0);
            ret.insert(blk_normal_cont.id(), blk_normal_cont);
            ret.insert(blk_exn_cont.id(), blk_exn_cont);
            ret
        }
    });
    
    vm.define_func_version(func_ver);
}

fn create_throw_exception_func (vm: &VM) {
    let type_int64 = vm.get_type(vm.id_of("int64"));
    let type_ref_int64 = vm.get_type(vm.id_of("ref_int64"));
    let type_iref_int64 = vm.get_type(vm.id_of("iref_int64"));
    
    // .funcsig @throw_exception = () -> ()
    let func_sig = vm.declare_func_sig(vm.next_id(), vec![], vec![]);
    vm.set_name(func_sig.as_entity(), Mu("throw_exception_sig"));
    
    // .funcdecl @catch_exception <@throw_exception>
    let func = MuFunction::new(vm.next_id(), func_sig.clone());
    vm.set_name(func.as_entity(), Mu("throw_exception"));
    let func_id = func.id();
    vm.declare_func(func);
    
    // .funcdef @catch_exception VERSION @v1 <@throw_exception>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, func_sig.clone());
    
    // %blk_0():
    let mut blk_0 = Block::new(vm.next_id());
    vm.set_name(blk_0.as_entity(), Mu("blk_0"));
    
    // %exception_obj = NEW <@int64>
    let blk_0_exception_obj = func_ver.new_ssa(vm.next_id(), type_ref_int64.clone());
    vm.set_name(blk_0_exception_obj.as_entity(), Mu("blk_0_exception_obj"));
    let blk_0_inst0 = func_ver.new_inst(vm.next_id(), Instruction {
            value: Some(vec![blk_0_exception_obj.clone_value()]),
            ops: RwLock::new(vec![]),
            v: Instruction_::New(type_ref_int64.clone())
    });
    
    // %exception_obj_iref = GETIREF <@int64> %exception_obj
    let blk_0_exception_obj_iref = func_ver.new_ssa(vm.next_id(), type_iref_int64.clone());
    vm.set_name(blk_0_exception_obj_iref.as_entity(), Mu("blk_0_exception_obj_iref"));
    let blk_0_inst1 = func_ver.new_inst(vm.next_id(), Instruction {
            value: Some(vec![blk_0_exception_obj_iref.clone_value()]),
            ops: RwLock::new(vec![blk_0_exception_obj.clone()]),
            v: Instruction_::GetIRef(0)
     });
    
    // STORE <@int64> %exception_obj_iref @int64_1
    let const_int64_1 = vm.get_const(vm.id_of("int64_1"));
    let blk_0_const_int64_1 = func_ver.new_constant(vm.next_id(), const_int64_1);
    let blk_0_inst2 = func_ver.new_inst(vm.next_id(), Instruction {
        value: None,
        ops: RwLock::new(vec![blk_0_exception_obj_iref.clone(), blk_0_const_int64_1.clone()]),
        v: Instruction_::Store {
            is_ptr: false,
            order: MemoryOrder::Relaxed,
            mem_loc: 0,
            value: 1
        }
    });
    
    let blk_0_term = func_ver.new_inst(vm.next_id(), Instruction {
        value: None,
        ops: RwLock::new(vec![blk_0_exception_obj.clone()]),
        v: Instruction_::Throw(0)
    });
    
    let blk_0_content = BlockContent {
        args: vec![],
        exn_arg: None,
        body: vec![blk_0_inst0, blk_0_inst1, blk_0_inst2, blk_0_term],
        keepalives: None
    };
    blk_0.content = Some(blk_0_content);
    
    func_ver.define(FunctionContent {
        entry: blk_0.id(),
        blocks: {
            let mut ret = HashMap::new();
            ret.insert(blk_0.id(), blk_0);
            ret
        }
    });
    
    vm.define_func_version(func_ver);
}