extern crate byteorder;

use std::io::{Read};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug, PartialEq)]
pub struct PID(u16);

pub enum StreamType {
    MPEG1Video = 0x01,
    MPEG2Video = 0x02,
    MPEG1Audio = 0x03,
    H264 = 0x1B,
}

impl StreamType {

    pub fn from_u8(byte: u8) -> Result<Self, String> {
        use StreamType::*;
        match byte {
            0x01 => Ok(MPEG1Video),
            0x02 => Ok(MPEG2Video),
            0x03 => Ok(MPEG1Audio),
            0x1B => Ok(H264),
            _ => Err(format!("unknown/unimplemented stream type: {}", byte)),
        }
    }
}

impl PID {
    pub fn new(pid: u16) -> Self {
        PID(pid)
    }

    pub fn as_u16(&self) -> u16 {
        self.0
    }

    pub fn read_from<R: Read>(mut reader: R) -> Self {
        let chunk = reader.read_u16::<BigEndian>().unwrap();
        PID(chunk & 0b0001_1111_1111_1111)
    }
}

pub struct PAT {
    transport_stream_id: u16,
    version_number: u8,
}

pub struct PMT {
    program_num: u16,
    pcr_pid: Option<PID>,
    version_number: u8,
    table: Vec<ESInfo>,
}

pub struct ESInfo {
    stream_type: StreamType,
    elementary_pid: PID,
    descriptors: Vec<Descriptor>,
}

pub struct Descriptor {
    tag: u8,
    data: Vec<u8>,
}

pub struct TSPacket {
    header: TSPacketHeader,
    payload: TSPayload,
    table: Vec<ProgramAssociation>,
}

struct ProgramAssociation {
    program_num: u16,
    program_map_pid: PID,
}

pub enum TSPayload {
    PAT(PAT),
    PMT(PMT),
//    PES(PES),
//    Null(Null),
//    Raw(Bytes),
}

#[derive(PartialEq, Debug)]
pub enum TransportScramblingControl {
    NotScrambled = 0x0,
    ReservedForFutureUse = 0x1,
    ScrambledWithEvenKey = 0x2,
    ScrambledWithOddKey = 0x3,
    Unknown,
}

pub struct TSPacketHeader {
    pub transport_error_indicator: bool,
    pub transport_priority: bool,
    pub pid: PID,
    pub transport_scrambling_control: TransportScramblingControl,
    pub continuity_counter: u8,
    pub adaptation_field_control: AdaptationFieldControl,

}

#[derive(PartialEq, Debug)]
pub enum AdaptationFieldControl {
    PayloadOnly = 0b01,
    AdaptationFieldOnly = 0b10,
    AdaptationFieldAndPayload = 0b11,
}

impl AdaptationFieldControl {
    pub fn from_u8(n: u8) -> Result<Self, String> {
        match n {
            0b01 => Ok(AdaptationFieldControl::PayloadOnly),
            0b10 => Ok(AdaptationFieldControl::AdaptationFieldOnly),
            0b11 => Ok(AdaptationFieldControl::AdaptationFieldAndPayload),
            0b00 => Err("Reserved for future use".to_string()),
            _ => Err(format!("Unexpected value: {}", n)),
        }
    }
}

#[derive(Debug)]
pub enum HeaderParseError {
    SyncByteNotFound,
    UnkownTransportScrambling,
}

type HeaderParseResult = Result<TSPacketHeader, HeaderParseError>;


impl TSPacketHeader {

    pub fn read_from<R: Read>(mut reader: R) -> HeaderParseResult {
        use self::TransportScramblingControl::*;
        let sync_byte = reader.read_u8().unwrap();
        if sync_byte != 0x47 {
            return Err(HeaderParseError::SyncByteNotFound);
        }
        let upper_header = reader.read_u16::<BigEndian>().unwrap();
        //println!("{:b}", upper_header);
        let transport_error_indicator = (upper_header & 0x8000) != 0;
        let transport_priority = (upper_header & 0x2000) != 0;
        let raw_pid = upper_header & 0x1FFF;
        let lower_header = reader.read_u8().unwrap();
        let transport_scrambling_control = match lower_header >> 6 {
            0x0 => NotScrambled,
            0x2 => ScrambledWithEvenKey,
            0x3 => ScrambledWithOddKey,
            0x1 => ReservedForFutureUse,
            _ => Unknown,
        };
        if transport_scrambling_control == Unknown {
            return Err(HeaderParseError::UnkownTransportScrambling);
        }
        let continuity_counter = lower_header & 0b1111;
        let adaptation_field_control = AdaptationFieldControl::from_u8((lower_header >> 4) & 0b11).unwrap();
        let pid = PID::new(raw_pid);
        Ok(TSPacketHeader {
            transport_error_indicator,
            transport_priority,
            transport_scrambling_control,
            adaptation_field_control,
            pid,
            continuity_counter,
        })
    }

}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::prelude::*;
    use super::*;

    #[test]
    fn ts_header_parsing() {
        let mut buf = vec![];
        let mut f = File::open("file.ts").expect("ts file not found");
        let _ = f.read_to_end(&mut buf);
        let mut count = 0;
        for raw_packet in buf.chunks_mut(188) {
            if count == 0 {

                let packet_header = TSPacketHeader::read_from(&mut raw_packet.as_ref()).unwrap();
                assert_eq!(packet_header.transport_error_indicator, false);
                assert_eq!(packet_header.transport_priority, false);
                assert_eq!(packet_header.pid.as_u16(), 0);
                assert_eq!(packet_header.transport_scrambling_control, TransportScramblingControl::NotScrambled);
                assert_eq!(packet_header.adaptation_field_control, AdaptationFieldControl::PayloadOnly);
                assert_eq!(packet_header.continuity_counter, 9);
            }
            count += 1;
        }
        println!("counted {} packets", count);
        //assert_eq!(packet_header.transport_priority, false);
        //assert_eq!(packet_header.pid, PID::new(0));
        //assert_eq!(packet_header.transport_scrambling_control, TransportScramblingControl::NotScrambled);
        //assert_eq!(packet_header.continuity_counter, 9);
    }
}

