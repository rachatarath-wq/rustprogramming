use macroquad::prelude::*;

#[macroquad::main("Mini Roblox")]
async fn main() {
    let mut camera = Camera3D {
        position: vec3(0.0, 2.0, 8.0),
        target: vec3(0.0, 1.0, 0.0),
        up: vec3(0.0, 1.0, 0.0),
        fovy: 45.0,
        ..Default::default()
    };

    loop {
        clear_background(SKYBLUE);

        // slowly orbit the camera around the cube
        let t = get_time() as f32;
        camera.position = vec3(t.sin() * 8.0, 3.0, t.cos() * 8.0);
        camera.target = vec3(0.0, 1.0, 0.0);

        set_camera(&camera);
        draw_grid(20, 1.0, BLACK, GRAY);
        draw_cube(vec3(0.0, 1.0, 0.0), vec3(1.0, 1.0, 1.0), None, GREEN);
        draw_cube(vec3(1.5, 0.5, 0.0), vec3(1.0, 1.0, 1.0), None, RED);
        set_default_camera();

        next_frame().await
    }
}
