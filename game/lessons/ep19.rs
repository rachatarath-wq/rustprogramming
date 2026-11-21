// บทที่ 19 — Macroquad + คิวบ์ 3D แรก
// ผู้เรียน: สร้างโปรเจกต์ cargo new miniroblox แล้วเพิ่ม macroquad ใน Cargo.toml
// แล้วแทน main.rs ด้วยโค้ดนี้ แล้วรัน cargo run
use macroquad::prelude::*;

#[macroquad::main("My 3D Game")]
async fn main() {
    loop {
        clear_background(SKYBLUE);

        // กล้อง 3D (ตำแหน่ง / จุดที่มอง / ทิศขึ้น / มุมมอง)
        let camera = Camera3D {
            position: vec3(0.0, 3.0, 5.0),
            target: vec3(0.0, 0.0, 0.0),
            up: vec3(0.0, 1.0, 0.0),
            fovy: 45.0,
            ..Default::default()
        };
        set_camera(&camera);

        // คิวบ์ 3D สีแดง
        draw_cube(vec3(0.0, 1.0, 0.0), vec3(2.0, 2.0, 2.0), None, RED);

        // พื้นสีเขียว
        draw_cube(vec3(0.0, -1.0, 0.0), vec3(6.0, 2.0, 6.0), None, GREEN);

        // กลับมาเป็นกล้อง 2D เพื่อวาดข้อความ
        set_default_camera();
        draw_text("My first 3D cube!", 20.0, 20.0, 40.0, WHITE);

        next_frame().await
    }
}
