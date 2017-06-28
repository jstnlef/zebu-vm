// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate docopt;

use self::docopt::Docopt;

use std;
use std::default::Default;

const USAGE: &'static str = "
zebu (mu implementation). Pass arguments as a strings to init it.

Usage:
  init_mu [options]

VM:
  --log-level=<level>                   logging level: none, error, warn, info, debug, trace, env [default: env]

Compiler:
  --disable-inline                      disable compiler function inlining
  --disable-regalloc-validate           disable register allocation validation
  --emit-debug-info                     emit debugging information

AOT Compiler:
  --aot-emit-dir=<dir>                  the emit directory for ahead-of-time compiling [default: emit]

  --bootimage-external-lib=<lib> ...           library that will be linked against when making bootimage [default: ]
  --bootimage-external-libpath=<path> ...      path for the libraries during bootimage generation [default: ]

Garbage Collection:
  --gc-disable-collection               disable collection
  --gc-immixspace-size=<kb>             immix space size (default 65536kb = 64mb) [default: 67108864]
  --gc-lospace-size=<kb>                large object space size (default 65536kb = 64mb) [default: 67108864]
  --gc-nthreads=<n>                     number of threads for parallel gc [default: 8]
";

// The fields need to be listed here in the order rust stores them in
rodal_struct!(VMOptions{
    flag_aot_emit_dir, flag_bootimage_external_lib, flag_bootimage_external_libpath,
    flag_gc_immixspace_size, flag_gc_lospace_size, flag_gc_nthreads,
    flag_log_level, flag_disable_inline, flag_disable_regalloc_validate, flag_emit_debug_info, flag_gc_disable_collection});
#[derive(Debug, Deserialize)]
pub struct VMOptions { // The comments here indicate the offset into the struct
    // VM
    pub flag_log_level: MuLogLevel, // +96

    // Compiler
    pub flag_disable_inline: bool, // +97
    pub flag_disable_regalloc_validate: bool, // +98
    pub flag_emit_debug_info: bool, // +99

    // AOT compiler
    pub flag_aot_emit_dir: String,      // +0
    pub flag_bootimage_external_lib: Vec<String>, // +24
    pub flag_bootimage_external_libpath: Vec<String>, // +48

    // GC
    pub flag_gc_disable_collection: bool, // +100
    pub flag_gc_immixspace_size: usize, // +72
    pub flag_gc_lospace_size: usize, // +80
    pub flag_gc_nthreads: usize // +88
}

//rodal_enum!(MuLogLevel{None, Error, Warn, Info, Debug, Trace, Env});
rodal_value!(MuLogLevel); // This enum has no fields with pointers, so just dump a strait value
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum MuLogLevel {
    None, Error, Warn, Info, Debug, Trace, Env
}
impl MuLogLevel {
    pub fn from_string(s: String) -> MuLogLevel {
        match s.as_str() {
            "none" => MuLogLevel::None,
            "error" => MuLogLevel::Error,
            "warn" => MuLogLevel::Warn,
            "info" => MuLogLevel::Info,
            "debug" => MuLogLevel::Debug,
            "trace" => MuLogLevel::Trace,
            _ => panic!("Unrecognised log level {}", s),
        }
    }
}

impl VMOptions {
    pub fn init(str: &str) -> VMOptions {
        info!("init vm options with: {:?}", str);

        let mut ret : VMOptions = Docopt::new(USAGE)
            .and_then(|d| d.argv(str.split_whitespace().into_iter()).parse())
            .unwrap_or_else(|e| e.exit()).deserialize().unwrap();

        info!("parsed as {:?}", ret);

        // at the moment disable collection for debugging
        ret.flag_gc_disable_collection = true;
        // at the moment always emit debug info
        ret.flag_emit_debug_info = true;
        // always disable register validation
        ret.flag_disable_regalloc_validate = true;

        ret
    }
}

impl Default for VMOptions {
    fn default() -> VMOptions {
        VMOptions::init("")
    }
}
