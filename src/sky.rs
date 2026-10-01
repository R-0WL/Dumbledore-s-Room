// Skybox procedural animado: ciclo dia/noche continuo + clima (nubes y lluvia) variable.
use crate::util::*;
use nalgebra_glm::Vec3;
use std::f32::consts::TAU;

pub const DAY_LEN: f32 = 120.0; // segundos reales por dia completo
pub const START_DAY: f32 = 0.22; // empieza justo antes del amanecer

pub struct Env {
    pub t: f32,
    pub abs_day: f32,
    pub day: f32,
    pub sun: Vec3,
    pub moon: Vec3,
    pub cloud: f32,
    pub rain: f32,
    pub day_w: f32,
    pub twi: f32,
    pub tw_col: Vec3,
    pub sun_tint: Vec3,
    pub zenith: Vec3,
    pub horizon: Vec3,
    pub key_dir: Vec3,
    pub key_col: Vec3,
    pub ambient: Vec3,
}

// Clima: nubosidad base suave + tormentas por dia. El dia 0 siempre tiene una tormenta por la tarde;
// los demas dias aparecen (o no) de forma pseudoaleatoria.
fn weather(abs_day: f32) -> (f32, f32) {
    let k0 = abs_day.floor() as i32;
    let mut c = 0.12 + 0.18 * (0.5 + 0.5 * (abs_day * TAU * 2.7 + 1.3).sin());
    for k in (k0 - 1)..=(k0 + 1) {
        let (center, strength) = if k == 0 {
            (0.62, 1.0)
        } else {
            let s = if hash1(k * 13 + 5) > 0.45 { 0.55 + 0.45 * hash1(k * 5 + 1) } else { 0.0 };
            (0.2 + 0.7 * hash1(k * 7 + 3), s)
        };
        let dd = (abs_day - (k as f32 + center)) / 0.11;
        c += strength * (-dd * dd).exp() * 0.95;
    }
    let cloud = clamp01(c);
    let rain = smoothstep(0.72, 0.90, cloud);
    (cloud, rain)
}

pub fn make_env(sim_time: f32, real_t: f32) -> Env {
    let abs_day = START_DAY + sim_time / DAY_LEN;
    let day = abs_day - abs_day.floor();
    let a = TAU * (day - 0.25);
    let sun = v(a.cos(), a.sin(), 0.55).normalize();
    let moon = v(-a.cos(), -a.sin(), 0.45).normalize();
    let (cloud, rain) = weather(abs_day);

    let e = sun.y;
    let day_w = smoothstep(-0.10, 0.30, e);
    let twi = (-(e / 0.17) * (e / 0.17)).exp();
    let tw_col = if day < 0.5 { v(1.0, 0.50, 0.45) } else { v(1.0, 0.42, 0.12) };

    let mut zenith = mixv(v(0.006, 0.010, 0.035), v(0.10, 0.30, 0.72), day_w);
    let mut horizon = mixv(v(0.025, 0.035, 0.085), v(0.60, 0.77, 0.95), day_w);
    horizon = horizon + tw_col * (0.75 * twi);
    zenith = zenith + v(0.10, 0.06, 0.18) * (0.5 * twi);
    let over_z = mixv(v(0.02, 0.025, 0.04), v(0.42, 0.45, 0.50), day_w);
    let over_h = mixv(v(0.03, 0.035, 0.05), v(0.58, 0.60, 0.63), day_w) + tw_col * (0.35 * twi);
    zenith = mixv(zenith, over_z, cloud * 0.85);
    horizon = mixv(horizon, over_h, cloud * 0.7);

    let sun_i = smoothstep(-0.04, 0.22, sun.y);
    let moon_i = smoothstep(-0.04, 0.22, moon.y) * (1.0 - sun_i);
    let sun_tint = mixv(v(1.0, 0.50, 0.22), v(1.0, 0.93, 0.80), smoothstep(0.0, 0.45, sun.y));
    let sun_col = sun_tint * (2.6 * sun_i * (1.0 - 0.85 * cloud));
    let moon_col = v(0.45, 0.55, 0.95) * (0.55 * moon_i * (1.0 - 0.6 * cloud));
    let (key_dir, key_col) = if sun_col.norm() >= moon_col.norm() { (sun, sun_col) } else { (moon, moon_col) };
    let ambient = mixv(zenith, horizon, 0.45) * 0.85 + v(0.012, 0.014, 0.022);

    Env {
        t: real_t, abs_day, day, sun, moon, cloud, rain, day_w, twi, tw_col, sun_tint,
        zenith, horizon, key_dir, key_col, ambient,
    }
}

pub fn phase_name(env: &Env) -> &'static str {
    let e = env.sun.y;
    if e < -0.12 {
        if env.cloud > 0.6 { "Noche nublada" } else { "Noche estrellada" }
    } else if e < 0.14 {
        if env.day < 0.5 { "Amanecer" } else { "Atardecer" }
    } else if env.rain > 0.3 {
        "Lluvia"
    } else if env.cloud > 0.55 {
        "Nublado"
    } else if e > 0.75 {
        "Mediodia"
    } else {
        "Dia"
    }
}

