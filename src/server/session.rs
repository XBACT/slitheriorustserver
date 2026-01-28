

use crate::game::sector::SectorTracker;
use crate::protocol::incoming::ProtocolState;
use crate::protocol::types::SnakeId;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;


pub type SessionId = u64;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
   
    Connected,
   
    Handshake,
   
    Playing,
   
    Dead,
   
    Disconnected,
}


pub struct Session {
   
    pub id: SessionId,
   
    pub addr: SocketAddr,
   
    pub snake_id: Option<SnakeId>,
   
    pub state: SessionState,
   
    pub protocol: ProtocolState,
   
    pub last_packet_time: Instant,
   
    pub last_sent_time: Instant,
   
    pub death_time: Option<Instant>,
   
    pub name: String,
   
    pub custom_skin: Option<String>,
   
    pub skin: u8,
   
    pub tx: mpsc::UnboundedSender<Vec<u8>>,
   
    pub sector_tracker: SectorTracker,
   
    pub is_modern_protocol: bool,
}

impl Session {
   
    pub fn new(id: SessionId, addr: SocketAddr, tx: mpsc::UnboundedSender<Vec<u8>>) -> Self {
        Self {
            id,
            addr,
            snake_id: None,
            state: SessionState::Connected,
            protocol: ProtocolState::new(),
            last_packet_time: Instant::now(),
            last_sent_time: Instant::now(),
            death_time: None,
            name: String::new(),
            custom_skin: None,
            skin: 0,
            tx,
            sector_tracker: SectorTracker::new(),
            is_modern_protocol: false,
        }
    }

   
    pub fn send(&self, data: Vec<u8>) -> bool {
        self.tx.send(data).is_ok()
    }

   
    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Playing | SessionState::Handshake)
    }

   
    pub fn is_playing(&self) -> bool {
        self.state == SessionState::Playing && self.snake_id.is_some()
    }

   
    pub fn touch(&mut self) {
        self.last_packet_time = Instant::now();
    }

   
    pub fn idle_time_ms(&self) -> u64 {
        self.last_packet_time.elapsed().as_millis() as u64
    }

   
    pub fn mark_dead(&mut self) {
        self.state = SessionState::Dead;
        self.death_time = Some(Instant::now());
    }

   
    pub fn client_time_delta(&self) -> u16 {
        self.last_packet_time.elapsed().as_millis() as u16
    }

   
    pub fn time_since_last_sent(&self) -> u16 {
        self.last_sent_time.elapsed().as_millis() as u16
    }

   
    pub fn update_last_sent(&mut self) {
        self.last_sent_time = Instant::now();
    }
}


pub struct SessionManager {
   
    sessions: DashMap<SessionId, Session>,
   
    snake_to_session: DashMap<SnakeId, SessionId>,
   
    next_id: AtomicU64,
}

impl SessionManager {
   
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
            snake_to_session: DashMap::new(),
            next_id: AtomicU64::new(1),
        }
    }

   
    pub fn create_session(
        &self,
        addr: SocketAddr,
        tx: mpsc::UnboundedSender<Vec<u8>>,
    ) -> SessionId {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let session = Session::new(id, addr, tx);
        self.sessions.insert(id, session);
        id
    }

   
    pub fn get(&self, id: SessionId) -> Option<dashmap::mapref::one::Ref<SessionId, Session>> {
        self.sessions.get(&id)
    }

   
    pub fn get_mut(
        &self,
        id: SessionId,
    ) -> Option<dashmap::mapref::one::RefMut<SessionId, Session>> {
        self.sessions.get_mut(&id)
    }

   
    pub fn get_by_snake(
        &self,
        snake_id: SnakeId,
    ) -> Option<dashmap::mapref::one::Ref<SessionId, Session>> {
        self.snake_to_session
            .get(&snake_id)
            .and_then(|sid| self.sessions.get(&*sid))
    }

   
    pub fn set_snake(&self, session_id: SessionId, snake_id: SnakeId) {
        if let Some(mut session) = self.sessions.get_mut(&session_id) {
            session.snake_id = Some(snake_id);
            session.state = SessionState::Playing;
        }
        self.snake_to_session.insert(snake_id, session_id);
    }

   
    pub fn clear_snake(&self, snake_id: SnakeId) {
        if let Some((_, session_id)) = self.snake_to_session.remove(&snake_id) {
            if let Some(mut session) = self.sessions.get_mut(&session_id) {
                session.snake_id = None;
            }
        }
    }

   
    pub fn remove(&self, id: SessionId) -> Option<Session> {
        if let Some((_, session)) = self.sessions.remove(&id) {
            if let Some(snake_id) = session.snake_id {
                self.snake_to_session.remove(&snake_id);
            }
            Some(session)
        } else {
            None
        }
    }

   
    pub fn session_ids(&self) -> Vec<SessionId> {
        self.sessions.iter().map(|r| *r.key()).collect()
    }

   
    pub fn playing_session_ids(&self) -> Vec<SessionId> {
        self.sessions
            .iter()
            .filter(|r| r.is_playing())
            .map(|r| *r.key())
            .collect()
    }

   
    pub fn active_count(&self) -> usize {
        self.sessions.iter().filter(|r| r.is_active()).count()
    }

   
    pub fn playing_count(&self) -> usize {
        self.sessions.iter().filter(|r| r.is_playing()).count()
    }

   
    pub fn broadcast(&self, data: &[u8]) {
        for session in self.sessions.iter() {
            let _ = session.send(data.to_vec());
        }
    }

   
    pub fn broadcast_playing(&self, data: &[u8]) {
        for session in self.sessions.iter() {
            if session.is_playing() {
                let _ = session.send(data.to_vec());
            }
        }
    }

   
    pub fn cleanup_stale(&self, timeout_ms: u64) -> Vec<SessionId> {
        let mut stale = Vec::new();

        for session in self.sessions.iter() {
            if session.idle_time_ms() > timeout_ms {
                stale.push(session.id);
            }
        }

        for id in &stale {
            self.remove(*id);
        }

        stale
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}


pub type SharedSessionManager = Arc<SessionManager>;


pub fn create_session_manager() -> SharedSessionManager {
    Arc::new(SessionManager::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let manager = SessionManager::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        let id = manager.create_session(addr, tx);
        assert!(manager.get(id).is_some());
    }

    #[tokio::test]
    async fn test_snake_association() {
        let manager = SessionManager::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        let session_id = manager.create_session(addr, tx);
        let snake_id: SnakeId = 1;

        manager.set_snake(session_id, snake_id);

        let session = manager.get(session_id).unwrap();
        assert_eq!(session.snake_id, Some(snake_id));
        assert!(session.is_playing());
    }

    #[tokio::test]
    async fn test_session_removal() {
        let manager = SessionManager::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        let id = manager.create_session(addr, tx);
        assert!(manager.remove(id).is_some());
        assert!(manager.get(id).is_none());
    }
}
