// Admin web UI: a tiny HTTP server showing live server status.
use crate::Block;
use rusqlite::Connection;
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Response, Server};

pub type Clients = Arc<Mutex<HashMap<String, TcpStream>>>;
pub type World = Arc<Mutex<Vec<Block>>>;

pub fn run(port: u16, clients: Clients, world: World, db: Arc<Mutex<Connection>>) {
    let server = Server::http(format!("0.0.0.0:{}", port)).expect("admin bind failed");
    println!("Admin UI at http://0.0.0.0:{}", port);
    for request in server.incoming_requests() {
        let html = page(&clients, &world, &db);
        let header = Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap();
        let _ = request.respond(Response::from_string(html).with_header(header));
    }
}

fn page(clients: &Clients, world: &World, db: &Arc<Mutex<Connection>>) -> String {
    let online = clients.lock().unwrap();
    let mut names: Vec<String> = online.keys().cloned().collect();
    names.sort();
    let world_blocks = world.lock().unwrap().len();
    let total_players: i64 = db
        .lock()
        .unwrap()
        .prepare("SELECT COUNT(*) FROM players")
        .unwrap()
        .query_row([], |r| r.get(0))
        .unwrap_or(0);

    let mut h = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>MiniRoblox Admin</title>\
         <meta http-equiv=\"refresh\" content=\"3\"></head>\
         <body style=\"font-family:sans-serif;padding:20px\">\
         <h1>MiniRoblox Admin</h1>\
         <p>ออนไลน์ <b>{}</b> คน · โลก <b>{}</b> บล็อก · ผู้เล่นทั้งหมด <b>{}</b> คน</p><ul>",
        names.len(),
        world_blocks,
        total_players
    );
    for n in &names {
        h.push_str(&format!("<li>{}</li>", n));
    }
    h.push_str("</ul></body></html>");
    h
}
