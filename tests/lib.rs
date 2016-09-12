extern crate mu;
extern crate log;
extern crate simple_logger;

mod test_ir;
mod test_compiler;
mod test_runtime;
mod test_api;

#[macro_use]
mod common {
    use std::fmt;
    
    pub fn assert_vector_ordered <T: fmt::Debug> (left: &Vec<T>, right: &Vec<T>) {
        assert_debug_str(left, right);
    }
    
    pub fn assert_vector_no_order <T: Ord + fmt::Debug + Clone> (left: &Vec<T>, right: &Vec<T>) {
        let mut left_clone = left.clone();
        left_clone.sort();
        let mut right_clone = right.clone();
        right_clone.sort();
        
        assert_debug_str(left_clone, right_clone);
    }
    
    pub fn assert_debug_str<T: fmt::Debug, U: fmt::Debug> (left: T, right: U) {
        assert_eq!(format!("{:?}", left), format!("{:?}", right))
    }
}

mod aot {
    use mu::ast::ir::MuName;
    use mu::runtime;
    use mu::compiler::backend;
    use std::path::PathBuf;
    use std::process::Command;        
    
    fn link (files: Vec<PathBuf>, out: PathBuf) -> PathBuf {
        let mut gcc = Command::new("gcc");
        
        for file in files {
            println!("link with {:?}", file.as_path());
            gcc.arg(file.as_path());
        }
        
        println!("output as {:?}", out.as_path());
        gcc.arg("-o");
        gcc.arg(out.as_os_str());
        
        println!("executing: {:?}", gcc);
        
        let status = gcc.status().expect("failed to link generated code");
        assert!(status.success());
        
        out
    }
    
    pub fn link_primordial (funcs: Vec<MuName>, out: &str) -> PathBuf {
        let emit_dir = PathBuf::from(backend::AOT_EMIT_DIR);        
        
        let files : Vec<PathBuf> = {
            use std::fs;
            
            let mut ret = vec![];
            
            // all interested mu funcs
            for func in funcs {
                let mut p = emit_dir.clone();
                p.push(func);
                p.set_extension("s");
                
                ret.push(p);
            }
            
            // mu context
            let mut p = emit_dir.clone();
            p.push(backend::AOT_EMIT_CONTEXT_FILE);
            ret.push(p);
            
            // copy primoridal entry
            let source   = PathBuf::from(runtime::PRIMORDIAL_ENTRY);
            let mut dest = PathBuf::from(backend::AOT_EMIT_DIR);
            dest.push("main.c");
            fs::copy(source.as_path(), dest.as_path()).unwrap();
            // include the primordial C main
            ret.push(dest);
            
            // include mu static lib
            let libmu = PathBuf::from("target/debug/libmu.a");
            ret.push(libmu);
            
            ret
        };
        
        let mut out_path = emit_dir.clone();
        out_path.push(out);
        
        link(files, out_path)
    }
    
    pub fn execute(exec: PathBuf) {
        let mut run = Command::new(exec.as_os_str());
        
        let output = run.output().expect("failed to execute");
        
        println!("---out---");
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("---err---");
        println!("{}", String::from_utf8_lossy(&output.stderr));
        
        assert!(output.status.success());
    }
}