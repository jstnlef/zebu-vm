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

_other_ptr_types = {
        # In the most recent muapi.h, these can be identified as explicit pointers.
        #"MuName", "MuCFP", "MuTrapHandler", "MuValueFreer"
        # Add more types here if the regexp cannot identify some pointer types.
        }

_self_getters = {
        "MuVM*": "getMicroVM",
        "MuCtx*": "getMuCtx",
        "MuIRBuilder*": "getMuIRBuilder",
        }

def type_is_explicit_ptr(ty):
    return ty.endswith("*")

r_handle_ty = re.compile(r'^Mu\w*(Value)$')

def type_is_handle(ty):
    return r_handle_ty.match(ty) is not None

r_node_ty = re.compile(r'^Mu\w*(Node|Clause)$')

def type_is_node(ty):
    return r_node_ty.match(ty) is not None

def type_is_ptr(ty):
    return type_is_explicit_ptr(ty) or type_is_handle(ty) or ty in _other_ptr_types

def type_is_handle_array(ty):
    return type_is_ptr(ty) and type_is_handle(ty[:-1])

def type_is_node_array(ty):
    return type_is_ptr(ty) and type_is_node(ty[:-1])

def to_rust_type(raw_type):
    if type_is_explicit_ptr(raw_type):
        base_type = raw_type[:-1]
        base_rust_type = to_rust_type(base_type)
        rust_type = "*mut " + base_rust_type
    elif raw_type in _primitive_types:
        rust_type = _primitive_types[raw_type]
    else:
        rust_type = "C" + raw_type

    return rust_type

