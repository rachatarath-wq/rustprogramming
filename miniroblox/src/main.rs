use macroquad::prelude::*;
use macroquad::audio::*;
use gilrs::{Axis, Button, Gilrs};
use std::collections::HashMap;
use std::fs;

// ตำแหน่งของบล็อกในโลก = พิกัดจำนวนเต็ม (x, y, z)
type BlockPos = (i32, i32, i32);

// ชนิดของบล็อก
#[derive(Clone, Copy, PartialEq)]
enum Block {
    Grass,
    Dirt,
    Stone,
    Wood,
    Sand,
    Glass,
    Brick,
    Snow,
}

// ลำดับบล็อกสำหรับเลื่อนเลือกด้วยจอย (DPad ซ้าย/ขวา)
fn block_next(b: Block) -> Block {
    match b {
        Block::Grass => Block::Dirt, Block::Dirt => Block::Stone, Block::Stone => Block::Wood,
        Block::Wood => Block::Sand, Block::Sand => Block::Glass, Block::Glass => Block::Brick,
        Block::Brick => Block::Snow, Block::Snow => Block::Grass,
    }
}
fn block_prev(b: Block) -> Block {
    match b {
        Block::Grass => Block::Snow, Block::Snow => Block::Brick, Block::Brick => Block::Glass,
        Block::Glass => Block::Sand, Block::Sand => Block::Wood, Block::Wood => Block::Stone,
        Block::Stone => Block::Dirt, Block::Dirt => Block::Grass,
    }
}

impl Block {
    fn color(&self) -> Color {
        match self {
            Block::Grass => Color::from_rgba(96, 190, 84, 255),
            Block::Dirt => Color::from_rgba(143, 104, 68, 255),
            Block::Stone => Color::from_rgba(130, 130, 134, 255),
            Block::Wood => Color::from_rgba(145, 92, 42, 255),
            Block::Sand => Color::from_rgba(220, 210, 140, 255),
            Block::Glass => Color::from_rgba(160, 210, 230, 150),
            Block::Brick => Color::from_rgba(160, 60, 50, 255),
            Block::Snow => Color::from_rgba(235, 240, 245, 255),
        }
    }
    fn name(&self) -> &str {
        match self {
            Block::Grass => "Grass",
            Block::Dirt => "Dirt",
            Block::Stone => "Stone",
            Block::Wood => "Wood",
            Block::Sand => "Sand",
            Block::Glass => "Glass",
            Block::Brick => "Brick",
            Block::Snow => "Snow",
        }
    }
    fn from_name(name: &str) -> Option<Block> {
        match name {
            "Grass" => Some(Block::Grass),
            "Dirt" => Some(Block::Dirt),
            "Stone" => Some(Block::Stone),
            "Wood" => Some(Block::Wood),
            "Sand" => Some(Block::Sand),
            "Glass" => Some(Block::Glass),
            "Brick" => Some(Block::Brick),
            "Snow" => Some(Block::Snow),
            _ => None,
        }
    }
}

// ม็อบหมาป่า
struct Wolf {
    pos: Vec3,
    vel: Vec3,
    hp: i32,
    grounded: bool,
    boss: bool,   // true = บอสใหญ่
}

// อนุภาค (ทุบบล็อก)
struct Particle {
    pos: Vec3,
    vel: Vec3,
    life: f32,
    color: Color,
}

// แอปเปิ้ลบนพื้น
struct Apple {
    pos: Vec3,
    taken: bool,
}

const REACH: f32 = 8.0;
const GRAVITY: f32 = 22.0;
const GAMEPAD_LOOK_SPEED: f32 = 3.0; // ความเร็วหมุนมองด้วยจอยขวา (rad/s)
const SAVE_FILE: &str = "world_v4.txt";
const SETTINGS_FILE: &str = "settings.txt";
const WORLD_SIZE: i32 = 32;

// เมล็ดภูมิประเทศ (เปลี่ยน seed -> เปลี่ยนรูปแผนที่)
static TERRAIN_SEED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

// เครื่องสุ่มตัวเลข xorshift64 (ไม่พึ่ง crate ภายนอก)
struct Rng {
    s: u64,
}
impl Rng {
    fn new(seed: u64) -> Self {
        Rng { s: seed.max(1) }
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.s;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.s = x;
        x
    }
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next_u64() % (hi - lo + 1) as u64) as i32
    }
}

// ความสูงภูมิประเทศ ณ (x, z) — คลื่นซ้อนกัน + เฟสจาก seed
fn terrain_height(x: i32, z: i32) -> i32 {
    let s = (TERRAIN_SEED.load(std::sync::atomic::Ordering::Relaxed) % 10000) as f32 * 0.001;
    let xf = x as f32 + s * 7.0;
    let zf = z as f32 + s * 5.0;
    let h = 8.0
        + 3.0 * (xf * 0.30).sin() * (zf * 0.26).cos()
        + 2.0 * ((xf + 1.7) * 0.13).sin() * ((zf + 0.6) * 0.19).cos()
        + 1.5 * (xf * 0.07).sin() * (zf * 0.09).cos();
    h.round() as i32
}

// ความสูงของที่ราบสูงจุดเกิด = จุดสูงสุดของภูมิประเทศรอบกลาง (13x13)
// ใช้ร่วมกันระหว่าง build_world และ main เพื่อให้จุดเกิดตรงกับลานจริง
fn plaza_height() -> i32 {
    let cx = WORLD_SIZE / 2;
    let mut h = terrain_height(cx, cx);
    for x in cx - 6..=cx + 6 {
        for z in cx - 6..=cx + 6 {
            h = h.max(terrain_height(x, z));
        }
    }
    h
}

// ความสูงผิวดินจริง ณ (x,z) — บนที่ราบสูงจุดเกิดจะสูงกว่า terrain_height
fn surface_height(x: i32, z: i32) -> i32 {
    let cx = WORLD_SIZE / 2;
    if (x - cx).abs() <= 4 && (z - cx).abs() <= 4 {
        plaza_height()
    } else {
        terrain_height(x, z)
    }
}

// สร้างต้นไม้ (ลำต้น + ใบ)
fn build_tree(w: &mut HashMap<BlockPos, Block>, x: i32, y: i32, z: i32) {
    for ty in y..y + 4 {
        w.insert((x, ty, z), Block::Wood);
    }
    let top = y + 4;
    for dx in -1..=1 {
        for dz in -1..=1 {
            w.insert((x + dx, top, z + dz), Block::Grass);
        }
    }
}

// สร้างบ้าน (พื้น + ผนัง + ช่องประตู + หลังคา)
fn build_house(w: &mut HashMap<BlockPos, Block>, x0: i32, base_y: i32, z0: i32, wall: Block, roof: Block) {
    let x1 = x0 + 4;
    let z1 = z0 + 4;
    for x in x0..=x1 {
        for z in z0..=z1 {
            w.insert((x, base_y, z), Block::Stone);   // พื้นบ้าน
            for y in 1..=2 {
                if x == x0 || x == x1 || z == z0 || z == z1 {
                    // เว้นช่องประตู (ผนังหน้า ตรงกลาง)
                    if z == z0 && x == x0 + 2 && y == 1 {
                        continue;
                    }
                    w.insert((x, base_y + y, z), wall);
                }
            }
        }
    }
    for x in (x0 - 1)..=(x1 + 1) {
        for z in (z0 - 1)..=(z1 + 1) {
            w.insert((x, base_y + 3, z), roof);   // หลังคา
        }
    }
}

