#![allow(dead_code)]

use compiler::CompilerPass;
use compiler::PassExecutionResult;
use ast::ir::*;
use vm::VM;

use compiler::backend::init_machine_regs_for_func;

mod graph_coloring;

pub struct RegisterAllocation {
    name: &'static str
}

impl RegisterAllocation {
    pub fn new() -> RegisterAllocation {
        RegisterAllocation {
            name: "Register Allcoation"
        }
    }
    
    #[allow(unused_variables)]
    // returns true if we spill registers (which requires another instruction selection)
    fn coloring(&mut self, vm: &VM, func: &mut MuFunctionVersion) -> bool {
        let compiled_funcs = vm.compiled_funcs().read().unwrap();
        let mut cf = compiled_funcs.get(&func.id()).unwrap().write().unwrap();
        
        cf.mc().trace_mc();
        
        // initialize machine registers for the function context
        init_machine_regs_for_func(&mut func.context);
        
        let liveness = graph_coloring::build_inteference_graph(&mut cf, func);
        liveness.print();
        
        let coloring = graph_coloring::GraphColoring::start(liveness);
        let spills = coloring.spills();
        
        if !spills.is_empty() {
            return false;
        }
        
        // replace regs
        trace!("Replacing Registers...");
        for node in coloring.ig.nodes() {
            let temp = coloring.ig.get_temp_of(node);
            
            // skip machine registers
            if temp < MACHINE_ID_END {
                continue;
            } else {
                let alias = coloring.get_alias(node);
                let machine_reg = coloring.ig.get_color_of(alias).unwrap();
                
                trace!("replacing {} with {}", temp, machine_reg);
                cf.mc_mut().replace_reg(temp, machine_reg);
                
                cf.temps.insert(temp, machine_reg);
            }
        }
        
        cf.mc().trace_mc();
        
        true
    }    
}

impl CompilerPass for RegisterAllocation {
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn execute(&mut self, vm: &VM, func: &mut MuFunctionVersion) -> PassExecutionResult {
        debug!("---CompilerPass {} for {}---", self.name(), func);
        
        if self.coloring(vm, func) {
            debug!("---finish---");
            
            PassExecutionResult::ProceedToNext
        } else {
            // PassExecutionResult::GoBackTo(compiler::PASS_INST_SEL)
                        
            unimplemented!()
        }
    }
}
