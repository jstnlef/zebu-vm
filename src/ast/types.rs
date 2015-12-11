extern crate std;

pub type MuID = usize;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MuType {
    /// int <length>
    MuInt          (usize),
    /// float
    MuFloat,
    /// double
    MuDouble,
    
    /// ref<T>
    MuRef          (Box<MuType>),    // Box is needed for non-recursive enum
    /// iref<T>: internal reference
    MuIRef         (Box<MuType>),
    /// weakref<T>
    MuWeakRef      (Box<MuType>),
    
    /// uptr<T>: unsafe pointer
    MuUPtr         (Box<MuType>),
    
    /// struct<T1 T2 ...>
    MuStruct       (MuID),
    
    /// array<T length>
    MuArray        (Box<MuType>, usize),
    
    /// hybrid<F1 F2 ... V>: a hybrid of fixed length parts and a variable length part
    MuHybrid       (Vec<Box<MuType>>, Box<MuType>),
    
    /// void
    MuVoid,
    
    /// threadref
    MuThreadRef,
    /// stackref
    MuStackRef,
    
    /// tagref64: hold a double or an int or an ref<void>
    MuTagref64,
    
    /// vector<T length>
    MuVector       (Box<MuType>, usize),
    
    /// funcref<@sig>
    MuFuncRef      (MuFuncSig),
    
    /// ufuncptr<@sig>
    MuUFuncPtr     (MuFuncSig),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MuStructType_ {
    tys: Vec<Box<MuType>>
}

impl MuStructType_ {
    pub fn set_tys(&mut self, list: Vec<&MuType>) {
        self.tys.append(&mut list.into_iter().map(|t| Box::new(t.clone())).collect());
    }
}

impl MuType {
    // constructors
    pub fn int(len: usize) -> MuType {
        MuType::MuInt(len)
    }
    pub fn float() -> MuType {
        MuType::MuFloat
    }
    pub fn double() -> MuType {
        MuType::MuDouble
    }
    pub fn muref(referent: &MuType) -> MuType {
        MuType::MuRef(Box::new(referent.clone()))
    }
    pub fn muref_void() -> MuType {
        MuType::MuRef(Box::new(MuType::void()))
    }
    pub fn iref(referent: &MuType) -> MuType {
        MuType::MuIRef(Box::new(referent.clone()))
    }
    pub fn weakref(referent: &MuType) -> MuType {
        MuType::MuWeakRef(Box::new(referent.clone()))
    }
    pub fn uptr(referent: &MuType) -> MuType {
        MuType::MuUPtr(Box::new(referent.clone()))
    }
    pub fn mustruct_empty(id: MuID) -> (MuType, MuStructType_) {
        (MuType::MuStruct(id), MuStructType_{tys: vec![]})
    }
    pub fn mustruct(id: MuID, list: Vec<&MuType>) -> (MuType, MuStructType_) {
        (MuType::MuStruct(id), MuStructType_{tys: list.into_iter().map(|t| Box::new(t.clone())).collect()})  
    }
    pub fn array(ty: &MuType, len: usize) -> MuType {
        MuType::MuArray(Box::new(ty.clone()), len)
    }
    pub fn hybrid(fix_tys: Vec<&MuType>, var_tys: &MuType) -> MuType {
        MuType::MuHybrid(
            fix_tys.into_iter().map(|t| Box::new(t.clone())).collect(),
            Box::new(var_tys.clone())
        )
    }
    pub fn void() -> MuType {
        MuType::MuVoid
    }
    pub fn threadref() -> MuType {
        MuType::MuThreadRef
    }
    pub fn stackref() -> MuType {
        MuType::MuStackRef
    }
    pub fn tagref64() -> MuType {
        MuType::MuTagref64
    }
    pub fn vector(ty: &MuType, len: usize) -> MuType {
        MuType::MuVector(Box::new(ty.clone()), len)
    }
    pub fn funcref(sig: MuFuncSig) -> MuType {
        MuType::MuFuncRef(sig)
    }
    pub fn ufuncptr(sig: MuFuncSig) -> MuType {
        MuType::MuUFuncPtr(sig)
    }
}

macro_rules! is_type (
    ($e:expr, $p:pat) => (
        match $e {
            $p => true,
            _ => false
        }
    )
);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MuFuncSig {
    ret_tys : Vec<MuType>,
    args_tys: Vec<MuType>
}

impl MuFuncSig {
    pub fn new(ret_tys: Vec<&MuType>, args_tys: Vec<&MuType>) -> MuFuncSig {
        MuFuncSig {
            ret_tys : ret_tys.into_iter().map(|t| t.clone()).collect(),
            args_tys: args_tys.into_iter().map(|t| t.clone()).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
    
    #[test]
    #[allow(unused_variables)]
    fn test_type_constructors() {
        let mut types = vec![];
        
        let t0 = MuType::int(8);
        types.push(Box::new(t0));
        assert_type!(*types[0], "MuInt(8)");
        
        let t1 = MuType::float();
        types.push(Box::new(t1));
        assert_type!(*types[1], "MuFloat");
        
        let t2 = MuType::double();
        types.push(Box::new(t2));
        assert_type!(*types[2], "MuDouble");
        
        let t3 = MuType::muref(&types[0]);
        types.push(Box::new(t3));
        assert_type!(*types[3], "MuRef(MuInt(8))");
        
        let t4 = MuType::iref(&types[0]);
        types.push(Box::new(t4));
        assert_type!(*types[4], "MuIRef(MuInt(8))");
        
        let t5 = MuType::weakref(&types[0]);
        types.push(Box::new(t5));
        assert_type!(*types[5], "MuWeakRef(MuInt(8))");
        
        let t6 = MuType::uptr(&types[0]);
        types.push(Box::new(t6));
        assert_type!(*types[6], "MuUPtr(MuInt(8))");
        
        let (t7, t7_struct) = MuType::mustruct(0, vec![&types[0], &types[1]]);
        types.push(Box::new(t7));
        assert_type!(*types[7], "MuStruct(0)");
        assert_type!(t7_struct, "MuStructType_ { tys: [MuInt(8), MuFloat] }");
        
        let t8 = MuType::array(&types[0], 5);
        types.push(Box::new(t8));
        assert_type!(*types[8], "MuArray(MuInt(8), 5)");
        
        let t9 = MuType::hybrid(vec![&types[7], &types[1]], &types[0]);
        types.push(Box::new(t9));
        assert_type!(*types[9], "MuHybrid([MuStruct(0), MuFloat], MuInt(8))");
        
        let t10 = MuType::void();
        types.push(Box::new(t10));
        assert_type!(*types[10], "MuVoid");
        
        let t11 = MuType::threadref();
        types.push(Box::new(t11));
        assert_type!(*types[11], "MuThreadRef");
        
        let t12 = MuType::stackref();
        types.push(Box::new(t12));
        assert_type!(*types[12], "MuStackRef");
        
        let t13 = MuType::tagref64();
        types.push(Box::new(t13));
        assert_type!(*types[13], "MuTagref64");
        
        let t14 = MuType::vector(&types[0], 5);
        types.push(Box::new(t14));
        assert_type!(*types[14], "MuVector(MuInt(8), 5)");
        
        let sig = MuFuncSig::new(vec![&types[10]], vec![&types[0], &types[0]]);
        
        let t15 = MuType::funcref(sig.clone());
        types.push(Box::new(t15));
        assert_type!(*types[15], "MuFuncRef(MuFuncSig { ret_tys: [MuVoid], args_tys: [MuInt(8), MuInt(8)] })");
        
        let t16 = MuType::ufuncptr(sig.clone());
        types.push(Box::new(t16));
        assert_type!(*types[16], "MuUFuncPtr(MuFuncSig { ret_tys: [MuVoid], args_tys: [MuInt(8), MuInt(8)] })");
    }
    
    #[test]
    fn test_cyclic_struct() {
        // .typedef @cyclic_struct_ty = struct<ref<@cyclic_struct_ty> int<32>>
        let (ty, mut struct_ty) = MuType::mustruct_empty(0);
        let ref_ty = MuType::muref(&ty);
        let i32_ty = MuType::int(32);
        struct_ty.set_tys(vec![&ref_ty, &i32_ty]);
        
        assert_type!(struct_ty, "MuStructType_ { tys: [MuRef(MuStruct(0)), MuInt(32)] }");
    }
}