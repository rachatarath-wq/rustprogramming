// ep44: server now persists players to SQLite (survives restart).
use miniroblox_server::db;
use miniroblox_server::{recv_msg, send_msg, Message};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type Clients = Arc<Mutex<Vec<TcpStream>>>;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7777").expect("bind failed");
    println!("Server ready at 0.0.0.0:7777");

    // One shared database connection (SQLite is not Sync, so wrap in a Mutex).
    let db: Arc<Mutex<rusqlite::Connection>> = Arc::new(Mutex::new(db::init()));

    let clients: Clients = Arc::new(Mutex::new(Vec::new()));

    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(s) => s,
            Err(e) => {
                eprintln!("accept failed: {}", e);
                continue;
            }
        };
        let clients = Arc::clone(&clients);
        let db = Arc::clone(&db);
        clients.lock().unwrap().push(stream.try_clone().unwrap());
        thread::spawn(move || handle(stream, clients, db));
    }
}

fn handle(
    mut stream: TcpStream,
    clients: Clients,
    db: Arc<Mutex<rusqlite::Connection>>,
) {
    loop {
        let msg = match recv_msg(&mut stream) {
            Ok(m) => m,
            Err(_) => break,
        };
        println!("got: {:?}", msg);

        // Persist a Join to SQLite, then print the full player list.
        if let Message::Join { name } = &msg {
            let conn = db.lock().unwrap();
            db::save_player(&conn, name, 0.0, 0.0, 0.0);
            println!("players: {:?}", db::list_players(&conn));
        }

        let mut guard = clients.lock().unwrap();
        for c in guard.iter_mut() {
            let _ = send_msg(c, &msg);
        }
    }
}
