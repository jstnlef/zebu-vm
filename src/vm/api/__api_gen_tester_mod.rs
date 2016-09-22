// Invoke "python3 muapi2rustapi.py", and then
// invoke "rustc --test __api_gen_tester_mod.rs -o /tmp/api_gen_tester_junk"
// to test whether the generated code compiles.

mod api_c;
mod api_bridge;
mod api_impl;

