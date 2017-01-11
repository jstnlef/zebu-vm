#![allow(dead_code)]

use std::sync::Arc;
use utils::POINTER_SIZE;
use utils::ByteSize;
use objectmodel;

use std::u32;
pub const GCTYPE_INIT_ID: u32 = u32::MAX;

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct GCType {
    pub id: u32,
    pub alignment: ByteSize,

    pub fix_size: ByteSize,
    pub fix_refs: Option<RefPattern>,

    pub var_refs: Option<RefPattern>,
    pub var_size: Option<ByteSize>
}

impl GCType {
    pub fn new_fix(id: u32, size: ByteSize, alignment: ByteSize, fix_refs: Option<RefPattern>) -> GCType {
        GCType {
            id: id,
            alignment: objectmodel::check_alignment(alignment),

            fix_refs: fix_refs,
            fix_size: size,

            var_refs: None,
            var_size: None
        }
    }

    pub fn new_hybrid(id: u32, size: ByteSize, alignment: ByteSize, fix_refs: Option<RefPattern>, var_refs: Option<RefPattern>, var_size: ByteSize) -> GCType {
        GCType {
            id: id,
            alignment: objectmodel::check_alignment(alignment),

            fix_refs: fix_refs,
            fix_size: size,

            var_refs: var_refs,
            var_size: Some(var_size)
        }
    }

    pub fn new_noreftype(size: ByteSize, align: ByteSize) -> GCType {
        GCType {
            id: GCTYPE_INIT_ID,
            alignment: align,

            fix_refs: None,
            fix_size: size,

            var_refs: None,
            var_size: None,
        }
    }

    pub fn new_reftype() -> GCType {
        GCType {
            id: GCTYPE_INIT_ID,
            alignment: POINTER_SIZE,

            fix_refs: Some(RefPattern::Map{
                offsets: vec![0],
                size: POINTER_SIZE
            }),
            fix_size: POINTER_SIZE,

            var_refs: None,
            var_size: None
        }
    }

    #[inline(always)]
    pub fn is_hybrid(&self) -> bool {
        self.var_size.is_some()
    }

    pub fn size(&self) -> ByteSize {
        self.fix_size
    }

    pub fn size_hybrid(&self, length: u32) -> ByteSize {
        assert!(self.var_size.is_some());

        self.fix_size + self.var_size.unwrap() * (length as usize)
    }

    #[allow(unused_assignments)]
    pub fn gen_ref_offsets(&self) -> Vec<ByteSize> {
        let mut ret = vec![];

        let mut cur_offset = 0;

        match self.fix_refs {
            Some(ref pattern) => {
                cur_offset = pattern.append_offsets(cur_offset, &mut ret);
            }
            None => {}
        }

        ret
    }

    pub fn gen_hybrid_ref_offsets(&self, length: u32) -> Vec<ByteSize> {
        debug_assert!(self.is_hybrid());

        let mut ret = vec![];

        let mut cur_offset = 0;

        // fix part
        match self.fix_refs {
            Some(ref pattern) => {
                cur_offset = pattern.append_offsets(cur_offset, &mut ret);
            },
            None => {}
        }

        // var part
        if self.var_refs.is_some() {
            let ref var_part = self.var_refs.as_ref().unwrap();
            for _ in 0..length {
                cur_offset = var_part.append_offsets(cur_offset, &mut ret);
            }
        }

        ret
    }
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum RefPattern {
    Map{
        offsets: Vec<ByteSize>,
        size : usize
    },
    NestedType(Vec<Arc<GCType>>),
    Repeat{
        pattern: Box<RefPattern>,
        count: usize
    }
}

impl RefPattern {
    pub fn size(&self) -> ByteSize {
        match self {
            &RefPattern::Map {size, ..} => size,
            &RefPattern::NestedType(ref vec) => {
                let mut size = 0;
                for ty in vec.iter() {
                    size += ty.size();
                }
                size
            },
            &RefPattern::Repeat{ref pattern, count} => {
                pattern.size() * count
            }
        }
    }

