

use crate::protocol::packet::{PacketSerialize, HANDSHAKE_SECRET};
use crate::protocol::types::*;
use crate::protocol::writer::PacketWriter;
use bytes::BytesMut;





#[derive(Debug, Clone)]
pub struct PacketPreInit;

impl PacketSerialize for PacketPreInit {
    fn serialize(&self, buf: &mut BytesMut) {
       
        let mut writer = PacketWriter::with_capacity(1 + HANDSHAKE_SECRET.len());
        writer.write_u8(b'6');
        writer.write_bytes(HANDSHAKE_SECRET);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        1 + HANDSHAKE_SECRET.len()
    }
}


















#[derive(Debug, Clone)]
pub struct PacketInit {
    pub game_radius: u32,
    pub max_snake_parts: u16,
    pub sector_size: u16,
    pub sector_count_along_edge: u16,
    pub spangdv: f32,
    pub nsp1: f32,
    pub nsp2: f32,
    pub nsp3: f32,
    pub snake_ang_speed: f32,
    pub prey_ang_speed: f32,
    pub snake_tail_k: f32,
    pub protocol_version: u8,
    pub default_msl: u8,
    pub snake_id: SnakeId,
}

impl Default for PacketInit {
    fn default() -> Self {
        Self {
            game_radius: 21600,
            max_snake_parts: 411,
            sector_size: 480,
            sector_count_along_edge: 90,
            spangdv: 4.8,
            nsp1: 5.39,
            nsp2: 0.4,
            nsp3: 14.0,
            snake_ang_speed: 0.033,
            prey_ang_speed: 0.028,
            snake_tail_k: 0.43,
            protocol_version: 14,
            default_msl: 42, 
            snake_id: 0,
        }
    }
}

impl PacketSerialize for PacketInit {
    fn serialize(&self, buf: &mut BytesMut) {
       
       
       
       
       
       
       
       
       
       
       
        let mut writer = PacketWriter::with_capacity(27);
        writer.write_u8(b'a');
        writer.write_u24(self.game_radius);
        writer.write_u16(self.max_snake_parts);
        writer.write_u16(self.sector_size);
        writer.write_u16(self.sector_count_along_edge);
        writer.write_u8((self.spangdv * 10.0) as u8);
        writer.write_u16((self.nsp1 * 100.0) as u16);
        writer.write_u16((self.nsp2 * 100.0) as u16);
        writer.write_u16((self.nsp3 * 100.0) as u16);
        writer.write_u16((self.snake_ang_speed * 1000.0) as u16);
        writer.write_u16((self.prey_ang_speed * 1000.0) as u16);
        writer.write_u16((self.snake_tail_k * 1000.0) as u16);
        writer.write_u8(self.protocol_version);

       
        writer.write_u8(self.default_msl);
        writer.write_u16(self.snake_id);

        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        27
    }
}

#[derive(Debug, Clone)]
pub struct PacketPong;

impl PacketSerialize for PacketPong {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(1);
        writer.write_u8(b'p');
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        1
    }
}


#[derive(Debug, Clone)]
pub struct PacketRotation {
    pub snake_id: SnakeId,
    pub angle: f32,
    pub target_angle: f32,
    pub speed: f32,
    pub include_angle: bool,
    pub include_target: bool,
    pub clockwise: bool,
}

impl PacketSerialize for PacketRotation {
    fn serialize(&self, buf: &mut BytesMut) {
        let packet_type = if self.clockwise {
            if self.include_angle {
                b'4'
            } else {
                b'5'
            }
        } else if self.include_angle && self.include_target {
            b'e'
        } else if self.include_angle {
            b'3'
        } else {
            b'E'
        };

        let mut writer = PacketWriter::with_capacity(6);
        writer.write_u8(packet_type);
        writer.write_u16(self.snake_id);

       
        if self.include_angle {
            writer.write_angle8(self.angle);
        }
        if self.include_target {
            writer.write_angle8(self.target_angle);
        }
        writer.write_speed(self.speed);

        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        6
    }
}


#[derive(Debug, Clone)]
pub struct PacketMove {
    pub snake_id: SnakeId,
    pub x: u16,
    pub y: u16,
}

impl PacketSerialize for PacketMove {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(7);
        writer.write_u8(b'g');
        writer.write_u16(self.snake_id);
        writer.write_u16(self.x);
        writer.write_u16(self.y);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        7
    }
}



#[derive(Debug, Clone)]
pub struct PacketMoveOwn {
    pub x: u16,
    pub y: u16,
}

