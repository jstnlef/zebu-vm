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
fn test_udiv() {
    let lib = testutil::compile_fnc("udiv", &udiv);

    unsafe {
        let udiv : libloading::Symbol<unsafe extern fn(u64, u64) -> u64> = lib.get(b"udiv").unwrap();

        let udiv_8_2 = udiv(8, 2);
        println!("udiv(8, 2) = {}", udiv_8_2);
        assert!(udiv_8_2 == 4);
    }
}

fn udiv() -> VM {
    let vm = VM::new();

    // .typedef @int64 = int<64>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), Mu("int64"));

    // .funcsig @udiv_sig = (@int64 @int64) -> (@int64)
    let udiv_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_int64.clone()], vec![type_def_int64.clone(), type_def_int64.clone()]);
    vm.set_name(udiv_sig.as_entity(), Mu("udiv_sig"));

    // .funcdecl @udiv <@udiv_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, udiv_sig.clone());
    vm.set_name(func.as_entity(), Mu("udiv"));
    vm.declare_func(func);

    // .funcdef @udiv VERSION @udiv_v1 <@udiv_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, udiv_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("udiv_v1"));

    // %entry(<@int64> %a, <@int64> %b):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    let blk_entry_a = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));
    let blk_entry_b = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_b.as_entity(), Mu("blk_entry_b"));

    // %r = UDIV %a %b
    let blk_entry_r = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_r.as_entity(), Mu("blk_entry_r"));
    let blk_entry_add = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_r.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone(), blk_entry_b.clone()]),
        v: Instruction_::BinOp(BinOp::Udiv, 0, 1)
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
            let mut map = LinkedHashMap::new();
            map.insert(blk_entry.id(), blk_entry);
            map
        }
    });

    vm.define_func_version(func_ver);

    vm
}

#[test]
fn test_sdiv() {
    let lib = testutil::compile_fnc("sdiv", &sdiv);

    unsafe {
        let sdiv : libloading::Symbol<unsafe extern fn(i64, i64) -> i64> = lib.get(b"sdiv").unwrap();

        let sdiv_8_2 = sdiv(8, 2);
        println!("sdiv(8, 2) = {}", sdiv_8_2);
        assert!(sdiv_8_2 == 4);

        let sdiv_8_m2 = sdiv(8, -2i64);
        println!("sdiv(8, -2) = {}", sdiv_8_m2);
        assert!(sdiv_8_m2 == -4i64);
    }
}

fn sdiv() -> VM {
    let vm = VM::new();

    // .typedef @int64 = int<64>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), Mu("int64"));

    // .funcsig @sdiv_sig = (@int64 @int64) -> (@int64)
    let sdiv_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_int64.clone()], vec![type_def_int64.clone(), type_def_int64.clone()]);
    vm.set_name(sdiv_sig.as_entity(), Mu("sdiv_sig"));

    // .funcdecl @sdiv <@sdiv_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, sdiv_sig.clone());
    vm.set_name(func.as_entity(), Mu("sdiv"));
    vm.declare_func(func);

    // .funcdef @sdiv VERSION @sdiv_v1 <@sdiv_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, sdiv_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("sdiv_v1"));

    // %entry(<@int64> %a, <@int64> %b):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    let blk_entry_a = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));
    let blk_entry_b = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_b.as_entity(), Mu("blk_entry_b"));

    // %r = SDIV %a %b
    let blk_entry_r = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_r.as_entity(), Mu("blk_entry_r"));
    let blk_entry_add = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_r.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone(), blk_entry_b.clone()]),
        v: Instruction_::BinOp(BinOp::Sdiv, 0, 1)
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
            let mut map = LinkedHashMap::new();
            map.insert(blk_entry.id(), blk_entry);
            map
        }
    });

    vm.define_func_version(func_ver);

    vm
}

#[test]
fn test_shl() {
    let lib = testutil::compile_fnc("shl", &shl);

    unsafe {
        let shl : libloading::Symbol<unsafe extern fn(u64, u8) -> u64> = lib.get(b"shl").unwrap();

        let shl_1_2 = shl(1, 2);
        println!("shl(1, 2) = {}", shl_1_2);
        assert!(shl_1_2 == 4);

        let shl_2_2 = shl(2, 2);
        println!("shl(2, 2) = {}", shl_2_2);
        assert!(shl_2_2 == 8);
    }
}

