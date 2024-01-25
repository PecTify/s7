// Copyright 2019 Petar Dambovaliev. All rights reserved.
// This software may be modified and distributed under the terms
// of the BSD license. See the LICENSE file for details.

use super::constant::{self, Area};
use super::error::{self, Error};
use super::transport::{self, Transport};
use crate::constant::{CpuStatus, BlockLang, SubBlockType, TS_RES_OCTET, TS_RES_REAL, TS_RES_BIT, WL_BIT, WL_COUNTER, WL_TIMER, TS_RES_BYTE};
use crate::field::{Word, DInt, to_chars, siemens_timestamp};
use crate::transport::{BLOCK_INFO_TELEGRAM, BLOCK_INFO_TELEGRAM_MIN_RESPONSE, BLOCK_LIST_TELEGRAM, BLOCK_LIST_TELEGRAM_MIN_RESPONSE, MAX_VARS_MULTI_READ_WRITE, MRD_HEADER, MRD_ITEM, MWR_HEADER, MWR_PARAM};
use byteorder::{BigEndian, ByteOrder};
use chrono::NaiveDateTime;
use std::str;

#[derive(Debug, Clone)]
pub struct S7DataItem {
    pub area: u8,
    pub word_len: u8,
    pub db_num: u16,
    pub start: u16,
    pub size: u16,
    pub buffer: Vec<u8>,
    pub err: Option<Error>,
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub module_type_name: String,
    pub serial_number: String,
    pub as_name: String,
    pub copyright: String,
    pub module_name: String,
}

#[derive(Debug, Clone)]
pub struct BlockList {
    pub ob_block_count: u16,
    pub fb_block_count: u16,
    pub fc_block_count: u16,
    pub db_block_count: u16,
    pub sdb_block_count: u16,
    pub sfc_block_count: u16,
    pub sfb_block_count: u16,
}

pub enum BlockType {
    OB = 0x38,
    DB = 0x41,
    SDB = 0x42,
    FC = 0x43,
    SFC = 0x44,
    FB = 0x45,
    SFB = 0x46,
}

#[derive(Debug)]
pub struct S7BlockInfo {
    pub block_type: SubBlockType, //Block Type (see SubBlkType)
    pub block_number: u16, //Block number
    pub block_lang: BlockLang, //Block Language (see BlockLang)
    pub block_flags: u8, //Block flags (bitmapped)
    pub mc7_size: u16, //The real size in bytes
    pub load_size: i32, //Load memory size
    pub local_data: u16, //Local data
    pub sbb_length: u16, //SBB Length
    pub version: u8, // Version (BCD 00<HI><LO>)
    pub code_date: NaiveDateTime,
    pub interface_date: NaiveDateTime,
    pub author: String,
    pub family: String,
    pub header: String,

}

#[derive(Debug, Clone)]
pub struct CPInfo {
    pub max_pdu_length: u16,
    pub max_connections: u16,
    pub max_mpi_rate: u16,
    pub max_bus_rate: u16,
}

#[derive(Debug, Clone)]
pub struct Client<T: Transport> {
    transport: T,
}

