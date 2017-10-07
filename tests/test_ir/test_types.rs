// Copyright 2017 The Australian National University
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate mu;

use self::mu::ast::ir::MuEntityHeader;
use self::mu::ast::ptr::*;
use self::mu::ast::types::*;
use std::sync::Arc;

macro_rules! assert_type (
    ($test:expr, $expect: expr) => (
        assert_eq!(format!("{}", $test), $expect)
    )
);

/// create one of each MuType
fn create_types() -> Vec<P<MuType>> {
    let mut types = vec![];

    let t0 = MuType::new(0, MuType_::int(8));
    types.push(P(t0));

    let t1 = MuType::new(1, MuType_::float());
    types.push(P(t1));

    let t2 = MuType::new(2, MuType_::double());
    types.push(P(t2));

    let t3 = MuType::new(3, MuType_::muref(types[0].clone()));
    types.push(P(t3));

    let t4 = MuType::new(4, MuType_::iref(types[0].clone()));
    types.push(P(t4));

    let t5 = MuType::new(5, MuType_::weakref(types[0].clone()));
    types.push(P(t5));

    let t6 = MuType::new(6, MuType_::uptr(types[0].clone()));
    types.push(P(t6));

    let t7 = MuType::new(
        7,
        MuType_::mustruct(
            Arc::new("MyStructTag1".to_string()),
            vec![types[0].clone(), types[1].clone()]
        )
    );
    types.push(P(t7));

    let t8 = MuType::new(8, MuType_::array(types[0].clone(), 5));
    types.push(P(t8));

    let t9 = MuType::new(
        9,
        MuType_::hybrid(
            Arc::new("MyHybridTag1".to_string()),
            vec![types[7].clone(), types[1].clone()],
            types[0].clone()
        )
    );
    types.push(P(t9));

    let t10 = MuType::new(10, MuType_::void());
    types.push(P(t10));

    let t11 = MuType::new(11, MuType_::threadref());
    types.push(P(t11));

    let t12 = MuType::new(12, MuType_::stackref());
    types.push(P(t12));

    let t13 = MuType::new(13, MuType_::tagref64());
    types.push(P(t13));

    let t14 = MuType::new(14, MuType_::vector(types[0].clone(), 5));
    types.push(P(t14));

    let sig = P(MuFuncSig {
        hdr: MuEntityHeader::unnamed(20),
        ret_tys: vec![types[10].clone()],
        arg_tys: vec![types[0].clone(), types[0].clone()]
    });

    let t15 = MuType::new(15, MuType_::funcref(sig.clone()));
    types.push(P(t15));

    let t16 = MuType::new(16, MuType_::ufuncptr(sig.clone()));
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
    assert_type!(*types[7], "MyStructTag1");
    {
        let map = STRUCT_TAG_MAP.read().unwrap();
        let t7_struct_ty = map.get(&"MyStructTag1".to_string()).unwrap();
        assert_type!(t7_struct_ty, "struct<int<8> float>");
    }
    assert_type!(*types[8], "array<int<8> 5>");
    assert_type!(*types[9], "MyHybridTag1");
    assert_type!(*types[10], "void");
    assert_type!(*types[11], "threadref");
    assert_type!(*types[12], "stackref");
    assert_type!(*types[13], "tagref64");
    assert_type!(*types[14], "vector<int<8> 5>");
    assert_type!(*types[15], "funcref<(int<8> int<8>)->(void)>");
    assert_type!(*types[16], "ufuncref<(int<8> int<8>)->(void)>");
}

#[test]
fn test_cyclic_struct() {
    // .typedef @cyclic_struct_ty = struct<ref<@cyclic_struct_ty> int<32>>
    let ty = P(MuType::new(
        0,
        MuType_::mustruct_empty(Arc::new("MyStructTag2".to_string()))
    ));
    let ref_ty = P(MuType::new(1, MuType_::muref(ty.clone())));
    let i32_ty = P(MuType::new(2, MuType_::int(32)));

    {
        STRUCT_TAG_MAP
            .write()
            .unwrap()
            .get_mut(&"MyStructTag2".to_string())
            .unwrap()
            .set_tys(vec![ref_ty.clone(), i32_ty.clone()]);
    }

    let map = STRUCT_TAG_MAP.read().unwrap();
    let struct_ty = map.get(&"MyStructTag2".to_string()).unwrap();
    assert_type!(struct_ty, "struct<ref<MyStructTag2> int<32>>");
}

