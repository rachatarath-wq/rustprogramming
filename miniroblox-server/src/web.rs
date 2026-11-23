// Admin web UI: live status + database backup / restore / repair.
use crate::db;
use crate::Block;
use rusqlite::Connection;
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Method, Response, Server};

pub type Clients = Arc<Mutex<HashMap<String, TcpStream>>>;
pub type World = Arc<Mutex<Vec<Block>>>;

pub fn run(port: u16, clients: Clients, world: World, db: Arc<Mutex<Connection>>) {
    let server = Server::http(format!("0.0.0.0:{}", port)).expect("admin bind failed");
    println!("Admin UI at http://0.0.0.0:{}", port);
    for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let path = url.split('?').next().unwrap_or("/").to_string();

        match (path.as_str(), request.method()) {
            ("/backup", _) => match backup_data(&db) {
                Ok(data) => {
                    let h1 = Header::from_bytes("Content-Type", "application/octet-stream").unwrap();
                    let h2 = Header::from_bytes(
                        "Content-Disposition",
                        "attachment; filename=\"world_backup.db\"",
                    )
                    .unwrap();
                    let _ = request
                        .respond(Response::from_data(data).with_header(h1).with_header(h2));
                }
                Err(e) => {
                    let _ = request.respond(Response::from_string(e));
                }
            },
            ("/repair", &Method::Post) => {
                let result = do_repair(&db);
                let _ = request.respond(text_response(result));
            }
            ("/restore", &Method::Post) => {
                let mut body = Vec::new();
                let _ = request.as_reader().read_to_end(&mut body);
                let result = do_restore(&db, &world, &body);
                let _ = request.respond(text_response(result));
            }
            _ => {
                let _ = request.respond(html_response(page(&clients, &world, &db)));
            }
        }
    }
}

fn html_response(body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap();
    Response::from_string(body).with_header(header)
}

fn text_response(body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes("Content-Type", "text/plain; charset=utf-8").unwrap();
    Response::from_string(body).with_header(header)
}

// --- database operations ---

fn backup_data(db: &Arc<Mutex<Connection>>) -> Result<Vec<u8>, String> {
    let conn = db.lock().unwrap();
    let fname = "world_backup.db";
    conn.execute_batch(&format!("VACUUM INTO '{}'", fname))
        .map_err(|e| format!("backup failed: {}", e))?;
    let data = std::fs::read(fname).map_err(|e| format!("read backup: {}", e))?;
    let _ = std::fs::remove_file(fname);
    Ok(data)
}

fn do_repair(db: &Arc<Mutex<Connection>>) -> String {
    let conn = db.lock().unwrap();
    let check: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap_or_else(|_| "?".into());
    let vacuum = conn
        .execute_batch("VACUUM")
        .map(|_| "VACUUM: ok".to_string())
        .unwrap_or_else(|e| format!("VACUUM failed: {}", e));
    format!("integrity_check: {}\n{}", check, vacuum)
}

fn do_restore(db: &Arc<Mutex<Connection>>, world: &World, data: &[u8]) -> String {
    // validate SQLite header
    if data.len() < 16 || &data[0..16] != b"SQLite format 3\0" {
        return "restore failed: not a valid SQLite file".to_string();
    }
    // safety backup of current DB, then close it
    {
        let mut guard = db.lock().unwrap();
        let _ = guard.execute_batch("VACUUM INTO 'world_pre_restore.db'");
        *guard = Connection::open(":memory:").expect("open memory");
    }
    if let Err(e) = std::fs::write("world.db", data) {
        return format!("restore failed: {}", e);
    }
    // reopen + reload the in-memory world
    {
        let mut guard = db.lock().unwrap();
        *guard = Connection::open("world.db").expect("reopen world.db");
        let blocks = db::load_blocks(&guard);
        *world.lock().unwrap() = blocks;
    }
    "restore OK: world reloaded".to_string()
}

// --- status page ---

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
         <meta http-equiv=\"refresh\" content=\"5\"></head>\
         <body style=\"font-family:sans-serif;padding:20px\">\
         <h1>MiniRoblox Admin</h1>\
         <p>ออนไลน์ <b>{}</b> คน · โลก <b>{}</b> บล็อก · ผู้เล่นทั้งหมด <b>{}</b> คน</p>\
         <h2>ผู้เล่นออนไลน์</h2><ul>",
        names.len(),
        world_blocks,
        total_players
    );
    for n in &names {
        h.push_str(&format!("<li>{}</li>", n));
    }
    h.push_str("</ul>");
    h.push_str(
        "<h2>ฐานข้อมูล</h2>\
         <p><a href=\"/backup\">สำรองข้อมูล (ดาวน์โหลด .db)</a></p>\
         <p><button onclick=\"repair()\">ซ่อมฐานข้อมูล (VACUUM + ตรวจ)</button></p>\
         <p><input type=\"file\" id=\"restoreFile\"> \
         <button onclick=\"restore()\">กู้คืนจากไฟล์</button></p>\
         <p id=\"msg\"></p>\
         <script>\
         function repair(){{fetch('/repair',{{method:'POST'}}).then(r=>r.text()).then(t=>document.getElementById('msg').textContent=t)}}\
         function restore(){{const f=document.getElementById('restoreFile').files[0];\
         if(!f){{alert('เลือกไฟล์ก่อน');return}}\
         fetch('/restore',{{method:'POST',body:f}}).then(r=>r.text()).then(t=>document.getElementById('msg').textContent=t)}}\
         </script>",
    );
    h.push_str("</body></html>");
    h
}
