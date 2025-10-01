#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferDesc {
    pub offset: usize,
    pub length: u16,
}

impl Default for BufferDesc {
    fn default() -> Self {
        Self {
            offset: 0,
            length: 0,
        }
    }
}

pub const BUFFER_DESC_ARRAY_LEN: usize = 128;

pub type BufferDescArray = [BufferDesc; BUFFER_DESC_ARRAY_LEN];

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferQueue {
    pub head: u16,
    pub tail: u16,
    pub consumer_signalled: u32,
    pub buffers: BufferDescArray,
}

impl Default for BufferQueue {
    fn default() -> Self {
        Self {
            head: 0,
            tail: 0,
            consumer_signalled: 0,
            buffers: [BufferDesc::default(); BUFFER_DESC_ARRAY_LEN],
        }
    }
}