impl PacketSerialize for PacketMoveOwn {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(5);
        writer.write_u8(b'g');
        writer.write_u16(self.x);
        writer.write_u16(self.y);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        5
    }
}


#[derive(Debug, Clone)]
pub struct PacketMoveRel {
    pub snake_id: SnakeId,
    pub dx: i16,
    pub dy: i16,
}

impl PacketSerialize for PacketMoveRel {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(5);
        writer.write_u8(b'G');
        writer.write_u16(self.snake_id);
        writer.write_relative_coord(self.dx);
        writer.write_relative_coord(self.dy);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        5
    }
}



#[derive(Debug, Clone)]
pub struct PacketMoveRelOwn {
    pub dx: i16,
    pub dy: i16,
}

impl PacketSerialize for PacketMoveRelOwn {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(3);
        writer.write_u8(b'G');
        writer.write_relative_coord(self.dx);
        writer.write_relative_coord(self.dy);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        3
    }
}


#[derive(Debug, Clone)]
pub struct PacketInc {
    pub snake_id: SnakeId,
    pub x: u16,
    pub y: u16,
    pub fullness: f32,
}

impl PacketSerialize for PacketInc {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(10);
        writer.write_u8(b'n');
        writer.write_u16(self.snake_id);
        writer.write_u16(self.x);
        writer.write_u16(self.y);
        writer.write_fp24(self.fullness);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        10
    }
}


#[derive(Debug, Clone)]
pub struct PacketIncRel {
    pub snake_id: SnakeId,
    pub dx: i16,
    pub dy: i16,
    pub fullness: f32,
}

impl PacketSerialize for PacketIncRel {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(8);
        writer.write_u8(b'N');
        writer.write_u16(self.snake_id);
        writer.write_relative_coord(self.dx);
        writer.write_relative_coord(self.dy);
        writer.write_fp24(self.fullness);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        8
    }
}


#[derive(Debug, Clone)]
pub struct PacketSetFullness {
    pub snake_id: SnakeId,
    pub fullness: f32,
}

impl PacketSerialize for PacketSetFullness {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(6);
        writer.write_u8(b'h');
        writer.write_u16(self.snake_id);
       
        writer.write_fp24(self.fullness);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        6
    }
}


#[derive(Debug, Clone)]
pub struct PacketRemovePart {
    pub snake_id: SnakeId,
    pub fullness: f32,
}

impl PacketSerialize for PacketRemovePart {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(6);
        writer.write_u8(b'r');
        writer.write_u16(self.snake_id);
        writer.write_fp24(self.fullness);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        6
    }
}



















#[derive(Debug, Clone)]
pub struct PacketAddSnake {
    pub snake_id: SnakeId,
    pub skin: u8,
    pub angle: f32,
    pub target_angle: f32,
    pub speed: f32,
    pub fullness: f32,
    pub head_x: f32,
    pub head_y: f32,
    pub name: String,
    pub custom_skin: Option<Vec<u8>>,
   
    pub body_parts: Vec<(f32, f32)>,
}

impl PacketSerialize for PacketAddSnake {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(self.estimated_size());
        writer.write_u8(b's');
        writer.write_u16(self.snake_id);

       
       
        writer.write_angle24(self.angle);

       
       
       
       
        writer.write_u8(48);

       
        writer.write_angle24(self.target_angle);

       
       
       
        writer.write_u16((self.speed * 1000.0 / 32.0) as u16);

       
       
        writer.write_fp24(self.fullness);

       
        writer.write_u8(self.skin);

       
        writer.write_u24((self.head_x * 5.0) as u32);
        writer.write_u24((self.head_y * 5.0) as u32);

       
        writer.write_string(&self.name);

       
        if let Some(ref custom) = self.custom_skin {
            writer.write_u8(custom.len().min(255) as u8);
            writer.write_bytes(&custom[..custom.len().min(255)]);
        } else {
            writer.write_u8(0);
        }

       
       
        writer.write_u8(255);

       
       
        if !self.body_parts.is_empty() {
           
            let tail_idx = self.body_parts.len() - 1;
            let (tail_x, tail_y) = self.body_parts[tail_idx];

           
            writer.write_u24((tail_x * 5.0) as u32);
            writer.write_u24((tail_y * 5.0) as u32);

           
            let mut last_x = tail_x;
            let mut last_y = tail_y;

            for i in (0..tail_idx).rev() {
                let (x, y) = self.body_parts[i];
               
                let dx = ((x - last_x) * 2.0 + 127.0).clamp(0.0, 255.0) as u8;
                let dy = ((y - last_y) * 2.0 + 127.0).clamp(0.0, 255.0) as u8;
                writer.write_u8(dx);
                writer.write_u8(dy);
                last_x = x;
                last_y = y;
            }
        }

        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
       
