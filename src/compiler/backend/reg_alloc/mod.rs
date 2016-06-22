#![allow(dead_code)]

use compiler::CompilerPass;
use ast::ir::*;
use vm::context::VMContext;

use compiler::backend::init_machine_regs_for_func;

mod liveness;
mod coloring;

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
        let compiled_funcs = vm_context.compiled_funcs().read().unwrap();
        let mut cf = compiled_funcs.get(func.fn_name).unwrap().borrow_mut();
        
        cf.mc.print();
        
        // initialize machine registers for the function context
        init_machine_regs_for_func(&mut func.context);
        
        let liveness = liveness::build(&mut cf, func);
        liveness.print();
        
        let coloring = coloring::GraphColoring::start(liveness);
        let spills = coloring.spills();
        
        if !spills.is_empty() {
            unimplemented!();
        }
        
        // replace regs
        trace!("Replacing Registers...");
        for node in coloring.ig.nodes() {
            let temp = coloring.ig.get_temp_of(node);
            
            // skip machine registers
            if temp < RESERVED_NODE_IDS_FOR_MACHINE {
                continue;
            } else {
                let alias = coloring.get_alias(node);
                let machine_reg = coloring.ig.get_color_of(alias).unwrap();
                
                trace!("replacing {} with {}", temp, machine_reg);
                cf.mc.replace_reg(temp, machine_reg);
            }
        }
        
        cf.mc.print();
    }
}