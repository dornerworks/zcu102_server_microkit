//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use eth_phy::dp83867::{DP83867Conf, Phy, PortMirroring};
use eth_phy::{configure_phy, GenPhy, PhyInterface, Supported};
use log::{error, info};
use zynqmp_hal::gem::{Device, MacAddress, Running};

mod dma;
mod sel4_interfaces;

use dma::{alloc_dma, GemDmaPtrs, RxRing, TxDummy, TxRing};
pub use dma::{DmaDef, MTU};

pub struct Driver {
    pub dev: Device<Running>,
    rx_ring: RxRing,
    tx_ring: TxRing,
}

#[derive(Debug)]
pub enum IrqType {
    TxComplete,
    RxComplete,
    Unknown,
}

const MAC: [u8; 6] = [0x00, 0x0A, 0x35, 0x03, 0x78, 0xA1];

impl Driver {
    pub fn new(ptr: *mut (), dma: DmaDef, rx_buf_paddr: *mut (), tx_buf_paddr: *mut ()) -> Self {
        let dma_ptrs = alloc_dma(dma, rx_buf_paddr, tx_buf_paddr);
        let rx_ring = RxRing::new(&dma_ptrs.rx);
        let tx_ring = TxRing::new(&dma_ptrs.tx);
        let _tx_dummy = TxDummy::new(&dma_ptrs.tx_dummy);
        let dev = Self::init(ptr, &dma_ptrs);

        Self {
            dev,
            rx_ring,
            tx_ring,
        }
    }

    fn init(ptr: *mut (), dma_ptrs: &GemDmaPtrs) -> Device<Running> {
        info!("Initializing Driver");
        let dev = Device::new(ptr.cast());
        let dev = dev.init();
        info!("Initialized GEM device");

        let (speed, duplex) = {
            let supported = Supported {
                autoneg: true,
                tp: true,
                mii: true,
                base10_t_half: true,
                base10_t_full: true,
                base100_t_half: true,
                base100_t_full: true,
                base1000_t_half: true,
                base1000_t_full: true,
                ..Default::default()
            };
            // ZCU102 PHY and its configuration
            let conf = DP83867Conf {
                rx_id_delay: 0x8,
                tx_id_delay: 0xa,
                fifo_depth: 1,
                io_impedance: None,
                rxctrl_strap_quirk: true,
                port_mirroring: PortMirroring::KEEP,
                set_clk_output: true,
                clk_output_sel: Some(0),
                sgmii_ref_clk_en: false,
                interface: PhyInterface::RgmiiId,
            };
            let genphy = GenPhy::new(0xc, &dev, supported);
            let phy = Phy::new(&genphy, conf);
            configure_phy(&genphy, &phy)
        };
        let dev = dev.phy_complete();

        dev.set_rx_desc(dma_ptrs.rx.desc.paddr as u32);
        // TODO: Should this be done each time a packet is sent?
        dev.set_tx_desc(dma_ptrs.tx.desc.paddr as u32);
        dev.set_tx_q1_desc(dma_ptrs.tx_dummy.paddr as u32);
        dev.set_mac_address(MacAddress::new(MAC));

        info!("PHY: Speed: {speed:?}, Duplex: {duplex:?}");
        dev.set_speed(speed);
        dev.set_duplex(duplex);

        dev.run()
    }

    pub fn get_irq_type(&self) -> IrqType {
        if self.dev.rx_is_complete() {
            IrqType::RxComplete
        } else if self.dev.tx_is_complete() {
            IrqType::TxComplete
        } else {
            IrqType::Unknown
        }
    }

    pub fn rx_mark_done(&mut self, offset: usize) {
        self.rx_ring.mark_done(offset);
    }

    pub fn receive(&mut self) -> Option<usize> {
        if self.rx_ring.next_entry_available() {
            Some(self.rx_ring.recv_next())
        } else {
            None
        }
    }

    pub fn transmit(&mut self, offset: usize, len: usize) {
        if self.tx_ring.entry_available(offset) {
            let paddr = self.tx_ring.get_buffer(offset, len);
            self.dev.wait_for_transmit_finish();
            self.dev.set_tx_desc(paddr);
            self.dev.transmit();
            self.dev.wait_for_transmit_finish();
        } else {
            error!("Tried to transmit with a descriptor that SW doesn't own: {offset}. Should not happen");
        }
    }
}
