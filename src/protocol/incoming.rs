

use crate::protocol::reader::PacketReader;
use std::io;


#[derive(Debug, Clone)]
pub enum IncomingPacket {
   
   
    ProtocolMode { want_etm: bool },
   
    StartLogin,
   
    Login(LoginPacket),
   
    SetIdentity(SetIdentityPacket),
   
    Rotation(RotationPacket),
   
    Angle(AnglePacket),
   
    StartAcceleration,
   
    StopAcceleration,
   
    Ping,
   
    VictoryMessage(String),
   
    Unknown(u8, Vec<u8>),
}


#[derive(Debug, Clone)]
pub struct LoginPacket {
   
    pub protocol_version: u8,
   
    pub version: u16,
   
    pub checksum: [u8; 20],
   
    pub skin: u8,
   
    pub nickname: String,
   
    pub custom_skin: Option<String>,
}


#[derive(Debug, Clone)]
pub struct SetIdentityPacket {
   
    pub protocol_version: u8,
   
    pub skin: u8,
   
    pub nickname: String,
   
    pub custom_skin: Option<String>,
}


#[derive(Debug, Clone)]
pub struct RotationPacket {
   
   
   
    pub value: u8,
   
    pub is_legacy_left: bool,
   
    pub is_legacy_right: bool,
}

impl RotationPacket {
   
    pub fn is_clockwise(&self) -> bool {
        if self.is_legacy_right {
            true
        } else if self.is_legacy_left {
            false
        } else {
            self.value >= 128
        }
    }

   
    pub fn intensity(&self) -> u8 {
        if self.is_legacy_left || self.is_legacy_right {
            self.value
        } else if self.value >= 128 {
            self.value - 128
        } else {
            self.value
        }
    }
}


#[derive(Debug, Clone)]
pub struct AnglePacket {
   
    pub angle: u8,
}

impl AnglePacket {
   
    pub fn to_radians(&self) -> f32 {
        std::f32::consts::PI * self.angle as f32 / 125.0
    }
}


pub fn parse_incoming_packet(data: &[u8], _protocol_version: u8) -> io::Result<IncomingPacket> {
    if data.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "empty packet"));
    }

    let len = data.len();
    let cmd = data[0];

   
    if len == 24 {
        return Ok(IncomingPacket::Unknown(cmd, data[1..].to_vec()));
    }

   
    if cmd == b'c' {
        return Ok(IncomingPacket::StartLogin);
    }

   
    if cmd == b's' {
        return parse_username_packet(&data[1..]);
    }

   
   
    if len == 1 && (cmd == 1 || cmd == 2) {
        return Ok(IncomingPacket::ProtocolMode { want_etm: cmd == 2 });
    }

   
   
    if len == 1 && cmd <= 250 {
        return Ok(IncomingPacket::Angle(AnglePacket { angle: cmd }));
    }

   
    match cmd {
       
        252 => {
            if len >= 2 {
                Ok(IncomingPacket::Rotation(RotationPacket {
                    value: data[1],
                    is_legacy_left: false,
                    is_legacy_right: false,
                }))
            } else {
                Ok(IncomingPacket::Rotation(RotationPacket {
                    value: 0,
                    is_legacy_left: false,
                    is_legacy_right: false,
                }))
            }
        }

       
        b'l' | 108 => {
            let value = if len >= 2 { data[1] } else { 64 };
            Ok(IncomingPacket::Rotation(RotationPacket {
                value,
                is_legacy_left: true,
                is_legacy_right: false,
            }))
        }

       
        b'r' | 114 => {
            let value = if len >= 2 { data[1] } else { 64 };
            Ok(IncomingPacket::Rotation(RotationPacket {
                value,
                is_legacy_left: false,
                is_legacy_right: true,
            }))
        }

       
        253 => Ok(IncomingPacket::StartAcceleration),

       
        254 => Ok(IncomingPacket::StopAcceleration),

       
        251 => Ok(IncomingPacket::Ping),

       
        255 => {
            let msg = if len > 2 && data[1] == b'v' {
                String::from_utf8_lossy(&data[2..]).to_string()
            } else if len > 1 {
                String::from_utf8_lossy(&data[1..]).to_string()
            } else {
                String::new()
            };
            Ok(IncomingPacket::VictoryMessage(msg))
        }

       
        _ => Ok(IncomingPacket::Unknown(cmd, data[1..].to_vec())),
    }
}