fn stars(d: Vec3, t: f32) -> Vec3 {
    let mut s = v(0.0, 0.0, 0.0);
    let layers: [(f32, f32, i32); 2] = [(48.0, 0.985, 1), (95.0, 0.992, 7)];
    for (scale, thr, seed) in layers.iter() {
        let p = d * *scale;
        let ix = p.x.floor();
        let iy = p.y.floor();
        let iz = p.z.floor();
        let (cx, cy, cz) = (ix as i32 + seed * 101, iy as i32, iz as i32 + seed * 37);
        let h = hash3(cx, cy, cz);
        if h > *thr {
            let sx = 0.2 + 0.6 * hash3(cx, cy, cz + 1);
            let sy = 0.2 + 0.6 * hash3(cx, cy + 1, cz);
            let sz = 0.2 + 0.6 * hash3(cx + 1, cy, cz);
            let dv = v(p.x - ix - sx, p.y - iy - sy, p.z - iz - sz).norm();
            let k = smoothstep(0.13, 0.0, dv) * (0.7 + 0.3 * (t * 3.0 + h * 50.0).sin());
            let tint = mixv(v(0.75, 0.85, 1.0), v(1.0, 0.92, 0.75), hash3(cx, cy, cz + 9));
            s = s + tint * (k * 1.6);
        }
    }
    s
}

pub fn sky_color(env: &Env, d: Vec3) -> Vec3 {
    let y = d.y;
    let h = clamp01(y);
    let mut col = mixv(env.horizon, env.zenith, h.powf(0.55));
    if y < 0.0 {
        col = env.horizon * (1.0 + y * 0.6).max(0.4);
    }

    // Sol
    let sd = d.dot(&env.sun);
    let sun_vis = smoothstep(-0.06, 0.04, env.sun.y);
    if sd > 0.0 {
        let glow = sd.powf(24.0) * 0.35 + sd.powf(400.0) * 1.2;
        col = col + env.sun_tint * (glow * sun_vis * (1.0 - 0.8 * env.cloud));
        if sd > 0.9996 {
            let k = smoothstep(0.9996, 0.99975, sd);
            col = col + env.sun_tint * (12.0 * k * sun_vis * (1.0 - 0.95 * env.cloud));
        }
    }
    // Luna
    let md = d.dot(&env.moon);
    let moon_vis = smoothstep(-0.06, 0.05, env.moon.y) * (1.0 - env.day_w);
    if md > 0.0 {
        col = col + v(0.35, 0.42, 0.60) * (md.powf(60.0) * 0.25 * moon_vis * (1.0 - 0.8 * env.cloud));
        if md > 0.9993 {
            let shade = 0.75 + 0.25 * noise3(d * 45.0);
            let k = smoothstep(0.9993, 0.9995, md) * moon_vis * (1.0 - 0.9 * env.cloud);
            col = mixv(col, v(0.92, 0.93, 0.85) * (shade * 1.2), k);
        }
    }
    // Estrellas
    let star_vis = (1.0 - env.day_w).powi(2) * (1.0 - env.cloud * 0.9) * smoothstep(0.0, 0.12, y);
    if star_vis > 0.01 {
        col = col + stars(d, env.t) * star_vis;
    }

    // Color de nubes segun la luz actual
    let lit = env.key_col * 0.30 + env.ambient * 1.3 + v(0.02, 0.02, 0.03);
    let base = v(0.97, 0.97, 1.0);
    let wind = env.t * 0.012;
    if y > 0.0 {
        let k = 1.0 / (y + 0.18);
        let n = fbm3(v(d.x * k * 1.3 + wind, d.z * k * 1.3 - wind * 0.5, env.t * 0.01), 4);
        let thr = 0.60 - 0.42 * env.cloud;
        let dens = smoothstep(thr, thr + 0.20, n) * smoothstep(0.0, 0.10, y);
        let shade = 1.0 - 0.50 * smoothstep(thr + 0.15, thr + 0.45, n) * (0.4 + 0.6 * env.cloud);
        let mut ccol = base.component_mul(&lit) * shade;
        ccol = ccol + env.tw_col * (0.6 * env.twi * (1.0 - 0.5 * env.cloud) * shade);
        ccol = ccol * (1.0 - 0.35 * env.rain);
        col = mixv(col, ccol, dens * (0.6 + 0.4 * env.cloud));
    } else {
        // mar de nubes bajo la isla flotante
        let k = 1.0 / (-y + 0.25);
        let n = fbm3(v(d.x * k * 0.9 + wind * 1.6, d.z * k * 0.9, 5.0 + env.t * 0.005), 4);
        let cs = smoothstep(0.38, 0.70, n) * smoothstep(0.0, 0.15, -y);
        let mut ccol = base.component_mul(&lit) * (0.75 + 0.25 * n);
        ccol = ccol + env.tw_col * (0.4 * env.twi);
        col = mixv(col, ccol, cs);
    }

    // Lluvia: estrias que caen
    if env.rain > 0.02 && y > -0.1 {
        let az = d.x.atan2(d.z);
        let s = az * 70.0;
        let ci = s.floor() as i32;
        let fr = s - s.floor();
        if (fr - 0.5).abs() < 0.14 {
            let yy = y * 28.0 + env.t * (3.0 + 3.0 * hash1(ci)) + hash1(ci * 3 + 1) * 10.0;
            let ph = yy - yy.floor();
            if ph < 0.22 {
                let k = env.rain * 0.30 * (1.0 - ph / 0.22) * (0.35 + 0.65 * env.day_w);
                col = col + v(0.70, 0.75, 0.85) * k;
            }
        }
        col = mixv(col, env.horizon * 0.9, env.rain * 0.30);
    }
    col
}
