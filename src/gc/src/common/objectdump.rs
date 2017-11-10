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

use utils::*;
use objectmodel::*;
use heap::*;
use heap::immix::*;
use heap::freelist::*;

use std::collections::HashMap;
use std::mem::transmute;

pub struct HeapDump {
    pub objects: HashMap<Address, ObjectDump>,
    pub relocatable_refs: HashMap<Address, String>
}

pub struct ObjectDump {
    pub addr: Address,
    pub size: ByteSize,
    pub align: ByteSize,
    pub encode: ObjectEncode,
    pub reference_offsets: Vec<ByteSize> // based on reference_addr
}

impl HeapDump {
    pub fn from_roots(roots: Vec<Address>) -> HeapDump {
        trace!("dump heap from {:?}", roots);
        let mut work_queue: Vec<Address> = roots;
        let mut heap: HeapDump = HeapDump {
            objects: HashMap::new(),
            relocatable_refs: HashMap::new()
        };

        while !work_queue.is_empty() {
            let obj = work_queue.pop().unwrap();

            if !heap.objects.contains_key(&obj) {
                // add this object to heap dump
                let obj_dump = heap.persist_object(obj);
                heap.objects.insert(obj, obj_dump);

                heap.keep_tracing(heap.objects.get(&obj).unwrap(), &mut work_queue);
            }
        }

        heap.label_relocatable_refs();

        heap
    }

    #[allow(unused_variables)]
    fn persist_object(&self, obj: Address) -> ObjectDump {
        trace!("dump object: {}", obj);

        match SpaceDescriptor::get(unsafe { obj.to_object_reference() }) {
            SpaceDescriptor::ImmixTiny => {
                let space = ImmixSpace::get::<ImmixSpace>(obj);
                let encode: TinyObjectEncode = unsafe {
                    space
                        .get_type_byte_slot(space.get_word_index(obj))
                        .load::<TinyObjectEncode>()
                };
                // get a vector of all reference offsets
                let mut ref_offsets = vec![];
                for i in 0..encode.n_fields() {
                    if encode.field(i) != WordType::NonRef {
                        ref_offsets.push(i * POINTER_SIZE);
                    }
                }
                ObjectDump {
                    addr: obj,
                    size: encode.size(),
                    align: MINIMAL_ALIGNMENT,
                    encode: ObjectEncode::Tiny(encode),
                    reference_offsets: ref_offsets
                }
            }
            SpaceDescriptor::ImmixNormal => {
                let space = ImmixSpace::get::<ImmixSpace>(obj);
                let encode: MediumObjectEncode = unsafe {
                    space
                        .get_type_byte_slot(space.get_word_index(obj))
                        .load::<MediumObjectEncode>()
                };
                let small_encode: &SmallObjectEncode = unsafe { transmute(&encode) };

                // get type id
                let (type_id, type_size) = if small_encode.is_small() {
                    (small_encode.type_id(), small_encode.size())
                } else {
                    (encode.type_id(), encode.size())
                };

                // get type encode, and find all references
                let type_encode: &ShortTypeEncode = &GlobalTypeTable::table()[type_id];
                let mut ref_offsets = vec![];
                let mut offset = 0;
                for i in 0..type_encode.fix_len() {
                    if type_encode.fix_ty(i) != WordType::NonRef {
                        ref_offsets.push(offset);
                    }
                    offset += POINTER_SIZE;
                }
                if type_encode.var_len() != 0 {
                    while offset < type_size {
                        for i in 0..type_encode.var_len() {
                            if type_encode.var_ty(i) != WordType::NonRef {
                                ref_offsets.push(offset);
                            }
                            offset += POINTER_SIZE;
                        }
                    }
                }
                ObjectDump {
                    addr: obj,
                    size: type_size,
                    align: type_encode.align(),
                    encode: if small_encode.is_small() {
                        ObjectEncode::Small(small_encode.clone())
                    } else {
                        ObjectEncode::Medium(encode)
                    },
                    reference_offsets: ref_offsets
                }
            }
            SpaceDescriptor::Freelist => {
                let space = FreelistSpace::get::<FreelistSpace>(obj);
                let encode = unsafe { space.get_type_encode_slot(obj).load::<LargeObjectEncode>() };
                let ty_encode = GlobalTypeTable::get_full_type(encode.type_id());

                let mut ref_offsets = vec![];
                let mut offset: ByteSize = 0;
                for &word in ty_encode.fix.iter() {
                    if word != WordType::NonRef {
                        ref_offsets.push(offset);
                    }
                    offset += POINTER_SIZE;
                }
                while offset < encode.size() {
                    for &word in ty_encode.var.iter() {
                        if word != WordType::NonRef {
                            ref_offsets.push(offset);
                        }
                        offset += POINTER_SIZE;
                    }
                }
                ObjectDump {
                    addr: obj,
                    size: encode.size(),
                    align: ty_encode.align,
                    encode: ObjectEncode::Large(encode),
                    reference_offsets: ref_offsets
                }
            }
            SpaceDescriptor::Immortal => unimplemented!()
        }
    }

    fn keep_tracing(&self, obj_dump: &ObjectDump, work_queue: &mut Vec<Address>) {
        let base = obj_dump.addr;

        for offset in obj_dump.reference_offsets.iter() {
            let field_addr = base + *offset;
            let edge = unsafe { field_addr.load::<Address>() };

            trace!(
                "object reference from {} -> {} at +[{}]",
                base,
                edge,
                offset
            );

            if !edge.is_zero() && !self.objects.contains_key(&edge) {
                work_queue.push(edge);
            }
        }
    }

    fn label_relocatable_refs(&mut self) {
        let mut count = 0;

        for addr in self.objects.keys() {
            let label = format!("GCDUMP_{}_{}", count, addr);
            self.relocatable_refs.insert(*addr, label);

            count += 1;
        }
    }
}

use std::fmt;

impl fmt::Debug for ObjectDump {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PersistedObject({}, {} bytes, {} bytes aligned, refs at {:?})",
            self.addr,
            self.size,
            self.align,
            self.reference_offsets
        )
    }
}

impl fmt::Debug for HeapDump {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Heap Dump\n").unwrap();

        write!(f, "---{} objects---\n", self.objects.len()).unwrap();
        for obj in self.objects.iter() {
            write!(f, "{:?}\n", obj).unwrap();
        }

        write!(f, "---{} ref labels---\n", self.relocatable_refs.len()).unwrap();
        for (addr, label) in self.relocatable_refs.iter() {
            write!(f, "{} = {}\n", addr, label).unwrap()
        }

        Ok(())
    }
}
