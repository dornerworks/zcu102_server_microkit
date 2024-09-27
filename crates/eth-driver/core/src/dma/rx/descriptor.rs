//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

use tock_registers::interfaces::{ReadWriteable, Readable};
use tock_registers::register_bitfields;
use tock_registers::registers::ReadWrite;

// Only support 32-bit addressing for now
#[repr(C)]
pub struct Descriptor {
    addr: ReadWrite<u32, Addr::Register>,
    status: u32,
}

register_bitfields![u32,
    Addr [
        ADDRESS OFFSET(2) NUMBITS(30) [],
        WRAP OFFSET(1) NUMBITS(1) [],
        AVAIL OFFSET(0) NUMBITS(1) [],
    ]
];

impl Descriptor {
    #[allow(dead_code)]
    pub fn addr(&self) -> usize {
        let unshifted_addr = self.addr.read(Addr::ADDRESS) as usize;
        unshifted_addr << Addr::ADDRESS.shift
    }

    pub fn set_addr(&mut self, addr: usize) {
        let shft_addr = addr as u32 >> Addr::ADDRESS.shift;
        self.addr.modify(Addr::ADDRESS.val(shft_addr));
    }

    pub fn is_available(&self) -> bool {
        self.addr.is_set(Addr::AVAIL)
    }

    pub fn mark_done(&mut self) {
        self.addr.modify(Addr::AVAIL::CLEAR);
    }

    pub fn mark_last(&mut self) {
        self.addr.modify(Addr::WRAP::SET);
    }
}
