// บทที่ 32 — ต่อสู้: คลิกซ้ายตีหมาป่า + เลือดหมาป่า + หมาป่าตาย
use macroquad::prelude::*;
use std::collections::HashMap;

type BlockPos = (i32, i32, i32);

#[derive(Clone, Copy, PartialEq)]
enum Block { Grass, Dirt, Stone, Wood, Sand, Glass, Brick, Snow }

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
}

const GRAVITY: f32 = 22.0;
const JUMP_SPEED: f32 = 8.0;
const SPEED: f32 = 6.0;
const HOME: (f32, f32, f32) = (4.0, 6.0, 4.0);
const DAY_LENGTH: f32 = 90.0;
const WOLF_SPEED: f32 = 3.5;
const MAX_WOLVES: usize = 4;
const REACH: f32 = 8.0;

struct Wolf {
    pos: Vec3,
    vel: Vec3,
    hp: i32,
    grounded: bool,
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

#[macroquad::main("My 3D Game")]
async fn main() {
    let mut world = build_world();
    let mut selected = Block::Stone;

    let mut pos = vec3(HOME.0, HOME.1, HOME.2);
    let mut vel = Vec3::ZERO;
    let mut grounded = false;
    let mut yaw = 0.0f32;
    let mut pitch = 0.0f32;
    let mut third_person = false;
    let mut mouse_grabbed = true;
    set_cursor_grab(true);

    let mut time_of_day: f32 = 0.0;
    let mut wolves: Vec<Wolf> = Vec::new();
    let mut wolf_spawn_timer: f32 = 0.0;
    let spawn_points: [(f32, f32, f32); 4] = [
        (-7.0, 2.0, -7.0),
        (7.0, 2.0, 7.0),
        (-7.0, 2.0, 7.0),
        (7.0, 2.0, -7.0),
    ];

    loop {
        let dt = get_frame_time();

        time_of_day = (time_of_day + dt / DAY_LENGTH) % 1.0;
        let day_factor = (time_of_day * std::f32::consts::TAU).cos();
        let nightness = (-day_factor).clamp(0.0, 1.0);
        let is_night = nightness > 0.5;

        if mouse_grabbed {
            let mouse_delta = mouse_delta_position();
            yaw -= mouse_delta.x * 0.008;
            pitch = (pitch - mouse_delta.y * 0.008).clamp(-1.5, 1.5);
        }

        let forward = vec3(
            pitch.cos() * yaw.sin(),
            pitch.sin(),
            pitch.cos() * yaw.cos(),
        )
        .normalize();
        let right = vec3(-yaw.cos(), 0.0, yaw.sin());

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

        vel.y -= GRAVITY * dt;
        vel.y = vel.y.max(-30.0);
        if grounded && is_key_pressed(KeyCode::Space) {
            vel.y = JUMP_SPEED;
            grounded = false;
        }

        let nx = pos.x + vel.x * dt;
        if !player_collides(vec3(nx, pos.y, pos.z), &world) {
            pos.x = nx;
        } else {
            vel.x = 0.0;
        }
        let ny = pos.y + vel.y * dt;
        if !player_collides(vec3(pos.x, ny, pos.z), &world) {
            pos.y = ny;
            grounded = false;
        } else {
            if vel.y < 0.0 { grounded = true; }
            vel.y = 0.0;
        }
        let nz = pos.z + vel.z * dt;
        if !player_collides(vec3(pos.x, pos.y, nz), &world) {
            pos.z = nz;
        } else {
            vel.z = 0.0;
        }

        if is_key_pressed(KeyCode::T) {
            pos = vec3(HOME.0, HOME.1, HOME.2);
            vel = Vec3::ZERO;
            grounded = false;
        }
        if is_key_pressed(KeyCode::Escape) {
            mouse_grabbed = !mouse_grabbed;
            set_cursor_grab(mouse_grabbed);
        }
        if !mouse_grabbed && is_mouse_button_pressed(MouseButton::Left) {
            mouse_grabbed = true;
            set_cursor_grab(true);
        }
        if is_key_pressed(KeyCode::V) {
            third_person = !third_person;
        }
        if is_key_pressed(KeyCode::Key1) { selected = Block::Grass; }
        if is_key_pressed(KeyCode::Key2) { selected = Block::Dirt; }
        if is_key_pressed(KeyCode::Key3) { selected = Block::Stone; }
        if is_key_pressed(KeyCode::Key4) { selected = Block::Wood; }
        if is_key_pressed(KeyCode::Key5) { selected = Block::Sand; }
        if is_key_pressed(KeyCode::Key6) { selected = Block::Glass; }
        if is_key_pressed(KeyCode::Key7) { selected = Block::Brick; }
        if is_key_pressed(KeyCode::Key8) { selected = Block::Snow; }

        // หมาป่าเกิดตอนกลางคืน
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
            }
        }

        // หมาป่า AI: ไล่ผู้เล่น
        for w in &mut wolves {
            let to_player = pos - w.pos;
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
        }

        // ต่อสู้: คลิกซ้ายตีหมาป่าที่อยู่ตรงหน้า
        if mouse_grabbed && is_mouse_button_pressed(MouseButton::Left) {
            for w in &mut wolves {
                let to_wolf = w.pos - pos;
                let d = to_wolf.length();
                if d < REACH && forward.dot(to_wolf / d.max(0.001)) > 0.7 {
                    w.hp -= 1;
                }
            }
        }
        // กำจัดหมาป่าที่ตายแล้ว (hp <= 0)
        wolves.retain(|w| w.hp > 0);

