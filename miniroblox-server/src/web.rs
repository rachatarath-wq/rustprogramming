// Admin web UI: professional dashboard (TypeScript frontend) + JSON API + DB ops.
use crate::db;
use crate::Block;
use rusqlite::Connection;
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Method, Response, Server};

pub type Clients = Arc<Mutex<HashMap<String, TcpStream>>>;
pub type World = Arc<Mutex<Vec<Block>>>;

// Static frontend (source in web/*.ts, bundled with esbuild).
const INDEX_HTML: &str = include_str!("../web/index.html");
const STYLE_CSS: &str = include_str!("../web/style.css");
const APP_JS: &str = include_str!("../web/app.js");

pub fn run(port: u16, clients: Clients, world: World, db: Arc<Mutex<Connection>>) {
    let server = Server::http(format!("0.0.0.0:{}", port)).expect("admin bind failed");
    println!("Admin UI at http://0.0.0.0:{}", port);
    for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let path = url.split('?').next().unwrap_or("/").to_string();

        match (path.as_str(), request.method()) {
            ("/", _) => {
                let _ = request.respond(html_response(INDEX_HTML.to_string()));
            }
            ("/style.css", _) => {
                let h = Header::from_bytes("Content-Type", "text/css; charset=utf-8").unwrap();
                let _ = request.respond(Response::from_string(STYLE_CSS).with_header(h));
            }
            ("/app.js", _) => {
                let h =
                    Header::from_bytes("Content-Type", "application/javascript; charset=utf-8").unwrap();
                let _ = request.respond(Response::from_string(APP_JS).with_header(h));
            }
            ("/api/status", _) => {
                let json = status_json(&clients, &world, &db);
                let _ = request.respond(json_response(json));
            }
            ("/api/coins", &Method::Post) => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let result = adjust_coins(&db, &body);
                let _ = request.respond(text_response(result));
            }
            ("/backup", _) => match backup_data(&db) {
                Ok(data) => {
                    let h1 =
                        Header::from_bytes("Content-Type", "application/octet-stream").unwrap();
                    let h2 = Header::from_bytes(
                        "Content-Disposition",
                        "attachment; filename=\"world_backup.db\"",
                    )
                    .unwrap();
                    let _ = request
                        .respond(Response::from_data(data).with_header(h1).with_header(h2));
                }
                Err(e) => {
                    let _ = request.respond(text_response(e));
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
                let _ = request.respond(text_response("not found".to_string()));
            }
        }
    }
}

fn html_response(body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap();
    Response::from_string(body).with_header(header)
}

fn json_response(body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes("Content-Type", "application/json; charset=utf-8").unwrap();
    Response::from_string(body).with_header(header)
}

fn text_response(body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes("Content-Type", "text/plain; charset=utf-8").unwrap();
    Response::from_string(body).with_header(header)
}

// --- JSON API ---

#[derive(serde::Serialize)]
struct PlayerInfo {
    name: String,
    coins: i64,
    online: bool,
    x: f32,
    y: f32,
    z: f32,
}

#[derive(serde::Serialize)]
struct Status {
    online_count: usize,
    world_blocks: usize,
    total_players: usize,
    total_coins: i64,
    players: Vec<PlayerInfo>,
}

fn status_json(clients: &Clients, world: &World, db: &Arc<Mutex<Connection>>) -> String {
    let online = clients.lock().unwrap();
    let world_blocks = world.lock().unwrap().len();
    let conn = db.lock().unwrap();
    let all = db::list_players(&conn);
    let players: Vec<PlayerInfo> = all
        .into_iter()
        .map(|(name, x, y, z, coins)| PlayerInfo {
            online: online.contains_key(&name),
            name,
            coins,
            x,
            y,
            z,
        })
        .collect();
    let total_coins: i64 = players.iter().map(|p| p.coins).sum();
    let status = Status {
        online_count: online.len(),
        world_blocks,
        total_players: players.len(),
        total_coins,
        players,
    };
    serde_json::to_string(&status).unwrap_or_else(|_| "{}".into())
}

fn adjust_coins(db: &Arc<Mutex<Connection>>, body: &str) -> String {
    #[derive(serde::Deserialize)]
    struct Req {
        name: String,
        delta: i64,
    }
    let req: Req = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => return format!("bad request: {}", e),
    };
    let conn = db.lock().unwrap();
    let coins = db::add_coins(&conn, &req.name, req.delta);
    format!("{}: {} coins", req.name, coins)
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
