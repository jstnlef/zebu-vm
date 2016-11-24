extern crate mu;
extern crate log;
extern crate libloading;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::compiler::*;
use self::mu::testutil;

use std::sync::RwLock;
use std::sync::Arc;
use mu::testutil::aot;

#[test]
fn test_fp_add() {
    VM::start_logging_trace();

    let vm = Arc::new(fp_add());

    let compiler = Compiler::new(CompilerPolicy::default(), vm.clone());

    let func_id = vm.id_of("fp_add");

    {
        let funcs = vm.funcs().read().unwrap();
        let func = funcs.get(&func_id).unwrap().read().unwrap();
        let func_vers = vm.func_vers().read().unwrap();
        let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();

        compiler.compile(&mut func_ver);
    }

    backend::emit_context(&vm);

    let dylib = aot::link_dylib(vec![Mu("fp_add")], "libfp_add.dylib", &vm);

    let lib = libloading::Library::new(dylib.as_os_str()).unwrap();
    unsafe {
        let fp_add : libloading::Symbol<unsafe extern fn(f64, f64) -> f64> = lib.get(b"fp_add").unwrap();

        let fp_add_1_1 = fp_add(1f64, 1f64);
        println!("fp_add(1, 1) = {}", fp_add_1_1);
        assert!(fp_add_1_1 == 2f64);
    }
}

fn fp_add() -> VM {
    let vm = VM::new();

    // .typedef @double = double
    let type_def_double = vm.declare_type(vm.next_id(), MuType_::double());
    vm.set_name(type_def_double.as_entity(), Mu("double"));

    // .funcsig @fp_add_sig = (@double @double) -> (@double)
    let fp_add_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_double.clone()], vec![type_def_double.clone(), type_def_double.clone()]);
    vm.set_name(fp_add_sig.as_entity(), Mu("fp_add_sig"));

    // .funcdecl @fp_add <@fp_add_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, fp_add_sig.clone());
    vm.set_name(func.as_entity(), Mu("fp_add"));
    vm.declare_func(func);

    // .funcdef @fp_add VERSION @fp_add_v1 <@fp_add_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, fp_add_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("fp_add_v1"));

    // %entry(<@double> %a, <@double> %b):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    let blk_entry_a = func_ver.new_ssa(vm.next_id(), type_def_double.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));
    let blk_entry_b = func_ver.new_ssa(vm.next_id(), type_def_double.clone());
    vm.set_name(blk_entry_b.as_entity(), Mu("blk_entry_b"));

    // %r = FADD %a %b
    let blk_entry_r = func_ver.new_ssa(vm.next_id(), type_def_double.clone());
    vm.set_name(blk_entry_r.as_entity(), Mu("blk_entry_r"));
    let blk_entry_add = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_r.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone(), blk_entry_b.clone()]),
        v: Instruction_::BinOp(BinOp::FAdd, 0, 1)
    });

    // RET %r
    let blk_entry_term = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_entry_r.clone()]),
        v: Instruction_::Return(vec![0])
    });

    blk_entry.content = Some(BlockContent{
        args: vec![blk_entry_a.clone_value(), blk_entry_b.clone_value()],
        exn_arg: None,
        body: vec![blk_entry_add, blk_entry_term],
        keepalives: None
    });

    func_ver.define(FunctionContent{
        entry: blk_entry.id(),
        blocks: hashmap!{
            blk_entry.id() => blk_entry
        }
    });

    vm.define_func_version(func_ver);

    vm
}

#[test]
fn test_fp_ogt_branch() {
    let lib = testutil::compile_fnc("fp_ogt_branch", &fp_ogt_branch);

    unsafe {
        let fp_ogt : libloading::Symbol<unsafe extern fn(f64, f64) -> u32> = lib.get(b"fp_ogt_branch").unwrap();

        let res = fp_ogt(-1f64, 0f64);
        println!("fp_ogt(-1, 0) = {}", res);
        assert!(res == 0);

        let res = fp_ogt(0f64, -1f64);
        println!("fp_ogt(0, -1) = {}", res);
        assert!(res == 1);

        let res = fp_ogt(-1f64, -1f64);
        println!("fp_ogt(-1, -1) = {}", res);
        assert!(res == 0);

        let res = fp_ogt(-1f64, -2f64);
        println!("fp_ogt(-1, -2) = {}", res);
        assert!(res == 1);

        let res = fp_ogt(-2f64, -1f64);
        println!("fp_ogt(-2, -1) = {}", res);
        assert!(res == 0);

        let res = fp_ogt(1f64, 2f64);
        println!("fp_ogt(1, 2) = {}", res);
        assert!(res == 0);

        let res = fp_ogt(2f64, 1f64);
        println!("fp_ogt(2, 1) = {}", res);
        assert!(res == 1);
    }
}