        let horiz_fwd = vec3(yaw.sin(), 0.0, yaw.cos()).normalize();
        let (cam_pos, cam_target) = if third_person {
            (pos - horiz_fwd * 4.0 + vec3(0.0, 0.6, 0.0), pos + vec3(0.0, 0.4, 0.0))
        } else {
            (pos, pos + forward)
        };
        let camera = Camera3D {
            position: cam_pos,
            target: cam_target,
            up: vec3(0.0, 1.0, 0.0),
            fovy: 45.0,
            ..Default::default()
        };
        set_camera(&camera);

        let sky = lerp_color(Color::from_rgba(20, 25, 50, 255), SKYBLUE, 1.0 - nightness);
        clear_background(sky);

        for (p, block) in &world {
            let center = vec3(p.0 as f32 + 0.5, p.1 as f32 + 0.5, p.2 as f32 + 0.5);
            draw_cube(center, vec3(1.0, 1.0, 1.0), None, block.color());
        }

        draw_cube_wires(vec3(HOME.0, HOME.1 - 2.5, HOME.2), vec3(0.4, 4.0, 0.4), GOLD);

        for w in &wolves {
            draw_cube(vec3(w.pos.x, w.pos.y + 1.3, w.pos.z), vec3(0.3, 1.6, 0.3), None, Color::from_rgba(255, 45, 45, 110));
            let to_player = (pos - w.pos).normalize();
            let body = w.pos + vec3(0.0, 0.2, 0.0);
            draw_cube(body, vec3(0.6, 0.5, 0.9), None, Color::from_rgba(90, 90, 95, 255));
            let head = body + to_player * 0.5 + vec3(0.0, 0.2, 0.0);
            draw_cube(head, vec3(0.4, 0.4, 0.4), None, Color::from_rgba(70, 70, 75, 255));
            let eye_r = right * 0.1;
            draw_cube(head + to_player * 0.2 - eye_r, vec3(0.08, 0.08, 0.02), None, RED);
            draw_cube(head + to_player * 0.2 + eye_r, vec3(0.08, 0.08, 0.02), None, RED);
            // แถบเลือดหมาป่า
            let hp_frac = w.hp as f32 / 3.0;
            let bar_pos = w.pos + vec3(0.0, 1.1, 0.0);
            draw_cube(bar_pos, vec3(0.4, 0.06, 0.02), None, Color::from_rgba(40, 40, 40, 200));
            draw_cube(bar_pos + vec3(-0.2 + 0.2 * hp_frac, 0.0, 0.0), vec3(0.4 * hp_frac, 0.04, 0.02), None, RED);
        }

        if third_person {
            let skin = Color::from_rgba(230, 190, 150, 255);
            let shirt = Color::from_rgba(40, 90, 200, 255);
            let pants = Color::from_rgba(30, 45, 90, 255);
            let eye = Color::from_rgba(25, 25, 25, 255);
            let rot = |lx: f32, ly: f32, lz: f32| vec3(
                pos.x + lx * yaw.cos() + lz * yaw.sin(),
                pos.y + ly,
                pos.z - lx * yaw.sin() + lz * yaw.cos(),
            );
            let moving = horiz != Vec3::ZERO;
            let swing = if moving { (get_time() as f32 * 8.0).sin() * 0.25 } else { 0.0 };
            draw_cube(rot(0.0, 0.45, 0.0), vec3(0.4, 0.4, 0.4), None, skin);
            draw_cube(rot(-0.11, 0.45, 0.21), vec3(0.07, 0.07, 0.02), None, eye);
            draw_cube(rot(0.11, 0.45, 0.21), vec3(0.07, 0.07, 0.02), None, eye);
            draw_cube(rot(0.0, 0.05, 0.0), vec3(0.5, 0.6, 0.25), None, shirt);
            draw_cube(rot(-0.33, 0.05, -swing), vec3(0.15, 0.6, 0.15), None, shirt);
            draw_cube(rot(0.33, 0.05, swing), vec3(0.15, 0.6, 0.15), None, shirt);
            draw_cube(rot(-0.15, -0.5, swing), vec3(0.2, 0.5, 0.2), None, pants);
            draw_cube(rot(0.15, -0.5, -swing), vec3(0.2, 0.5, 0.2), None, pants);
        }

        if mouse_grabbed {
            if let Some((hit, normal)) = raycast(pos, forward, &world, 8.0) {
                let hp = vec3(hit.0 as f32 + 0.5, hit.1 as f32 + 0.5, hit.2 as f32 + 0.5);
                draw_cube_wires(hp, vec3(1.01, 1.01, 1.01), BLACK);

                if is_mouse_button_pressed(MouseButton::Left) {
                    world.remove(&hit);
                }
                if is_mouse_button_pressed(MouseButton::Right) {
                    let place = (hit.0 + normal.0, hit.1 + normal.1, hit.2 + normal.2);
                    let player_cell = (
                        pos.x.floor() as i32,
                        pos.y.floor() as i32,
                        pos.z.floor() as i32,
                    );
                    if place != player_cell {
                        world.insert(place, selected);
                    }
                }
            }
        }

        set_default_camera();
        let day_str = if nightness < 0.5 { "DAY" } else { "NIGHT" };
        draw_text(&format!("{} | wolves: {} | LMB = hit wolf", day_str, wolves.len()), 20.0, 20.0, 32.0, WHITE);

        next_frame().await
    }
}
