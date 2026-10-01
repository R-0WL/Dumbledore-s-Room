// Despacho de Dumbledore - diorama de cubos texturizados con ray tracing en Rust.
// Controles: ver README.md
mod camera;
mod material;
mod model_data;
mod room;
mod sdf;
mod sky;
mod texture;
mod tracer;
mod util;
mod voxel;

use camera::Camera;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use nalgebra_glm::Vec3;
use sdf::*;
use std::f32::consts::PI;
use std::time::Instant;
use tracer::{render, Ctx, Light};
use util::*;

const WIN_W: usize = 960;
const WIN_H: usize = 720;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Exterior,
    Interior,
}

#[derive(Clone, Copy)]
struct Orbit {
    yaw: f32,
    pitch: f32,
    dist: f32,
}

fn ext_target() -> Vec3 {
    v(10.0, 10.5, 10.0)
}

fn int_target() -> Vec3 {
    v(10.0, 9.2, 10.0)
}

// Ventana (si existe) hacia la que apunta la camara exterior/interior.
fn window_near(yaw: f32, tol: f32) -> Option<f32> {
    for w in room::WINDOWS.iter() {
        if adiff(yaw, w.alpha).abs() < tol {
            return Some(w.alpha);
        }
    }
    None
}

fn build_chars(t: f32) -> Vec<Character> {
    let dumb = Model::build(
        model_data::DUMBLEDORE,
        false,
        WingRule::None,
        0.0,
        v(0.0, 0.0, 0.0),
        true,
        v(0.08, 2.54, 0.05),
        v(1.25, 2.54, 0.78),
    );
    let fawkes = Model::build(
        model_data::FAWKES,
        false,
        WingRule::Fawkes,
        0.18 * (t * 1.4).sin(),
        v(0.0, 2.6, 0.0),
        false,
        v(0.0, 2.0, -0.8),
        v(4.05, 2.05, 1.65),
    );
    let phi = t * 0.25;
    let harry = Model::build(
        model_data::HARRY_BUCKBEAK,
        true,
        WingRule::Buckbeak,
        0.45 * (t * 3.2).sin(),
        v(0.0, 2.5, 0.2),
        false,
        v(0.0, 2.4, -0.2),
        v(4.6, 2.3, 2.5),
    );
    let c = room::center_world();
    vec![
        Character::new(dumb, CharKind::Dumbledore, v(11.4, 7.5, 9.8), -2.24, 0.40),
        Character::new(fawkes, CharKind::Fawkes, v(12.25, 9.5, 8.75), -2.1, 0.36),
        Character::new(
            harry,
            CharKind::Harry,
            v(c.x + 15.0 * phi.sin(), 12.5 + 2.0 * (t * 0.7).sin(), c.z + 15.0 * phi.cos()),
            phi + PI / 2.0,
            0.5,
        ),
    ]
}

fn build_lights(chars: &[Character], t: f32) -> Vec<Light> {
    let flick = 1.0 + 0.12 * (t * 3.1).sin() + 0.06 * (t * 8.3).sin();
    let chest = chars[1].to_world(v(0.0, 2.4, 0.15));
    vec![
        Light { pos: chest, col: v(1.0, 0.50, 0.14) * (2.8 * flick), k: 0.45, shadow: true, ignore_char: Some(1) },
        Light { pos: v(10.0, 8.8, 6.4), col: v(1.0, 0.45, 0.12) * (2.2 * flick), k: 0.5, shadow: true, ignore_char: None },
        Light { pos: v(10.0, 12.3, 10.0), col: v(1.0, 0.72, 0.42) * 2.0, k: 0.12, shadow: false, ignore_char: None },
        Light { pos: v(7.75, 9.7, 11.25), col: v(0.45, 0.65, 1.0) * 1.4, k: 0.8, shadow: false, ignore_char: None },
        Light { pos: v(10.75, 9.4, 8.75), col: v(1.0, 0.65, 0.30) * 0.9, k: 1.2, shadow: false, ignore_char: None },
    ]
}

