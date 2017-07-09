// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(dead_code)]

use compiler::CompilerPass;
use ast::ir::*;
use vm::VM;
use compiler::backend::emit_code;
use std::any::Any;

use std::path;
use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;

pub const EMIT_MUIR : bool = true;
pub const EMIT_MC_DOT : bool = true;

pub fn create_emit_directory(vm: &VM) {
    use std::fs;
    match fs::create_dir(&vm.vm_options.flag_aot_emit_dir) {
        Ok(_) => {},
        Err(_) => {}
    }
}

fn create_emit_file(name: String, vm: &VM) -> File {
    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push(name);

    match File::create(file_path.as_path()) {
        Err(why) => panic!("couldn't create emit file {}: {}", file_path.to_str().unwrap(), why),
        Ok(file) => file
    }
}

pub struct CodeEmission {
    name: &'static str
}

impl CodeEmission {
    pub fn new() -> CodeEmission {
        CodeEmission {
            name: "Code Emission"
        }
    }
}

#[allow(dead_code)]
pub fn emit_mu_types(vm: &VM) {
    if EMIT_MUIR {
        create_emit_directory(vm);

        let mut file_path = path::PathBuf::new();
        file_path.push(&vm.vm_options.flag_aot_emit_dir);
        file_path.push("___types.muty");
        let mut file = match File::create(file_path.as_path()) {
            Err(why) => panic!("couldn't create mu types file {}: {}", file_path.to_str().unwrap(), why),
            Ok(file) => file
        };

        {
            use ast::types::*;

            let ty_guard = vm.types().read().unwrap();
            let struct_map = STRUCT_TAG_MAP.read().unwrap();
            let hybrid_map = HYBRID_TAG_MAP.read().unwrap();

            for ty in ty_guard.values() {
                if ty.is_struct() {
                    write!(file, "{}", ty).unwrap();

                    let struct_ty = struct_map.get(&ty.get_struct_hybrid_tag().unwrap()).unwrap();
                    writeln!(file, " -> {}", struct_ty).unwrap();
                    writeln!(file, "  {}", vm.get_backend_type_info(ty.id())).unwrap();
                } else if ty.is_hybrid() {
                    write!(file, "{}", ty).unwrap();
                    let hybrid_ty = hybrid_map.get(&ty.get_struct_hybrid_tag().unwrap()).unwrap();
                    writeln!(file, " -> {}", hybrid_ty).unwrap();
                    writeln!(file, "  {}", vm.get_backend_type_info(ty.id())).unwrap();
                } else {
                    // we only care about struct
                }
            }
        }


    }
}

#[allow(dead_code)]
fn emit_muir(func: &MuFunctionVersion, vm: &VM) {
    let func_name = func.name();

    // create emit directory
    create_emit_directory(vm);

    // final IR
    {
        let mut file_path = path::PathBuf::new();
        file_path.push(&vm.vm_options.flag_aot_emit_dir);
        file_path.push(func_name.clone() + ".muir");
        let mut file = match File::create(file_path.as_path()) {
            Err(why) => panic!("couldn't create muir file {}: {}", file_path.to_str().unwrap(), why),
            Ok(file) => file
        };

        write!(file, "{:?}", func).unwrap();
    }

    // original IR (source/input)
    {
        let mut file_path = path::PathBuf::new();
        file_path.push(&vm.vm_options.flag_aot_emit_dir);
        file_path.push(func_name.clone() + "_orig.muir");
        let mut file = match File::create(file_path.as_path()) {
            Err(why) => panic!("couldn't create muir file {}: {}", file_path.to_str().unwrap(), why),
            Ok(file) => file
        };

        writeln!(file, "FuncVer {} of Func #{}", func.hdr, func.func_id).unwrap();
        writeln!(file, "Signature: {}", func.sig).unwrap();
        writeln!(file, "IR:").unwrap();
        if func.get_orig_ir().is_some() {
            writeln!(file, "{:?}", func.get_orig_ir().as_ref().unwrap()).unwrap();
        } else {
            writeln!(file, "Empty").unwrap();
        }
    }
}

