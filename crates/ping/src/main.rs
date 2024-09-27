//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use log::{debug, info};
use sel4_bounce_buffer_allocator::{Basic, BounceBufferAllocator};
use sel4_driver_interfaces::net::GetNetDeviceMeta;
use sel4_externally_shared::{ExternallySharedRef, ExternallySharedRefExt};
use sel4_microkit::{
    memory_region_symbol, protection_domain, Channel, Handler, Infallible, MessageInfo,
};
use sel4_microkit_driver_adapters::net::client::Client as NetClient;
use sel4_shared_ring_buffer::RingBuffers;
use sel4_shared_ring_buffer_smoltcp::DeviceImpl;
use smoltcp::{
    iface::{Config, Interface, SocketHandle, SocketSet},
    phy::{Device, DeviceCapabilities, Medium},
    socket::icmp,
    time::Instant,
    wire::{EthernetAddress, HardwareAddress, IpCidr},
};

mod config;

#[protection_domain(
    heap_size = 16*1024*1024,
)]
fn init() -> HandlerImpl<'static> {
    config::log::LOGGER.set().unwrap();
    let mut net_client = NetClient::new(config::channels::NET_DEV);
    let notify_net: fn() = || config::channels::NET_DEV.notify();

    let mut net_device = {
        let dma_region = unsafe {
            ExternallySharedRef::<'static, _>::new(
                memory_region_symbol!(net_client_dma_vaddr: *mut [u8], n = config::sizes::NET_CLIENT_DMA),
            )
        };

        let bounce_buffer_allocator =
            BounceBufferAllocator::new(Basic::new(dma_region.as_ptr().len()), 1);

        DeviceImpl::new(
            Default::default(),
            dma_region,
            bounce_buffer_allocator,
            RingBuffers::from_ptrs_using_default_initialization_strategy_for_role(
                unsafe { ExternallySharedRef::new(memory_region_symbol!(net_rx_free: *mut _)) },
                unsafe { ExternallySharedRef::new(memory_region_symbol!(net_rx_used: *mut _)) },
                notify_net,
            ),
            RingBuffers::from_ptrs_using_default_initialization_strategy_for_role(
                unsafe { ExternallySharedRef::new(memory_region_symbol!(net_tx_free: *mut _)) },
                unsafe { ExternallySharedRef::new(memory_region_symbol!(net_tx_used: *mut _)) },
                notify_net,
            ),
            128,
            1600,
            {
                // TODO: Should this be queried from the driver via a protection call?
                let mut caps = DeviceCapabilities::default();
                caps.max_transmission_unit = 1600;
                caps
            },
        )
        .unwrap()
    };

    let net_config = {
        assert_eq!(net_device.capabilities().medium, Medium::Ethernet);
        let mac_address = EthernetAddress(net_client.get_mac_address().unwrap().0);
        let hardware_addr = HardwareAddress::Ethernet(mac_address);
        let mut this = Config::new(hardware_addr);
        this.random_seed = 0;
        this
    };

    // TODO: Need a timer driver?
    // let mut iface = Interface::new(net_config, &mut net_device, Instant::now());
    let iface = {
        let mut iface = Interface::new(net_config, &mut net_device, Instant::ZERO);
        iface.update_ip_addrs(|ip_addrs| {
            ip_addrs.push(IpCidr::new(config::network::IP, 24)).unwrap();
        });
        iface
            .routes_mut()
            .add_default_ipv4_route(config::network::GATEWAY)
            .unwrap();
        iface
    };

    let (sockets, icmp_handle) = {
        let icmp_rx_buffer =
            icmp::PacketBuffer::new(vec![icmp::PacketMetadata::EMPTY], vec![0; 256]);
        let icmp_tx_buffer =
            icmp::PacketBuffer::new(vec![icmp::PacketMetadata::EMPTY], vec![0; 256]);
        let icmp_socket = icmp::Socket::new(icmp_rx_buffer, icmp_tx_buffer);
        let mut sockets = SocketSet::new(vec![]);
        let icmp_handle = sockets.add(icmp_socket);
        (sockets, icmp_handle)
    };

    info!("Initialized Ping Server: {}", config::network::IP);
    HandlerImpl {
        net_driver_channel: config::channels::NET_DEV,
        net_device,
        iface,
        sockets,
        icmp_handle,
    }
}

struct HandlerImpl<'a> {
    net_driver_channel: sel4_microkit::Channel,
    net_device: DeviceImpl<Basic>,
    iface: Interface,
    sockets: SocketSet<'a>,
    icmp_handle: SocketHandle,
}

impl Handler for HandlerImpl<'_> {
    type Error = Infallible;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        if channel == self.net_driver_channel {
            // Can the socket close? Should this be done in init or here?
            {
                let socket = self.sockets.get_mut::<icmp::Socket>(self.icmp_handle);
                if !socket.is_open() {
                    let ident = 0xb;
                    debug!("Bind icmp socket: {ident}");
                    socket.bind(icmp::Endpoint::Ident(ident)).unwrap();
                }
            }
            let timestamp = Instant::ZERO;
            self.net_device.poll();
            self.iface
                .poll(timestamp, &mut self.net_device, &mut self.sockets);
        }
        Ok(())
    }

    fn protected(
        &mut self,
        _channel: Channel,
        _msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        debug!("Shouldn't be in protected");
        unreachable!()
    }
}
