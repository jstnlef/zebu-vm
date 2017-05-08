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

const EMIT_MUIR : bool = true;
const EMIT_MC_DOT : bool = true;

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
fn emit_muir(func: &MuFunctionVersion, vm: &VM) {
    let func_name = match func.name() {
        Some(name) => name,
        None => {
            // use func name
            vm.name_of(func.func_id)
        }
    };

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

        file.write_fmt(format_args!("{:?}", func)).unwrap();
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

        file.write_fmt(format_args!("FuncVer {} of Func #{}\n", func.hdr, func.func_id)).unwrap();
        file.write_fmt(format_args!("Signature: {}\n", func.sig)).unwrap();
        file.write_fmt(format_args!("IR:\n")).unwrap();
        if func.get_orig_ir().is_some() {
            file.write_fmt(format_args!("{:?}\n", func.get_orig_ir().as_ref().unwrap())).unwrap();
        } else {
            file.write_fmt(format_args!("Empty\n")).unwrap();
        }
    }
}

fn emit_muir_dot(func: &MuFunctionVersion, vm: &VM) {
    let func_name = match func.name() {
        Some(name) => name,
        None => {
            // use func name
            vm.name_of(func.func_id)
        }
    };

    // create emit directory
    create_emit_directory(vm);

    // original
    {
        let mut file_path = path::PathBuf::new();
        file_path.push(&vm.vm_options.flag_aot_emit_dir);
        file_path.push(func_name.clone() + "_orig.dot");
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
    file.write_fmt(format_args!("digraph {} {{\n", f_name)).unwrap();

    // node shape: rect
    file.write("node [shape=rect];\n".as_bytes()).unwrap();

    // every graph node (basic block)
    for (id, block) in f_content.blocks.iter() {
        let block_name = block.name().unwrap();
        // BBid [label = "name
        file.write_fmt(format_args!("BB{} [label = \"{} ", *id, &block_name)).unwrap();

        let block_content = block.content.as_ref().unwrap();

        // (args)
        file.write_fmt(format_args!("{}", vec_utils::as_str(&block_content.args))).unwrap();
        if block_content.exn_arg.is_some() {
            // [exc_arg]
            file.write_fmt(format_args!("[{}]", block_content.exn_arg.as_ref().unwrap())).unwrap();
        }
        // :\n\n
        file.write(":\\n\\n".as_bytes()).unwrap();

        // all the instructions
        for inst in block_content.body.iter() {
            file.write_fmt(format_args!("{}\\n", inst)).unwrap();
        }

        // "];
        file.write("\"];\n".as_bytes()).unwrap();
    }

    // every edge
    for (id, block) in f_content.blocks.iter() {
        use ast::inst::Instruction_::*;

        let cur_block = *id;

        // get last instruction
        let last_inst = block.content.as_ref().unwrap().body.last().unwrap();

        match last_inst.v {
            TreeNode_::Instruction(ref inst) => {
                let ops = inst.ops.read().unwrap();

                match inst.v {
                    Branch1(ref dest) => {
                        file.write_fmt(format_args!("BB{} -> BB{} [label = \"{}\"];\n",
                            cur_block, dest.target, vec_utils::as_str(&dest.get_arguments(&ops))
                        )).unwrap();
                    }
                    Branch2{ref true_dest, ref false_dest, ..} => {
                        file.write_fmt(format_args!("BB{} -> BB{} [label = \"true: {}\"]\n",
                            cur_block, true_dest.target, vec_utils::as_str(&true_dest.get_arguments(&ops))
                        )).unwrap();
                        file.write_fmt(format_args!("BB{} -> BB{} [label = \"false: {}\"]\n",
                            cur_block, false_dest.target, vec_utils::as_str(&false_dest.get_arguments(&ops))
                        )).unwrap();
                    }
                    Switch{ref default, ref branches, ..} => {
                        for &(op, ref dest) in branches.iter() {
                            file.write_fmt(format_args!("BB{} -> BB{} [label = \"case {}: {}\"]\n",
                                cur_block, dest.target, ops[op], vec_utils::as_str(&dest.get_arguments(&ops))
                            )).unwrap();
                        }

                        file.write_fmt(format_args!("BB{} -> BB{} [label = \"default: {}\"]\n",
                            cur_block, default.target, vec_utils::as_str(&default.get_arguments(&ops))
                        )).unwrap();
                    }
                    Call{ref resume, ..}
                    | CCall{ref resume, ..}
                    | SwapStack{ref resume, ..}
                    | ExnInstruction{ref resume, ..} => {
                        let ref normal = resume.normal_dest;
                        let ref exn    = resume.exn_dest;

                        file.write_fmt(format_args!("BB{} -> BB{} [label = \"normal: {}\"];\n",
                            cur_block, normal.target, vec_utils::as_str(&normal.get_arguments(&ops))
                        )).unwrap();

                        file.write_fmt(format_args!("BB{} -> BB{} [label = \"exception: {}\"];\n",
                            cur_block, exn.target, vec_utils::as_str(&exn.get_arguments(&ops))
                        )).unwrap();
                    }
                    Watchpoint{ref id, ref disable_dest, ref resume, ..} if id.is_some() => {
                        let ref normal = resume.normal_dest;
                        let ref exn    = resume.exn_dest;

                        if id.is_some() {
                            let disable_dest = disable_dest.as_ref().unwrap();
                            file.write_fmt(format_args!("BB{} -> {} [label = \"disabled: {}\"];\n",
                                 cur_block, disable_dest.target, vec_utils::as_str(&disable_dest.get_arguments(&ops))
                            )).unwrap();
                        }


                        file.write_fmt(format_args!("BB{} -> BB{} [label = \"normal: {}\"];\n",
                             cur_block, normal.target, vec_utils::as_str(&normal.get_arguments(&ops))
                        )).unwrap();

                        file.write_fmt(format_args!("BB{} -> BB{} [label = \"exception: {}\"];\n",
                             cur_block, exn.target, vec_utils::as_str(&exn.get_arguments(&ops))
                        )).unwrap();
                    }
                    WPBranch{ref disable_dest, ref enable_dest, ..} => {
                        file.write_fmt(format_args!("BB{} -> BB{} [label = \"disabled: {}\"];\n",
                            cur_block, disable_dest.target, vec_utils::as_str(&disable_dest.get_arguments(&ops))
                        )).unwrap();

                        file.write_fmt(format_args!("BB{} -> BB{} [label = \"enabled: {}\"];\n",
                            cur_block, enable_dest.target, vec_utils::as_str(&enable_dest.get_arguments(&ops))
                        )).unwrap();
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

    file.write("}\n".as_bytes()).unwrap();
}

fn emit_mc_dot(func: &MuFunctionVersion, vm: &VM) {
    let func_name = match func.name() {
        Some(name) => name,
        None => {
            // use func name
            vm.name_of(func.func_id)
        }
    };

    // create emit directory/file
    create_emit_directory(vm);
    let mut file = create_emit_file(func_name.clone() + "_mc.dot", &vm);

    // diagraph func {
    file.write_fmt(format_args!("digraph {} {{\n", func_name)).unwrap();
    // node shape: rect
    file.write("node [shape=rect];\n".as_bytes()).unwrap();

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
        file.write_fmt(format_args!("{} [label = \"{}:\\n\\n", id(block_name.clone()), block_name)).unwrap();

        for inst in mc.get_block_range(&block_name).unwrap() {
            file.write(&mc.emit_inst(inst)).unwrap();
            file.write("\\l".as_bytes()).unwrap();
        }

        // "];
        file.write("\"];\n".as_bytes()).unwrap();
    }

    for block_name in blocks.iter() {
        let end_inst = mc.get_block_range(block_name).unwrap().end;

        for succ in mc.get_succs(mc.get_last_inst(end_inst).unwrap()).into_iter() {
            match mc.get_block_for_inst(*succ) {
                Some(target) => {
                    let source_id = id(block_name.clone());
                    let target_id = id(target.clone());
                    file.write_fmt(format_args!("{} -> {};\n", source_id, target_id)).unwrap();
                }
                None => {
                    panic!("cannot find block for inst {}", succ);
                }
            }
        }
    }

    file.write("}\n".as_bytes()).unwrap();
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
