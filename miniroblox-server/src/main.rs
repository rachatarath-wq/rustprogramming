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
    // การเทรดที่รอตอบรับ: ชื่อผู้รับ -> (ชื่อผู้ส่ง, จำนวนเหรียญ)
    let pending_trades: Arc<Mutex<HashMap<String, (String, i64)>>> =
        Arc::new(Mutex::new(HashMap::new()));
    // อีเมลเจ้าของเซิร์ฟเวอร์ — ได้มงกุฎทอง + ตัวใหญ่กว่า (ตั้งค่า OWNER_EMAIL เพื่อเปลี่ยน)
    let owner_email: String =
        std::env::var("OWNER_EMAIL").unwrap_or_else(|_| "rachata.rath@gmail.com".to_string());

    // Admin web UI on a separate port (read-only status page).
    let admin_port: u16 = std::env::var("ADMIN_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);
    {
        let c = Arc::clone(&clients);
        let w = Arc::clone(&world);
        let d = Arc::clone(&db);
        thread::spawn(move || miniroblox_server::web::run(admin_port, c, w, d));
    }

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
        let owner_email = owner_email.clone();
        let pending_trades = Arc::clone(&pending_trades);
        thread::spawn(move || handle(stream, clients, db, world, owner_email, pending_trades));
    }
}

