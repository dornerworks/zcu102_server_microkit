//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub mod network {
    use smoltcp::wire::{IpAddress, Ipv4Address};

    pub const IP: IpAddress = IpAddress::v4(192, 168, 60, 146);
    pub const GATEWAY: Ipv4Address = Ipv4Address::new(192, 168, 60, 158);
}

pub mod channels {
    use sel4_microkit::Channel;

    pub const NET_DEV: Channel = Channel::new(0);
}

pub mod sizes {
    pub const NET_CLIENT_DMA: usize = 0x20_0000;
}

pub mod log {
    use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
    use sel4_microkit::debug_print;

    const LOG_LEVEL: LevelFilter = {
        // LevelFilter::Trace
        // LevelFilter::Debug
        LevelFilter::Info
        // LevelFilter::Warn
    };

    pub static LOGGER: Logger = LoggerBuilder::const_default()
        .level_filter(LOG_LEVEL)
        .write(|s| debug_print!("{}", s))
        .build();
}
