

use std::io::{self, Cursor, Read};
use byteorder::{BigEndian, ReadBytesExt};


pub struct PacketReader<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> PacketReader<'a> {
   
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }

   
    pub fn remaining(&self) -> usize {
        self.cursor.get_ref().len() - self.cursor.position() as usize
    }

   
    pub fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }

   
    pub fn position(&self) -> usize {
        self.cursor.position() as usize
    }

   
    pub fn skip(&mut self, n: usize) -> io::Result<()> {
        let new_pos = self.cursor.position() + n as u64;
        if new_pos > self.cursor.get_ref().len() as u64 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "skip past end"));
        }
        self.cursor.set_position(new_pos);
        Ok(())
    }

   
    pub fn read_u8(&mut self) -> io::Result<u8> {
        self.cursor.read_u8()
    }

   
    pub fn read_i8(&mut self) -> io::Result<i8> {
        self.cursor.read_i8()
    }

   
    pub fn read_u16(&mut self) -> io::Result<u16> {
        self.cursor.read_u16::<BigEndian>()
    }

   
    pub fn read_i16(&mut self) -> io::Result<i16> {
        self.cursor.read_i16::<BigEndian>()
    }

   
    pub fn read_u24(&mut self) -> io::Result<u32> {
        let b1 = self.read_u8()? as u32;
        let b2 = self.read_u8()? as u32;
        let b3 = self.read_u8()? as u32;
        Ok((b1 << 16) | (b2 << 8) | b3)
    }

   
    pub fn read_u32(&mut self) -> io::Result<u32> {
        self.cursor.read_u32::<BigEndian>()
    }

   
    pub fn read_fp8(&mut self) -> io::Result<f32> {
        let v = self.read_i8()?;
        Ok(v as f32 / 10.0)
    }

   
    pub fn read_fp16(&mut self, precision: u8) -> io::Result<f32> {
        let v = self.read_i16()?;
        let divisor = 10_i32.pow(precision as u32) as f32;
        Ok(v as f32 / divisor)
    }

   
    pub fn read_fp24(&mut self) -> io::Result<f32> {
        let v = self.read_u24()?;
        Ok(v as f32 / 16777215.0)
    }

   
    pub fn read_angle8(&mut self) -> io::Result<f32> {
        let v = self.read_u8()?;
        Ok(v as f32 * std::f32::consts::PI * 2.0 / 256.0)
    }

   
    pub fn read_angle24(&mut self) -> io::Result<f32> {
        let v = self.read_u24()?;
        Ok(v as f32 * std::f32::consts::PI * 2.0 / 0xFFFFFF as f32)
    }

   
    pub fn read_string(&mut self) -> io::Result<String> {
        let len = self.read_u8()? as usize;
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

   
    pub fn read_bytes(&mut self, len: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

   
    pub fn read_relative_coord(&mut self) -> io::Result<i16> {
        let v = self.read_u8()?;
        Ok(v as i16 - 128)
    }

   
    pub fn read_remaining(&mut self) -> io::Result<Vec<u8>> {
        let remaining = self.remaining();
        self.read_bytes(remaining)
    }

   
    pub fn peek_u8(&self) -> io::Result<u8> {
        let pos = self.cursor.position() as usize;
        let data = self.cursor.get_ref();
        if pos >= data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "peek past end"));
        }
        Ok(data[pos])
    }
}



pub fn parse_protocol14_header(data: &[u8], want_seq: bool, want_etm: bool) -> (Option<u16>, Option<u16>, usize) {
    let mut offset = 0;
    let mut seq = None;
    let mut etm = None;

    if want_seq && data.len() >= 2 {
        seq = Some(((data[0] as u16) << 8) | (data[1] as u16));
        offset += 2;
    }

    if want_etm && data.len() >= offset + 2 {
        etm = Some(((data[offset] as u16) << 8) | (data[offset + 1] as u16));
        offset += 2;
    }

    (seq, etm, offset)
}


pub fn parse_stacked_packets(data: &[u8], offset: usize) -> Vec<&[u8]> {
    let mut packets = Vec::new();
    let mut pos = offset;

    while pos < data.len() {
        let len = if data[pos] < 32 {
            if pos + 1 >= data.len() {
                break;
            }
            let len = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
            pos += 2;
            len
        } else {
            let len = (data[pos] - 32) as usize;
            pos += 1;
            len
        };

        if pos + len > data.len() {
            break;
        }

        packets.push(&data[pos..pos + len]);
        pos += len;
    }

    packets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u8() {
        let data = [0x42];
        let mut reader = PacketReader::new(&data);
        assert_eq!(reader.read_u8().unwrap(), 0x42);
    }

    #[test]
    fn test_read_u16() {
        let data = [0x12, 0x34];
        let mut reader = PacketReader::new(&data);
        assert_eq!(reader.read_u16().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_u24() {
        let data = [0x12, 0x34, 0x56];
        let mut reader = PacketReader::new(&data);
        assert_eq!(reader.read_u24().unwrap(), 0x123456);
    }

    #[test]
    fn test_parse_stacked_packets() {
       
        let data = [
            35,
            b'a', b'b', b'c',
            34,
            b'd', b'e',
        ];
        let packets = parse_stacked_packets(&data, 0);
        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0], b"abc");
        assert_eq!(packets[1], b"de");
    }
}
