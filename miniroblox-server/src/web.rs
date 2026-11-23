// Admin web UI: Google-login dashboard with role-based access (admin/manager/sales/user).
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

const CLIENT_ID: &str =
    "682624579295-rtgjcfqmkk0deda7ji43r4lcje5jsiqp.apps.googleusercontent.com";
// Redirect สำหรับ admin web — ต้องตรงกับที่ลงทะเบียนใน Google Cloud Console
const WEB_REDIRECT: &str = "https://admin.bcchon.com/oauth2callback";

#[derive(Clone, Copy, PartialEq)]
enum Role {
    Admin,
    Manager,
    Sales,
    User,
}

impl Role {
    fn from_str(s: &str) -> Role {
        match s {
            "admin" => Role::Admin,
            "manager" => Role::Manager,
            "sales" => Role::Sales,
            _ => Role::User,
        }
    }
    fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Manager => "manager",
            Role::Sales => "sales",
            Role::User => "user",
        }
    }
    fn can_coins(&self) -> bool {
        matches!(self, Role::Admin | Role::Manager | Role::Sales)
    }
    fn can_db_ops(&self) -> bool {
        matches!(self, Role::Admin | Role::Manager)
    }
    fn can_restore(&self) -> bool {
        matches!(self, Role::Admin)
    }
    fn can_manage_roles(&self) -> bool {
        matches!(self, Role::Admin)
    }
}

type Sessions = Arc<Mutex<HashMap<String, String>>>; // token -> email

pub fn run(
    port: u16,
    clients: Clients,
    world: World,
    db: Arc<Mutex<Connection>>,
    owner_email: String,
) {
    let sessions: Sessions = Arc::new(Mutex::new(HashMap::new()));
    let server = Server::http(format!("0.0.0.0:{}", port)).expect("admin bind failed");
    println!("Admin UI at http://0.0.0.0:{}", port);
    for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let (path, query) = match url.split_once('?') {
            Some((p, q)) => (p.to_string(), q.to_string()),
            None => (url.clone(), String::new()),
        };

        let session_email = cookie_value(&request, "session")
            .and_then(|tok| sessions.lock().unwrap().get(&tok).cloned());
        let role = session_email
            .as_ref()
            .map(|e| role_for(&db, e, &owner_email));

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
                    Header::from_bytes("Content-Type", "application/javascript; charset=utf-8")
                        .unwrap();
                let _ = request.respond(Response::from_string(APP_JS).with_header(h));
            }
            ("/login", _) => {
                let _ = request.respond(redirect(&google_auth_url()));
            }
            ("/oauth2callback", _) => {
                match query_param(&query, "code").map(|c| percent_decode(&c)) {
                    Some(c) => {
                        eprintln!(
                            "[web] oauth code len={} head={}",
                            c.len(),
                            &c[..c.len().min(14)]
                        );
                        match crate::auth::login_with_code_web(&c, WEB_REDIRECT) {
                            Some(email) => {
                                let tok = new_token();
                                sessions.lock().unwrap().insert(tok.clone(), email);
                                let _ = request.respond(redirect_with_cookie("/", &tok));
                            }
                            None => {
                                let _ = request.respond(html_response("login failed".to_string()));
                            }
                        }
                    }
                    None => {
                        let _ = request.respond(redirect("/login"));
                    }
                }
            }
            ("/logout", _) => {
                if let Some(tok) = cookie_value(&request, "session") {
                    sessions.lock().unwrap().remove(&tok);
                }
                let _ = request.respond(redirect("/"));
            }
            ("/api/me", _) => match (&session_email, role) {
                (Some(e), Some(r)) => {
                    let json = format!("{{\"email\":\"{}\",\"role\":\"{}\"}}", e, r.as_str());
                    let _ = request.respond(json_response(json));
                }
                _ => {
                    let _ = request.respond(unauthorized());
                }
            },
            ("/api/status", _) => {
                if session_email.is_none() {
                    let _ = request.respond(unauthorized());
                } else {
                    let json = status_json(&clients, &world, &db);
                    let _ = request.respond(json_response(json));
                }
            }
            ("/api/coins", &Method::Post) => match role {
                Some(r) if r.can_coins() => {
                    let mut body = String::new();
                    let _ = request.as_reader().read_to_string(&mut body);
                    let result = adjust_coins(&db, &body);
                    let _ = request.respond(text_response(result));
                }
                _ => {
                    let _ = request.respond(forbidden());
                }
            },
            ("/api/roles", &Method::Get) => match role {
                Some(r) if r.can_manage_roles() => {
                    let roles = {
                        let conn = db.lock().unwrap();
                        db::list_roles(&conn)
                    };
                    let json = serde_json::to_string(&roles).unwrap_or_else(|_| "[]".into());
                    let _ = request.respond(json_response(json));
                }
                _ => {
                    let _ = request.respond(forbidden());
                }
            },
            ("/api/roles", &Method::Post) => match role {
                Some(r) if r.can_manage_roles() => {
                    let mut body = String::new();
                    let _ = request.as_reader().read_to_string(&mut body);
                    let result = set_role_api(&db, &body);
                    let _ = request.respond(text_response(result));
                }
                _ => {
                    let _ = request.respond(forbidden());
                }
            },
            ("/backup", _) => match role {
                Some(r) if r.can_db_ops() => match backup_data(&db) {
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
                _ => {
                    let _ = request.respond(forbidden());
                }
            },
            ("/repair", &Method::Post) => match role {
                Some(r) if r.can_db_ops() => {
                    let result = do_repair(&db);
                    let _ = request.respond(text_response(result));
                }
                _ => {
                    let _ = request.respond(forbidden());
                }
            },
            ("/restore", &Method::Post) => match role {
                Some(r) if r.can_restore() => {
                    let mut body = Vec::new();
                    let _ = request.as_reader().read_to_end(&mut body);
                    let result = do_restore(&db, &world, &body);
                    let _ = request.respond(text_response(result));
                }
                _ => {
                    let _ = request.respond(forbidden());
                }
            },
            _ => {
                let _ = request.respond(text_response("not found".to_string()));
            }
        }
    }
}

