// Test client: join and print the world the server sends back.
use miniroblox_server::{recv_msg, send_msg, Message};
use std::net::TcpStream;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:7777").unwrap();
    send_msg(&mut stream, &Message::Join { name: "player1".into() }).unwrap();
    match recv_msg(&mut stream).unwrap() {
        Message::World { blocks } => {
            println!("world received: {} blocks", blocks.len());
            println!("first 3: {:?}", &blocks[..3.min(blocks.len())]);
        }
        other => println!("unexpected: {:?}", other),
    }
}