fn shl() -> VM {
    let vm = VM::new();

    // .typedef @int64 = int<64>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), Mu("int64"));
    // .typedef @int8 = int<8>
    let type_def_int8 = vm.declare_type(vm.next_id(), MuType_::int(8));
    vm.set_name(type_def_int8.as_entity(), Mu("int8"));

    // .funcsig @shl_sig = (@int64 @int8) -> (@int64)
    let shl_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_int64.clone()], vec![type_def_int64.clone(), type_def_int8.clone()]);
    vm.set_name(shl_sig.as_entity(), Mu("shl_sig"));

    // .funcdecl @shl <@shl_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, shl_sig.clone());
    vm.set_name(func.as_entity(), Mu("shl"));
    vm.declare_func(func);

    // .funcdef @shl VERSION @shl_v1 <@shl_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, shl_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("shl_v1"));

    // %entry(<@int64> %a, <@int8> %b):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    let blk_entry_a = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));
    let blk_entry_b = func_ver.new_ssa(vm.next_id(), type_def_int8.clone());
    vm.set_name(blk_entry_b.as_entity(), Mu("blk_entry_b"));

    // %r = SHL %a %b
    let blk_entry_r = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_r.as_entity(), Mu("blk_entry_r"));
    let blk_entry_add = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_r.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone(), blk_entry_b.clone()]),
        v: Instruction_::BinOp(BinOp::Shl, 0, 1)
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
            let mut map = LinkedHashMap::new();
            map.insert(blk_entry.id(), blk_entry);
            map
        }
    });

    vm.define_func_version(func_ver);

    vm
}

#[test]
fn test_lshr() {
    let lib = testutil::compile_fnc("lshr", &lshr);

    unsafe {
        let lshr : libloading::Symbol<unsafe extern fn(u64, u8) -> u64> = lib.get(b"lshr").unwrap();

        let lshr_8_3 = lshr(8, 3);
        println!("lshr(8, 3) = {}", lshr_8_3);
        assert!(lshr_8_3 == 1);
    }
}

fn lshr() -> VM {
    let vm = VM::new();

    // .typedef @int64 = int<64>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), Mu("int64"));
    // .typedef @int8 = int<8>
    let type_def_int8 = vm.declare_type(vm.next_id(), MuType_::int(8));
    vm.set_name(type_def_int8.as_entity(), Mu("int8"));

    // .funcsig @lshr_sig = (@int64 @int8) -> (@int64)
    let lshr_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_int64.clone()], vec![type_def_int64.clone(), type_def_int8.clone()]);
    vm.set_name(lshr_sig.as_entity(), Mu("lshr_sig"));

    // .funcdecl @lshr <@lshr_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, lshr_sig.clone());
    vm.set_name(func.as_entity(), Mu("lshr"));
    vm.declare_func(func);

    // .funcdef @lshr VERSION @lshr_v1 <@lshr_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, lshr_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("lshr_v1"));

    // %entry(<@int64> %a, <@int8> %b):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    let blk_entry_a = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));
    let blk_entry_b = func_ver.new_ssa(vm.next_id(), type_def_int8.clone());
    vm.set_name(blk_entry_b.as_entity(), Mu("blk_entry_b"));

    // %r = LSHR %a %b
    let blk_entry_r = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_r.as_entity(), Mu("blk_entry_r"));
    let blk_entry_add = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_r.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone(), blk_entry_b.clone()]),
        v: Instruction_::BinOp(BinOp::Lshr, 0, 1)
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
            let mut map = LinkedHashMap::new();
            map.insert(blk_entry.id(), blk_entry);
            map
        }
    });

    vm.define_func_version(func_ver);

    vm
}

#[test]
fn test_add_simple() {
    let lib = testutil::compile_fnc("add", &add);

    unsafe {
        let add : libloading::Symbol<unsafe extern fn(u64, u64) -> u64> = lib.get(b"add").unwrap();

        let res = add(1, 1);
        println!("add(1, 1) = {}", res);
        assert!(res == 2);
    }
}

