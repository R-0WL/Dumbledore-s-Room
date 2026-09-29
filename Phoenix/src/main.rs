// Fenix simplificado, basado en la estructura del archivo modeladors(1).rs.
// Reemplaza src/main.rs; conserva minifb y nalgebra-glm en Cargo.toml.
// Geometria procedural: no necesita scene.bin, modelos ni texturas externas.
// Frente: +Z. Vertical: +Y. Alas extendidas y cola hacia -Z.
// Geometria nueva inspirada en el fenix.glb adjunto; no requiere cargar ese archivo.
// Ejecutar con cargo run --release (el ray marching es costoso en modo debug).
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nalgebra_glm::{cross, dot, Vec3};
use std::f32::consts::PI;

const WIDTH: usize = 900;
const HEIGHT: usize = 680;
const EPS: f32 = 0.00003;

fn v(x: f32, y: f32, z: f32) -> Vec3 { Vec3::new(x, y, z) }

// Cota conservadora de distancia: permite avanzar sin atravesar el elipsoide.
fn ellipsoid(p: Vec3, c: Vec3, r: Vec3, angle: f32) -> f32 {
    let q = p - c;
    let (s, co) = angle.sin_cos();
    let local = v(co*q.x + s*q.y, -s*q.x + co*q.y, q.z);
    (local.component_div(&r).norm() - 1.0) * r.x.min(r.y).min(r.z)
}

// Distancia exacta a un segmento con extremos redondeados.
fn capsule(p: Vec3, a: Vec3, b: Vec3, radius: f32) -> f32 {
    let pa = p-a;
    let ba = b-a;
    let h = (dot(&pa,&ba)/dot(&ba,&ba)).clamp(0.0,1.0);
    (pa-ba*h).norm()-radius
}
fn choose(hit: &mut (f32,usize), distance: f32, material: usize) {
    if distance < hit.0 { *hit = (distance,material); }
}

// Pluma elipsoidal orientada entre dos extremos en el espacio.
fn plume(p: Vec3, a: Vec3, b: Vec3, width: f32, thickness: f32) -> f32 {
    let axis = (b-a).normalize();
    let reference = if axis.z.abs()<0.95 {v(0.0,0.0,1.0)} else {v(0.0,1.0,0.0)};
    let side = cross(&axis,&reference).normalize();
    let back = cross(&side,&axis);
    let q = p-(a+b)*0.5;
    let local = v(dot(&q,&side),dot(&q,&axis),dot(&q,&back));
    ellipsoid(local,v(0.0,0.0,0.0),v(width,(b-a).norm()*0.5,thickness),0.0)
}

