// บทที่ 23 — ทุบ / วางบล็อก (คลิกซ้ายทุบ, คลิกขวาวาง)
use macroquad::prelude::*;
use std::collections::HashMap;

type BlockPos = (i32, i32, i32);

#[derive(Clone, Copy, PartialEq)]
enum Block { Grass, Dirt, Stone, Wood }

impl Block {
    fn color(&self) -> Color {
        match self {
            Block::Grass => Color::from_rgba(96, 190, 84, 255),
            Block::Dirt => Color::from_rgba(143, 104, 68, 255),
            Block::Stone => Color::from_rgba(130, 130, 134, 255),
            Block::Wood => Color::from_rgba(145, 92, 42, 255),
        }
    }
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

#[macroquad::main("My 3D Game")]
async fn main() {
    let mut world = build_world();   // mut เพราะเราทุบ/วางได้
    let selected = Block::Stone;      // บล็อกที่จะวาง

    let mut pos = vec3(0.0, 2.0, 8.0);
    let mut yaw = 0.0f32;
    let mut pitch = 0.0f32;
    let speed = 5.0;

    loop {
        let mouse_delta = mouse_delta_position();
        yaw -= mouse_delta.x * 0.008;
        pitch = (pitch - mouse_delta.y * 0.008).clamp(-1.5, 1.5);

        let forward = vec3(
            pitch.cos() * yaw.sin(),
            pitch.sin(),
            pitch.cos() * yaw.cos(),
        )
        .normalize();
        let right = vec3(-yaw.cos(), 0.0, yaw.sin());

        let mut move_dir = Vec3::ZERO;
        if is_key_down(KeyCode::W) { move_dir += forward; }
        if is_key_down(KeyCode::S) { move_dir -= forward; }
        if is_key_down(KeyCode::D) { move_dir += right; }
        if is_key_down(KeyCode::A) { move_dir -= right; }
        if move_dir != Vec3::ZERO {
            pos += move_dir.normalize() * speed * get_frame_time();
        }

        let camera = Camera3D {
            position: pos,
            target: pos + forward,
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

        // เล็ง + ทุบ/วาง
        if let Some((hit, normal)) = raycast(pos, forward, &world, 8.0) {
            let hp = vec3(hit.0 as f32 + 0.5, hit.1 as f32 + 0.5, hit.2 as f32 + 0.5);
            draw_cube_wires(hp, vec3(1.01, 1.01, 1.01), BLACK);

            // คลิกซ้าย = ทุบบล็อก
            if is_mouse_button_pressed(MouseButton::Left) {
                world.remove(&hit);
            }
            // คลิกขวา = วางบล็อกที่ช่องติดกับด้านที่โดน
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

        set_default_camera();
        draw_text("LMB = break | RMB = place", 20.0, 20.0, 32.0, WHITE);

        next_frame().await
    }
}
