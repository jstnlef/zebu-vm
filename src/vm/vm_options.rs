use std::default::Default;
use utils::ByteSize;

pub struct VMOptions {
    // gc options
    pub immix_size: ByteSize,
    pub lo_size: ByteSize,
    pub n_gcthreads: usize
}

pub const DEFAULT_IMMIX_SIZE : ByteSize = 1 << 16;  // 64Mb
pub const DEFAULT_LO_SIZE    : ByteSize = 1 << 16;  // 64Mb
pub const DEFAULT_N_GCTHREADS: usize = 8;

impl Default for VMOptions {
    fn default() -> VMOptions {
        VMOptions {
            immix_size: DEFAULT_IMMIX_SIZE,
            lo_size: DEFAULT_LO_SIZE,
            n_gcthreads: DEFAULT_N_GCTHREADS
        }
    }
}
