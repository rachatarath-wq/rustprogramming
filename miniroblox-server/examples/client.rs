// Test client: connect, send a Join, print the relayed reply.
use miniroblox_server::{recv_msg, send_msg, Message};
use std::net::TcpStream;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:7777").unwrap();
    send_msg(&mut stream, &Message::Join { name: "player1".into() }).unwrap();
    let reply = recv_msg(&mut stream).unwrap();
    println!("reply: {:?}", reply);
}
