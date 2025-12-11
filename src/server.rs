use crate::message::{ClientCommand, ServerMessage};
use crate::game::SharedState;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use std::sync::Arc;

pub type ClientList = Arc<Mutex<Vec<tokio::net::tcp::OwnedWriteHalf>>>;

pub async fn start_server(
    state: SharedState,
    cmd_tx: mpsc::Sender<ClientCommand>,
    clients: ClientList,
) {
    let listener = TcpListener::bind("0.0.0.0:9000")
        .await
        .expect("bind failed");
    println!("Server listening on port 9000");

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        println!("Client connected: {}", addr);

        let state = Arc::clone(&state);
        let cmd_tx = cmd_tx.clone();
        let clients = Arc::clone(&clients);

        tokio::spawn(async move {
            handle_client(stream, state, cmd_tx, clients).await;
        });
    }
}

pub async fn handle_client(
    stream: TcpStream,
    _state: SharedState,
    cmd_tx: mpsc::Sender<ClientCommand>,
    clients: ClientList,
) {
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Add the writer to the shared list when the client connects
    {
        let mut list = clients.lock().await;
        list.push(writer);
    }

    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await.unwrap_or(0);
        if n == 0 {
            // Client disconnected
            break;
        }

        if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&line) {
            // Send the command to the game loop
            let _ = cmd_tx.send(cmd).await;
        }
    }

    // Client disconnection cleanup will occur in the broadcaster on write failure.
    println!("Client disconnected");
}


// NEW: Dedicated Broadcaster Task
pub async fn run_broadcaster(
    mut broadcast_rx: mpsc::Receiver<ServerMessage>,
    clients: ClientList,
) {
    println!("Broadcaster task started, handling all outbound network I/O...");
    
    // Await messages from the Game Loop
    while let Some(msg) = broadcast_rx.recv().await {
        // Prepare the message (only once per frame)
        let json = serde_json::to_string(&msg).unwrap() + "\n";
        
        let mut list = clients.lock().await;
        let mut i = 0;

        // Iterate and write to every client writer
        while i < list.len() {
            // write_all is awaited, but this task is separate from the game loop!
            if list[i].write_all(json.as_bytes()).await.is_err() {
                // Remove client on write error (connection loss)
                list.remove(i);
            } else {
                i += 1;
            }
        }
    }
}