extern crate byteorder;
use std::io::{Read};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub struct TSPacket {
    header: TSPacketHeader,
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
    pub pid: u16,
    pub transport_scrambling_control: TransportScramblingControl,
    pub continuity_counter: u8,
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
        println!("{:b}", upper_header);
        let transport_error_indicator = (upper_header & 0b1000_0000_0000_0000) != 0;
        let transport_priority = (upper_header & 0b0010_0000_0000_0000) != 0;
        let pid = upper_header & 0b0001_1111_1111_1111;
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
        Ok(TSPacketHeader {
            transport_error_indicator,
            transport_priority,
            transport_scrambling_control,
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
        let f = File::open("file.ts").expect("ts file not found");
        let packet_header = TSPacketHeader::read_from(f).unwrap();
        assert_eq!(packet_header.transport_error_indicator, false);
        assert_eq!(packet_header.transport_priority, false);
        assert_eq!(packet_header.pid, 0);
        assert_eq!(packet_header.transport_scrambling_control, TransportScramblingControl::NotScrambled);
        assert_eq!(packet_header.continuity_counter, 9);
    }
}
