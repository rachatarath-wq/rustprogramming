use macroquad::prelude::*;
use macroquad::audio::*;
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
const SPEED: f32 = 6.0;
const GRAVITY: f32 = 22.0;
const JUMP_SPEED: f32 = 8.0;
const SAVE_FILE: &str = "world_v2.txt";
// ความยาว 1 รอบกลางวัน+กลางคืน (วินาที)
const DAY_LENGTH: f32 = 90.0;

// สร้างบ้าน (พื้น + ผนัง + ช่องประตู + หลังคา)
fn build_house(w: &mut HashMap<BlockPos, Block>, x0: i32, z0: i32, wall: Block, roof: Block) {
    let x1 = x0 + 4;
    let z1 = z0 + 4;
    for x in x0..=x1 {
        for z in z0..=z1 {
            w.insert((x, 0, z), Block::Stone);   // พื้นบ้าน
            for y in 1..=2 {
                if x == x0 || x == x1 || z == z0 || z == z1 {
                    // เว้นช่องประตู (ผนังหน้า ตรงกลาง)
                    if z == z0 && x == x0 + 2 && y == 1 {
                        continue;
                    }
                    w.insert((x, y, z), wall);
                }
            }
        }
    }
    for x in (x0 - 1)..=(x1 + 1) {
        for z in (z0 - 1)..=(z1 + 1) {
            w.insert((x, 3, z), roof);   // หลังคา
        }
    }
}

