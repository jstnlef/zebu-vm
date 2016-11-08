"""
Converts MuCtx methods in muapi.h to nativeClientSupport

USAGE: python3 muapitoncs.py

Code will be automatically generated to cStubs.scala

"""

import sys
import os, os.path
import re
import tempfile
from typing import Tuple, List

import muapiparser
import injecttools
from muimplfastinjectablefiles import injectable_files, muapi_h_path

# C types to Rust types

_primitive_types = {
        "void"      : "c_void",
        "char"      : "c_char",
        "int"       : "c_int",
        "long"      : "c_long",
        "int8_t"    : "i8",
        "uint8_t"   : "u8",
        "int16_t"   : "i16",
        "uint16_t"  : "u16",
        "int32_t"   : "i32",
        "uint32_t"  : "u32",
        "int64_t"   : "i64",
        "uint64_t"  : "u64",
        "intptr_t"  : "isize",
        "uintptr_t" : "usize",
        "float"     : "f32",
        "double"    : "f64",
        }

def cty_is_explicit_ptr(cty):
    return cty.endswith("*")

def cty_get_base_type(cty):
    assert(cty_is_explicit_ptr(cty))
    return cty[:-1]

r_handle_ty = re.compile(r'^Mu\w*(Value)$')

def cty_is_handle(cty):
    return r_handle_ty.match(cty) is not None

r_node_ty = re.compile(r'^Mu\w*(Node|Clause)$')

def cty_is_node(cty):
    return r_node_ty.match(cty) is not None

def cty_should_be_mutable(base_cty):
    return not cty_is_handle(base_cty)

def to_rust_type(cty):
    if cty in _primitive_types:
        rty = _primitive_types[cty]
    elif cty_is_explicit_ptr(cty):
        base_type = cty_get_base_type(cty)
        base_rust_type = to_rust_type(base_type)
        rty = "*mut " + base_rust_type
    else:
        rty = "C" + cty

    return rty

__rust_kw_rewrite = {
        "ref": "reff",
        }

def avoid_rust_kws(name):
    return __rust_kw_rewrite.get(name, name)

def filler_name_for(struct_name):
    return "_fill__" + struct_name

def forwarder_name_for(struct_name, meth_name):
    return "_forwarder__" + struct_name + "__" + meth_name

def generate_struct_field(meth) -> str:
    name    = meth['name']
    params  = meth['params']
    ret_ty  = meth['ret_ty']

    rust_param_tys = []
    for param in params:
        c_ty = param['type']
        rust_ty = to_rust_type(c_ty)
        rust_param_tys.append(rust_ty)

    rust_ret_ty = None if ret_ty == "void" else to_rust_type(ret_ty)
    ret_ty_text = "" if rust_ret_ty == None else " -> {}".format(rust_ret_ty)
    
    field_def = "    pub {}: extern fn({}){},".format(
            name, ", ".join(rust_param_tys), ret_ty_text)

    return field_def

_no_conversion = {
        *_primitive_types.keys(),

        # These are used as raw data.
        # Even the implementation layer has to use the raw C types.
        "MuCPtr",
        "MuCFP",

        # These are C functions provided and regisered by the client.
        # They should be treated like C functions.
        "MuValueFreer",
        "MuTrapHandler",

        # Watch point ID is considered as primitive.
        "MuWPID",

        # These are enum types. Passed to the micro VM as is.
        "MuBinOpStatus",
        "MuBinOptr",
        "MuCmpOptr",
        "MuConvOptr",
        "MuMemOrd",
        "MuAtomicRMWOptr",
        "MuCallConv",
        "MuCommInst",
        }

_cty_to_high_level_ty = {
        "MuVM*": "*mut CMuVM",
        "MuCtx*": "*mut CMuCtx",
        "MuIRBuilder*": "*mut CMuIRBuilder",
        "MuBool": "bool",
        "MuID": "MuID",
        }