#[test]
fn test_is_traced() {
    let types = create_types();

    assert_eq!(types[0].is_traced(), false);
    assert_eq!(types[1].is_traced(), false);
    assert_eq!(types[2].is_traced(), false);
    assert_eq!(types[3].is_traced(), true);
    assert_eq!(types[4].is_traced(), true);
    assert_eq!(types[5].is_traced(), true);
    assert_eq!(types[6].is_traced(), false);
    assert_eq!(types[7].is_traced(), false);
    let struct3 = MuType::new(
        100,
        MuType_::mustruct(
            Arc::new("MyStructTag3".to_string()),
            vec![types[3].clone(), types[0].clone()]
        )
    );
    assert_eq!(struct3.is_traced(), true);
    let struct4 = MuType::new(
        101,
        MuType_::mustruct(
            Arc::new("MyStructTag4".to_string()),
            vec![types[3].clone(), types[4].clone()]
        )
    );
    assert_eq!(struct4.is_traced(), true);
    assert_eq!(types[8].is_traced(), false);
    let ref_array = MuType::new(102, MuType_::array(types[3].clone(), 5));
    assert_eq!(ref_array.is_traced(), true);
    assert_eq!(types[9].is_traced(), false);
    let fix_ref_hybrid = MuType::new(
        103,
        MuType_::hybrid(
            Arc::new("FixRefHybrid".to_string()),
            vec![types[3].clone(), types[0].clone()],
            types[0].clone()
        )
    );
    assert_eq!(fix_ref_hybrid.is_traced(), true);
    let var_ref_hybrid = MuType::new(
        104,
        MuType_::hybrid(
            Arc::new("VarRefHybrid".to_string()),
            vec![types[0].clone(), types[1].clone()],
            types[3].clone()
        )
    );
    assert_eq!(var_ref_hybrid.is_traced(), true);
    assert_eq!(types[10].is_traced(), false);
    assert_eq!(types[11].is_traced(), true);
    assert_eq!(types[12].is_traced(), true);
    assert_eq!(types[13].is_traced(), true);
    assert_eq!(types[14].is_traced(), false);
    assert_eq!(types[15].is_traced(), false);
    assert_eq!(types[16].is_traced(), false);
}

#[test]
fn test_is_native_safe() {
    let types = create_types();

    assert_eq!(types[0].is_native_safe(), true);
    assert_eq!(types[1].is_native_safe(), true);
    assert_eq!(types[2].is_native_safe(), true);
    assert_eq!(types[3].is_native_safe(), false);
    assert_eq!(types[4].is_native_safe(), false);
    assert_eq!(types[5].is_native_safe(), false);
    assert_eq!(types[6].is_native_safe(), true);
    assert_eq!(types[7].is_native_safe(), true);
    let struct3 = MuType::new(
        100,
        MuType_::mustruct(
            Arc::new("MyStructTag3".to_string()),
            vec![types[3].clone(), types[0].clone()]
        )
    );
    assert_eq!(struct3.is_native_safe(), false);
    let struct4 = MuType::new(
        101,
        MuType_::mustruct(
            Arc::new("MyStructTag4".to_string()),
            vec![types[3].clone(), types[4].clone()]
        )
    );
    assert_eq!(struct4.is_native_safe(), false);
    assert_eq!(types[8].is_native_safe(), true);
    let ref_array = MuType::new(102, MuType_::array(types[3].clone(), 5));
    assert_eq!(ref_array.is_native_safe(), false);
    assert_eq!(types[9].is_native_safe(), true);
    let fix_ref_hybrid = MuType::new(
        103,
        MuType_::hybrid(
            Arc::new("FixRefHybrid".to_string()),
            vec![types[3].clone(), types[0].clone()],
            types[0].clone()
        )
    );
    assert_eq!(fix_ref_hybrid.is_native_safe(), false);
    let var_ref_hybrid = MuType::new(
        104,
        MuType_::hybrid(
            Arc::new("VarRefHybrid".to_string()),
            vec![types[0].clone(), types[1].clone()],
            types[3].clone()
        )
    );
    assert_eq!(var_ref_hybrid.is_native_safe(), false);
    assert_eq!(types[10].is_native_safe(), true);
    assert_eq!(types[11].is_native_safe(), false);
    assert_eq!(types[12].is_native_safe(), false);
    assert_eq!(types[13].is_native_safe(), false);
    assert_eq!(types[14].is_native_safe(), true);
    assert_eq!(types[15].is_native_safe(), false); // funcref is not native safe
    // and not traced either
    assert_eq!(types[16].is_native_safe(), true);
}
