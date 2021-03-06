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
pub use vm::uir_output::create_emit_directory;
/// should emit machien code dot graph?
pub const EMIT_MC_DOT: bool = true;

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

impl CompilerPass for CodeEmission {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        // emit the actual code
        emit_code(func, vm);

        // emit debug graphs
        if EMIT_MC_DOT {
            emit_mc_dot(func, vm);
        }
    }
}

/// creates an file to write, panics if the creation fails
fn create_emit_file(name: String, vm: &VM) -> File {
    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push(name);

    match File::create(file_path.as_path()) {
        Err(why) => {
            panic!(
                "couldn't create emit file {}: {}",
                file_path.to_str().unwrap(),
                why
            )
        }
        Ok(file) => file
    }
}


fn emit_mc_dot(func: &MuFunctionVersion, vm: &VM) {
    let func_name = func.name();

    // create emit directory/file
    create_emit_directory(vm);
    let mut file = create_emit_file((*func_name).clone() + ".mc.dot", &vm);

    // diagraph func {
    writeln!(file, "digraph {} {{", mangle_name(func_name)).unwrap();
    // node shape: rect
    writeln!(file, "node [shape=rect];").unwrap();

    let compiled_funcs = vm.compiled_funcs().read().unwrap();
    let cf = compiled_funcs.get(&func.id()).unwrap().read().unwrap();
    let mc = cf.mc();

    let blocks = mc.get_all_blocks();

    type DotID = usize;
    let name_id_map: HashMap<MuName, DotID> = {
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
        write!(
            file,
            "{} [label = \"{}:\\l\\l",
            id(block_name.clone()),
            block_name
        ).unwrap();

        for inst in mc.get_block_range(&block_name).unwrap() {
            file.write_all(&mc.emit_inst(inst)).unwrap();
            write!(file, "\\l").unwrap();
        }

        // "];
        writeln!(file, "\"];").unwrap();
    }

    for block_name in blocks.iter() {
        let end_inst = mc.get_block_range(block_name).unwrap().end;

        for succ in mc.get_succs(mc.get_last_inst(end_inst).unwrap())
            .into_iter()
        {
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