fn parse_username_packet(data: &[u8]) -> io::Result<IncomingPacket> {
    if data.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "username packet empty"));
    }

    let mut pos = 0;

   
    let client_protocol = data[pos];
    pos += 1;

   
   
   
   
   
    let looks_like_official = client_protocol >= 25 && data.len() >= 1 + 2 + 20 + 1 + 1;

    if looks_like_official {
       
        if pos + 2 > data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "missing client_version"));
        }
        let version = ((data[pos] as u16) << 8) | data[pos + 1] as u16;
        pos += 2;

       
        if pos + 20 > data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "missing checksum"));
        }
        let mut checksum = [0u8; 20];
        checksum.copy_from_slice(&data[pos..pos + 20]);
        pos += 20;

       
        if pos >= data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "missing skin"));
        }
        let skin = data[pos];
        pos += 1;

       
        if pos >= data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "missing name length"));
        }
        let name_len = data[pos] as usize;
        pos += 1;

       
        if pos + name_len > data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "name truncated"));
        }
        let nickname = String::from_utf8_lossy(&data[pos..pos + name_len]).to_string();
        pos += name_len;

       
       
       
       
        Ok(IncomingPacket::Login(LoginPacket {
            protocol_version: client_protocol,
            version,
            checksum,
            skin,
            nickname,
            custom_skin: None,
        }))
    } else {
       
        if data.len() < pos + 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("identity packet too short: {} bytes", data.len()),
            ));
        }

        let skin = data[pos];
        pos += 1;

        let name_len = data[pos] as usize;
        pos += 1;

        let name_end = (pos + name_len).min(data.len());
        let actual_name_len = name_end - pos;
        let nickname = if actual_name_len > 0 {
            String::from_utf8_lossy(&data[pos..name_end]).to_string()
        } else {
            String::new()
        };
        pos = name_end;

       
        let custom_skin = if pos < data.len() {
            Some(String::from_utf8_lossy(&data[pos..]).to_string())
        } else {
            None
        };

        Ok(IncomingPacket::SetIdentity(SetIdentityPacket {
            protocol_version: client_protocol,
            skin,
            nickname,
            custom_skin,
        }))
    }
}
fn parse_identity_packet(data: &[u8]) -> io::Result<IncomingPacket> {
    if data.len() < 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("identity packet too short: {} bytes", data.len()),
        ));
    }

    let mut pos = 0;
    let protocol_version = data[pos];
    pos += 1;

   
   
   
    if data.len() >= 27 {
        let mut p = pos;

       
        if p + 2 <= data.len() {
            let version = u16::from_be_bytes([data[p], data[p + 1]]);
            p += 2;

           
            if p + 20 <= data.len() {
                let mut checksum = [0u8; 20];
                checksum.copy_from_slice(&data[p..p + 20]);
                p += 20;

               
                if p + 2 <= data.len() {
                    let skin = data[p];
                    p += 1;
                    let name_len = data[p] as usize;
                    p += 1;

                   
                    if p + name_len + 2 <= data.len() {
                        let nickname = if name_len > 0 {
                            String::from_utf8_lossy(&data[p..p + name_len]).to_string()
                        } else {
                            String::new()
                        };
                        p += name_len;

                       
                        if p + 2 <= data.len() {
                            p += 2;
                        }

                        let custom_skin = if p < data.len() {
                            Some(String::from_utf8_lossy(&data[p..]).to_string())
                        } else {
                            None
                        };

                        return Ok(IncomingPacket::Login(LoginPacket {
                            protocol_version,
                            version,
                            checksum,
                            skin,
                            nickname,
                            custom_skin,
                        }));
                    }
                }
            }
        }
       
    }

   
   
    if pos + 2 > data.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "missing skin/name_len"));
    }
    let skin = data[pos];
    pos += 1;

    let name_len = data[pos] as usize;
    pos += 1;

    if pos + name_len > data.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "nickname out of range"));
    }

    let nickname = if name_len > 0 {
        String::from_utf8_lossy(&data[pos..pos + name_len]).to_string()
    } else {
        String::new()
    };
    pos += name_len;

    let custom_skin = if pos < data.len() {
        Some(String::from_utf8_lossy(&data[pos..]).to_string())
    } else {
        None
    };

    Ok(IncomingPacket::SetIdentity(SetIdentityPacket {
        protocol_version,
        skin,
        nickname,
        custom_skin,
    }))
}


#[derive(Debug, Clone, Default)]
pub struct ProtocolState {
   
    pub want_seq: bool,
   
    pub want_etm: bool,
   
    pub current_seq: u16,
   
    pub protocol_version: u8,
   
    pub handshake_complete: bool,
}

impl ProtocolState {
    pub fn new() -> Self {
        Self {
            protocol_version: 14,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_start_login() {
        let data = [b'c'];
        let packet = parse_incoming_packet(&data, 14).unwrap();
        assert!(matches!(packet, IncomingPacket::StartLogin));
    }

    #[test]
    fn test_parse_rotation() {
        let data = [252, 64];
        let packet = parse_incoming_packet(&data, 14).unwrap();

        if let IncomingPacket::Rotation(rot) = packet {
            assert!(!rot.is_clockwise());
            assert_eq!(rot.intensity(), 64);
        } else {
            panic!("Expected rotation packet");
        }
    }

    #[test]
    fn test_parse_identity() {
       
        let data = [14, 3, 4, b'T', b'e', b's', b't'];
        let packet = parse_username_packet(&data).unwrap();

        if let IncomingPacket::SetIdentity(id) = packet {
            assert_eq!(id.protocol_version, 14);
            assert_eq!(id.skin, 3);
            assert_eq!(id.nickname, "Test");
        } else {
            panic!("Expected identity packet");
        }
    }

    #[test]
    fn test_parse_angle() {
        let data = [125];
        let packet = parse_incoming_packet(&data, 14).unwrap();

        if let IncomingPacket::Angle(ang) = packet {
            assert_eq!(ang.angle, 125);
        } else {
            panic!("Expected angle packet");
        }
    }
}