fn add() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));

    funcsig!    ((vm) sig = (int64, int64) -> (int64));
    funcdecl!   ((vm) <sig> add);
    funcdef!    ((vm) <sig> add VERSION add_v1);

    block!      ((vm, add_v1) blk_entry);
    ssa!        ((vm, add_v1) <int64> a);
    ssa!        ((vm, add_v1) <int64> b);

    // sum = Add %a %b
    ssa!        ((vm, add_v1) <int64> sum);
    inst!       ((vm, add_v1) blk_entry_add:
        sum = BINOP (BinOp::Add) a b
    );

    inst!       ((vm, add_v1) blk_entry_ret:
        RET (sum)
    );

    define_block!   ((vm, add_v1) blk_entry(a, b) {
        blk_entry_add, blk_entry_ret
    });

    define_func_ver!((vm) add_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_add_int64_n() {
    let lib = testutil::compile_fnc("add_int64_n", &add_int64_n);

    unsafe {
        let add_int64_n : libloading::Symbol<unsafe extern fn(i64, i64) -> u8> = lib.get(b"add_int64_n").unwrap();

        let flag = add_int64_n(1, 1);
        println!("add_int64_n(1, 1), #N = {}", flag);
        assert!(flag == 0);

        let flag = add_int64_n(1, -2);
        println!("add_int64_n(1, -2), #N = {}", flag);
        assert!(flag == 1);

        let flag = add_int64_n(1, -1);
        println!("add_int64_n(1, -1), #N = {}", flag);
        assert!(flag == 0);

        let flag = add_int64_n(-1, -1);
        println!("add_int64_n(-1, -1), #N = {}", flag);
        assert!(flag == 1);
    }
}

fn add_int64_n() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int1  = mu_int(1));

    funcsig!    ((vm) sig = (int64, int64) -> (int1));
    funcdecl!   ((vm) <sig> add_int64_n);
    funcdef!    ((vm) <sig> add_int64_n VERSION add_int64_n_v1);

    block!      ((vm, add_int64_n_v1) blk_entry);
    ssa!        ((vm, add_int64_n_v1) <int64> a);
    ssa!        ((vm, add_int64_n_v1) <int64> b);

    // (sum, flag_n) = Add #N %a %b
    ssa!        ((vm, add_int64_n_v1) <int64> sum);
    ssa!        ((vm, add_int64_n_v1) <int1> flag_n);
    inst!       ((vm, add_int64_n_v1) blk_entry_add:
        sum, flag_n = BINOP_STATUS (BinOp::Add) (BinOpStatus::n()) a b
    );

    inst!       ((vm, add_int64_n_v1) blk_entry_ret:
        RET (flag_n)
    );

    define_block!   ((vm, add_int64_n_v1) blk_entry(a, b) {
        blk_entry_add, blk_entry_ret
    });

    define_func_ver!((vm) add_int64_n_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_add_int64_z() {
    let lib = testutil::compile_fnc("add_int64_z", &add_int64_z);

    unsafe {
        let add_int64_z : libloading::Symbol<unsafe extern fn(i64, i64) -> u8> = lib.get(b"add_int64_z").unwrap();

        let flag = add_int64_z(1, 1);
        println!("add_int64_z(1, 1), #Z = {}", flag);
        assert!(flag == 0);

        let flag = add_int64_z(1, -2);
        println!("add_int64_z(1, -2), #Z = {}", flag);
        assert!(flag == 0);

        let flag = add_int64_z(1, -1);
        println!("add_int64_z(1, -1), #Z = {}", flag);
        assert!(flag == 1);
    }
}

fn add_int64_z() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int1  = mu_int(1));

    funcsig!    ((vm) sig = (int64, int64) -> (int1));
    funcdecl!   ((vm) <sig> add_int64_z);
    funcdef!    ((vm) <sig> add_int64_z VERSION add_int64_z_v1);

    block!      ((vm, add_int64_z_v1) blk_entry);
    ssa!        ((vm, add_int64_z_v1) <int64> a);
    ssa!        ((vm, add_int64_z_v1) <int64> b);

    // (sum, flag_n) = Add #N %a %b
    ssa!        ((vm, add_int64_z_v1) <int64> sum);
    ssa!        ((vm, add_int64_z_v1) <int1> flag_z);
    inst!       ((vm, add_int64_z_v1) blk_entry_add:
        sum, flag_z = BINOP_STATUS (BinOp::Add) (BinOpStatus::z()) a b
    );

    inst!       ((vm, add_int64_z_v1) blk_entry_ret:
        RET (flag_z)
    );

    define_block!   ((vm, add_int64_z_v1) blk_entry(a, b) {
        blk_entry_add, blk_entry_ret
    });

    define_func_ver!((vm) add_int64_z_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_add_int64_c() {
    use std::u64;

    let lib = testutil::compile_fnc("add_int64_c", &add_int64_c);

    unsafe {
        let add_int64_c : libloading::Symbol<unsafe extern fn(u64, u64) -> u8> = lib.get(b"add_int64_c").unwrap();

        let flag = add_int64_c(u64::MAX, 1);
        println!("add_int64_c(u64::MAX, 1), #C = {}", flag);
        assert!(flag == 1);

        let flag = add_int64_c(u64::MAX, 0);
        println!("add_int64_c(i64::MAX, 0), #C = {}", flag);
        assert!(flag == 0);
    }
}

fn add_int64_c() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int1  = mu_int(1));

    funcsig!    ((vm) sig = (int64, int64) -> (int1));
    funcdecl!   ((vm) <sig> add_int64_c);
    funcdef!    ((vm) <sig> add_int64_c VERSION add_int64_c_v1);

    block!      ((vm, add_int64_c_v1) blk_entry);
    ssa!        ((vm, add_int64_c_v1) <int64> a);
    ssa!        ((vm, add_int64_c_v1) <int64> b);

    // (sum, flag_n) = Add #N %a %b
    ssa!        ((vm, add_int64_c_v1) <int64> sum);
    ssa!        ((vm, add_int64_c_v1) <int1> flag_c);
    inst!       ((vm, add_int64_c_v1) blk_entry_add:
        sum, flag_c = BINOP_STATUS (BinOp::Add) (BinOpStatus::c()) a b
    );

    inst!       ((vm, add_int64_c_v1) blk_entry_ret:
        RET (flag_c)
    );

    define_block!   ((vm, add_int64_c_v1) blk_entry(a, b) {
        blk_entry_add, blk_entry_ret
    });

    define_func_ver!((vm) add_int64_c_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_add_int64_v() {
    use std::i64;

    let lib = testutil::compile_fnc("add_int64_v", &add_int64_v);

    unsafe {
        let add_int64_v : libloading::Symbol<unsafe extern fn(i64, i64) -> u8> = lib.get(b"add_int64_v").unwrap();

        let flag = add_int64_v(i64::MAX, 1);
        println!("add_int64_v(i64::MAX, 1), #V = {}", flag);
        assert!(flag == 1);

        let flag = add_int64_v(i64::MAX, 0);
        println!("add_int64_v(i64::MAX, 0), #V = {}", flag);
        assert!(flag == 0);

        let flag = add_int64_v(i64::MIN, 0);
        println!("add_int64_v(i64::MIN, 0), #V = {}", flag);
        assert!(flag == 0);

        let flag = add_int64_v(i64::MIN, -1);
        println!("add_int64_v(i64::MIN, -1), #V = {}", flag);
        assert!(flag == 1);
    }
}

fn add_int64_v() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int1  = mu_int(1));

    funcsig!    ((vm) sig = (int64, int64) -> (int1));
    funcdecl!   ((vm) <sig> add_int64_v);
    funcdef!    ((vm) <sig> add_int64_v VERSION add_int64_v_v1);

    block!      ((vm, add_int64_v_v1) blk_entry);
    ssa!        ((vm, add_int64_v_v1) <int64> a);
    ssa!        ((vm, add_int64_v_v1) <int64> b);

    // (sum, flag_n) = Add #N %a %b
    ssa!        ((vm, add_int64_v_v1) <int64> sum);
    ssa!        ((vm, add_int64_v_v1) <int1> flag_v);
    inst!       ((vm, add_int64_v_v1) blk_entry_add:
        sum, flag_v = BINOP_STATUS (BinOp::Add) (BinOpStatus::v()) a b
    );

    inst!       ((vm, add_int64_v_v1) blk_entry_ret:
        RET (flag_v)
    );

    define_block!   ((vm, add_int64_v_v1) blk_entry(a, b) {
        blk_entry_add, blk_entry_ret
    });

    define_func_ver!((vm) add_int64_v_v1 (entry: blk_entry) {blk_entry});

    vm
}

