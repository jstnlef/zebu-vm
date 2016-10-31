#ifndef __MUAPI_H__
#define __MUAPI_H__

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

// ABOUT HEADER PARSERS
//
// This header is designed to be processed by a regular-expression-based parser
// so that it will be convenient to to generate language bindings. There is one
// such parser in the reference implementation v2: muapiparser.py
//
// It can identify the type hierarchies of the typedefs below, so language
// bindings can auto-generate a class hierarchy for MuValue handles if the
// high-level language supports, such as Python. For the convenience of this
// purpose, all typedefs are in the simplest form: "typedef FromTy ToType;"
// where FromTy may end with an asterisk '*'. Typedefs of function pointer types
// are deliberately separated into two steps (see MuCFP below), because
// regexp-based parsers may not be smart enough to parse function pointer types.
//
// It can find all definitions of macro constants. These constants, such as
// MU_BINOP_ADD, all have prefixes to make parsing easy.
//
// It can find methods of MuVM and MuCtx which are structs of function pointers.
// Parameter types use high-level typedef-ed types so that the language binding
// generator has more information (such as MuInstResNode) than the raw C types
// (such as void*).
//
// The "/// MUAPIPARSER" comments provide extra annotations (pragmas) for the
// method just before it. Multiple pragmas are separated by semicolons ';'.
//
//  param:array:sz_param
//      means param is a pointer to an array, and the effective length is
//      determined by the sz_param parameter. param can be NULL, in which case
//      it is considered as a 0-length array.
//  param:optional
//      means param is a pointer and it may be NULL.
//  param:out
//      means param is a pointer and it is supposed to be used as an output
//      parameter. Values should be written in rather than read out of it.


// MuValue and MuXxxValue type are opaque handles to values in the Mu type
// system.
//
// The actual values are held by MuCtx. MuValue opaquely refers to one such
// value. Copies of MuValue values refer to the same value. A MuValue instance
// can only be used in the MuCtx holding it.
//
// Values of subtypes can be cast to/from their abstract parents using the type
// cast expression in C, similar to casting one pointer to another.

// Top value type.
typedef void *MuValue;                // Any Mu value

// Abstract value type.
typedef MuValue MuSeqValue;           // array or vector
typedef MuValue MuGenRefValue;        // ref, iref, funcref, threadref, stackref, framecursorref, irbuilderref

// concrete value types
typedef MuValue MuIntValue;           // int<n>
typedef MuValue MuFloatValue;         // float
typedef MuValue MuDoubleValue;        // double
typedef MuValue MuUPtrValue;          // uptr
typedef MuValue MuUFPValue;           // ufuncptr

typedef MuValue MuStructValue;        // struct<...>
typedef MuSeqValue MuArrayValue;      // array<T l>
typedef MuSeqValue MuVectorValue;     // vector<T l>

typedef MuGenRefValue MuRefValue;           // ref<T>
typedef MuGenRefValue MuIRefValue;          // iref<T>
typedef MuGenRefValue MuTagRef64Value;      // tagref64
typedef MuGenRefValue MuFuncRefValue;       // funcref<sig>
typedef MuGenRefValue MuThreadRefValue;     // threadref
typedef MuGenRefValue MuStackRefValue;      // stackref
typedef MuGenRefValue MuFCRefValue;         // framecursorref
typedef MuGenRefValue MuIBRefValue;         // irbuilderref
// NOTE: The "irbuilderref" exists so that Mu IR programs themselves can build
// Mu IR. If you want to build IR using this C API, use MuCtx->new_ir_builder.

// C-style '\0'-terminated string
typedef char *MuCString;

// Identifiers and names of Mu
typedef uint32_t MuID;
typedef MuCString MuName;

// Invalid ID. Used when an ID is optional.
#define MU_NO_ID    ((MuID)0)

// Convenient types for the void* type and the void(*)() type in C
typedef void *MuCPtr;
typedef void _MuCFP_Func();
typedef _MuCFP_Func* MuCFP;

// Boolean type. The size of the C99 standard bool type seems to vary a lot
// among ABIs. So we use int instead. In C, the result type of relational and
// logical expressions are int.
typedef int MuBool;

// Use uintptr_t for the size of all array parameters.
typedef uintptr_t MuArraySize;

// Watch point ID
typedef uint32_t MuWPID;

// Super type for numerical flags used by Mu.
typedef uint32_t MuFlag;

// Result of a trap handler
typedef MuFlag MuTrapHandlerResult;

// Values or MuTrapHandlerResult
#define MU_THREAD_EXIT          ((MuTrapHandlerResult)0x00)
#define MU_REBIND_PASS_VALUES   ((MuTrapHandlerResult)0x01)
#define MU_REBIND_THROW_EXC     ((MuTrapHandlerResult)0x02)

// Used by MuTrapHandler
typedef void _MuValuesFreer_Func(MuValue *values, MuCPtr freerdata);
typedef _MuValuesFreer_Func* MuValuesFreer;

// Declare the types here because they are used in the following signatures.
typedef struct MuVM MuVM;
typedef struct MuCtx MuCtx;
typedef struct MuIRBuilder MuIRBuilder;

// Signature of the trap handler
typedef void _MuTrapHandler_Func(
        // input parameters
        MuCtx *ctx,
        MuThreadRefValue thread,
        MuStackRefValue stack,
        MuWPID wpid,
        // output parameters
        MuTrapHandlerResult *result,
        MuStackRefValue *new_stack,
        MuValue **values,
        MuArraySize *nvalues,
        MuValuesFreer *freer,
        MuCPtr *freerdata,
        MuRefValue *exception,
        // input parameter (userdata)
        MuCPtr userdata);
typedef _MuTrapHandler_Func* MuTrapHandler;


// Binary operators
typedef MuFlag MuBinOpStatus;
#define MU_BOS_N    ((MuBinOpStatus)0x01)
#define MU_BOS_Z    ((MuBinOpStatus)0x02)
#define MU_BOS_C    ((MuBinOpStatus)0x04)
#define MU_BOS_V    ((MuBinOpStatus)0x08)

typedef MuFlag MuBinOptr;
#define MU_BINOP_ADD    ((MuBinOptr)0x01)
#define MU_BINOP_SUB    ((MuBinOptr)0x02)
#define MU_BINOP_MUL    ((MuBinOptr)0x03)
#define MU_BINOP_SDIV   ((MuBinOptr)0x04)
#define MU_BINOP_SREM   ((MuBinOptr)0x05)
#define MU_BINOP_UDIV   ((MuBinOptr)0x06)
#define MU_BINOP_UREM   ((MuBinOptr)0x07)
#define MU_BINOP_SHL    ((MuBinOptr)0x08)
#define MU_BINOP_LSHR   ((MuBinOptr)0x09)
#define MU_BINOP_ASHR   ((MuBinOptr)0x0A)
#define MU_BINOP_AND    ((MuBinOptr)0x0B)
#define MU_BINOP_OR     ((MuBinOptr)0x0C)
#define MU_BINOP_XOR    ((MuBinOptr)0x0D)
#define MU_BINOP_FADD   ((MuBinOptr)0xB0)
#define MU_BINOP_FSUB   ((MuBinOptr)0xB1)
#define MU_BINOP_FMUL   ((MuBinOptr)0xB2)
#define MU_BINOP_FDIV   ((MuBinOptr)0xB3)
#define MU_BINOP_FREM   ((MuBinOptr)0xB4)

