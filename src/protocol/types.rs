

use std::f32::consts::PI;


pub type SnakeId = u16;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IncomingPacketType {
   
    StartLogin = b'c',
   
    VerifyCode = b'o',
   
    UsernameSkin = b's',
   
    Rotation = 252,
   
    Angle = 0,
   
    Ping = 251,
   
    RotLeft = b'l',
   
    RotRight = b'r',
   
    StartAcc = 253,
   
    StopAcc = 254,
   
    VictoryMessage = 255,
}

impl TryFrom<u8> for IncomingPacketType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'c' => Ok(Self::StartLogin),
            b'o' => Ok(Self::VerifyCode),
            b's' => Ok(Self::UsernameSkin),
            252 => Ok(Self::Rotation),
            0..=250 => Ok(Self::Angle),
            251 => Ok(Self::Ping),
            b'l' => Ok(Self::RotLeft),
            b'r' => Ok(Self::RotRight),
            253 => Ok(Self::StartAcc),
            254 => Ok(Self::StopAcc),
            255 => Ok(Self::VictoryMessage),
            _ => Err(value),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OutgoingPacketType {
   
    Init = b'a',
   
    PreInit = b'6',
   
    RotCcw = b'e',
   
    RotCcwNoAng = b'E',
   
    RotCcwSpeed = b'3',
   
    RotCwSpeed = b'4',
   
    RotCw = b'5',
   
    SetFullness = b'h',
   
    RemovePart = b'r',
   
    Move = b'g',
   
    MoveRel = b'G',
   
    Inc = b'n',
   
    IncRel = b'N',
   
    Leaderboard = b'l',
   
    End = b'v',
   
    AddSector = b'W',
   
    RemoveSector = b'w',
   
    HighScore = b'm',
   
    Pong = b'p',
   
    Minimap = b'M',
   
    MinimapLegacy = b'u',
   
    Snake = b's',
   
    SetFood = b'F',
   
    SpawnFood = b'b',
   
    AddFood = b'f',
   
    EatFood = b'c',
   
    EatFoodRel = b'<',
   
    MovePrey = b'j',
   
    AddPrey = b'y',
   
    RemovePrey = b'Y',
   
    Kill = b'k',
   
    DebugReset = b'0',
   
    DebugDraw = b'!',
}

impl From<OutgoingPacketType> for u8 {
    fn from(val: OutgoingPacketType) -> Self {
        val as u8
    }
}


#[derive(Debug, Clone, Copy, Default)]
pub struct SnakeChanges(pub u8);

impl SnakeChanges {
    pub const CHANGE_POS: u8 = 0x01;
    pub const CHANGE_ANGLE: u8 = 0x02;
    pub const CHANGE_WANGLE: u8 = 0x04;
    pub const CHANGE_SPEED: u8 = 0x08;
    pub const CHANGE_FULLNESS: u8 = 0x10;
    pub const CHANGE_DYING: u8 = 0x20;
    pub const CHANGE_DEAD: u8 = 0x40;

    pub fn has_pos(&self) -> bool {
        self.0 & Self::CHANGE_POS != 0
    }
    pub fn has_angle(&self) -> bool {
        self.0 & Self::CHANGE_ANGLE != 0
    }
    pub fn has_wangle(&self) -> bool {
        self.0 & Self::CHANGE_WANGLE != 0
    }
    pub fn has_speed(&self) -> bool {
        self.0 & Self::CHANGE_SPEED != 0
    }
    pub fn has_fullness(&self) -> bool {
        self.0 & Self::CHANGE_FULLNESS != 0
    }
    pub fn is_dying(&self) -> bool {
        self.0 & Self::CHANGE_DYING != 0
    }
    pub fn is_dead(&self) -> bool {
        self.0 & Self::CHANGE_DEAD != 0
    }

    pub fn set_pos(&mut self) {
        self.0 |= Self::CHANGE_POS;
    }
    pub fn set_angle(&mut self) {
        self.0 |= Self::CHANGE_ANGLE;
    }
    pub fn set_wangle(&mut self) {
        self.0 |= Self::CHANGE_WANGLE;
    }
    pub fn set_speed(&mut self) {
        self.0 |= Self::CHANGE_SPEED;
    }
    pub fn set_fullness(&mut self) {
        self.0 |= Self::CHANGE_FULLNESS;
    }
    pub fn set_dying(&mut self) {
        self.0 |= Self::CHANGE_DYING;
    }
    pub fn set_dead(&mut self) {
        self.0 |= Self::CHANGE_DEAD;
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
}


pub fn angle_to_u8(angle: f32) -> u8 {
    let normalized = angle.rem_euclid(2.0 * PI);
    ((normalized / (2.0 * PI)) * 256.0) as u8
}


pub fn u8_to_angle(value: u8) -> f32 {
    (value as f32 / 256.0) * 2.0 * PI
}


pub fn angle_to_u24(angle: f32) -> u32 {
    let normalized = angle.rem_euclid(2.0 * PI);
    ((normalized / (2.0 * PI)) * 0xFFFFFF as f32) as u32
}


pub fn u24_to_angle(value: u32) -> f32 {
    ((value & 0xFFFFFF) as f32 / 0xFFFFFF as f32) * 2.0 * PI
}


pub fn is_clockwise(current_angle: f32, target_angle: f32) -> bool {
    let diff = (target_angle - current_angle).rem_euclid(2.0 * PI);
    diff > PI
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GameEndStatus {
    Normal = 0,
    HighScoreOfDay = 1,
    Disconnect = 2,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SnakeRemoveStatus {
    Left = 0,
    Died = 1,
}
