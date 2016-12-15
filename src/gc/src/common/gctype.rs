use utils::ByteSize;

#[derive(Clone, Debug)]
pub struct GCType {
    id: usize,
    size: ByteSize,
    non_repeat_refs: Option<RefPattern>,
    repeat_refs    : Option<RepeatingRefPattern>
}

impl GCType {
    pub fn gen_ref_offsets(&self) -> Vec<ByteSize> {
        let mut ret = vec![];

        let mut cur_offset = 0;

        match self.non_repeat_refs {
            Some(ref pattern) => {
                cur_offset = pattern.append_offsets(cur_offset, &mut ret);
            }
            None => {}
        }

        if self.repeat_refs.is_some() {
            let repeat_refs = self.repeat_refs.as_ref().unwrap();

            cur_offset = repeat_refs.append_offsets(cur_offset, &mut ret);
        }

        ret
    }
}

#[derive(Clone, Debug)]
pub enum RefPattern {
    Map{
        offsets: Vec<ByteSize>,
        size : usize
    },
    NestedType(Vec<GCType>)
}

impl RefPattern {
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

                    cur_base += ty.size;
                }

                cur_base
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct RepeatingRefPattern {
    pattern: RefPattern,
    count: usize
}

impl RepeatingRefPattern {
    pub fn append_offsets(&self, base: ByteSize, vec: &mut Vec<ByteSize>) -> ByteSize {
        let mut cur_base = base;

        for i in 0..self.count {
            cur_base = self.pattern.append_offsets(cur_base, vec);
        }

        cur_base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::ByteSize;

    fn create_types() -> Vec<GCType> {
        // linked list: struct {ref, int64}
        let a = GCType{
            id: 0,
            size: 16,
            non_repeat_refs: Some(RefPattern::Map{
                offsets: vec![0],
                size: 16
            }),
            repeat_refs    : None
        };

        // array of struct {ref, int64} with length 10
        let b = GCType {
            id: 1,
            size: 160,
            non_repeat_refs: None,
            repeat_refs    : Some(RepeatingRefPattern {
                pattern: RefPattern::Map{
                    offsets: vec![0],
                    size   : 16
                },
                count  : 10
            })
        };

        // array(10) of array(10) of struct {ref, int64}
        let c = GCType {
            id: 2,
            size: 1600,
            non_repeat_refs: None,
            repeat_refs    : Some(RepeatingRefPattern {
                pattern: RefPattern::NestedType(vec![b.clone()]),
                count  : 10
            })
        };

        vec![a, b, c]
    }

    #[test]
    fn test_types() {
        create_types();
    }

    #[test]
    fn test_ref_offsets() {
        let vec = create_types();

        assert_eq!(vec[0].gen_ref_offsets(), vec![0]);
        assert_eq!(vec[1].gen_ref_offsets(), vec![0, 16, 32, 48, 64, 80, 96, 112, 128, 144]);
        assert_eq!(vec[2].gen_ref_offsets(), (0..100).map(|x| x * 16).collect::<Vec<ByteSize>>());
    }
}