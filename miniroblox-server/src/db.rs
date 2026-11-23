// SQLite persistence: players + world blocks survive a server restart.
use crate::Block;
use rusqlite::{params, Connection};

// Open (or create) world.db and ensure tables exist.
pub fn init() -> Connection {
    let conn = Connection::open("world.db").expect("open db failed");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS players (
            name TEXT PRIMARY KEY,
            x REAL, y REAL, z REAL,
            coins INTEGER DEFAULT 0
        )",
        [],
    )
    .expect("create players table failed");
    // Migration: older databases may not have the coins column yet.
    let has_coins = conn
        .prepare("SELECT 1 FROM pragma_table_info('players') WHERE name = 'coins'")
        .and_then(|mut s| s.exists([]))
        .unwrap_or(false);
    if !has_coins {
        conn.execute("ALTER TABLE players ADD COLUMN coins INTEGER DEFAULT 0", [])
            .expect("add coins column failed");
    }
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

// Save (or update) one player's last position (and ensure they have a coin row).
pub fn save_player(conn: &Connection, name: &str, x: f32, y: f32, z: f32) {
    conn.execute(
        "INSERT INTO players (name, x, y, z) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(name) DO UPDATE SET x=?2, y=?3, z=?4",
        params![name, x, y, z],
    )
    .expect("save player failed");
}

// Add coins to a player's balance and return the new balance.
pub fn add_coins(conn: &Connection, name: &str, delta: i64) -> i64 {
    let affected = conn
        .execute(
            "UPDATE players SET coins = coins + ?2 WHERE name = ?1",
            params![name, delta],
        )
        .expect("add coins failed");
    if affected == 0 {
        // Not in the table yet (e.g. admin granting coins before first login).
        conn.execute(
            "INSERT INTO players (name, x, y, z, coins) VALUES (?1, 0, 0, 0, ?2)",
            params![name, delta],
        )
        .expect("insert coins failed");
    }
    get_coins(conn, name)
}

// Read one player's current coin balance (0 if unknown).
pub fn get_coins(conn: &Connection, name: &str) -> i64 {
    conn.query_row(
        "SELECT coins FROM players WHERE name = ?1",
        params![name],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

// Read back all players as (name, x, y, z, coins).
pub fn list_players(conn: &Connection) -> Vec<(String, f32, f32, f32, i64)> {
    let mut stmt = conn
        .prepare("SELECT name, x, y, z, coins FROM players")
        .unwrap();
    let rows = stmt
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap();
    rows.filter_map(|r| r.ok()).collect()
}

// Load the whole world (all blocks) into memory.
pub fn load_blocks(conn: &Connection) -> Vec<Block> {
    let mut stmt = conn.prepare("SELECT x, y, z, block FROM blocks").unwrap();
    let rows = stmt
        .query_map([], |r| {
            Ok(Block {
                x: r.get(0)?,
                y: r.get(1)?,
                z: r.get(2)?,
                block: r.get(3)?,
            })
        })
        .unwrap();
    rows.filter_map(|r| r.ok()).collect()
}

// Save one block (upsert on x,y,z).
pub fn save_block(conn: &Connection, b: &Block) {
    conn.execute(
        "INSERT INTO blocks (x, y, z, block) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(x, y, z) DO UPDATE SET block=?4",
        params![b.x, b.y, b.z, b.block],
    )
    .expect("save block failed");
}

// Remove one block (used when a player breaks it).
pub fn delete_block(conn: &Connection, x: i32, y: i32, z: i32) {
    conn.execute(
        "DELETE FROM blocks WHERE x = ?1 AND y = ?2 AND z = ?3",
        params![x, y, z],
    )
    .expect("delete block failed");
}
