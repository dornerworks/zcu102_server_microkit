//
// Copyright 2024, DornerWorks
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use eth_driver_core::{DmaDef, Driver};
use log::info;
use sel4_driver_interfaces::HandleInterrupt;
use sel4_externally_shared::{ExternallySharedRef, ExternallySharedRefExt};
use sel4_microkit::{memory_region_symbol, protection_domain};
use sel4_microkit_driver_adapters::net::driver::HandlerImpl;
use sel4_shared_ring_buffer::{roles::Use, RingBuffers};

mod config;

#[protection_domain]
fn init() -> HandlerImpl<Driver> {
    config::log::LOGGER.set().unwrap();
    let mut dev = {
        let dma = DmaDef {
            vaddr: memory_region_symbol!(net_driver_dma_vaddr: *mut ()),
            paddr: memory_region_symbol!(net_driver_dma_paddr: *mut ()),
            size: config::sizes::DRIVER_DMA,
        };
        Driver::new(
            memory_region_symbol!(gem_register_block: *mut ()).as_ptr(),
            dma,
        )
    };

    let client_region = unsafe {
        ExternallySharedRef::<'static, _>::new(
            memory_region_symbol!(net_client_dma_vaddr: *mut [u8], n = config::sizes::NET_CLIENT_DMA),
        )
    };

    let notify_client: fn() = || config::channels::CLIENT.notify();

    let rx_ring_buffers =
        RingBuffers::<'_, Use, fn()>::from_ptrs_using_default_initialization_strategy_for_role(
            unsafe { ExternallySharedRef::new(memory_region_symbol!(net_rx_free: *mut _)) },
            unsafe { ExternallySharedRef::new(memory_region_symbol!(net_rx_used: *mut _)) },
            notify_client,
        );

    let tx_ring_buffers =
        RingBuffers::<'_, Use, fn()>::from_ptrs_using_default_initialization_strategy_for_role(
            unsafe { ExternallySharedRef::new(memory_region_symbol!(net_tx_free: *mut _)) },
            unsafe { ExternallySharedRef::new(memory_region_symbol!(net_tx_used: *mut _)) },
            notify_client,
        );

    info!("Finished Initializing Driver");
    dev.handle_interrupt();
    info!("Acked driver IRQ");
    config::channels::DEVICE.irq_ack().unwrap();
    info!("Acked physical IRQ");

    HandlerImpl::<Driver>::new(
        dev,
        client_region,
        rx_ring_buffers,
        tx_ring_buffers,
        config::channels::DEVICE,
        config::channels::CLIENT,
    )
}
