from util import fncptr_from_py_script, may_spawn_proc
from rpython.rlib.rmu import zebu as rmu

@may_spawn_proc
def test_load_int_from_gcell():
    def build_test_bundle(bldr, rmu):
        """
        Builds the following test bundle.
            .typedef @i64 = int<64>
            .global @gcl <@i64>
            .funcsig @sig__i64 = () -> (@i64)
            .funcdef @test_fnc VERSION @test_fnc.v1 <@sig__i64> {
                %blk0():
                    %res = LOAD <@i64> @gcl
                    RET %res
            }
        :type bldr: rpython.rlib.rmu.MuIRBuilder
        :type rmu: rpython.rlib.rmu
        :return: (rmu.MuVM(), rmu.MuCtx, rmu.MuIRBuilder, MuID, MuID)
        """
        i64 = bldr.gen_sym("@i64"); bldr.new_type_int(i64, 64)

        gcl = bldr.gen_sym("@gcl"); bldr.new_global_cell(gcl, i64)

        sig__i64 = bldr.gen_sym("@sig__i64"); bldr.new_funcsig(sig__i64, [], [i64])

        test_fnc = bldr.gen_sym("@test_fnc"); bldr.new_func(test_fnc, sig__i64)

        test_fnc_v1 = bldr.gen_sym("@test_fnc.v1")
        blk0 = bldr.gen_sym("@test_fnc.v1.blk0")
        res = bldr.gen_sym("@test_fnc.v1.blk0.res")
        op_load = bldr.gen_sym(); bldr.new_load(op_load, res, False, rmu.MuMemOrd.NOT_ATOMIC, i64, gcl)
        op_ret = bldr.gen_sym(); bldr.new_ret(op_ret, [res])
        bldr.new_bb(blk0, [], [], rmu.MU_NO_ID, [op_load, op_ret])

        bldr.new_func_ver(test_fnc_v1, test_fnc, [blk0])

        return {
            "test_fnc": test_fnc,
            "test_fnc_sig": sig__i64,
            "result_type": i64,
            "@i64": i64,
            "@gcl": gcl,
        }

    def init_heap(ctx, id_dic, rmu):
        """
        :type ctx: rpython.rlib.rmu.MuCtx
        :type id_dic: dict
        :type rmu: rpython.rlib.rmu
        """
        gcl_hdl = ctx.handle_from_global(id_dic['@gcl'])
        hdl_num = ctx.handle_from_sint64(42, 64)
        ctx.store(rmu.MuMemOrd.NOT_ATOMIC, gcl_hdl, hdl_num)

    (fnp, _), (mu, ctx, bldr) = fncptr_from_py_script(build_test_bundle, init_heap, 'test_fnc')

    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    assert fnp() == 42


@may_spawn_proc
def test_load_ref_from_global():
    def build_test_bundle(bldr, rmu):
        """
        Builds the following test bundle.
            .typedef @i64 = int<64>
            .typedef @refi64 = ref<@i64>
            .global @gcl <@refi64>
            .funcsig @sig__i64 = () -> (@i64)
            .funcdef @test_fnc VERSION @test_fnc.v1 <@sig__i64> {
                %blk0():
                    %r = LOAD <@refi64> @gcl
                    %res = LOAD <@i64> %r
                    RET %res
            }
        :type bldr: rpython.rlib.rmu.MuIRBuilder
        :type rmu: rpython.rlib.rmu
        :return: (rmu.MuVM(), rmu.MuCtx, rmu.MuIRBuilder, MuID, MuID)
        """
        i64 = bldr.gen_sym("@i64"); bldr.new_type_int(i64, 64)
        refi64 = bldr.gen_sym("@refi64"); bldr.new_type_ref(refi64, i64)

        gcl = bldr.gen_sym("@gcl"); bldr.new_global_cell(gcl, refi64)

        sig__i64 = bldr.gen_sym("@sig__i64"); bldr.new_funcsig(sig__i64, [], [i64])

        test_fnc = bldr.gen_sym("@test_fnc"); bldr.new_func(test_fnc, sig__i64)

        test_fnc_v1 = bldr.gen_sym("@test_fnc.v1")
        blk0 = bldr.gen_sym("@test_fnc.v1.blk0")
        r = bldr.gen_sym("@test_fnc.v1.blk0.r")
        res = bldr.gen_sym("@test_fnc.v1.blk0.res")
        op_load1 = bldr.gen_sym(); bldr.new_load(op_load1, r, False, rmu.MuMemOrd.NOT_ATOMIC, refi64, gcl)
        op_load2 = bldr.gen_sym(); bldr.new_load(op_load2, res, False, rmu.MuMemOrd.NOT_ATOMIC, i64, r)
        op_ret = bldr.gen_sym(); bldr.new_ret(op_ret, [res])
        bldr.new_bb(blk0, [], [], rmu.MU_NO_ID, [op_load1, op_load2, op_ret])

        bldr.new_func_ver(test_fnc_v1, test_fnc, [blk0])

        return {
            "test_fnc": test_fnc,
            "test_fnc_sig": sig__i64,
            "result_type": i64,
            "@i64": i64,
            "@refi64": refi64,
            "@gcl": gcl,
        }

    def init_heap(ctx, id_dic, rmu):
        """
        :type ctx: rpython.rlib.rmu.MuCtx
        :type id_dic: dict
        :type rmu: rpython.rlib.rmu
        """
        ref_hdl = ctx.new_fixed(id_dic['@i64'])
        iref_hdl = ctx.get_iref(ref_hdl)
        hdl_num = ctx.handle_from_sint64(42, 64)
        ctx.store(rmu.MuMemOrd.NOT_ATOMIC, iref_hdl, hdl_num)

        gcl_hdl = ctx.handle_from_global(id_dic['@gcl'])
        ctx.store(rmu.MuMemOrd.NOT_ATOMIC, gcl_hdl, ref_hdl)

    (fnp, _), (mu, ctx, bldr) = fncptr_from_py_script(build_test_bundle, init_heap, 'test_fnc')

    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    assert fnp() == 42


