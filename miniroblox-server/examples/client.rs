// Test client: login, get world + coins, break a block, print the coin balance.
use miniroblox_server::{recv_msg, send_msg, Message};
use std::net::TcpStream;

fn main() {
    let name = std::env::args().nth(1).unwrap_or_else(|| "p1".into());
    let addr = std::env::var("SERVER").unwrap_or_else(|_| "127.0.0.1:7777".into());
    let mut stream = TcpStream::connect(&addr).unwrap();
    send_msg(&mut stream, &Message::Login { code: format!("dev:{}", name) }).unwrap();

    // Login -> World, then Coins (starting balance).
    match recv_msg(&mut stream).unwrap() {
        Message::World { blocks } => println!("{} got world: {} blocks", name, blocks.len()),
        other => println!("{} unexpected: {:?}", name, other),
    }
    match recv_msg(&mut stream).unwrap() {
        Message::Coins { coins } => println!("{} starting coins: {}", name, coins),
        other => println!("{} unexpected2: {:?}", name, other),
    }

    // Break a block -> +1 coin (server answers with the new balance).
    send_msg(
        &mut stream,
        &Message::BlockBreak { name: name.clone(), x: 0, y: 0, z: 0 },
    )
    .unwrap();
    match recv_msg(&mut stream).unwrap() {
        Message::Coins { coins } => println!("{} after break: {} coins", name, coins),
        other => println!("{} unexpected3: {:?}", name, other),
    }
}
