// SQLite persistence: players + world blocks survive a server restart.
use rusqlite::{params, Connection};

// Open (or create) world.db and ensure tables exist.
pub fn init() -> Connection {
    let conn = Connection::open("world.db").expect("open db failed");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS players (
            name TEXT PRIMARY KEY,
            x REAL, y REAL, z REAL
        )",
        [],
    )
    .expect("create players table failed");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS blocks (
            x INTEGER, y INTEGER, z INTEGER,
            block INTEGER,
            PRIMARY KEY (x, y, z)
        )",
        [],
    )
    .expect("create blocks table failed");
    conn
}

// Save (or update) one player's last position.
pub fn save_player(conn: &Connection, name: &str, x: f32, y: f32, z: f32) {
    conn.execute(
        "INSERT INTO players (name, x, y, z) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(name) DO UPDATE SET x=?2, y=?3, z=?4",
        params![name, x, y, z],
    )
    .expect("save player failed");
}

// Read back all players as (name, x, y, z).
pub fn list_players(conn: &Connection) -> Vec<(String, f32, f32, f32)> {
    let mut stmt = conn
        .prepare("SELECT name, x, y, z FROM players")
        .unwrap();
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap();
    rows.filter_map(|r| r.ok()).collect()
}
