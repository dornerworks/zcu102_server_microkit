//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
use tock_registers::register_bitfields;
use tock_registers::registers::ReadWrite;

// Only support 32-bit addressing for now
#[repr(C)]
pub struct Descriptor {
    addr: u32,
    status: ReadWrite<u32, Status::Register>,
}

register_bitfields![u32,
    Status [
        USED OFFSET(31) NUMBITS(1) [],
        WRAP OFFSET(30) NUMBITS(1) [],
        RETRY_LIMIT OFFSET(29) NUMBITS(1) [],
        AXI_ERR OFFSET(27) NUMBITS(1) [],
        LATE_COLLISION OFFSET(26) NUMBITS(1) [],
        // Only used for Extended Buff Desc Mode
        TIMESTAMP_CAPTURED OFFSET(23) NUMBITS(1) [],
        CHKSUM_GEN_ERR OFFSET(20) NUMBITS(3) [
            NoErr = 0b000,
            VlanErr = 0b001,
            SnapErr = 0b010,
            IpErr = 0b011,
            Unidentified = 0b100,
            BadPktFrag = 0b101,
            TcpUdpErr = 0b110,
            PrematurePktEnd = 0b111,
        ],
        NO_CRC OFFSET(16) NUMBITS(1) [],
        FRAME_END OFFSET(15) NUMBITS(1) [],
        LEN OFFSET(0) NUMBITS(14) [],
    ]
];

impl Descriptor {
    pub fn set_addr(&mut self, addr: usize) {
        self.addr = addr as u32;
    }

    pub fn is_available(&self) -> bool {
        self.status.is_set(Status::USED)
    }

    pub fn mark_gem_owned(&mut self) {
        self.status.modify(Status::USED::CLEAR);
    }

    pub fn mark_sw_owned(&mut self) {
        self.status.modify(Status::USED::SET);
    }

    pub fn clear_status(&mut self) {
        let mut update = Status::LEN::CLEAR;
        if self.status.is_set(Status::USED) {
            update += Status::USED::SET;
        }
        if self.status.is_set(Status::WRAP) {
            update += Status::WRAP::SET;
        }
        // Clear everything but USED and WRAP bits
        self.status.write(update);
    }

    pub fn set_len(&mut self, len: usize) {
        self.status.modify(Status::LEN.val(len as u32));
    }

    pub fn mark_frame_end(&mut self) {
        self.status.modify(Status::FRAME_END::SET);
    }

    pub fn mark_last(&mut self) {
        self.status.modify(Status::WRAP::SET);
    }
}
