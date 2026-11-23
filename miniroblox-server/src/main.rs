// ep48: Google login — server verifies the ID token and uses the verified email.
use miniroblox_server::auth;
use miniroblox_server::db;
use miniroblox_server::{recv_msg, send_msg, Block, Message};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type Clients = Arc<Mutex<HashMap<String, TcpStream>>>;
type World = Arc<Mutex<Vec<Block>>>;

fn main() {
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(7777);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("bind failed");
    println!("Server ready at 0.0.0.0:{}", port);

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

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

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
        thread::spawn(move || handle(stream, clients, db, world));
    }
}

fn handle(
    mut stream: TcpStream,
    clients: Clients,
    db: Arc<Mutex<rusqlite::Connection>>,
    world: World,
) {
    let mut my_name: Option<String> = None;
    loop {
        let msg = match recv_msg(&mut stream) {
            Ok(m) => m,
            Err(_) => break,
        };
        println!("got: {:?}", msg);

        match &msg {
            // Login with an OAuth authorization code -> exchange + verify.
            Message::Login { code } => {
                let email = match auth::login_with_code(code) {
                    Some(e) => e,
                    None => {
                        eprintln!("login rejected: invalid code");
                        break;
                    }
                };
                let conn = db.lock().unwrap();
                db::save_player(&conn, &email, 0.0, 0.0, 0.0);
                let blocks = world.lock().unwrap().clone();
                let n = blocks.len();
                let _ = send_msg(&mut stream, &Message::World { blocks });
                clients.lock().unwrap().insert(email.clone(), stream.try_clone().unwrap());
                my_name = Some(email.clone());
                println!("{} logged in, sent {} blocks", email, n);
            }
            // Movement: overwrite the name with the verified identity (anti-spoofing)
            // then broadcast to everyone EXCEPT the sender.
            Message::Move { name: _, x, y, z } => {
                let out = Message::Move {
                    name: my_name.clone().unwrap_or_default(),
                    x: *x,
                    y: *y,
                    z: *z,
                };
                let mut guard = clients.lock().unwrap();
                for (name, c) in guard.iter_mut() {
                    if Some(name) != my_name.as_ref() {
                        let _ = send_msg(c, &out);
                    }
                }
            }
            _ => {}
        }
    }
    // Client disconnected: remove from the player list.
    if let Some(name) = my_name {
        clients.lock().unwrap().remove(&name);
        println!("{} left", name);
    }
}