#def to_jffi_getter(raw_type):
#    if raw_type in _primitive_types:
#        getter = _primitive_types[raw_type][2]
#    elif type_is_ptr(raw_type):
#        getter = "getAddress"
#    else:
#        raise ValueError("No JFFI Buffer getter: " + raw_type)
#
#    return getter
#
#def to_jffi_setter(raw_type):
#    if raw_type in _primitive_types:
#        getter = _primitive_types[raw_type][3]
#    elif type_is_ptr(raw_type):
#        getter = "setAddressReturn"
#    else:
#        raise ValueError("No JFFI Buffer getter: " + raw_type)
#
#    return getter
#
#_special_cases = {
#        "id":             "ID",
#        "sint8":          "SInt8",
#        "uint8":          "UInt8",
#        "sint16":         "SInt16",
#        "uint16":         "UInt16",
#        "sint32":         "SInt32",
#        "uint32":         "UInt32",
#        "sint64":         "SInt64",
#        "uint64":         "UInt64",
#        "uint64s":        "UInt64s",
#        "fp":             "FP",
#        "uptr":           "UPtr",
#        "ufuncptr":       "UFuncPtr",
#        "iref":           "IRef",
#        "weakref":        "WeakRef",
#        "funcref":        "FuncRef",
#        "tagref64":       "TagRef64",
#        "threadref":      "ThreadRef",
#        "stackref":       "StackRef",
#        "framecursorref": "FrameCursorRef",
#        "irnoderef":      "IRNodeRef",
#        "funcsig":        "FuncSig",
#        "bb":             "BB",
#        "binop":          "BinOp",
#        "tailcall":       "TailCall",
#        "extractvalue":   "ExtractValue",
#        "insertvalue":    "InsertValue",
#        "extractelement": "ExtractElement",
#        "insertelement":  "InsertElement",
#        "shufflevector":  "ShuffleVector",
#        "newhybrid":      "NewHybrid",
#        "allocahybrid":   "AllocaHybrid",
#        "getiref":        "GetIRef",
#        "getfieldiref":   "GetFieldIRef",
#        "getelemiref":    "GetElemIRef",
#        "shiftiref":      "ShiftIRef",
#        "getvarpartiref": "GetVarPartIRef",
#        "cmpxchg":        "CmpXchg",
#        "atomicrmw":      "AtomicRMW",
#        "watchpoint":     "WatchPoint",
#        "wpbranch":       "WPBranch",
#        "ccall":          "CCall",
#        "newthread":      "NewThread",
#        "newstack":       "NewStack",
#        "swapstack":      "SwapStack",
#        "comminst":       "CommInst",
#        "ir":             "IR",
#        "irbuilderref":   "IRBuilderRef",
#        }
#
#def toCamelCase(name):
#    ins = name.split("_")
#    outs = [ins[0]]
#    for inn in ins[1:]:
#        if inn in _special_cases:
#            outs.append(_special_cases[inn])
#        else:
#            outs.append(inn[0].upper()+inn[1:])
#
#    return "".join(outs)
#
#def to_basic_type(typedefs, name):
#    while name in typedefs:
#        name = typedefs[name]
#    return name
#
#_no_conversion = {
#        # "MuID",          # It may be optional, in which case it needs conversion.
#        "MuTrapHandler", # It is a function pointer. Handle in Scala.
#        "MuCPtr",        # Intended to be raw pointer. Passed directly.
#        "MuCFP",         # ditto
#        "MuWPID",        # Just Int
#        # "MuCommInst",    # same as MuID
#        }
#
#_array_converters = {
#        "char*"     : "readCharArray",
#        "uint64_t*" : "readLongArray",
#        "MuFlag*"   : "readFlagArray",
#        "MuID*"     : "readIntArray",
#        "MuCString*": "readCStringArray",
#        }
#
#_special_converters = {
#        "MuBool"          : "intToBoolean",
#        "MuName"          : "readCString",
#        "MuCString"       : "readCString",
#        "MuMemOrd"        : "toMemoryOrder",
#        "MuAtomicRMWOptr" : "toAtomicRMWOptr",
#        "MuBinOpStatus"   : "toBinOpStatus",
#        "MuBinOptr"       : "toBinOptr",
#        "MuCmpOptr"       : "toCmpOptr",
#        "MuConvOptr"      : "toConvOptr",
#        "MuCallConv"      : "toCallConv",
#        "MuCommInst"      : "toCommInst",
#        }
#
#_special_return_converters = {
#        "MuBool" : "booleanToInt",
#        "MuName" : "exposeString",
#        "MuVM*"  : "exposeMicroVM",
#        "MuCtx*" : "exposeMuCtx",
#        "MuIRBuilder*" : "exposeMuIRBuilder",
#        }
#
#def param_converter(pn, pt, rn, rt, is_optional, array_sz, is_out):
#    if pt == "void":
#        raise ValueError("Parameter cannot be void. Param name: {}".format(pn))
#
#    if pt in _primitive_types or pt in _no_conversion or is_out:
#        return rn   # does not need conversion
#
#    if type_is_node(pt) or pt == "MuID":
#        if is_optional:
#            return "readMuIDOptional({})".format(rn)
#        return rn   # does not need conversion
#
#    if array_sz is not None:
#        if type_is_handle_array(pt):
#            ac = "readMuValueArray"
#        elif type_is_node_array(pt):
#            ac = "readMuIDArray"
#        elif pt in _array_converters:
#            ac = _array_converters[pt]
#        else:
#            raise ValueError("I don't know how to convert array {}. "
#                    "Param name: {}, array size: {}".format(pt, pn, array_sz))
#        return "{}({}, {})".format(ac, rn, "_raw_"+array_sz)
#
#    if type_is_handle(pt):
#        if is_optional:
#            return "getMuValueNullable({}).asInstanceOf[Option[{}]]".format(rn, pt)
#        else:
#            return "getMuValueNotNull({}).asInstanceOf[{}]".format(rn, pt)
#
#    if pt in _special_converters:
#        converter = _special_converters[pt]
#        if is_optional:
#            converter = converter + "Optional"
#        return "{}({})".format(converter, rn)
#
#    raise ValueError("I don't know how to convert {}. Param name: {}".format(
#        pt, pn))
#
#def generate_method(typedefs, strname, meth) -> Tuple[str, str]:
#    name    = meth['name']
#    params  = meth['params']
#    ret_ty  = meth['ret_ty']
#
#    valname = strname.upper() + "__" + name.upper()
#
#    jffi_retty = to_jffi_ty(to_basic_type(typedefs, ret_ty))
#    jffi_paramtys = [to_jffi_ty(to_basic_type(typedefs, p["type"])) for p in params]
#
#    pretty_name = "{}.{}".format(strname, name)
#
#    header = 'val {} = exposedMethod("{}", {}, Array({})) {{ _jffiBuffer =>'.format(
#            valname, pretty_name, jffi_retty, ", ".join(jffi_paramtys))
#
#    stmts = []
#
#    # get raw parameters
#    for i in range(len(params)):
#        param = params[i]
#        pn = param['name']
#        pt = param['type']
#        rt = to_basic_type(typedefs, pt) # raw type
#        jffi_getter = to_jffi_getter(rt)
#
#        rn = "_raw_" + pn # raw name
#
#        stmts.append("val {} = _jffiBuffer.{}({})".format(rn,
#            jffi_getter, i))
#
#    self_param_name = params[0]["name"]
#    self_param_type = params[0]["type"]
#
#    # get the self object (MuVM or MuCtx)
#
#    stmts.append("val {} = {}({})".format(
#        self_param_name,
#        _self_getters[self_param_type],
#        "_raw_"+self_param_name))
#
#    # convert parameters
#    args_to_pass = []
#
#    for i in range(1, len(params)):
#        param = params[i]
#        pn = param['name']
#
#        if param.get("is_sz_param", False):
#            continue    # Array sizes don't need to be passed explicitly.
#
#        args_to_pass.append(pn)
#
#        pt = param['type']
#        rn = "_raw_" + pn
#        rt = to_basic_type(typedefs, pt)
#
#        array_sz = param.get("array_sz_param", None)
#        is_optional = param.get("is_optional", False)
#        is_out = param.get("is_out", False)
#
#        pc = param_converter(pn, pt, rn, rt, is_optional, array_sz, is_out)
#
#        stmts.append("val {} = {}".format(pn, pc))
#
#    # make the call
#
#    camelName = toCamelCase(name)
#    stmts.append("val _RV = {}.{}({})".format(
#        self_param_name, camelName, ", ".join(args_to_pass)))
#
#    # return value
#
#    if ret_ty != "void":
#        raw_ret_ty = to_basic_type(typedefs, ret_ty)
#        jffi_setter = to_jffi_setter(raw_ret_ty)
#
#        if type_is_handle(ret_ty):
#            assert(strname == "MuCtx")
#            assert(jffi_setter == "setAddressReturn")
#            stmts.append("val _RV_FAK = exposeMuValue({}, _RV)".format(
#                self_param_name))
#            stmts.append("_jffiBuffer.{}(_RV_FAK)".format(jffi_setter))
#        elif ret_ty in _special_return_converters:
#            assert(ret_ty == "MuBool" or jffi_setter == "setAddressReturn")
#            stmts.append("val _RV_FAK = {}(_RV)".format(
#                _special_return_converters[ret_ty]))
#            stmts.append("_jffiBuffer.{}(_RV_FAK)".format(jffi_setter))
#        else:
#            stmts.append("_jffiBuffer.{}(_RV)".format(jffi_setter))
#
#
#    footer = "}"
#
#    return (valname, "\n".join([header] + stmts + [footer]))

