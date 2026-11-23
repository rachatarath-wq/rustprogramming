// ep45: server holds the authoritative world and sends it to new players.
use miniroblox_server::db;
use miniroblox_server::{recv_msg, send_msg, Block, Message};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type Clients = Arc<Mutex<Vec<TcpStream>>>;
type World = Arc<Mutex<Vec<Block>>>;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7777").expect("bind failed");
    println!("Server ready at 0.0.0.0:7777");

    let db: Arc<Mutex<rusqlite::Connection>> = Arc::new(Mutex::new(db::init()));

    // Load the world from SQLite at startup; seed a floor if empty.
    let world: World = {
        let conn = db.lock().unwrap();
        let mut w = db::load_blocks(&conn);
        if w.is_empty() {
            for x in -2..=2 {
                for z in -2..=2 {
                    let b = Block { x, y: 0, z, block: 1 };
                    db::save_block(&conn, &b);
                    w.push(b);
                }
            }
        }
        Arc::new(Mutex::new(w))
    };
    println!("world ready: {} blocks", world.lock().unwrap().len());

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
        let world = Arc::clone(&world);
        clients.lock().unwrap().push(stream.try_clone().unwrap());
        thread::spawn(move || handle(stream, clients, db, world));
    }
}

fn handle(
    mut stream: TcpStream,
    clients: Clients,
    db: Arc<Mutex<rusqlite::Connection>>,
    world: World,
) {
    loop {
        let msg = match recv_msg(&mut stream) {
            Ok(m) => m,
            Err(_) => break,
        };
        println!("got: {:?}", msg);

        match &msg {
            // A new player: save to DB and send them the whole world.
            Message::Join { name } => {
                let conn = db.lock().unwrap();
                db::save_player(&conn, name, 0.0, 0.0, 0.0);
                let blocks = world.lock().unwrap().clone();
                let n = blocks.len();
                let _ = send_msg(&mut stream, &Message::World { blocks });
                println!("{} joined, sent {} blocks", name, n);
            }
            // Everything else: relay to every client.
            _ => {
                let mut guard = clients.lock().unwrap();
                for c in guard.iter_mut() {
                    let _ = send_msg(c, &msg);
                }
            }
        }
    }
}
