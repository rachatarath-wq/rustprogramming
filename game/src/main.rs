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
const SAVE_FILE: &str = "world.txt";
// ความยาว 1 รอบกลางวัน+กลางคืน (วินาที)
const DAY_LENGTH: f32 = 90.0;
const WOLF_SPEED: f32 = 3.5;
const MAX_WOLVES: usize = 4;

fn build_world() -> HashMap<BlockPos, Block> {
    let mut w = HashMap::new();
    for x in -8..8 {
        for z in -8..8 {
            w.insert((x, 0, z), Block::Grass);
        }
    }
    for x in 0..3 {
        for z in 0..3 {
            w.insert((x, 1, z), Block::Stone);
        }
    }
    for y in 1..4 {
        w.insert((5, y, 5), Block::Wood);
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

    // --- โลก ---
    let mut world = load_world(SAVE_FILE);
    if world.is_empty() {
        world = build_world();
    }

    // --- ผู้เล่น ---
    let mut cam_pos = vec3(4.0, 6.0, 4.0);
    let mut vel = vec3(0.0, 0.0, 0.0);
    let mut grounded = false;
    let mut third_person = true;
    let mut mouse_grabbed = true;
    let mut yaw: f32 = -135.0_f32.to_radians();
    let mut pitch: f32 = -25.0_f32.to_radians();
    let mut selected = Block::Grass;

    // --- สถานะ survival ---
    let mut time_of_day: f32 = 0.3;
    let mut hp: f32 = 100.0;
    let mut hunger: f32 = 100.0;
    let mut apples: i32 = 0;
    let mut wolves: Vec<Wolf> = Vec::new();
    let mut wolf_spawn_timer: f32 = 0.0;
    let mut hurt_cd: f32 = 0.0;
    let mut step_timer: f32 = 0.0;

    let spawn_points: [(f32, f32, f32); 4] =
        [(-7.0, 2.0, -7.0), (7.0, 2.0, 7.0), (-7.0, 2.0, 7.0), (7.0, 2.0, -7.0)];
    let mut apple_items: Vec<Apple> = vec![
        Apple { pos: vec3(2.0, 1.2, -3.0), taken: false },
        Apple { pos: vec3(-3.0, 1.2, 2.0), taken: false },
        Apple { pos: vec3(6.0, 1.2, -5.0), taken: false },
        Apple { pos: vec3(-5.0, 1.2, 6.0), taken: false },
    ];

    set_cursor_grab(true);

    loop {
        let dt = get_frame_time();

        // --- วงจรกลางวัน/กลางคืน ---
        time_of_day = (time_of_day + dt / DAY_LENGTH) % 1.0;
        let day_factor = (time_of_day * std::f32::consts::TAU).cos(); // 1=เที่ยง, -1=เที่ยงคืน
        let nightness = (-day_factor).clamp(0.0, 1.0); // 0=กลางวัน, 1=กลางคืน
        let is_night = day_factor < -0.2;

        // --- มองรอบด้วยเมาส์ ---
        if mouse_grabbed {
            let mouse_delta = mouse_delta_position();
            yaw -= mouse_delta.x * 0.003;
            pitch = (pitch - mouse_delta.y * 0.003).clamp(-1.5, 1.5);
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
        if is_key_down(KeyCode::W) { horiz += forward; }
        if is_key_down(KeyCode::S) { horiz -= forward; }
        if is_key_down(KeyCode::D) { horiz += right; }
        if is_key_down(KeyCode::A) { horiz -= right; }
        if horiz != Vec3::ZERO {
            horiz = horiz.normalize() * SPEED;
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
        if is_key_pressed(KeyCode::Space) && grounded {
            vel.y = JUMP_SPEED;
            grounded = false;
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

        // --- ความหิว ---
        hunger = (hunger - dt * 0.6).max(0.0);
        if hunger <= 0.0 {
            hp -= dt * 2.0;
        }
        // กินแอปเปิ้ล (E)
        if is_key_pressed(KeyCode::E) && apples > 0 {
            apples -= 1;
            hunger = (hunger + 35.0).min(100.0);
            play(&snd_eat);
        }

        // --- หมาป่าเกิดตอนกลางคืน ---
        if is_night && wolves.len() < MAX_WOLVES {
            wolf_spawn_timer -= dt;
            if wolf_spawn_timer <= 0.0 {
                let sp = spawn_points[wolves.len()];
                wolves.push(Wolf {
                    pos: vec3(sp.0, sp.1, sp.2),
                    vel: Vec3::ZERO,
                    hp: 3,
                    grounded: false,
                });
                wolf_spawn_timer = 3.0;
                play(&snd_growl);
            }
        }

        // --- หมาป่า AI: ไล่ + กัด ---
        for w in &mut wolves {
            let to_player = cam_pos - w.pos;
            let dist = to_player.length();
            let dir = to_player / dist.max(0.001);
            w.vel.x = dir.x * WOLF_SPEED;
            w.vel.z = dir.z * WOLF_SPEED;
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
                    hp -= 12.0;
                    hurt_cd = 1.0;
                    play(&snd_hurt);
                }
            }
        }

        // --- ต่อสู้: คลิกซ้ายตีหมาป่า ---
        if mouse_grabbed && is_mouse_button_pressed(MouseButton::Left) {
            let mut hit_wolf = false;
            for w in &mut wolves {
                let to_wolf = w.pos - cam_pos;
                let d = to_wolf.length();
                if d < REACH && forward.dot(to_wolf / d.max(0.001)) > 0.7 {
                    w.hp -= 1;
                    hit_wolf = true;
                }
            }
            if hit_wolf {
                play(&snd_break);
            }
        }
        wolves.retain(|w| w.hp > 0);

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
            for a in &mut apple_items {
                a.taken = false;
            }
        }

        // --- วาด ---
        let sky = lerp_color(Color::from_rgba(20, 25, 50, 255), SKYBLUE, 1.0 - nightness);
        clear_background(sky);

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

        draw_grid(20, 1.0, Color::from_rgba(0, 0, 0, 60), Color::from_rgba(200, 200, 200, 40));
        for (pos, block) in &world {
            let p = vec3(pos.0 as f32 + 0.5, pos.1 as f32 + 0.5, pos.2 as f32 + 0.5);
            draw_cube(p, vec3(1.0, 1.0, 1.0), None, block.color());
        }
        draw_cube_wires(vec3(4.0, 3.0, 4.0), vec3(0.4, 4.0, 0.4), GOLD);

        // แอปเปิ้ลบนพื้น
        for a in &apple_items {
            if !a.taken {
                draw_cube(a.pos, vec3(0.25, 0.25, 0.25), None, Color::from_rgba(220, 60, 50, 255));
            }
        }

        // หมาป่า
        for w in &wolves {
            let to_player = (cam_pos - w.pos).normalize();
            let body = w.pos + vec3(0.0, 0.2, 0.0);
            draw_cube(body, vec3(0.6, 0.5, 0.9), None, Color::from_rgba(90, 90, 95, 255));
            let head = body + to_player * 0.5 + vec3(0.0, 0.2, 0.0);
            draw_cube(head, vec3(0.4, 0.4, 0.4), None, Color::from_rgba(70, 70, 75, 255));
            // ตาสีแดง (ด้านหน้า = ไปทางผู้เล่น)
            let eye_r = right * 0.1;
            draw_cube(head + to_player * 0.2 - eye_r, vec3(0.08, 0.08, 0.02), None, RED);
            draw_cube(head + to_player * 0.2 + eye_r, vec3(0.08, 0.08, 0.02), None, RED);
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
                    world.remove(&hit);
                    play(&snd_break);
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
        }
        if !mouse_grabbed && is_mouse_button_pressed(MouseButton::Left) {
            mouse_grabbed = true;
            set_cursor_grab(true);
        }

        set_default_camera();

        // --- HUD ---
        draw_text("WASD: walk | Space: jump | LMB: hit/break | RMB: place | E: eat | V: view | T: home | Esc: mouse | F10: quit", 20.0, 30.0, 22.0, WHITE);
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
        draw_text(&format!("Apples: {} | {} | FPS {}", apples, day_label, get_fps()), 20.0, 112.0, 22.0, WHITE);
        if !mouse_grabbed {
            draw_text("Mouse released - click the window to grab again", 20.0, 140.0, 20.0, YELLOW);
        }

        next_frame().await
    }
}
