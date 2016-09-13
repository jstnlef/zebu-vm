extern crate gcc;

#[cfg(target_os = "macos")]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libgc_clib_x64.a", &["src/heap/gc/clib_x64.c"]);
}