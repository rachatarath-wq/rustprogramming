// เน็ตเวิร์ก: โปรโตคอลเดียวกับเซิร์ฟเวอร์ (Message + bincode framing)
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Join { name: String },
    Move { name: String, x: f32, y: f32, z: f32 },
    World { blocks: Vec<Block> },
}

pub fn send_msg(stream: &mut TcpStream, msg: &Message) -> std::io::Result<()> {
    let bytes = bincode::serialize(msg).unwrap();
    let len = bytes.len() as u32;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&bytes)?;
    Ok(())
}

pub fn recv_msg(stream: &mut TcpStream) -> std::io::Result<Message> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    Ok(bincode::deserialize(&buf).unwrap())
}

// เชื่อมต่อ + join + รับโลกกลับมา (ถ้าเซิร์ฟเวอร์ไม่เปิด -> None, เล่นเดี่ยว)
pub fn connect_and_join(addr: &str, name: &str) -> Option<(TcpStream, Vec<Block>)> {
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().ok()?,
        Duration::from_secs(2),
    )
    .ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(3))).ok()?;
    send_msg(&mut stream, &Message::Join { name: name.into() }).ok()?;
    let blocks = match recv_msg(&mut stream).ok()? {
        Message::World { blocks } => blocks,
        _ => return None,
    };
    stream.set_read_timeout(None).ok();
    Some((stream, blocks))
}