_cty_to_high_level_param_ty = {
        **_cty_to_high_level_ty,

        # If the micro VM wants a string, we make it convenient.
        "MuName": "MuName",
        "MuCString": "String",
        }

_cty_to_high_level_ret_ty = {
        **_cty_to_high_level_ty,

        # If the client wants a string, it has to be kept permanent in the micro VM.
        "MuName": "CMuCString",
        }

_cty_directly_returned = {
        *_no_conversion,
        # see above
        "MuCString",
        "MuName",

        # To be safe, let the micro VM fill up the structs.
        "MuVM*",
        "MuCtx*",
        "MuIRBuilder*", 
        }

def to_high_level_ret_ty(cty, rty):
    assert cty != "void"
    if cty in _cty_to_high_level_ret_ty:
        hlt = _cty_to_high_level_ret_ty[cty]
    elif cty_is_handle(cty):
        hlt = "*const APIMuValue"
    elif cty_is_node(cty):
        hlt = "MuID"
    else:
        hlt = rty

    return hlt

_special_self_style = {
        }

def generate_forwarder_and_stub(st, meth) -> Tuple[str, str]:
    st_name = st['name']

    name    = meth['name']
    params  = meth['params']
    ret_ty  = meth['ret_ty']

    stmts = []

    fwdr_name = forwarder_name_for(st["name"], name)

    # forwarder formal parameter list

    fwdr_param_list = []

    for param in params:
        cpn = param['name']
        rpn = avoid_rust_kws(cpn)
        cty = param['type']
        rty = to_rust_type(cty)
        fwdr_param_list.append("{}: {}".format(rpn, rty))

    fwdr_param_list_joined = ", ".join(fwdr_param_list)
    
    # forwarder return type
    
    rust_ret_ty = None if ret_ty == "void" else to_rust_type(ret_ty)
    fwdr_ret_ty_text = "" if rust_ret_ty == None else " -> {}".format(rust_ret_ty)

    # stub formal parameter list and return type

    stub_param_list = []

    stub_ret_ty = None if rust_ret_ty is None else to_high_level_ret_ty(ret_ty, rust_ret_ty)
    stub_ret_ty_text = "" if stub_ret_ty == None else " -> {}".format(stub_ret_ty)

    # Preparing actual arguments in the forwarder body,
    # and compute the corresponding stub formal parameter type.

    args = []

    for param in params:
        is_sz_param = param.get("is_sz_param", False)

        if is_sz_param:
            # Skip size parameters. Instead, make slices from them.
            continue

        cpn = param['name']
        rpn = avoid_rust_kws(cpn)
        cty = param['type']
        rty = to_rust_type(cty)

        arg_name = "_arg_" + rpn

        array_sz_param = param.get("array_sz_param", None)
        is_optional    = param.get("is_optional", False)
        is_out         = param.get("is_out", False)

        # Compute `converter` (the expression to get the actual argument)
        # and `stub_rty` (the stub's corresponding formal parameter type).
        if is_out:
            assert cty_is_explicit_ptr(cty)
            # Do not convert out param.
            converter = rpn
            # Keep as ptr so that Rust prog can store into it.
            stub_rty = rty
        elif array_sz_param != None:
            assert cty_is_explicit_ptr(cty)

            # It is array. Make a slice out of it.

            c_base_ty = cty_get_base_type(cty)
            # We don't have array of explicit pointers, but we do have array of MuValue or MuCPtr.
            assert not cty_is_explicit_ptr(c_base_ty)
            r_base_ty = to_rust_type(c_base_ty)

            sz_cpn = array_sz_param
            sz_rpn = avoid_rust_kws(sz_cpn)
            if cty_is_handle(c_base_ty):
                converter = "from_handle_array({}, {})".format(
                        rpn, sz_rpn)
                stub_rty = "Vec<&APIMuValue>"
            elif cty_is_node(c_base_ty) or c_base_ty == "MuID":
                converter = "from_MuID_array({}, {})".format(
                        rpn, sz_rpn)
                stub_rty = "Vec<MuID>"
            else:
                converter = "from_{}_array({}, {})".format(
                        c_base_ty, rpn, sz_rpn)
                if c_base_ty == "MuCString":
                    stub_rty = "Vec<String>"
                else:
                    stub_rty = "&[{}]".format(r_base_ty)
        elif is_optional:
            # The parameter is optional. Wrap it with Option<>

            if cty_is_handle(cty):
                converter = "from_handle_optional({})".format(rpn)
                stub_rty = "Option<&APIMuValue>"
            elif cty_is_node(cty):
                converter = "from_MuID_optional({})".format(rpn)
                stub_rty = "Option<MuID>"
            elif cty in ["MuCString", "MuID"]:
                converter = "from_{}_optional({})".format(cty, rpn)
                stub_rty = "Option<{}>".format(_cty_to_high_level_param_ty[cty])
            else:
                raise Exception("Not expecting {} to be optional".format(cty))
        else:
            # scalar value
            if cty_is_explicit_ptr(cty):   # MuVM*, MuCtx*, MuIRBuilder*
                c_base_ty = cty_get_base_type(cty)
                converter = "from_{}_ptr({})".format(c_base_ty, rpn)
                stub_rty = to_rust_type(c_base_ty)
            elif cty_is_handle(cty):
                converter = "from_handle({})".format(rpn)
                stub_rty = "&APIMuValue"
            elif cty_is_node(cty):
                converter = "from_MuID({})".format(rpn)
                stub_rty = "MuID"
            elif cty in _no_conversion:
                converter = rpn     # Do not convert primitive types.
                stub_rty = rty
            elif cty in _cty_to_high_level_param_ty:
                converter = "from_{}({})".format(cty, rpn)
                stub_rty = _cty_to_high_level_param_ty[cty]
            else:
                raise Exception("Don't know how to handle param type {}".format(cty))
                
        stmt = "    let mut {} = {};".format(arg_name, converter)
        stmts.append(stmt)

        args.append(arg_name)

        stub_param_list.append("{}: {}".format(rpn, stub_rty))

    # call

    self_arg = args[0]
    other_args = args[1:]
    args_joined = ", ".join(other_args)
    ret_val_bind = "" if rust_ret_ty is None else "let _rv = "
    stmts.append("    {}unsafe {{".format(ret_val_bind))
    call_stmt = '        (*{}).{}({})'.format(
            self_arg, name, args_joined)
    stmts.append(call_stmt)
    stmts.append("    };")

    # return values

    if rust_ret_ty is not None:
        if ret_ty in _cty_directly_returned:
            converter = "_rv"
        elif cty_is_handle(ret_ty):
            converter = "to_handle(_rv)"
        elif cty_is_node(ret_ty):
            converter = "to_MuID(_rv)"
        else:
            converter = "to_{}(_rv)".format(ret_ty)
        stmts.append("    let _rv_prep = {};".format(converter))
        stmts.append("    _rv_prep")

    # stmts.append('    panic!("not implemented")')

    # forwarder

    all_stmts = "\n".join(stmts)

    bridge = """\
extern fn {fwdr_name}({fwdr_param_list_joined}){fwdr_ret_ty_text} {{
{all_stmts}
}}
""".format(**locals())

    # stub

    stub_param_list[0] = _special_self_style.get((st_name, name), "&mut self")
    stub_param_list_joined = ", ".join(stub_param_list)

    stub = """\
    pub fn {name}({stub_param_list_joined}){stub_ret_ty_text} {{
        panic!("Not implemented")
    }}
""".format(**locals())

    return bridge, stub

