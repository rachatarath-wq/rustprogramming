// บทที่ 20 — กล้อง first-person (เมาส์มอง + WASD เดิน)
use macroquad::prelude::*;

#[macroquad::main("My 3D Game")]
async fn main() {
    let mut pos = vec3(0.0, 2.0, 5.0);   // ตำแหน่งผู้เล่น
    let mut yaw = 0.0f32;                 // หันซ้าย/ขวา
    let mut pitch = 0.0f32;               // เงย/ก้ม
    let speed = 5.0;

    loop {
        // 1. มองรอบด้วยเมาส์
        let mouse_delta = mouse_delta_position();
        yaw -= mouse_delta.x * 0.008;
        pitch = (pitch - mouse_delta.y * 0.008).clamp(-1.5, 1.5);

        // 2. ทิศทางกล้อง (forward / right)
        let forward = vec3(
            pitch.cos() * yaw.sin(),
            pitch.sin(),
            pitch.cos() * yaw.cos(),
        )
        .normalize();
        let right = vec3(-yaw.cos(), 0.0, yaw.sin());

        // 3. เดินด้วย WASD
        let mut move_dir = Vec3::ZERO;
        if is_key_down(KeyCode::W) { move_dir += forward; }
        if is_key_down(KeyCode::S) { move_dir -= forward; }
        if is_key_down(KeyCode::D) { move_dir += right; }
        if is_key_down(KeyCode::A) { move_dir -= right; }
        if move_dir != Vec3::ZERO {
            pos += move_dir.normalize() * speed * get_frame_time();
        }

        // 4. กล้อง first-person
        let camera = Camera3D {
            position: pos,
            target: pos + forward,
            up: vec3(0.0, 1.0, 0.0),
            fovy: 45.0,
            ..Default::default()
        };
        set_camera(&camera);

        clear_background(SKYBLUE);

        // คิวบ์อ้างอิงเพื่อให้เห็นการเคลื่อนที่
        draw_cube(vec3(0.0, 1.0, 0.0), vec3(2.0, 2.0, 2.0), None, RED);
        draw_cube(vec3(4.0, 1.0, 2.0), vec3(2.0, 2.0, 2.0), None, GREEN);
        draw_cube(vec3(-4.0, 1.0, 2.0), vec3(2.0, 2.0, 2.0), None, BLUE);

        // พื้น
        draw_cube(vec3(0.0, -1.0, 0.0), vec3(12.0, 2.0, 12.0), None, GRAY);

        // กลับกล้อง 2D เพื่อวาดข้อความ
        set_default_camera();
        draw_text("WASD = walk | mouse = look", 20.0, 20.0, 32.0, WHITE);

        next_frame().await
    }
}
