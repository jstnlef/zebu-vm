use compiler::CompilerPass;
use ast::ir::*;
use vm::context::VMContext;

mod liveness;

pub struct RegisterAllocation {
    name: &'static str
}

impl RegisterAllocation {
    pub fn new() -> RegisterAllocation {
        RegisterAllocation {
            name: "Register Allcoation"
        }
    }
}

impl CompilerPass for RegisterAllocation {
    fn name(&self) -> &'static str {
        self.name
    }
    
    #[allow(unused_variables)]
    fn visit_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        let mut compiled_funcs = vm_context.compiled_funcs().read().unwrap();
        let mut cf = compiled_funcs.get(func.fn_name).unwrap().borrow_mut();
        
        cf.mc.print();
        
//        liveness::build(&mut cf, func);
    }
}