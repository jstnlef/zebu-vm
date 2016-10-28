extern crate libloading;

use testutil::get_test_clang_path;
use testutil::exec;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub fn compile_run_c_test(test_file_path: &'static str) -> PathBuf {
    let mut src = Path::new("tests/test_jit").to_path_buf();
    src.push(test_file_path);

    let output = {
        use std::fs;

        let temp = Path::new("tests/test_jit/temp");
        fs::create_dir_all(temp);

        let mut ret = temp.to_path_buf();
        ret.push(src.file_stem().unwrap());
        ret
    };

    // compile the C test
    let mut cc = Command::new(get_test_clang_path());
    cc.arg("-std=c99");
    cc.arg("-Isrc/vm/api");
    cc.arg("-Ltarget/debug");
    cc.arg("-lmu");
    // src
    cc.arg(src.as_os_str());
    // output
    cc.arg("-o");
    cc.arg(output.as_os_str());

    exec(cc);

    // run the executable
    let test = Command::new(output.as_os_str());
    let test_out = exec(test);

    Path::new(&String::from_utf8(test_out.stdout).unwrap()).to_path_buf()
}