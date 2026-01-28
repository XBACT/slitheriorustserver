

use crate::config::GameConfig;
use crate::game::world::{create_shared_world, SharedWorld};
use crate::server::handler::GameHandler;
use crate::server::session::{create_session_manager, SessionId, SharedSessionManager};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};


type SharedHandler = Arc<RwLock<GameHandler>>;


pub async fn run_server(port: u16, config: GameConfig) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    info!("Slither.io server listening on {}", addr);

   
    let world = create_shared_world(config.clone());
    let sessions = create_session_manager();
    let handler = Arc::new(RwLock::new(GameHandler::new(
        world.clone(),
        sessions.clone(),
        config.clone(),
    )));

   
    let game_handler = handler.clone();
    let frame_time = config.frame_time_ms;
    tokio::spawn(async move {
        game_loop(game_handler, frame_time).await;
    });

   
    while let Ok((stream, addr)) = listener.accept().await {
        let handler = handler.clone();
        let sessions = sessions.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr, handler, sessions).await {
                error!("Connection error from {}: {}", addr, e);
            }
        });
    }

    Ok(())
}


async fn game_loop(handler: SharedHandler, frame_time_ms: u64) {
    let mut ticker = interval(Duration::from_millis(frame_time_ms));

    loop {
        ticker.tick().await;

        let mut handler = handler.write().await;
        handler.tick(frame_time_ms);
    }
}


async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    handler: SharedHandler,
    sessions: SharedSessionManager,
) -> anyhow::Result<()> {
    info!("New connection from {}", addr);

   
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

   
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();

   
    let session_id = sessions.create_session(addr, tx);

   
    {
        let handler = handler.read().await;
        handler.on_connect(session_id);
    }

   
    let send_task = tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            if ws_sender.send(Message::Binary(data)).await.is_err() {
                break;
            }
        }
    });

   
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(msg) => {
                match msg {
                    Message::Binary(data) => {
                        let handler = handler.read().await;
                        handler.on_packet(session_id, &data);
                    }
                    Message::Text(text) => {
                       
                        let handler = handler.read().await;
                        handler.on_packet(session_id, text.as_bytes());
                    }
                    Message::Ping(data) => {
                       
                    }
                    Message::Pong(_) => {
                       
                    }
                    Message::Close(_) => {
                        break;
                    }
                    Message::Frame(_) => {
                       
                    }
                }
            }
            Err(e) => {
                warn!("WebSocket error from {}: {}", addr, e);
                break;
            }
        }
    }

   
    info!("Connection closed from {}", addr);
    send_task.abort();

    {
        let handler = handler.read().await;
        handler.on_disconnect(session_id);
    }

    Ok(())
}


#[derive(Debug, Clone, Default)]
pub struct ServerStats {
    pub connections: usize,
    pub players: usize,
    pub snakes: usize,
    pub food: usize,
    pub tick_count: u64,
}

impl ServerStats {
    pub fn gather(world: &SharedWorld, sessions: &SharedSessionManager) -> Self {
        let world = world.read();

        Self {
            connections: sessions.active_count(),
            players: sessions.playing_count(),
            snakes: world.snake_count(),
            food: world.sectors.total_food(),
            tick_count: world.tick_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
       
        let config = GameConfig::default();
        let world = create_shared_world(config.clone());
        let sessions = create_session_manager();

        let stats = ServerStats::gather(&world, &sessions);
        assert_eq!(stats.players, 0);
    }
}
