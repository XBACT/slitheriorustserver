

use clap::Parser;


#[derive(Parser, Debug, Clone)]
#[command(name = "rust_slither")]
#[command(about = "Slither.io compatible game server")]
pub struct ServerArgs {
   
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

   
    #[arg(short, long)]
    pub verbose: bool,

   
    #[arg(short, long)]
    pub debug: bool,

   
    #[arg(long, default_value = "0")]
    pub bots: u16,

   
    #[arg(long, default_value = "true")]
    pub bot_respawn: bool,
}


#[derive(Debug, Clone)]
pub struct GameConfig {
   
    pub game_radius: u32,
   
    pub max_snake_parts: u16,
   
    pub sector_size: u16,
   
    pub sector_count_along_edge: u16,
   
    pub protocol_version: u8,
   
    pub frame_time_ms: u64,
   
    pub death_radius: u32,
   
    pub move_step_distance: u16,

   
    pub initial_bots: u16,
    pub bot_respawn: bool,

   
    pub food_spawn_rate: u16,
    pub spawn_prob_near_snake: u16,
    pub spawn_prob_on_snake: u16,
    pub spawn_prob_random: u16,

   
    pub human_snake_start_score: u16,
    pub bot_snake_start_score: u16,
    pub snake_min_length: u16,

   
    pub boost_cost: u16,
    pub boost_drop_size: u8,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            game_radius: 21600,
            max_snake_parts: 411,
            sector_size: 480,
            sector_count_along_edge: 90,
            protocol_version: 14,
            frame_time_ms: 8,
            death_radius: 21120,
            move_step_distance: 42,

            initial_bots: 0,
            bot_respawn: true,

            food_spawn_rate: 2,
            spawn_prob_near_snake: 25,
            spawn_prob_on_snake: 25,
            spawn_prob_random: 50,

            human_snake_start_score: 5,
            bot_snake_start_score: 5,
            snake_min_length: 2,

            boost_cost: 20,
            boost_drop_size: 10,
        }
    }
}

impl GameConfig {
   
    pub fn sector_diag_size(&self) -> u16 {
       
        680
    }

   
    pub fn total_sectors(&self) -> u32 {
        (self.sector_count_along_edge as u32).pow(2)
    }

   
    pub fn world_to_sector(&self, x: f32, y: f32) -> (u8, u8) {
        let sector_x = ((x + self.game_radius as f32) / self.sector_size as f32) as u8;
        let sector_y = ((y + self.game_radius as f32) / self.sector_size as f32) as u8;
        (sector_x, sector_y)
    }
}


pub mod snake_consts {
   
    pub const BASE_MOVE_SPEED: u16 = 172;
   
    pub const BOOST_SPEED: u16 = 448;
   
    pub const SPEED_ACCELERATION: u16 = 1000;
   
    pub const ANGULAR_SPEED: f32 = 4.125;
   
    pub const TAIL_K: f32 = 0.43;
   
    pub const TAIL_STEP_DISTANCE: f32 = 24.0;
   
    pub const PARTS_SKIP_COUNT: usize = 3;
   
    pub const PARTS_START_MOVE_COUNT: usize = 4;
   
    pub const ROT_STEP_INTERVAL_MS: u64 = 123;
   
    pub const AI_STEP_INTERVAL_MS: u64 = 250;
   
    pub const BOOST_COST: u32 = 20;
   
    pub const BOOST_DROP_SIZE: u8 = 10;
}


pub mod timing {
   
    pub const LEADERBOARD_INTERVAL_MS: u64 = 1000;
   
    pub const MINIMAP_INTERVAL_MS: u64 = 2000;
   
    pub const PING_TIMEOUT_MS: u64 = 30000;
   
    pub const SESSION_CLEANUP_INTERVAL_MS: u64 = 5000;
}