// Comparing operators
typedef MuFlag MuCmpOptr;
#define MU_CMP_EQ       ((MuCmpOptr)0x20)
#define MU_CMP_NE       ((MuCmpOptr)0x21)
#define MU_CMP_SGE      ((MuCmpOptr)0x22)
#define MU_CMP_SGT      ((MuCmpOptr)0x23)
#define MU_CMP_SLE      ((MuCmpOptr)0x24)
#define MU_CMP_SLT      ((MuCmpOptr)0x25)
#define MU_CMP_UGE      ((MuCmpOptr)0x26)
#define MU_CMP_UGT      ((MuCmpOptr)0x27)
#define MU_CMP_ULE      ((MuCmpOptr)0x28)
#define MU_CMP_ULT      ((MuCmpOptr)0x29)
#define MU_CMP_FFALSE   ((MuCmpOptr)0xC0)
#define MU_CMP_FTRUE    ((MuCmpOptr)0xC1)
#define MU_CMP_FUNO     ((MuCmpOptr)0xC2)
#define MU_CMP_FUEQ     ((MuCmpOptr)0xC3)
#define MU_CMP_FUNE     ((MuCmpOptr)0xC4)
#define MU_CMP_FUGT     ((MuCmpOptr)0xC5)
#define MU_CMP_FUGE     ((MuCmpOptr)0xC6)
#define MU_CMP_FULT     ((MuCmpOptr)0xC7)
#define MU_CMP_FULE     ((MuCmpOptr)0xC8)
#define MU_CMP_FORD     ((MuCmpOptr)0xC9)
#define MU_CMP_FOEQ     ((MuCmpOptr)0xCA)
#define MU_CMP_FONE     ((MuCmpOptr)0xCB)
#define MU_CMP_FOGT     ((MuCmpOptr)0xCC)
#define MU_CMP_FOGE     ((MuCmpOptr)0xCD)
#define MU_CMP_FOLT     ((MuCmpOptr)0xCE)
#define MU_CMP_FOLE     ((MuCmpOptr)0xCF)

// Conversion operators
typedef MuFlag MuConvOptr;
#define MU_CONV_TRUNC   ((MuConvOptr)0x30)
#define MU_CONV_ZEXT    ((MuConvOptr)0x31)
#define MU_CONV_SEXT    ((MuConvOptr)0x32)
#define MU_CONV_FPTRUNC ((MuConvOptr)0x33)
#define MU_CONV_FPEXT   ((MuConvOptr)0x34)
#define MU_CONV_FPTOUI  ((MuConvOptr)0x35)
#define MU_CONV_FPTOSI  ((MuConvOptr)0x36)
#define MU_CONV_UITOFP  ((MuConvOptr)0x37)
#define MU_CONV_SITOFP  ((MuConvOptr)0x38)
#define MU_CONV_BITCAST ((MuConvOptr)0x39)
#define MU_CONV_REFCAST ((MuConvOptr)0x3A)
#define MU_CONV_PTRCAST ((MuConvOptr)0x3B)

// Memory orders
typedef MuFlag MuMemOrd;
#define MU_ORD_NOT_ATOMIC   ((MuMemOrd)0x00)
#define MU_ORD_RELAXED      ((MuMemOrd)0x01)
#define MU_ORD_CONSUME      ((MuMemOrd)0x02)
#define MU_ORD_ACQUIRE      ((MuMemOrd)0x03)
#define MU_ORD_RELEASE      ((MuMemOrd)0x04)
#define MU_ORD_ACQ_REL      ((MuMemOrd)0x05)
#define MU_ORD_SEQ_CST      ((MuMemOrd)0x06)

// Operations for the atomicrmw API function
typedef MuFlag MuAtomicRMWOptr;
#define MU_ARMW_XCHG    ((MuAtomicRMWOptr)0x00)
#define MU_ARMW_ADD     ((MuAtomicRMWOptr)0x01)
#define MU_ARMW_SUB     ((MuAtomicRMWOptr)0x02)
#define MU_ARMW_AND     ((MuAtomicRMWOptr)0x03)
#define MU_ARMW_NAND    ((MuAtomicRMWOptr)0x04)
#define MU_ARMW_OR      ((MuAtomicRMWOptr)0x05)
#define MU_ARMW_XOR     ((MuAtomicRMWOptr)0x06)
#define MU_ARMW_MAX     ((MuAtomicRMWOptr)0x07)
#define MU_ARMW_MIN     ((MuAtomicRMWOptr)0x08)
#define MU_ARMW_UMAX    ((MuAtomicRMWOptr)0x09)
#define MU_ARMW_UMIN    ((MuAtomicRMWOptr)0x0A)

// Calling conventions.
typedef MuFlag MuCallConv;
#define MU_CC_DEFAULT   ((MuCallConv)0x00)
// Concrete Mu implementations may define more calling conventions.

// Common instructions.
typedef MuFlag MuCommInst;

// NOTE: MuVM, MuCtx and MuIRBuilder are structures with many function
// pointers. This approach loosens the coupling between the client module and
// the Mu implementation.  At compile time, the client does not need to link
// against any dynamic libraries. At run time, more than one Mu implementations
// can be used by the same client.

// A handle and method lists of a micro VM
struct MuVM {
    void *header;   // Implementation-specific private field

    // Create context
    MuCtx*  (*new_context)(MuVM *mvm);
    
    // Convert between IDs and names. Cannot be used on the bundles being built.
    MuID    (*id_of  )(MuVM *mvm, MuName name);
    MuName  (*name_of)(MuVM *mvm, MuID id);

    // Set handlers
    void    (*set_trap_handler)(MuVM *mvm, MuTrapHandler trap_handler, MuCPtr userdata);

    void    (*compile_to_sharedlib)(MuVM *mvm, MuCString lib_name, MuCString *extra_srcs, MuArraySize n_extra_srcs); /// MUAPIPARSER extra_srcs:array:n_extra_srcs
};

// A local context. It can only be used by one thread at a time. It holds many
// states which are typically held by a Mu thread, such as object references,
// local heap allocation pool, and an object-pinning set. It also holds many Mu
// values and expose them to the client as opaque handles (MuValue and its
// subtypes).
struct MuCtx {
    void *header;   // Implementation-specific private field

    // Convert between IDs and names. Cannot be used on the bundles being built.
    MuID        (*id_of  )(MuCtx *ctx, MuName name);
    MuName      (*name_of)(MuCtx *ctx, MuID id);

    // Close the current context, releasing all resources
    void        (*close_context)(MuCtx *ctx);

    // Load bundles and HAIL scripts
    void        (*load_bundle)(MuCtx *ctx, char *buf, MuArraySize sz); /// MUAPIPARSER buf:array:sz
    void        (*load_hail  )(MuCtx *ctx, char *buf, MuArraySize sz); /// MUAPIPARSER buf:array:sz

    // Convert from C values to Mu values
    MuIntValue      (*handle_from_sint8  )(MuCtx *ctx, int8_t     num, int len);
    MuIntValue      (*handle_from_uint8  )(MuCtx *ctx, uint8_t    num, int len);
    MuIntValue      (*handle_from_sint16 )(MuCtx *ctx, int16_t    num, int len);
    MuIntValue      (*handle_from_uint16 )(MuCtx *ctx, uint16_t   num, int len);
    MuIntValue      (*handle_from_sint32 )(MuCtx *ctx, int32_t    num, int len);
    MuIntValue      (*handle_from_uint32 )(MuCtx *ctx, uint32_t   num, int len);
    MuIntValue      (*handle_from_sint64 )(MuCtx *ctx, int64_t    num, int len);
    MuIntValue      (*handle_from_uint64 )(MuCtx *ctx, uint64_t   num, int len);
    MuIntValue      (*handle_from_uint64s)(MuCtx *ctx, uint64_t *nums, MuArraySize nnums, int len); /// MUAPIPARSER nums:array:nnums
    MuFloatValue    (*handle_from_float  )(MuCtx *ctx, float      num);
    MuDoubleValue   (*handle_from_double )(MuCtx *ctx, double     num);
    MuUPtrValue     (*handle_from_ptr    )(MuCtx *ctx, MuID mu_type, MuCPtr ptr);
    MuUFPValue      (*handle_from_fp     )(MuCtx *ctx, MuID mu_type, MuCFP fp);

    // Convert from Mu values to C values
    int8_t      (*handle_to_sint8 )(MuCtx *ctx, MuIntValue    opnd);
    uint8_t     (*handle_to_uint8 )(MuCtx *ctx, MuIntValue    opnd);
    int16_t     (*handle_to_sint16)(MuCtx *ctx, MuIntValue    opnd);
    uint16_t    (*handle_to_uint16)(MuCtx *ctx, MuIntValue    opnd);
    int32_t     (*handle_to_sint32)(MuCtx *ctx, MuIntValue    opnd);
    uint32_t    (*handle_to_uint32)(MuCtx *ctx, MuIntValue    opnd);
    int64_t     (*handle_to_sint64)(MuCtx *ctx, MuIntValue    opnd);
    uint64_t    (*handle_to_uint64)(MuCtx *ctx, MuIntValue    opnd);
    float       (*handle_to_float )(MuCtx *ctx, MuFloatValue  opnd);
    double      (*handle_to_double)(MuCtx *ctx, MuDoubleValue opnd);
    MuCPtr      (*handle_to_ptr   )(MuCtx *ctx, MuUPtrValue   opnd);
    MuCFP       (*handle_to_fp    )(MuCtx *ctx, MuUFPValue    opnd);