        25 + self.name.len() + self.custom_skin.as_ref().map_or(0, |s| s.len())
            + if self.body_parts.is_empty() { 0 } else { 6 + (self.body_parts.len() - 1) * 2 }
    }
}


#[derive(Debug, Clone)]
pub struct PacketRemoveSnake {
    pub snake_id: SnakeId,
    pub status: SnakeRemoveStatus,
}

impl PacketSerialize for PacketRemoveSnake {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(4);
        writer.write_u8(b's');
        writer.write_u16(self.snake_id);
        writer.write_u8(self.status as u8);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        4
    }
}


#[derive(Debug, Clone)]
pub struct PacketEnd {
    pub status: GameEndStatus,
}

impl PacketSerialize for PacketEnd {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(2);
        writer.write_u8(b'v');
        writer.write_u8(self.status as u8);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        2
    }
}


#[derive(Debug, Clone)]
pub struct PacketKill {
    pub killer_snake_id: SnakeId,
    pub total_kills: u32,
}

impl PacketSerialize for PacketKill {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(6);
        writer.write_u8(b'k');
        writer.write_u16(self.killer_snake_id);
        writer.write_u24(self.total_kills);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        6
    }
}


#[derive(Debug, Clone)]
pub struct PacketAddSector {
    pub x: u8,
    pub y: u8,
}

impl PacketSerialize for PacketAddSector {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(3);
        writer.write_u8(b'W');
        writer.write_u8(self.x);
        writer.write_u8(self.y);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        3
    }
}


#[derive(Debug, Clone)]
pub struct PacketRemoveSector {
    pub x: u8,
    pub y: u8,
}

impl PacketSerialize for PacketRemoveSector {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(3);
        writer.write_u8(b'w');
        writer.write_u8(self.x);
        writer.write_u8(self.y);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        3
    }
}


#[derive(Debug, Clone, Copy)]
pub struct FoodData {
    pub x: u16,
    pub y: u16,
    pub size: u8,
    pub color: u8,
}








#[derive(Debug, Clone)]
pub struct PacketSetFood {
    pub sector_x: u8,
    pub sector_y: u8,
    pub sector_size: u16,
    pub foods: Vec<FoodData>,
}

impl PacketSerialize for PacketSetFood {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(3 + self.foods.len() * 4);
        writer.write_u8(b'F');
        writer.write_u8(self.sector_x);
        writer.write_u8(self.sector_y);

        let base_x = self.sector_x as u32 * self.sector_size as u32;
        let base_y = self.sector_y as u32 * self.sector_size as u32;

        for food in &self.foods {
           
           
            let rx = ((food.x as u32).saturating_sub(base_x) * 256 / self.sector_size as u32).min(255) as u8;
            let ry = ((food.y as u32).saturating_sub(base_y) * 256 / self.sector_size as u32).min(255) as u8;

            writer.write_u8(food.color);
            writer.write_u8(rx);
            writer.write_u8(ry);
            writer.write_u8(food.size * 5); 
        }

        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        3 + self.foods.len() * 4
    }
}








#[derive(Debug, Clone)]
pub struct PacketAddFood {
    pub food: FoodData,
    pub sector_size: u16,
}

impl PacketSerialize for PacketAddFood {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(7);
        writer.write_u8(b'f');

       
        let sx = (self.food.x / self.sector_size) as u8;
        let sy = (self.food.y / self.sector_size) as u8;
        let base_x = sx as u32 * self.sector_size as u32;
        let base_y = sy as u32 * self.sector_size as u32;
        let rx = ((self.food.x as u32).saturating_sub(base_x) * 256 / self.sector_size as u32).min(255) as u8;
        let ry = ((self.food.y as u32).saturating_sub(base_y) * 256 / self.sector_size as u32).min(255) as u8;

        writer.write_u8(sx);
        writer.write_u8(sy);
        writer.write_u8(rx);
        writer.write_u8(ry);
        writer.write_u8(self.food.color);
        writer.write_u8(self.food.size * 5); 
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        7
    }
}








#[derive(Debug, Clone)]
pub struct PacketSpawnFood {
    pub food: FoodData,
    pub sector_size: u16,
}