fn build_world() -> HashMap<BlockPos, Block> {
    let mut w = HashMap::new();
    // 1. พื้นหญ้าใหญ่ 40x40
    for x in -20..20 {
        for z in -20..20 {
            w.insert((x, 0, z), Block::Grass);
        }
    }
    // 2. ถนนหินตัดกัน (กว้าง 3)
    for x in -20..20 {
        for dz in -1..=1 {
            w.insert((x, 0, dz), Block::Stone);
        }
    }
    for z in -20..20 {
        for dx in -1..=1 {
            w.insert((dx, 0, z), Block::Stone);
        }
    }
    // 3. ลานกลางหิน 9x9 (จุดเกิด)
    for x in -4..=4 {
        for z in -4..=4 {
            w.insert((x, 0, z), Block::Stone);
        }
    }
    // 4. ต้นไม้ 8 ต้น (ลำต้นไม้ + ใบ)
    let trees = [(-12, -12), (12, -12), (-12, 12), (12, 12), (8, -16), (-8, 16), (16, 6), (-16, -6)];
    for (tx, tz) in trees {
        for y in 1..=4 {
            w.insert((tx, y, tz), Block::Wood);
        }
        for dx in -1..=1 {
            for dz in -1..=1 {
                w.insert((tx + dx, 5, tz + dz), Block::Grass);
            }
        }
    }
    // 5. บ้าน 4 หลัง (มุมเมือง)
    build_house(&mut w, -10, -6, Block::Wood, Block::Brick);
    build_house(&mut w, 6, -12, Block::Wood, Block::Snow);
    build_house(&mut w, -12, 8, Block::Stone, Block::Wood);
    build_house(&mut w, 8, 8, Block::Stone, Block::Brick);
    // 6. เนินเขา (dirt + หญ้ายอด)
    let hills = [(-17, -17, 3), (17, -17, 2), (17, 17, 3), (-17, 17, 2)];
    for (hx, hz, hh) in hills {
        for y in 1..=hh {
            let b = if y == hh { Block::Grass } else { Block::Dirt };
            for (dx, dz) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                w.insert((hx + dx, y, hz + dz), b);
            }
        }
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

#[macroquad::main("Mini Roblox")]
async fn main() {
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
    println!("เสียงโหลด {}/9{}", sound_n, if sound_n == 0 { " — ไม่มี sound device" } else { "" });

    // ข้อความไทย (render เป็น PNG ล่วงหน้าด้วย HarfBuzz shaping แล้ว)
    let thai_win_tex = Texture2D::from_file_with_format(include_bytes!("../assets/thai_win.png"), None);
    let thai_auto_tex = Texture2D::from_file_with_format(include_bytes!("../assets/thai_auto.png"), None);

    // --- โลก ---
    let mut world = load_world(SAVE_FILE);
    if world.is_empty() {
        world = build_world();
    }
    println!("โลก {} บล็อก (บ้าน/ต้นไม้อยู่รอบเมือง)", world.len());

    // --- ผู้เล่น ---
    let mut cam_pos = vec3(4.0, 6.0, 4.0);
    let mut vel = vec3(0.0, 0.0, 0.0);
    let mut grounded = false;
    let mut third_person = true;
    let mut mouse_grabbed = true;
    let mut yaw: f32 = -135.0_f32.to_radians();
    let mut pitch: f32 = -25.0_f32.to_radians();
    let mut selected = Block::Stone;

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
    let dummy_pos = vec3(-4.0, 1.9, -3.0);
    // ที่กำบัง (กำแพง 2 ชั้น รอบพื้นที่ 2x2 ตรงจุดเกิด) สร้างตอนเย็น
    let mut shelter_built = false;
    let mut shelter_idx: usize = 0;
    let mut shelter_timer: f32 = 0.0;
    let mut shelter_blocks: Vec<BlockPos> = Vec::new();
    for sy in 1..=2 {
        for sx in 3..=6 {
            for sz in 3..=6 {
                if sx >= 4 && sx <= 5 && sz >= 4 && sz <= 5 {
                    continue;
                }
                shelter_blocks.push((sx, sy, sz));
            }
        }
    }

    let spawn_points: [(f32, f32, f32); 4] =
        [(-15.0, 2.0, -15.0), (15.0, 2.0, 15.0), (-15.0, 2.0, 15.0), (15.0, 2.0, -15.0)];
    let mut apple_items: Vec<Apple> = vec![
        Apple { pos: vec3(5.0, 1.2, -6.0), taken: false },
        Apple { pos: vec3(-6.0, 1.2, 5.0), taken: false },
        Apple { pos: vec3(10.0, 1.2, -10.0), taken: false },
        Apple { pos: vec3(-10.0, 1.2, 10.0), taken: false },
        Apple { pos: vec3(14.0, 1.2, -3.0), taken: false },
        Apple { pos: vec3(-14.0, 1.2, 3.0), taken: false },
    ];

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

    loop {
        let dt = get_frame_time();

        // --- วงจรกลางวัน/กลางคืน ---
        time_of_day = (time_of_day + dt / DAY_LENGTH) % 1.0;
        let day_factor = (time_of_day * std::f32::consts::TAU).cos(); // 1=เที่ยง, -1=เที่ยงคืน
        let nightness = if practice { 0.0 } else { (-day_factor).clamp(0.0, 1.0) }; // 0=กลางวัน, 1=กลางคืน
        let is_night = if practice { false } else { day_factor < -0.2 };
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

        // --- มองรอบด้วยเมาส์ (เฉพาะตอนเล่นเอง) ---
        if mouse_grabbed && !autoplay {
            let mouse_delta = mouse_delta_position();
            yaw -= mouse_delta.x * 0.008;
            pitch = (pitch - mouse_delta.y * 0.008).clamp(-1.5, 1.5);
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
                let home = vec3(4.5, cam_pos.y, 4.5);
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
                    target = Some(vec3(4.5, cam_pos.y, 4.5));
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
            horiz = auto_dir * SPEED;
        } else {
            if is_key_down(KeyCode::W) { horiz += forward; }
            if is_key_down(KeyCode::S) { horiz -= forward; }
            if is_key_down(KeyCode::D) { horiz += right; }
            if is_key_down(KeyCode::A) { horiz -= right; }
            if horiz != Vec3::ZERO {
                horiz = horiz.normalize() * SPEED;
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
        if grounded && (is_key_pressed(KeyCode::Space) || (autoplay && stuck_timer > 0.4)) {
            vel.y = JUMP_SPEED;
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
        if is_key_pressed(KeyCode::R) {
            save_world(&world, SAVE_FILE);
        }
        if is_key_pressed(KeyCode::T) {
            cam_pos = vec3(4.0, 6.0, 4.0);
            vel = Vec3::ZERO;
            grounded = false;
        }
        if is_key_pressed(KeyCode::V) {
            third_person = !third_person;
        }
        if is_key_pressed(KeyCode::Escape) {
            mouse_grabbed = !mouse_grabbed;
            set_cursor_grab(mouse_grabbed);
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
        if mouse_grabbed && is_mouse_button_pressed(MouseButton::Left) {
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
            cam_pos = vec3(4.0, 6.0, 4.0);
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

        draw_grid(20, 1.0, Color::from_rgba(0, 0, 0, 60), Color::from_rgba(200, 200, 200, 40));
        for (pos, block) in &world {
            let p = vec3(pos.0 as f32 + 0.5, pos.1 as f32 + 0.5, pos.2 as f32 + 0.5);
            let dist = (p - cam_pos).length();
            let fog = ((dist - 20.0) / 20.0).clamp(0.0, 0.85);
            let c = lerp_color(block.color(), sky, fog);
            draw_cube(p, vec3(1.0, 1.0, 1.0), None, c);
        }
        // อนุภาคทุบบล็อก
        for p in &particles {
            let s = 0.14 * (p.life / 0.7).max(0.05);
            draw_cube(p.pos, vec3(s, s, s), None, p.color);
        }
        draw_cube_wires(vec3(4.0, 3.0, 4.0), vec3(0.4, 4.0, 0.4), GOLD);

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
                if is_mouse_button_pressed(MouseButton::Left) {
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
                if is_mouse_button_pressed(MouseButton::Right) {
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
        for w in &wolves {
            let rel = w.pos - cam_pos;
            let angle = rel.x.atan2(rel.z);
            let rel_angle = angle - yaw;
            let dot_x = radar_cx - rel_angle.sin() * 35.0;
            let dot_y = radar_cy - rel_angle.cos() * 35.0;
            draw_circle(dot_x, dot_y, 5.0, RED);
        }
        if level_flash > 0.0 && !game_won {
            draw_text(&format!("YOU SURVIVED! -> Level {}!", level), screen_width() / 2.0 - 250.0, 180.0, 48.0, GOLD);
            draw_texture(&thai_win_tex, screen_width() / 2.0 - thai_win_tex.width() / 2.0, 250.0, WHITE);
        }
        if !game_won {
            let goal = if level >= 3 { "GOAL: DEFEAT THE BIG BOSS!" } else { "GOAL: SURVIVE THE NIGHT TO LEVEL UP" };
            draw_text(goal, screen_width() / 2.0 - 300.0, 300.0, 30.0, if level >= 3 { RED } else { YELLOW });
        }
        if game_won {
            draw_text("YOU DEFEATED THE BOSS! YOU WIN!", screen_width() / 2.0 - 340.0, 190.0, 48.0, GOLD);
        }
        if !mouse_grabbed {
            draw_text("Mouse released - click the window to grab again", 20.0, 140.0, 20.0, YELLOW);
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

        last_auto_pos = cam_pos;
        was_auto_moving = auto_dir != Vec3::ZERO;

        next_frame().await
    }
}
