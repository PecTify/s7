// Copyright 2019 Petar Dambovaliev. All rights reserved.
// This software may be modified and distributed under the terms
// of the BSD license. See the LICENSE file for details.

//! Transport definition for PLC

use super::constant;
use super::error::Error;

/// Client Connection Type
/// 16 possible connections limited by the hardware
/// The types are defined from the highest to lowest priority
/// The basic connections are the first which would be closed
/// if there aren't enough resources
#[derive(Debug, Copy, Clone)]
pub enum Connection {
    /// Connect to the PLC programming console (Programmiergeräte). German for programming device.
    PG = 1,
    /// Connect to the PLC Siemens HMI panel
    OP = 2,
    /// Basic connection for generic data transfer connection
    /// 14 Basic connections
    Basic = 3,
}

/// an abstract communication used by the client to send requests
/// ## How can I implement `Transport`?
///
/// Types that are [`Transport`] should store the `pdu_length`
/// at the connection phase `self.pdu_length = BigEndian::read_u16(&response[25..]) as i32;`
pub trait Transport {
    /// send request to the plc.
    /// returns a response and an error, if there was any.
    fn send(&mut self, request: &[u8]) -> Result<Vec<u8>, Error>;
    /// pdu length needs to be set by the implementor, during the connection phase.
    fn pdu_length(&self) -> i32;
    /// negotiate is called by the client and should only be defined by the implementor
    fn negotiate(&mut self) -> Result<(), Error>;

    fn connection_type(&self) -> Connection;
}

/// response from the plc that the connection has been confirmed
pub const CONFIRM_CONNECTION: u8 = 0xD0;

/// ISO Connection Request telegram (contains also ISO Header and COTP Header)
/// TPKT (RFC1006 Header)
pub const ISO_CONNECTION_REQUEST_TELEGRAM: [u8; 22] = [
    3,  // RFC 1006 ID (3)
    0,  // Reserved, always 0
    0,  // High part of packet lenght (entire frame, payload and TPDU included)
    22, // Low part of packet lenght (entire frame, payload and TPDU included)
    // COTP (ISO 8073 Header)
    17,  // PDU Size Length
    224, // CR - Connection Request ID
    0,   // Dst Reference HI
    0,   // Dst Reference LO
    0,   // Src Reference HI
    1,   // Src Reference LO
    0,   // Class + Options Flags
    192, // PDU Max Length ID
    1,   // PDU Max Length HI
    10,  // PDU Max Length LO
    193, // Src TSAP Identifier
    2,   // Src TSAP Length (2 bytes)
    1,   // Src TSAP HI (will be overwritten)
    0,   // Src TSAP LO (will be overwritten)
    194, // Dst TSAP Identifier
    2,   // Dst TSAP Length (2 bytes)
    1,   // Dst TSAP HI (will be overwritten)
    2,
]; // Dst TSAP LO (will be overwritten)

/// S7 Read/Write Request Header (contains also ISO Header and COTP Header)
pub const READ_WRITE_TELEGRAM: [u8; 35] = [
    // 31-35 bytes
    3,
    0,
    0,
    31, // Telegram Length (Data Size + 31 or 35)
    2,
    240,
    128, // COTP (see above for info)
    50,  // S7 Protocol ID
    1,   // Job Type
    0,
    0, // Redundancy identification
    5,
    0, // PDU Reference //lth this use for request S7 packet id
    0,
    14, // Parameters Length
    0,
    0,                       // Data Length = Size(bytes) + 4
    4,                       // Function 4 Read Var, 5 Write Var
    1,                       // Items count
    18,                      // Var spec.
    10,                      // Length of remaining bytes
    16,                      // Syntax ID
    constant::WL_BYTE as u8, // Transport Size idx=22
    0,
    0, // Num Elements
    0,
    0,   // DB Number (if any, else 0)
    132, // Area Type
    0,
    0,
    0, // Area Offset
    // WR area
    0, // Reserved
    4, // Transport size
    0,
    0,
]; // Data Length * 8 (if not bit or timer or counter)

// used during establishing a connection
pub const PDU_NEGOTIATION_TELEGRAM: [u8; 25] = [
    3, 0, 0, 25, 2, 240, 128, // TPKT + COTP (see above for info)
    50, 1, 0, 0, 4, 0, 0, 8, 0, 0, 240, 0, 0, 1, 0, 1, 0, 30,
]; // PDU Length Requested = HI-LO Here Default 480 bytes

/// warm start request
pub(crate) const WARM_START_TELEGRAM: [u8; 37] = [
    3, 0, 0, 37, 2, 240, 128, 50, 1, 0, 0, 12, 0, 0, 20, 0, 0, 40, 0, 0, 0, 0, 0, 0, 253, 0, 0, 9,
    80, 95, 80, 82, 79, 71, 82, 65, 77,
];

/// cold start request
pub(crate) const COLD_START_TELEGRAM: [u8; 39] = [
    3, 0, 0, 39, 2, 240, 128, 50, 1, 0, 0, 15, 0, 0, 22, 0, 0, 40, 0, 0, 0, 0, 0, 0, 253, 0, 2, 67,
    32, 9, 80, 95, 80, 82, 79, 71, 82, 65, 77,
];

/// stop request
pub(crate) const STOP_TELEGRAM: [u8; 33] = [
    3, 0, 0, 33, 2, 240, 128, 50, 1, 0, 0, 14, 0, 0, 16, 0, 0, 41, 0, 0, 0, 0, 0, 9, 80, 95, 80,
    82, 79, 71, 82, 65, 77,
];

