// Test client: login, get world + spawn + coins, move with color, break a block.
use miniroblox_server::{recv_msg, send_msg, Message};
use std::net::TcpStream;

fn main() {
    let name = std::env::args().nth(1).unwrap_or_else(|| "p1".into());
    let addr = std::env::var("SERVER").unwrap_or_else(|_| "127.0.0.1:7777".into());
    let mut stream = TcpStream::connect(&addr).unwrap();
    send_msg(&mut stream, &Message::Login { code: format!("dev:{}", name) }).unwrap();

    // Login -> World, Spawn (offset), Coins (balance)
    match recv_msg(&mut stream).unwrap() {
        Message::World { blocks } => println!("{} world: {} blocks", name, blocks.len()),
        other => println!("{} unexpected: {:?}", name, other),
    }
    match recv_msg(&mut stream).unwrap() {
        Message::Spawn { x, z } => println!("{} spawn offset: ({}, {})", name, x, z),
        other => println!("{} unexpected2: {:?}", name, other),
    }
    match recv_msg(&mut stream).unwrap() {
        Message::Coins { coins } => println!("{} coins: {}", name, coins),
        other => println!("{} unexpected3: {:?}", name, other),
    }

    // Move with a color (server broadcasts color + owner to others)
    send_msg(
        &mut stream,
        &Message::Move { name: String::new(), x: 1.0, y: 2.0, z: 3.0, color: 7, owner: false },
    )
    .unwrap();

    // Break a block -> +1 coin
    send_msg(&mut stream, &Message::BlockBreak { name: String::new(), x: 0, y: 0, z: 0 }).unwrap();
    match recv_msg(&mut stream).unwrap() {
        Message::Coins { coins } => println!("{} after break: {} coins", name, coins),
        other => println!("{} unexpected4: {:?}", name, other),
    }
}