fn objects(p: Vec3, mut hit: (f32,usize)) -> (f32,usize) {
    choose(&mut hit,ellipsoid(p,v(0.0,2.12,0.0),v(0.43,0.68,0.35),0.0),0);
    choose(&mut hit,ellipsoid(p,v(0.0,2.39,0.18),v(0.36,0.47,0.24),0.0),1);
    choose(&mut hit,ellipsoid(p,v(0.0,2.88,0.04),v(0.215,0.46,0.23),-0.03),0);
    choose(&mut hit,ellipsoid(p,v(0.0,3.4,0.12),v(0.28,0.29,0.26),0.0),0);
    choose(&mut hit,ellipsoid(p,v(0.0,3.32,0.26),v(0.22,0.2,0.2),0.0),1);
    choose(&mut hit,ellipsoid(p,v(0.0,3.36,0.455),v(0.125,0.1,0.21),0.0),2);
    choose(&mut hit,ellipsoid(p,v(0.0,3.3,0.584),v(0.069,0.11,0.085),0.0),2);
    choose(&mut hit,ellipsoid(p,v(0.0,3.235,0.607),v(0.032,0.065,0.04),0.0),3);
    choose(&mut hit,ellipsoid(p,v(-0.191,3.445,0.301),v(0.076,0.06,0.033),0.0),3);
    choose(&mut hit,ellipsoid(p,v(-0.191,3.449,0.329),v(0.04,0.043,0.013),0.0),2);
    choose(&mut hit,ellipsoid(p,v(-0.191,3.449,0.341),v(0.016,0.031,0.008),0.0),4);
    choose(&mut hit,ellipsoid(p,v(-0.18,3.464,0.349),v(0.008,0.01,0.004),0.0),5);
    choose(&mut hit,ellipsoid(p,v(-0.25,1.66,0.04),v(0.16,0.25,0.18),0.18),0);
    choose(&mut hit,ellipsoid(p,v(0.191,3.445,0.301),v(0.076,0.06,0.033),0.0),3);
    choose(&mut hit,ellipsoid(p,v(0.191,3.449,0.329),v(0.04,0.043,0.013),0.0),2);
    choose(&mut hit,ellipsoid(p,v(0.191,3.449,0.341),v(0.016,0.031,0.008),0.0),4);
    choose(&mut hit,ellipsoid(p,v(0.18,3.464,0.349),v(0.008,0.01,0.004),0.0),5);
    choose(&mut hit,ellipsoid(p,v(0.25,1.66,0.04),v(0.16,0.25,0.18),-0.18),0);
    choose(&mut hit,ellipsoid(p,v(0.0,0.64,0.23),v(0.76,0.13,0.47),0.0),6);
    choose(&mut hit,ellipsoid(p,v(0.0,0.07,0.22),v(0.63,0.075,0.47),0.0),6);
    choose(&mut hit,capsule(p,v(-0.25,1.54,0.07),v(-0.3,1.06,0.14),0.055),3);
    choose(&mut hit,capsule(p,v(-0.3,1.06,0.14),v(-0.3,0.85,0.28),0.046),3);
    choose(&mut hit,capsule(p,v(-0.3,0.86,0.25),v(-0.43,0.79,0.4715),0.032),3);
    choose(&mut hit,capsule(p,v(-0.43,0.79,0.4715),v(-0.43,0.765,0.5265),0.02),4);
    choose(&mut hit,capsule(p,v(-0.3,0.86,0.25),v(-0.3,0.79,0.53),0.032),3);
    choose(&mut hit,capsule(p,v(-0.3,0.79,0.53),v(-0.3,0.765,0.585),0.02),4);
    choose(&mut hit,capsule(p,v(-0.3,0.86,0.25),v(-0.17,0.79,0.4715),0.032),3);
    choose(&mut hit,capsule(p,v(-0.17,0.79,0.4715),v(-0.17,0.765,0.5265),0.02),4);
    choose(&mut hit,capsule(p,v(0.25,1.54,0.07),v(0.3,1.06,0.14),0.055),3);
    choose(&mut hit,capsule(p,v(0.3,1.06,0.14),v(0.3,0.85,0.28),0.046),3);
    choose(&mut hit,capsule(p,v(0.3,0.86,0.25),v(0.17,0.79,0.4715),0.032),3);
    choose(&mut hit,capsule(p,v(0.17,0.79,0.4715),v(0.17,0.765,0.5265),0.02),4);
    choose(&mut hit,capsule(p,v(0.3,0.86,0.25),v(0.3,0.79,0.53),0.032),3);
    choose(&mut hit,capsule(p,v(0.3,0.79,0.53),v(0.3,0.765,0.585),0.02),4);
    choose(&mut hit,capsule(p,v(0.3,0.86,0.25),v(0.43,0.79,0.4715),0.032),3);
    choose(&mut hit,capsule(p,v(0.43,0.79,0.4715),v(0.43,0.765,0.5265),0.02),4);
    choose(&mut hit,capsule(p,v(0.0,0.1,0.22),v(0.0,0.58,0.22),0.18),6);
    choose(&mut hit,plume(p,v(-0.025,3.59,0.08),v(-0.05,3.96,0.01),0.055,0.04),0);
    choose(&mut hit,plume(p,v(-0.073,3.59,0.055),v(-0.155,3.925,-0.065),0.055,0.04),1);
    choose(&mut hit,plume(p,v(-0.121,3.59,0.03),v(-0.26,3.89,-0.14),0.055,0.04),0);
    choose(&mut hit,plume(p,v(-0.25,2.6,-0.03),v(-1.45,2.95,-0.08),0.29,0.16),0);
    choose(&mut hit,plume(p,v(-1.15,2.84,-0.08),v(-2.65,2.76,-0.16),0.25,0.12),0);
    choose(&mut hit,plume(p,v(-1.2,2.9,-0.12),v(-2.15,1.8,-0.22),0.145,0.066),0);
    choose(&mut hit,plume(p,v(-1.789,2.218,-0.164),v(-2.15,1.8,-0.202),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(-1.315,2.875,-0.146),v(-2.355,1.92,-0.256),0.145,0.066),0);
    choose(&mut hit,plume(p,v(-1.9598,2.2829,-0.1962),v(-2.355,1.92,-0.238),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(-1.43,2.85,-0.172),v(-2.56,2.04,-0.292),0.145,0.066),0);
    choose(&mut hit,plume(p,v(-2.1306,2.3478,-0.2284),v(-2.56,2.04,-0.274),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(-1.545,2.825,-0.198),v(-2.765,2.16,-0.328),0.145,0.066),0);
    choose(&mut hit,plume(p,v(-2.3014,2.4127,-0.2606),v(-2.765,2.16,-0.31),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(-1.66,2.8,-0.224),v(-2.97,2.28,-0.364),0.145,0.066),0);
    choose(&mut hit,plume(p,v(-2.4722,2.4776,-0.2928),v(-2.97,2.28,-0.346),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(-1.775,2.775,-0.25),v(-3.175,2.4,-0.4),0.145,0.066),0);
    choose(&mut hit,plume(p,v(-2.643,2.5425,-0.325),v(-3.175,2.4,-0.382),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(-1.89,2.75,-0.276),v(-3.38,2.52,-0.436),0.145,0.066),0);
    choose(&mut hit,plume(p,v(-2.8138,2.6074,-0.3572),v(-3.38,2.52,-0.418),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(-2.005,2.725,-0.302),v(-3.585,2.64,-0.472),0.145,0.066),0);
    choose(&mut hit,plume(p,v(-2.9846,2.6723,-0.3894),v(-3.585,2.64,-0.454),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(-2.12,2.7,-0.328),v(-3.79,2.76,-0.508),0.145,0.066),0);
    choose(&mut hit,plume(p,v(-3.1554,2.7372,-0.4216),v(-3.79,2.76,-0.49),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(-0.48,2.65,0.055),v(-0.74,2.02,0.065),0.145,0.07),1);
    choose(&mut hit,plume(p,v(-0.6412,2.2594,0.0792),v(-0.74,2.02,0.083),0.0899,0.0602),2);
    choose(&mut hit,plume(p,v(-0.66,2.695,0.055),v(-1.01,2.05,0.065),0.145,0.07),1);
    choose(&mut hit,plume(p,v(-0.877,2.2951,0.0792),v(-1.01,2.05,0.083),0.0899,0.0602),2);
    choose(&mut hit,plume(p,v(-0.84,2.74,0.055),v(-1.28,2.08,0.065),0.145,0.07),1);
    choose(&mut hit,plume(p,v(-1.1128,2.3308,0.0792),v(-1.28,2.08,0.083),0.0899,0.0602),2);
    choose(&mut hit,plume(p,v(-1.02,2.785,0.055),v(-1.55,2.11,0.065),0.145,0.07),1);
    choose(&mut hit,plume(p,v(-1.3486,2.3665,0.0792),v(-1.55,2.11,0.083),0.0899,0.0602),2);
    choose(&mut hit,plume(p,v(-1.2,2.83,0.055),v(-1.82,2.14,0.065),0.145,0.07),1);
    choose(&mut hit,plume(p,v(-1.5844,2.4022,0.0792),v(-1.82,2.14,0.083),0.0899,0.0602),2);
    choose(&mut hit,plume(p,v(-0.55,2.78,0.135),v(-0.89,2.58,0.15),0.105,0.055),1);
    choose(&mut hit,plume(p,v(-0.84,2.795,0.135),v(-1.18,2.6,0.15),0.105,0.055),1);
    choose(&mut hit,plume(p,v(-1.13,2.81,0.135),v(-1.47,2.62,0.15),0.105,0.055),1);
    choose(&mut hit,plume(p,v(-1.42,2.825,0.135),v(-1.76,2.64,0.15),0.105,0.055),1);
    choose(&mut hit,plume(p,v(-1.71,2.84,0.135),v(-2.05,2.66,0.15),0.105,0.055),1);
    choose(&mut hit,plume(p,v(-2.0,2.855,0.135),v(-2.34,2.68,0.15),0.105,0.055),1);
    choose(&mut hit,plume(p,v(0.025,3.59,0.08),v(0.05,3.96,0.01),0.055,0.04),0);
    choose(&mut hit,plume(p,v(0.073,3.59,0.055),v(0.155,3.925,-0.065),0.055,0.04),1);
    choose(&mut hit,plume(p,v(0.121,3.59,0.03),v(0.26,3.89,-0.14),0.055,0.04),0);
    choose(&mut hit,plume(p,v(0.25,2.6,-0.03),v(1.45,2.95,-0.08),0.29,0.16),0);
    choose(&mut hit,plume(p,v(1.15,2.84,-0.08),v(2.65,2.76,-0.16),0.25,0.12),0);
    choose(&mut hit,plume(p,v(1.2,2.9,-0.12),v(2.15,1.8,-0.22),0.145,0.066),0);
    choose(&mut hit,plume(p,v(1.789,2.218,-0.164),v(2.15,1.8,-0.202),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(1.315,2.875,-0.146),v(2.355,1.92,-0.256),0.145,0.066),0);
    choose(&mut hit,plume(p,v(1.9598,2.2829,-0.1962),v(2.355,1.92,-0.238),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(1.43,2.85,-0.172),v(2.56,2.04,-0.292),0.145,0.066),0);
    choose(&mut hit,plume(p,v(2.1306,2.3478,-0.2284),v(2.56,2.04,-0.274),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(1.545,2.825,-0.198),v(2.765,2.16,-0.328),0.145,0.066),0);
    choose(&mut hit,plume(p,v(2.3014,2.4127,-0.2606),v(2.765,2.16,-0.31),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(1.66,2.8,-0.224),v(2.97,2.28,-0.364),0.145,0.066),0);
    choose(&mut hit,plume(p,v(2.4722,2.4776,-0.2928),v(2.97,2.28,-0.346),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(1.775,2.775,-0.25),v(3.175,2.4,-0.4),0.145,0.066),0);
    choose(&mut hit,plume(p,v(2.643,2.5425,-0.325),v(3.175,2.4,-0.382),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(1.89,2.75,-0.276),v(3.38,2.52,-0.436),0.145,0.066),0);
    choose(&mut hit,plume(p,v(2.8138,2.6074,-0.3572),v(3.38,2.52,-0.418),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(2.005,2.725,-0.302),v(3.585,2.64,-0.472),0.145,0.066),0);
    choose(&mut hit,plume(p,v(2.9846,2.6723,-0.3894),v(3.585,2.64,-0.454),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(2.12,2.7,-0.328),v(3.79,2.76,-0.508),0.145,0.066),0);
    choose(&mut hit,plume(p,v(3.1554,2.7372,-0.4216),v(3.79,2.76,-0.49),0.0899,0.05676),1);
    choose(&mut hit,plume(p,v(0.48,2.65,0.055),v(0.74,2.02,0.065),0.145,0.07),1);
    choose(&mut hit,plume(p,v(0.6412,2.2594,0.0792),v(0.74,2.02,0.083),0.0899,0.0602),2);
    choose(&mut hit,plume(p,v(0.66,2.695,0.055),v(1.01,2.05,0.065),0.145,0.07),1);
    choose(&mut hit,plume(p,v(0.877,2.2951,0.0792),v(1.01,2.05,0.083),0.0899,0.0602),2);
    choose(&mut hit,plume(p,v(0.84,2.74,0.055),v(1.28,2.08,0.065),0.145,0.07),1);
    choose(&mut hit,plume(p,v(1.1128,2.3308,0.0792),v(1.28,2.08,0.083),0.0899,0.0602),2);
    choose(&mut hit,plume(p,v(1.02,2.785,0.055),v(1.55,2.11,0.065),0.145,0.07),1);
    choose(&mut hit,plume(p,v(1.3486,2.3665,0.0792),v(1.55,2.11,0.083),0.0899,0.0602),2);
    choose(&mut hit,plume(p,v(1.2,2.83,0.055),v(1.82,2.14,0.065),0.145,0.07),1);
    choose(&mut hit,plume(p,v(1.5844,2.4022,0.0792),v(1.82,2.14,0.083),0.0899,0.0602),2);
    choose(&mut hit,plume(p,v(0.55,2.78,0.135),v(0.89,2.58,0.15),0.105,0.055),1);
    choose(&mut hit,plume(p,v(0.84,2.795,0.135),v(1.18,2.6,0.15),0.105,0.055),1);
    choose(&mut hit,plume(p,v(1.13,2.81,0.135),v(1.47,2.62,0.15),0.105,0.055),1);
    choose(&mut hit,plume(p,v(1.42,2.825,0.135),v(1.76,2.64,0.15),0.105,0.055),1);
    choose(&mut hit,plume(p,v(1.71,2.84,0.135),v(2.05,2.66,0.15),0.105,0.055),1);
    choose(&mut hit,plume(p,v(2.0,2.855,0.135),v(2.34,2.68,0.15),0.105,0.055),1);
    choose(&mut hit,plume(p,v(-0.165,1.75,-0.18),v(-1.29,0.32,-0.76),0.145,0.074),0);
    choose(&mut hit,plume(p,v(-0.8625,0.8634,-0.5216),v(-1.29,0.32,-0.742),0.0899,0.06364),1);
    choose(&mut hit,plume(p,v(-0.11,1.75,-0.18),v(-0.86,0.28,-0.88),0.145,0.074),1);
    choose(&mut hit,plume(p,v(-0.575,0.8386,-0.596),v(-0.86,0.28,-0.862),0.0899,0.06364),2);
    choose(&mut hit,plume(p,v(-0.055,1.75,-0.18),v(-0.43,0.24,-1.0),0.145,0.074),0);
    choose(&mut hit,plume(p,v(-0.2875,0.8138,-0.6704),v(-0.43,0.24,-0.982),0.0899,0.06364),1);
    choose(&mut hit,plume(p,v(0.0,1.75,-0.18),v(0.0,0.2,-1.12),0.18,0.074),1);
    choose(&mut hit,plume(p,v(0.0,0.789,-0.7448),v(0.0,0.2,-1.102),0.1116,0.06364),2);
    choose(&mut hit,plume(p,v(0.055,1.75,-0.18),v(0.43,0.24,-1.0),0.145,0.074),0);
    choose(&mut hit,plume(p,v(0.2875,0.8138,-0.6704),v(0.43,0.24,-0.982),0.0899,0.06364),1);
    choose(&mut hit,plume(p,v(0.11,1.75,-0.18),v(0.86,0.28,-0.88),0.145,0.074),1);
    choose(&mut hit,plume(p,v(0.575,0.8386,-0.596),v(0.86,0.28,-0.862),0.0899,0.06364),2);
    choose(&mut hit,plume(p,v(0.165,1.75,-0.18),v(1.29,0.32,-0.76),0.145,0.074),0);
    choose(&mut hit,plume(p,v(0.8625,0.8634,-0.5216),v(1.29,0.32,-0.742),0.0899,0.06364),1);
    choose(&mut hit,plume(p,v(-0.14,1.87,-0.23),v(-0.4,1.16,-0.87),0.14,0.09),1);
    choose(&mut hit,plume(p,v(-0.07,1.87,-0.23),v(-0.2,1.16,-0.87),0.14,0.09),1);
    choose(&mut hit,plume(p,v(0.0,1.87,-0.23),v(0.0,1.16,-0.87),0.14,0.09),1);
    choose(&mut hit,plume(p,v(0.07,1.87,-0.23),v(0.2,1.16,-0.87),0.14,0.09),1);
    choose(&mut hit,plume(p,v(0.14,1.87,-0.23),v(0.4,1.16,-0.87),0.14,0.09),1);
    hit
}