impl<T: Transport> Client<T> {
    pub fn new(mut transport: T) -> Result<Client<T>, Error> {
        transport.negotiate()?;
        Ok(Client { transport })
    }

    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, IpAddr};
    /// use s7::{client, tcp, transport};
    /// use std::time::Duration;
    /// use s7::field::{Bool, Field};
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// let mut opts = tcp::Options::new(IpAddr::from(addr), 0, 5, 5, transport::Connection::PG);
    ///
    /// opts.read_timeout = Duration::from_secs(2);
    /// opts.write_timeout = Duration::from_secs(2);
    ///
    /// let t = tcp::Transport::connect(opts).unwrap();
    /// let mut cl = client::Client::new(t).unwrap();
    ///
    /// let buffer = &mut vec![0u8; Bool::size() as usize];
    /// let db = 888;
    /// let offset = 8.4;
    ///
    /// cl.ag_read(db, offset as i32, Bool::size(), buffer).unwrap();
    ///
    /// let mut  lights = Bool::new(db, offset, buffer.to_vec()).unwrap();
    /// lights.set_value(!lights.value()); // toggle the light switch
    ///
    /// // save
    /// cl.ag_write(
    ///     lights.data_block(),
    ///     lights.offset(),
    ///     Bool::size(),
    ///     lights.to_bytes().as_mut()
    /// ).unwrap();
    ///
    /// ```
    pub fn ag_read(
        &mut self,
        db_number: i32,
        start: i32,
        size: i32,
        buffer: &mut Vec<u8>,
    ) -> Result<(), Error> {
        self.read(
            Area::DataBausteine,
            db_number,
            start,
            size,
            constant::WL_BYTE,
            buffer,
        )
    }

    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, IpAddr};
    /// use s7::{client, tcp, transport};
    /// use std::time::Duration;
    /// use s7::field::{Bool, Field};
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// let mut opts = tcp::Options::new(IpAddr::from(addr),0, 5, 5, transport::Connection::PG);
    ///
    /// opts.read_timeout = Duration::from_secs(2);
    /// opts.write_timeout = Duration::from_secs(2);
    ///
    /// let t = tcp::Transport::connect(opts).unwrap();
    /// let mut cl = client::Client::new(t).unwrap();
    ///
    /// let buffer = &mut vec![0u8; Bool::size() as usize];
    /// let db = 888;
    /// let offset = 8.4;
    ///
    /// cl.ag_read(db, offset as i32, Bool::size(), buffer).unwrap();
    ///
    /// let mut  lights = Bool::new(db, offset, buffer.to_vec()).unwrap();
    /// lights.set_value(!lights.value()); // toggle the light switch
    ///
    /// // save
    /// cl.ag_write(
    ///     lights.data_block(),
    ///     lights.offset(),
    ///     Bool::size(),
    ///     lights.to_bytes().as_mut()
    /// ).unwrap();
    ///
    /// ```
    pub fn ag_write(
        &mut self,
        db_number: i32,
        start: i32,
        size: i32,
        buffer: &mut Vec<u8>,
    ) -> Result<(), Error> {
        self.write(
            Area::DataBausteine,
            db_number,
            start,
            size,
            constant::WL_BYTE,
            buffer,
        )
    }

    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, IpAddr};
    /// use s7::{client, tcp, transport};
    /// use std::time::Duration;
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// let mut opts = tcp::Options::new(IpAddr::from(addr),0, 5, 5, transport::Connection::PG);
    ///
    /// opts.read_timeout = Duration::from_secs(2);
    /// opts.write_timeout = Duration::from_secs(2);
    ///
    ///
    /// let t = tcp::Transport::connect(opts).unwrap();
    /// let mut cl = client::Client::new(t).unwrap();
    ///
    /// let buffer = &mut vec![0u8; 255];
    ///
    /// cl.mb_read(1, 3, buffer).unwrap();
    /// ```
    pub fn mb_read(&mut self, start: i32, size: i32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        self.read(Area::Merker, 0, start, size, constant::WL_BYTE, buffer)
    }

    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, IpAddr};
    /// use s7::{client, tcp, transport};
    /// use std::time::Duration;
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// let mut opts = tcp::Options::new(IpAddr::from(addr),0, 5, 5, transport::Connection::PG);
    ///
    /// opts.read_timeout = Duration::from_secs(2);
    /// opts.write_timeout = Duration::from_secs(2);
    ///
    ///
    /// let t = tcp::Transport::connect(opts).unwrap();
    /// let mut cl = client::Client::new(t).unwrap();
    ///
    /// let buffer = &mut vec![0u8; 255];
    ///
    /// cl.mb_write(1, 3, buffer).unwrap();
    /// ```
    pub fn mb_write(&mut self, start: i32, size: i32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        self.write(Area::Merker, 0, start, size, constant::WL_BYTE, buffer)
    }

    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, IpAddr};
    /// use s7::{client, tcp, transport};
    /// use std::time::Duration;
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// let mut opts = tcp::Options::new(IpAddr::from(addr),0, 5, 5, transport::Connection::PG);
    ///
    /// opts.read_timeout = Duration::from_secs(2);
    /// opts.write_timeout = Duration::from_secs(2);
    ///
    ///
    /// let t = tcp::Transport::connect(opts).unwrap();
    /// let mut cl = client::Client::new(t).unwrap();
    ///
    /// let buffer = &mut vec![0u8; 255];
    ///
    /// cl.eb_read(1, 3, buffer).unwrap();
    /// ```
    pub fn eb_read(&mut self, start: i32, size: i32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        self.read(
            Area::ProcessInput,
            0,
            start,
            size,
            constant::WL_BYTE,
            buffer,
        )
    }

    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, IpAddr};
    /// use s7::{client, tcp, transport};
    /// use std::time::Duration;
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// let mut opts = tcp::Options::new(IpAddr::from(addr),0, 5, 5, transport::Connection::PG);
    ///
    /// opts.read_timeout = Duration::from_secs(2);
    /// opts.write_timeout = Duration::from_secs(2);
    ///
    ///
    /// let t = tcp::Transport::connect(opts).unwrap();
    /// let mut cl = client::Client::new(t).unwrap();
    ///
    /// let buffer = &mut vec![0u8; 255];
    ///
    /// cl.eb_write(1, 3, buffer).unwrap();
    /// ```
    pub fn eb_write(&mut self, start: i32, size: i32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        self.write(
            Area::ProcessInput,
            0,
            start,
            size,
            constant::WL_BYTE,
            buffer,
        )
    }

    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, IpAddr};
    /// use s7::{client, tcp, transport};
    /// use std::time::Duration;
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// let mut opts = tcp::Options::new(IpAddr::from(addr),0, 5, 5, transport::Connection::PG);
    ///
    /// opts.read_timeout = Duration::from_secs(2);
    /// opts.write_timeout = Duration::from_secs(2);
    ///
    ///
    /// let t = tcp::Transport::connect(opts).unwrap();
    /// let mut cl = client::Client::new(t).unwrap();
    ///
    /// let buffer = &mut vec![0u8; 255];
    ///
    /// cl.ab_read(1, 3, buffer).unwrap();
    /// ```
    pub fn ab_read(&mut self, start: i32, size: i32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        self.read(
            Area::ProcessOutput,
            0,
            start,
            size,
            constant::WL_BYTE,
            buffer,
        )
    }

    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, IpAddr};
    /// use s7::{client, tcp, transport};
    /// use std::time::Duration;
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// let mut opts = tcp::Options::new(IpAddr::from(addr), 0, 5, 5, transport::Connection::PG);
    ///
    /// opts.read_timeout = Duration::from_secs(2);
    /// opts.write_timeout = Duration::from_secs(2);
    ///
    ///
    /// let t = tcp::Transport::connect(opts).unwrap();
    /// let mut cl = client::Client::new(t).unwrap();
    ///
    /// let buffer = &mut vec![0u8; 255];
    ///
    /// cl.ab_write(1, 3, buffer).unwrap();
    /// ```
    pub fn ab_write(&mut self, start: i32, size: i32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        self.write(
            Area::ProcessOutput,
            0,
            start,
            size,
            constant::WL_BYTE,
            buffer,
        )
    }

    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, IpAddr};
    /// use s7::{client, tcp, transport};
    /// use std::time::Duration;
    /// use s7::constant;
    /// use s7::constant::Area;
    /// use s7::client::S7DataItem;
    /// use s7::field::Word;
    /// use s7::field::Bool;
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// let mut opts = tcp::Options::new(IpAddr::from(addr), 0, 5, 5, transport::Connection::PG);
    ///
    /// opts.read_timeout = Duration::from_secs(2);
    /// opts.write_timeout = Duration::from_secs(2);
    ///
    ///
    /// let t = tcp::Transport::connect(opts).unwrap();
    /// let mut cl = client::Client::new(t).unwrap();
    ///
    /// let mut items: Vec<S7DataItem> = vec![
    /// S7DataItem{area: Area::DataBausteine as u8,word_len:2,db_num:88,start:0,size:8,buffer:vec![0u8; Bool::size() as usize], err: Ok(()) },
    /// S7DataItem{area: Area::ProcessInput as u8,word_len: constant::WL_BYTE as u8,db_num:0,start:0,size:1,buffer:vec![0u8; Bool::size() as usize], err: Ok(()) },
    /// S7DataItem{area: Area:: ProcessOutput as u8,word_len:constant::WL_BYTE as u8,db_num:0,start:0,size:1,buffer:vec![0u8; Bool::size() as usize], err: Ok(()) },
    /// S7DataItem{area: Area::Merker as u8,word_len: constant::WL_BYTE as u8,db_num:0,start:3,size:1,buffer:vec![0u8; Bool::size() as usize], err: Ok(()) },
    /// S7DataItem{area:Area::Counter as u8,word_len: constant::WL_COUNTER as u8,db_num:0,start:0,size:1,buffer:vec![0u8; Word::size() as usize], err: Ok(()) },
    /// S7DataItem{area:Area::Timer as u8,word_len: constant::WL_TIMER as u8,db_num:0,start:0,size:1,buffer:vec![0u8; Word::size() as usize], err: Ok(()) },
    /// ];
    ///     
    /// 
    ///
    /// cl.read_multi_vars(&mut items).unwrap();
    /// for item in items {
    ///     println!("{:08b}", item.buffer[0]);
    /// }
    /// ```
    pub fn read_multi_vars(&mut self, items: &mut Vec<S7DataItem>) -> Result<(), Error>{
        let item_len = items.len();

        if item_len > MAX_VARS_MULTI_READ_WRITE {
            return Err(Error::InvalidInput { input: "Too many items".to_string() });
        }

        let mut s7_item = vec![0u8; 12];
        let mut s7_item_read;
        let mut item_size: u16 = 0;

        //Fill Header
        let mut request = MRD_HEADER.to_vec();
        let header_bytes = ((item_len * s7_item.len() + 2) as u16).to_be_bytes();
        request[13] = header_bytes[0];
        request[14] = header_bytes[1];
        request[18] = item_len as u8;

        //Fill the Items
        let mut offset: u16 = 19;

        for (_c, item) in items.iter().enumerate() {
            s7_item = MRD_ITEM.to_vec();
            s7_item[3] = item.word_len;
    
            //Size
            let size_bytes = (item.size).to_be_bytes();
            s7_item[4] = size_bytes[0];
            s7_item[5] = size_bytes[1];
    
            //DB number
            let db_bytes = (item.db_num).to_be_bytes();
            s7_item[6] = db_bytes[0];
            s7_item[7] = db_bytes[1];
    
            //Area
            s7_item[8] = item.area;
    
             // Adjusts Start and Word length
             let mut address = match item.word_len as i32 {
                constant::WL_BIT | constant::WL_COUNTER | constant::WL_TIMER => {
                    s7_item[3] = item.word_len;
                    item.start
                }
                _ => item.start << 3,
            };
    
            // Address into the PLC
            s7_item[11] = (address & 0x0FF) as u8;
            address >>= 8;
            s7_item[10] = (address & 0x0FF) as u8;
            address >>= 8;
            s7_item[9] = (address & 0x0FF) as u8;
    
            
            request.append(&mut s7_item);
            item_size += MRD_ITEM.len() as u16;
        }

        //Request Size
        offset += item_size;
        let request_size = (offset).to_be_bytes();
        request[2] = request_size[0];
        request[3] = request_size[1];

        let response = self.transport.send(request.as_slice())?;

        //PDU too small?
        if response.len() < 22 { 
            return Err(Error::InvalidResponse { reason: "PDU too small".to_string(), bytes: response } );
        }

        let error_code = Word::new(0, 0.0, response[17..19].to_vec())?.value();
        
        if error_code != 0 {
            return Err(Error::CPU { code: error_code as i32 });
        }

        //Check item count
        let items_read = response[20];
        if items_read != item_len as u8 || items_read > MAX_VARS_MULTI_READ_WRITE as u8 {
            return Err(Error::InvalidResponse { reason: "Recived Items to large".to_string(), bytes: response })
        }

        let mut offset = 21;

        for (_c, item) in items.iter_mut().enumerate().take(items_read as usize) {
            //Get Item
            s7_item_read = response[offset..response.len()].to_vec();

            //Check Error Byte  0xff = success
            if s7_item_read[0] == 0xff {
                let mut item_size = Word::new(0, 0.0, s7_item_read[2..4].to_vec())?.value();

                if s7_item_read[1] != TS_RES_OCTET && s7_item_read[1] != TS_RES_REAL && s7_item_read[1] != TS_RES_BIT {
                    item_size >>= 3;
                }

                item.buffer = s7_item_read[4..4 + item_size as usize].to_vec();

                    if item_size % 2 != 0 {
                        item_size += 1;
                    }

                    offset = offset + 4 + item_size as usize;
            } else {
                item.err = Some(Error::CPU { code: s7_item_read[0] as i32 });
                //Skip Item (headersize)
                offset += 4;
            }
        }
        Ok(())
    }

    fn write_word_at(start: usize, source: &[u8; 2], destination: &mut Vec<u8>) {
        destination[start] = source[0];
        destination[start+1] = source[1];
    }

    pub fn write_multi_vars(&mut self, items: &mut Vec<S7DataItem>) -> Result<(), Error>{
        let item_count = items.len();

        if item_count > MAX_VARS_MULTI_READ_WRITE {
            return Err(Error::InvalidInput { input: "Too many items".to_string() });
        }

        //Fill Header
        let mut request = MWR_HEADER.to_vec();

        let par_length: i16 = item_count as i16 * MRD_HEADER.len() as i16 + 2;
        Self::write_word_at(13, &par_length.to_be_bytes(), &mut request);
        request[18] = item_count as u8;
        

        //Fill Params
        let mut offset = MWR_HEADER.len();
        
        let mut s7_par_item;
        for item in items.clone() {
            s7_par_item = MWR_PARAM;
            s7_par_item[3] = item.word_len;
            s7_par_item[8] = item.area;

            let size_bytes = (item.size).to_be_bytes();
            s7_par_item[4] = size_bytes[0];
            s7_par_item[5] = size_bytes[1];

            let db_num_bytes = (item.db_num).to_be_bytes();
            s7_par_item[6] = db_num_bytes[0];
            s7_par_item[7] = db_num_bytes[1];

            //Address into PLC
            let mut address = item.start;
            s7_par_item[11] = (address & 0x0FF) as u8;
            address = address >> 8;
            s7_par_item[10] = (address & 0x0FF) as u8;
            address = address >> 8;
            s7_par_item[9] = (address & 0x0FF) as u8;

            request.append(&mut s7_par_item.to_vec());

            offset += MWR_PARAM.len();
        }

        //Fills Data
        // start data section -->
        let mut data_length = 0;
        for item in items.clone() {
            let mut s7_data_item = vec![0; 6]; //20 <--- !TODO
            
            s7_data_item[0] = 0x00;
            match item.word_len as i32 {
                WL_BIT => s7_data_item[1] = TS_RES_BIT,
                WL_COUNTER | WL_TIMER => s7_data_item[1] = TS_RES_OCTET,
                _ => s7_data_item[1] = TS_RES_BYTE,
            }

            let mut item_data_size;
            if item.word_len == WL_TIMER as u8 || item.word_len == WL_COUNTER as u8 {
                item_data_size = item.size * 2;
            } else {
                item_data_size = item.size;
            }
            

            if s7_data_item[1] !=  TS_RES_OCTET && s7_data_item[1] != TS_RES_BIT {
                let item_data_size_bytes = (item_data_size * 8).to_be_bytes();
                s7_data_item[2] = item_data_size_bytes[0];
                s7_data_item[3] = item_data_size_bytes[1];
            } else {
                let item_data_size_bytes = (item_data_size).to_be_bytes();
                s7_data_item[2] = item_data_size_bytes[0];
                s7_data_item[3] = item_data_size_bytes[1];
            }

            for (c, item) in item.buffer.iter().enumerate() {
                s7_data_item[c+4] = item.clone();
            }

            if item_data_size % 2 != 0 {
                s7_data_item[item_data_size as usize + 4 ] = 0x00;
                item_data_size += 1;
            } //<-- end datasection

           
            request.append(&mut s7_data_item);
            offset = offset + item_data_size as usize + 4;
            data_length = data_length + item_data_size + 4;
        }
        //Check the size
        let pdu_length = self.transport.pdu_length();
        if offset > pdu_length as usize {
            return Err(Error::PduLength(pdu_length));
        }
        let offset_bytes = (offset).to_be_bytes();
        request[2] = offset_bytes[6];
        request[3] = offset_bytes[7];

        let data_length_bytes = (data_length).to_be_bytes();
        request[15] = data_length_bytes[0];
        request[16] = data_length_bytes[1];
        
        let response = self.transport.send(request.as_slice())?;

        // Check Global Operation Result
        let global_operation_result = Word::new(0, 0.0, response[17..19].to_vec())?.value();
        if global_operation_result != 0  {
            return Err(Error::CPU { code: global_operation_result as i32 });
        }

        // Get true ItemCount
        let items_written = response[20] as usize;
        if item_count != items_written {
            return Err(Error::InvalidResponse { reason: "items_written does not match item_count".to_string(), bytes: response })
        }
        if items_written > MAX_VARS_MULTI_READ_WRITE {
            return Err(Error::InvalidResponse { reason: "items_written is larger than MAX_VARS ".to_string(), bytes: response })
        }

        //todo!()

        Ok(())
    }

    //read generic area, pass result into a buffer
    fn read(
        &mut self,
        area: Area,
        db_number: i32,
        mut start: i32,
        mut amount: i32,
        mut word_len: i32,
        buffer: &mut Vec<u8>,
    ) -> Result<(), Error> {
        // Some adjustment
        match area {
            Area::Counter => word_len = constant::WL_COUNTER,
            Area::Timer => word_len = constant::WL_TIMER,
            _ => {}
        };

        // Calc Word size
        let mut word_size = constant::data_size_byte(word_len);

        if word_size == 0 {
            return Err(Error::Response {
                code: error::ISO_INVALID_DATA_SIZE,
            });
        }

        if word_len == constant::WL_BIT {
            amount = 1; // Only 1 bit can be transferred at time
        } else if word_len != constant::WL_COUNTER && word_len != constant::WL_TIMER {
            amount *= word_size;
            word_size = 1;
            word_len = constant::WL_BYTE;
        }

        let pdu_length = self.transport.pdu_length();

        if pdu_length == 0 {
            return Err(Error::PduLength(pdu_length));
        }

        let max_elements = (pdu_length - 18) / word_size; // 18 = Reply telegram header //lth note here

        let mut tot_elements = amount;
        let db_bytes = (db_number as u16).to_be_bytes();
        let mut offset = 0;

        while tot_elements > 0 {
            let mut num_elements = tot_elements;

            if num_elements > max_elements {
                num_elements = max_elements;
            }

            let size_requested = num_elements * word_size;
            // Setup the telegram
            let mut request =
                transport::READ_WRITE_TELEGRAM[..constant::SIZE_HEADER_READ as usize].to_vec();

            // Set DB Number
            request[25] = db_bytes[0];
            request[26] = db_bytes[1];

            // Set Area
            request[27] = area as u8;
            // match area {
            //     Area::DataBausteine => request[27] = area as u8,
            //     _ => {}
            // }

            // Adjusts Start and word length
            let mut address = match word_len {
                constant::WL_BIT | constant::WL_COUNTER | constant::WL_TIMER => {
                    request[22] = word_len as u8;
                    start
                }
                _ => start << 3,
            };

            // Num elements
            let num_elements_bytes = (num_elements as u16).to_be_bytes();
            request[23] = num_elements_bytes[0];
            request[24] = num_elements_bytes[1];

            // Address into the PLC (only 3 bytes)
            request[30] = (address & 0x0FF) as u8;
            address >>= 8;
            request[29] = (address & 0x0FF) as u8;
            address >>= 8;
            request[28] = (address & 0x0FF) as u8;

            let result = self.transport.send(request.as_slice());

            match result {
                Ok(response) => {
                    if response.len() < 25 {
                        return Err(Error::Response {
                            code: error::ISO_INVALID_DATA_SIZE,
                        });
                    }

                    if response[21] != 0xFF {
                        return Err(Error::CPU {
                            code: response[21] as i32,
                        });
                    }
                    let (mut i, end): (usize, usize) = (25, 25 + (size_requested as usize));

                    //copy response to buffer
                    for k in offset..size_requested {
                        if i == end {
                            break;
                        }
                        buffer[k as usize] = response[i];
                        i += 1;
                    }
                    offset += size_requested;
                }
                Err(e) => {
                    return Err(e);
                }
            }

            tot_elements -= num_elements;
            start += num_elements * word_size
        }
        Ok(())
    }

    fn write(
        &mut self,
        area: Area,
        db_number: i32,
        mut start: i32,
        mut amount: i32,
        mut word_len: i32,
        buffer: &mut Vec<u8>,
    ) -> Result<(), Error> {
        // Some adjustment
        word_len = match area {
            Area::Counter => constant::WL_COUNTER,
            Area::Timer => constant::WL_TIMER,
            _ => word_len,
        };

        // Calc Word size
        let mut word_size = constant::data_size_byte(word_len);

        if word_size == 0 {
            return Err(Error::Response {
                code: error::ISO_INVALID_DATA_SIZE,
            });
        }

        if word_len == constant::WL_BIT {
            amount = 1; // Only 1 bit can be transferred at time
        } else if word_len != constant::WL_COUNTER && word_len != constant::WL_TIMER {
            amount *= word_size;
            word_size = 1;
            word_len = constant::WL_BYTE;
        }

        let mut offset: i32 = 0;
        let pdu_length = self.transport.pdu_length();
        let max_elements = (pdu_length - 35) / word_size; // 35 = Reply telegram header
        let mut tot_elements = amount;

        while tot_elements > 0 {
            let mut num_elements = tot_elements;
            if num_elements > max_elements {
                num_elements = max_elements;
            }
            let data_size = num_elements * word_size;
            let iso_size = constant::SIZE_HEADER_WRITE + data_size;

            // Setup the telegram
            let mut request_data = transport::READ_WRITE_TELEGRAM.to_vec();
            // Whole telegram Size
            BigEndian::write_u16(request_data[2..].as_mut(), iso_size as u16);
            // Data length
            let mut length = data_size + 4;
            BigEndian::write_u16(request_data[15..].as_mut(), length as u16);
            // Function
            request_data[17] = 0x05;
            // Set DB Number
            request_data[27] = area as u8;

            
            if let Area::DataBausteine = area {
                BigEndian::write_u16(request_data[25..].as_mut(), db_number as u16)
            }
            // Adjusts start and word length
            let mut address = match word_len {
                constant::WL_BIT | constant::WL_COUNTER | constant::WL_TIMER => {
                    length = data_size;
                    request_data[22] = word_len as u8;
                    start
                }
                _ => {
                    length = data_size << 3;
                    start << 3
                }
            };

            // Num elements
            BigEndian::write_u16(request_data[23..].as_mut(), num_elements as u16);
            // address into the PLC
            request_data[30] = (address & 0x0FF) as u8;
            address >>= 8;
            request_data[29] = (address & 0x0FF) as u8;
            address >>= 8;
            request_data[28] = (address & 0x0FF) as u8;

            // Transport Size
            match word_len {
                constant::WL_BIT => request_data[32] = constant::TS_RES_BIT,
                constant::WL_COUNTER | constant::WL_TIMER => {
                    request_data[32] = constant::TS_RES_OCTET
                }
                _ => request_data[32] = constant::TS_RES_BYTE, // byte/word/dword etc.
            }
            // length
            BigEndian::write_u16(request_data[33..].as_mut(), length as u16);

            //expand values into array
            request_data.splice(
                35..35,
                buffer[offset as usize..offset as usize + data_size as usize].to_vec(),
            );

            let result = self.transport.send(request_data.as_mut_slice());

            match result {
                Ok(response) => {
                    if response.len() != 22 {
                        return Err(Error::Response {
                            code: error::ISO_INVALID_PDU,
                        });
                    }

                    if response[21] != 0xFF {
                        return Err(Error::CPU {
                            code: response[21] as i32,
                        });
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }

            offset += data_size;
            tot_elements -= num_elements;
            start += num_elements * word_size;
        }
        Ok(())
    }
}

impl<T: Transport> Client<T> {
    /// Starting the CPU from power off,Current configuration is discarded and program processing begins again with the initial values.
    pub fn start(&mut self) -> Result<(), Error> {
        self.cold_warm_start_stop(
            transport::COLD_START_TELEGRAM.as_ref(),
            transport::PDU_START,
            error::CLI_CANNOT_START_PLC,
            transport::PDU_ALREADY_STARTED,
            error::CLI_ALREADY_RUN,
        )
    }

    /// Restarting the CPU without turning the power off, Program processing starts once again where Retentive data is retained.
    pub fn restart(&mut self) -> Result<(), Error> {
        self.cold_warm_start_stop(
            transport::WARM_START_TELEGRAM.as_ref(),
            transport::PDU_START,
            error::CLI_CANNOT_START_PLC,
            transport::PDU_ALREADY_STARTED,
            error::CLI_ALREADY_RUN,
        )
    }

    /// Shut down
    pub fn stop(&mut self) -> Result<(), Error> {
        self.cold_warm_start_stop(
            transport::STOP_TELEGRAM.as_ref(),
            transport::PDU_STOP,
            error::CLI_CANNOT_STOP_PLC,
            transport::PDU_ALREADY_STOPPED,
            error::CLI_ALREADY_STOP,
        )
    }

    /// get plc status
    pub fn plc_status(&mut self) -> Result<CpuStatus, Error> {
        let response = self
            .transport
            .send(transport::PLC_STATUS_TELEGRAM.as_ref())?;

        if response.len() < transport::PLC_STATUS_MIN_RESPONSE {
            return Err(Error::Response {
                code: error::ISO_INVALID_PDU,
            });
        }

        let result = BigEndian::read_u16(response[27..29].as_ref());

        if result != 0 {
            return Err(Error::CPU {
                code: result as i32,
            });
        }

        CpuStatus::from_u8(response[44])
    }

    pub fn cp_info(&mut self) -> Result<CPInfo, Error> {
        let szl = self.read_szl(0x0131, 0x000)?;

        Ok(CPInfo {
            max_pdu_length: BigEndian::read_u16(szl.data[2..].as_ref()),
            max_connections: BigEndian::read_u16(szl.data[4..].as_ref()),
            max_mpi_rate: BigEndian::read_u16(szl.data[6..].as_ref()),
            max_bus_rate: BigEndian::read_u16(szl.data[10..].as_ref()),
        })
    }

    /// get cpu info
    pub fn cpu_info(&mut self) -> Result<CpuInfo, Error> {
        let szl = self.read_szl(0x001C, 0x000)?;

        if szl.data.len() < transport::SZL_MIN_RESPONSE {
            return Err(Error::Response {
                code: error::ISO_INVALID_PDU,
            });
        }

        let module_type_name = match str::from_utf8(szl.data[172..204].as_ref()) {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::InvalidResponse {
                    bytes: szl.data[172..204].to_vec(),
                    reason: e.to_string(),
                })
            }
        };

        let serial_number = match str::from_utf8(szl.data[138..162].as_ref()) {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::InvalidResponse {
                    bytes: szl.data[138..162].to_vec(),
                    reason: e.to_string(),
                })
            }
        };

        let as_name = match str::from_utf8(szl.data[2..26].as_ref()) {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::InvalidResponse {
                    bytes: szl.data[2..26].to_vec(),
                    reason: e.to_string(),
                })
            }
        };

        let copyright = match str::from_utf8(szl.data[104..130].as_ref()) {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::InvalidResponse {
                    bytes: szl.data[104..130].to_vec(),
                    reason: e.to_string(),
                })
            }
        };

        let module_name = match str::from_utf8(szl.data[36..60].as_ref()) {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::InvalidResponse {
                    bytes: szl.data[36..60].to_vec(),
                    reason: e.to_string(),
                })
            }
        };

        Ok(CpuInfo {
            module_type_name: module_type_name.to_string(),
            serial_number: serial_number.to_string(),
            as_name: as_name.to_string(),
            copyright: copyright.to_string(),
            module_name: module_name.to_string(),
        })
    }

    fn read_szl(&mut self, id: u16, index: u16) -> Result<transport::S7SZL, Error> {
        let mut offset = 0;
        let seq_out: u16 = 0x0000;

        let mut s7_szlfirst = transport::SZL_FIRST_TELEGRAM.to_vec();

        BigEndian::write_u16(s7_szlfirst[11..].as_mut(), seq_out + 1);
        BigEndian::write_u16(s7_szlfirst[29..].as_mut(), id);
        BigEndian::write_u16(s7_szlfirst[31..].as_mut(), index);

        let mut res = self.transport.send(s7_szlfirst.as_ref())?;

        let validate = |res: &[u8], size: usize| -> Result<(), Error> {
            if res.len() < transport::MIN_SZL_FIRST_TELEGRAM + size {
                return Err(Error::Response {
                    code: error::ISO_INVALID_PDU,
                });
            }

            if BigEndian::read_u16(res[27..].as_ref()) != 0 && res[29] != 0xFF {
                return Err(Error::CPU {
                    code: error::CLI_INVALID_PLC_ANSWER,
                });
            }
            Ok(())
        };

        validate(res.as_ref(), 0)?;

        // Skips extra params (ID, Index ...)
        let mut data_szl = BigEndian::read_u16(res[31..].as_ref()) - 8;

        validate(res.as_ref(), data_szl as usize)?;

        let mut done = res[26] == 0x00;
        // Slice sequence
        let mut seq_in: u8 = res[24];
        let header = transport::SZLHeader {
            length_header: BigEndian::read_u16(res[37..].as_ref()) * 2,
            number_of_data_record: BigEndian::read_u16(res[39..].as_ref()),
        };

        let len = (offset + data_szl) as usize;
        let mut data = vec![0u8; len];

        data[offset as usize..len].copy_from_slice(res[41..41 + data_szl as usize].as_ref());

        let mut szl = transport::S7SZL { header, data };
        offset += data_szl;

        let mut s7szlnext: Vec<u8> = transport::SZL_NEXT_TELEGRAM.to_vec();

        while !done {
            BigEndian::write_u16(s7_szlfirst[11..].as_mut(), seq_out + 1);
            s7szlnext[24] = seq_in;

            res = self.transport.send(s7szlnext.as_ref())?;

            validate(res.as_ref(), 0)?;

            data_szl = BigEndian::read_u16(res[31..].as_ref());
            done = res[26] == 0x00;
            seq_in = res[24];

            szl.data = vec![0u8; len];
            offset += data_szl;
            szl.header.length_header += szl.header.length_header;
        }
        Ok(szl)
    }

    fn cold_warm_start_stop(
        &mut self,
        req: &[u8],
        start_cmp: u8,
        start: i32,
        already_cmp: u8,
        already: i32,
    ) -> Result<(), Error> {
        let response = self.transport.send(req)?;

        if response.len() < transport::TELEGRAM_MIN_RESPONSE {
            return Err(Error::Response {
                code: error::ISO_INVALID_PDU,
            });
        }

        if response[19] != start_cmp {
            return Err(Error::Response { code: start });
        }
        if response[18] == already_cmp {
            return Err(Error::Response { code: already });
        }
        Ok(())
    }

     /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, IpAddr};
    /// use s7::{client, tcp, transport, client::BlockType};
    /// use std::time::Duration;
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// let mut opts = tcp::Options::new(IpAddr::from(addr),0, 5, 5, transport::Connection::PG);
    ///
    /// opts.read_timeout = Duration::from_secs(2);
    /// opts.write_timeout = Duration::from_secs(2);
    ///
    ///
    /// let t = tcp::Transport::connect(opts).unwrap();
    /// let mut cl = client::Client::new(t).unwrap();
    ///
    /// let result = cl.get_ag_block_info(BlockType::DB, 888).unwrap();
    /// 
    /// println!("{:#?}", result);
    /// ```
    pub fn get_ag_block_info(&mut self, block_type: BlockType, mut block_number: u32) -> Result<S7BlockInfo, Error> {
        
        let mut s7_bi = BLOCK_INFO_TELEGRAM;

         // Block Type
        s7_bi[30] = block_type as u8;

        //Blocknumber
        s7_bi[31] = ((block_number / 10000) + 0x30) as u8;
        block_number %= 10000;
        s7_bi[32] = ((block_number / 1000) + 0x30) as u8;
        block_number %= 1000;
        s7_bi[33] = ((block_number / 100) + 0x30) as u8;
        block_number %= 100;
        s7_bi[34] = ((block_number / 10) + 0x30) as u8;
        block_number %= 10;
        s7_bi[35] = (block_number + 0x30) as u8;
        

        let response = self.transport.send(&s7_bi)?;
        if response.len() < BLOCK_INFO_TELEGRAM_MIN_RESPONSE {
            return Err(Error::Response {
                code: error::ISO_INVALID_PDU,
            });
        }

        //Error code |  0 = no error
        let response_error = Word::new(0, 0.0, response[27..29].to_vec())?.value();
        if response_error != 0 {
            return Err(Error::CPU { code: response_error as i32 });
        }

        Ok(S7BlockInfo { 
            block_type: SubBlockType::from_u8(response[44])?, 
            block_number: Word::new(0, 0.0, response[45..47].to_vec())?.value(),
            block_lang: BlockLang::from_u8(response[43])?, 
            block_flags: response[42], 
            mc7_size: Word::new(0, 0.0, response[73..75].to_vec())?.value(),
            load_size: DInt::new(0, 0.0, response[47..51].to_vec())?.value(),
            local_data: Word::new(0, 0.0, response[71..73].to_vec())?.value(), 
            sbb_length: Word::new(0, 0.0, response[67..69].to_vec())?.value(), 
            version: response[99], 
            code_date: siemens_timestamp(Word::new(0, 0.0, response[59..61].to_vec())?.value() as i64).ok_or(Error::Response { code: error::CLI_INVALID_PLC_ANSWER })?,
            interface_date: siemens_timestamp(Word::new(0, 0.0, response[65..67].to_vec())?.value() as i64).ok_or(Error::Response { code: error::CLI_INVALID_PLC_ANSWER })?,
            author: to_chars(response[75..83].to_vec()).unwrap(),
            family: to_chars(response[83..91].to_vec()).unwrap(),
            header: to_chars(response[91..99].to_vec()).unwrap(),
        })
    }

    pub fn get_ag_block_list(&mut self) -> Result<BlockList, Error> {
        
        let s7_bl = BLOCK_LIST_TELEGRAM;

        let response = self.transport.send(&s7_bl)?;
        if response.len() < BLOCK_LIST_TELEGRAM_MIN_RESPONSE {
            return Err(Error::Response {
                code: error::ISO_INVALID_PDU,
            });
        }

        //0xff = success
        if response[29] != 0xff {
            return Err(Error::CPU { code: response[29] as i32 });
        }

        //Error code |  0 = no error
        let response_error = Word::new(0, 0.0, response[27..29].to_vec())?.value();
        if response_error != 0 {
            return Err(Error::CPU { code: response_error as i32 });
        }

        Ok(BlockList {
             ob_block_count:  Word::new(0, 0.0, response[35..37].to_vec())?.value(), 
             fb_block_count: Word::new(0, 0.0, response[39..41].to_vec())?.value(), 
             fc_block_count: Word::new(0, 0.0, response[43..45].to_vec())?.value(), 
             db_block_count: Word::new(0, 0.0, response[47..49].to_vec())?.value(), 
             sdb_block_count: Word::new(0, 0.0, response[51..53].to_vec())?.value(), 
             sfc_block_count: Word::new(0, 0.0, response[55..57].to_vec())?.value(), 
             sfb_block_count: Word::new(0, 0.0, response[59..61].to_vec())?.value(), 
            })
    }


     /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, IpAddr};
    /// use s7::{client, tcp, transport};
    /// use std::time::Duration;
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// let mut opts = tcp::Options::new(IpAddr::from(addr),0, 5, 5, transport::Connection::PG);
    ///
    /// opts.read_timeout = Duration::from_secs(2);
    /// opts.write_timeout = Duration::from_secs(2);
    ///
    ///
    /// let t = tcp::Transport::connect(opts).unwrap();
    /// let mut cl = client::Client::new(t).unwrap();
    ///
    /// let buffer = &mut vec![0u8; 255];
    ///
    /// cl.read_full_db(888, buffer); //reads the complete DB888 and stores the result in buffer
    /// 
    /// ```
    pub fn read_full_db(&mut self, db_number: u32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        let block_info = self.get_ag_block_info(BlockType::DB, db_number)?;
        let db_size = block_info.mc7_size;
        if db_size as usize > buffer.len() {
            return Err(Error::Response {
                code: error::CLI_BUFFER_TOO_SMALL,
            });
        }
        self.ag_read(db_number as i32, 0, db_size as i32, buffer)?;
        Ok(())
    }

}
