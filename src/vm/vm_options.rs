extern crate rustc_serialize;
extern crate docopt;

use self::docopt::Docopt;

use std::default::Default;

const USAGE: &'static str = "
zebu (mu implementation). Pass arguments as a strings to init it.

Usage:
  init_mu [options]

VM:
  --log-level=<level>               logging level: none, error, warn, info, debug, trace [default: trace]

Compiler:
  --disable-inline                  disable compiler function inlining

AOT Compiler:
  --aot-emit-dir=<dir>              the emit directory for ahead-of-time compiling [default: emit]

Garbage Collection:
  --gc-immixspace-size=<kb>         immix space size (default 65536kb = 64mb) [default: 65536]
  --gc-lospace-size=<kb>            large object space size (default 65536kb = 64mb) [default: 65536]
  --gc-nthreads=<n>                 number of threads for parallel gc [default: 8]
";

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct VMOptions {
    pub flag_log_level: MuLogLevel,
    pub flag_disable_inline: bool,
    pub flag_aot_emit_dir: String,
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
        println!("init vm options with: {:?}", str);

        let ret : VMOptions = Docopt::new(USAGE)
            .and_then(|d| d.argv(str.split_whitespace().into_iter()).parse())
            .unwrap_or_else(|e| e.exit()).decode().unwrap();

        println!("parsed as {:?}", ret);

        ret
    }
}

impl Default for VMOptions {
    fn default() -> VMOptions {
        VMOptions::init("")
    }
}