    pub fn append_offsets(&self, base: ByteSize, vec: &mut Vec<ByteSize>) -> ByteSize {
        match self {
            &RefPattern::Map{ref offsets, size} => {
                for off in offsets {
                    vec.push(base + off);
                }

                base + size
            }
            &RefPattern::NestedType(ref types) => {
                let mut cur_base = base;

                for ty in types {
                    let nested_offset = ty.gen_ref_offsets();
                    let mut nested_offset = nested_offset.iter().map(|x| x + cur_base).collect();

                    vec.append(&mut nested_offset);

                    cur_base += ty.size();
                }

                cur_base
            },
            &RefPattern::Repeat{ref pattern, count} => {
                let mut cur_base = base;

                for _ in 0..count {
                    cur_base = pattern.append_offsets(cur_base, vec);
                }

                cur_base
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use utils::ByteSize;

    fn create_types() -> Vec<GCType> {
        // linked list: struct {ref, int64}
        let a = GCType{
            id: 0,
            alignment: 8,

            fix_size: 16,
            fix_refs: Some(RefPattern::Map{
                offsets: vec![0],
                size: 16
            }),

            var_size: None,
            var_refs: None
        };

        // array of struct {ref, int64} with length 10
        let b = GCType {
            id: 1,
            alignment: 8,

            fix_size: 160,
            fix_refs: Some(RefPattern::Repeat {
                pattern: Box::new(RefPattern::Map{
                    offsets: vec![0],
                    size   : 16
                }),
                count: 10
            }),

            var_size: None,
            var_refs: None
        };

        // array(10) of array(10) of struct {ref, int64}
        let c = GCType {
            id: 2,
            alignment: 8,

            fix_size: 1600,
            fix_refs: Some(RefPattern::Repeat {
                pattern: Box::new(RefPattern::NestedType(vec![Arc::new(b.clone()).clone()])),
                count  : 10
            }),

            var_size: None,
            var_refs: None
        };

        vec![a, b, c]
    }

    #[test]
    fn test_types() {
        create_types();
    }

    #[test]
    fn test_hybrid_type() {
        // hybrid { fix: ref, int } { var: int }
        let a = GCType {
            id: 10,
            alignment: 8,

            fix_size: 16,
            fix_refs: Some(RefPattern::Map {
                offsets: vec![0],
                size: 16
            }),

            var_size: Some(8),
            var_refs: None
        };

        assert_eq!(a.gen_hybrid_ref_offsets(5), vec![0]);
        assert_eq!(a.size_hybrid(5), 56);
    }

    #[test]
    fn test_hybrid_type2() {
        // hybrid { fix: ref, int } { var: ref }
        let a = GCType {
            id: 10,
            alignment: 8,

            fix_size: 16,
            fix_refs: Some(RefPattern::Map {
                offsets: vec![0],
                size: 16
            }),

            var_size: Some(8),
            var_refs: Some(RefPattern::Map {
                offsets: vec![0],
                size: 8
            })
        };

        assert_eq!(a.gen_hybrid_ref_offsets(5), vec![0, 16, 24, 32, 40, 48]);
        assert_eq!(a.size_hybrid(5), 56);
    }

    #[test]
    fn test_ref_offsets() {
        let vec = create_types();

        assert_eq!(vec[0].gen_ref_offsets(), vec![0]);
        assert_eq!(vec[1].gen_ref_offsets(), vec![0, 16, 32, 48, 64, 80, 96, 112, 128, 144]);
        assert_eq!(vec[2].gen_ref_offsets(), (0..100).map(|x| x * 16).collect::<Vec<ByteSize>>());

        let int = GCType {
            id: 3,
            alignment: 8,

            fix_size: 8,
            fix_refs: None,

            var_size: None,
            var_refs: None
        };

        assert_eq!(int.gen_ref_offsets(), vec![]);
    }
}