fn scene(p: Vec3) -> (f32,usize) {
    let mut hit = (p.y,7);
    let q = (p-v(0.0,2.0,-0.8)).map(|x| x.abs())-v(4.05,2.05,1.65);
    let bound = q.map(|x| x.max(0.0)).norm()+q.x.max(q.y).max(q.z).min(0.0);
    if bound>0.015 { choose(&mut hit,bound,0); return hit; }
    objects(p,hit)
}

fn normal(p: Vec3) -> Vec3 {
    let e = 0.001;
    v(scene(p+v(e,0.0,0.0)).0-scene(p-v(e,0.0,0.0)).0,
      scene(p+v(0.0,e,0.0)).0-scene(p-v(0.0,e,0.0)).0,
      scene(p+v(0.0,0.0,e)).0-scene(p-v(0.0,0.0,e)).0).normalize()
}

fn trace(origin: Vec3, direction: Vec3) -> Option<(Vec3,usize)> {
    let mut t = 0.0;
    for _ in 0..512 {
        let p = origin+direction*t;
        let (d,m) = scene(p);
        if d < EPS { return Some((p,m)); }
        t += d * 0.85;
        if t > 35.0 { break; }
    }
    None
}

fn shadow(p: Vec3, light: Vec3) -> f32 {
    let delta = light-p;
    let direction = delta.normalize();
    let mut t: f32 = 0.012;
    let visibility: f32 = 1.0;
    for _ in 0..128 {
        let h = scene(p+direction*t).0;
        if h < EPS*0.1 { return 0.0; }
        t += h.max(0.0001)*0.85;
        if t >= delta.norm() { break; }
    }
    visibility.clamp(0.0,1.0)
}

