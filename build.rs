extern crate gcc;

#[cfg(target_os = "macos")]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libruntime.a", &["src/runtime/runtime_x64_macos.c"]);
    gcc::compile_library("libgc_clib_x64.a", &["src/runtime/mm/heap/gc/clib_x64.c"]);
    
    gcc::Config::new().flag("-O3")
                     .file("src/runtime/swap_stack_x64_macos.s")
                     .compile("libswap_stack.a"); 
}