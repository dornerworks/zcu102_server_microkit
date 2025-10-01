#![cfg_attr(not(test), no_std)]

use core::ops::{Deref, DerefMut};

pub const QUEUE_SIZE: usize = 8;

pub struct QueuePair {
    pub avail: BufferQueue,
    pub free: BufferQueue,
}

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
pub struct BufferQueueInner {
    pub head: u16,
    pub tail: u16,
    pub consumer_signalled: u32,
    pub buffers: BufferDescArray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferQueue {
    ptr: *mut BufferQueueInner,
}

// impl Default for BufferQueue {
//     fn default() -> Self {
//         Self {
//             head: 0,
//             tail: 0,
//             consumer_signalled: 0,
//             buffers: [BufferDesc::default(); BUFFER_DESC_ARRAY_LEN],
//         }
//     }
// }

impl BufferQueue {
    pub fn new(ptr: *mut BufferQueueInner) -> Self {
        Self { ptr }
    }

    fn ptr(&self) -> *mut BufferQueueInner {
        self.ptr
    }

    pub fn full(&self) -> bool {
        (self.tail as usize - self.head as usize) == QUEUE_SIZE
    }

    pub fn empty(&self) -> bool {
        (self.tail as usize - self.head as usize) == 0
    }

    pub fn enqueue(&mut self, buffer: BufferDesc) -> bool {
        if self.full() {
            false
        } else {
            let tail = self.tail;
            self.buffers[tail as usize % QUEUE_SIZE] = buffer;
            // TODO: Not needed until we support multicore
            // memory_release();
            self.tail = tail + 1;
            true
        }
    }

    pub fn dequeue(&mut self) -> Option<BufferDesc> {
        if self.empty() {
            None
        } else {
            let head = self.head;
            let buffer = self.buffers[head as usize % QUEUE_SIZE];
            self.head = head + 1;
            Some(buffer)
        }
    }
}

impl Deref for BufferQueue {
    type Target = BufferQueueInner;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl DerefMut for BufferQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr() }
    }
}
