// Shared protocol: message types + length-prefixed bincode framing.
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;

pub mod db;

// One block in the world.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block: u8,
}

// Messages exchanged between client and server.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Join { name: String },
    Move { x: f32, y: f32, z: f32 },
    World { blocks: Vec<Block> },
}

// Send one message: [4-byte length][bincode bytes].
pub fn send_msg(stream: &mut TcpStream, msg: &Message) -> std::io::Result<()> {
    let bytes = bincode::serialize(msg).unwrap();
    let len = bytes.len() as u32;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&bytes)?;
    Ok(())
}

// Receive one message (blocking).
pub fn recv_msg(stream: &mut TcpStream) -> std::io::Result<Message> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    Ok(bincode::deserialize(&buf).unwrap())
}
