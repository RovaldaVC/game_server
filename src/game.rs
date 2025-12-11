use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex};

use crate::message::{ClientCommand, ServerMessage, PlayerState};
use crate::server::ClientList; // Note: No longer importing 'broadcast'

#[derive(Debug, Default)]
pub struct GameState {
    pub players: Vec<PlayerState>,
}

pub type SharedState = Arc<Mutex<GameState>>;

pub async fn run_game_loop(
    state: SharedState,
    mut cmd_rx: mpsc::Receiver<ClientCommand>,
    // We keep clients here just in case, but typically unused for pure logic
    _clients: ClientList,
    // NEW: The channel sender for sending world snapshots
    broadcast_tx: mpsc::Sender<ServerMessage>,
) {
    let tick = Duration::from_millis(33); // ~30 FPS

    loop {
        // process all queued commands
        while let Ok(cmd) = cmd_rx.try_recv() {
            let mut s = state.lock().await;

            match cmd {
                ClientCommand::Join { name: _ } => {
                    let id = s.players.len() as u64 + 1;
                    s.players.push(PlayerState {
                        id,
                        x: 0.0,
                        y: 0.0,
                        hp: 100,
                    });
                }

                ClientCommand::Move { x, y } => {
                    // This still only moves the first player (simplified logic)
                    if let Some(p) = s.players.first_mut() {
                        p.x += x;
                        p.y += y;
                    }
                }

                ClientCommand::Attack { target: _ } => {
                    // to implement later
                }
            }
        }

        // send snapshot
        {
            let s = state.lock().await;
            let msg = ServerMessage::WorldSnapshot {
                players: s.players.clone(),
            };
            
            // CRITICAL CHANGE: Send the message to the Broadcaster task via channel.
            // This is non-blocking for the Game Loop!
            let _ = broadcast_tx.send(msg).await;
        }

        tokio::time::sleep(tick).await;
    }
}