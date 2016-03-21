extern crate mu;

use self::mu::ast::ptr::*;
use self::mu::ast::types::*;

macro_rules! assert_type (
    ($test:expr, $expect: expr) => (
        assert_eq!(format!("{:?}", $test), $expect)
    )
);

macro_rules! println_type (
    ($test:expr) => (
        println!("{:?}", $test)
    )  
);

/// create one of each MuType
fn create_types() -> Vec<P<MuType>> {
    let mut types = vec![];
    
    let t0 = MuType::int(8);
    types.push(P(t0));
    
    let t1 = MuType::float();
    types.push(P(t1));
    
    let t2 = MuType::double();
    types.push(P(t2));
    
    let t3 = MuType::muref(types[0].clone());
    types.push(P(t3));
    
    let t4 = MuType::iref(types[0].clone());
    types.push(P(t4));
    
    let t5 = MuType::weakref(types[0].clone());
    types.push(P(t5));
    
    let t6 = MuType::uptr(types[0].clone());
    types.push(P(t6));
    
    let t7 = MuType::mustruct("MyStructTag1", vec![types[0].clone(), types[1].clone()]);
    types.push(P(t7));
    
    let t8 = MuType::array(types[0].clone(), 5);
    types.push(P(t8));
    
    let t9 = MuType::hybrid(vec![types[7].clone(), types[1].clone()], types[0].clone());
    types.push(P(t9));
    
    let t10 = MuType::void();
    types.push(P(t10));
    
    let t11 = MuType::threadref();
    types.push(P(t11));
    
    let t12 = MuType::stackref();
    types.push(P(t12));
    
    let t13 = MuType::tagref64();
    types.push(P(t13));
    
    let t14 = MuType::vector(types[0].clone(), 5);
    types.push(P(t14));
    
    let sig = P(MuFuncSig{ret_tys: vec![types[10].clone()], arg_tys: vec![types[0].clone(), types[0].clone()]});
    
    let t15 = MuType::funcref(sig.clone());
    types.push(P(t15));
    
    let t16 = MuType::ufuncptr(sig.clone());
    types.push(P(t16));
    
    types
}

#[test]
#[allow(unused_variables)]
fn test_type_constructors() {
    let types = create_types();
    
    assert_type!(*types[0], "int<8>");
    assert_type!(*types[1], "float");
    assert_type!(*types[2], "double");
    assert_type!(*types[3], "ref<int<8>>");
    assert_type!(*types[4], "iref<int<8>>");
    assert_type!(*types[5], "weakref<int<8>>");
    assert_type!(*types[6], "uptr<int<8>>");
    assert_type!(*types[7], "MyStructTag1(struct)");
    {
        let map = STRUCT_TAG_MAP.read().unwrap();
        let t7_struct_ty = map.get("MyStructTag1").unwrap();
        assert_type!(t7_struct_ty, "struct<int<8> float>");
    }
    assert_type!(*types[8], "array<int<8> 5>");
    assert_type!(*types[9], "hybrid<[MyStructTag1(struct), float] int<8>>");
    assert_type!(*types[10], "void");
    assert_type!(*types[11], "threadref");
    assert_type!(*types[12], "stackref");
    assert_type!(*types[13], "tagref64");
    assert_type!(*types[14], "vector<int<8> 5>");
    assert_type!(*types[15], "funcref<[void] -> [int<8>, int<8>]>");
    assert_type!(*types[16], "ufuncref<[void] -> [int<8>, int<8>]>");
}

#[test]
fn test_cyclic_struct() {
    // .typedef @cyclic_struct_ty = struct<ref<@cyclic_struct_ty> int<32>>
    let ty = P(MuType::mustruct_empty("MyStructTag2"));
    let ref_ty = P(MuType::muref(ty.clone()));
    let i32_ty = P(MuType::int(32));
    
    {
        STRUCT_TAG_MAP.write().unwrap().
            get_mut("MyStructTag2").unwrap().set_tys(vec![ref_ty.clone(), i32_ty.clone()]);
    }
    
    let map = STRUCT_TAG_MAP.read().unwrap();
    let struct_ty = map.get("MyStructTag2").unwrap();
    assert_type!(struct_ty, "struct<ref<MyStructTag2(struct)> int<32>>");
}

