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

//! This module contains utility functions for linking generated code and
//! running tests for Zebu.

extern crate libloading as ll;

use ast::ir::*;
use compiler::*;
use std::sync::Arc;

use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

/// linking utilities for ahead-of-time compilation
pub mod aot;

/// gets a C compiler (for assembling and linking generated assembly code)
/// This function will check CC environment variable and return it.
/// Otherwise, it returns the default C compiler (clang).
fn get_c_compiler() -> String {
    use std::env;

    match env::var("CC") {
        Ok(val) => val,
        Err(_) => "clang".to_string()
    }
}

/// concatenates the given path related to Zebu root path
/// This function will check MU_ZEBU environment variable and use it as Zebu root path.
/// Otherwise, it uses the current directory as Zebu root path.
fn get_path_under_zebu(str: &'static str) -> PathBuf {
    use std::env;

    match env::var("MU_ZEBU") {
        Ok(v) => {
            let mut ret = PathBuf::from(v);
            ret.push(str);
            ret
        }
        Err(_) => PathBuf::from(str)
    }
}

/// executes the executable of the given path, checks its exit status
/// panics if this executable does not finish normally
pub fn exec_path(executable: PathBuf) -> Output {
    let run = Command::new(executable.as_os_str());
    exec_cmd(run)
}

/// executes the executable of the given path, does not check exit status
pub fn exec_path_nocheck(executable: PathBuf) -> Output {
    let run = Command::new(executable.as_os_str());
    exec_cmd_nocheck(run)
}

/// executes the given command, checks its exit status,
/// panics if this command does not finish normally
fn exec_cmd(cmd: Command) -> Output {
    let output = exec_cmd_nocheck(cmd);
    assert!(output.status.success());
    output
}

/// executes the given command, does not check exit status
fn exec_cmd_nocheck(mut cmd: Command) -> Output {
    info!("executing: {:?}", cmd);
    let output = match cmd.output() {
        Ok(res) => res,
        Err(e) => panic!("failed to execute: {}", e)
    };

    info!("---out---");
    info!("{}", String::from_utf8_lossy(&output.stdout));
    info!("---err---");
    info!("{}", String::from_utf8_lossy(&output.stderr));

    if output.status.signal().is_some() {
        info!("terminated by a signal: {}", output.status.signal().unwrap());
    }

    output
}

/// returns a name for dynamic library
#[cfg(not(feature = "sel4-rumprun"))]
#[cfg(target_os = "macos")]
pub fn get_dylib_name(name: &'static str) -> String {
    format!("lib{}.dylib", name)
}

/// returns a name for dynamic library
#[cfg(not(feature = "sel4-rumprun"))]
#[cfg(target_os = "linux")]
pub fn get_dylib_name(name: &'static str) -> String {
    format!("lib{}.so", name)
}

/// returns a name for dynamic library
/// Must not be used for sel4-rumprun
#[cfg(feature = "sel4-rumprun")]
pub fn get_dylib_name(name: &'static str) -> String {
    format!("lib{}.UNKNOWN", name)
}
