//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr::NonNull;
use log::debug;

pub struct GemDmaPtrs {
    pub rx: DmaPtrs,
    pub tx: DmaPtrs,
    pub tx_dummy: DmaPtr,
}

pub struct DmaPtrs {
    pub desc: DmaPtr,
    pub buf: *mut (),
}

pub struct DmaPtr {
    pub vaddr: *mut (),
    pub paddr: *mut (),
}

pub struct DmaDef {
    pub vaddr: NonNull<()>,
    pub paddr: NonNull<()>,
    pub size: usize,
}

use super::NUM_BUFS;

use super::rx::DESC_SIZE as RX_DESC_SIZE;
use super::tx::DESC_SIZE as TX_DESC_SIZE;

pub fn alloc_dma(dma: DmaDef, rx_buf_paddr: *mut (), tx_buf_paddr: *mut ()) -> GemDmaPtrs {
    // TODO: Still need to handle alignment
    // TODO: Implement a more sophisticated allocator?
    let rx_desc_size = (RX_DESC_SIZE * NUM_BUFS) as isize;
    let tx_desc_size = (TX_DESC_SIZE * NUM_BUFS) as isize;
    // let buf_size = (MTU * NUM_BUFS) as isize;

    let rx_desc_vaddr = dma.vaddr;
    let rx_desc_paddr = dma.paddr;
    unsafe {
        let tx_desc_vaddr = rx_desc_vaddr.byte_offset(rx_desc_size);
        let tx_dummy_vaddr = tx_desc_vaddr.byte_offset(tx_desc_size);

        let tx_desc_paddr = rx_desc_paddr.byte_offset(rx_desc_size);
        let tx_dummy_paddr = tx_desc_paddr.byte_offset(tx_desc_size);

        let end = tx_dummy_vaddr.byte_offset(TX_DESC_SIZE as isize);
        let size = end.byte_offset_from(rx_desc_vaddr);
        debug!("dma.size: {}, needed_size: {size}", dma.size);
        assert!(dma.size >= size as usize);

        GemDmaPtrs {
            rx: DmaPtrs {
                desc: DmaPtr {
                    vaddr: rx_desc_vaddr.as_ptr(),
                    paddr: rx_desc_paddr.as_ptr(),
                },
                buf: rx_buf_paddr,
            },
            tx: DmaPtrs {
                desc: DmaPtr {
                    vaddr: tx_desc_vaddr.as_ptr(),
                    paddr: tx_desc_paddr.as_ptr(),
                },
                buf: tx_buf_paddr,
            },
            tx_dummy: DmaPtr {
                vaddr: tx_dummy_vaddr.as_ptr(),
                paddr: tx_dummy_paddr.as_ptr(),
            },
        }
    }
}
