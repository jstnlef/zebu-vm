extern crate gcc;

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libgc_clib_x64.a", &["src/heap/gc/clib_x64.c"]);
}

#[cfg(target_os = "linux")]
#[cfg(target_arch = "aarch64")]
fn main() {
    gcc::compile_library("libgc_clib_aarch64.a", &["src/heap/gc/clib_aarch64.c"]);
}

// This is here to enable cross compiling from windows/x86_64 to linux/aarch64
#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libgc_clib_aarch64.a", &["src/heap/gc/clib_aarch64.c"]);
}
