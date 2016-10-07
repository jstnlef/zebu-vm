#[derive(Debug)]
enum TypeNode {
    TypeInt { len: usize },
    TypeFloat,
    TypeDouble,
    TypeUPtr { ty: MuID },
    TypeUFuncPtr { sig: MuID },

    TypeStruct { fieldtys: Vec<MuID> },
    TypeHybrid { fixedtys: Vec<MuID>, varty: MuID },
    TypeArray  { elemty: MuID, len: usize },
    TypeVector { elemty: MuID, lem: usize },

    TypeRef     { ty: MuID },
    TypeIRef    { ty: MuID },
    TypeWeakRef { ty: MuID },
    TypeFuncRef { sig: MuID },
    TypeThreadRef,
    TypeStackRef,
    TypeFrameCursorRef,
}

struct FuncSigNode { paramtys: Vec<MuID>, rettys: Vec<MuID> };

enum ConstNode {
    ConstInt    { ty: MuID, value:  usize },
    ConstFloat  { ty: MuID, value:  f32 },
    ConstDouble { ty: MuID, value:  f64 },
    ConstNull   { ty: MuID },
    ConstSeq    { ty: MuID, elems: Vec<MuID> },
    ConstExtern { ty: MuID, symbol: String },
}

struct GlobalCellNode { ty: MuID };

struct FuncNode { sig: MuID },
