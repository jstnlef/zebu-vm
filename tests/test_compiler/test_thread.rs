#![allow(unused_imports)]

extern crate mu;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::compiler::*;
use self::mu::utils::LinkedHashMap;
use self::mu::testutil::aot;

use std::sync::Arc;
use std::sync::RwLock;

#[test]
fn test_thread_create() {
    VM::start_logging_trace();
    
    let vm = Arc::new(primordial_main());
    
    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());
    
    let func_id = vm.id_of("primordial_main");    
    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
        
        compiler.compile(&mut func_ver);
    }
    
    vm.make_primordial_thread(func_id, vec![]);
    backend::emit_context(&vm);
    
    let executable = aot::link_primordial(vec!["primordial_main".to_string()], "primordial_main_test", &vm);
    aot::execute(executable);
}

fn primordial_main() -> VM {
    let vm = VM::new();
    
    let sig = vm.declare_func_sig(vm.next_id(), vec![], vec![]);
    let func = MuFunction::new(vm.next_id(), sig.clone());
    vm.set_name(func.as_entity(), "primordial_main".to_string());
    let func_id = func.id();
    vm.declare_func(func);
    
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, sig.clone());
    
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), "entry".to_string());
    let thread_exit = func_ver.new_inst(Instruction {
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![]),
        v: Instruction_::ThreadExit
    });
    
    let blk_entry_content = BlockContent {
        args: vec![],
        exn_arg: None,
        body: vec![thread_exit],
        keepalives: None
    };
    blk_entry.content = Some(blk_entry_content);
    
    func_ver.define(FunctionContent {
        entry: blk_entry.id(),
        blocks: {
            let mut blocks = LinkedHashMap::new();
            blocks.insert(blk_entry.id(), blk_entry);
            blocks
        }
    });
    
    vm.define_func_version(func_ver);
    
    vm
}
