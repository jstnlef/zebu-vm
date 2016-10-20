extern crate gcc;

#[cfg(target_os = "macos")]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libruntime.a", &["src/runtime/runtime_x64_sysv.c"]);
    
    gcc::Config::new().flag("-O3")
                     .file("src/runtime/swap_stack_x64_sysv.S")
                     .compile("libswap_stack.a"); 
}

#[cfg(target_os = "linux")]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libruntime.a", &["src/runtime/runtime_x64_sysv.c"]);
    
    gcc::Config::new().flag("-O3")
                     .file("src/runtime/swap_stack_x64_sysv.S")
                     .compile("libswap_stack.a"); 
}