def generate_filler_stmt(st, meth) -> str:
    name = meth['name']
    forwarder_name = forwarder_name_for(st["name"], name)

    stmt = "        {}: {},".format(
            name, forwarder_name)

    return stmt


def visit_method(st, meth) -> Tuple[str, str, str, str]:
    field_def = generate_struct_field(meth)
    bridge, stub = generate_forwarder_and_stub(st, meth)
    filler_stmt = generate_filler_stmt(st, meth)

    return field_def, bridge, filler_stmt, stub

_lifetime_params = {
        "MuVM": "",
        "MuCtx": "<'v>",
        "MuIRBuilder": "<'c>",
        }

def visit_struct(st) -> Tuple[str, List[str], str, str]:
    name    = st["name"]
    methods = st["methods"]

    rust_name = "C" + name

    field_defs = []
    forwarders = []
    filler_stmts = []
    stubs = []

    for meth in methods:
        field_def, forwarder, filler_stmt, stub = visit_method(st, meth)
        field_defs.append(field_def)
        forwarders.append(forwarder)
        filler_stmts.append(filler_stmt)
        stubs.append(stub)

    fields = "\n".join(field_defs)

    # Note: The header is private to the IMPLEMENTATION, but the implementation
    # is in another Rust module. So it should be "pub" w.r.t. Rust modules.
    struct_def = """\
#[repr(C)]
pub struct {rust_name} {{
    pub header: *mut c_void,
{fields}
}}
""".format(**locals())

    filler_stmts_joined = "\n".join(filler_stmts)

    filler = """\
pub fn make_new_{name}(header: *mut c_void) -> *mut {rust_name} {{
    let bx = Box::new({rust_name} {{
        header: header,
{filler_stmts_joined}
    }});

    Box::into_raw(bx)
}}
""".format(**locals())

    stubs_joined = "\n".join(stubs)

    #lifetime_params = _lifetime_params[name]
    lifetime_params = ""

    stub_impl = """\
impl{lifetime_params} {name}{lifetime_params} {{
{stubs_joined}
}}
""".format(**locals())

    return struct_def, forwarders, filler, stub_impl


