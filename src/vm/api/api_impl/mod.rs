/*!
 * This module contains the high-level implementation of the Mu API.
 *
 * Structs are written in idiomatic Rust code. The internal structures of these structs are
 * implementation-specific. Methods are defined using `impl`. 
 */

#![allow(unused_imports)]   // work in progress
#![allow(unused_variables)] // stubs
#![allow(dead_code)]        // stubs

mod muvm;
mod muctx;
mod muirbuilder;

pub use self::muvm::*;
pub use self::muctx::*;
pub use self::muirbuilder::*;

mod common {
    pub use std::os::raw::*;
    pub use std::ptr;
    pub use std::slice;
    pub use std::ffi::CStr;
    pub use std::ffi::CString;

    pub use std::collections::HashMap;
    pub use std::collections::HashSet;

    pub use std::sync::Mutex;
    pub use std::sync::RwLock;

    pub use super::muvm::*;
    pub use super::muctx::*;
    pub use super::muirbuilder::*;

    pub use super::super::super::vm::VM;

    pub use super::super::api_c::*;
    pub use super::super::api_bridge::*;
    pub use super::super::irnodes::*;

    pub use ast::bundle::*;
    pub use ast::ir::*;
    pub use ast::ptr::*;
    pub use ast::types::*;
}

