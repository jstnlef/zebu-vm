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
use std::any::Any;

use std::path;
use std::io::prelude::*;
use std::fs::File;
use vm::uir_output::create_emit_directory;
pub const EMIT_MUIR: bool = true;


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

pub struct UIRGen {
    name: &'static str,
    suffix: &'static str
}

impl UIRGen {
    pub fn new(suffix: &'static str) -> UIRGen {
        UIRGen {
            name: "UIRGen",
            suffix: suffix
        }
    }
}

#[allow(dead_code)]
fn emit_uir(suffix: &str, func_ver: &MuFunctionVersion, vm: &VM) {
    let func_ver_name = func_ver.name();

    // create emit directory
    create_emit_directory(vm);

    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push((*func_ver_name).clone() + suffix + ".uir");

    let mut file = match File::create(file_path.as_path()) {
        Err(why) => {
            panic!(
                "couldnt create muir dot {}: {}",
                file_path.to_str().unwrap(),
                why
            )
        }
        Ok(file) => file
    };
    let func_name = {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_ver.func_id).unwrap().read().unwrap();
        func.name()
    };
    emit_uir_inner(&mut file, func_name, func_ver);
}

fn emit_uir_inner(file: &mut File, func_name: MuName, func_ver: &MuFunctionVersion) {
    let f_content = func_ver.content.as_ref().unwrap();
    // self.abbreviate_name()
    writeln!(
        file,
        ".funcdef {} <{}> VERSION {}",
        func_name,
        func_ver.sig,
        func_ver.hdr.abbreviate_name()
    ).unwrap();
    writeln!(file, "{{").unwrap();
    // every basic block
    for (_, block) in f_content.blocks.iter() {
        write!(file, "\t{}", block.hdr.abbreviate_name()).unwrap();
        let block_content = block.content.as_ref().unwrap();
        // (args)
        write!(file, "(").unwrap();
        let mut first = true;
        for arg in &block_content.args {
            if !first {
                write!(file, " ").unwrap();
            }
            first = false;
            write!(file, "<{}> {}", arg.ty, arg).unwrap();
        }
        write!(file, ")").unwrap();

        if block_content.exn_arg.is_some() {
            write!(file, "[{}]", block_content.exn_arg.as_ref().unwrap()).unwrap();
        }
        writeln!(file, ":").unwrap();

        // all the instructions
        for inst in block_content.body.iter() {
            writeln!(file, "\t\t{}", inst.as_inst_ref()).unwrap();
        }

        // "];
        writeln!(file, "").unwrap();
    }
    writeln!(file, "}}").unwrap();
}

impl CompilerPass for UIRGen {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        if EMIT_MUIR {
            emit_uir(self.suffix, func, vm);
        }
    }
}