    // Make MuValue instances from Mu global SSA variables
    MuValue         (*handle_from_const )(MuCtx *ctx, MuID id);
    MuIRefValue     (*handle_from_global)(MuCtx *ctx, MuID id);
    MuFuncRefValue  (*handle_from_func  )(MuCtx *ctx, MuID id);
    MuValue         (*handle_from_expose)(MuCtx *ctx, MuID id);

    // Delete the value held by the MuCtx, making it unusable, but freeing up
    // the resource.
    void        (*delete_value)(MuCtx *ctx, MuValue opnd);

    // Compare reference or general reference types.
    // EQ. Available for ref, iref, funcref, threadref, stackref, framecursorref and irbuilderref
    MuBool      (*ref_eq )(MuCtx *ctx, MuGenRefValue lhs, MuGenRefValue rhs);
    // ULT. Available for iref only.
    MuBool      (*ref_ult)(MuCtx *ctx, MuIRefValue lhs, MuIRefValue rhs);

    // Manipulate Mu values of the struct<...> type
    MuValue         (*extract_value)(MuCtx *ctx, MuStructValue str, int index);
    MuStructValue   (*insert_value )(MuCtx *ctx, MuStructValue str, int index, MuValue newval);

    // Manipulate Mu values of the array or vector type
    // str can be MuArrayValue or MuVectorValue
    MuValue     (*extract_element)(MuCtx *ctx, MuSeqValue str, MuIntValue index);
    MuSeqValue  (*insert_element )(MuCtx *ctx, MuSeqValue str, MuIntValue index, MuValue newval);

    // Heap allocation
    MuRefValue  (*new_fixed )(MuCtx *ctx, MuID mu_type);
    MuRefValue  (*new_hybrid)(MuCtx *ctx, MuID mu_type, MuIntValue length);

    // Change the T or sig in ref<T>, iref<T> or func<sig>
    MuGenRefValue   (*refcast)(MuCtx *ctx, MuGenRefValue opnd, MuID new_type);

    // Memory addressing
    MuIRefValue     (*get_iref           )(MuCtx *ctx, MuRefValue opnd);
    MuIRefValue     (*get_field_iref     )(MuCtx *ctx, MuIRefValue opnd, int field);
    MuIRefValue     (*get_elem_iref      )(MuCtx *ctx, MuIRefValue opnd, MuIntValue index);
    MuIRefValue     (*shift_iref         )(MuCtx *ctx, MuIRefValue opnd, MuIntValue offset);
    MuIRefValue     (*get_var_part_iref  )(MuCtx *ctx, MuIRefValue opnd);

    // Memory accessing
    MuValue     (*load     )(MuCtx *ctx, MuMemOrd ord, MuIRefValue loc);
    void        (*store    )(MuCtx *ctx, MuMemOrd ord, MuIRefValue loc, MuValue newval);
    MuValue     (*cmpxchg  )(MuCtx *ctx, MuMemOrd ord_succ, MuMemOrd ord_fail,
                        MuBool weak, MuIRefValue loc, MuValue expected, MuValue desired,
                        MuBool *is_succ); /// MUAPIPARSER is_succ:out
    MuValue     (*atomicrmw)(MuCtx *ctx, MuMemOrd ord, MuAtomicRMWOptr op,
                        MuIRefValue loc, MuValue opnd);
    void        (*fence    )(MuCtx *ctx, MuMemOrd ord);

    // Thread and stack creation and stack destruction
    MuStackRefValue     (*new_stack )(MuCtx *ctx, MuFuncRefValue func);
    MuThreadRefValue    (*new_thread_nor)(MuCtx *ctx, MuStackRefValue stack,
                            MuRefValue threadlocal,
                            MuValue *vals, MuArraySize nvals); /// MUAPIPARSER threadlocal:optional;vals:array:nvals
    MuThreadRefValue    (*new_thread_exc)(MuCtx *ctx, MuStackRefValue stack,
                            MuRefValue threadlocal,
                            MuRefValue exc); /// MUAPIPARSER threadlocal:optional
    void                (*kill_stack)(MuCtx *ctx, MuStackRefValue stack);

    // Thread-local object reference
    void        (*set_threadlocal)(MuCtx *ctx, MuThreadRefValue thread,
                    MuRefValue threadlocal);
    MuRefValue  (*get_threadlocal)(MuCtx *ctx, MuThreadRefValue thread);

    // Frame cursor operations
    MuFCRefValue    (*new_cursor  )(MuCtx *ctx, MuStackRefValue stack);
    void            (*next_frame  )(MuCtx *ctx, MuFCRefValue cursor);
    MuFCRefValue    (*copy_cursor )(MuCtx *ctx, MuFCRefValue cursor);
    void            (*close_cursor)(MuCtx *ctx, MuFCRefValue cursor);

    // Stack introspection
    MuID        (*cur_func       )(MuCtx *ctx, MuFCRefValue cursor);
    MuID        (*cur_func_ver   )(MuCtx *ctx, MuFCRefValue cursor);
    MuID        (*cur_inst       )(MuCtx *ctx, MuFCRefValue cursor);
    void        (*dump_keepalives)(MuCtx *ctx, MuFCRefValue cursor, MuValue *results); /// MUAPIPARSER results:out
    
    // On-stack replacement
    void        (*pop_frames_to)(MuCtx *ctx, MuFCRefValue cursor);
    void        (*push_frame   )(MuCtx *ctx, MuStackRefValue stack, MuFuncRefValue func);

    // 64-bit tagged reference operations
    MuBool          (*tr64_is_fp   )(MuCtx *ctx, MuTagRef64Value value);
    MuBool          (*tr64_is_int  )(MuCtx *ctx, MuTagRef64Value value);
    MuBool          (*tr64_is_ref  )(MuCtx *ctx, MuTagRef64Value value);
    MuDoubleValue   (*tr64_to_fp   )(MuCtx *ctx, MuTagRef64Value value);
    MuIntValue      (*tr64_to_int  )(MuCtx *ctx, MuTagRef64Value value);
    MuRefValue      (*tr64_to_ref  )(MuCtx *ctx, MuTagRef64Value value);
    MuIntValue      (*tr64_to_tag  )(MuCtx *ctx, MuTagRef64Value value);
    MuTagRef64Value (*tr64_from_fp )(MuCtx *ctx, MuDoubleValue value);
    MuTagRef64Value (*tr64_from_int)(MuCtx *ctx, MuIntValue value);
    MuTagRef64Value (*tr64_from_ref)(MuCtx *ctx, MuRefValue ref, MuIntValue tag);

    // Watchpoint operations
    void        (*enable_watchpoint )(MuCtx *ctx, MuWPID wpid);
    void        (*disable_watchpoint)(MuCtx *ctx, MuWPID wpid);

    // Mu memory pinning, usually object pinning
    MuUPtrValue (*pin     )(MuCtx *ctx, MuValue loc);      // loc is either MuRefValue or MuIRefValue
    void        (*unpin   )(MuCtx *ctx, MuValue loc);      // loc is either MuRefValue or MuIRefValue
    MuUPtrValue (*get_addr)(MuCtx *ctx, MuValue loc);      // loc is either MuRefValue or MuIRefValue

    // Expose Mu functions as native callable things, usually function pointers
    MuValue     (*expose  )(MuCtx *ctx, MuFuncRefValue func, MuCallConv call_conv, MuIntValue cookie);
    void        (*unexpose)(MuCtx *ctx, MuCallConv call_conv, MuValue value);

    // Create an IR builder and start building.
    MuIRBuilder*    (*new_ir_builder)(MuCtx *ctx);

