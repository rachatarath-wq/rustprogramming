// เน็ตเวิร์ก: โปรโตคอลเดียวกับเซิร์ฟเวอร์ + Google OAuth login
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub const CLIENT_ID: &str =
    "682624579295-hc9fqes8btvs795f2udl0mivhelc349i.apps.googleusercontent.com";
const REDIRECT_URI: &str = "http://127.0.0.1:8080";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Login { code: String },
    Move { name: String, x: f32, y: f32, z: f32 },
    Chat { name: String, text: String },
    World { blocks: Vec<Block> },
}

pub fn send_msg(stream: &mut TcpStream, msg: &Message) -> std::io::Result<()> {
    let bytes = bincode::serialize(msg).unwrap();
    let len = bytes.len() as u32;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&bytes)?;
    Ok(())
}

pub fn recv_msg(stream: &mut TcpStream) -> std::io::Result<Message> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    Ok(bincode::deserialize(&buf).unwrap())
}

// ตรวจก่อนว่ามีเซิร์ฟเวอร์ไหม (กันเด้งเบราว์เซอร์ตอนเล่นเดี่ยว)
// แล้วทำ Google OAuth login และรับโลกกลับมา
pub fn connect_with_google_login(addr: &str) -> Option<(TcpStream, Vec<Block>)> {
    let _probe =
        TcpStream::connect_timeout(&addr.parse().ok()?, Duration::from_millis(800)).ok()?;
    // DEV=1: ข้าม Google (ใช้ "dev:player") สำหรับทดสอบเร็ว ๆ
    let code = if std::env::var("DEV").map(|v| v == "1").unwrap_or(false) {
        "dev:player".to_string()
    } else {
        oauth_login()?
    };
    connect_and_login(addr, &code)
}

fn connect_and_login(addr: &str, code: &str) -> Option<(TcpStream, Vec<Block>)> {
    let mut stream =
        TcpStream::connect_timeout(&addr.parse().ok()?, Duration::from_secs(2)).ok()?;
    // เซิร์ฟเวอร์ต้องแลก code เป็น token (เรียก Google 2 รอบ) — ให้เวลามากหน่อย
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok()?;
    send_msg(&mut stream, &Message::Login { code: code.to_string() }).ok()?;
    let blocks = match recv_msg(&mut stream).ok()? {
        Message::World { blocks } => blocks,
        _ => return None,
    };
    stream.set_read_timeout(None).ok();
    Some((stream, blocks))
}

// เปิดเบราว์เซอร์ให้ผู้ใช้ล็อกอิน Google แล้วรอ redirect กลับมา -> ได้ code
pub fn oauth_login() -> Option<String> {
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&access_type=offline",
        CLIENT_ID, REDIRECT_URI
    );
    let _ = open::that(&auth_url);
    let listener = TcpListener::bind("127.0.0.1:8080").ok()?;
    let (mut sock, _) = listener.accept().ok()?;
    let mut buf = [0u8; 4096];
    let n = sock.read(&mut buf).ok()?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let code = extract_code(&req)?;
    let body = "<html><body><h2>Login OK — close this window.</h2></body></html>";
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = sock.write_all(resp.as_bytes());
    Some(code)
}

// ดึงค่า code จาก query string ในคำขอ GET
fn extract_code(req: &str) -> Option<String> {
    let path = req.lines().next()?.split_whitespace().nth(1)?;
    let query = path.split('?').nth(1)?;
    for kv in query.split('&') {
        let mut it = kv.splitn(2, '=');
        if it.next() == Some("code") {
            return it.next().map(|s| s.to_string());
        }
    }
    None
}
