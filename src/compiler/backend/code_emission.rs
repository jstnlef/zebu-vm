#![allow(dead_code)]

use compiler::CompilerPass;
use ast::ir::*;
use vm::context::VMContext;

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

    fn visit_function(&mut self, vm_context: &VMContext, func: &mut MuFunctionVersion) {
        use std::io::prelude::*;
        use std::fs::File;
        use std::fs;

        let compiled_funcs = vm_context.compiled_funcs().read().unwrap();
        let cf = compiled_funcs.get(func.fn_name).unwrap().borrow();

        let code = cf.mc.emit();

        // FIXME: this is only for asm backend
        const EMIT_DIR : &'static str = "emit";
//        match fs::remove_dir_all(EMIT_DIR) {
//            Ok(dir) => {},
//            Err(_) => {}
//        }
        match fs::create_dir(EMIT_DIR) {
            Ok(_) => {},
            Err(_) => {}
        }

        let file_name = EMIT_DIR.to_string() + "/" + func.fn_name + ".s";
        let mut file = match File::create(file_name.clone()) {
            Err(why) => panic!("couldn't create emission file {}: {}", file_name, why),
            Ok(file) => file
        };

        match file.write_all(code.as_slice()) {
            Err(why) => panic!("couldn'd write to file {}: {}", file_name, why),
            Ok(_) => println!("emit code to {}", file_name)
        }
    }
}