    // Build boot image
    void    (*make_boot_image)(MuCtx *ctx,
                MuID* whitelist, MuArraySize whitelist_sz,
                MuFuncRefValue primordial_func, MuStackRefValue primordial_stack,
                MuRefValue primordial_threadlocal,
                MuIRefValue *sym_fields, MuCString *sym_strings, MuArraySize nsyms,
                MuIRefValue *reloc_fields, MuCString *reloc_strings, MuArraySize nrelocs,
                MuCString output_file); /// MUAPIPARSER whitelist:array:whitelist_sz;sym_fields:array:nsyms;sym_strings:array:nsyms;reloc_fields:array:nrelocs;reloc_strings:array:nrelocs;primordial_func:optional;primordial_stack:optional;primordial_threadlocal:optional
};

// These types are all aliases of MuID. They are provided to guide C
// programmers, not for micro VM implementation or type checking.  The C
// programming langauge cannot really check the types.
//
// NOTE for micro VM implementers: it is not necessary to strictly adhere to
// this hierarchy, especially if the implementation uses a functional language
// where polymorphism is difficult to describe.
typedef MuID MuTypeNode;        // Type
typedef MuID MuFuncSigNode;     // Function signature
typedef MuID MuVarNode;         // Variable: constant, global cell, function, expfunc, parameter, instruction result
typedef MuID MuGlobalVarNode;   // Global variable: constant, global cell, function, expfunc
typedef MuID MuLocalVarNode;    // Local variable: parameter, instruction result
typedef MuID MuConstNode;       // Constant (actually a kind of global var)
typedef MuID MuFuncNode;        // Function (actually a kind of global var)
typedef MuID MuFuncVerNode;     // Function version
typedef MuID MuBBNode;          // Basic block
typedef MuID MuInstNode;        // Instruction itself, not its results
typedef MuID MuDestClause;      // Destination clause: branching detination
typedef MuID MuExcClause;       // Exception clause
typedef MuID MuKeepaliveClause; // Keep-alive clause
typedef MuID MuCurStackClause;  // Current stack clause (SWAPSTACK)
typedef MuID MuNewStackClause;  // New stack clause (NEWTHREAD, SWAPSTACK)

// An IR builder. Each MuIRBuilder creates exactly one bundle. It is destroyed
// when the "load" or the "abort" method is called.
//
// Like MuCtx, the client must not use it from multiple client threads without
// synchronisation.  If the client needs to build bundles concurrently, each
// client thread can use a different MuIRBuilder instrance. Usually the client
// would like to load a tiny "initial" bundle that contains common
// types/sigs/constants used by multiple bundles, and then subsequent bundles
// can refer to them concurrently.
struct MuIRBuilder {
    void *header;   // Implementation-specific private field

    // Load the bundle into the micro VM.
    void        (*load  )(MuIRBuilder *b);
    // Cancel bundle building. Release all resources held by the current MuIRBuilder.
    void        (*abort )(MuIRBuilder *b);

    // Generate a symbol. A symbol has a MuID and optionally a name. A symbol
    // can refer to any Mu IR node, such as type, signature, constant, global
    // cell, function, funcver, basic block, parameter, instruction result,
    // branch destination, exception clause, keep-alive clause, etc.
    //
    // It can also refer to top-level nodes in previously loaded bundles.
    //
    // The "name" parameter can be NULL, which means it does not have a name.
    MuID (*gen_sym  )(MuIRBuilder *b, MuCString name);   /// MUAPIPARSER name:optional

    /// Create top-level definitions. When created, they are added to the bundle "b".
    
    // Create types
    void (*new_type_int     )(MuIRBuilder *b, MuID id, int len);
    void (*new_type_float   )(MuIRBuilder *b, MuID id);
    void (*new_type_double  )(MuIRBuilder *b, MuID id);
    void (*new_type_uptr    )(MuIRBuilder *b, MuID id, MuTypeNode ty);
    void (*new_type_ufuncptr)(MuIRBuilder *b, MuID id, MuFuncSigNode sig);

    void (*new_type_struct  )(MuIRBuilder *b, MuID id,
                              MuTypeNode *fieldtys, MuArraySize nfieldtys);
                              /// MUAPIPARSER fieldtys:array:nfieldtys
    void (*new_type_hybrid  )(MuIRBuilder *b, MuID id,
                              MuTypeNode *fixedtys, MuArraySize nfixedtys,
                              MuTypeNode varty);
                              /// MUAPIPARSER fixedtys:array:nfixedtys
    void (*new_type_array   )(MuIRBuilder *b, MuID id, MuTypeNode elemty, uint64_t len);
    void (*new_type_vector  )(MuIRBuilder *b, MuID id, MuTypeNode elemty, uint64_t len);
    void (*new_type_void    )(MuIRBuilder *b, MuID id);

    void (*new_type_ref     )(MuIRBuilder *b, MuID id, MuTypeNode ty);
    void (*new_type_iref    )(MuIRBuilder *b, MuID id, MuTypeNode ty);
    void (*new_type_weakref )(MuIRBuilder *b, MuID id, MuTypeNode ty);
    void (*new_type_funcref )(MuIRBuilder *b, MuID id, MuFuncSigNode sig);
    void (*new_type_tagref64)(MuIRBuilder *b, MuID id);

    void (*new_type_threadref     )(MuIRBuilder *b, MuID id);
    void (*new_type_stackref      )(MuIRBuilder *b, MuID id);
    void (*new_type_framecursorref)(MuIRBuilder *b, MuID id);
    void (*new_type_irbuilderref  )(MuIRBuilder *b, MuID id);

    // Create function signatures
    void (*new_funcsig)(MuIRBuilder *b, MuID id,
                        MuTypeNode *paramtys, MuArraySize nparamtys,
                        MuTypeNode *rettys, MuArraySize nrettys);
                        /// MUAPIPARSER paramtys:array:nparamtys;rettys:array:nrettys

    // Create constants
    // - new_const_int works for int<n> with n <= 64, uptr and ufuncptr.
    // - new_const_int_ex works for int<n> with n > 64. The number is segmented into 64-bit words, lower word first.
    // - new_const_null works for all general reference types, but not uptr or ufuncptr.
    // - new_const_seq works for structs, arrays and vectors.
    // - new_const_extern works for uptr and ufuncptr.
    //
    // TODO: There is only one 'float' type and one 'double' type.
    // Theoretically the 'ty' param for new_const_float/double is unnecessary
    // It is just added to mirror the text form. Eliminate them when we are
    // ready to change the text form.
    void (*new_const_int    )(MuIRBuilder *b, MuID id, MuTypeNode ty, uint64_t value);
    void (*new_const_int_ex )(MuIRBuilder *b, MuID id, MuTypeNode ty, uint64_t *values, MuArraySize nvalues); /// MUAPIPARSER values:array:nvalues
    void (*new_const_float  )(MuIRBuilder *b, MuID id, MuTypeNode ty, float value);
    void (*new_const_double )(MuIRBuilder *b, MuID id, MuTypeNode ty, double value);
    void (*new_const_null   )(MuIRBuilder *b, MuID id, MuTypeNode ty);
    void (*new_const_seq    )(MuIRBuilder *b, MuID id, MuTypeNode ty, MuGlobalVarNode *elems, MuArraySize nelems); /// MUAPIPARSER elems:array:nelems
    void (*new_const_extern )(MuIRBuilder *b, MuID id, MuTypeNode ty, MuCString symbol);

    // Other top-level SSA variables:
    
    // Create global cell
    void (*new_global_cell  )(MuIRBuilder *b, MuID id, MuTypeNode ty);

    // Create function (versions are created separately)
    void (*new_func     )(MuIRBuilder *b, MuID id, MuFuncSigNode sig);

    // Create exposed function
    void (*new_exp_func )(MuIRBuilder *b, MuID id, MuFuncNode func, MuCallConv callconv, MuConstNode cookie);

    /// Create CFG

    // Create function version
    void (*new_func_ver )(MuIRBuilder *b, MuID id,
                          MuFuncNode func,
                          MuBBNode *bbs, MuArraySize nbbs); /// MUAPIPARSER bbs:array:nbbs
    
    // Create basic block. Also create its parameters.
    void (*new_bb)(MuIRBuilder *b, MuID id,
                   MuID *nor_param_ids, MuTypeNode *nor_param_types, MuArraySize n_nor_params,
                   MuID exc_param_id,
                   MuInstNode *insts, MuArraySize ninsts);
                   /// MUAPIPARSER nor_param_ids:array:n_nor_params;nor_param_types:array:n_nor_params;exc_param_id:optional;insts:array:ninsts