fn emit_muir_dot(func: &MuFunctionVersion, vm: &VM) {
    let func_name = func.name();

    // create emit directory
    create_emit_directory(vm);

    // original
    {
        let mut file_path = path::PathBuf::new();
        file_path.push(&vm.vm_options.flag_aot_emit_dir);
        file_path.push(func_name.clone() + ".orig.dot");
        let mut file = match File::create(file_path.as_path()) {
            Err(why) => panic!("couldn't create muir dot {}: {}", file_path.to_str().unwrap(), why),
            Ok(file) => file
        };

        emit_muir_dot_inner(&mut file, func_name.clone(), func.get_orig_ir().unwrap());
    }

    // final
    {
        let mut file_path = path::PathBuf::new();
        file_path.push(&vm.vm_options.flag_aot_emit_dir);
        file_path.push(func_name.clone() + ".dot");

        let mut file = match File::create(file_path.as_path()) {
            Err(why) => panic!("couldnt create muir dot {}: {}", file_path.to_str().unwrap(), why),
            Ok(file) => file
        };

        emit_muir_dot_inner(&mut file, func_name.clone(), func.content.as_ref().unwrap());
    }
}

fn emit_muir_dot_inner(file: &mut File,
                       f_name: String,
                       f_content: &FunctionContent) {
    use utils::vec_utils;

    // digraph func {
    writeln!(file, "digraph {} {{", mangle_name(f_name)).unwrap();

    // node shape: rect
    writeln!(file, "node [shape=rect];").unwrap();

    // every graph node (basic block)
    for (id, block) in f_content.blocks.iter() {
        let block_name = block.name();
        // BBid [label = "name
        write!(file, "BB{} [label = \"[{}]{} ", *id, *id, &block_name).unwrap();

        let block_content = block.content.as_ref().unwrap();

        // (args)
        write!(file, "{}", vec_utils::as_str(&block_content.args)).unwrap();
        if block_content.exn_arg.is_some() {
            // [exc_arg]
            write!(file, "[{}]", block_content.exn_arg.as_ref().unwrap()).unwrap();
        }

        write!(file, ":\\l\\l").unwrap();

        // all the instructions
        for inst in block_content.body.iter() {
            write!(file, "{}\\l", inst).unwrap();
        }

        // "];
        writeln!(file, "\"];").unwrap();
    }

    // every edge
    for (id, block) in f_content.blocks.iter() {
        use ast::inst::Instruction_::*;

        let cur_block = *id;

        // get last instruction
        let last_inst = block.content.as_ref().unwrap().body.last().unwrap();

        match last_inst.v {
            TreeNode_::Instruction(ref inst) => {
                let ref ops = inst.ops;

                match inst.v {
                    Branch1(ref dest) => {
                        writeln!(file, "BB{} -> BB{} [label = \"{}\"];",
                            cur_block, dest.target, vec_utils::as_str(&dest.get_arguments(&ops))
                        ).unwrap();
                    }
                    Branch2{ref true_dest, ref false_dest, ..} => {
                        writeln!(file, "BB{} -> BB{} [label = \"true: {}\"]",
                            cur_block, true_dest.target, vec_utils::as_str(&true_dest.get_arguments(&ops))
                        ).unwrap();
                        writeln!(file, "BB{} -> BB{} [label = \"false: {}\"]",
                            cur_block, false_dest.target, vec_utils::as_str(&false_dest.get_arguments(&ops))
                        ).unwrap();
                    }
                    Switch{ref default, ref branches, ..} => {
                        for &(op, ref dest) in branches.iter() {
                            writeln!(file, "BB{} -> BB{} [label = \"case {}: {}\"]",
                                cur_block, dest.target, ops[op], vec_utils::as_str(&dest.get_arguments(&ops))
                            ).unwrap();
                        }

                        writeln!(file, "BB{} -> BB{} [label = \"default: {}\"]",
                            cur_block, default.target, vec_utils::as_str(&default.get_arguments(&ops))
                        ).unwrap();
                    }
                    Call{ref resume, ..}
                    | CCall{ref resume, ..}
                    | SwapStack{ref resume, ..}
                    | ExnInstruction{ref resume, ..} => {
                        let ref normal = resume.normal_dest;
                        let ref exn    = resume.exn_dest;

                        writeln!(file, "BB{} -> BB{} [label = \"normal: {}\"];",
                            cur_block, normal.target, vec_utils::as_str(&normal.get_arguments(&ops))
                        ).unwrap();

                        writeln!(file, "BB{} -> BB{} [label = \"exception: {}\"];",
                            cur_block, exn.target, vec_utils::as_str(&exn.get_arguments(&ops))
                        ).unwrap();
                    }
                    Watchpoint{ref id, ref disable_dest, ref resume, ..} if id.is_some() => {
                        let ref normal = resume.normal_dest;
                        let ref exn    = resume.exn_dest;

                        if id.is_some() {
                            let disable_dest = disable_dest.as_ref().unwrap();
                            writeln!(file, "BB{} -> {} [label = \"disabled: {}\"];",
                                 cur_block, disable_dest.target, vec_utils::as_str(&disable_dest.get_arguments(&ops))
                            ).unwrap();
                        }


                        writeln!(file, "BB{} -> BB{} [label = \"normal: {}\"];",
                             cur_block, normal.target, vec_utils::as_str(&normal.get_arguments(&ops))
                        ).unwrap();

                        writeln!(file, "BB{} -> BB{} [label = \"exception: {}\"];",
                             cur_block, exn.target, vec_utils::as_str(&exn.get_arguments(&ops))
                        ).unwrap();
                    }
                    WPBranch{ref disable_dest, ref enable_dest, ..} => {
                        writeln!(file, "BB{} -> BB{} [label = \"disabled: {}\"];",
                            cur_block, disable_dest.target, vec_utils::as_str(&disable_dest.get_arguments(&ops))
                        ).unwrap();

                        writeln!(file, "BB{} -> BB{} [label = \"enabled: {}\"];",
                            cur_block, enable_dest.target, vec_utils::as_str(&enable_dest.get_arguments(&ops))
                        ).unwrap();
                    }
                    Return(_)
                    | Throw(_)
                    | ThreadExit
                    | TailCall(_) => {}

                    _ => {
                        panic!("unexpected terminating instruction: {}", inst);
                    }
                }
            },
            TreeNode_::Value(_) => {
                panic!("expected last tree node to be an instruction");
            }
        }
    }

    writeln!(file, "}}").unwrap();
}