#[test]
fn test_is_traced() {
    let types = create_types();
    
    assert_eq!(is_traced(&types[0]), false);
    assert_eq!(is_traced(&types[1]), false);
    assert_eq!(is_traced(&types[2]), false);
    assert_eq!(is_traced(&types[3]), true);
    assert_eq!(is_traced(&types[4]), true);
    assert_eq!(is_traced(&types[5]), true);
    assert_eq!(is_traced(&types[6]), false);
    assert_eq!(is_traced(&types[7]), false);
    let struct3 = MuType::mustruct("MyStructTag3", vec![types[3].clone(), types[0].clone()]);
    assert_eq!(is_traced(&struct3), true);
    let struct4 = MuType::mustruct("MyStructTag4", vec![types[3].clone(), types[4].clone()]);
    assert_eq!(is_traced(&struct4), true);
    assert_eq!(is_traced(&types[8]), false);
    let ref_array = MuType::array(types[3].clone(), 5);
    assert_eq!(is_traced(&ref_array), true);
    assert_eq!(is_traced(&types[9]), false);
    let fix_ref_hybrid = MuType::hybrid(vec![types[3].clone(), types[0].clone()], types[0].clone());
    assert_eq!(is_traced(&fix_ref_hybrid), true);
    let var_ref_hybrid = MuType::hybrid(vec![types[0].clone(), types[1].clone()], types[3].clone());
    assert_eq!(is_traced(&var_ref_hybrid), true);
    assert_eq!(is_traced(&types[10]), false);
    assert_eq!(is_traced(&types[11]), true);
    assert_eq!(is_traced(&types[12]), true);
    assert_eq!(is_traced(&types[13]), true);
    assert_eq!(is_traced(&types[14]), false);
    assert_eq!(is_traced(&types[15]), false);
    assert_eq!(is_traced(&types[16]), false);
}

#[test]
fn test_is_native_safe() {
    let types = create_types();    
    
    assert_eq!(is_native_safe(&types[0]), true);
    assert_eq!(is_native_safe(&types[1]), true);
    assert_eq!(is_native_safe(&types[2]), true);
    assert_eq!(is_native_safe(&types[3]), false);
    assert_eq!(is_native_safe(&types[4]), false);
    assert_eq!(is_native_safe(&types[5]), false);
    assert_eq!(is_native_safe(&types[6]), true);
    assert_eq!(is_native_safe(&types[7]), true);
    let struct3 = MuType::mustruct("MyStructTag3", vec![types[3].clone(), types[0].clone()]);
    assert_eq!(is_native_safe(&struct3), false);
    let struct4 = MuType::mustruct("MyStructTag4", vec![types[3].clone(), types[4].clone()]);
    assert_eq!(is_native_safe(&struct4), false);
    assert_eq!(is_native_safe(&types[8]), true);
    let ref_array = MuType::array(types[3].clone(), 5);
    assert_eq!(is_native_safe(&ref_array), false);
    assert_eq!(is_native_safe(&types[9]), true);
    let fix_ref_hybrid = MuType::hybrid(vec![types[3].clone(), types[0].clone()], types[0].clone());
    assert_eq!(is_native_safe(&fix_ref_hybrid), false);
    let var_ref_hybrid = MuType::hybrid(vec![types[0].clone(), types[1].clone()], types[3].clone());
    assert_eq!(is_native_safe(&var_ref_hybrid), false);
    assert_eq!(is_native_safe(&types[10]), true);
    assert_eq!(is_native_safe(&types[11]), false);
    assert_eq!(is_native_safe(&types[12]), false);
    assert_eq!(is_native_safe(&types[13]), false);
    assert_eq!(is_native_safe(&types[14]), true);
    assert_eq!(is_native_safe(&types[15]), false);    // funcref is not native safe
                                                      // and not traced either
    assert_eq!(is_native_safe(&types[16]), true);
}