fn pack(c: Vec3) -> u32 {
    let channel = |x: f32| (255.0*x.clamp(0.0,1.0).sqrt()) as u32;
    (channel(c.x)<<16) | (channel(c.y)<<8) | channel(c.z)
}

fn render(buffer: &mut [u32], yaw: f32, elevation: f32, night: bool) {
    let target = v(0.0,1.98,-0.35);
    let eye = target + v(yaw.sin()*elevation.cos(), elevation.sin(), yaw.cos()*elevation.cos())*7.3;
    let forward = (target-eye).normalize();
    let right = cross(&forward,&v(0.0,1.0,0.0)).normalize();
    let up = cross(&right,&forward);
    let scale = (PI/6.0).tan();
    let light = v(-3.5,7.0,5.0);
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let rows = (HEIGHT+threads-1)/threads;
    std::thread::scope(|scope| {
    for (block, pixels) in buffer.chunks_mut(rows*WIDTH).enumerate() {
        scope.spawn(move || {
        for (row,line) in pixels.chunks_mut(WIDTH).enumerate() {
            let y = block*rows+row;

        for x in 0..WIDTH {
            let sx = (2.0*(x as f32+0.5)/WIDTH as f32-1.0)*WIDTH as f32/HEIGHT as f32*scale;
            let sy = (1.0-2.0*(y as f32+0.5)/HEIGHT as f32)*scale;
            let direction = (forward+right*sx+up*sy).normalize();
            let c = if let Some((p,m)) = trace(eye,direction) {
                let n = normal(p);
                let l = (light-p).normalize();
                let diffuse = dot(&n,&l).max(0.0);
                let visibility = shadow(p+n*0.015,light);
                let half = (l-direction).normalize();
                let specular = dot(&n,&half).max(0.0).powf(48.0)*0.18*visibility;
                let albedo = match m {
                    0 => v(0.72,0.035,0.012), // Rojo del cuerpo y plumas.
                    1 => v(1.0,0.24,0.018), // Naranja.
                    2 => v(1.0,0.64,0.055), // Dorado de puntas y pico.
                    3 => v(0.15,0.10,0.085), // Patas y contorno de ojos.
                    4 => v(0.012,0.009,0.012), // Pupilas y garras.
                    5 => v(1.0,0.95,0.76), // Brillo de ojos.
                    6 => v(0.23,0.21,0.20), // Percha de piedra.
                    _ => v(0.075,0.10,0.13),
                };
                let ambient = if night {0.075} else {0.23};
                let power = if night {0.55} else {0.95};
                albedo*(ambient+power*diffuse*visibility)+v(specular,specular,specular)
            } else if night { v(0.006,0.012,0.035) } else { v(0.14,0.20,0.26) };
            line[x] = pack(c);
        }
        }
        });
    }
    });
}

