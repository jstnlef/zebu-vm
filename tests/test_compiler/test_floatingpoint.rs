extern crate mu;
extern crate log;
extern crate libloading;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::testutil;
use mu::utils::LinkedHashMap;

use std::sync::RwLock;

#[test]
fn test_fp_add() {
    let lib = testutil::compile_fnc("fp_add", &fp_add);

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
        blocks: {
            let mut ret = LinkedHashMap::new();
            ret.insert(blk_entry.id(), blk_entry);
            ret
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

#[test]
fn test_uitofp() {
    let lib = testutil::compile_fnc("uitofp", &uitofp);

    unsafe {
        let uitofp : libloading::Symbol<unsafe extern fn(u64) -> f64> = lib.get(b"uitofp").unwrap();

        let res = uitofp(0u64);
        println!("uitofp(0) = {}", res);
        assert!(res == 0f64);

        let res = uitofp(1u64);
        println!("uitofp(1) = {}", res);
        assert!(res == 1f64);
    }
}

fn uitofp() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) double = mu_double);

    funcsig!    ((vm) sig = (int64) -> (double));
    funcdecl!   ((vm) <sig> uitofp);
    funcdef!    ((vm) <sig> uitofp VERSION uitofp_v1);

    // blk entry
    block!      ((vm, uitofp_v1) blk_entry);
    ssa!        ((vm, uitofp_v1) <int64> x);

    ssa!        ((vm, uitofp_v1) <double> res);
    inst!       ((vm, uitofp_v1) blk_entry_conv:
        res = CONVOP (ConvOp::UITOFP) <int64 double> x
    );

    inst!       ((vm, uitofp_v1) blk_entry_ret:
        RET (res)
    );

    define_block!((vm, uitofp_v1) blk_entry(x){
        blk_entry_conv, blk_entry_ret
    });

    define_func_ver!((vm) uitofp_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_fp_arraysum() {
    use std::os::raw::c_double;

    let lib = testutil::compile_fnc("fp_arraysum", &fp_arraysum);

    unsafe {
        let fp_arraysum : libloading::Symbol<unsafe extern fn(*const c_double, u64) -> f64> = lib.get(b"fp_arraysum").unwrap();

        let array : [f64; 10] = [0f64, 0.1f64, 0.2f64, 0.3f64, 0.4f64, 0.5f64, 0.6f64, 0.7f64, 0.8f64, 0.9f64];
        let c_array = array.as_ptr() as *const c_double;

        let res = fp_arraysum(c_array, 10);
        println!("fp_arraysum(array, 10) = {}", res);
        assert!(res == 4.5f64);
    }
}

fn fp_arraysum() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64  = mu_int(64));
    typedef!    ((vm) int1   = mu_int(1));
    typedef!    ((vm) double = mu_double);
    typedef!    ((vm) hybrid = mu_hybrid(none; double));
    typedef!    ((vm) uptr_hybrid = mu_uptr(hybrid));
    typedef!    ((vm) uptr_double = mu_uptr(double));

    constdef!   ((vm) <int64> int64_0   = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1   = Constant::Int(1));
    constdef!   ((vm) <double> double_0 = Constant::Double(0f64));

    funcsig!    ((vm) sig = (uptr_hybrid, int64) -> (double));
    funcdecl!   ((vm) <sig> fp_arraysum);
    funcdef!    ((vm) <sig> fp_arraysum VERSION fp_arraysum_v1);

    // blk entry
    block!      ((vm, fp_arraysum_v1) blk_entry);
    ssa!        ((vm, fp_arraysum_v1) <uptr_hybrid> blk_entry_arr);
    ssa!        ((vm, fp_arraysum_v1) <int64> blk_entry_sz);

    block!      ((vm, fp_arraysum_v1) blk1);
    consta!     ((vm, fp_arraysum_v1) int64_0_local  = int64_0);
    consta!     ((vm, fp_arraysum_v1) int64_1_local  = int64_1);
    consta!     ((vm, fp_arraysum_v1) double_0_local = double_0);
    inst!       ((vm, fp_arraysum_v1) blk_entry_branch:
        BRANCH blk1 (blk_entry_arr, double_0_local, int64_0_local, blk_entry_sz)
    );

    define_block!   ((vm, fp_arraysum_v1) blk_entry(blk_entry_arr, blk_entry_sz) {blk_entry_branch});

    // blk1
    ssa!        ((vm, fp_arraysum_v1) <uptr_hybrid> blk1_arr);
    ssa!        ((vm, fp_arraysum_v1) <double> blk1_sum);
    ssa!        ((vm, fp_arraysum_v1) <int64> blk1_v1);
    ssa!        ((vm, fp_arraysum_v1) <int64> blk1_v2);

    ssa!        ((vm, fp_arraysum_v1) <int1> blk1_rtn);
    inst!       ((vm, fp_arraysum_v1) blk1_sge:
        blk1_rtn = CMPOP (CmpOp::SGE) blk1_v1 blk1_v2
    );

    block!      ((vm, fp_arraysum_v1) blk2);
    block!      ((vm, fp_arraysum_v1) blk3);
    inst!       ((vm, fp_arraysum_v1) blk1_branch2:
        BRANCH2 (blk1_rtn, blk1_sum, blk1_v2, blk1_v1, blk1_arr)
        IF (OP 0)
        THEN blk3 (vec![1]) WITH 0.2f32,
        ELSE blk2 (vec![2, 3, 4, 1])
    );

    define_block!   ((vm, fp_arraysum_v1) blk1(blk1_arr, blk1_sum, blk1_v1, blk1_v2) {
        blk1_sge, blk1_branch2
    });

    // blk2
    ssa!        ((vm, fp_arraysum_v1) <int64> blk2_v4);
    ssa!        ((vm, fp_arraysum_v1) <int64> blk2_next);
    ssa!        ((vm, fp_arraysum_v1) <uptr_hybrid> blk2_arr);
    ssa!        ((vm, fp_arraysum_v1) <double> blk2_sum);

    ssa!        ((vm, fp_arraysum_v1) <int64> blk2_v5);
    inst!       ((vm, fp_arraysum_v1) blk2_add:
        blk2_v5 = BINOP (BinOp::Add) blk2_next int64_1_local
    );

    ssa!        ((vm, fp_arraysum_v1) <uptr_double> blk2_rtn2);
    inst!       ((vm, fp_arraysum_v1) blk2_getvarpart:
        blk2_rtn2 = GETVARPARTIREF blk2_arr (is_ptr: true)
    );

    ssa!        ((vm, fp_arraysum_v1) <uptr_double> blk2_rtn3);
    inst!       ((vm, fp_arraysum_v1) blk2_shiftiref:
        blk2_rtn3 = SHIFTIREF blk2_rtn2 blk2_next (is_ptr: true)
    );

    ssa!        ((vm, fp_arraysum_v1) <double> blk2_v7);
    inst!       ((vm, fp_arraysum_v1) blk2_load:
        blk2_v7 = LOAD blk2_rtn3 (is_ptr: true, order: MemoryOrder::NotAtomic)
    );

    ssa!        ((vm, fp_arraysum_v1) <double> blk2_sum2);
    inst!       ((vm, fp_arraysum_v1) blk2_fadd:
        blk2_sum2 = BINOP (BinOp::FAdd) blk2_sum blk2_v7
    );

    inst!       ((vm, fp_arraysum_v1) blk2_branch:
        BRANCH blk1 (blk2_arr, blk2_sum2, blk2_v5, blk2_v4)
    );

    define_block!   ((vm, fp_arraysum_v1) blk2(blk2_v4, blk2_next, blk2_arr, blk2_sum) {
        blk2_add, blk2_getvarpart, blk2_shiftiref, blk2_load, blk2_fadd, blk2_branch
    });

    // blk3
    ssa!        ((vm, fp_arraysum_v1) <double> blk3_v8);
    inst!       ((vm, fp_arraysum_v1) blk3_ret:
        RET (blk3_v8)
    );

    define_block!   ((vm, fp_arraysum_v1) blk3(blk3_v8) {blk3_ret});

    define_func_ver!    ((vm) fp_arraysum_v1 (entry: blk_entry) {blk_entry, blk1, blk2, blk3});

    vm
}