fn build_world(seed: u64) -> HashMap<BlockPos, Block> {
    let mut w = HashMap::new();
    let mut rng = Rng::new(seed);
    let cx = WORLD_SIZE / 2;

    // 1. ภูมิประเทศ (เขา/หุบ) 32x32
    for x in 0..WORLD_SIZE {
        for z in 0..WORLD_SIZE {
            let h = terrain_height(x, z);
            for y in 0..=h {
                let block = if y == h {
                    Block::Grass
                } else if y >= h - 3 {
                    Block::Dirt
                } else {
                    Block::Stone
                };
                w.insert((x, y, z), block);
            }
        }
    }

    // 2. ที่ราบสูงกลางแผนที่ 9x9 (จุดเกิด) — ยกเป็นจุดสูงสุดของบริเวณ
    //    เพื่อให้จุดเกิดอยู่บนที่สูงเสมอ เดินลงออกได้ทุกทิศ ไม่ติดหลุม
    let plaza_h = plaza_height();
    for x in cx - 4..=cx + 4 {
        for z in cx - 4..=cx + 4 {
            // ถมหินเต็มตั้งแต่ผิวดินเดิมขึ้นไปถึงยอดที่ราบ (ไม่ให้มีช่องว่างใต้ลาน)
            let th = terrain_height(x, z);
            for y in th..=plaza_h {
                w.insert((x, y, z), Block::Stone);
            }
        }
    }

    // 3. ต้นไม้สุ่ม 8-12 ต้น (หลีกเลี่ยงกลาง)
    let trees = rng.range(8, 12);
    for _ in 0..trees {
        let x = rng.range(3, WORLD_SIZE - 4);
        let z = rng.range(3, WORLD_SIZE - 4);
        if (x - cx).abs() < 5 && (z - cx).abs() < 5 {
            continue;
        }
        let h = terrain_height(x, z);
        build_tree(&mut w, x, h + 1, z);
    }

    // 4. บ้านสุ่ม 3 หลัง (ปรับพื้นให้เรียบก่อน)
    for _ in 0..3 {
        let x = rng.range(3, WORLD_SIZE - 8);
        let z = rng.range(3, WORLD_SIZE - 8);
        if (x - cx).abs() < 7 && (z - cx).abs() < 7 {
            continue;
        }
        let base = terrain_height(x, z);
        for hx in x - 1..=x + 5 {
            for hz in z - 1..=z + 5 {
                for y in 0..=terrain_height(hx, hz).max(base + 3) {
                    w.remove(&(hx, y, hz));
                }
                w.insert((hx, base, hz), Block::Stone);
            }
        }
        build_house(&mut w, x, base, z, Block::Wood, Block::Brick);
    }

    w
}

fn save_world(world: &HashMap<BlockPos, Block>, path: &str) {
    let mut out = String::new();
    for ((x, y, z), b) in world {
        out.push_str(&format!("{} {} {} {}\n", x, y, z, b.name()));
    }
    if let Err(e) = fs::write(path, out) {
        eprintln!("บันทึกไม่สำเร็จ: {e}");
    } else {
        println!("บันทึกโลกแล้ว ({} บล็อก)", world.len());
    }
}

fn load_world(path: &str) -> HashMap<BlockPos, Block> {
    let mut w = HashMap::new();
    if let Ok(data) = fs::read_to_string(path) {
        for line in data.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 4 {
                if let (Ok(x), Ok(y), Ok(z)) =
                    (parts[0].parse(), parts[1].parse(), parts[2].parse())
                {
                    if let Some(b) = Block::from_name(parts[3]) {
                        w.insert((x, y, z), b);
                    }
                }
            }
        }
    }
    w
}

fn raycast(
    origin: Vec3,
    dir: Vec3,
    world: &HashMap<BlockPos, Block>,
    max_dist: f32,
) -> Option<(BlockPos, (i32, i32, i32))> {
    let mut voxel = [
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    ];
    let step: [i32; 3] = [
        if dir.x > 0.0 { 1 } else { -1 },
        if dir.y > 0.0 { 1 } else { -1 },
        if dir.z > 0.0 { 1 } else { -1 },
    ];
    let o = [origin.x, origin.y, origin.z];
    let d = [dir.x, dir.y, dir.z];

    let mut t_delta = [0.0f32; 3];
    let mut t_max = [0.0f32; 3];
    for i in 0..3 {
        t_delta[i] = if d[i] == 0.0 { f32::MAX } else { (1.0 / d[i]).abs() };
        let boundary = if step[i] > 0 {
            (voxel[i] + 1) as f32
        } else {
            voxel[i] as f32
        };
        t_max[i] = if d[i] == 0.0 { f32::MAX } else { (boundary - o[i]) / d[i] };
    }

    let mut normal = (0i32, 0i32, 0i32);
    let mut t = 0.0f32;
    while t <= max_dist {
        let key = (voxel[0], voxel[1], voxel[2]);
        if world.contains_key(&key) {
            return Some((key, normal));
        }
        if t_max[0] < t_max[1] && t_max[0] < t_max[2] {
            voxel[0] += step[0];
            t = t_max[0];
            t_max[0] += t_delta[0];
            normal = (-step[0], 0, 0);
        } else if t_max[1] < t_max[2] {
            voxel[1] += step[1];
            t = t_max[1];
            t_max[1] += t_delta[1];
            normal = (0, -step[1], 0);
        } else {
            voxel[2] += step[2];
            t = t_max[2];
            t_max[2] += t_delta[2];
            normal = (0, 0, -step[2]);
        }
    }
    None
}

// ตรวจ AABB ชนบล็อก (ใช้กับผู้เล่นและม็อบ)
fn aabb_collides(pos: Vec3, half: Vec3, world: &HashMap<BlockPos, Block>) -> bool {
    let min = pos - half;
    let max = pos + half;
    for x in (min.x.floor() as i32)..=(max.x.floor() as i32) {
        for y in (min.y.floor() as i32)..=(max.y.floor() as i32) {
            for z in (min.z.floor() as i32)..=(max.z.floor() as i32) {
                if world.contains_key(&(x, y, z)) {
                    return true;
                }
            }
        }
    }
    false
}

