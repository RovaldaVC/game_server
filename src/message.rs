use serde::{Serialize, Deserialize};

//
// CLIENT → SERVER
//
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientCommand {
    Move { x: f32, y: f32 },
    Attack { target: u64 },
    Join { name: String },
}

//
// SERVER → CLIENT
//
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Welcome { msg: String },
    PlayerUpdate { id: u64, x: f32, y: f32, hp: u32 },
    WorldSnapshot { players: Vec<PlayerState> },
    Error { msg: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub hp: u32,
}