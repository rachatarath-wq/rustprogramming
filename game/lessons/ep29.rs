// บทที่ 29 — จัดการเมาส์ (Esc) + ย้อนจุดเกิด (T) + ไฟสัญญาณ
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

// จุดเกิด (home)
const HOME: (f32, f32, f32) = (4.0, 6.0, 4.0);

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

    loop {
        let dt = get_frame_time();

        // look only while mouse is grabbed
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

        // T = ย้อนจุดเกิด
        if is_key_pressed(KeyCode::T) {
            pos = vec3(HOME.0, HOME.1, HOME.2);
            vel = Vec3::ZERO;
            grounded = false;
        }
        // Esc = ปล่อย / จับเมาส์
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

        clear_background(SKYBLUE);

        for (p, block) in &world {
            let center = vec3(p.0 as f32 + 0.5, p.1 as f32 + 0.5, p.2 as f32 + 0.5);
            draw_cube(center, vec3(1.0, 1.0, 1.0), None, block.color());
        }

        // ไฟสัญญาณจุดเกิด (เสาเส้นทอง)
        draw_cube_wires(vec3(HOME.0, HOME.1 - 2.5, HOME.2), vec3(0.4, 4.0, 0.4), GOLD);

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

        // ทุบ/วางเฉพาะเมื่อเมาส์ถูกจับ
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
        draw_text("Esc = mouse | T = home | V = view | Space = jump", 20.0, 20.0, 32.0, WHITE);

        next_frame().await
    }
}
