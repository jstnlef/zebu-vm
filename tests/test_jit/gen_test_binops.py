"""
Generate C source file that builds a bundle to
test the binary operations

Run on reference implementation with:
    $ PYTHONPATH=$PYPY_MU:$MU/tools python gen_test_binops.py
Compile to C, then compile with clang, then run:
    $ PYTHONPATH=$PYPY_MU:$MU/tools python gen_test_binops.py -c gen_test_binops.c
    $ clang -std=c99 -I$MU/cbinding -L$MU/cbinding -lmurefimpl2start -o test_binops gen_test_binops.c
    $ ./test_binops
"""
def main(opts):
    """
    Builds the following bundle:
    .typedef @i8 = int<8>
    .typedef @i32 = int<32>
    .typedef @i64 = int<64>
    .typedef @pi8 = uptr<@i8>
    .typdef @ppi8= uptr<@pi8>
    .const @42_i64 <@i64> = 42
    .const @10_i64 <@i64> = 10
    .const @687_i64 <@i64> = 687
    .const @31_i64 <@i64> = 31
    .const @5_i64 <@i64> = 5
    .const @8_i64 <@i64> = 8
    .global @result <@i64>
    .funcsig @sig_i32ppi8_= () -> ()
    .funcdef @test_binops VERSION @v1 <@sig_i32ppi8_> {
        %blk0 (<@i32> %argc <@ppi8> %argv):
            %res0 = ADD <@i64> @42_i64 @10_i64
            %res1 = SUB <@i64> %res0 @687_i64
            %res2 = MUL <@i64> %res1 @31_i64
            %res3 = SDIV <@i64> %res2 @42_i64
            %res4 = UREM <@i64> %res3 @10_i64
            %res5 = SHL <@i64> %res4 @5_i64
            %res6 = LSHR <@i64> %res5 @8_i64
            %res7 = AND <@i64> %res6 @687_i64
            %res8 = XOR <@i64> %res7 @31_i64
            STORE <@i64> @result %res8
            COMMINST @uvm.thread_exit
    }
    """
    if opts.compile:
        from rpython.rlib import rmu_genc as rmu
    else:
        from rpython.rlib import rmu

    mu = rmu.MuVM()
    ctx = mu.new_context()
    bldr = ctx.new_ir_builder()

    i8 = bldr.gen_sym("@i8")
    bldr.new_type_int(i8, 8)
    i32 = bldr.gen_sym("@i32")
    bldr.new_type_int(i32, 32)
    i64 = bldr.gen_sym("@i64")
    bldr.new_type_int(i64, 64)
    pi8 = bldr.gen_sym("@pi8")
    bldr.new_type_uptr(pi8, i8)
    ppi8 = bldr.gen_sym("@ppi8")
    bldr.new_type_uptr(ppi8, pi8)

    c_42_i64 = bldr.gen_sym("@42_i64")
    bldr.new_const_int(c_42_i64, i64, 42)
    c_10_i64 = bldr.gen_sym("@10_i64")
    bldr.new_const_int(c_10_i64, i64, 10)
    c_687_i64 = bldr.gen_sym("@687_i64")
    bldr.new_const_int(c_687_i64, i64, 687)
    c_31_i64 = bldr.gen_sym("@31_i64")
    bldr.new_const_int(c_31_i64, i64, 31)
    c_5_i64 = bldr.gen_sym("@5_i64")
    bldr.new_const_int(c_5_i64, i64, 5)
    c_8_i64 = bldr.gen_sym("@8_i64")
    bldr.new_const_int(c_8_i64, i64, 8)

    result = bldr.gen_sym("@result")
    bldr.new_global_cell(result, i64)

    sig_i32ppi8_ = bldr.gen_sym("@sig_i32ppi8_")
    bldr.new_funcsig(sig_i32ppi8_, [i32, ppi8], [])

    test_binops = bldr.gen_sym("@test_binops")
    bldr.new_func(test_binops, sig_i32ppi8_)

    # function body
    v1 = bldr.gen_sym("@test_binops_v1")

    blk0 = bldr.gen_sym()
    argc = bldr.gen_sym()
    argv = bldr.gen_sym()
    res0 = bldr.gen_sym()
    op_add = bldr.gen_sym()
    bldr.new_binop(op_add, res0, rmu.MuBinOptr.ADD, i64, c_42_i64, c_10_i64)
    res1 = bldr.gen_sym()
    op_sub = bldr.gen_sym()
    bldr.new_binop(op_sub, res1, rmu.MuBinOptr.SUB, i64, res0, c_687_i64)
    res2 = bldr.gen_sym()
    op_mul = bldr.gen_sym()
    bldr.new_binop(op_mul, res2, rmu.MuBinOptr.MUL, i64, res1, c_31_i64)
    res3 = bldr.gen_sym()
    op_sdiv = bldr.gen_sym()
    bldr.new_binop(op_sdiv, res3, rmu.MuBinOptr.SDIV, i64, res2, c_42_i64)
    res4 = bldr.gen_sym()
    op_urem = bldr.gen_sym()
    bldr.new_binop(op_urem, res4, rmu.MuBinOptr.UREM, i64, res3, c_10_i64)
    res5 = bldr.gen_sym()
    op_shl = bldr.gen_sym()
    bldr.new_binop(op_shl, res5, rmu.MuBinOptr.SHL, i64, res4, c_5_i64)
    res6 = bldr.gen_sym()
    op_lshr = bldr.gen_sym()
    bldr.new_binop(op_lshr, res6, rmu.MuBinOptr.LSHR, i64, res5, c_8_i64)
    res7 = bldr.gen_sym()
    op_and = bldr.gen_sym()
    bldr.new_binop(op_and, res7, rmu.MuBinOptr.AND, i64, res6, c_687_i64)
    res8 = bldr.gen_sym()
    op_xor = bldr.gen_sym()
    bldr.new_binop(op_xor, res8, rmu.MuBinOptr.XOR, i64, res7, c_31_i64)
    op_store = bldr.gen_sym()
    bldr.new_store(op_store, False, rmu.MuMemOrd.NOT_ATOMIC, i64, result, res8)
    op_exit = bldr.gen_sym()
    bldr.new_comminst(op_exit, [], rmu.MuCommInst.THREAD_EXIT, [], [], [], [])
    bldr.new_bb(blk0, [argc, argv], [i32, ppi8], rmu.MU_NO_ID,
                [op_add, op_sub, op_mul, op_sdiv, op_urem, op_shl, op_lshr, op_and, op_xor, op_store, op_exit])
    bldr.new_func_ver(v1, test_binops, [blk0])

    bldr.load()

    # execute and get result
    hdl = ctx.handle_from_func(test_binops)
    stk = ctx.new_stack(hdl)
    hargc = ctx.handle_from_sint32(1, i32)
    if opts.compile:
        hargv = ctx.handle_from_ptr(ppi8, '(char **){&"test_binops"}')
    else:
        from rpython.rtyper.lltypesystem import rffi
        hargv = ctx.handle_from_ptr(ppi8, rffi.cast(rffi.VOIDP, rffi.liststr2charpp(["test_binops"])))
    thd = ctx.new_thread_nor(stk, rmu.null(rmu.MuThreadRefValue), [hargc, hargv])
    mu.execute()

    hres = ctx.load(rmu.MuMemOrd.NOT_ATOMIC, ctx.handle_from_global(result))

    if opts.compile:
        log = rmu.get_global_apilogger()
        res_val = rmu.CVar('int', 'res_val')
        log.logcall("handle_to_sint32", [ctx._ctx, hres], res_val, ctx._ctx)
        log.logcall("printf", [rmu.CStr("result = %d\\n"), res_val], None, context=None, check_err=False)
        with open(opts.compile, 'w') as fp:
            log.genc(fp)
    else:
        res_val = ctx.handle_to_sint32(hres)
        print "result =", res_val

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('-c', '--compile', type=str,
                        help='The script is compiled as API call trace to a C file. The parameter specifies the file name.'
                             'The absence of the flag defaults to run the script under RPython FFI on Mu reference implementation.')
    opts = parser.parse_args()
    main(opts)