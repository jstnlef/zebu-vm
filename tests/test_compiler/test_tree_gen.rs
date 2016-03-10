extern crate mu;
extern crate log;
extern crate simple_logger;

use test_ir::test_ir::factorial;
use self::mu::compiler::*;
use self::mu::vm::context::VMContext;

#[test]
fn test_tree_gen() {
    simple_logger::init_with_level(log::LogLevel::Trace).unwrap();
    
    let vm_context : VMContext = factorial();
    let compiler = Compiler::new(CompilerPolicy::default());
    
    let mut factorial_func = {
        vm_context.get_func("fac").unwrap().borrow_mut()
    };
    
    compiler.compile(&vm_context, &mut factorial_func);
}