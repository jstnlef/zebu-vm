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

extern crate rustc_serialize;
extern crate docopt;

use self::docopt::Docopt;

use std::default::Default;

const USAGE: &'static str = "
zebu (mu implementation). Pass arguments as a strings to init it.

Usage:
  init_mu [options]

VM:
  --log-level=<level>                   logging level: none, error, warn, info, debug, trace [default: trace]

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

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct VMOptions {
    // VM
    pub flag_log_level: MuLogLevel,

    // Compiler
    pub flag_disable_inline: bool,
    pub flag_disable_regalloc_validate: bool,
    pub flag_emit_debug_info: bool,

    // AOT compiler
    pub flag_aot_emit_dir: String,
    pub flag_bootimage_external_lib: Vec<String>,
    pub flag_bootimage_external_libpath: Vec<String>,

    // GC
    pub flag_gc_disable_collection: bool,
    pub flag_gc_immixspace_size: usize,
    pub flag_gc_lospace_size: usize,
    pub flag_gc_nthreads: usize
}

#[derive(Debug, Clone, Copy, RustcDecodable, RustcEncodable)]
pub enum MuLogLevel {
    None, Error, Warn, Info, Debug, Trace
}

impl VMOptions {
    pub fn init(str: &str) -> VMOptions {
        info!("init vm options with: {:?}", str);

        let mut ret : VMOptions = Docopt::new(USAGE)
            .and_then(|d| d.argv(str.split_whitespace().into_iter()).parse())
            .unwrap_or_else(|e| e.exit()).decode().unwrap();

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
