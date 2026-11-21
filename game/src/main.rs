use macroquad::prelude::*;
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
    // สีของบล็อกแต่ละชนิด
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
    // ชื่อสำหรับแสดงผล และใช้บันทึก/โหลด
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

// ระยะเอื้อมมือ (กี่บล็อก)
const REACH: f32 = 8.0;
// ความเร็วเดิน
const SPEED: f32 = 6.0;
// แรงโน้มถ่วง (บล็อก/วินาที²)
const GRAVITY: f32 = 22.0;
// แรงกระโดด (บล็อก/วินาที)
const JUMP_SPEED: f32 = 8.0;
// ไฟล์สำหรับบันทึกโลก
const SAVE_FILE: &str = "world.txt";

// สร้างโลกเริ่มต้น: พื้นหญ้า + แท่นหิน + เสาไม้
fn build_world() -> HashMap<BlockPos, Block> {
    let mut w = HashMap::new();
    for x in -8..8 {
        for z in -8..8 {
            w.insert((x, 0, z), Block::Grass);
        }
    }
    // แท่นหิน 3x3 หนึ่งชั้น
    for x in 0..3 {
        for z in 0..3 {
            w.insert((x, 1, z), Block::Stone);
        }
    }
    // เสาไม้สูง 3 บล็อก
    for y in 1..4 {
        w.insert((5, y, 5), Block::Wood);
    }
    w
}

// บันทึกโลกเป็นไฟล์ข้อความ บรรทัดละ "x y z ชนิด"
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

// โหลดโลกจากไฟล์ (ถ้าไม่มีไฟล์ คืน HashMap ว่าง)
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

// เดินทางตามแนวรังสีผ่านก้อน voxel (อัลกอริทึม Amanatides & Woo)
// คืนค่า (พิกัดบล็อกที่ชน, เวกเตอร์ปกติของด้านที่ชน) หรือ None ถ้าไม่ชนอะไร
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
        // เดินไป voxel ถัดไปตามแกนที่ t_max น้อยที่สุด
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