fn player_collides(pos: Vec3, world: &HashMap<BlockPos, Block>) -> bool {
    aabb_collides(pos, vec3(0.3, 0.85, 0.3), world)
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn play(s: &Option<Sound>) {
    if let Some(snd) = s {
        play_sound_once(snd);
    }
}

fn draw_loading(progress: f32, label: &str) {
    clear_background(Color::from_rgba(12, 16, 32, 255));
    let w = screen_width();
    let h = screen_height();
    let bar_w = 420.0;
    let bar_h = 22.0;
    let x = w / 2.0 - bar_w / 2.0;
    let y = h / 2.0;
    draw_text("Mini Roblox - Loading...", x, y - 52.0, 30.0, WHITE);
    draw_rectangle(x, y, bar_w, bar_h, Color::from_rgba(255, 255, 255, 40));
    draw_rectangle(x, y, bar_w * progress.clamp(0.0, 1.0), bar_h, Color::from_rgba(80, 200, 120, 255));
    draw_text(label, x, y + bar_h + 16.0, 20.0, Color::from_rgba(210, 210, 210, 255));
}

#[macroquad::main("Mini Roblox")]
async fn main() {
    let t0 = std::time::Instant::now();
    // หน้าจอโหลดทันที — กันจอดำระหว่างโหลด assets
    draw_loading(0.05, "Loading sounds...");
    next_frame().await;
    eprintln!("[load] window ok +{:.1}s, loading sounds...", t0.elapsed().as_secs_f32());

    // --- โหลดเสียง (ถ้าโหลดไม่ได้ก็เล่นแบบเงียบ) ---
    let snd_break = load_sound_from_bytes(include_bytes!("../assets/break.wav")).await.ok();
    let snd_place = load_sound_from_bytes(include_bytes!("../assets/place.wav")).await.ok();
    let snd_hurt = load_sound_from_bytes(include_bytes!("../assets/hurt.wav")).await.ok();
    let snd_eat = load_sound_from_bytes(include_bytes!("../assets/eat.wav")).await.ok();
    let snd_step = load_sound_from_bytes(include_bytes!("../assets/step.wav")).await.ok();
    let snd_growl = load_sound_from_bytes(include_bytes!("../assets/growl.wav")).await.ok();
    let snd_boss_growl = load_sound_from_bytes(include_bytes!("../assets/boss_growl.wav")).await.ok();
    let snd_boss_die = load_sound_from_bytes(include_bytes!("../assets/boss_die.wav")).await.ok();
    let snd_oof = load_sound_from_bytes(include_bytes!("../assets/oof.wav")).await.ok();

    // วินิจฉัยเสียง: พิมพ์จำนวนเสียงที่โหลดได้ (ถ้า 0 = ไม่มี sound device)
    let sound_n = [&snd_break, &snd_place, &snd_hurt, &snd_eat, &snd_step, &snd_growl,
        &snd_boss_growl, &snd_boss_die, &snd_oof]
        .iter()
        .filter(|s| s.is_some())
        .count();
    println!("เสียงโหลด {}/9{} +{:.1}s", sound_n, if sound_n == 0 { " — ไม่มี sound device" } else { "" }, t0.elapsed().as_secs_f32());
    draw_loading(0.3, "Loading textures...");
    next_frame().await;

    // ข้อความไทย (render เป็น PNG ล่วงหน้าด้วย HarfBuzz shaping แล้ว)
    let thai_win_tex = Texture2D::from_file_with_format(include_bytes!("../assets/thai_win.png"), None);
    let thai_auto_tex = Texture2D::from_file_with_format(include_bytes!("../assets/thai_auto.png"), None);
    // textures บล็อก
    let tex_grass = Texture2D::from_file_with_format(include_bytes!("../assets/tex_grass.png"), None);
    let tex_dirt = Texture2D::from_file_with_format(include_bytes!("../assets/tex_dirt.png"), None);
    let tex_stone = Texture2D::from_file_with_format(include_bytes!("../assets/tex_stone.png"), None);
    let tex_wood = Texture2D::from_file_with_format(include_bytes!("../assets/tex_wood.png"), None);
    let tex_sand = Texture2D::from_file_with_format(include_bytes!("../assets/tex_sand.png"), None);
    let tex_glass = Texture2D::from_file_with_format(include_bytes!("../assets/tex_glass.png"), None);
    let tex_brick = Texture2D::from_file_with_format(include_bytes!("../assets/tex_brick.png"), None);
    let tex_snow = Texture2D::from_file_with_format(include_bytes!("../assets/tex_snow.png"), None);
    draw_loading(0.55, "Building world...");
    next_frame().await;

    // --- โลก ---
    // แผนที่สุ่ม (seed จากเวลาปัจจุบัน -> ทุกครั้งที่เปิดเกมได้แผนที่ใหม่)
    let mut seed: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    TERRAIN_SEED.store(seed, std::sync::atomic::Ordering::Relaxed);
    let mut world = load_world(SAVE_FILE);
    if world.is_empty() {
        world = build_world(seed);
    }
    println!("โลก {} บล็อก (สุ่ม seed {}) +{:.1}s", world.len(), seed, t0.elapsed().as_secs_f32());

    // วอร์ม shader/ไปป์ไลน์ทั้งหมดล่วงหน้า — compile ตอนจอโหลด ไม่ให้เฟรมแรกค้าง
    // (สำคัญมากบนเครื่องที่ render ด้วย software/llvmpipe: JIT ครั้งแรกนานมาก)
    draw_loading(0.75, "Compiling shaders...");
    next_frame().await;
    let warm_tex = Texture2D::from_file_with_format(include_bytes!("../assets/tex_grass.png"), None);
    // วอร์มทุกไปป์ไลน์ที่เกมใช้: 3D มี/ไม่มี texture, ทรงกลม, เส้น, รูป 2D, ตัวอักษร
    draw_cube(vec3(0.0, 0.0, -3.0), vec3(1.0, 1.0, 1.0), Some(&warm_tex), WHITE);
    draw_cube(vec3(2.0, 0.0, -3.0), vec3(1.0, 1.0, 1.0), None, WHITE);
    draw_sphere(vec3(4.0, 0.0, -3.0), 0.5, None, WHITE);
    draw_grid(2, 1.0, WHITE, WHITE);
    draw_rectangle(10.0, 10.0, 8.0, 8.0, WHITE);
    draw_circle(30.0, 30.0, 4.0, WHITE);
    draw_text("warmup", 50.0, 30.0, 16.0, WHITE);
    next_frame().await;
    eprintln!("[load] shaders warmed +{:.1}s", t0.elapsed().as_secs_f32());
    draw_loading(0.92, "Rendering first frame...");
    next_frame().await;

    // จุดเกิด (กลางแผนที่ บนที่ราบสูง)
    let cx = WORLD_SIZE / 2;
    let mut plaza_h = plaza_height();
    let mut spawn_pos = vec3(cx as f32 + 0.5, plaza_h as f32 + 2.0, cx as f32 + 0.5);

    // --- ผู้เล่น ---
    let mut cam_pos = spawn_pos;
    let mut vel = vec3(0.0, 0.0, 0.0);
    let mut grounded = false;
    let mut third_person = true;
    let mut mouse_grabbed = true;
    let mut yaw: f32 = -135.0_f32.to_radians();
    let mut pitch: f32 = -25.0_f32.to_radians();
    let mut selected = Block::Stone;

    // --- การตั้งค่า (ปรับได้ในเกม เมนู F2 + เซฟ settings.txt) ---
    let mut mouse_sens: f32 = 0.02;
    let mut move_speed: f32 = 6.0;
    let mut jump_speed: f32 = 8.0;
    let mut day_length: f32 = 90.0;
    let mut gpad_inv_x: bool = false;
    let mut gpad_inv_y: bool = false;
    let mut settings_open = false;
    let mut settings_sel: usize = 0;
    if let Ok(txt) = std::fs::read_to_string(SETTINGS_FILE) {
        for line in txt.lines() {
            let mut it = line.split_whitespace();
            let key = it.next().unwrap_or("");
            let val = it.next().unwrap_or("");
            match key {
                "mouse_sens" => if let Ok(v) = val.parse::<f32>() { mouse_sens = v.clamp(0.005, 0.10) },
                "move_speed" => if let Ok(v) = val.parse::<f32>() { move_speed = v.clamp(3.0, 12.0) },
                "jump_speed" => if let Ok(v) = val.parse::<f32>() { jump_speed = v.clamp(5.0, 12.0) },
                "day_length" => if let Ok(v) = val.parse::<f32>() { day_length = v.clamp(30.0, 300.0) },
                "gpad_inv_x" => gpad_inv_x = val == "1",
                "gpad_inv_y" => gpad_inv_y = val == "1",
                _ => {}
            }
        }
    }

    let mut gilrs = Gilrs::new().ok();

    // --- สถานะ survival ---
    let mut time_of_day: f32 = 0.0; // เริ่มตอนเที่ยง (กลางวัน มีเวลาเก็บแอปเปิ้ลก่อนมืด)
    let mut hp: f32 = 100.0;
    let mut hunger: f32 = 100.0;
    let mut apples: i32 = 0;
    let mut wolves: Vec<Wolf> = Vec::new();
    let mut wolf_spawn_timer: f32 = 0.0;
    let mut hurt_cd: f32 = 0.0;
    let mut step_timer: f32 = 0.0;
    // โหมดเล่นอัตโนมัติ (AI เล่นให้ดู)
    let mut autoplay = false;
    let mut auto_attack_cd: f32 = 0.0;
    // ตรวจจับการติดขัดของ AI
    let mut prev_night = false;
    let mut stuck_timer: f32 = 0.0;
    let mut last_auto_pos = cam_pos;
    let mut was_auto_moving = false;
    // โหมดฝึก (ไม่มีศัตรู): หัดวาง/ทุบ/ตีหุ่น
    let mut practice = false;
    let mut dummy_hits: i32 = 0;
    let mut dummy_flash: f32 = 0.0;
    let dummy_pos = vec3(cx as f32 - 1.0, plaza_h as f32 + 1.9, cx as f32 + 2.0);
    // ที่กำบัง (กำแพง 2 ชั้น รอบพื้นที่ 2x2 ตรงจุดเกิด) สร้างตอนเย็น
    let mut shelter_built = false;
    let mut shelter_idx: usize = 0;
    let mut shelter_timer: f32 = 0.0;
    let mut shelter_blocks: Vec<BlockPos> = Vec::new();
    for sy in 1..=2 {
        for sx in cx - 1..=cx + 2 {
            for sz in cx - 1..=cx + 2 {
                if sx >= cx && sx <= cx + 1 && sz >= cx && sz <= cx + 1 {
                    continue;
                }
                shelter_blocks.push((sx, plaza_h + sy, sz));
            }
        }
    }

    // จุดเกิดหมาป่า (มุมแผนที่)
    let e = WORLD_SIZE - 2;
    let mut spawn_points: [(f32, f32, f32); 4] = [
        (2.0, terrain_height(2, 2) as f32 + 2.0, 2.0),
        (e as f32, terrain_height(e, e) as f32 + 2.0, e as f32),
        (2.0, terrain_height(2, e) as f32 + 2.0, e as f32),
        (e as f32, terrain_height(e, 2) as f32 + 2.0, 2.0),
    ];
    // แอปเปิ้ลสุ่ม 8 ลูกบนพื้น
    let mut arng = Rng::new(seed ^ 0x9E3779B97F4A7C15);
    let mut apple_items: Vec<Apple> = Vec::new();
    for _ in 0..8 {
        let ax = arng.range(3, WORLD_SIZE - 4);
        let az = arng.range(3, WORLD_SIZE - 4);
        let ay = surface_height(ax, az) as f32 + 1.2;
        apple_items.push(Apple { pos: vec3(ax as f32, ay, az as f32), taken: false });
    }

    // --- ระบบอนุภาค + ระดับชั้น + บอส ---
    let mut particles: Vec<Particle> = Vec::new();
    let mut level: i32 = 1;             // 1 = พื้นฐาน, 2 = ยากขึ้น, 3 = บอส
    let mut nights_survived: i32 = 0;   // จำนวนคืนที่รอดแล้ว
    let mut game_won = false;           // ชนะเกม (ปราบบอส)
    let mut boss_spawned = false;       // บอสเกิดแล้วหรือยัง
    let mut level_flash: f32 = 0.0;     // แสดง LEVEL UP ชั่วคราว
    let mut sword_swing: f32 = 0.0;     // จังหวะฟันดาบ
    let mut cam_dist: f32 = 4.0;        // ระยะกล้องบุคคลที่ 3 (ซูมด้วย scroll)

    set_cursor_grab(true);
    let mut first_frame = true;

    loop {
        let dt = get_frame_time();

        // --- วงจรกลางวัน/กลางคืน ---
        time_of_day = (time_of_day + dt / day_length) % 1.0;
        let day_factor = (time_of_day * std::f32::consts::TAU).cos(); // 1=เที่ยง, -1=เที่ยงคืน
        let nightness = if practice { 0.0 } else { (-day_factor).clamp(0.0, 1.0) }; // 0=กลางวัน, 1=กลางคืน
        let is_night = if practice { false } else { day_factor < -0.2 };

        // --- อ่านเกมแพด (จอยสติ๊ก) ---
        let mut gp_move = Vec2::ZERO;
        let mut gp_look = Vec2::ZERO;
        let mut gp_jump = false;
        let mut gp_break = false;
        let mut gp_place = false;
        let mut gp_view = false;
        let mut gp_sel_next = false;
        let mut gp_sel_prev = false;
        let mut gp_settings = false;
        let mut gp_connected = false;
        if let Some(g) = gilrs.as_mut() {
            while g.next_event().is_some() {}
            for (_id, pad) in g.gamepads() {
                gp_connected = true;
                let lx = pad.value(Axis::LeftStickX);
                let ly = pad.value(Axis::LeftStickY);
                let rx = pad.value(Axis::RightStickX);
                let ry = pad.value(Axis::RightStickY);
                const DZ: f32 = 0.15; // dead zone กันจอยดริฟต์
                let sx = if gpad_inv_x { -1.0 } else { 1.0 };
                let sy = if gpad_inv_y { -1.0 } else { 1.0 };
                if lx.abs() > DZ || ly.abs() > DZ { gp_move = vec2(lx * sx, ly * sy); }
                if rx.abs() > DZ || ry.abs() > DZ { gp_look = vec2(rx * sx, ry * sy); }
                gp_jump |= pad.is_pressed(Button::South);
                gp_break |= pad.is_pressed(Button::West);
                gp_place |= pad.is_pressed(Button::East);
                gp_view |= pad.is_pressed(Button::North);
                gp_sel_next |= pad.is_pressed(Button::DPadRight);
                gp_sel_prev |= pad.is_pressed(Button::DPadLeft);
                gp_settings |= pad.is_pressed(Button::Start);
            }
        }
        // รอดกลางคืน (กลางคืน → กลางวัน) = ผ่าน 1 คืน → เลื่อนระดับ
        if !is_night && prev_night && !game_won {
            nights_survived += 1;
            let new_level = (nights_survived + 1).min(3);
            if new_level > level {
                level_flash = 3.0;   // แสดง "LEVEL UP!"
            }
            level = new_level;
        }
        prev_night = is_night;

        sword_swing = (sword_swing - dt).max(0.0);
        cam_dist = (cam_dist - mouse_wheel().1 * 0.5).clamp(2.0, 10.0);
        level_flash = (level_flash - dt).max(0.0);
        let max_wolves = match level { 1 => 2, 2 => 3, _ => 4 };
        let wolf_speed = match level { 1 => 3.5, 2 => 4.0, _ => 4.5 };

        // --- เปิด/ปิดโหมดเล่นอัตโนมัติ (ปุ่ม P) ---
        if is_key_pressed(KeyCode::P) {
            autoplay = !autoplay;
        }
        // --- เปิด/ปิดโหมดฝึก (ปุ่ม M, ไม่มีศัตรู) ---
        if is_key_pressed(KeyCode::M) {
            practice = !practice;
            if practice {
                wolves.clear();
                dummy_hits = 0;
            }
        }

        // --- เมนูตั้งค่า (F2 หรือ O) ---
        if is_key_pressed(KeyCode::F2) || is_key_pressed(KeyCode::O) || gp_settings {
            settings_open = !settings_open;
            if settings_open {
                mouse_grabbed = false;
                set_cursor_grab(false);
            }
        }
        if settings_open {
            if is_key_pressed(KeyCode::Up) {
                settings_sel = (settings_sel + 5) % 6;
            }
            if is_key_pressed(KeyCode::Down) {
                settings_sel = (settings_sel + 1) % 6;
            }
            let dir = if is_key_pressed(KeyCode::Left) {
                -1.0
            } else if is_key_pressed(KeyCode::Right) {
                1.0
            } else {
                0.0
            };
            if dir != 0.0 {
                match settings_sel {
                    0 => mouse_sens = (mouse_sens + dir * 0.005).clamp(0.005, 0.10),
                    1 => move_speed = (move_speed + dir * 0.5).clamp(3.0, 12.0),
                    2 => jump_speed = (jump_speed + dir * 0.5).clamp(5.0, 12.0),
                    3 => day_length = (day_length + dir * 15.0).clamp(30.0, 300.0),
                    4 => gpad_inv_x = dir > 0.0,
                    _ => gpad_inv_y = dir > 0.0,
                }
                let txt = format!(
                    "mouse_sens {:.3}\nmove_speed {:.1}\njump_speed {:.1}\nday_length {:.0}\ngpad_inv_x {}\ngpad_inv_y {}\n",
                    mouse_sens, move_speed, jump_speed, day_length,
                    gpad_inv_x as u8, gpad_inv_y as u8
                );
                let _ = std::fs::write(SETTINGS_FILE, txt);
            }
        }

        // --- มองรอบด้วยเมาส์ (เฉพาะตอนเล่นเอง) ---
        if mouse_grabbed && !autoplay && !settings_open {
            let mouse_delta = mouse_delta_position();
            yaw -= mouse_delta.x * mouse_sens;
            pitch = (pitch - mouse_delta.y * mouse_sens).clamp(-1.5, 1.5);
            yaw -= gp_look.x * GAMEPAD_LOOK_SPEED * dt;
            pitch = (pitch - gp_look.y * GAMEPAD_LOOK_SPEED * dt).clamp(-1.5, 1.5);
        }

        // --- โหมดเล่นอัตโนมัติ: AI เดินหาแอปเปิ้ล + สู้หมาป่า ---
        let mut auto_dir = Vec3::ZERO;
        if autoplay {
            // ตรวจจับการติดขัด (เฟรมที่แล้วกำลังเดิน แต่ขยับแทบไม่ได้)
            if was_auto_moving {
                let moved = (cam_pos - last_auto_pos).length();
                if moved < 0.05 {
                    stuck_timer += dt;
                } else {
                    stuck_timer = 0.0;
                }
            }
            let mut target: Option<Vec3> = None;
            let mut min_d = f32::MAX;
            let mut target_is_wolf = false;
            if !shelter_built && !is_night && day_factor < 0.3 {
                // ตอนเย็น: กลับไปจุดเกิดแล้วสร้างกำแพงกันหมาป่า
                let home = vec3(spawn_pos.x, cam_pos.y, spawn_pos.z);
                let dist_home = (home - cam_pos).length();
                if dist_home > 1.5 {
                    target = Some(home);
                } else {
                    // อยู่ที่บ้านแล้ว → วางกำแพงทีละก้อน (หันหน้าไปทางก้อนที่วาง)
                    auto_dir = Vec3::ZERO;
                    if shelter_idx < shelter_blocks.len() {
                        let b = shelter_blocks[shelter_idx];
                        let bp = vec3(b.0 as f32 + 0.5, b.1 as f32 + 0.5, b.2 as f32 + 0.5);
                        let to = bp - cam_pos;
                        let flat = vec3(to.x, 0.0, to.z);
                        let fl = flat.length();
                        if fl > 0.05 {
                            let dn = flat / fl;
                            yaw = dn.x.atan2(dn.z);
                            pitch = (to.y / fl).atan().clamp(-1.5, 1.5);
                        }
                        shelter_timer -= dt;
                        if shelter_timer <= 0.0 {
                            if !world.contains_key(&b) {
                                world.insert(b, Block::Stone);
                                play(&snd_place);
                            }
                            shelter_idx += 1;
                            shelter_timer = 0.25;
                        }
                    } else {
                        shelter_built = true;
                    }
                }
            } else if is_night {
                // กลางคืน: สู้หมาป่าที่ใกล้ ไม่งั้นรออยู่ในที่กำบัง
                for w in &wolves {
                    let d = (w.pos - cam_pos).length();
                    if d < 8.0 && d < min_d {
                        min_d = d;
                        target = Some(w.pos);
                        target_is_wolf = true;
                    }
                }
                if target.is_none() {
                    target = Some(vec3(spawn_pos.x, cam_pos.y, spawn_pos.z));
                }
            } else {
                // กลางวัน: เก็บแอปเปิ้ลใกล้สุด
                for a in &apple_items {
                    if !a.taken {
                        let d = (a.pos - cam_pos).length();
                        if d < min_d {
                            min_d = d;
                            target = Some(a.pos);
                            target_is_wolf = false;
                        }
                    }
                }
                if target.is_none() {
                    for w in &wolves {
                        let d = (w.pos - cam_pos).length();
                        if d < min_d {
                            min_d = d;
                            target = Some(w.pos);
                            target_is_wolf = true;
                        }
                    }
                }
            }
            if let Some(t) = target {
                let to = t - cam_pos;
                let flat = vec3(to.x, 0.0, to.z);
                let hd = flat.length();
                if hd > 0.2 {
                    let dirn = flat / hd;
                    yaw = dirn.x.atan2(dirn.z);
                    pitch = (to.y / hd).atan().clamp(-1.5, 1.5);
                    auto_dir = dirn;
                }
                // หยุดเดินเมื่อถึงเป้าหมาย (หมาป่าใกล้ / ถึงจุดหมาย)
                if (target_is_wolf && min_d < 2.0) || (!target_is_wolf && min_d < 0.6) {
                    auto_dir = Vec3::ZERO;
                }
                // ตีหมาป่าเมื่อเข้าใกล้พอ
                if target_is_wolf && min_d < 2.5 {
                    auto_attack_cd -= dt;
                    if auto_attack_cd <= 0.0 {
                        for w in &mut wolves {
                            let tw = w.pos - cam_pos;
                            if tw.length() < 2.5 {
                                w.hp -= 1;
                                play(&snd_break);
                                auto_attack_cd = 0.5;
                                break;
                            }
                        }
                    }
                }
            }
        }

        let forward = vec3(
            pitch.cos() * yaw.sin(),
            pitch.sin(),
            pitch.cos() * yaw.cos(),
        )
        .normalize();
        let right = vec3(-yaw.cos(), 0.0, yaw.sin());

        // --- เดิน ---
        let mut horiz = Vec3::ZERO;
        if autoplay {
            horiz = auto_dir * move_speed;
        } else if !settings_open {
            if is_key_down(KeyCode::W) { horiz += forward; }
            if is_key_down(KeyCode::S) { horiz -= forward; }
            if is_key_down(KeyCode::D) { horiz += right; }
            if is_key_down(KeyCode::A) { horiz -= right; }
            if gp_move != Vec2::ZERO {
                horiz += forward * (-gp_move.y) + right * gp_move.x;
            }
            if horiz != Vec3::ZERO {
                horiz = horiz.normalize() * move_speed;
            }
        }
        vel.x = horiz.x;
        vel.z = horiz.z;

        // เสียงเดิน
        if horiz != Vec3::ZERO && grounded {
            step_timer -= dt;
            if step_timer <= 0.0 {
                play(&snd_step);
                step_timer = 0.4;
            }
        }

        vel.y -= GRAVITY * dt;
        vel.y = vel.y.max(-30.0);
        if grounded && ((!settings_open && (is_key_pressed(KeyCode::Space) || gp_jump)) || (autoplay && stuck_timer > 0.4)) {
            vel.y = jump_speed;
            grounded = false;
            stuck_timer = 0.0;
        }

        let nx = cam_pos.x + vel.x * dt;
        if !player_collides(vec3(nx, cam_pos.y, cam_pos.z), &world) {
            cam_pos.x = nx;
        } else {
            vel.x = 0.0;
        }
        let ny = cam_pos.y + vel.y * dt;
        if !player_collides(vec3(cam_pos.x, ny, cam_pos.z), &world) {
            cam_pos.y = ny;
            grounded = false;
        } else {
            if vel.y < 0.0 { grounded = true; }
            vel.y = 0.0;
        }
        let nz = cam_pos.z + vel.z * dt;
        if !player_collides(vec3(cam_pos.x, cam_pos.y, nz), &world) {
            cam_pos.z = nz;
        } else {
            vel.z = 0.0;
        }

        // --- ปุ่มต่าง ๆ ---
        if is_key_pressed(KeyCode::Key1) { selected = Block::Grass; }
        if is_key_pressed(KeyCode::Key2) { selected = Block::Dirt; }
        if is_key_pressed(KeyCode::Key3) { selected = Block::Stone; }
        if is_key_pressed(KeyCode::Key4) { selected = Block::Wood; }
        if is_key_pressed(KeyCode::Key5) { selected = Block::Sand; }
        if is_key_pressed(KeyCode::Key6) { selected = Block::Glass; }
        if is_key_pressed(KeyCode::Key7) { selected = Block::Brick; }
        if is_key_pressed(KeyCode::Key8) { selected = Block::Snow; }
        if gp_sel_next { selected = block_next(selected); }
        if gp_sel_prev { selected = block_prev(selected); }
        if is_key_pressed(KeyCode::R) {
            save_world(&world, SAVE_FILE);
        }
        if is_key_pressed(KeyCode::T) {
            cam_pos = spawn_pos;
            vel = Vec3::ZERO;
            grounded = false;
        }
        if is_key_pressed(KeyCode::V) || gp_view {
            third_person = !third_person;
        }
        if is_key_pressed(KeyCode::Escape) {
            if settings_open {
                settings_open = false;
            } else {
                mouse_grabbed = !mouse_grabbed;
                set_cursor_grab(mouse_grabbed);
            }
        }
        // เริ่มเกมใหม่ (กด Enter หลังชนะ) -> สุ่มแผนที่ใหม่
        if game_won && is_key_pressed(KeyCode::Enter) {
            seed = seed.wrapping_add(0x9E3779B97F4A7C15);
            TERRAIN_SEED.store(seed, std::sync::atomic::Ordering::Relaxed);
            world = build_world(seed);
            plaza_h = plaza_height();
            spawn_pos = vec3(cx as f32 + 0.5, plaza_h as f32 + 2.0, cx as f32 + 0.5);
            // ที่กำบังใหม่รอบจุดเกิด
            shelter_blocks.clear();
            for sy in 1..=2 {
                for sx in cx - 1..=cx + 2 {
                    for sz in cx - 1..=cx + 2 {
                        if sx >= cx && sx <= cx + 1 && sz >= cx && sz <= cx + 1 {
                            continue;
                        }
                        shelter_blocks.push((sx, plaza_h + sy, sz));
                    }
                }
            }
            // จุดเกิดหมาป่าใหม่
            spawn_points = [
                (2.0, terrain_height(2, 2) as f32 + 2.0, 2.0),
                (e as f32, terrain_height(e, e) as f32 + 2.0, e as f32),
                (2.0, terrain_height(2, e) as f32 + 2.0, e as f32),
                (e as f32, terrain_height(e, 2) as f32 + 2.0, 2.0),
            ];
            // แอปเปิ้ลใหม่
            let mut arng = Rng::new(seed ^ 0x9E3779B97F4A7C15);
            apple_items.clear();
            for _ in 0..8 {
                let ax = arng.range(3, WORLD_SIZE - 4);
                let az = arng.range(3, WORLD_SIZE - 4);
                let ay = surface_height(ax, az) as f32 + 1.2;
                apple_items.push(Apple { pos: vec3(ax as f32, ay, az as f32), taken: false });
            }
            level = 1;
            nights_survived = 0;
            game_won = false;
            boss_spawned = false;
            level_flash = 0.0;
            hp = 100.0;
            hunger = 100.0;
            apples = 0;
            wolves.clear();
            particles.clear();
            cam_pos = spawn_pos;
            vel = Vec3::ZERO;
            grounded = false;
            time_of_day = 0.0;
        }
        if is_key_pressed(KeyCode::F10) {
            break;
        }

        // --- ความหิว (โหมดฝึกไม่หิว) ---
        if !practice {
            hunger = (hunger - dt * 0.6).max(0.0);
            if hunger <= 0.0 {
                hp -= dt * 2.0;
            }
        }
        // กินแอปเปิ้ล (กด E หรือ AI กินเองเมื่อหิว)
        if apples > 0 && (is_key_pressed(KeyCode::E) || (autoplay && hunger < 55.0)) {
            apples -= 1;
            hunger = (hunger + 35.0).min(100.0);
            play(&snd_eat);
        }

        // --- หมาป่าเกิดตอนกลางคืน ---
        if is_night && wolves.len() < max_wolves {
            wolf_spawn_timer -= dt;
            if wolf_spawn_timer <= 0.0 {
                let sp = spawn_points[wolves.len()];
                wolves.push(Wolf {
                    pos: vec3(sp.0, sp.1, sp.2),
                    vel: Vec3::ZERO,
                    hp: 3,
                    grounded: false,
                    boss: false,
                });
                wolf_spawn_timer = 3.0;
                play(&snd_growl);
            }
        }

        // บอส: ระดับ 3 ตอนกลางคืน เกิดครั้งเดียว (ตัวใหญ่ เลือดเยอะ)
        if level >= 3 && is_night && !boss_spawned && !game_won {
            boss_spawned = true;
            wolves.push(Wolf {
                pos: vec3(0.0, 2.0, -9.0),
                vel: Vec3::ZERO,
                hp: 30,
                grounded: false,
                boss: true,
            });
            play(&snd_boss_growl);
        }

        // อัปเดตอนุภาค (แรงโน้มถ่วง + อายุ)
        for p in &mut particles {
            p.vel.y -= GRAVITY * 0.5 * dt;
            p.pos += p.vel * dt;
            p.life -= dt;
        }
        particles.retain(|p| p.life > 0.0);

        // --- หมาป่า AI: ไล่ + กัด ---
        for w in &mut wolves {
            let to_player = cam_pos - w.pos;
            let dist = to_player.length();
            let dir = to_player / dist.max(0.001);
            w.vel.x = dir.x * wolf_speed;
            w.vel.z = dir.z * wolf_speed;
            w.vel.y -= GRAVITY * dt;
            w.vel.y = w.vel.y.max(-30.0);

            let nx = w.pos.x + w.vel.x * dt;
            if !aabb_collides(vec3(nx, w.pos.y, w.pos.z), vec3(0.35, 0.4, 0.35), &world) {
                w.pos.x = nx;
            } else {
                w.vel.x = 0.0;
            }
            let ny = w.pos.y + w.vel.y * dt;
            if !aabb_collides(vec3(w.pos.x, ny, w.pos.z), vec3(0.35, 0.4, 0.35), &world) {
                w.pos.y = ny;
                w.grounded = false;
            } else {
                if w.vel.y < 0.0 { w.grounded = true; }
                w.vel.y = 0.0;
            }
            let nz = w.pos.z + w.vel.z * dt;
            if !aabb_collides(vec3(w.pos.x, w.pos.y, nz), vec3(0.35, 0.4, 0.35), &world) {
                w.pos.z = nz;
            } else {
                w.vel.z = 0.0;
            }

            if dist < 1.6 {
                hurt_cd -= dt;
                if hurt_cd <= 0.0 {
                    hp -= if w.boss { 20.0 } else { 12.0 };
                    hurt_cd = 1.0;
                    play(&snd_hurt);
                }
            }
        }

        // --- ต่อสู้: คลิกซ้ายตีหมาป่า ---
        if mouse_grabbed && (is_mouse_button_pressed(MouseButton::Left) || gp_break) {
            sword_swing = 0.3;
            let mut hit_wolf = false;
            for w in &mut wolves {
                let to_wolf = w.pos - cam_pos;
                let d = to_wolf.length();
                if d < REACH && forward.dot(to_wolf / d.max(0.001)) > 0.7 {
                    w.hp -= 2;   // ดาบฟันแรงกว่า
                    hit_wolf = true;
                }
            }
            if hit_wolf {
                play(&snd_break);
            }
        }
        let boss_dead_pos = wolves.iter().find(|w| w.boss && w.hp <= 0).map(|w| w.pos);
        wolves.retain(|w| w.hp > 0);
        // บอสตาย → ชนะเกม + ระเบิด
        if boss_spawned && !game_won && !wolves.iter().any(|w| w.boss) {
            game_won = true;
            play(&snd_boss_die);
            if let Some(bp) = boss_dead_pos {
                for _ in 0..60 {
                    particles.push(Particle {
                        pos: bp + vec3(0.0, 1.0, 0.0),
                        vel: vec3(rand::gen_range(-8.0, 8.0), rand::gen_range(0.0, 10.0), rand::gen_range(-8.0, 8.0)),
                        life: rand::gen_range(0.4, 1.2),
                        color: Color::from_rgba(255, 140, 40, 255),
                    });
                }
            }
        }

        // --- เก็บแอปเปิ้ล ---
        for a in &mut apple_items {
            if !a.taken && (a.pos - cam_pos).length() < 1.5 {
                a.taken = true;
                apples += 1;
                play(&snd_eat);
            }
        }

        // --- ตาย -> เกิดใหม่ ---
        if hp <= 0.0 {
            hp = 100.0;
            hunger = 100.0;
            cam_pos = spawn_pos;
            vel = Vec3::ZERO;
            wolves.clear();
            play(&snd_oof);
            for a in &mut apple_items {
                a.taken = false;
            }
        }

        // --- วาด ---
        let sky = lerp_color(Color::from_rgba(20, 25, 50, 255), SKYBLUE, 1.0 - nightness);
        clear_background(sky);

        let horiz_fwd = vec3(yaw.sin(), 0.0, yaw.cos()).normalize();
        let (cam_position, cam_target) = if third_person {
            (cam_pos - horiz_fwd * cam_dist + vec3(0.0, 0.6, 0.0), cam_pos + vec3(0.0, 0.4, 0.0))
        } else {
            (cam_pos, cam_pos + forward)
        };
        let camera = Camera3D {
            position: cam_position,
            target: cam_target,
            up: vec3(0.0, 1.0, 0.0),
            fovy: 45.0,
            ..Default::default()
        };
        set_camera(&camera);

        // ดวงอาทิตย์/ดวงจันทร์โคจรตามเวลา
        let sun_angle = time_of_day * std::f32::consts::TAU;
        let sun_dir = vec3(sun_angle.cos(), sun_angle.sin().abs() * 0.7 + 0.25, 0.25);
        let sun_pos = cam_pos + sun_dir * 40.0;
        let sun_color = lerp_color(Color::from_rgba(255, 235, 150, 255), Color::from_rgba(225, 228, 242, 255), nightness);
        draw_sphere(sun_pos, 2.8, None, sun_color);

        draw_grid(20, 1.0, Color::from_rgba(0, 0, 0, 60), Color::from_rgba(200, 200, 200, 40));
        let light = 1.0 - nightness * 0.6;
        for (pos, block) in &world {
            let p = vec3(pos.0 as f32 + 0.5, pos.1 as f32 + 0.5, pos.2 as f32 + 0.5);
            let d2 = (p - cam_pos).length_squared();
            if d2 > 45.0 * 45.0 {
                continue; // ไกลเกินหมอก ไม่ต้องวาด (ประหยัดมาก)
            }
            let dist = d2.sqrt();
            let fog = ((dist - 20.0) / 25.0).clamp(0.0, 1.0);
            let b = (1.0 - fog) * light;
            let color = Color::new(b, b, b, 1.0);
            let tex = match block {
                Block::Grass => &tex_grass,
                Block::Dirt => &tex_dirt,
                Block::Stone => &tex_stone,
                Block::Wood => &tex_wood,
                Block::Sand => &tex_sand,
                Block::Glass => &tex_glass,
                Block::Brick => &tex_brick,
                Block::Snow => &tex_snow,
            };
            // วาดเฉพาะด้านที่ติดอากาศ (face culling) — ลดจำนวน face ที่ต้องวาดลง ~3-4 เท่า
            let (x, y, z) = *pos;
            let draw_face = |o: Vec3, e1: Vec3, e2: Vec3| {
                draw_affine_parallelogram(o, e1, e2, Some(tex), color);
            };
            if !world.contains_key(&(x, y + 1, z)) {
                draw_face(p + vec3(-0.5, 0.5, -0.5), vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));
            }
            if !world.contains_key(&(x, y - 1, z)) {
                draw_face(p + vec3(-0.5, -0.5, 0.5), vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, -1.0));
            }
            if !world.contains_key(&(x + 1, y, z)) {
                draw_face(p + vec3(0.5, -0.5, -0.5), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0));
            }
            if !world.contains_key(&(x - 1, y, z)) {
                draw_face(p + vec3(-0.5, -0.5, 0.5), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, -1.0));
            }
            if !world.contains_key(&(x, y, z + 1)) {
                draw_face(p + vec3(-0.5, -0.5, 0.5), vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
            }
            if !world.contains_key(&(x, y, z - 1)) {
                draw_face(p + vec3(0.5, -0.5, -0.5), vec3(-1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
            }
        }
        // เงาใต้ตัวละคร (blob shadow)
        let shadow = Color::from_rgba(0, 0, 0, 90);
        draw_cube(vec3(cam_pos.x, 0.03, cam_pos.z), vec3(0.7, 0.03, 0.7), None, shadow);
        for w in &wolves {
            draw_cube(vec3(w.pos.x, 0.03, w.pos.z), vec3(0.8, 0.03, 0.8), None, shadow);
        }
        // อนุภาคทุบบล็อก
        for p in &particles {
            let s = 0.14 * (p.life / 0.7).max(0.05);
            draw_cube(p.pos, vec3(s, s, s), None, p.color);
        }
        // เสาแสงที่จุดเกิด (มองเห็นจากไกล — รู้ทางกลับบ้าน)
        let beacon_base = vec3(spawn_pos.x, plaza_h as f32 + 1.0, spawn_pos.z);
        draw_cube(beacon_base + vec3(0.0, 7.0, 0.0), vec3(0.6, 7.0, 0.6), None, Color::from_rgba(255, 200, 60, 90));
        draw_cube_wires(beacon_base + vec3(0.0, 7.0, 0.0), vec3(0.6, 7.0, 0.6), GOLD);

        // ดาบ (tool ในมือ — กวัดแกว่งตอนฟัน)
        if sword_swing > 0.0 {
            let t = 1.0 - sword_swing / 0.3;
            let blade = cam_pos + forward * (1.0 - t * 0.3) + right * (0.4 - t * 0.7) - vec3(0.0, 0.15 + t * 0.5, 0.0);
            draw_cube(blade, vec3(0.07, 0.07, 1.1), None, Color::from_rgba(215, 215, 230, 255));
            draw_cube(blade - forward * 0.65, vec3(0.11, 0.11, 0.28), None, Color::from_rgba(120, 70, 30, 255));
        }

        // แอปเปิ้ลบนพื้น
        for a in &apple_items {
            if !a.taken {
                draw_cube(a.pos, vec3(0.25, 0.25, 0.25), None, Color::from_rgba(220, 60, 50, 255));
            }
        }

        // หมาป่า
        for w in &wolves {
            let s = if w.boss { 2.2 } else { 1.0 };
            let body_col = if w.boss { Color::from_rgba(150, 25, 35, 255) } else { Color::from_rgba(90, 90, 95, 255) };
            let head_col = if w.boss { Color::from_rgba(110, 15, 25, 255) } else { Color::from_rgba(70, 70, 75, 255) };
            // เสาแสงแดงเหนือหมาป่า (มองเห็นตำแหน่งจากไกล)
            draw_cube(vec3(w.pos.x, w.pos.y + 1.3 * s, w.pos.z), vec3(0.3 * s, 1.6 * s, 0.3 * s), None, Color::from_rgba(255, 45, 45, 110));
            draw_cube_wires(vec3(w.pos.x, w.pos.y + 1.3 * s, w.pos.z), vec3(0.3 * s, 1.6 * s, 0.3 * s), Color::from_rgba(255, 70, 70, 220));
            let to_player = (cam_pos - w.pos).normalize();
            let body = w.pos + vec3(0.0, 0.2 * s, 0.0);
            draw_cube(body, vec3(0.6 * s, 0.5 * s, 0.9 * s), None, body_col);
            let head = body + to_player * 0.5 * s + vec3(0.0, 0.2 * s, 0.0);
            draw_cube(head, vec3(0.4 * s, 0.4 * s, 0.4 * s), None, head_col);
            // ตาสีแดง (ด้านหน้า = ไปทางผู้เล่น)
            let eye_r = right * 0.1 * s;
            draw_cube(head + to_player * 0.2 * s - eye_r, vec3(0.08 * s, 0.08 * s, 0.02), None, RED);
            draw_cube(head + to_player * 0.2 * s + eye_r, vec3(0.08 * s, 0.08 * s, 0.02), None, RED);
            // แถบเลือดบอส (กว้างกว่า เหนือหัว)
            if w.boss {
                let hp_frac = (w.hp as f32 / 30.0).max(0.0);
                let bar = w.pos + vec3(0.0, 1.6 * s, 0.0);
                draw_cube(bar, vec3(1.6, 0.12, 0.05), None, Color::from_rgba(40, 40, 40, 200));
                draw_cube(bar + vec3(-0.8 + 0.8 * hp_frac, 0.0, 0.0), vec3(1.6 * hp_frac, 0.09, 0.05), None, RED);
            }
        }

        // หุ่นฝึก (โหมดฝึก)
        if practice {
            if dummy_flash > 0.0 {
                dummy_flash -= dt;
            }
            let dcolor = if dummy_flash > 0.0 { WHITE } else { Color::from_rgba(220, 50, 50, 255) };
            let to_dummy = (cam_pos - dummy_pos).normalize();
            draw_cube(dummy_pos, vec3(0.7, 1.8, 0.4), None, dcolor);
            draw_cube(dummy_pos + to_dummy * 0.21, vec3(0.32, 0.32, 0.05), None, WHITE);
        }

        // ตัวผู้เล่น (บุคคลที่ 3)
        if third_person {
            let skin = Color::from_rgba(230, 190, 150, 255);
            let shirt = Color::from_rgba(40, 90, 200, 255);
            let pants = Color::from_rgba(30, 45, 90, 255);
            let rot = |lx: f32, ly: f32, lz: f32| vec3(
                cam_pos.x + lx * yaw.cos() + lz * yaw.sin(),
                cam_pos.y + ly,
                cam_pos.z - lx * yaw.sin() + lz * yaw.cos(),
            );
            let moving = horiz != Vec3::ZERO;
            let swing = if moving { (get_time() as f32 * 8.0).sin() * 0.25 } else { 0.0 };
            draw_cube(rot(0.0, 0.45, 0.0), vec3(0.4, 0.4, 0.4), None, skin);
            draw_cube(rot(-0.11, 0.45, 0.21), vec3(0.07, 0.07, 0.02), None, Color::from_rgba(25, 25, 25, 255));
            draw_cube(rot(0.11, 0.45, 0.21), vec3(0.07, 0.07, 0.02), None, Color::from_rgba(25, 25, 25, 255));
            // ปากยิ้ม (สไตล์ Roblox)
            draw_cube(rot(0.0, 0.37, 0.21), vec3(0.15, 0.03, 0.02), None, Color::from_rgba(25, 25, 25, 255));
            draw_cube(rot(0.0, 0.05, 0.0), vec3(0.5, 0.6, 0.25), None, shirt);
            draw_cube(rot(-0.33, 0.05, -swing), vec3(0.15, 0.6, 0.15), None, shirt);
            draw_cube(rot(0.33, 0.05, swing), vec3(0.15, 0.6, 0.15), None, shirt);
            draw_cube(rot(-0.15, -0.5, swing), vec3(0.2, 0.5, 0.2), None, pants);
            draw_cube(rot(0.15, -0.5, -swing), vec3(0.2, 0.5, 0.2), None, pants);
        }

        // ทุบ/วางบล็อก
        if let Some((hit, normal)) = raycast(cam_pos, forward, &world, REACH) {
            let hp_pos = vec3(hit.0 as f32 + 0.5, hit.1 as f32 + 0.5, hit.2 as f32 + 0.5);
            draw_cube_wires(hp_pos, vec3(1.01, 1.01, 1.01), BLACK);

            if mouse_grabbed {
                if is_mouse_button_pressed(MouseButton::Left) || gp_break {
                    let col = world.get(&hit).map(|b| b.color()).unwrap_or(GRAY);
                    world.remove(&hit);
                    play(&snd_break);
                    for _ in 0..12 {
                        particles.push(Particle {
                            pos: hp_pos,
                            vel: vec3(rand::gen_range(-3.0, 3.0), rand::gen_range(1.0, 5.0), rand::gen_range(-3.0, 3.0)),
                            life: 0.7,
                            color: col,
                        });
                    }
                }
                if is_mouse_button_pressed(MouseButton::Right) || gp_place {
                    let place = (hit.0 + normal.0, hit.1 + normal.1, hit.2 + normal.2);
                    let player_cell = (
                        cam_pos.x.floor() as i32,
                        cam_pos.y.floor() as i32,
                        cam_pos.z.floor() as i32,
                    );
                    if place != player_cell {
                        world.insert(place, selected);
                        play(&snd_place);
                    }
                }
            }
        } else if practice && mouse_grabbed && is_mouse_button_pressed(MouseButton::Left) {
            // ตีหุ่นฝึก (โหมดฝึก) เมื่อคลิกซ้ายไม่โดนบล็อก
            let to_dummy = dummy_pos - cam_pos;
            let d = to_dummy.length();
            if d < REACH && forward.dot(to_dummy / d.max(0.001)) > 0.7 {
                dummy_hits += 1;
                dummy_flash = 0.25;
                play(&snd_break);
            }
        }
        if !mouse_grabbed && is_mouse_button_pressed(MouseButton::Left) {
            mouse_grabbed = true;
            set_cursor_grab(true);
        }

        set_default_camera();

        // --- HUD ---
        draw_text("WASD: walk | Space: jump | LMB: hit/break | RMB: place | E: eat | V: view | T: home | P: auto | M: train | Esc: mouse | F10: quit", 20.0, 30.0, 22.0, WHITE);
        if autoplay {
            draw_texture(&thai_auto_tex, 20.0, 168.0, WHITE);
        }
        if practice {
            draw_text(&format!("PRACTICE MODE (no enemies) | Dummy hits: {} | M: back to survival", dummy_hits), 20.0, 140.0, 24.0, GREEN);
        }
        // แถบเลือด
        draw_rectangle(20.0, 62.0, 200.0, 16.0, Color::from_rgba(0, 0, 0, 180));
        draw_rectangle(20.0, 62.0, 200.0 * (hp / 100.0), 16.0, RED);
        draw_text(&format!("HP {:.0}", hp), 230.0, 62.0, 22.0, WHITE);
        // แถบหิว
        draw_rectangle(20.0, 86.0, 200.0, 16.0, Color::from_rgba(0, 0, 0, 180));
        draw_rectangle(20.0, 86.0, 200.0 * (hunger / 100.0), 16.0, Color::from_rgba(240, 170, 40, 255));
        draw_text(&format!("Hunger {:.0}", hunger), 230.0, 86.0, 22.0, WHITE);
        // แอปเปิ้ล + เวลา
        let day_label = if is_night { "NIGHT (wolves!)" } else { "DAY" };
        draw_text(&format!("Level {} | Apples: {} | Wolves: {} | {} | FPS {}", level, apples, wolves.len(), day_label, get_fps()), 20.0, 112.0, 22.0, WHITE);
        // เรดาร์หาหมาป่า (มุมขวาบน) — จุดแดง = หมาป่า, จุดเขียว = ตัวเรา
        let radar_cx = screen_width() - 70.0;
        let radar_cy = 70.0;
        draw_circle(radar_cx, radar_cy, 45.0, Color::from_rgba(0, 0, 0, 130));
        draw_circle_lines(radar_cx, radar_cy, 45.0, 2.0, Color::from_rgba(255, 255, 255, 120));
        draw_circle(radar_cx, radar_cy, 4.0, GREEN);
        // จุดเกิด (home) บนเรดาร์ — จุดน้ำเงิน ชี้ทางกลับบ้าน
        let home_rel = vec3(spawn_pos.x - cam_pos.x, 0.0, spawn_pos.z - cam_pos.z);
        if home_rel.length() > 3.0 {
            let home_angle = home_rel.x.atan2(home_rel.z);
            let home_rel_angle = home_angle - yaw;
            let hx = radar_cx - home_rel_angle.sin() * 35.0;
            let hy = radar_cy - home_rel_angle.cos() * 35.0;
            draw_circle(hx, hy, 5.0, BLUE);
        }
        for w in &wolves {
            let rel = w.pos - cam_pos;
            let angle = rel.x.atan2(rel.z);
            let rel_angle = angle - yaw;
            let dot_x = radar_cx - rel_angle.sin() * 35.0;
            let dot_y = radar_cy - rel_angle.cos() * 35.0;
            draw_circle(dot_x, dot_y, 5.0, RED);
        }
        if level_flash > 0.0 && !game_won {
            draw_text(&format!("YOU SURVIVED! -> Level {}!", level), screen_width() / 2.0 - 240.0, 150.0, 44.0, GOLD);
            draw_texture(&thai_win_tex, screen_width() / 2.0 - thai_win_tex.width() / 2.0, 205.0, WHITE);
        }
        if !game_won {
            let goal = if level >= 3 { "GOAL: DEFEAT THE BIG BOSS!" } else { "GOAL: SURVIVE THE NIGHT TO LEVEL UP" };
            draw_text(goal, screen_width() / 2.0 - 280.0, 260.0, 28.0, if level >= 3 { RED } else { YELLOW });
        }
        if game_won {
            draw_text("YOU WIN!", screen_width() / 2.0 - 120.0, 150.0, 64.0, GOLD);
            draw_text("You defeated the Big Boss!", screen_width() / 2.0 - 240.0, 230.0, 36.0, WHITE);
            draw_text("Press Enter to play again", screen_width() / 2.0 - 200.0, 300.0, 30.0, YELLOW);
        }
        if !mouse_grabbed && !settings_open {
            draw_text("Mouse released - click the window to grab again", 20.0, 140.0, 20.0, YELLOW);
        }
        if settings_open {
            let sw = screen_width();
            let sh = screen_height();
            let cx = sw / 2.0;
            let cy = sh / 2.0;
            let w = 540.0;
            let h = 430.0;
            let x0 = cx - w / 2.0;
            let y0 = cy - h / 2.0;
            draw_rectangle(x0, y0, w, h, Color::from_rgba(18, 22, 30, 225));
            draw_rectangle_lines(x0, y0, w, h, 2.0, GOLD);
            draw_text("SETTINGS (F2 or O = close)", cx - 205.0, y0 + 46.0, 32.0, GOLD);
            let rows = [
                format!("Mouse Sensitivity: {:.3}", mouse_sens),
                format!("Move Speed: {:.1}", move_speed),
                format!("Jump Power: {:.1}", jump_speed),
                format!("Day Length: {:.0} sec", day_length),
                format!("GPad Invert X: {}", if gpad_inv_x { "ON" } else { "OFF" }),
                format!("GPad Invert Y: {}", if gpad_inv_y { "ON" } else { "OFF" }),
            ];
            for (i, r) in rows.iter().enumerate() {
                let y = y0 + 108.0 + i as f32 * 46.0;
                if i == settings_sel {
                    draw_rectangle(x0 + 16.0, y - 14.0, w - 32.0, 38.0, Color::from_rgba(255, 255, 255, 26));
                }
                draw_text(r, x0 + 32.0, y, 26.0, if i == settings_sel { YELLOW } else { WHITE });
            }
            draw_text("Up/Down = select    Left/Right = adjust", cx - 205.0, y0 + h - 28.0, 20.0, Color::from_rgba(255, 255, 255, 190));
        }

        // Hotbar (เลือกบล็อก 1-8 สไตล์ Roblox)
        {
            let slot = 52.0;
            let gap = 8.0;
            let total = 8.0 * slot + 7.0 * gap;
            let x0 = screen_width() / 2.0 - total / 2.0;
            let y0 = screen_height() - 64.0;
            let blocks = [Block::Grass, Block::Dirt, Block::Stone, Block::Wood, Block::Sand, Block::Glass, Block::Brick, Block::Snow];
            for (i, b) in blocks.iter().enumerate() {
                let x = x0 + i as f32 * (slot + gap);
                let sel = *b == selected;
                draw_rectangle(x, y0, slot, slot, if sel { Color::from_rgba(255, 255, 255, 90) } else { Color::from_rgba(0, 0, 0, 130) });
                draw_rectangle_lines(x, y0, slot, slot, 2.0, if sel { WHITE } else { Color::from_rgba(255, 255, 255, 60) });
                draw_rectangle(x + 9.0, y0 + 9.0, slot - 18.0, slot - 18.0, b.color());
            }
        }

        // --- สถานะเกมแพด (มุมขวาล่าง ใช้ทดสอบจอย) ---
        {
            let gx = screen_width() - 560.0;
            let gy = screen_height() - 58.0;
            if gp_connected {
                draw_text(
                    &format!("GAMEPAD: ON  L(x{:.2},y{:.2})  R(x{:.2},y{:.2})", gp_move.x, gp_move.y, gp_look.x, gp_look.y),
                    gx, gy, 20.0, GREEN,
                );
                let mut btn = String::new();
                if gp_jump { btn.push_str("A "); }
                if gp_break { btn.push_str("X "); }
                if gp_place { btn.push_str("B "); }
                if gp_view { btn.push_str("Y "); }
                if gp_sel_next { btn.push_str("R "); }
                if gp_sel_prev { btn.push_str("L "); }
                if gp_settings { btn.push_str("START "); }
                if !btn.is_empty() {
                    draw_text(&format!("button: {}", btn.trim()), gx, gy + 24.0, 20.0, YELLOW);
                }
            } else {
                draw_text("GAMEPAD: none (plug controller - auto detect)", gx, gy, 20.0, Color::from_rgba(255, 255, 255, 150));
            }
        }

        last_auto_pos = cam_pos;
        was_auto_moving = auto_dir != Vec3::ZERO;

        if first_frame {
            first_frame = false;
            eprintln!("[game] first frame rendered +{:.1}s", t0.elapsed().as_secs_f32());
        }

        next_frame().await
    }
}
