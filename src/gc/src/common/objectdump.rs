use utils::Address;
use utils::ByteSize;
use std::collections::HashMap;

pub struct HeapDump {
    pub objects: HashMap<Address, ObjectDump>,
    pub relocatable_refs: HashMap<Address, String>
}

#[derive(Debug, Clone)]
pub struct ObjectDump {
    pub reference_addr: Address,

    pub mem_start: Address,
    pub mem_size : ByteSize,
    pub reference_offsets: Vec<ByteSize>
}