fn fp_ogt_branch() -> VM {
    let vm = VM::new();

    typedef!    ((vm) double = mu_double);
    typedef!    ((vm) int32  = mu_int(32));
    typedef!    ((vm) int1   = mu_int(1));

    constdef!   ((vm) <int32> int32_0 = Constant::Int(0));
    constdef!   ((vm) <int32> int32_1 = Constant::Int(1));

    funcsig!    ((vm) sig = (double, double) -> (int32));
    funcdecl!   ((vm) <sig> fp_ogt_branch);
    funcdef!    ((vm) <sig> fp_ogt_branch VERSION fp_ogt_branch_v1);

    // blk entry
    block!      ((vm, fp_ogt_branch_v1) blk_entry);
    ssa!        ((vm, fp_ogt_branch_v1) <double> a);
    ssa!        ((vm, fp_ogt_branch_v1) <double> b);

    ssa!        ((vm, fp_ogt_branch_v1) <int1> cond);
    inst!       ((vm, fp_ogt_branch_v1) blk_entry_cmp:
        cond = CMPOP (CmpOp::FOGT) a b
    );

    block!      ((vm, fp_ogt_branch_v1) blk_ret1);
    consta!     ((vm, fp_ogt_branch_v1) int32_1_local = int32_1);
    block!      ((vm, fp_ogt_branch_v1) blk_ret0);
    consta!     ((vm, fp_ogt_branch_v1) int32_0_local = int32_0);

    inst!       ((vm, fp_ogt_branch_v1) blk_entry_branch:
        BRANCH2 (cond, int32_1_local, int32_0_local)
            IF (OP 0)
            THEN blk_ret1 (vec![1]) WITH 0.6f32,
            ELSE blk_ret0 (vec![2])
    );

    define_block! ((vm, fp_ogt_branch_v1) blk_entry(a, b){
        blk_entry_cmp, blk_entry_branch
    });

    // blk_ret1
    ssa!        ((vm, fp_ogt_branch_v1) <int32> blk_ret1_res);
    inst!       ((vm, fp_ogt_branch_v1) blk_ret1_inst:
        RET (blk_ret1_res)
    );

    define_block! ((vm, fp_ogt_branch_v1) blk_ret1(blk_ret1_res){
        blk_ret1_inst
    });

    // blk_ret0
    ssa!        ((vm, fp_ogt_branch_v1) <int32> blk_ret0_res);
    inst!       ((vm, fp_ogt_branch_v1) blk_ret0_inst:
        RET (blk_ret0_res)
    );

    define_block! ((vm, fp_ogt_branch_v1) blk_ret0(blk_ret0_res){
        blk_ret0_inst
    });

    define_func_ver!((vm) fp_ogt_branch_v1 (entry: blk_entry) {
        blk_entry, blk_ret1, blk_ret0
    });

    vm
}

#[test]
fn test_sitofp() {
    let lib = testutil::compile_fnc("sitofp", &sitofp);

    unsafe {
        let sitofp : libloading::Symbol<unsafe extern fn(i64) -> f64> = lib.get(b"sitofp").unwrap();

        let res = sitofp(-1i64);
        println!("sitofp(-1) = {}", res);
        assert!(res == -1f64);

        let res = sitofp(0i64);
        println!("sitofp(0) = {}", res);
        assert!(res == 0f64);

        let res = sitofp(1i64);
        println!("sitofp(1) = {}", res);
        assert!(res == 1f64);
    }
}

fn sitofp() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) double = mu_double);

    funcsig!    ((vm) sig = (int64) -> (double));
    funcdecl!   ((vm) <sig> sitofp);
    funcdef!    ((vm) <sig> sitofp VERSION sitofp_v1);

    // blk entry
    block!      ((vm, sitofp_v1) blk_entry);
    ssa!        ((vm, sitofp_v1) <int64> x);

    ssa!        ((vm, sitofp_v1) <double> res);
    inst!       ((vm, sitofp_v1) blk_entry_conv:
        res = CONVOP (ConvOp::SITOFP) <int64 double> x
    );

    inst!       ((vm, sitofp_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, sitofp_v1) blk_entry(x){
        blk_entry_conv, blk_entry_ret
    });

    define_func_ver!((vm) sitofp_v1 (entry: blk_entry) {blk_entry});

    vm
}