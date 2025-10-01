//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

use super::{DmaPtr, DmaPtrs, MTU, NUM_BUFS};
use core::ops::{Deref, DerefMut};

mod descriptor;
use descriptor::Descriptor;

pub const DESC_SIZE: usize = core::mem::size_of::<Descriptor>();

pub struct TxDummy {
    desc: *mut Descriptor,
}

impl TxDummy {
    pub fn new(dma_ptr: &DmaPtr) -> Self {
        let mut tx_dummy = Self {
            desc: dma_ptr.vaddr.cast(),
        };
        tx_dummy.setup();
        tx_dummy
    }

    fn setup(&mut self) {
        self.clear_status();
        self.mark_sw_owned();
        self.mark_last();
    }
}

impl Deref for TxDummy {
    type Target = Descriptor;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.desc }
    }
}

impl DerefMut for TxDummy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.desc }
    }
}

pub struct TxRing {
    base_paddr: usize,
    entries: *mut [Descriptor; NUM_BUFS],
}

impl TxRing {
    pub fn new(dma_ptrs: &DmaPtrs) -> Self {
        let entries = dma_ptrs.desc.vaddr.cast();
        let mut ring = Self {
            base_paddr: dma_ptrs.desc.paddr as usize,
            entries,
        };
        ring.setup(dma_ptrs.buf as usize);
        ring
    }

    fn entries(&self) -> *mut [Descriptor; NUM_BUFS] {
        self.entries
    }

    fn setup(&mut self, buffers_paddr: usize) {
        for (i, entry) in self.iter_mut().enumerate() {
            entry.set_addr(buffers_paddr + (i * MTU));
            entry.mark_sw_owned();
        }
        self.last_mut().unwrap().mark_last();
    }

    pub fn entry_available(&self, offset: usize) -> bool {
        self.get(offset / MTU).unwrap().is_available()
    }

    pub fn get_buffer(&mut self, offset: usize, len: usize) -> u32 {
        let idx = offset / MTU;
        let desc = self.get_mut(idx).unwrap();
        desc.clear_status();
        desc.set_len(len);
        // Assume only single buffer sized frames
        desc.mark_frame_end();
        desc.mark_gem_owned();

        self.desc_paddr(idx)
    }

    fn desc_paddr(&self, idx: usize) -> u32 {
        (self.base_paddr + idx * 8).try_into().unwrap()
    }
}

impl Deref for TxRing {
    type Target = [Descriptor; NUM_BUFS];

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.entries() }
    }
}

impl DerefMut for TxRing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.entries() }
    }
}
