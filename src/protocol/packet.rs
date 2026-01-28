

use bytes::{BufMut, BytesMut};


pub trait PacketSerialize {
   
    fn serialize(&self, buf: &mut BytesMut);

   
    fn estimated_size(&self) -> usize {
        64
    }

   
    fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(self.estimated_size());
        self.serialize(&mut buf);
        buf
    }
}


#[derive(Debug, Clone, Copy, Default)]
pub struct PacketHeader {
   
    pub client_time: u16,
   
    pub packet_type: u8,
}

impl PacketHeader {
    pub fn new(packet_type: u8) -> Self {
        Self {
            client_time: 0,
            packet_type,
        }
    }

    pub fn with_time(client_time: u16, packet_type: u8) -> Self {
        Self {
            client_time,
            packet_type,
        }
    }

    pub fn serialize(&self, buf: &mut BytesMut) {
        buf.put_u16(self.client_time);
        buf.put_u8(self.packet_type);
    }

    pub const SIZE: usize = 3;
}



pub const HANDSHAKE_SECRET: &[u8] = b"dakrtywcilopuhgrmzwsdolitualksrrarjsrzyjhrnzvfdfkrsyahjvuobhjkmzwvgoppxaagiwvscjlqoualghnuvdedozuwcdjosrcnhjprwlkfqbyegkorwtepmlstcfhksxakilruwdhhouwdchnsqecngvqpcz";


pub mod protocol {
   
    pub const VERSION_LEGACY: u8 = 14;
   
    pub const VERSION_MODERN: u8 = 25;
   
    pub const VERSION_CURRENT: u8 = 14;
}


pub fn is_modern_protocol(version: u8) -> bool {
    version >= protocol::VERSION_MODERN
}


pub const MAX_PACKET_SIZE: usize = 65536;


pub const MIN_PACKET_SIZE: usize = PacketHeader::SIZE;