fn role_for(db: &Arc<Mutex<Connection>>, email: &str, owner_email: &str) -> Role {
    if email == owner_email {
        return Role::Admin;
    }
    let conn = db.lock().unwrap();
    Role::from_str(&db::get_role(&conn, email))
}

fn new_token() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", now)
}

fn cookie_value(request: &tiny_http::Request, name: &str) -> Option<String> {
    for h in request.headers() {
        if h.field.equiv("Cookie") {
            for part in h.value.as_str().split(';') {
                let mut kv = part.trim().splitn(2, '=');
                if kv.next() == Some(name) {
                    return kv.next().map(|s| s.to_string());
                }
            }
        }
    }
    None
}

fn query_param(query: &str, name: &str) -> Option<String> {
    for kv in query.split('&') {
        let mut it = kv.splitn(2, '=');
        if it.next() == Some(name) {
            return it.next().map(|s| s.to_string());
        }
    }
    None
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn google_auth_url() -> String {
    format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&access_type=offline",
        CLIENT_ID, WEB_REDIRECT
    )
}

fn redirect(location: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let h = Header::from_bytes("Location", location).unwrap();
    Response::from_string(String::new())
        .with_status_code(302)
        .with_header(h)
}

fn redirect_with_cookie(location: &str, token: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let h1 = Header::from_bytes("Location", location).unwrap();
    let h2 = Header::from_bytes(
        "Set-Cookie",
        format!("session={}; Path=/; HttpOnly; SameSite=Lax", token),
    )
    .unwrap();
    Response::from_string(String::new())
        .with_status_code(302)
        .with_header(h1)
        .with_header(h2)
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

fn unauthorized() -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string("unauthorized".to_string()).with_status_code(401)
}

fn forbidden() -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string("forbidden".to_string()).with_status_code(403)
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

fn set_role_api(db: &Arc<Mutex<Connection>>, body: &str) -> String {
    #[derive(serde::Deserialize)]
    struct Req {
        email: String,
        role: String,
    }
    let req: Req = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => return format!("bad request: {}", e),
    };
    let valid = ["admin", "manager", "sales", "user"];
    if !valid.contains(&req.role.as_str()) {
        return "invalid role".to_string();
    }
    let conn = db.lock().unwrap();
    db::set_role(&conn, &req.email, &req.role);
    format!("{} -> {}", req.email, req.role)
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
