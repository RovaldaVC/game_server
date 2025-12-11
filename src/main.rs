use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

mod message;
mod game;
mod server;

use game::{run_game_loop, GameState, SharedState};
use message::{ClientCommand, ServerMessage};
use server::{start_server, run_broadcaster, ClientList};

#[tokio::main]
async fn main() {
    let state: SharedState = Arc::new(Mutex::new(GameState::default()));
    let clients: ClientList = Arc::new(Mutex::new(Vec::new()));

    // 1. Client Commands Channel (Input: Net -> Game)
    let (cmd_tx, cmd_rx) = mpsc::channel::<ClientCommand>(256);

    // 2. Server Messages Channel (Output: Game -> Broadcaster)
    let (broadcast_tx, broadcast_rx) = mpsc::channel::<ServerMessage>(256);

    // Server Task: Handles TCP connections and sends ClientCommands to the Game Loop
    let server_task = {
        let state = Arc::clone(&state);
        let clients = Arc::clone(&clients);
        let cmd_tx = cmd_tx.clone();
        tokio::spawn(async move {
            start_server(state, cmd_tx, clients).await;
        })
    };

    // Game Loop Task: Handles game logic and sends ServerMessages to the Broadcaster
    let game_task = {
        let state = Arc::clone(&state);
        let clients = Arc::clone(&clients);
        let broadcast_tx = broadcast_tx.clone(); // Pass the output Sender
        tokio::spawn(async move {
            run_game_loop(state, cmd_rx, clients, broadcast_tx).await;
        })
    };

    // Broadcaster Task: Receives ServerMessages and writes them to all TCP clients
    let broadcast_task = {
        let clients = Arc::clone(&clients);
        tokio::spawn(async move {
            run_broadcaster(broadcast_rx, clients).await; // Pass the output Receiver
        })
    };

    let _ = tokio::join!(server_task, game_task, broadcast_task);
}