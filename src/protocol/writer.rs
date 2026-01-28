

use bytes::{BufMut, BytesMut};
use std::f32::consts::PI;


pub struct PacketWriter {
    buf: BytesMut,
}

impl PacketWriter {
   
    pub fn new() -> Self {
        Self {
            buf: BytesMut::with_capacity(128),
        }
    }

   
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: BytesMut::with_capacity(capacity),
        }
    }

   
    pub fn into_inner(self) -> BytesMut {
        self.buf
    }

   
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

   
    pub fn len(&self) -> usize {
        self.buf.len()
    }

   
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

   
    pub fn clear(&mut self) {
        self.buf.clear();
    }

   
    pub fn write_u8(&mut self, v: u8) -> &mut Self {
        self.buf.put_u8(v);
        self
    }

   
    pub fn write_i8(&mut self, v: i8) -> &mut Self {
        self.buf.put_i8(v);
        self
    }

   
    pub fn write_u16(&mut self, v: u16) -> &mut Self {
        self.buf.put_u16(v);
        self
    }

   
    pub fn write_i16(&mut self, v: i16) -> &mut Self {
        self.buf.put_i16(v);
        self
    }

   
    pub fn write_u24(&mut self, v: u32) -> &mut Self {
        self.buf.put_u8((v >> 16) as u8);
        self.buf.put_u8((v >> 8) as u8);
        self.buf.put_u8(v as u8);
        self
    }

   
    pub fn write_u32(&mut self, v: u32) -> &mut Self {
        self.buf.put_u32(v);
        self
    }

   
    pub fn write_fp8(&mut self, v: f32) -> &mut Self {
        self.buf.put_i8((v * 10.0) as i8);
        self
    }

   
    pub fn write_fp16(&mut self, v: f32, precision: u8) -> &mut Self {
        let multiplier = 10_i32.pow(precision as u32) as f32;
        self.buf.put_i16((v * multiplier) as i16);
        self
    }

   
    pub fn write_fp24(&mut self, v: f32) -> &mut Self {
        let encoded = (v.clamp(0.0, 1.0) * 16777215.0) as u32;
        self.write_u24(encoded);
        self
    }

   
    pub fn write_angle8(&mut self, angle: f32) -> &mut Self {
        let normalized = angle.rem_euclid(2.0 * PI);
        let encoded = ((normalized / (2.0 * PI)) * 256.0) as u8;
        self.buf.put_u8(encoded);
        self
    }

   
    pub fn write_angle24(&mut self, angle: f32) -> &mut Self {
        let normalized = angle.rem_euclid(2.0 * PI);
        let encoded = ((normalized / (2.0 * PI)) * 0xFFFFFF as f32) as u32;
        self.write_u24(encoded);
        self
    }

   
    pub fn write_string(&mut self, s: &str) -> &mut Self {
        let bytes = s.as_bytes();
        let len = bytes.len().min(255) as u8;
        self.buf.put_u8(len);
        self.buf.put_slice(&bytes[..len as usize]);
        self
    }

   
    pub fn write_bytes(&mut self, data: &[u8]) -> &mut Self {
        self.buf.put_slice(data);
        self
    }

   
    pub fn write_relative_coord(&mut self, v: i16) -> &mut Self {
        let encoded = (v + 128).clamp(0, 255) as u8;
        self.buf.put_u8(encoded);
        self
    }

   
    pub fn write_header(&mut self, client_time: u16, packet_type: u8) -> &mut Self {
        self.write_u16(client_time);
        self.write_u8(packet_type);
        self
    }

   
    pub fn write_speed(&mut self, speed: f32) -> &mut Self {
        self.buf.put_u8((speed / 18.0) as u8);
        self
    }
}

impl Default for PacketWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl From<PacketWriter> for BytesMut {
    fn from(writer: PacketWriter) -> Self {
        writer.buf
    }
}

impl From<PacketWriter> for Vec<u8> {
    fn from(writer: PacketWriter) -> Self {
        writer.buf.to_vec()
    }
}


pub fn create_packet(client_time: u16, packet_type: u8, payload_capacity: usize) -> PacketWriter {
    let mut writer = PacketWriter::with_capacity(3 + payload_capacity);
    writer.write_header(client_time, packet_type);
    writer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_u16() {
        let mut writer = PacketWriter::new();
        writer.write_u16(0x1234);
        assert_eq!(writer.as_bytes(), &[0x12, 0x34]);
    }

    #[test]
    fn test_write_u24() {
        let mut writer = PacketWriter::new();
        writer.write_u24(0x123456);
        assert_eq!(writer.as_bytes(), &[0x12, 0x34, 0x56]);
    }

    #[test]
    fn test_write_angle8() {
        let mut writer = PacketWriter::new();
        writer.write_angle8(PI);
        assert_eq!(writer.as_bytes()[0], 128);
    }

    #[test]
    fn test_write_string() {
        let mut writer = PacketWriter::new();
        writer.write_string("test");
        assert_eq!(writer.as_bytes(), &[4, b't', b'e', b's', b't']);
    }

    #[test]
    fn test_write_relative_coord() {
        let mut writer = PacketWriter::new();
        writer.write_relative_coord(0);
        assert_eq!(writer.as_bytes()[0], 128);

        writer.clear();
        writer.write_relative_coord(-128);
        assert_eq!(writer.as_bytes()[0], 0);

        writer.clear();
        writer.write_relative_coord(127);
        assert_eq!(writer.as_bytes()[0], 255);
    }
}