fn emit_mc_dot(func: &MuFunctionVersion, vm: &VM) {
    let func_name = func.name();

    // create emit directory/file
    create_emit_directory(vm);
    let mut file = create_emit_file(func_name.clone() + ".mc.dot", &vm);

    // diagraph func {
    writeln!(file, "digraph {} {{", mangle_name(func_name)).unwrap();
    // node shape: rect
    writeln!(file, "node [shape=rect];").unwrap();

    let compiled_funcs = vm.compiled_funcs().read().unwrap();
    let cf = compiled_funcs.get(&func.id()).unwrap().read().unwrap();
    let mc = cf.mc();

    let blocks = mc.get_all_blocks();

    type DotID = usize;
    let name_id_map : HashMap<MuName, DotID> = {
        let mut ret = HashMap::new();
        let mut index = 0;

        for block_name in blocks.iter() {
            ret.insert(block_name.clone(), index);
            index += 1;
        }

        ret
    };
    let id = |x: MuName| name_id_map.get(&x).unwrap();

    for block_name in blocks.iter() {
        // BB [label = "
        write!(file, "{} [label = \"{}:\\l\\l", id(block_name.clone()), block_name).unwrap();

        for inst in mc.get_block_range(&block_name).unwrap() {
            file.write_all(&mc.emit_inst(inst)).unwrap();
            write!(file, "\\l").unwrap();
        }

        // "];
        writeln!(file, "\"];").unwrap();
    }

    for block_name in blocks.iter() {
        let end_inst = mc.get_block_range(block_name).unwrap().end;

        for succ in mc.get_succs(mc.get_last_inst(end_inst).unwrap()).into_iter() {
                match mc.get_block_for_inst(*succ) {
                Some(target) => {
                    let source_id = id(block_name.clone());
                    let target_id = id(target.clone());
                    writeln!(file, "{} -> {};", source_id, target_id).unwrap();
                }
                None => {
                    panic!("cannot find succesor {} for block {}", succ, block_name);
                }
            }
        }
    }

    writeln!(file, "}}").unwrap();
}

impl CompilerPass for CodeEmission {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        emit_code(func, vm);

        if EMIT_MUIR {
            emit_muir_dot(func, vm);
        }

        if EMIT_MC_DOT {
            emit_mc_dot(func, vm);
        }
    }
}
