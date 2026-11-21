// บทที่ 21 — โลกบล็อก (HashMap + พื้นหญ้า + แท่นหิน + เสาไม้)
use macroquad::prelude::*;
use std::collections::HashMap;

// ตำแหน่งบล็อก = พิกัดจำนวนเต็ม (x, y, z)
type BlockPos = (i32, i32, i32);

// ชนิดของบล็อก
#[derive(Clone, Copy, PartialEq)]
enum Block {
    Grass,
    Dirt,
    Stone,
    Wood,
}

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

// สร้างโลก: พื้นหญ้า + แท่นหิน + เสาไม้
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

#[macroquad::main("My 3D Game")]
async fn main() {
    let world = build_world();

    let mut pos = vec3(0.0, 2.0, 8.0);   // ตำแหน่งผู้เล่น
    let mut yaw = 0.0f32;                 // หันซ้าย/ขวา
    let mut pitch = 0.0f32;               // เงย/ก้ม
    let speed = 5.0;

    loop {
        // มองรอบด้วยเมาส์ (จากตอนที่ 20)
        let mouse_delta = mouse_delta_position();
        yaw -= mouse_delta.x * 0.008;
        pitch = (pitch - mouse_delta.y * 0.008).clamp(-1.5, 1.5);

        // ทิศทางกล้อง
        let forward = vec3(
            pitch.cos() * yaw.sin(),
            pitch.sin(),
            pitch.cos() * yaw.cos(),
        )
        .normalize();
        let right = vec3(-yaw.cos(), 0.0, yaw.sin());

        // เดิน WASD
        let mut move_dir = Vec3::ZERO;
        if is_key_down(KeyCode::W) { move_dir += forward; }
        if is_key_down(KeyCode::S) { move_dir -= forward; }
        if is_key_down(KeyCode::D) { move_dir += right; }
        if is_key_down(KeyCode::A) { move_dir -= right; }
        if move_dir != Vec3::ZERO {
            pos += move_dir.normalize() * speed * get_frame_time();
        }

        // กล้อง first-person
        let camera = Camera3D {
            position: pos,
            target: pos + forward,
            up: vec3(0.0, 1.0, 0.0),
            fovy: 45.0,
            ..Default::default()
        };
        set_camera(&camera);

        clear_background(SKYBLUE);

        // วาดทุกบล็อกในโลก
        for (p, block) in &world {
            let center = vec3(
                p.0 as f32 + 0.5,
                p.1 as f32 + 0.5,
                p.2 as f32 + 0.5,
            );
            draw_cube(center, vec3(1.0, 1.0, 1.0), None, block.color());
        }

        set_default_camera();
        draw_text("WASD = walk | mouse = look", 20.0, 20.0, 32.0, WHITE);

        next_frame().await
    }
}
