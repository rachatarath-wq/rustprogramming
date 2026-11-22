// ep43: server relays typed (bincode) messages instead of raw lines.
use miniroblox_server::{recv_msg, send_msg};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type Clients = Arc<Mutex<Vec<TcpStream>>>;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7777").expect("bind failed");
    println!("Server ready at 0.0.0.0:7777");

    let clients: Clients = Arc::new(Mutex::new(Vec::new()));

    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(s) => s,
            Err(e) => { eprintln!("accept failed: {}", e); continue; }
        };
        let clients = Arc::clone(&clients);
        clients.lock().unwrap().push(stream.try_clone().unwrap());
        thread::spawn(move || handle(stream, clients));
    }
}

fn handle(mut stream: TcpStream, clients: Clients) {
    loop {
        let msg = match recv_msg(&mut stream) {
            Ok(m) => m,
            Err(_) => break,
        };
        println!("got: {:?}", msg);
        let mut guard = clients.lock().unwrap();
        for c in guard.iter_mut() {
            let _ = send_msg(c, &msg);
        }
    }
}