// Exportacion opcional sin abrir ventana: cargo run --release -- --preview
fn save_preview() {
    use std::io::Write;
    let mut buffer = vec![0;WIDTH*HEIGHT];
    render(&mut buffer,0.35,0.16,false);
    let mut file = std::fs::File::create("fenix.ppm").expect("No se pudo guardar la imagen");
    write!(file,"P6\n{} {}\n255\n",WIDTH,HEIGHT).unwrap();
    for pixel in buffer {
        file.write_all(&[(pixel>>16) as u8,(pixel>>8) as u8,pixel as u8]).unwrap();
    }
    println!("Imagen guardada en fenix.ppm");
}

fn main() {
    if std::env::args().any(|arg| arg == "--preview") { save_preview(); return; }
    let mut window = Window::new("Fenix | Flechas: girar | N: noche | R: reiniciar | Esc: salir",WIDTH,HEIGHT,WindowOptions::default())
        .expect("No se pudo crear la ventana");
    let mut buffer = vec![0;WIDTH*HEIGHT];
    let mut yaw: f32 = 0.35;
    let mut elevation: f32 = 0.16;
    let mut night = false;
    let mut dirty = true;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::N,KeyRepeat::No) { night = !night; dirty = true; }
        if window.is_key_pressed(Key::R,KeyRepeat::No) {
            yaw = 0.35; elevation = 0.16; night = false; dirty = true;
        }
        for (key,dy,de) in [(Key::Left,0.08,0.0),(Key::Right,-0.08,0.0),
                            (Key::Up,0.0,0.06),(Key::Down,0.0,-0.06)] {
            if window.is_key_down(key) { yaw += dy; elevation = (elevation+de).clamp(-0.05,1.30); dirty = true; }
        }
        if dirty { render(&mut buffer,yaw,elevation,night); dirty = false; }
        window.update_with_buffer(&buffer,WIDTH,HEIGHT).expect("No se pudo actualizar la ventana");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