#[test]
fn test_add_int64_nzc() {
    use std::u64;

    let lib = testutil::compile_fnc("add_int64_nzc", &add_int64_nzc);

    unsafe {
        let add_int64_nzc : libloading::Symbol<unsafe extern fn(u64, u64) -> u8> = lib.get(b"add_int64_nzc").unwrap();

        let flag = add_int64_nzc(u64::MAX, 1);
        println!("add_int64_nzc(u64::MAX, 1), #C = {:b}", flag);
        assert!(flag == 0b110);

        let flag = add_int64_nzc(u64::MAX, 0);
        println!("add_int64_nzc(u64::MAX, 0), #C = {:b}", flag);
        assert!(flag == 0b001);
    }
}

fn add_int64_nzc() -> VM {
    let vm = VM::new();

    typedef!    ((vm) int64 = mu_int(64));
    typedef!    ((vm) int8  = mu_int(8));
    typedef!    ((vm) int1  = mu_int(1));

    constdef!   ((vm) <int8> int8_1 = Constant::Int(1));
    constdef!   ((vm) <int8> int8_2 = Constant::Int(2));
    constdef!   ((vm) <int8> int8_3 = Constant::Int(3));

    funcsig!    ((vm) sig = (int64, int64) -> (int1));
    funcdecl!   ((vm) <sig> add_int64_nzc);
    funcdef!    ((vm) <sig> add_int64_nzc VERSION add_int64_nzc_v1);

    block!      ((vm, add_int64_nzc_v1) blk_entry);
    ssa!        ((vm, add_int64_nzc_v1) <int64> a);
    ssa!        ((vm, add_int64_nzc_v1) <int64> b);

    // (sum, flag_n) = Add #N %a %b
    ssa!        ((vm, add_int64_nzc_v1) <int64> sum);
    ssa!        ((vm, add_int64_nzc_v1) <int1> flag_n);
    ssa!        ((vm, add_int64_nzc_v1) <int1> flag_z);
    ssa!        ((vm, add_int64_nzc_v1) <int1> flag_c);

    inst!       ((vm, add_int64_nzc_v1) blk_entry_add:
        sum, flag_n, flag_z, flag_c = BINOP_STATUS (BinOp::Add) (BinOpStatus{flag_n: true, flag_z: true, flag_c: true, flag_v: false}) a b
    );

    ssa!        ((vm, add_int64_nzc_v1) <int8> shift_z);
    consta!     ((vm, add_int64_nzc_v1) int8_1_local = int8_1);
    inst!       ((vm, add_int64_nzc_v1) blk_entry_shift_z:
        shift_z = BINOP (BinOp::Shl) flag_z int8_1_local
    );

    ssa!        ((vm, add_int64_nzc_v1) <int8> ret);
    inst!       ((vm, add_int64_nzc_v1) blk_entry_add_ret1:
        ret = BINOP (BinOp::Add) flag_n shift_z
    );

    ssa!        ((vm, add_int64_nzc_v1) <int8> shift_c);
    consta!     ((vm, add_int64_nzc_v1) int8_2_local = int8_2);
    inst!       ((vm, add_int64_nzc_v1) blk_entry_shift_c:
        shift_c = BINOP (BinOp::Shl) flag_c int8_2_local
    );

    ssa!        ((vm, add_int64_nzc_v1) <int8> ret2);
    inst!       ((vm, add_int64_nzc_v1) blk_entry_add_ret2:
        ret2 = BINOP (BinOp::Add) ret shift_c
    );

    inst!       ((vm, add_int64_nzc_v1) blk_entry_ret:
        RET (ret2)
    );

    define_block!   ((vm, add_int64_nzc_v1) blk_entry(a, b) {
        blk_entry_add, blk_entry_shift_z, blk_entry_add_ret1, blk_entry_shift_c, blk_entry_add_ret2, blk_entry_ret
    });

    define_func_ver!((vm) add_int64_nzc_v1 (entry: blk_entry) {blk_entry});

    vm
}