@may_spawn_proc
def test_preserve_ref_field():
    def build_test_bundle(bldr, rmu):
        """
        Builds the following test bundle.
            .typedef @i64 = int<64>
            .typedef @refi64 = ref<@i64>
            .typedef @node = struct <@i64 @refnode>
            .typedef @refnode = ref<@node>
            .const @NULL <@refnode> = NULL
            .global @gcl <@refnode>
            .funcsig @sig__i64 = () -> (@i64)
            .funcdef @test_fnc VERSION @test_fnc.v1 <@sig__i64> {
                %blk0():
                    %rhd = LOAD <@refnode> @gcl
                    %irhd = GETIREF <@node> %rhd
                    %irhd_fld = GETFIELDIREF <@node 0> %irhd
                    %num = LOAD <@i64> %irhd_fld

                    %irhd_fld2 = GETFIELDIREF <@node 1> %irhd
                    %rnxt = LOAD <@refnode> %irhd_fld2
                    %irnxt = GETIREF <@node> %rnxt
                    %irnxt_fld = GETFIELDIREF <@node 0> %irnxt
                    %n_nxt = LOAD <@i64> %irnxt_fld

                    %res = ADD <@i64> %num %n_nxt
                    RET %res
            }
        :type bldr: rpython.rlib.rmu.MuIRBuilder
        :type rmu: rpython.rlib.rmu
        :return: (rmu.MuVM(), rmu.MuCtx, rmu.MuIRBuilder, MuID, MuID)
        """
        i64 = bldr.gen_sym("@i64"); bldr.new_type_int(i64, 64)
        refi64 = bldr.gen_sym("@refi64"); bldr.new_type_ref(refi64, i64)
        node = bldr.gen_sym("@node")
        refnode = bldr.gen_sym("@refnode")
        bldr.new_type_struct(node, [i64, refnode])
        bldr.new_type_ref(refnode, node)

        NULL = bldr.gen_sym("@NULL"); bldr.new_const_null(NULL, refnode)
        gcl = bldr.gen_sym("@gcl"); bldr.new_global_cell(gcl, refnode)

        sig__i64 = bldr.gen_sym("@sig__i64"); bldr.new_funcsig(sig__i64, [], [i64])

        test_fnc = bldr.gen_sym("@test_fnc"); bldr.new_func(test_fnc, sig__i64)

        test_fnc_v1 = bldr.gen_sym("@test_fnc.v1")
        blk0 = bldr.gen_sym("@test_fnc.v1.blk0")
        rhd = bldr.gen_sym("@test_fnc.v1.blk0.rhd")
        irhd = bldr.gen_sym("@test_fnc.v1.blk0.irhd")
        irhd_fld = bldr.gen_sym("@test_fnc.v1.blk0.irhd_fld")
        num = bldr.gen_sym("@test_fnc.v1.blk0.num")
        irhd_fld2 = bldr.gen_sym("@test_fnc.v1.blk0.irhd_fld2")
        rnxt = bldr.gen_sym("@test_fnc.v1.blk0.rnxt")
        irnxt = bldr.gen_sym("@test_fnc.v1.blk0.irnxt")
        irnxt_fld = bldr.gen_sym("@test_fnc.v1.blk0.irnxt_fld")
        n_nxt = bldr.gen_sym("@test_fnc.v1.blk0.n_nxt")
        res = bldr.gen_sym("@test_fnc.v1.blk0.res")
        op_load1 = bldr.gen_sym(); bldr.new_load(op_load1, rhd, False, rmu.MuMemOrd.NOT_ATOMIC, refnode, gcl)
        op_getiref1 = bldr.gen_sym(); bldr.new_getiref(op_getiref1, irhd, node, rhd)
        op_getfieldiref1 = bldr.gen_sym(); bldr.new_getfieldiref(op_getfieldiref1, irhd_fld, False, node, 0, irhd)
        op_load2 = bldr.gen_sym(); bldr.new_load(op_load2, num, False, rmu.MuMemOrd.NOT_ATOMIC, i64, irhd_fld)
        op_getfieldiref2 = bldr.gen_sym(); bldr.new_getfieldiref(op_getfieldiref2, irhd_fld2, False, node, 1, irhd)
        op_load3 = bldr.gen_sym(); bldr.new_load(op_load3, rnxt, False, rmu.MuMemOrd.NOT_ATOMIC, refnode, irhd_fld2)
        op_getiref2 = bldr.gen_sym(); bldr.new_getiref(op_getiref2, irnxt, node, rnxt)
        op_getfieldiref3 = bldr.gen_sym(); bldr.new_getfieldiref(op_getfieldiref3, irnxt_fld, False, node, 0, irnxt)
        op_load4 = bldr.gen_sym(); bldr.new_load(op_load4, n_nxt, False, rmu.MuMemOrd.NOT_ATOMIC, i64, irnxt_fld)
        op_add = bldr.gen_sym(); bldr.new_binop(op_add, res, rmu.MuBinOptr.ADD, i64, num, n_nxt)
        op_ret = bldr.gen_sym(); bldr.new_ret(op_ret, [res])
        bldr.new_bb(blk0, [], [], rmu.MU_NO_ID, [op_load1, op_getiref1, op_getfieldiref1, op_load2, op_getfieldiref2,
                                                 op_load3, op_getiref2, op_getfieldiref3, op_load4, op_add, op_ret])

        bldr.new_func_ver(test_fnc_v1, test_fnc, [blk0])

        return {
            "test_fnc": test_fnc,
            "test_fnc_sig": sig__i64,
            "result_type": i64,
            "@i64": i64,
            "@refi64": refi64,
            "@node": node,
            "@refnode": refnode,
            "@NULL": NULL,
            "@gcl": gcl,
        }

    def init_heap(ctx, id_dic, rmu):
        """
        :type ctx: rpython.rlib.rmu.MuCtx
        :type id_dic: dict
        :type rmu: rpython.rlib.rmu
        """
        ref_hd = ctx.new_fixed(id_dic['@node'])
        ref_nxt = ctx.new_fixed(id_dic['@node'])
        iref_hd = ctx.get_iref(ref_hd)
        iref_hd_num = ctx.get_field_iref(iref_hd, 0)
        hdl_num = ctx.handle_from_sint64(42, 64)
        ctx.store(rmu.MuMemOrd.NOT_ATOMIC, iref_hd_num, hdl_num)

        iref_hd_nxt = ctx.get_field_iref(iref_hd, 1)
        ctx.store(rmu.MuMemOrd.NOT_ATOMIC, iref_hd_nxt, ref_nxt)

        iref_nxt = ctx.get_iref(ref_nxt)
        iref_nxt_num = ctx.get_field_iref(iref_nxt, 0)
        hdl_num = ctx.handle_from_sint64(256, 64)
        ctx.store(rmu.MuMemOrd.NOT_ATOMIC, iref_nxt_num, hdl_num)

        iref_nxt_nxt = ctx.get_field_iref(iref_nxt, 1)
        hdl_NULL = ctx.handle_from_const(id_dic['@NULL'])
        ctx.store(rmu.MuMemOrd.NOT_ATOMIC, iref_nxt_nxt, hdl_NULL)

        gcl_hdl = ctx.handle_from_global(id_dic['@gcl'])
        ctx.store(rmu.MuMemOrd.NOT_ATOMIC, gcl_hdl, ref_hd)

    (fnp, _), (mu, ctx, bldr) = fncptr_from_py_script(build_test_bundle, init_heap, 'test_fnc')

    mu.current_thread_as_mu_thread(rmu.null(rmu.MuCPtr))
    assert fnp() == 298
