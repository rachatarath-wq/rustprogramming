// ep42: TCP relay server — accepts many clients and relays each line to all.
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

// Shared list of all connected clients (thread-safe).
type Clients = Arc<Mutex<Vec<TcpStream>>>;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7777").expect("bind failed");
    println!("Server ready at 0.0.0.0:7777");

    let clients: Clients = Arc::new(Mutex::new(Vec::new()));

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                let clients = Arc::clone(&clients);
                clients.lock().unwrap().push(stream.try_clone().unwrap());
                thread::spawn(move || handle(stream, clients));
            }
            Err(e) => eprintln!("accept failed: {}", e),
        }
    }
}

// Read lines from one client and broadcast them to everyone.
fn handle(stream: TcpStream, clients: Clients) {
    let reader = BufReader::new(stream.try_clone().unwrap());
    for line in reader.lines() {
        let msg = match line {
            Ok(m) => m,
            Err(_) => break,
        };
        let mut guard = clients.lock().unwrap();
        for client in guard.iter_mut() {
            let _ = writeln!(client, "{}", msg);
        }
    }
}
