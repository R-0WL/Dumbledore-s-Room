// Dumbledore simplificado, basado en la estructura del archivo modeladors(1).rs.
// Reemplaza src/main.rs; conserva minifb y nalgebra-glm en Cargo.toml.
// Geometria procedural: no necesita scene.bin, modelos ni texturas externas.
// Frente: +Z. Vertical: +Y. Suelo: Y=0. Altura aproximada: 5.05 unidades.
// Ejecutar con cargo run --release (el ray marching es costoso en modo debug).
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nalgebra_glm::{cross, dot, Vec3};
use std::f32::consts::PI;

const WIDTH: usize = 600;
const HEIGHT: usize = 640;
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

// Perfil cerrado de la tunica; Y vertical, personaje mirando hacia +Z.
fn robe(p: Vec3) -> f32 {
    const PROFILE: [(f32,f32); 9] = [
        (0.17,0.0),(0.17,0.83),(0.28,0.84),(0.70,0.77),
        (1.50,0.63),(2.35,0.46),(2.85,0.40),(3.12,0.25),(3.12,0.0),
    ];
    // Seccion eliptica: mas estrecha de frente a espalda.
    let x = (p.x*p.x+(p.z/0.85).powi(2)).sqrt();
    let y = p.y;
    let mut distance2 = f32::INFINITY;
    let mut inside = false;
    for i in 0..PROFILE.len() {
        let (ay,ax) = PROFILE[i];
        let (by,bx) = PROFILE[(i+1)%PROFILE.len()];
        let dx = bx-ax;
        let dy = by-ay;
        let t = (((x-ax)*dx+(y-ay)*dy)/(dx*dx+dy*dy)).clamp(0.0,1.0);
        distance2 = distance2.min((x-ax-t*dx).powi(2)+(y-ay-t*dy).powi(2));
        if (ay>y)!=(by>y) && x<ax+(y-ay)*dx/dy { inside = !inside; }
    }
    distance2.sqrt()*0.85*if inside {-1.0} else {1.0}
}

