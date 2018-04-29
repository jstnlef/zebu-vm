use ast::ir::*;
use vm::VM;
use std::path;
use std::io::prelude::*;
use std::fs::File;

/// should emit Mu IR dot graph?
pub const EMIT_MUIR: bool = true;

pub fn emit_uir(suffix: &str, vm: &VM) {
    if EMIT_MUIR {
        emit_mu_types(suffix, vm);
        emit_mu_globals(suffix, vm);
        emit_mu_funcdecls(suffix, vm);
    }
}
fn emit_mu_types(suffix: &str, vm: &VM) {
    create_emit_directory(vm);

    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push("___types".to_string() + suffix + ".uir");
    let mut file = match File::create(file_path.as_path()) {
        Err(why) => panic!(
            "couldn't create mu types file {}: {}",
            file_path.to_str().unwrap(),
            why
        ),
        Ok(file) => file
    };

    {
        use ast::types::*;

        let ty_guard = vm.types().read().unwrap();
        let struct_map = STRUCT_TAG_MAP.read().unwrap();
        let hybrid_map = HYBRID_TAG_MAP.read().unwrap();

        for ty in ty_guard.values() {
            if ty.is_struct() {
                write!(file, ".typedef {} = ", ty.hdr).unwrap();

                let struct_ty = struct_map
                    .get(&ty.get_struct_hybrid_tag().unwrap())
                    .unwrap();
                writeln!(file, "{}", struct_ty).unwrap();
                writeln!(file, "\n\t/*{}*/", vm.get_backend_type_info(ty.id())).unwrap();
            } else if ty.is_hybrid() {
                write!(file, ".typedef {} = ", ty.hdr).unwrap();
                let hybrid_ty = hybrid_map
                    .get(&ty.get_struct_hybrid_tag().unwrap())
                    .unwrap();
                writeln!(file, "{}", hybrid_ty).unwrap();
                writeln!(file, "\n\t/*{}*/", vm.get_backend_type_info(ty.id())).unwrap();
            } else {
                // we only care about struct
            }
        }
    }
}
fn emit_mu_globals(suffix: &str, vm: &VM) {
    create_emit_directory(vm);

    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push("___globals".to_string() + suffix + ".uir");
    let mut file = match File::create(file_path.as_path()) {
        Err(why) => panic!(
            "couldn't create mu globals file {}: {}",
            file_path.to_str().unwrap(),
            why
        ),
        Ok(file) => file
    };

    let global_guard = vm.globals().read().unwrap();

    for g in global_guard.values() {
        writeln!(
            file,
            ".global {}<{}>",
            g.name(),
            g.ty.get_referent_ty().unwrap()
        ).unwrap();
    }
}
fn emit_mu_funcdecls(suffix: &str, vm: &VM) {
    create_emit_directory(vm);

    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push("___funcdecls".to_string() + suffix + ".uir");
    let mut file = match File::create(file_path.as_path()) {
        Err(why) => panic!(
            "couldn't create mu funcdecls file {}: {}",
            file_path.to_str().unwrap(),
            why
        ),
        Ok(file) => file
    };

    let funcs_guard = vm.funcs().read().unwrap();

    for f in funcs_guard.values() {
        let f_lock = f.read().unwrap();
        writeln!(file, ".funcdecl {}<{}>", f_lock.name(), f_lock.sig).unwrap();
    }
}

pub fn create_emit_directory(vm: &VM) {
    use std::fs;
    match fs::create_dir(&vm.vm_options.flag_aot_emit_dir) {
        Ok(_) => {}
        Err(_) => {}
    }
}
