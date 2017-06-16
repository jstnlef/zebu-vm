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

use test_ir::test_ir::factorial;
use mu::vm::*;

use std::sync::Arc;

use self::rustc_serialize::json;
use self::rustc_serialize::base64::ToBase64;
use self::rustc_serialize::hex::ToHex;
use self::rustc_serialize::base64::STANDARD;

#[test]
fn test_vm_serialize_factorial() {
    VM::start_logging_trace();
    
    let vm = Arc::new(factorial());
    
    let serialized_json = json::encode(&vm).unwrap();
    println!("JSON(len={}):", serialized_json.len());
    println!("{}", serialized_json);
    
    let base64 = serialized_json.as_bytes().to_base64(STANDARD);
    println!("base64(len={}):", base64.len());
    println!("{}", base64);

    let hex = serialized_json.as_bytes().to_hex();
    println!("hex(len={}):", hex.len());
    println!("{}", hex);
    
    let reconstruct_vm : VM = json::decode(&serialized_json).unwrap();
    let serialized_again = json::encode(&reconstruct_vm).unwrap();
    println!("JSON for reconstructed VM(len={}):", serialized_again.len());
    println!("{}", serialized_again);
    
//    check_string_eq_char_by_char(serialized, serialized_again);
}

#[allow(dead_code)]
fn check_string_eq_char_by_char(str1: String, str2: String) {
    use std::cmp;
    
    let min_len = cmp::min(str1.len(), str2.len());

    println!("str1_len = {}, str2_len = {}", str1.len(), str2.len());
    
    let b1 = str1.into_bytes();
    let b2 = str2.into_bytes();
    
    for i in 0..min_len {
        if b1[i] != b2[i] {
            println!("different here ({}):", i);
            
            print!("str1: ..");
            for j in 0..20 {
                print!("{}", b1[i + j] as char);
            }
            println!("..");
            print!("str2: ..");
            for j in 0..20 {
                print!("{}", b2[i + j] as char);
            }
            println!("..");
            
            panic!("found difference in two strings");
        }
    }
}