def generate_stubs_for_struct(typedefs, st) -> str:
    name    = st["name"]
    methods = st["methods"]

    results = []
    ptrs    = []

    for meth in methods:
        ptrname, code = generate_method(typedefs, name, meth)
        ptrs.append(ptrname)
        results.append(code)

    results.append("val stubsOf{} = new Array[Word]({})".format(name, len(ptrs)))
    for i,ptr in enumerate(ptrs):
        results.append("stubsOf{}({}) = {}.address".format(name, i, ptr))

    return "\n".join(results)

def generate_stubs(ast):
    struct_codes = []

    for st in ast["structs"]:
        code = generate_stubs_for_struct(ast["typedefs"], st)
        struct_codes.append(code)

    return "\n".join(struct_codes)

_enum_types_to_generate_converters = [
        ("MuBinOptr",       "BinOptr",       'MU_BINOP_'),
        ("MuCmpOptr",       "CmpOptr",       'MU_CMP_'),
        ("MuConvOptr",      "ConvOptr",      'MU_CONV_'),
        ("MuMemOrd",        "MemoryOrder",   'MU_ORD_'),
        ("MuAtomicRMWOptr", "AtomicRMWOptr", 'MU_ARMW_'),
        ]

def generate_enum_converters(ast):
    enums = ast['enums']
    edict = {}

    for e in enums:
        edict[e['name']] = e['defs']

    lines = []

    for cty, sty, prefix in _enum_types_to_generate_converters:
        func_name = "to"+sty
        lines.append("def {}(cval: {}): {}.Value = cval match {{".format(
            func_name, cty, sty))

        defs = edict[cty]
        for d in defs:
            dn = d['name']
            dv = d['value']
            assert(dn.startswith(prefix))
            sn = dn[len(prefix):]
            lines.append("  case {} => {}.{}".format(dv, sty, sn))

        lines.append("}")

    return "\n".join(lines)

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
    ret_ty_text = "" if rust_ret_ty == None else "-> {}".format(rust_ret_ty)
    
    field_def = "    pub {}: fn({}){},".format(
            name, ", ".join(rust_param_tys), ret_ty_text)

    return field_def