// ตรวจว่าตัวผู้เล่น (AABB) ทับบล็อกใดในโลกหรือไม่
fn player_collides(pos: Vec3, world: &HashMap<BlockPos, Block>) -> bool {
    let min = pos - vec3(0.3, 0.85, 0.3);
    let max = pos + vec3(0.3, 0.85, 0.3);
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

#[macroquad::main("Mini Roblox")]
async fn main() {
    // โหลดโลกจากไฟล์ก่อน ถ้าไม่มีก็สร้างโลกเริ่มต้น
    let mut world = load_world(SAVE_FILE);
    if world.is_empty() {
        world = build_world();
    }

    let mut cam_pos = vec3(4.0, 6.0, 4.0);
    let mut vel = vec3(0.0, 0.0, 0.0);
    let mut grounded = false;
    let mut third_person = false;
    // หันกล้องมาที่จุดกำเนิด
    let mut yaw: f32 = -135.0_f32.to_radians();
    let mut pitch: f32 = -25.0_f32.to_radians();
    let mut selected = Block::Grass;

    set_cursor_grab(true);

    loop {
        let dt = get_frame_time();

        // --- มองรอบด้วยเมาส์ ---
        let mouse_delta = mouse_delta_position();
        yaw -= mouse_delta.x * 0.003;
        pitch = (pitch - mouse_delta.y * 0.003).clamp(-1.5, 1.5);

        // เวกเตอร์ทิศทางที่มอง
        let forward = vec3(
            pitch.cos() * yaw.sin(),
            pitch.sin(),
            pitch.cos() * yaw.cos(),
        )
        .normalize();
        let right = vec3(yaw.cos(), 0.0, -yaw.sin());

        // --- เดินด้วย WASD (แนวนอน) ---
        let mut horiz = Vec3::ZERO;
        if is_key_down(KeyCode::W) { horiz += forward; }
        if is_key_down(KeyCode::S) { horiz -= forward; }
        if is_key_down(KeyCode::D) { horiz += right; }
        if is_key_down(KeyCode::A) { horiz -= right; }
        if horiz != Vec3::ZERO {
            horiz = horiz.normalize() * SPEED;
        }
        vel.x = horiz.x;
        vel.z = horiz.z;

        // แรงโน้มถ่วง
        vel.y -= GRAVITY * dt;
        vel.y = vel.y.max(-30.0);

        // กระโดด (เฉพาะตอนยืนบนพื้น)
        if is_key_pressed(KeyCode::Space) && grounded {
            vel.y = JUMP_SPEED;
            grounded = false;
        }

        // ขยับทีละแกน + กันชนบล็อก
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

        // --- เลือกชนิดบล็อกด้วยปุ่ม 1-4 ---
        if is_key_pressed(KeyCode::Key1) { selected = Block::Grass; }
        if is_key_pressed(KeyCode::Key2) { selected = Block::Dirt; }
        if is_key_pressed(KeyCode::Key3) { selected = Block::Stone; }
        if is_key_pressed(KeyCode::Key4) { selected = Block::Wood; }
        if is_key_pressed(KeyCode::Key5) { selected = Block::Sand; }
        if is_key_pressed(KeyCode::Key6) { selected = Block::Glass; }
        if is_key_pressed(KeyCode::Key7) { selected = Block::Brick; }
        if is_key_pressed(KeyCode::Key8) { selected = Block::Snow; }
        // บันทึกโลกด้วยปุ่ม R
        if is_key_pressed(KeyCode::R) {
            save_world(&world, SAVE_FILE);
        }
        // ย้อนกลับจุดเกิดด้วยปุ่ม T
        if is_key_pressed(KeyCode::T) {
            cam_pos = vec3(4.0, 6.0, 4.0);
            vel = Vec3::ZERO;
            grounded = false;
        }
        // สลับมุมมองบุคคลที่ 1 / 3 ด้วยปุ่ม V
        if is_key_pressed(KeyCode::V) {
            third_person = !third_person;
        }
        // ออกจากโปรแกรมด้วยปุ่ม Esc
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        clear_background(SKYBLUE);

        // กล้อง: บุคคลที่ 1 (ตาตัวเอง) หรือบุคคลที่ 3 (มองจากด้านหลัง)
        let (cam_position, cam_target) = if third_person {
            (cam_pos - forward * 4.0 + vec3(0.0, 1.2, 0.0), cam_pos + vec3(0.0, 0.2, 0.0))
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

        // วาดพื้นอ้างอิง + บล็อกทั้งหมด
        draw_grid(20, 1.0, Color::from_rgba(0, 0, 0, 60), Color::from_rgba(200, 200, 200, 40));
        for (pos, block) in &world {
            let p = vec3(pos.0 as f32 + 0.5, pos.1 as f32 + 0.5, pos.2 as f32 + 0.5);
            draw_cube(p, vec3(1.0, 1.0, 1.0), None, block.color());
        }
        // ไฟสัญญาณจุดเกิด (beacon สีทอง มองเห็นแต่ไกล)
        draw_cube_wires(vec3(4.0, 3.0, 4.0), vec3(0.4, 4.0, 0.4), GOLD);

        // ตัวผู้เล่น (เห็นตอนมุมมองบุคคลที่ 3) — ลำตัว + หัว
        if third_person {
            let body = cam_pos - vec3(0.0, 0.4, 0.0);
            draw_cube(body, vec3(0.5, 0.8, 0.25), None, Color::from_rgba(40, 90, 200, 255));
            draw_cube(body + vec3(0.0, 0.65, 0.0), vec3(0.4, 0.4, 0.4), None, Color::from_rgba(230, 190, 150, 255));
        }

        // --- เล็งบล็อก: ทุบ (คลิกซ้าย) / วาง (คลิกขวา) ---
        if let Some((hit, normal)) = raycast(cam_pos, forward, &world, REACH) {
            let hp = vec3(hit.0 as f32 + 0.5, hit.1 as f32 + 0.5, hit.2 as f32 + 0.5);
            draw_cube_wires(hp, vec3(1.01, 1.01, 1.01), BLACK);

            if is_mouse_button_pressed(MouseButton::Left) {
                world.remove(&hit);
            }
            if is_mouse_button_pressed(MouseButton::Right) {
                let place = (hit.0 + normal.0, hit.1 + normal.1, hit.2 + normal.2);
                // ไม่วางบล็อกทับตัวผู้เล่น
                let player_cell = (
                    cam_pos.x.floor() as i32,
                    cam_pos.y.floor() as i32,
                    cam_pos.z.floor() as i32,
                );
                if place != player_cell {
                    world.insert(place, selected);
                }
            }
        }

        set_default_camera();

        // --- HUD ---
        draw_text("Mini Roblox - LMB break, RMB place, WASD move, Space jump, V view, R save, T respawn, Esc exit", 20.0, 30.0, 26.0, WHITE);
        draw_text(&format!("Block (1-8): {} | Blocks: {} | FPS {}", selected.name(), world.len(), get_fps()), 20.0, 62.0, 26.0, WHITE);

        next_frame().await
    }
}
