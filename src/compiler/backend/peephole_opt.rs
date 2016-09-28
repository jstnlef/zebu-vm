use compiler::CompilerPass;
use ast::ir::*;
use vm::VM;
use compiler::machine_code::CompiledFunction;

pub struct PeepholeOptimization {
    name: &'static str
}

impl PeepholeOptimization {
    pub fn new() -> PeepholeOptimization {
        PeepholeOptimization {
            name: "Peephole Optimization"
        }
    }
    
    pub fn remove_redundant_move(&mut self, inst: usize, cf: &mut CompiledFunction) {
        if cf.mc().is_move(inst) && !cf.mc().is_using_mem_op(inst) {
            cf.mc().trace_inst(inst);
            
            let src : MuID = {
                let uses = cf.mc().get_inst_reg_uses(inst);
                if uses.len() != 1 {
                    // moving immediate to register, its not redundant
                    return;
                }                
                uses[0]
            };
            let dst : MuID = cf.mc().get_inst_reg_defines(inst)[0];
            
            let src_machine_reg : MuID = {
                match cf.temps.get(&src) {
                    Some(reg) => *reg,
                    None => src
                }
            };
            let dst_machine_reg : MuID = {
                match cf.temps.get(&dst) {
                    Some(reg) => *reg,
                    None => dst
                }
            };
            
            if src_machine_reg == dst_machine_reg {
                trace!("Redundant! removed");
                // redundant, remove this move
                cf.mc_mut().set_inst_nop(inst);
            }
        }
    }
}

impl CompilerPass for PeepholeOptimization {
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        let compiled_funcs = vm.compiled_funcs().read().unwrap();
        let mut cf = compiled_funcs.get(&func.id()).unwrap().write().unwrap();
        
        for i in 0..cf.mc().number_of_insts() {
            self.remove_redundant_move(i, &mut cf);
        }
        
        trace!("after peephole optimization:");
        cf.mc().trace_mc();
    }
}