def visit_structs(ast) -> Tuple[str, str, str, str]:
    struct_defs = []
    forwarders = []
    fillers = []
    stub_impls = []

    structs = ast["structs"]

    for struct in structs:
        struct_def, my_forwarders, filler, stub_impl = visit_struct(struct)
        struct_defs.append(struct_def)
        forwarders.extend(my_forwarders)
        fillers.append(filler)
        stub_impls.append(stub_impl)

    return ("\n".join(struct_defs), "\n".join(forwarders), "\n".join(fillers),
            "\n".join(stub_impls))

def visit_enums(ast):
    const_defs = []

    for enum in ast['enums']:
        cty = enum['name']
        rty = to_rust_type(cty)
        for d in enum['defs']:
            const_name = 'C' + d['name']
            const_value = d['value']
            const_defs.append("pub const {}: {} = {};".format(const_name, rty, const_value))

    return "\n".join(const_defs)

# Manually define the following types in Rust, disregarding the typedefs in muapi.h
_manual_typedefs = {
        "MuCString",
        "MuValue",
        }

def visit_types(ast):
    types = []
    for c, p in ast["typedefs_order"]:
        if p.startswith("_"):
            # Such types are function types. The muapiparser is not smart enough
            # to parse C funcptr types, so we define these types manually.
            continue
        elif c in _manual_typedefs:
            # These types are defined manually, overriding the muapi.h
            continue
        rc = to_rust_type(c)
        rp = to_rust_type(p)
        types.append("pub type {} = {};".format(rc, rp))

    return "\n".join(types)

def main():
    with open(muapi_h_path) as f:
        src_text = f.read()

    ast = muapiparser.parse_muapi(src_text)

    types = visit_types(ast)

    structs, forwarders, fillers, stub_impls = visit_structs(ast)

    enums = visit_enums(ast)

    injectable_files["api_c.rs"].inject_many({
        "Types": types,
        "Structs": structs,
        "Enums": enums,
        })

    injectable_files["api_bridge.rs"].inject_many({
        "Forwarders": forwarders,
        "Fillers": fillers,
        })

    injectable_files["__api_impl_stubs.rs"].inject_many({
        "StubImpls": stub_impls,
        })

if __name__=='__main__':
    main()