fn upscale(src: &[u32], sw: usize, sh: usize, dst: &mut [u32], fade: f32) {
    let k = (1.0 - fade).max(0.0);
    for y in 0..WIN_H {
        let sy = y * sh / WIN_H;
        for x in 0..WIN_W {
            let c = src[sy * sw + x * sw / WIN_W];
            dst[y * WIN_W + x] = if fade > 0.001 {
                let r = (((c >> 16) & 255) as f32 * k) as u32;
                let g = (((c >> 8) & 255) as f32 * k) as u32;
                let b = ((c & 255) as f32 * k) as u32;
                (r << 16) | (g << 8) | b
            } else {
                c
            };
        }
    }
}

fn main() {
    let textures = texture::build_textures();
    let mats = material::build_materials();
    let room = room::build_room(&mats);
    let center = room::center_world();

    let mut window = Window::new("Despacho de Dumbledore", WIN_W, WIN_H, WindowOptions::default())
        .expect("No se pudo crear la ventana");

    let mut quality: usize = 2;
    let mut rw = WIN_W / quality;
    let mut rh = WIN_H / quality;
    let mut buf = vec![0u32; rw * rh];
    let mut out = vec![0u32; WIN_W * WIN_H];

    let mut mode = Mode::Exterior;
    let mut ext = Orbit { yaw: 0.65, pitch: 0.15, dist: 30.0 };
    let mut int = Orbit { yaw: PI + 0.35, pitch: 0.12, dist: 3.2 };

    let start = Instant::now();
    let mut last = Instant::now();
    let mut sim_time: f32 = 0.0;
    let mut paused = false;
    let mut fade: f32 = 0.0;
    let mut last_mouse: Option<(f32, f32)> = None;
    let mut title_timer: f32 = 0.0;
    let mut fps: f32 = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f32().min(0.2);
        last = now;
        let anim_t = now.duration_since(start).as_secs_f32();
        fps = fps * 0.9 + (1.0 / dt.max(1e-4)) * 0.1;

        // ---- Entrada ----
        let mut dyaw = 0.0f32;
        let mut dpitch = 0.0f32;
        let mut zoom = 0.0f32; // > 0 aleja, < 0 acerca
        if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {
            dyaw -= 1.4 * dt;
        }
        if window.is_key_down(Key::Right) || window.is_key_down(Key::D) {
            dyaw += 1.4 * dt;
        }
        if window.is_key_down(Key::Up) {
            dpitch += 0.9 * dt;
        }
        if window.is_key_down(Key::Down) {
            dpitch -= 0.9 * dt;
        }
        if window.is_key_down(Key::W) {
            zoom -= 1.0 * dt;
        }
        if window.is_key_down(Key::S) {
            zoom += 1.0 * dt;
        }
        if let Some((_, sy)) = window.get_scroll_wheel() {
            zoom -= sy * 0.12;
        }
        let down = window.get_mouse_down(MouseButton::Left);
        if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Pass) {
            if down {
                if let Some((lx, ly)) = last_mouse {
                    dyaw -= (mx - lx) * 0.008;
                    dpitch += (my - ly) * 0.008;
                }
            }
            last_mouse = Some((mx, my));
        }

        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            paused = !paused;
        }
        if window.is_key_down(Key::Period) {
            sim_time += dt * 20.0;
        }
        if window.is_key_down(Key::Comma) {
            sim_time -= dt * 20.0;
        }
        let jumps = [
            (Key::Key1, 0.25f32),
            (Key::Key2, 0.36),
            (Key::Key3, 0.50),
            (Key::Key4, 0.52),
            (Key::Key5, 0.62),
            (Key::Key6, 0.74),
            (Key::Key7, 0.90),
            (Key::Key8, 0.98),
        ];
        for (k, day) in jumps.iter() {
            if window.is_key_pressed(*k, KeyRepeat::No) {
                sim_time = (*day - sky::START_DAY) * sky::DAY_LEN;
            }
        }
        if window.is_key_pressed(Key::Z, KeyRepeat::No) {
            quality = match quality {
                2 => 3,
                3 => 4,
                4 => 1,
                _ => 2,
            };
            rw = WIN_W / quality;
            rh = WIN_H / quality;
            buf = vec![0u32; rw * rh];
        }
        if !paused {
            sim_time += dt;
        }

        // ---- Camara y cambio de escena ----
        let mut switch_to: Option<Mode> = None;
        match mode {
            Mode::Exterior => {
                ext.yaw += dyaw;
                ext.pitch = (ext.pitch + dpitch).max(-0.5).min(1.2);
                ext.dist = (ext.dist * zoom.exp()).min(60.0);
                if ext.dist < 9.6 {
                    match window_near(ext.yaw, 0.32) {
                        Some(wa) if ext.pitch.abs() < 0.55 => {
                            int = Orbit { yaw: wa, pitch: ext.pitch.max(-0.2).min(0.5), dist: 3.9 };
                            switch_to = Some(Mode::Interior);
                        }
                        _ => ext.dist = 9.6,
                    }
                }
            }
            Mode::Interior => {
                int.yaw += dyaw;
                int.pitch = (int.pitch + dpitch).max(-0.3).min(1.1);
                int.dist = (int.dist * zoom.exp()).max(0.7);
                if int.dist > 3.9 {
                    match window_near(int.yaw, 0.4) {
                        Some(wa) if zoom > 0.0 => {
                            ext = Orbit { yaw: wa, pitch: int.pitch, dist: 10.5 };
                            switch_to = Some(Mode::Exterior);
                        }
                        _ => int.dist = 3.9,
                    }
                }
            }
        }
        if window.is_key_pressed(Key::Tab, KeyRepeat::No) {
            if mode == Mode::Exterior {
                int = Orbit { yaw: ext.yaw, pitch: ext.pitch.max(-0.2).min(0.5), dist: 3.2 };
                switch_to = Some(Mode::Interior);
            } else {
                ext = Orbit { yaw: int.yaw, pitch: int.pitch, dist: 26.0 };
                switch_to = Some(Mode::Exterior);
            }
        }
        if let Some(m) = switch_to {
            mode = m;
            fade = 1.0;
        }
        fade = (fade - dt * 3.0).max(0.0);

        let aspect = WIN_W as f32 / WIN_H as f32;
        let cam = match mode {
            Mode::Exterior => Camera::orbit(ext_target(), ext.yaw, ext.pitch, ext.dist, 55.0, aspect),
            Mode::Interior => Camera::orbit(int_target(), int.yaw, int.pitch, int.dist, 65.0, aspect),
        };

        // ---- Escena dinamica ----
        let env = sky::make_env(sim_time, anim_t);
        let chars = build_chars(anim_t);
        let lights = build_lights(&chars, anim_t);
        let ctx = Ctx {
            grid: &room.grid,
            mats: &mats,
            texs: &textures,
            spheres: &room.spheres,
            chars: &chars,
            lights: &lights,
            env: &env,
            center,
        };
        render(&ctx, &cam, &mut buf, rw, rh);
        upscale(&buf, rw, rh, &mut out, fade);

        title_timer += dt;
        if title_timer > 0.4 {
            title_timer = 0.0;
            let place = if mode == Mode::Exterior { "Exterior" } else { "Interior" };
            window.set_title(&format!(
                "Despacho de Dumbledore | {} | {} | {:.1} fps | calidad 1/{}",
                place,
                sky::phase_name(&env),
                fps,
                quality
            ));
        }
        window.update_with_buffer(&out, WIN_W, WIN_H).expect("No se pudo actualizar la ventana");
    }
}
