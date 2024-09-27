//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

use super::Driver;
use core::ops::{Deref, DerefMut};
use log::debug;
use smoltcp::{
    phy::{ChecksumCapabilities, Device, DeviceCapabilities, Medium, RxToken, TxToken},
    time::Instant,
};
use zynqmp_hal::gem::Running;

mod alloc;
mod rx;
mod tx;

pub use alloc::{alloc_dma, DmaDef, DmaPtr, DmaPtrs, GemDmaPtrs};
pub use rx::RxRing;
pub use tx::{TxDummy, TxRing};

const NUM_BUFS: usize = 128;
pub const MTU: usize = 1600;

impl Device for Driver {
    type RxToken<'token> = GemRxToken<'token> where Self: 'token;
    type TxToken<'token> = GemTxToken<'token> where Self: 'token;

    // Required methods
    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if self.tx_available() && self.rx_available() {
            let rx = GemRxToken {
                rx_ring: &mut self.rx_ring,
            };

            let tx = GemTxToken {
                tx_ring: &mut self.tx_ring,
                dev: &self.dev,
            };
            Some((rx, tx))
        } else {
            None
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        if self.tx_available() {
            Some(GemTxToken {
                tx_ring: &mut self.tx_ring,
                dev: &self.dev,
            })
        } else {
            None
        }
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut dev_caps = DeviceCapabilities::default();
        dev_caps.medium = Medium::Ethernet;
        dev_caps.max_transmission_unit = MTU;
        dev_caps.max_burst_size = Some(1);
        dev_caps.checksum = ChecksumCapabilities::ignored();
        dev_caps
    }
}

pub struct GemRxToken<'a> {
    rx_ring: &'a mut RxRing,
}

impl<'a> RxToken for GemRxToken<'a> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let packet = self.rx_ring.recv_next();
        let result = f(packet);
        self.rx_ring.mark_done();
        result
    }
}

pub struct GemTxToken<'a> {
    tx_ring: &'a mut TxRing,
    dev: &'a zynqmp_hal::gem::Device<Running>,
}

impl<'a> TxToken for GemTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // TODO: This sends a malformed packet if len > MTU. Should we panic instead?
        let send_len = if len > MTU { MTU } else { len };
        let tx_packet = self.tx_ring.get_next_buffer(send_len);
        let result = f(&mut tx_packet[..send_len]);
        let _desc_paddr = self.tx_ring.send_complete();
        // TODO: Should we set tx_desc every time?
        self.dev.transmit();
        debug!("tx_desc: 0x{:0X}", self.dev.get_tx_desc());
        result
    }
}

// TODO: Alignment
type BufPkt = [u8; MTU];

struct DataBuf {
    buffer: *mut [BufPkt; NUM_BUFS],
}

impl DataBuf {
    fn new(buffer: *mut [BufPkt; NUM_BUFS]) -> Self {
        Self { buffer }
    }

    fn get(&mut self, curr_entry: usize) -> &mut [u8] {
        &mut self[curr_entry]
    }
}

impl Deref for DataBuf {
    type Target = [BufPkt; NUM_BUFS];

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.buffer }
    }
}

impl DerefMut for DataBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.buffer }
    }
}
