//
// Copyright 2024, DornerWorks
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

// use crate::microkit_channel;
// use serde::{Deserialize, Serialize};

use eth_driver_core::{DmaDef, Driver, MTU};
use log::info;
use sddf_utils::{BufferDesc, BufferQueue, QueuePair, BUFFER_DESC_ARRAY_LEN, QUEUE_SIZE};
// use sel4_driver_interfaces::net::{GetNetDeviceMeta, MacAddress};
use sel4_driver_interfaces::HandleInterrupt;
use sel4_microkit::{memory_region_symbol, protection_domain};
use sel4_microkit::{Channel, Handler, Infallible, MessageInfo};
// use sel4_microkit_driver_adapters::net::ErrorResponse;

mod config;

#[protection_domain]
fn init() -> HandlerImpl {
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
            memory_region_symbol!(net_rx_dma_data_paddr: *mut ()).as_ptr(),
            memory_region_symbol!(net_tx_dma_data_paddr: *mut ()).as_ptr(),
        )
    };

    // Just ignore these?
    let _ = memory_region_symbol!(net_rx_dma_data_vaddr: *mut ()).as_ptr();
    let _ = memory_region_symbol!(net_tx_dma_data_vaddr: *mut ()).as_ptr();

    info!("Finished Initializing Driver");
    dev.handle_interrupt();
    info!("Acked driver IRQ");
    config::channels::DEVICE.irq_ack().unwrap();
    info!("Acked physical IRQ");

    let mut handler = HandlerImpl {
        rx: QueuePair {
            avail: BufferQueue::new(memory_region_symbol!(net_rx_used: *mut ()).as_ptr().cast()),
            free: BufferQueue::new(memory_region_symbol!(net_rx_free: *mut ()).as_ptr().cast()),
        },
        tx: QueuePair {
            avail: BufferQueue::new(memory_region_symbol!(net_tx_used: *mut ()).as_ptr().cast()),
            free: BufferQueue::new(memory_region_symbol!(net_tx_free: *mut ()).as_ptr().cast()),
        },
        drv: dev,
        client_channel: config::channels::CLIENT,
        device_channel: config::channels::DEVICE,
    };

    handler.tx_free_init();

    handler
}

struct HandlerImpl {
    rx: QueuePair,
    tx: QueuePair,
    drv: Driver,
    client_channel: Channel,
    device_channel: Channel,
}

impl HandlerImpl {
    pub fn tx_free_init(&mut self) {
        for i in 0..QUEUE_SIZE {
            let buffer = BufferDesc {
                offset: i * MTU,
                length: BUFFER_DESC_ARRAY_LEN as u16,
            };
            self.tx.free.enqueue(buffer);
        }
    }
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        if channel == self.client_channel || channel == self.device_channel {
            let mut notify_client = false;
            loop {
                match self.rx.free.dequeue() {
                    Some(buffer) => {
                        self.drv.rx_mark_done(buffer.offset);
                        notify_client = true;
                        // info!("Mark done {}", buffer.index);
                    }
                    None => break,
                }
            }

            // TODO: Split up device handling vs client handling?
            for _ in 0..QUEUE_SIZE {
                match self.drv.receive() {
                    Some(offset) => {
                        let buffer = BufferDesc {
                            offset,
                            length: MTU as u16,
                        };
                        self.rx.avail.enqueue(buffer);
                        notify_client = true;
                    }
                    None => break,
                }
            }

            loop {
                match self.tx.avail.dequeue() {
                    Some(buffer) => {
                        // info!("Transmit buffer {}", buffer.index);
                        self.drv.transmit(buffer.offset, buffer.length.into());
                        // TODO: Should we do this somewhere else?
                        self.tx.free.enqueue(buffer);
                        notify_client = true;
                    }
                    None => break,
                }
            }

            if notify_client {
                self.client_channel.notify();
            }

            self.drv.handle_interrupt();
            self.device_channel.irq_ack().unwrap();
        }
        Ok(())
    }

    fn protected(
        &mut self,
        _channel: Channel,
        _msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        unreachable!()
        // if channel == self.client_channel {
        //     Ok(handle_client_request(&mut self.drv, msg_info))
        // } else {
        //     unreachable!()
        // }
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub enum Request {
//     GetMacAddress,
// }

// pub type Response = Result<SuccessResponse, ErrorResponse>;

// #[derive(Debug, Serialize, Deserialize)]
// pub enum SuccessResponse {
//     GetMacAddress(MacAddress),
// }

// #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
// pub enum ErrorResponse {
//     Unspecified,
// }

// pub fn handle_client_request<T: GetNetDeviceMeta>(
//     dev: &mut T,
//     msg_info: MessageInfo,
// ) -> MessageInfo {
//     match msg_info.recv_using_postcard::<Request>() {
//         Ok(req) => {
//             let resp: Response = match req {
//                 Request::GetMacAddress => dev
//                     .get_mac_address()
//                     .map(SuccessResponse::GetMacAddress)
//                     .map_err(|_| ErrorResponse::Unspecified),
//             };
//             MessageInfo::send_using_postcard(resp).unwrap()
//         }
//         Err(_) => MessageInfo::send_unspecified_error(),
//     }
// }