    // Create a destination clause.
    void (*new_dest_clause)(MuIRBuilder *b, MuID id,
                            MuBBNode dest,
                            MuVarNode *vars, MuArraySize nvars); /// MUAPIPARSER vars:array:nvars

    // Create an exception clause.
    void (*new_exc_clause)(MuIRBuilder *b, MuID id,
                           MuDestClause nor, MuDestClause exc);

    // Create a keep-alive clause.
    void (*new_keepalive_clause)(MuIRBuilder *b, MuID id,
                                 MuLocalVarNode *vars, MuArraySize nvars); /// MUAPIPARSER vars:array:nvars

    // Create a new "current-stack clause" for the RET_WITH <Ts> case
    void (*new_csc_ret_with)(MuIRBuilder *b, MuID id, MuTypeNode *rettys, MuArraySize nrettys); /// MUAPIPARSER rettys:array:nrettys
    // Create a new "current-stack clause" for the KILL_OLD case
    void (*new_csc_kill_old)(MuIRBuilder *b, MuID id);

    // Create a new "new-stack clause" for the PASS_VALUES <Ts> (Vs) case
    void (*new_nsc_pass_values)(MuIRBuilder *b, MuID id,
                                MuTypeNode *tys, MuVarNode *vars, MuArraySize ntysvars); /// MUAPIPARSER tys:array:ntysvars;vars:array:ntysvars
    // Create a new "new-stack clause" for the THROW_EXC (exc) case
    void (*new_nsc_throw_exc  )(MuIRBuilder *b, MuID id, MuVarNode exc);
    

    /// Create instructions. Also create their results.

    void (*new_binop )(MuIRBuilder *b, MuID id, MuID result_id,
                       MuBinOptr   optr,
                       MuTypeNode  ty,
                       MuVarNode   opnd1,
                       MuVarNode   opnd2,
                       MuExcClause exc_clause);  /// MUAPIPARSER exc_clause:optional
    void (*new_binop_with_status)(MuIRBuilder *b, MuID id, MuID result_id,
                       MuID *status_result_ids, MuArraySize n_status_result_ids,
                       MuBinOptr     optr,
                       MuBinOpStatus status_flags,
                       MuTypeNode    ty,
                       MuVarNode     opnd1,
                       MuVarNode     opnd2,
                       MuExcClause   exc_clause);  /// MUAPIPARSER status_result_ids:array:n_status_result_ids;exc_clause:optional
    void (*new_cmp   )(MuIRBuilder *b, MuID id, MuID result_id,
                       MuCmpOptr  optr,
                       MuTypeNode ty,  
                       MuVarNode  opnd1,
                       MuVarNode  opnd2);
    void (*new_conv  )(MuIRBuilder *b, MuID id, MuID result_id,
                       MuConvOptr optr,
                       MuTypeNode from_ty,
                       MuTypeNode to_ty,
                       MuVarNode  opnd);
    void (*new_select)(MuIRBuilder *b, MuID id, MuID result_id,
                       MuTypeNode cond_ty,
                       MuTypeNode opnd_ty,
                       MuVarNode  cond,
                       MuVarNode  if_true,
                       MuVarNode  if_false);

    void (*new_branch )(MuIRBuilder *b, MuID id, MuDestClause dest);
    void (*new_branch2)(MuIRBuilder *b, MuID id,
                        MuVarNode    cond,
                        MuDestClause if_true,
                        MuDestClause if_false);
    void (*new_switch )(MuIRBuilder *b, MuID id,
                        MuTypeNode   opnd_ty,
                        MuVarNode    opnd,
                        MuDestClause default_dest,
                        MuConstNode  *cases,
                        MuDestClause *dests,
                        MuArraySize  ncasesdests);
                        /// MUAPIPARSER cases:array:ncasesdests;dests:array:ncasesdests

    void (*new_call    )(MuIRBuilder *b, MuID id, MuID *result_ids, MuArraySize n_result_ids,
                         MuFuncSigNode sig,
                         MuVarNode     callee,
                         MuVarNode     *args, MuArraySize nargs,
                         MuExcClause   exc_clause,
                         MuKeepaliveClause keepalive_clause);
                         /// MUAPIPARSER result_ids:array:n_result_ids;args:array:nargs;exc_clause:optional;keepalive_clause:optional
    void (*new_tailcall)(MuIRBuilder *b, MuID id,
                         MuFuncSigNode sig,
                         MuVarNode     callee,
                         MuVarNode     *args, MuArraySize nargs);
                         /// MUAPIPARSER args:array:nargs
    void (*new_ret  )(MuIRBuilder *b, MuID id, MuVarNode *rvs, MuArraySize nrvs); /// MUAPIPARSER rvs:array:nrvs
    void (*new_throw)(MuIRBuilder *b, MuID id, MuVarNode exc);

    void (*new_extractvalue  )(MuIRBuilder *b, MuID id, MuID result_id, MuTypeNode strty, int index,         MuVarNode opnd);
    void (*new_insertvalue   )(MuIRBuilder *b, MuID id, MuID result_id, MuTypeNode strty, int index,         MuVarNode opnd, MuVarNode newval);
    void (*new_extractelement)(MuIRBuilder *b, MuID id, MuID result_id, MuTypeNode seqty, MuTypeNode indty,  MuVarNode opnd, MuVarNode index);
    void (*new_insertelement )(MuIRBuilder *b, MuID id, MuID result_id, MuTypeNode seqty, MuTypeNode indty,  MuVarNode opnd, MuVarNode index, MuVarNode newval);
    void (*new_shufflevector )(MuIRBuilder *b, MuID id, MuID result_id, MuTypeNode vecty, MuTypeNode maskty, MuVarNode vec1, MuVarNode vec2,  MuVarNode mask);

    void (*new_new           )(MuIRBuilder *b, MuID id, MuID result_id,
                               MuTypeNode allocty,
                               MuExcClause exc_clause); /// MUAPIPARSER exc_clause:optional
    void (*new_newhybrid     )(MuIRBuilder *b, MuID id, MuID result_id,
                               MuTypeNode allocty, MuTypeNode lenty, MuVarNode length,
                               MuExcClause exc_clause); /// MUAPIPARSER exc_clause:optional
    void (*new_alloca        )(MuIRBuilder *b, MuID id, MuID result_id,
                               MuTypeNode allocty,
                               MuExcClause exc_clause); /// MUAPIPARSER exc_clause:optional
    void (*new_allocahybrid  )(MuIRBuilder *b, MuID id, MuID result_id,
                               MuTypeNode allocty, MuTypeNode lenty, MuVarNode length,
                               MuExcClause exc_clause); /// MUAPIPARSER exc_clause:optional

    void (*new_getiref       )(MuIRBuilder *b, MuID id, MuID result_id, MuTypeNode refty, MuVarNode opnd);
    void (*new_getfieldiref  )(MuIRBuilder *b, MuID id, MuID result_id, MuBool is_ptr, MuTypeNode refty, int index, MuVarNode opnd);
    void (*new_getelemiref   )(MuIRBuilder *b, MuID id, MuID result_id, MuBool is_ptr, MuTypeNode refty, MuTypeNode indty, MuVarNode opnd, MuVarNode index);
    void (*new_shiftiref     )(MuIRBuilder *b, MuID id, MuID result_id, MuBool is_ptr, MuTypeNode refty, MuTypeNode offty, MuVarNode opnd, MuVarNode offset);
    void (*new_getvarpartiref)(MuIRBuilder *b, MuID id, MuID result_id, MuBool is_ptr, MuTypeNode refty, MuVarNode opnd);

