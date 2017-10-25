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

use utils::Address;
use utils::ByteSize;
use utils::POINTER_SIZE;
use common::gctype::*;

use MY_GC;
use objectmodel;

use std::collections::HashMap;
use std::sync::Arc;

pub struct HeapDump {
    pub objects: HashMap<Address, ObjectDump>,
    pub relocatable_refs: HashMap<Address, String>
}

pub struct ObjectDump {
    pub mem_start: Address,
    pub mem_size: ByteSize,

    pub reference_addr: Address,
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

    #[cfg(feature = "use-sidemap")]
    fn persist_object(&self, obj: Address) -> ObjectDump {
        unimplemented!()
    }

    #[cfg(not(feature = "use-sidemap"))]
    fn persist_object(&self, obj: Address) -> ObjectDump {
        trace!("dump object: {}", obj);
        let hdr_addr = obj + objectmodel::OBJECT_HEADER_OFFSET;
        let hdr = unsafe { hdr_addr.load::<u64>() };

        if objectmodel::header_is_fix_size(hdr) {
            // fix sized type
            if objectmodel::header_has_ref_map(hdr) {
                // has ref map
                let ref_map = objectmodel::header_get_ref_map(hdr);

                trace!("fix sized, ref map as {:b}", ref_map);

                let mut offsets = vec![];
                let mut i = 0;
                while i < objectmodel::REF_MAP_LENGTH {
                    let has_ref: bool = ((ref_map >> i) & 1) == 1;

                    if has_ref {
                        offsets.push(i * POINTER_SIZE);
                    }

                    i += 1;
                }

                ObjectDump {
                    reference_addr: obj,
                    mem_start: hdr_addr,
                    mem_size: objectmodel::header_get_object_size(hdr) as usize +
                        objectmodel::OBJECT_HEADER_SIZE,
                    reference_offsets: offsets
                }
            } else {
                // by type ID
                let gctype_id = objectmodel::header_get_gctype_id(hdr);

                trace!("fix size, type id as {}", gctype_id);

                let gc_lock = MY_GC.read().unwrap();
                let gctype: Arc<GCType> =
                    gc_lock.as_ref().unwrap().gc_types[gctype_id as usize].clone();

                ObjectDump {
                    reference_addr: obj,
                    mem_start: hdr_addr,
                    mem_size: gctype.size() + objectmodel::OBJECT_HEADER_SIZE,
                    reference_offsets: gctype.gen_ref_offsets()
                }
            }
        } else {
            // hybrids - same as above
            let gctype_id = objectmodel::header_get_gctype_id(hdr);
            let var_length = objectmodel::header_get_hybrid_length(hdr);

            trace!("var sized, type id as {}", gctype_id);

            let gc_lock = MY_GC.read().unwrap();
            let gctype: Arc<GCType> =
                gc_lock.as_ref().unwrap().gc_types[gctype_id as usize].clone();

            ObjectDump {
                reference_addr: obj,
                mem_start: hdr_addr,
                mem_size: gctype.size_hybrid(var_length) + objectmodel::OBJECT_HEADER_SIZE,
                reference_offsets: gctype.gen_hybrid_ref_offsets(var_length)
            }
        }
    }

    fn keep_tracing(&self, obj_dump: &ObjectDump, work_queue: &mut Vec<Address>) {
        let base = obj_dump.reference_addr;

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
            "PersistedObject({}, {} bytes from {}, offsets at {:?})",
            self.reference_addr,
            self.mem_size,
            self.mem_start,
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
