// Test client: join, get world, send one Move, then print any broadcast it sees.
use miniroblox_server::{recv_msg, send_msg, Message};
use std::net::TcpStream;
use std::time::Duration;

fn main() {
    let name = std::env::args().nth(1).unwrap_or_else(|| "p1".into());
    let addr = std::env::var("SERVER").unwrap_or_else(|_| "127.0.0.1:7777".into());
    let mut stream = TcpStream::connect(&addr).unwrap();
    send_msg(&mut stream, &Message::Login { code: format!("dev:{}", name) }).unwrap();
    match recv_msg(&mut stream).unwrap() {
        Message::World { blocks } => println!("{} got world: {} blocks", name, blocks.len()),
        other => println!("{} unexpected: {:?}", name, other),
    }
    // Send a Move, then listen for broadcasts for 4 seconds.
    send_msg(&mut stream, &Message::Move { name: name.clone(), x: 1.0, y: 2.0, z: 3.0 }).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(4))).ok();
    loop {
        match recv_msg(&mut stream) {
            Ok(Message::Move { name: n, x, y, z }) => {
                println!("{} sees {} at {},{},{}", name, n, x, y, z);
            }
            Ok(other) => println!("{} got: {:?}", name, other),
            Err(_) => break,
        }
    }
}
