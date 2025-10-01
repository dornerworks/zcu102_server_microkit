#![cfg_attr(not(test), no_std)]

mod queues;
pub use queues::{BufferDesc, BufferQueue, BUFFER_DESC_ARRAY_LEN};

pub const QUEUE_SIZE: usize = 8;

pub struct QueuePair {
    pub avail: BufferQueue,
    pub free: BufferQueue,
}

pub const fn empty_buf_queue() -> BufferQueue {
    BufferQueue {
        head: 0,
        tail: 0,
        consumer_signalled: 0,
        buffers: [BufferDesc {
            offset: 0,
            length: 0,
        }; BUFFER_DESC_ARRAY_LEN],
    }
}

pub fn full(queue: &BufferQueue) -> bool {
    (queue.tail as usize - queue.head as usize) == QUEUE_SIZE
}

pub fn empty(queue: &BufferQueue) -> bool {
    (queue.tail as usize - queue.head as usize) == 0
}

pub fn enqueue(queue: &mut BufferQueue, buffer: BufferDesc) -> bool {
    if full(queue) {
        false
    } else {
        queue.buffers[queue.tail as usize % QUEUE_SIZE] = buffer;
        // TODO: Not needed until we support multicore
        // memory_release();
        let old_tail = queue.tail;
        queue.tail = old_tail + 1;
        true
    }
}

pub fn dequeue(queue: &mut BufferQueue) -> Option<BufferDesc> {
    if empty(queue) {
        None
    } else {
        let buffer = queue.buffers[queue.head as usize % QUEUE_SIZE];
        let old_head = queue.head;
        queue.head = old_head + 1;
        Some(buffer)
    }
}
