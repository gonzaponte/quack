use h5rio::h5type;

#[h5type]
pub struct DaqEventMeta {
    pub event    : u32,
    pub timestamp: u64,
}
