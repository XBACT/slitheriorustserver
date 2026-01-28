

use crate::config::{timing, GameConfig};
use crate::game::sector::SectorEvent;
use crate::game::world::SharedWorld;
use crate::game::Snake;
use crate::protocol::incoming::{parse_incoming_packet, IncomingPacket, LoginPacket};
use crate::protocol::outgoing::*;
use crate::protocol::packet::PacketSerialize;
use crate::protocol::types::SnakeId;
use crate::server::session::{SessionId, SessionManager, SessionState, SharedSessionManager};
use bytes::BytesMut;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};


pub struct GameHandler {
   
    world: SharedWorld,
   
    sessions: SharedSessionManager,
   
    config: GameConfig,
   
    last_leaderboard: Instant,
   
    last_minimap: Instant,
}

impl GameHandler {
   
    pub fn new(world: SharedWorld, sessions: SharedSessionManager, config: GameConfig) -> Self {
        Self {
            world,
            sessions,
            config,
            last_leaderboard: Instant::now(),
            last_minimap: Instant::now(),
        }
    }

   
    pub fn on_connect(&self, session_id: SessionId) {
        info!("New connection: session {}", session_id);
       
    }

   
    pub fn on_disconnect(&self, session_id: SessionId) {
        debug!("Disconnection: session {}", session_id);

       
        let snake_id = self
            .sessions
            .get(session_id)
            .and_then(|s| s.snake_id);

       
        if let Some(snake_id) = snake_id {
            let mut world = self.world.write();
            world.remove_snake(snake_id);
        }

       
        self.sessions.remove(session_id);
    }

   
    pub fn on_packet(&self, session_id: SessionId, data: &[u8]) {
       
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.touch();
        }

       
        if !data.is_empty() {
            let hex_preview: String = data.iter()
                .take(32)
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            debug!("Packet from session {}: len={} cmd={} data=[{}]",
                   session_id, data.len(), data[0], hex_preview);
        }

       
        let protocol_version = self
            .sessions
            .get(session_id)
            .map(|s| s.protocol.protocol_version)
            .unwrap_or(14);

       
        match parse_incoming_packet(data, protocol_version) {
            Ok(packet) => {
                debug!("Parsed packet: {:?}", packet);
                self.handle_packet(session_id, packet);
            }
            Err(e) => {
                warn!("Failed to parse packet from session {}: {} (data len={})",
                      session_id, e, data.len());
            }
        }
    }

   
    fn handle_packet(&self, session_id: SessionId, packet: IncomingPacket) {
        match packet {
            IncomingPacket::ProtocolMode { want_etm } => {
                self.handle_protocol_mode(session_id, want_etm)
            }
            IncomingPacket::StartLogin => self.handle_start_login(session_id),
            IncomingPacket::Login(login) => self.handle_login(session_id, login),
            IncomingPacket::SetIdentity(identity) => {
                self.handle_identity(session_id, identity.skin, identity.nickname, identity.protocol_version)
            }
            IncomingPacket::Rotation(rot) => self.handle_rotation(session_id, rot),
            IncomingPacket::Angle(ang) => self.handle_angle(session_id, ang.to_radians()),
            IncomingPacket::StartAcceleration => self.handle_acceleration(session_id, true),
            IncomingPacket::StopAcceleration => self.handle_acceleration(session_id, false),
            IncomingPacket::Ping => self.handle_ping(session_id),
            IncomingPacket::VictoryMessage(msg) => self.handle_victory_message(session_id, msg),
            IncomingPacket::Unknown(cmd, data) => {
                debug!("Unknown packet cmd={} len={} from session {}", cmd, data.len(), session_id);
            }
        }
    }

   
    fn handle_protocol_mode(&self, session_id: SessionId, want_etm: bool) {
        info!("ProtocolMode from session {}: want_etm={}", session_id, want_etm);
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.protocol.want_etm = want_etm;
            session.protocol.handshake_complete = true;
        }
    }

    fn handle_start_login(&self, session_id: SessionId) {
        info!("StartLogin from session {}, sending PreInit", session_id);

       
       
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            if !session.protocol.handshake_complete {
                session.protocol.want_etm = true;
            }
            session.protocol.handshake_complete = true;
            session.state = SessionState::Handshake;
        }

        let packet = PacketPreInit;
        self.send_packet(session_id, &packet);
    }

    fn handle_login(&self, session_id: SessionId, login: LoginPacket) {
        info!(
            "Login from session {}: name={}, skin={}, version={}",
            session_id, login.nickname, login.skin, login.protocol_version
        );

        self.handle_identity(session_id, login.skin, login.nickname, login.protocol_version);
    }

   
    fn handle_identity(&self, session_id: SessionId, skin: u8, name: String, protocol_version: u8) {
        info!("Identity setup for session {}: name={}, skin={}, protocol={}",
              session_id, name, skin, protocol_version);

       
        {
            let mut session = match self.sessions.get_mut(session_id) {
                Some(s) => s,
                None => return,
            };
            session.name = name.clone();
            session.skin = skin;
            session.protocol.protocol_version = self.config.protocol_version;
            session.is_modern_protocol = self.config.protocol_version >= 25;
        }

       
        let snake_id = {
            let mut world = self.world.write();
            world.create_snake(name, skin)
        };

       
        self.sessions.set_snake(session_id, snake_id);

       
        self.send_init(session_id, snake_id);

       
        self.send_initial_state(session_id, snake_id);
    }

   
    fn send_init(&self, session_id: SessionId, snake_id: SnakeId) {
        let packet = PacketInit {
            game_radius: self.config.game_radius,
            max_snake_parts: self.config.max_snake_parts,
            sector_size: self.config.sector_size,
            sector_count_along_edge: self.config.sector_count_along_edge,
            protocol_version: self.config.protocol_version,
            snake_id, 
            ..Default::default()
        };

        self.send_packet(session_id, &packet);
    }

   
    fn send_initial_state(&self, session_id: SessionId, snake_id: SnakeId) {
        let world = self.world.read();

       
        let player_snake = match world.get_snake(snake_id) {
            Some(s) => s,
            None => return,
        };

        let (head_x, head_y) = player_snake.head_pos();

       
        let view_radius = 2000.0;
        let sectors = world.sectors.sectors_in_viewport(head_x, head_y, view_radius);

        for (sx, sy) in &sectors {
            self.send_packet(session_id, &PacketAddSector { x: *sx, y: *sy });

           
            if let Some(sector) = world.sectors.get(*sx, *sy) {
                let foods: Vec<FoodData> = sector.food.iter()
                    .map(|f| f.to_packet_data())
                    .collect();

                if !foods.is_empty() {
                    self.send_packet(session_id, &PacketSetFood {
                        sector_x: *sx,
                        sector_y: *sy,
                        sector_size: self.config.sector_size,
                        foods,
                    });
                }
            }
        }

       
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session
                .sector_tracker
                .update(&world.sectors, head_x, head_y, view_radius);
        }

       
        self.send_snake(session_id, player_snake);

       
        let (hx, hy) = player_snake.head_pos_u16();
        self.send_packet(session_id, &PacketMoveOwn {
            x: hx,
            y: hy,
        });

       
        for (id, snake) in world.snakes() {
            if *id != snake_id && !snake.dead {
                let (sx, sy) = snake.head_pos();
                if (sx - head_x).abs() < view_radius && (sy - head_y).abs() < view_radius {
                    self.send_snake(session_id, snake);
                }
            }
        }

       
        self.send_leaderboard(session_id);
    }

   
    fn send_snake(&self, session_id: SessionId, snake: &Snake) {
        let (head_x, head_y) = snake.head_pos();

       
        let body_parts: Vec<(f32, f32)> = snake.body.iter()
            .map(|part| (part.x, part.y))
            .collect();

        let packet = PacketAddSnake {
            snake_id: snake.id,
            skin: snake.skin,
            angle: snake.angle,
            target_angle: snake.target_angle,
            speed: snake.speed,
            fullness: snake.fullness as f32 / 100.0, 
            head_x,
            head_y,
            name: snake.name.clone(),
            custom_skin: snake.custom_skin.as_ref().map(|s| s.as_bytes().to_vec()),
            body_parts,
        };

        self.send_packet(session_id, &packet);
    }

   
    fn handle_rotation(
        &self,
        session_id: SessionId,
        rot: crate::protocol::incoming::RotationPacket,
    ) {
        let snake_id = match self.sessions.get(session_id) {
            Some(s) => match s.snake_id {
                Some(id) => id,
                None => return,
            },
            None => return,
        };

        let mut world = self.world.write();
        if let Some(snake) = world.get_snake_mut(snake_id) {
           
            let intensity = rot.intensity() as f32 / 127.0;
            let turn_rate = std::f32::consts::PI * intensity;

            let delta = if rot.is_clockwise() {
                -turn_rate
            } else {
                turn_rate
            };

            let new_angle = snake.angle + delta * 0.1;
            snake.set_target_angle(new_angle);
        }
    }

   
    fn handle_angle(&self, session_id: SessionId, angle: f32) {
        let snake_id = match self.sessions.get(session_id) {
            Some(s) => match s.snake_id {
                Some(id) => id,
                None => return,
            },
            None => return,
        };

        let mut world = self.world.write();
        if let Some(snake) = world.get_snake_mut(snake_id) {
            snake.set_target_angle(angle);
        }
    }

   
    fn handle_acceleration(&self, session_id: SessionId, accelerating: bool) {
        let snake_id = match self.sessions.get(session_id) {
            Some(s) => match s.snake_id {
                Some(id) => id,
                None => return,
            },
            None => return,
        };

        let mut world = self.world.write();
        if let Some(snake) = world.get_snake_mut(snake_id) {
            snake.set_accelerating(accelerating);
        }
    }

   
    fn handle_ping(&self, session_id: SessionId) {
        self.send_packet(session_id, &PacketPong);
    }

   
    fn handle_victory_message(&self, session_id: SessionId, message: String) {
        debug!("Victory message from {}: {}", session_id, message);
       
    }

   
    pub fn tick(&mut self, dt_ms: u64) {
       
        {
            let mut world = self.world.write();
            world.tick(dt_ms);
        }

       
        self.broadcast_updates();

       
        let now = Instant::now();

        if now.duration_since(self.last_leaderboard).as_millis() as u64
            >= timing::LEADERBOARD_INTERVAL_MS
        {
            self.last_leaderboard = now;
            self.broadcast_leaderboard();
        }

        if now.duration_since(self.last_minimap).as_millis() as u64 >= timing::MINIMAP_INTERVAL_MS {
            self.last_minimap = now;
            self.broadcast_minimap();
        }

       
        let stale = self.sessions.cleanup_stale(timing::PING_TIMEOUT_MS);
        for session_id in stale {
            self.on_disconnect(session_id);
        }
    }

   
    fn broadcast_updates(&self) {
        let world = self.world.read();

       
        for session_id in self.sessions.playing_session_ids() {
            let snake_id = match self.sessions.get(session_id) {
                Some(s) => match s.snake_id {
                    Some(id) => id,
                    None => continue,
                },
                None => continue,
            };

           
            let player_pos = match world.get_snake(snake_id) {
                Some(s) => s.head_pos(),
                None => continue,
            };

            let view_radius = 2000.0;

           
            let sector_events = {
                let mut session = match self.sessions.get_mut(session_id) {
                    Some(s) => s,
                    None => continue,
                };
                session
                    .sector_tracker
                    .update(&world.sectors, player_pos.0, player_pos.1, view_radius)
            };

           
            for event in sector_events {
                match event {
                    SectorEvent::Entered { x, y } => {
                        self.send_packet(session_id, &PacketAddSector { x, y });

                       
                        if let Some(sector) = world.sectors.get(x, y) {
                            let foods: Vec<FoodData> = sector.food.iter()
                                .map(|f| f.to_packet_data())
                                .collect();

                            if !foods.is_empty() {
                                self.send_packet(session_id, &PacketSetFood {
                                    sector_x: x,
                                    sector_y: y,
                                    sector_size: self.config.sector_size,
                                    foods,
                                });
                            }
                        }
                    }
                    SectorEvent::Left { x, y } => {
                        self.send_packet(session_id, &PacketRemoveSector { x, y });
                    }
                }
            }

           
            for changed_id in world.changed_snakes() {
                if let Some(snake) = world.get_snake(*changed_id) {
                    let (sx, sy) = snake.head_pos();
                    let is_own_snake = snake.id == snake_id;

                   
                    if (sx - player_pos.0).abs() < view_radius
                        && (sy - player_pos.1).abs() < view_radius
                    {
                       
                        if snake.changes.has_pos() {
                            let (dx, dy) = snake.head_delta();
                            if dx.abs() < 128 && dy.abs() < 128 {
                               
                                if is_own_snake {
                                    self.send_packet(
                                        session_id,
                                        &PacketMoveRelOwn { dx, dy },
                                    );
                                } else {
                                    self.send_packet(
                                        session_id,
                                        &PacketMoveRel {
                                            snake_id: snake.id,
                                            dx,
                                            dy,
                                        },
                                    );
                                }
                            } else {
                                let (x, y) = snake.head_pos_u16();
                               
                                if is_own_snake {
                                    self.send_packet(
                                        session_id,
                                        &PacketMoveOwn { x, y },
                                    );
                                } else {
                                    self.send_packet(
                                        session_id,
                                        &PacketMove {
                                            snake_id: snake.id,
                                            x,
                                            y,
                                        },
                                    );
                                }
                            }
                        }

                        if snake.changes.has_angle() || snake.changes.has_wangle() {
                            let clockwise = crate::protocol::types::is_clockwise(
                                snake.angle,
                                snake.target_angle,
                            );
                            self.send_packet(
                                session_id,
                                &PacketRotation {
                                    snake_id: snake.id,
                                    angle: snake.angle,
                                    target_angle: snake.target_angle,
                                    speed: snake.speed,
                                    include_angle: true,
                                    include_target: true,
                                    clockwise,
                                },
                            );
                        }

                        if snake.changes.has_fullness() {
                            self.send_packet(
                                session_id,
                                &PacketSetFullness {
                                    snake_id: snake.id,
                                    fullness: snake.fullness as f32 / 100.0,
                                },
                            );
                        }
                    }
                }
            }

           
            for (eater_id, food) in world.eaten_food() {
                self.send_packet(
                    session_id,
                    &PacketEatFood {
                        snake_id: *eater_id,
                        food_x: food.x,
                        food_y: food.y,
                        sector_size: world.config.sector_size,
                    },
                );
            }

           
            for food in world.new_food() {
                let (sx, sy) = food.sector_coords(world.config.sector_size);

               
                if let Some(session) = self.sessions.get(session_id) {
                    if session.sector_tracker.is_visible(sx, sy) {
                        self.send_packet(
                            session_id,
                            &PacketSpawnFood {
                                food: food.to_packet_data(),
                                sector_size: world.config.sector_size,
                            },
                        );
                    }
                }
            }
        }
    }

   
    fn send_leaderboard(&self, session_id: SessionId) {
        let world = self.world.read();

        let snake_id = match self.sessions.get(session_id) {
            Some(s) => s.snake_id.unwrap_or(0),
            None => return,
        };

        let player_rank = world.player_rank(snake_id).unwrap_or(0) as u8;
        let leaderboard = world.leaderboard(10);

        let entries: Vec<LeaderboardEntry> = leaderboard
            .iter()
            .map(|(snake, _score)| LeaderboardEntry {
                parts: snake.length() as u16,
                fullness: snake.fullness as f32 / 100.0,
                font_color: snake.skin,
                name: snake.name.clone(),
            })
            .collect();

        let packet = PacketLeaderboard {
            player_rank,
            local_rank: player_rank as u16,
            player_count: world.snake_count() as u16,
            entries,
        };

        self.send_packet(session_id, &packet);
    }

   
    fn broadcast_leaderboard(&self) {
        for session_id in self.sessions.playing_session_ids() {
            self.send_leaderboard(session_id);
        }
    }

   
    fn broadcast_minimap(&self) {
        let world = self.world.read();
        let minimap_data = world.minimap_data(80);

        for session_id in self.sessions.playing_session_ids() {
            let use_modern = self
                .sessions
                .get(session_id)
                .map(|s| s.is_modern_protocol)
                .unwrap_or(false);

            let packet = PacketMinimap {
                grid_size: 80,
                data: minimap_data.clone(),
                use_modern,
            };

            self.send_packet(session_id, &packet);
        }
    }

   
   
    fn send_packet<T: PacketSerialize>(&self, session_id: SessionId, packet: &T) {
        let packet_bytes = packet.to_bytes();

        if let Some(mut session) = self.sessions.get_mut(session_id) {
            let data = if session.protocol.want_etm {
               
                let etm = session.time_since_last_sent();
                let mut framed = Vec::with_capacity(2 + packet_bytes.len());
                framed.push((etm >> 8) as u8);
                framed.push((etm & 0xFF) as u8);
                framed.extend_from_slice(&packet_bytes);
                framed
            } else {
                packet_bytes.to_vec()
            };

            session.update_last_sent();
            let _ = session.send(data);
        }
    }
}