    void (*new_load     )(MuIRBuilder *b, MuID id, MuID result_id,
                          MuBool is_ptr, MuMemOrd ord, MuTypeNode refty, MuVarNode loc,
                          MuExcClause exc_clause); /// MUAPIPARSER exc_clause:optional
    void (*new_store    )(MuIRBuilder *b, MuID id,
                          MuBool is_ptr, MuMemOrd ord, MuTypeNode refty, MuVarNode loc, MuVarNode newval,
                          MuExcClause exc_clause); /// MUAPIPARSER exc_clause:optional
    void (*new_cmpxchg  )(MuIRBuilder *b, MuID id, MuID value_result_id, MuID succ_result_id,
                          MuBool is_ptr, MuBool is_weak, MuMemOrd ord_succ, MuMemOrd ord_fail,
                          MuTypeNode refty, MuVarNode loc, MuVarNode expected, MuVarNode desired,
                          MuExcClause exc_clause); /// MUAPIPARSER exc_clause:optional
    void (*new_atomicrmw)(MuIRBuilder *b, MuID id, MuID result_id,
                          MuBool is_ptr, MuMemOrd ord, MuAtomicRMWOptr optr,
                          MuTypeNode ref_ty, MuVarNode loc, MuVarNode opnd,
                          MuExcClause exc_clause); /// MUAPIPARSER exc_clause:optional
    void (*new_fence    )(MuIRBuilder *b, MuID id, MuMemOrd ord);
    
    void (*new_trap      )(MuIRBuilder *b, MuID id,
                           MuID *result_ids, MuTypeNode *rettys, MuArraySize nretvals,
                           MuExcClause exc_clause, MuKeepaliveClause keepalive_clause);
                           /// MUAPIPARSER result_ids:array:nretvals;rettys:array:nretvals;exc_clause:optional;keepalive_clause:optional
    void (*new_watchpoint)(MuIRBuilder *b, MuID id, MuWPID wpid,
                           MuID *result_ids, MuTypeNode *rettys, MuArraySize nretvals,
                           MuDestClause dis, MuDestClause ena, MuDestClause exc,
                           MuKeepaliveClause keepalive_clause);
                           /// MUAPIPARSER result_ids:array:nretvals;rettys:array:nretvals;exc:optional;keepalive_clause:optional
    void (*new_wpbranch  )(MuIRBuilder *b, MuID id, MuWPID wpid,
                           MuDestClause dis, MuDestClause ena);

    void (*new_ccall)(MuIRBuilder *b, MuID id, MuID *result_ids, MuArraySize n_result_ids,
                      MuCallConv    callconv,
                      MuTypeNode    callee_ty,
                      MuFuncSigNode sig,
                      MuVarNode     callee,
                      MuVarNode     *args, MuArraySize nargs,
                      MuExcClause   exc_clause,
                      MuKeepaliveClause keepalive_clause);
                      /// MUAPIPARSER result_ids:array:n_result_ids;args:array:nargs;exc_clause:optional;keepalive_clause:optional

    void (*new_newthread)(MuIRBuilder *b, MuID id, MuID result_id,
                          MuVarNode        stack,
                          MuVarNode        threadlocal,
                          MuNewStackClause new_stack_clause,
                          MuExcClause      exc_clause); /// MUAPIPARSER threadlocal:optional;exc_clause:optional
    void (*new_swapstack)(MuIRBuilder *b, MuID id, MuID *result_ids, MuArraySize n_result_ids,
                          MuVarNode         swappee,
                          MuCurStackClause  cur_stack_clause,
                          MuNewStackClause  new_stack_clause,
                          MuExcClause       exc_clause,
                          MuKeepaliveClause keepalive_clause); /// MUAPIPARSER result_ids:array:n_result_ids;exc_clause:optional;keepalive_clause:optional

    void (*new_comminst)(MuIRBuilder *b, MuID id, MuID *result_ids, MuArraySize n_result_ids,
                         MuCommInst opcode,
                         MuFlag        *flags, MuArraySize nflags,
                         MuTypeNode    *tys,   MuArraySize ntys,
                         MuFuncSigNode *sigs,  MuArraySize nsigs,
                         MuVarNode     *args,  MuArraySize nargs,
                         MuExcClause       exc_clause,
                         MuKeepaliveClause keepalive_clause); 
                         /// MUAPIPARSER result_ids:array:n_result_ids;flags:array:nflags;tys:array:ntys;sigs:array:nsigs;args:array:nargs;exc_clause:optional;keepalive_clause:optional
};

