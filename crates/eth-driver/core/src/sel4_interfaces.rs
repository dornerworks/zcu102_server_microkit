//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

use super::Driver;
use sel4_driver_interfaces::net::GetNetDeviceMeta;
use sel4_driver_interfaces::HandleInterrupt;

// This is the only dependency on rust sel4 code in core.
// TODO: Should/can this be implemented outside of core? If so, should this also just become an included lib?

impl HandleInterrupt for Driver {
    fn handle_interrupt(&mut self) {
        if self.dev.rx_is_complete() {
            let _sta = self.dev.get_receive_status();
        }
        if self.dev.tx_is_complete() {
            let _val = self.dev.get_transmit_status();
        }

        self.dev.clear_all_interrupts();
    }
}

impl GetNetDeviceMeta for Driver {
    type Error = core::convert::Infallible;
    fn get_mac_address(&mut self) -> Result<sel4_driver_interfaces::net::MacAddress, Self::Error> {
        Ok(sel4_driver_interfaces::net::MacAddress(
            self.dev.mac_address().inner(),
        ))
    }
}