impl PacketSerialize for PacketSpawnFood {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(7);
        writer.write_u8(b'b');

       
        let sx = (self.food.x / self.sector_size) as u8;
        let sy = (self.food.y / self.sector_size) as u8;
        let base_x = sx as u32 * self.sector_size as u32;
        let base_y = sy as u32 * self.sector_size as u32;
        let rx = ((self.food.x as u32).saturating_sub(base_x) * 256 / self.sector_size as u32).min(255) as u8;
        let ry = ((self.food.y as u32).saturating_sub(base_y) * 256 / self.sector_size as u32).min(255) as u8;

        writer.write_u8(sx);
        writer.write_u8(sy);
        writer.write_u8(rx);
        writer.write_u8(ry);
        writer.write_u8(self.food.color);
        writer.write_u8(self.food.size * 5); 
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        7
    }
}







#[derive(Debug, Clone)]
pub struct PacketEatFood {
    pub snake_id: SnakeId,
    pub food_x: u16,
    pub food_y: u16,
    pub sector_size: u16,
}

impl PacketSerialize for PacketEatFood {
    fn serialize(&self, buf: &mut BytesMut) {
       
        let cmd = if self.snake_id > 0 { b'<' } else { b'c' };

        let mut writer = PacketWriter::with_capacity(7);
        writer.write_u8(cmd);

       
        let sx = (self.food_x / self.sector_size) as u8;
        let sy = (self.food_y / self.sector_size) as u8;
        let base_x = sx as u32 * self.sector_size as u32;
        let base_y = sy as u32 * self.sector_size as u32;
        let rx = ((self.food_x as u32).saturating_sub(base_x) * 256 / self.sector_size as u32).min(255) as u8;
        let ry = ((self.food_y as u32).saturating_sub(base_y) * 256 / self.sector_size as u32).min(255) as u8;

        writer.write_u8(sx);
        writer.write_u8(sy);
        writer.write_u8(rx);
        writer.write_u8(ry);

       
        if self.snake_id > 0 {
            writer.write_u16(self.snake_id);
        }

        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        7
    }
}


#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub parts: u16,
    pub fullness: f32,
    pub font_color: u8,
    pub name: String,
}


#[derive(Debug, Clone)]
pub struct PacketLeaderboard {
    pub player_rank: u8,
    pub local_rank: u16,
    pub player_count: u16,
    pub entries: Vec<LeaderboardEntry>,
}

impl PacketSerialize for PacketLeaderboard {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(self.estimated_size());
        writer.write_u8(b'l');
        writer.write_u8(self.player_rank);
        writer.write_u16(self.local_rank);
        writer.write_u16(self.player_count);

        for entry in &self.entries {
            writer.write_u16(entry.parts);
           
            writer.write_fp24(entry.fullness);
            writer.write_u8(entry.font_color);
            writer.write_string(&entry.name);
        }

        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        6 + self.entries.iter().map(|e| 7 + e.name.len()).sum::<usize>()
    }
}


#[derive(Debug, Clone)]
pub struct PacketHighScore {
    pub snake_length: u32,
    pub winner_name: String,
    pub message: String,
}

impl PacketSerialize for PacketHighScore {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(self.estimated_size());
        writer.write_u8(b'm');
        writer.write_u24(self.snake_length);
        writer.write_u24(0);
        writer.write_string(&self.winner_name);
        writer.write_bytes(self.message.as_bytes());

        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        8 + self.winner_name.len() + self.message.len()
    }
}


#[derive(Debug, Clone)]
pub struct PacketMinimap {
    pub grid_size: u16,
    pub data: Vec<u8>,
    pub use_modern: bool,
}

impl PacketSerialize for PacketMinimap {
    fn serialize(&self, buf: &mut BytesMut) {
        let mut writer = PacketWriter::with_capacity(self.estimated_size());
        let packet_type = if self.use_modern { b'M' } else { b'u' };
        writer.write_u8(packet_type);

        if self.use_modern {
            writer.write_u16(self.grid_size);
        }

        writer.write_bytes(&self.data);
        buf.extend_from_slice(writer.as_bytes());
    }

    fn estimated_size(&self) -> usize {
        1 + if self.use_modern { 2 } else { 0 } + self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_pong() {
        let packet = PacketPong;
        let bytes = packet.to_bytes();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], b'p');
    }

    #[test]
    fn test_packet_init() {
        let packet = PacketInit::default();
        let bytes = packet.to_bytes();
       
       
       
        assert_eq!(bytes.len(), 24);
        assert_eq!(bytes[0], b'a');
    }

    #[test]
    fn test_packet_move() {
        let packet = PacketMove {
            snake_id: 1,
            x: 1000,
            y: 2000,
        };
        let bytes = packet.to_bytes();
       
        assert_eq!(bytes.len(), 7);
        assert_eq!(bytes[0], b'g');
    }
}