fn handle(
    mut stream: TcpStream,
    clients: Clients,
    db: Arc<Mutex<rusqlite::Connection>>,
    world: World,
    owner_email: String,
    pending_trades: Arc<Mutex<HashMap<String, (String, i64)>>>,
) {
    let mut my_name: Option<String> = None;
    let mut my_owner = false;
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
                let coins = {
                    let conn = db.lock().unwrap();
                    db::save_player(&conn, &email, 0.0, 0.0, 0.0);
                    db::get_coins(&conn, &email)
                };
                // กระจายจุดเกิด — ออฟเซตจาก hash ของอีเมล ให้แต่ละคนเกิดคนละจุด
                let mut h: u32 = 2166136261;
                for b in email.bytes() {
                    h = (h ^ b as u32).wrapping_mul(16777619);
                }
                let sx = (h % 9) as i32 - 4;
                let sz = ((h >> 8) % 9) as i32 - 4;
                let blocks = world.lock().unwrap().clone();
                let n = blocks.len();
                let owner = email == owner_email;
                let _ = send_msg(&mut stream, &Message::World { blocks });
                let _ = send_msg(&mut stream, &Message::Spawn { x: sx as f32, z: sz as f32, owner });
                let _ = send_msg(&mut stream, &Message::Coins { coins });
                clients.lock().unwrap().insert(email.clone(), stream.try_clone().unwrap());
                my_name = Some(email.clone());
                my_owner = owner;
                println!("{} logged in, sent {} blocks ({} coins, spawn {},{})", email, n, coins, sx, sz);
            }
            // Movement: overwrite the name with the verified identity (anti-spoofing)
            // then broadcast to everyone EXCEPT the sender.
            Message::Move { name: _, x, y, z, color, owner: _ } => {
                let out = Message::Move {
                    name: my_name.clone().unwrap_or_default(),
                    x: *x,
                    y: *y,
                    z: *z,
                    color: *color,
                    owner: my_owner,
                };
                let mut guard = clients.lock().unwrap();
                for (name, c) in guard.iter_mut() {
                    if Some(name) != my_name.as_ref() {
                        let _ = send_msg(c, &out);
                    }
                }
            }
            // Chat: overwrite name (anti-spoofing), broadcast to EVERYONE (incl. sender).
            Message::Chat { name: _, text } => {
                let out = Message::Chat {
                    name: my_name.clone().unwrap_or_default(),
                    text: text.clone(),
                };
                let mut guard = clients.lock().unwrap();
                for c in guard.values_mut() {
                    let _ = send_msg(c, &out);
                }
            }
            // Break a block: authoritative removal + reward 1 coin, then tell everyone.
            Message::BlockBreak { name: _, x, y, z } => {
                let name = my_name.clone().unwrap_or_default();
                world
                    .lock()
                    .unwrap()
                    .retain(|b| !(b.x == *x && b.y == *y && b.z == *z));
                let coins = {
                    let conn = db.lock().unwrap();
                    db::delete_block(&conn, *x, *y, *z);
                    db::add_coins(&conn, &name, 1)
                };
                let _ = send_msg(&mut stream, &Message::Coins { coins });
                let out = Message::BlockBreak { name: name.clone(), x: *x, y: *y, z: *z };
                let mut guard = clients.lock().unwrap();
                for (n, c) in guard.iter_mut() {
                    if Some(n) != Some(&name) {
                        let _ = send_msg(c, &out);
                    }
                }
            }
            // Buy an item from the shop: validate price + balance, then deduct.
            Message::Buy { name: _, item } => {
                let name = my_name.clone().unwrap_or_default();
                let price = match item.as_str() {
                    "apple" => 5,
                    "heal" => 10,
                    _ => 0,
                };
                if price > 0 {
                    let conn = db.lock().unwrap();
                    let coins = db::get_coins(&conn, &name);
                    if coins >= price {
                        let new_coins = db::add_coins(&conn, &name, -price);
                        let _ = send_msg(
                            &mut stream,
                            &Message::Bought { item: item.clone(), coins: new_coins },
                        );
                    } else {
                        // เหรียญไม่พอ — ส่งยอดปัจจุบันกลับไป
                        let _ = send_msg(&mut stream, &Message::Coins { coins });
                    }
                }
            }
            // Trade: เสนอให้เหรียญผู้เล่นอื่น -> ส่งคำขอไปให้เป้าหมาย
            Message::TradeOffer { to, amount } => {
                let name = my_name.clone().unwrap_or_default();
                let my_coins = {
                    let conn = db.lock().unwrap();
                    db::get_coins(&conn, &name)
                };
                let target_stream = clients
                    .lock()
                    .unwrap()
                    .get(to.as_str())
                    .and_then(|s| s.try_clone().ok());
                if *amount > 0 && my_coins >= *amount && target_stream.is_some() {
                    pending_trades
                        .lock()
                        .unwrap()
                        .insert(to.clone(), (name.clone(), *amount));
                    let ask = Message::TradeAsk { from: name, amount: *amount };
                    let _ = send_msg(&mut target_stream.unwrap(), &ask);
                } else {
                    let done = Message::TradeDone {
                        ok: false,
                        coins: my_coins,
                        msg: "trade failed: not enough coins or player not found".to_string(),
                    };
                    let _ = send_msg(&mut stream, &done);
                }
            }
            // Trade reply: เป้าหมายตอบรับหรือปฏิเสธ
            Message::TradeReply { accept } => {
                let name = my_name.clone().unwrap_or_default();
                let pending = pending_trades.lock().unwrap().remove(&name);
                if let Some((from, amount)) = pending {
                    if *accept {
                        let (sender_coins, my_coins) = {
                            let conn = db.lock().unwrap();
                            let a = db::add_coins(&conn, &from, -amount);
                            let b = db::add_coins(&conn, &name, amount);
                            (a, b)
                        };
                        if let Some(s) = clients
                            .lock()
                            .unwrap()
                            .get(from.as_str())
                            .and_then(|s| s.try_clone().ok())
                        {
                            let mut ss = s;
                            let _ = send_msg(&mut ss, &Message::Coins { coins: sender_coins });
                        }
                        let done = Message::TradeDone {
                            ok: true,
                            coins: my_coins,
                            msg: format!("received {} coins from {}", amount, from),
                        };
                        let _ = send_msg(&mut stream, &done);
                    } else {
                        let coins = {
                            let conn = db.lock().unwrap();
                            db::get_coins(&conn, &name)
                        };
                        let done = Message::TradeDone {
                            ok: false,
                            coins,
                            msg: "trade rejected".to_string(),
                        };
                        let _ = send_msg(&mut stream, &done);
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