fn objects(p: Vec3, mut hit: (f32, usize)) -> (f32, usize) {
    // Volumenes elipsoidales (centro, radios, giro en XY, material).
    choose(&mut hit, ellipsoid(p,v(0.0,2.85,0.0),v(0.49,0.43,0.36),0.0),0);
    choose(&mut hit, ellipsoid(p,v(-0.65,2.75,0.0),v(0.28,0.55,0.3),-0.42),0);
    choose(&mut hit, ellipsoid(p,v(-0.85,2.4,0.1),v(0.26,0.38,0.29),-0.2),0);
    choose(&mut hit, ellipsoid(p,v(-0.91,2.17,0.15),v(0.125,0.19,0.125),0.0),2);
    choose(&mut hit, ellipsoid(p,v(-0.89,2.32,0.13),v(0.22,0.075,0.26),0.0),1);
    choose(&mut hit, ellipsoid(p,v(-0.3,0.12,0.12),v(0.21,0.12,0.31),0.0),7);
    choose(&mut hit, ellipsoid(p,v(0.65,2.75,0.0),v(0.28,0.55,0.3),0.42),0);
    choose(&mut hit, ellipsoid(p,v(0.85,2.4,0.1),v(0.26,0.38,0.29),0.2),0);
    choose(&mut hit, ellipsoid(p,v(0.91,2.17,0.15),v(0.125,0.19,0.125),0.0),2);
    choose(&mut hit, ellipsoid(p,v(0.89,2.32,0.13),v(0.22,0.075,0.26),0.0),1);
    choose(&mut hit, ellipsoid(p,v(0.3,0.12,0.12),v(0.21,0.12,0.31),0.0),7);
    choose(&mut hit, ellipsoid(p,v(0.0,3.65,0.0),v(0.21,0.29,0.22),0.0),2);
    choose(&mut hit, ellipsoid(p,v(0.0,4.1,0.0),v(0.365,0.49,0.315),0.0),2);
    choose(&mut hit, ellipsoid(p,v(-0.365,4.08,0.0),v(0.085,0.15,0.09),0.0),2);
    choose(&mut hit, ellipsoid(p,v(-0.31,3.96,-0.09),v(0.16,0.63,0.3),0.1),3);
    choose(&mut hit, ellipsoid(p,v(-0.37,3.5,-0.06),v(0.125,0.36,0.22),-0.12),3);
    choose(&mut hit, ellipsoid(p,v(0.365,4.08,0.0),v(0.085,0.15,0.09),0.0),2);
    choose(&mut hit, ellipsoid(p,v(0.31,3.96,-0.09),v(0.16,0.63,0.3),-0.1),3);
    choose(&mut hit, ellipsoid(p,v(0.37,3.5,-0.06),v(0.125,0.36,0.22),0.12),3);
    choose(&mut hit, ellipsoid(p,v(0.0,4.2,-0.23),v(0.33,0.4,0.15),0.0),3);
    choose(&mut hit, ellipsoid(p,v(-0.145,4.205,0.288),v(0.087,0.054,0.038),0.0),5);
    choose(&mut hit, ellipsoid(p,v(-0.145,4.205,0.323),v(0.031,0.034,0.012),0.0),6);
    choose(&mut hit, ellipsoid(p,v(-0.145,4.205,0.334),v(0.014,0.022,0.006),0.0),7);
    choose(&mut hit, ellipsoid(p,v(-0.138,4.305,0.305),v(0.115,0.029,0.033),-0.1),3);
    choose(&mut hit, ellipsoid(p,v(0.145,4.205,0.288),v(0.087,0.054,0.038),0.0),5);
    choose(&mut hit, ellipsoid(p,v(0.145,4.205,0.323),v(0.031,0.034,0.012),0.0),6);
    choose(&mut hit, ellipsoid(p,v(0.145,4.205,0.334),v(0.014,0.022,0.006),0.0),7);
    choose(&mut hit, ellipsoid(p,v(0.138,4.305,0.305),v(0.115,0.029,0.033),0.1),3);
    choose(&mut hit, ellipsoid(p,v(0.0,4.105,0.335),v(0.065,0.16,0.09),0.0),2);
    choose(&mut hit, ellipsoid(p,v(0.0,4.015,0.408),v(0.085,0.075,0.075),0.0),2);
    choose(&mut hit, ellipsoid(p,v(0.0,3.905,0.301),v(0.11,0.022,0.032),0.0),7);
    choose(&mut hit, ellipsoid(p,v(0.0,3.54,0.305),v(0.29,0.47,0.21),0.0),3);
    choose(&mut hit, ellipsoid(p,v(0.0,3.2,0.36),v(0.18,0.37,0.13),-0.06),3);
    choose(&mut hit, ellipsoid(p,v(-0.18,3.63,0.36),v(0.135,0.35,0.15),0.17),3);
    choose(&mut hit, ellipsoid(p,v(-0.1,3.925,0.361),v(0.15,0.052,0.055),0.2),3);
    choose(&mut hit, ellipsoid(p,v(-0.095,3.36,0.474),v(0.037,0.31,0.018),-0.12),9);
    choose(&mut hit, ellipsoid(p,v(0.18,3.63,0.36),v(0.135,0.35,0.15),-0.17),3);
    choose(&mut hit, ellipsoid(p,v(0.1,3.925,0.361),v(0.15,0.052,0.055),-0.2),3);
    choose(&mut hit, ellipsoid(p,v(0.095,3.36,0.474),v(0.037,0.31,0.018),0.12),9);
    choose(&mut hit, ellipsoid(p,v(0.0,3.53,0.503),v(0.028,0.34,0.016),0.04),9);
    choose(&mut hit, ellipsoid(p,v(0.0,4.535,-0.015),v(0.405,0.085,0.345),0.0),0);
    choose(&mut hit, ellipsoid(p,v(-0.025,4.71,-0.035),v(0.335,0.25,0.295),-0.15),0);
    choose(&mut hit, ellipsoid(p,v(-0.105,4.89,-0.035),v(0.215,0.18,0.205),-0.36),0);
    choose(&mut hit, ellipsoid(p,v(0.0,2.7,0.385),v(0.032,0.032,0.025),0.0),4);
    choose(&mut hit, ellipsoid(p,v(0.0,2.48,0.385),v(0.032,0.032,0.025),0.0),4);
    choose(&mut hit, ellipsoid(p,v(0.0,2.26,0.385),v(0.032,0.032,0.025),0.0),4);
    choose(&mut hit, ellipsoid(p,v(1.1,3.51,0.21),v(0.065,0.12,0.065),0.0),8);
    // Segmentos redondeados: lentes, bordes y baston.
    choose(&mut hit, capsule(p,v(0.374,4.566,-0.015),v(0.361256,4.566,0.066528),0.014),4);
    choose(&mut hit, capsule(p,v(0.361256,4.566,0.066528),v(0.323894,4.566,0.1425),0.014),4);
    choose(&mut hit, capsule(p,v(0.323894,4.566,0.1425),v(0.264458,4.566,0.207739),0.014),4);
    choose(&mut hit, capsule(p,v(0.264458,4.566,0.207739),v(0.187,4.566,0.257798),0.014),4);
    choose(&mut hit, capsule(p,v(0.187,4.566,0.257798),v(0.096798,4.566,0.289267),0.014),4);
    choose(&mut hit, capsule(p,v(0.096798,4.566,0.289267),v(0.0,4.566,0.3),0.014),4);
    choose(&mut hit, capsule(p,v(0.0,4.566,0.3),v(-0.096798,4.566,0.289267),0.014),4);
    choose(&mut hit, capsule(p,v(-0.096798,4.566,0.289267),v(-0.187,4.566,0.257798),0.014),4);
    choose(&mut hit, capsule(p,v(-0.187,4.566,0.257798),v(-0.264458,4.566,0.207739),0.014),4);
    choose(&mut hit, capsule(p,v(-0.264458,4.566,0.207739),v(-0.323894,4.566,0.1425),0.014),4);
    choose(&mut hit, capsule(p,v(-0.323894,4.566,0.1425),v(-0.361256,4.566,0.066528),0.014),4);
    choose(&mut hit, capsule(p,v(-0.361256,4.566,0.066528),v(-0.374,4.566,-0.015),0.014),4);
    choose(&mut hit, capsule(p,v(-0.374,4.566,-0.015),v(-0.361256,4.566,-0.096528),0.014),4);
    choose(&mut hit, capsule(p,v(-0.361256,4.566,-0.096528),v(-0.323894,4.566,-0.1725),0.014),4);
    choose(&mut hit, capsule(p,v(-0.323894,4.566,-0.1725),v(-0.264458,4.566,-0.237739),0.014),4);
    choose(&mut hit, capsule(p,v(-0.264458,4.566,-0.237739),v(-0.187,4.566,-0.287798),0.014),4);
    choose(&mut hit, capsule(p,v(-0.187,4.566,-0.287798),v(-0.096798,4.566,-0.319267),0.014),4);
    choose(&mut hit, capsule(p,v(-0.096798,4.566,-0.319267),v(-0.0,4.566,-0.33),0.014),4);
    choose(&mut hit, capsule(p,v(-0.0,4.566,-0.33),v(0.096798,4.566,-0.319267),0.014),4);
    choose(&mut hit, capsule(p,v(0.096798,4.566,-0.319267),v(0.187,4.566,-0.287798),0.014),4);
    choose(&mut hit, capsule(p,v(0.187,4.566,-0.287798),v(0.264458,4.566,-0.237739),0.014),4);
    choose(&mut hit, capsule(p,v(0.264458,4.566,-0.237739),v(0.323894,4.566,-0.1725),0.014),4);
    choose(&mut hit, capsule(p,v(0.323894,4.566,-0.1725),v(0.361256,4.566,-0.096528),0.014),4);
    choose(&mut hit, capsule(p,v(0.361256,4.566,-0.096528),v(0.374,4.566,-0.015),0.014),4);
    choose(&mut hit, capsule(p,v(-0.256,4.18,0.371),v(-0.250812,4.158987,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(-0.250812,4.158987,0.371),v(-0.235756,4.140031,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(-0.235756,4.140031,0.371),v(-0.212305,4.124987,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(-0.212305,4.124987,0.371),v(-0.182756,4.115328,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(-0.182756,4.115328,0.371),v(-0.15,4.112,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(-0.15,4.112,0.371),v(-0.117244,4.115328,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(-0.117244,4.115328,0.371),v(-0.087695,4.124987,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(-0.087695,4.124987,0.371),v(-0.064244,4.140031,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(-0.064244,4.140031,0.371),v(-0.049188,4.158987,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(-0.049188,4.158987,0.371),v(-0.044,4.18,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(-0.256,4.18,0.371),v(-0.044,4.18,0.371),0.007),4);
    choose(&mut hit, capsule(p,v(-0.256,4.18,0.371),v(-0.344,4.18,0.1),0.008),4);
    choose(&mut hit, capsule(p,v(0.044,4.18,0.371),v(0.049188,4.158987,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(0.049188,4.158987,0.371),v(0.064244,4.140031,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(0.064244,4.140031,0.371),v(0.087695,4.124987,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(0.087695,4.124987,0.371),v(0.117244,4.115328,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(0.117244,4.115328,0.371),v(0.15,4.112,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(0.15,4.112,0.371),v(0.182756,4.115328,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(0.182756,4.115328,0.371),v(0.212305,4.124987,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(0.212305,4.124987,0.371),v(0.235756,4.140031,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(0.235756,4.140031,0.371),v(0.250812,4.158987,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(0.250812,4.158987,0.371),v(0.256,4.18,0.371),0.009),4);
    choose(&mut hit, capsule(p,v(0.044,4.18,0.371),v(0.256,4.18,0.371),0.007),4);
    choose(&mut hit, capsule(p,v(0.256,4.18,0.371),v(0.344,4.18,0.1),0.008),4);
    choose(&mut hit, capsule(p,v(-0.043,4.18,0.371),v(0.0,4.205,0.39),0.008),4);
    choose(&mut hit, capsule(p,v(0.0,4.205,0.39),v(0.043,4.18,0.371),0.008),4);
    choose(&mut hit, capsule(p,v(-0.18,2.89,0.33),v(-0.25,1.68,0.54),0.018),1);
    choose(&mut hit, capsule(p,v(-0.25,1.68,0.54),v(-0.38,0.22,0.707),0.018),1);
    choose(&mut hit, capsule(p,v(0.18,2.89,0.33),v(0.25,1.68,0.54),0.018),1);
    choose(&mut hit, capsule(p,v(0.25,1.68,0.54),v(0.38,0.22,0.707),0.018),1);
    choose(&mut hit, capsule(p,v(1.08,0.12,0.22),v(1.04,2.18,0.22),0.036),8);
    choose(&mut hit, capsule(p,v(1.04,2.18,0.22),v(1.1,3.48,0.21),0.033),8);
    choose(&mut hit, capsule(p,v(0.91,2.18,0.18),v(1.06,2.19,0.23),0.075),2);
    hit
}

fn scene(p: Vec3) -> (f32,usize) {
    let mut hit = (p.y,10); // Piso en Y=0.
    // Caja envolvente conservadora para evitar evaluar detalles lejanos.
    let q = (p-v(0.08,2.54,0.05)).map(|x| x.abs())-v(1.25,2.54,0.78);
    let bound = q.map(|x| x.max(0.0)).norm()+q.x.max(q.y).max(q.z).min(0.0);
    if bound>0.015 { choose(&mut hit,bound,0); return hit; }
    choose(&mut hit,robe(p),0);
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
    let target = v(0.0,2.55,0.0);
    let eye = target + v(yaw.sin()*elevation.cos(), elevation.sin(), yaw.cos()*elevation.cos())*6.4;
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
                let mut albedo = match m {
                    0 => v(0.22,0.16,0.29), // Tunica y gorro violeta grisaceo.
                    1 => v(0.52,0.35,0.17), // Ribetes de tela.
                    2 => v(0.72,0.43,0.29), // Piel.
                    3 => v(0.80,0.80,0.75), // Cabello, cejas y barba.
                    4 => v(0.72,0.48,0.16), // Metal dorado de los lentes.
                    5 => v(0.90,0.92,0.89), // Blanco de ojos.
                    6 => v(0.12,0.39,0.58), // Iris azul.
                    7 => v(0.025,0.020,0.028),
                    8 => v(0.22,0.10,0.045), // Madera.
                    9 => v(0.66,0.68,0.65), // Mechones de la barba.
                    _ => v(0.11,0.15,0.18),
                };
                if m == 0 {
                    // Pliegues sugeridos por color, sin texturas externas.
                    let fold = (p.x.atan2(p.z/0.85)*22.0+p.y*0.8).cos();
                    albedo *= 0.93+0.07*fold;
                }
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
    render(&mut buffer,0.12,0.08,false);
    let mut file = std::fs::File::create("dumbledore.ppm").expect("No se pudo guardar la imagen");
    write!(file,"P6\n{} {}\n255\n",WIDTH,HEIGHT).unwrap();
    for pixel in buffer {
        file.write_all(&[(pixel>>16) as u8,(pixel>>8) as u8,pixel as u8]).unwrap();
    }
    println!("Imagen guardada en dumbledore.ppm");
}

fn main() {
    if std::env::args().any(|arg| arg == "--preview") { save_preview(); return; }
    let mut window = Window::new("Dumbledore | Flechas: girar | N: noche | R: reiniciar | Esc: salir",WIDTH,HEIGHT,WindowOptions::default())
        .expect("No se pudo crear la ventana");
    let mut buffer = vec![0;WIDTH*HEIGHT];
    let mut yaw: f32 = 0.12;
    let mut elevation: f32 = 0.08;
    let mut night = false;
    let mut dirty = true;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::N,KeyRepeat::No) { night = !night; dirty = true; }
        if window.is_key_pressed(Key::R,KeyRepeat::No) {
            yaw = 0.12; elevation = 0.08; night = false; dirty = true;
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