/// get plc status telegram
pub(crate) const PLC_STATUS_TELEGRAM: [u8; 33] = [
    3, 0, 0, 33, 2, 240, 128, 50, 7, 0, 0, 44, 0, 0, 8, 0, 8, 0, 1, 18, 4, 17, 68, 1, 0, 255, 9, 0,
    4, 4, 36, 0, 0,
];

pub(crate) const SZL_FIRST_TELEGRAM: [u8; 33] = [
    3, 0, 0, 33, 2, 240, 128, 50, 7, 0, 0, 5, 0, // Sequence out
    0, 8, 0, 8, 0, 1, 18, 4, 17, 68, 1, 0, 255, 9, 0, 4, 0, 0, // ID (29)
    0, 0,
]; // Index (31)];

pub(crate) const MIN_SZL_FIRST_TELEGRAM: usize = 42;

pub(crate) const SZL_NEXT_TELEGRAM: [u8; 33] = [
    3, 0, 0, 33, 2, 240, 128, 50, 7, 0, 0, 6, 0, 0, 12, 0, 4, 0, 1, 18, 8, 18, 68, 1,
    1, // Sequence
    0, 0, 0, 0, 10, 0, 0, 0,
]; // Index (31)];

pub(crate) const BLOCK_INFO_TELEGRAM: [u8; 37] = [
    0x03, 0x00, 0x00, 0x25,
	0x02, 0xf0, 0x80, 0x32,
	0x07, 0x00, 0x00, 0x05,
	0x00, 0x00, 0x08, 0x00,
	0x0c, 0x00, 0x01, 0x12,
	0x04, 0x11, 0x43, 0x03,
	0x00, 0xff, 0x09, 0x00,
	0x08, 0x30,
	0x41, // Block Type
	0x30, 0x30, 0x30, 0x30, 0x30, // ASCII Block Number
	0x41
];

pub(crate) const BLOCK_LIST_TELEGRAM: [u8; 29] = [
    0x03, 0x00, 0x00, 0x1d,
    0x02, 0xf0, 0x80, 0x32,
    0x07, 0x00, 0x00, 0x18,
    0x00, 0x00, 0x08, 0x00,
    0x04, 0x00, 0x01, 0x12,
    0x04, 0x11, 0x43, 0x01,
    0x00, 0x0a, 0x00, 0x00,
    0x00
];

// S7 Variable MultiRead Header
pub(crate) const MRD_HEADER: [u8; 19] = [
    0x03,0x00,
    0x00,0x1f,       // Telegram Length 
    0x02,0xf0, 0x80, // COTP (see above for info)
    0x32,            // S7 Protocol ID 
    0x01,            // Job Type
    0x00,0x00,       // Redundancy identification
    0x00,0x01,       // PDU Reference
    0x00,0x0e,       // Parameters Length
    0x00,0x00,       // Data Length = Size(bytes) + 4      
    0x04,            // Function 4 Read Var, 5 Write Var  
    0x01             // Items count (idx 18)
];

// S7 Variable MultiRead Item
pub(crate) const MRD_ITEM: [u8; 12] = [
    0x12,            // Var spec.
    0x0a,            // Length of remaining bytes
    0x10,            // Syntax ID 
    0x02,            // Transport Size idx=3                   
    0x00,0x00,       // Num Elements                          
    0x00,0x00,       // DB Number (if any, else 0)            
    0x84,            // Area Type                            
    0x00,0x00,0x00   // Area Offset   
];

// S7 Variable MultiWrite Header
//Todo implement multi write
#[allow(dead_code)]
pub(crate) const MWR_HEADER: [u8; 19] = [
    0x03,0x00,
    0x00,0x1f,       // Telegram Length 
    0x02,0xf0, 0x80, // COTP (see above for info)
    0x32,            // S7 Protocol ID 
    0x01,            // Job Type
    0x00,0x00,       // Redundancy identification
    0x05,0x00,       // PDU Reference
    0x00,0x0e,       // Parameters Length (idx 13)
    0x00,0x00,       // Data Length = Size(bytes) + 4 (idx 15)     
    0x05,            // Function 5 Write Var  
    0x01    
];

// S7 Variable MultiWrite Item (Param)
//Todo implement multi write
#[allow(dead_code)]
pub(crate) const MWR_PARAM: [u8; 12] = [
    0x12,            // Var spec.
    0x0a,            // Length of remaining bytes
    0x10,            // Syntax ID 
    0x02,  // Transport Size idx=3                      
    0x00,0x00,       // Num Elements                          
    0x00,0x00,       // DB Number (if any, else 0)            
    0x84,            // Area Type                            
    0x00,0x00,0x00,  // Area Offset     
];

pub(crate) const PLC_STATUS_MIN_RESPONSE: usize = 45;

pub(crate) const TELEGRAM_MIN_RESPONSE: usize = 19;

pub(crate) const BLOCK_INFO_TELEGRAM_MIN_RESPONSE: usize = 33;

pub(crate) const BLOCK_LIST_TELEGRAM_MIN_RESPONSE: usize = 61;

pub(crate) const SZL_MIN_RESPONSE: usize = 205;

pub(crate) const PDU_START: u8 = 0x28; // CPU start
pub(crate) const PDU_STOP: u8 = 0x29; // CPU stop

pub(crate) const PDU_ALREADY_STARTED: u8 = 0x02; // CPU already in run mode
pub(crate) const PDU_ALREADY_STOPPED: u8 = 0x07; // CPU already in stop mode

pub(crate) const MAX_VARS_MULTI_READ_WRITE: usize = 20;


pub struct SZLHeader {
    pub length_header: u16,
    pub number_of_data_record: u16,
}

pub(crate) struct S7SZL {
    pub header: SZLHeader,
    pub data: Vec<u8>,
}