def generate_forwarder(st, meth) -> str:
    name    = meth['name']
    params  = meth['params']
    ret_ty  = meth['ret_ty']

    stmts = []

    forwarder_name = forwarder_name_for(st["name"], name)

    # formal parameter list

    param_nts = []

    for param in params:
        cpn = param['name']
        rpn = avoid_rust_kws(cpn)
        cty = param['type']
        rty = to_rust_type(cty)
        param_nts.append("{}: {}".format(rpn, rty))

    formal_param_list = ", ".join(param_nts)
    
    # return type
    
    rust_ret_ty = None if ret_ty == "void" else to_rust_type(ret_ty)
    ret_ty_text = "" if rust_ret_ty == None else "-> {}".format(rust_ret_ty)

    # preparing args
    for param in params:
        is_sz_param = param.get("is_sz_param", False)

        if is_sz_param:
            continue

        cpn = param['name']
        rpn = avoid_rust_kws(cpn)
        cty = param['type']
        rty = to_rust_type(cty)

        arg_name = "_arg_" + rpn

        array_sz_param = param.get("array_sz_param", None)
        is_optional    = param.get("is_optional", False)

        if array_sz_param != None:
            assert cty.endswith("*")
            c_base_ty = cty[:-1]

            sz_cpn = array_sz_param
            sz_rpn = avoid_rust_kws(sz_cpn)
            if type_is_ptr(c_base_ty):
                converter = "from_ptr_array({}, {})".format(
                        rpn, sz_rpn)
            elif type_is_node(c_base_ty):
                converter = "from_MuID_array({}, {})".format(
                        rpn, sz_rpn)
            else:
                converter = "from_{}_array({}, {})".format(
                        c_base_ty, rpn, sz_rpn)
        elif is_optional:
            if type_is_ptr(cty):
                converter = "from_ptr_optional({})".format(rpn)
            elif type_is_node(cty):
                converter = "from_node_optional({})".format(rpn)
            elif cty in ["MuCString", "MuID"]:
                converter = "from_{}_optional({})".format(cty, rpn)
            else:
                raise Exception("Not expecting {} to be optional".format(cty))
        else:
            if cty.endswith("*"):
                c_base_ty = cty[:-1]
                converter = "from_{}_ptr({})".format(c_base_ty, rpn)
            else:
                converter = "from_{}({})".format(cty, rpn)

        stmt = "    let mut {} = {};".format(arg_name, converter)
        stmts.append(stmt)

    # call

    stmts.append('    panic!("not implemented")')
    all_stmts = "\n".join(stmts)

    bridge = """\
fn {forwarder_name}({formal_param_list}){ret_ty_text} {{
{all_stmts}
}}
""".format(**locals())

    return bridge

def visit_method(st, meth) -> Tuple[str, str]:
    field_def = generate_struct_field(meth)
    bridge = generate_forwarder(st, meth)

    return field_def, bridge

def visit_struct(st) -> Tuple[str, List[str]]:
    name    = st["name"]
    methods = st["methods"]

    rust_name = "C" + name

    field_defs = []
    forwarders = []

    for meth in methods:
        field_def, forwarder = visit_method(st, meth)
        field_defs.append(field_def)
        forwarders.append(forwarder)

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

    return struct_def, forwarders


def visit_structs(ast) -> Tuple[str, str]:
    struct_defs = []
    forwarders = []

    structs = ast["structs"]

    for struct in structs:
        struct_def, my_forwarders = visit_struct(struct)
        struct_defs.append(struct_def)
        forwarders.extend(my_forwarders)

    return "\n".join(struct_defs), "\n".join(forwarders)

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

def visit_types(ast):
    types = []
    for c, p in ast["typedefs_order"]:
        if p.startswith("_"):
            # Such types are function types. The muapiparser is not smart enough
            # to parse C funcptr types, so we define these types manually.
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

    structs, forwarders = visit_structs(ast)

    enums = visit_enums(ast)

    injectable_files["api_c.rs"].inject_many({
        "Types": types,
        "Structs": structs,
        "Enums": enums,
        })

    injectable_files["api_bridge.rs"].inject_many({
        "Forwarders": forwarders,
        })

if __name__=='__main__':
    main()
