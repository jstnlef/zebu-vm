#![allow(dead_code)]

use compiler::CompilerPass;
use ast::ir::*;
use vm::VM;
use compiler::backend::emit_code;
use std::any::Any;

const EMIT_MUIR : bool = true;

pub fn create_emit_directory(vm: &VM) {
    use std::fs;
    match fs::create_dir(&vm.vm_options.flag_aot_emit_dir) {
        Ok(_) => {},
        Err(_) => {}
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

    fn emit_muir(&self, func: &MuFunctionVersion, vm: &VM) {
        use std::path;
        use std::io::prelude::*;
        use std::fs::File;

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
            self.emit_muir(func, vm);
        }
    }
}
