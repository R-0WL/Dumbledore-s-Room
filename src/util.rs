// Utilidades matematicas: hash, ruido, mezcla, reflexion/refraccion.
use nalgebra_glm::Vec3;

pub fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

pub fn clamp01(x: f32) -> f32 {
    x.max(0.0).min(1.0)
}

pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = clamp01((x - e0) / (e1 - e0));
    t * t * (3.0 - 2.0 * t)
}

pub fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn mixv(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a * (1.0 - t) + b * t
}

pub fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^= x >> 16;
    x
}

pub fn hash3(x: i32, y: i32, z: i32) -> f32 {
    let h = hash_u32(
        (x as u32).wrapping_mul(73856093)
            ^ (y as u32).wrapping_mul(19349663)
            ^ (z as u32).wrapping_mul(83492791),
    );
    (h & 0x00FF_FFFF) as f32 / 16777216.0
}

pub fn hash1(x: i32) -> f32 {
    hash3(x, 17, 91)
}

/// Ruido de valor 3D suave en [0,1].
pub fn noise3(p: Vec3) -> f32 {
    let fx = p.x.floor();
    let fy = p.y.floor();
    let fz = p.z.floor();
    let ix = fx as i32;
    let iy = fy as i32;
    let iz = fz as i32;
    let tx = p.x - fx;
    let ty = p.y - fy;
    let tz = p.z - fz;
    let sx = tx * tx * (3.0 - 2.0 * tx);
    let sy = ty * ty * (3.0 - 2.0 * ty);
    let sz = tz * tz * (3.0 - 2.0 * tz);
    let c = |dx: i32, dy: i32, dz: i32| hash3(ix + dx, iy + dy, iz + dz);
    let x00 = mix(c(0, 0, 0), c(1, 0, 0), sx);
    let x10 = mix(c(0, 1, 0), c(1, 1, 0), sx);
    let x01 = mix(c(0, 0, 1), c(1, 0, 1), sx);
    let x11 = mix(c(0, 1, 1), c(1, 1, 1), sx);
    let y0 = mix(x00, x10, sy);
    let y1 = mix(x01, x11, sy);
    mix(y0, y1, sz)
}

pub fn fbm3(p: Vec3, octaves: u32) -> f32 {
    let mut amp = 0.5;
    let mut sum = 0.0;
    let mut q = p;
    for _ in 0..octaves {
        sum += amp * noise3(q);
        q = q * 2.03 + v(11.3, 7.1, 3.7);
        amp *= 0.5;
    }
    sum
}

pub fn reflect(d: Vec3, n: Vec3) -> Vec3 {
    d - n * (2.0 * d.dot(&n))
}

/// Refraccion de Snell. `n` debe apuntar contra `d`. eta = n1 / n2.
pub fn refract(d: Vec3, n: Vec3, eta: f32) -> Option<Vec3> {
    let cosi = -d.dot(&n);
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
    if k < 0.0 {
        None
    } else {
        Some(d * eta + n * (eta * cosi - k.sqrt()))
    }
}

/// Aproximacion de Schlick para la reflectancia de Fresnel.
pub fn schlick(cos_i: f32, ior: f32) -> f32 {
    let r0 = ((1.0 - ior) / (1.0 + ior)).powi(2);
    r0 + (1.0 - r0) * (1.0 - clamp01(cos_i)).powi(5)
}

/// Diferencia angular envuelta en [-PI, PI].
pub fn adiff(a: f32, b: f32) -> f32 {
    let pi = std::f32::consts::PI;
    let mut d = a - b;
    while d > pi {
        d -= 2.0 * pi;
    }
    while d < -pi {
        d += 2.0 * pi;
    }
    d
}
