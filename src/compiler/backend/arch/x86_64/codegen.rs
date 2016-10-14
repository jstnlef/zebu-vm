use ast::ptr::P;
use ast::ir::*;
use runtime::ValueLocation;

use compiler::machine_code::MachineCode;
use compiler::backend::x86_64::ASMCodeGen;

pub trait CodeGenerator {
    fn start_code(&mut self, func_name: MuName) -> ValueLocation;
    fn finish_code(&mut self, func_name: MuName) -> (Box<MachineCode + Sync + Send>, ValueLocation);

    // generate unnamed sequence of linear code (no branch)
    fn start_code_sequence(&mut self);
    fn finish_code_sequence(&mut self) -> Box<MachineCode + Sync + Send>;
    
    fn print_cur_code(&self);
    
    fn start_block(&mut self, block_name: MuName);
    fn start_exception_block(&mut self, block_name: MuName) -> ValueLocation;
    fn set_block_livein(&mut self, block_name: MuName, live_in: &Vec<P<Value>>);
    fn set_block_liveout(&mut self, block_name: MuName, live_out: &Vec<P<Value>>);
    fn end_block(&mut self, block_name: MuName);
    
    fn emit_nop(&mut self, bytes: usize);
    
    fn emit_cmp_r64_r64(&mut self, op1: &P<Value>, op2: &P<Value>);
    fn emit_cmp_r64_imm32(&mut self, op1: &P<Value>, op2: i32);
    fn emit_cmp_r64_mem64(&mut self, op1: &P<Value>, op2: &P<Value>);
    
    fn emit_mov_r64_imm32(&mut self, dest: &P<Value>, src: i32);
    fn emit_mov_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>); // load
    fn emit_mov_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_mov_mem64_r64(&mut self, dest: &P<Value>, src: &P<Value>); // store
    fn emit_mov_mem64_imm32(&mut self, dest: &P<Value>, src: i32);
    
    fn emit_lea_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    
    fn emit_and_r64_imm32(&mut self, dest: &P<Value>, src: i32);
    fn emit_and_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    
    fn emit_add_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_add_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_add_r64_imm32(&mut self, dest: &P<Value>, src: i32);
    
    fn emit_sub_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_sub_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>);
    fn emit_sub_r64_imm32(&mut self, dest: &P<Value>, src: i32);
    
    fn emit_mul_r64(&mut self, src: &P<Value>);
    fn emit_mul_mem64(&mut self, src: &P<Value>);
    
    fn emit_jmp(&mut self, dest: MuName);
    fn emit_je(&mut self, dest: MuName);
    fn emit_jne(&mut self, dest: MuName);
    fn emit_ja(&mut self, dest: MuName);
    fn emit_jae(&mut self, dest: MuName);
    fn emit_jb(&mut self, dest: MuName);
    fn emit_jbe(&mut self, dest: MuName);
    fn emit_jg(&mut self, dest: MuName);
    fn emit_jge(&mut self, dest: MuName);
    fn emit_jl(&mut self, dest: MuName);
    fn emit_jle(&mut self, dest: MuName);
    
    fn emit_call_near_rel32(&mut self, callsite: String, func: MuName) -> ValueLocation;
    fn emit_call_near_r64(&mut self, callsite: String, func: &P<Value>) -> ValueLocation;
    fn emit_call_near_mem64(&mut self, callsite: String, func: &P<Value>) -> ValueLocation;
    
    fn emit_ret(&mut self);
    
    fn emit_push_r64(&mut self, src: &P<Value>);
    fn emit_push_imm32(&mut self, src: i32);
    fn emit_pop_r64(&mut self, dest: &P<Value>);
}

use std::collections::HashMap;
use compiler::machine_code::CompiledFunction;
use vm::VM;

#[cfg(feature = "aot")]
pub fn spill_rewrite(
    spills: &HashMap<MuID, P<Value>>,
    func: &mut MuFunctionVersion,
    cf: &mut CompiledFunction,
    vm: &VM)
{
    // record code and their insertion point, so we can do the copy/insertion all at once
    let mut spill_code_before: HashMap<usize, Vec<Box<MachineCode>>> = HashMap::new();
    let mut spill_code_after: HashMap<usize, Vec<Box<MachineCode>>> = HashMap::new();

    // iterate through all instructions
    for i in 0..cf.mc().number_of_insts() {
        // find use of any register that gets spilled
        {
            let reg_uses = cf.mc().get_inst_reg_uses(i).to_vec();
            for reg in reg_uses {
                if spills.contains_key(&reg) {
                    // a register used here is spilled
                    let spill_mem = spills.get(&reg).unwrap();

                    // generate a random new temporary
                    let temp_ty = func.context.get_value(reg).unwrap().ty().clone();
                    let temp = func.new_ssa(vm.next_id(), temp_ty).clone_value();

                    // generate a load
                    let code = {
                        let mut codegen = ASMCodeGen::new();
                        codegen.start_code_sequence();
                        codegen.emit_mov_r64_mem64(&temp, spill_mem);

                        codegen.finish_code_sequence()
                    };
                    // record that this load will be inserted at i
                    if spill_code_before.contains_key(&i) {
                        spill_code_before.get_mut(&i).unwrap().push(code);
                    } else {
                        spill_code_before.insert(i, vec![code]);
                    }

                    // replace register reg with temp
                    cf.mc_mut().replace_reg_for_inst(reg, temp.id(), i);
                }
            }
        }

        // fine define of any register that gets spilled
        {
            let reg_defines = cf.mc().get_inst_reg_defines(i).to_vec();
            for reg in reg_defines {
                if spills.contains_key(&reg) {
                    let spill_mem = spills.get(&reg).unwrap();

                    let temp_ty = func.context.get_value(reg).unwrap().ty().clone();
                    let temp = func.new_ssa(vm.next_id(), temp_ty).clone_value();

                    let code = {
                        let mut codegen = ASMCodeGen::new();
                        codegen.start_code_sequence();
                        codegen.emit_mov_mem64_r64(spill_mem, &temp);

                        codegen.finish_code_sequence()
                    };

                    if spill_code_after.contains_key(&i) {
                        spill_code_after.get_mut(&i).unwrap().push(code);
                    } else {
                        spill_code_after.insert(i, vec![code]);
                    }

                    cf.mc_mut().replace_reg_for_inst(reg, temp.id(), i);
                }
            }
        }
    }

    // copy and insert the code
    unimplemented!()
}