// Common instruction opcodes
/// GEN:BEGIN:COMMINSTS
#define MU_CI_UVM_NEW_STACK                         ((MuCommInst)0x201) /// MUAPIPARSER muname:@uvm.new_stack
#define MU_CI_UVM_KILL_STACK                        ((MuCommInst)0x202) /// MUAPIPARSER muname:@uvm.kill_stack
#define MU_CI_UVM_THREAD_EXIT                       ((MuCommInst)0x203) /// MUAPIPARSER muname:@uvm.thread_exit
#define MU_CI_UVM_CURRENT_STACK                     ((MuCommInst)0x204) /// MUAPIPARSER muname:@uvm.current_stack
#define MU_CI_UVM_SET_THREADLOCAL                   ((MuCommInst)0x205) /// MUAPIPARSER muname:@uvm.set_threadlocal
#define MU_CI_UVM_GET_THREADLOCAL                   ((MuCommInst)0x206) /// MUAPIPARSER muname:@uvm.get_threadlocal
#define MU_CI_UVM_TR64_IS_FP                        ((MuCommInst)0x211) /// MUAPIPARSER muname:@uvm.tr64.is_fp
#define MU_CI_UVM_TR64_IS_INT                       ((MuCommInst)0x212) /// MUAPIPARSER muname:@uvm.tr64.is_int
#define MU_CI_UVM_TR64_IS_REF                       ((MuCommInst)0x213) /// MUAPIPARSER muname:@uvm.tr64.is_ref
#define MU_CI_UVM_TR64_FROM_FP                      ((MuCommInst)0x214) /// MUAPIPARSER muname:@uvm.tr64.from_fp
#define MU_CI_UVM_TR64_FROM_INT                     ((MuCommInst)0x215) /// MUAPIPARSER muname:@uvm.tr64.from_int
#define MU_CI_UVM_TR64_FROM_REF                     ((MuCommInst)0x216) /// MUAPIPARSER muname:@uvm.tr64.from_ref
#define MU_CI_UVM_TR64_TO_FP                        ((MuCommInst)0x217) /// MUAPIPARSER muname:@uvm.tr64.to_fp
#define MU_CI_UVM_TR64_TO_INT                       ((MuCommInst)0x218) /// MUAPIPARSER muname:@uvm.tr64.to_int
#define MU_CI_UVM_TR64_TO_REF                       ((MuCommInst)0x219) /// MUAPIPARSER muname:@uvm.tr64.to_ref
#define MU_CI_UVM_TR64_TO_TAG                       ((MuCommInst)0x21a) /// MUAPIPARSER muname:@uvm.tr64.to_tag
#define MU_CI_UVM_FUTEX_WAIT                        ((MuCommInst)0x220) /// MUAPIPARSER muname:@uvm.futex.wait
#define MU_CI_UVM_FUTEX_WAIT_TIMEOUT                ((MuCommInst)0x221) /// MUAPIPARSER muname:@uvm.futex.wait_timeout
#define MU_CI_UVM_FUTEX_WAKE                        ((MuCommInst)0x222) /// MUAPIPARSER muname:@uvm.futex.wake
#define MU_CI_UVM_FUTEX_CMP_REQUEUE                 ((MuCommInst)0x223) /// MUAPIPARSER muname:@uvm.futex.cmp_requeue
#define MU_CI_UVM_KILL_DEPENDENCY                   ((MuCommInst)0x230) /// MUAPIPARSER muname:@uvm.kill_dependency
#define MU_CI_UVM_NATIVE_PIN                        ((MuCommInst)0x240) /// MUAPIPARSER muname:@uvm.native.pin
#define MU_CI_UVM_NATIVE_UNPIN                      ((MuCommInst)0x241) /// MUAPIPARSER muname:@uvm.native.unpin
#define MU_CI_UVM_NATIVE_GET_ADDR                   ((MuCommInst)0x242) /// MUAPIPARSER muname:@uvm.native.get_addr
#define MU_CI_UVM_NATIVE_EXPOSE                     ((MuCommInst)0x243) /// MUAPIPARSER muname:@uvm.native.expose
#define MU_CI_UVM_NATIVE_UNEXPOSE                   ((MuCommInst)0x244) /// MUAPIPARSER muname:@uvm.native.unexpose
#define MU_CI_UVM_NATIVE_GET_COOKIE                 ((MuCommInst)0x245) /// MUAPIPARSER muname:@uvm.native.get_cookie
#define MU_CI_UVM_META_ID_OF                        ((MuCommInst)0x250) /// MUAPIPARSER muname:@uvm.meta.id_of
#define MU_CI_UVM_META_NAME_OF                      ((MuCommInst)0x251) /// MUAPIPARSER muname:@uvm.meta.name_of
#define MU_CI_UVM_META_LOAD_BUNDLE                  ((MuCommInst)0x252) /// MUAPIPARSER muname:@uvm.meta.load_bundle
#define MU_CI_UVM_META_LOAD_HAIL                    ((MuCommInst)0x253) /// MUAPIPARSER muname:@uvm.meta.load_hail
#define MU_CI_UVM_META_NEW_CURSOR                   ((MuCommInst)0x254) /// MUAPIPARSER muname:@uvm.meta.new_cursor
#define MU_CI_UVM_META_NEXT_FRAME                   ((MuCommInst)0x255) /// MUAPIPARSER muname:@uvm.meta.next_frame
#define MU_CI_UVM_META_COPY_CURSOR                  ((MuCommInst)0x256) /// MUAPIPARSER muname:@uvm.meta.copy_cursor
#define MU_CI_UVM_META_CLOSE_CURSOR                 ((MuCommInst)0x257) /// MUAPIPARSER muname:@uvm.meta.close_cursor
#define MU_CI_UVM_META_CUR_FUNC                     ((MuCommInst)0x258) /// MUAPIPARSER muname:@uvm.meta.cur_func
#define MU_CI_UVM_META_CUR_FUNC_VER                 ((MuCommInst)0x259) /// MUAPIPARSER muname:@uvm.meta.cur_func_Ver
#define MU_CI_UVM_META_CUR_INST                     ((MuCommInst)0x25a) /// MUAPIPARSER muname:@uvm.meta.cur_inst
#define MU_CI_UVM_META_DUMP_KEEPALIVES              ((MuCommInst)0x25b) /// MUAPIPARSER muname:@uvm.meta.dump_keepalives
#define MU_CI_UVM_META_POP_FRAMES_TO                ((MuCommInst)0x25c) /// MUAPIPARSER muname:@uvm.meta.pop_frames_to
#define MU_CI_UVM_META_PUSH_FRAME                   ((MuCommInst)0x25d) /// MUAPIPARSER muname:@uvm.meta.push_frame
#define MU_CI_UVM_META_ENABLE_WATCHPOINT            ((MuCommInst)0x25e) /// MUAPIPARSER muname:@uvm.meta.enable_watchpoint
#define MU_CI_UVM_META_DISABLE_WATCHPOINT           ((MuCommInst)0x25f) /// MUAPIPARSER muname:@uvm.meta.disable_watchpoint
#define MU_CI_UVM_META_SET_TRAP_HANDLER             ((MuCommInst)0x260) /// MUAPIPARSER muname:@uvm.meta.set_trap_handler
#define MU_CI_UVM_IRBUILDER_NEW_IR_BUILDER          ((MuCommInst)0x270) /// MUAPIPARSER muname:@uvm.irbuilder.new_ir_builder
#define MU_CI_UVM_IRBUILDER_LOAD                    ((MuCommInst)0x300) /// MUAPIPARSER muname:@uvm.irbuilder.load
#define MU_CI_UVM_IRBUILDER_ABORT                   ((MuCommInst)0x301) /// MUAPIPARSER muname:@uvm.irbuilder.abort
#define MU_CI_UVM_IRBUILDER_GEN_SYM                 ((MuCommInst)0x302) /// MUAPIPARSER muname:@uvm.irbuilder.gen_sym
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_INT            ((MuCommInst)0x303) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_int
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_FLOAT          ((MuCommInst)0x304) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_float
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_DOUBLE         ((MuCommInst)0x305) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_double
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_UPTR           ((MuCommInst)0x306) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_uptr
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_UFUNCPTR       ((MuCommInst)0x307) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_ufuncptr
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_STRUCT         ((MuCommInst)0x308) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_struct
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_HYBRID         ((MuCommInst)0x309) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_hybrid
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_ARRAY          ((MuCommInst)0x30a) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_array
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_VECTOR         ((MuCommInst)0x30b) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_vector
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_VOID           ((MuCommInst)0x30c) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_void
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_REF            ((MuCommInst)0x30d) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_ref
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_IREF           ((MuCommInst)0x30e) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_iref
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_WEAKREF        ((MuCommInst)0x30f) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_weakref
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_FUNCREF        ((MuCommInst)0x310) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_funcref
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_TAGREF64       ((MuCommInst)0x311) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_tagref64
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_THREADREF      ((MuCommInst)0x312) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_threadref
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_STACKREF       ((MuCommInst)0x313) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_stackref
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_FRAMECURSORREF ((MuCommInst)0x314) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_framecursorref
#define MU_CI_UVM_IRBUILDER_NEW_TYPE_IRBUILDERREF   ((MuCommInst)0x315) /// MUAPIPARSER muname:@uvm.irbuilder.new_type_irbuilderref
#define MU_CI_UVM_IRBUILDER_NEW_FUNCSIG             ((MuCommInst)0x316) /// MUAPIPARSER muname:@uvm.irbuilder.new_funcsig
#define MU_CI_UVM_IRBUILDER_NEW_CONST_INT           ((MuCommInst)0x317) /// MUAPIPARSER muname:@uvm.irbuilder.new_const_int
#define MU_CI_UVM_IRBUILDER_NEW_CONST_INT_EX        ((MuCommInst)0x318) /// MUAPIPARSER muname:@uvm.irbuilder.new_const_int_ex
#define MU_CI_UVM_IRBUILDER_NEW_CONST_FLOAT         ((MuCommInst)0x319) /// MUAPIPARSER muname:@uvm.irbuilder.new_const_float
#define MU_CI_UVM_IRBUILDER_NEW_CONST_DOUBLE        ((MuCommInst)0x31a) /// MUAPIPARSER muname:@uvm.irbuilder.new_const_double
#define MU_CI_UVM_IRBUILDER_NEW_CONST_NULL          ((MuCommInst)0x31b) /// MUAPIPARSER muname:@uvm.irbuilder.new_const_null
#define MU_CI_UVM_IRBUILDER_NEW_CONST_SEQ           ((MuCommInst)0x31c) /// MUAPIPARSER muname:@uvm.irbuilder.new_const_seq
#define MU_CI_UVM_IRBUILDER_NEW_CONST_EXTERN        ((MuCommInst)0x31d) /// MUAPIPARSER muname:@uvm.irbuilder.new_const_extern
#define MU_CI_UVM_IRBUILDER_NEW_GLOBAL_CELL         ((MuCommInst)0x31e) /// MUAPIPARSER muname:@uvm.irbuilder.new_global_cell
#define MU_CI_UVM_IRBUILDER_NEW_FUNC                ((MuCommInst)0x31f) /// MUAPIPARSER muname:@uvm.irbuilder.new_func
#define MU_CI_UVM_IRBUILDER_NEW_EXP_FUNC            ((MuCommInst)0x320) /// MUAPIPARSER muname:@uvm.irbuilder.new_exp_func
#define MU_CI_UVM_IRBUILDER_NEW_FUNC_VER            ((MuCommInst)0x321) /// MUAPIPARSER muname:@uvm.irbuilder.new_func_ver
#define MU_CI_UVM_IRBUILDER_NEW_BB                  ((MuCommInst)0x322) /// MUAPIPARSER muname:@uvm.irbuilder.new_bb
#define MU_CI_UVM_IRBUILDER_NEW_DEST_CLAUSE         ((MuCommInst)0x323) /// MUAPIPARSER muname:@uvm.irbuilder.new_dest_clause
#define MU_CI_UVM_IRBUILDER_NEW_EXC_CLAUSE          ((MuCommInst)0x324) /// MUAPIPARSER muname:@uvm.irbuilder.new_exc_clause
#define MU_CI_UVM_IRBUILDER_NEW_KEEPALIVE_CLAUSE    ((MuCommInst)0x325) /// MUAPIPARSER muname:@uvm.irbuilder.new_keepalive_clause
#define MU_CI_UVM_IRBUILDER_NEW_CSC_RET_WITH        ((MuCommInst)0x326) /// MUAPIPARSER muname:@uvm.irbuilder.new_csc_ret_with
#define MU_CI_UVM_IRBUILDER_NEW_CSC_KILL_OLD        ((MuCommInst)0x327) /// MUAPIPARSER muname:@uvm.irbuilder.new_csc_kill_old
#define MU_CI_UVM_IRBUILDER_NEW_NSC_PASS_VALUES     ((MuCommInst)0x328) /// MUAPIPARSER muname:@uvm.irbuilder.new_nsc_pass_values
#define MU_CI_UVM_IRBUILDER_NEW_NSC_THROW_EXC       ((MuCommInst)0x329) /// MUAPIPARSER muname:@uvm.irbuilder.new_nsc_throw_exc
#define MU_CI_UVM_IRBUILDER_NEW_BINOP               ((MuCommInst)0x32a) /// MUAPIPARSER muname:@uvm.irbuilder.new_binop
#define MU_CI_UVM_IRBUILDER_NEW_BINOP_WITH_STATUS   ((MuCommInst)0x32b) /// MUAPIPARSER muname:@uvm.irbuilder.new_binop_with_status
#define MU_CI_UVM_IRBUILDER_NEW_CMP                 ((MuCommInst)0x32c) /// MUAPIPARSER muname:@uvm.irbuilder.new_cmp
#define MU_CI_UVM_IRBUILDER_NEW_CONV                ((MuCommInst)0x32d) /// MUAPIPARSER muname:@uvm.irbuilder.new_conv
#define MU_CI_UVM_IRBUILDER_NEW_SELECT              ((MuCommInst)0x32e) /// MUAPIPARSER muname:@uvm.irbuilder.new_select
#define MU_CI_UVM_IRBUILDER_NEW_BRANCH              ((MuCommInst)0x32f) /// MUAPIPARSER muname:@uvm.irbuilder.new_branch
#define MU_CI_UVM_IRBUILDER_NEW_BRANCH2             ((MuCommInst)0x330) /// MUAPIPARSER muname:@uvm.irbuilder.new_branch2
#define MU_CI_UVM_IRBUILDER_NEW_SWITCH              ((MuCommInst)0x331) /// MUAPIPARSER muname:@uvm.irbuilder.new_switch
#define MU_CI_UVM_IRBUILDER_NEW_CALL                ((MuCommInst)0x332) /// MUAPIPARSER muname:@uvm.irbuilder.new_call
#define MU_CI_UVM_IRBUILDER_NEW_TAILCALL            ((MuCommInst)0x333) /// MUAPIPARSER muname:@uvm.irbuilder.new_tailcall
#define MU_CI_UVM_IRBUILDER_NEW_RET                 ((MuCommInst)0x334) /// MUAPIPARSER muname:@uvm.irbuilder.new_ret
#define MU_CI_UVM_IRBUILDER_NEW_THROW               ((MuCommInst)0x335) /// MUAPIPARSER muname:@uvm.irbuilder.new_throw
#define MU_CI_UVM_IRBUILDER_NEW_EXTRACTVALUE        ((MuCommInst)0x336) /// MUAPIPARSER muname:@uvm.irbuilder.new_extractvalue
#define MU_CI_UVM_IRBUILDER_NEW_INSERTVALUE         ((MuCommInst)0x337) /// MUAPIPARSER muname:@uvm.irbuilder.new_insertvalue
#define MU_CI_UVM_IRBUILDER_NEW_EXTRACTELEMENT      ((MuCommInst)0x338) /// MUAPIPARSER muname:@uvm.irbuilder.new_extractelement
#define MU_CI_UVM_IRBUILDER_NEW_INSERTELEMENT       ((MuCommInst)0x339) /// MUAPIPARSER muname:@uvm.irbuilder.new_insertelement
#define MU_CI_UVM_IRBUILDER_NEW_SHUFFLEVECTOR       ((MuCommInst)0x33a) /// MUAPIPARSER muname:@uvm.irbuilder.new_shufflevector
#define MU_CI_UVM_IRBUILDER_NEW_NEW                 ((MuCommInst)0x33b) /// MUAPIPARSER muname:@uvm.irbuilder.new_new
#define MU_CI_UVM_IRBUILDER_NEW_NEWHYBRID           ((MuCommInst)0x33c) /// MUAPIPARSER muname:@uvm.irbuilder.new_newhybrid
#define MU_CI_UVM_IRBUILDER_NEW_ALLOCA              ((MuCommInst)0x33d) /// MUAPIPARSER muname:@uvm.irbuilder.new_alloca
#define MU_CI_UVM_IRBUILDER_NEW_ALLOCAHYBRID        ((MuCommInst)0x33e) /// MUAPIPARSER muname:@uvm.irbuilder.new_allocahybrid
#define MU_CI_UVM_IRBUILDER_NEW_GETIREF             ((MuCommInst)0x33f) /// MUAPIPARSER muname:@uvm.irbuilder.new_getiref
#define MU_CI_UVM_IRBUILDER_NEW_GETFIELDIREF        ((MuCommInst)0x340) /// MUAPIPARSER muname:@uvm.irbuilder.new_getfieldiref
#define MU_CI_UVM_IRBUILDER_NEW_GETELEMIREF         ((MuCommInst)0x341) /// MUAPIPARSER muname:@uvm.irbuilder.new_getelemiref
#define MU_CI_UVM_IRBUILDER_NEW_SHIFTIREF           ((MuCommInst)0x342) /// MUAPIPARSER muname:@uvm.irbuilder.new_shiftiref
#define MU_CI_UVM_IRBUILDER_NEW_GETVARPARTIREF      ((MuCommInst)0x343) /// MUAPIPARSER muname:@uvm.irbuilder.new_getvarpartiref
#define MU_CI_UVM_IRBUILDER_NEW_LOAD                ((MuCommInst)0x344) /// MUAPIPARSER muname:@uvm.irbuilder.new_load
#define MU_CI_UVM_IRBUILDER_NEW_STORE               ((MuCommInst)0x345) /// MUAPIPARSER muname:@uvm.irbuilder.new_store
#define MU_CI_UVM_IRBUILDER_NEW_CMPXCHG             ((MuCommInst)0x346) /// MUAPIPARSER muname:@uvm.irbuilder.new_cmpxchg
#define MU_CI_UVM_IRBUILDER_NEW_ATOMICRMW           ((MuCommInst)0x347) /// MUAPIPARSER muname:@uvm.irbuilder.new_atomicrmw
#define MU_CI_UVM_IRBUILDER_NEW_FENCE               ((MuCommInst)0x348) /// MUAPIPARSER muname:@uvm.irbuilder.new_fence
#define MU_CI_UVM_IRBUILDER_NEW_TRAP                ((MuCommInst)0x349) /// MUAPIPARSER muname:@uvm.irbuilder.new_trap
#define MU_CI_UVM_IRBUILDER_NEW_WATCHPOINT          ((MuCommInst)0x34a) /// MUAPIPARSER muname:@uvm.irbuilder.new_watchpoint
#define MU_CI_UVM_IRBUILDER_NEW_WPBRANCH            ((MuCommInst)0x34b) /// MUAPIPARSER muname:@uvm.irbuilder.new_wpbranch
#define MU_CI_UVM_IRBUILDER_NEW_CCALL               ((MuCommInst)0x34c) /// MUAPIPARSER muname:@uvm.irbuilder.new_ccall
#define MU_CI_UVM_IRBUILDER_NEW_NEWTHREAD           ((MuCommInst)0x34d) /// MUAPIPARSER muname:@uvm.irbuilder.new_newthread
#define MU_CI_UVM_IRBUILDER_NEW_SWAPSTACK           ((MuCommInst)0x34e) /// MUAPIPARSER muname:@uvm.irbuilder.new_swapstack
#define MU_CI_UVM_IRBUILDER_NEW_COMMINST            ((MuCommInst)0x34f) /// MUAPIPARSER muname:@uvm.irbuilder.new_comminst
/// GEN:END:COMMINSTS

#ifdef __cplusplus
}
#endif

#endif // __MUAPI_H__
