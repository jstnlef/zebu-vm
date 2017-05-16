extern crate gcc;

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libruntime.a", &["src/runtime/runtime_x64_sysv.c"]);
    
    gcc::Config::new().flag("-O3").flag("-c")
                     .file("src/runtime/swap_stack_x64_sysv.S")
                     .compile("libswap_stack.a"); 
}

#[cfg(target_os = "linux")]
#[cfg(target_arch = "aarch64")]
fn main() {
    gcc::compile_library("libruntime.a", &["src/runtime/runtime_aarch64_sysv.c"]);

    gcc::Config::new().flag("-O3").flag("-c")
        .file("src/runtime/swap_stack_aarch64_sysv.S")
        .compile("libswap_stack.a");
}

// This is here to enable cross compiling from windows/x86_64 to linux/aarch64
#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libruntime.a", &["src/runtime/runtime_aarch64_sysv.c"]);

    gcc::Config::new().flag("-O3").flag("-c")
        .file("src/runtime/swap_stack_aarch64_sysv.S")
        .compile("libswap_stack.a");
}