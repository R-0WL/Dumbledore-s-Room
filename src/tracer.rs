// Trazador de rayos tipo Whitted: sombras, reflexion, refraccion (Snell + Fresnel), emision y skybox.
use crate::camera::Camera;
use crate::material::*;
use crate::sdf::*;
use crate::sky::*;
use crate::texture::Texture;
use crate::util::*;
use crate::voxel::Grid;
use nalgebra_glm::Vec3;

pub const MAX_DEPTH: u32 = 3;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub color: Vec3,
    pub albedo: [f32; 4],
    pub shininess: f32,
    pub ior: f32,
    pub emission: Vec3,
    pub tint: Vec3,
}

impl Sphere {
    fn hit(&self, o: Vec3, d: Vec3) -> Option<f32> {
        let oc = o - self.center;
        let b = oc.dot(&d);
        let c = oc.dot(&oc) - self.radius * self.radius;
        let disc = b * b - c;
        if disc < 0.0 {
            return None;
        }
        let s = disc.sqrt();
        let t0 = -b - s;
        if t0 > 1e-3 {
            return Some(t0);
        }
        let t1 = -b + s;
        if t1 > 1e-3 {
            return Some(t1);
        }
        None
    }
}

pub struct Light {
    pub pos: Vec3,
    pub col: Vec3,
    pub k: f32,
    pub shadow: bool,
    pub ignore_char: Option<usize>,
}

pub struct Ctx<'a> {
    pub grid: &'a Grid,
    pub mats: &'a [VoxMat],
    pub texs: &'a [Texture],
    pub spheres: &'a [Sphere],
    pub chars: &'a [Character],
    pub lights: &'a [Light],
    pub env: &'a Env,
    pub center: Vec3,
}

#[derive(Clone, Copy)]
enum HitObj {
    Voxel { cell: (i32, i32, i32), mat: u8 },
    Sphere(usize),
    Char(usize, usize, Vec3),
}

#[derive(Clone, Copy)]
struct Hit {
    t: f32,
    p: Vec3,
    n: Vec3,
    obj: HitObj,
}

struct Surf {
    color: Vec3,
    diff: f32,
    spec: f32,
    shin: f32,
    refl: f32,
    transp: f32,
    ior: f32,
    emit: Vec3,
    tint: Vec3,
    rtint: Vec3,
}

fn surf_default(color: Vec3) -> Surf {
    Surf {
        color,
        diff: 1.0,
        spec: 0.18,
        shin: 48.0,
        refl: 0.0,
        transp: 0.0,
        ior: 1.0,
        emit: v(0.0, 0.0, 0.0),
        tint: v(1.0, 1.0, 1.0),
        rtint: v(1.0, 1.0, 1.0),
    }
}

const DUMB_PAL: [(f32, f32, f32); 10] = [
    (0.22, 0.16, 0.29),
    (0.52, 0.35, 0.17),
    (0.72, 0.43, 0.29),
    (0.80, 0.80, 0.75),
    (0.72, 0.48, 0.16),
    (0.90, 0.92, 0.89),
    (0.12, 0.39, 0.58),
    (0.025, 0.020, 0.028),
    (0.22, 0.10, 0.045),
    (0.66, 0.68, 0.65),
];
const FAWKES_PAL: [(f32, f32, f32); 7] = [
    (0.72, 0.035, 0.012),
    (1.0, 0.24, 0.018),
    (1.0, 0.64, 0.055),
    (0.15, 0.10, 0.085),
    (0.012, 0.009, 0.012),
    (1.0, 0.95, 0.76),
    (0.75, 0.58, 0.22),
];
// Cuanta luz propia emite cada material de Fawkes (multiplica su color).
const FAWKES_GLOW: [f32; 7] = [1.5, 2.0, 2.4, 0.2, 0.0, 1.2, 0.0];
const HARRY_PAL: [(f32, f32, f32); 14] = [
    (0.47, 0.52, 0.55),
    (0.72, 0.76, 0.75),
    (0.28, 0.30, 0.32),
    (0.64, 0.54, 0.35),
    (0.95, 0.57, 0.08),
    (0.018, 0.020, 0.023),
    (0.74, 0.47, 0.31),
    (0.045, 0.052, 0.065),
    (0.81, 0.81, 0.74),
    (0.46, 0.025, 0.035),
    (0.85, 0.56, 0.10),
    (0.16, 0.18, 0.21),
    (0.055, 0.039, 0.030),
    (0.93, 0.95, 0.94),
];

impl<'a> Ctx<'a> {
    fn is_interior(&self, p: Vec3) -> bool {
        let dx = p.x - self.center.x;
        let dz = p.z - self.center.z;
        (dx * dx + dz * dz).sqrt() < 4.55 && p.y > 7.45 && p.y < 14.0
    }

    fn closest(&self, o: Vec3, d: Vec3, tmax: f32) -> Option<Hit> {
        let mut best: Option<Hit> = None;
        let mut bt = tmax;
        if let Some(vh) = self.grid.trace(o, d, bt) {
            bt = vh.t;
            best = Some(Hit {
                t: vh.t,
                p: o + d * vh.t,
                n: vh.normal,
                obj: HitObj::Voxel { cell: vh.cell, mat: vh.mat },
            });
        }
        for (i, s) in self.spheres.iter().enumerate() {
            if let Some(t) = s.hit(o, d) {
                if t < bt {
                    bt = t;
                    let p = o + d * t;
                    best = Some(Hit { t, p, n: (p - s.center) / s.radius, obj: HitObj::Sphere(i) });
                }
            }
        }
        for (i, c) in self.chars.iter().enumerate() {
            if let Some(ch) = c.intersect(o, d, bt, 160) {
                if ch.t < bt {
                    bt = ch.t;
                    let n = c.normal_world(ch.p);
                    best = Some(Hit { t: ch.t, p: o + d * ch.t, n, obj: HitObj::Char(i, ch.mat, ch.p) });
                }
            }
        }
        best
    }

    fn visibility(&self, o: Vec3, d: Vec3, dist: f32, ignore: Option<usize>) -> f32 {
        let mut vis = self.grid.shadow(o, d, dist);
        if vis <= 0.001 {
            return 0.0;
        }
        for s in self.spheres.iter() {
            if let Some(t) = s.hit(o, d) {
                if t < dist {
                    vis *= if s.albedo[3] > 0.3 { 0.7 } else { 0.0 };
                }
            }
        }
        for (i, c) in self.chars.iter().enumerate() {
            if Some(i) == ignore {
                continue;
            }
            if c.intersect(o, d, dist, 64).is_some() {
                return 0.0;
            }
        }
        vis
    }

    fn voxel_surface(&self, hit: &Hit, cell: (i32, i32, i32), mat: u8) -> Surf {
        let m = &self.mats[mat as usize];
        let cs = self.grid.cell;
        let lp = hit.p / cs;
        let fx = lp.x - lp.x.floor();
        let fy = lp.y - lp.y.floor();
        let fz = lp.z - lp.z.floor();
        let n = hit.n;
        let (u, vv) = if n.y.abs() > 0.5 {
            (fx, fz)
        } else if n.x.abs() > 0.5 {
            (if n.x > 0.0 { 1.0 - fz } else { fz }, 1.0 - fy)
        } else {
            (if n.z > 0.0 { fx } else { 1.0 - fx }, 1.0 - fy)
        };
        let mut ti = if n.y > 0.5 { m.tex_top } else { m.tex };
        if m.variants > 1 {
            let k = ((hash3(cell.0, cell.1, cell.2 + 7) * m.variants as f32) as usize).min(m.variants - 1);
            ti = m.tex + k;
        }
        let (texel, alpha) = self.texs[ti].sample(u, vv);
        let hh = hash3(cell.0, cell.1, cell.2);
        let var = 0.92 + 0.16 * hh;
        let env = self.env;

        let mut s = surf_default(texel * var);
        s.diff = m.albedo[0];
        s.spec = m.albedo[1];
        s.shin = m.shininess;
        s.refl = m.albedo[2];
        s.transp = m.albedo[3] * alpha;
        s.ior = m.ior;
        s.tint = m.tint;
        if m.emission > 0.0 {
            let flick = 0.85 + 0.15 * (env.t * 9.0 + hh * 30.0).sin();
            s.emit = texel * (m.emission * flick);
        }
        if mat == M_GOLD || mat == M_SILVER {
            s.rtint = texel;
        }
        if mat == M_SLATE {
            s.refl += 0.35 * env.rain;
            s.spec += 0.4 * env.rain;
        }
        if mat == M_GLASS && env.rain > 0.05 {
            let col = (u * 32.0) as i32;
            let seed = hash3(cell.0 * 7 + col, cell.2 * 5 + (n.x * 3.0) as i32, 11);
            let yy = vv - env.t * 0.15 * (0.5 + seed) + seed;
            let ph = yy - yy.floor();
            let drop = if ph < 0.10 && hash3(col, cell.0 + cell.2, 5) < env.rain { 1.0 } else { 0.0 };
            s.transp *= 1.0 - 0.45 * drop;
            s.refl += 0.08 * env.rain + 0.2 * drop;
            s.color = s.color + v(0.5, 0.55, 0.6) * (0.3 * drop);
        }
        if mat == M_WATER {
            s.emit = s.emit * (0.8 + 0.2 * (env.t * 2.0 + hh * 6.0).sin());
        }
        s
    }

    fn char_surface(&self, ci: usize, m: usize, pl: Vec3) -> Surf {
        let ch = &self.chars[ci];
        let t = self.env.t;
        match ch.kind {
            CharKind::Dumbledore => {
                let c = DUMB_PAL[m.min(9)];
                let mut col = v(c.0, c.1, c.2);
                if m == 0 {
                    let fold = (pl.x.atan2(pl.z / 0.85) * 22.0 + pl.y * 0.8).cos();
                    col = col * (0.93 + 0.07 * fold);
                }
                let mut s = surf_default(col);
                if m == 4 {
                    s.spec = 0.9;
                    s.shin = 90.0;
                    s.refl = 0.18;
                    s.rtint = col;
                }
                s
            }
            CharKind::Fawkes => {
                let mi = m.min(6);
                let c = FAWKES_PAL[mi];
                let col = v(c.0, c.1, c.2);
                let mut s = surf_default(col);
                let pulse = 1.0 + 0.18 * (t * 3.1).sin() + 0.08 * (t * 7.7 + pl.y).sin();
                s.emit = col * (FAWKES_GLOW[mi] * pulse);
                if mi == 6 {
                    s.spec = 0.8;
                    s.shin = 80.0;
                    s.refl = 0.3;
                    s.rtint = col;
                }
                s
            }
            CharKind::Harry => {
                let c = HARRY_PAL[m.min(13)];
                let col = v(c.0, c.1, c.2);
                let mut s = surf_default(col);
                s.emit = col * 0.10;
                s
            }
        }
    }

    fn surface(&self, hit: &Hit) -> Surf {
        match hit.obj {
            HitObj::Voxel { cell, mat } => self.voxel_surface(hit, cell, mat),
            HitObj::Sphere(i) => {
                let sp = &self.spheres[i];
                let mut s = surf_default(sp.color);
                s.diff = sp.albedo[0];
                s.spec = sp.albedo[1];
                s.refl = sp.albedo[2];
                s.transp = sp.albedo[3];
                s.shin = sp.shininess;
                s.ior = sp.ior;
                s.emit = sp.emission;
                s.tint = sp.tint;
                s
            }
            HitObj::Char(i, m, pl) => self.char_surface(i, m, pl),
        }
    }

    fn local_shading(&self, p: Vec3, n: Vec3, d: Vec3, s: &Surf) -> Vec3 {
        let env = self.env;
        let interior = self.is_interior(p);
        let hemi = 0.62 + 0.38 * n.y.max(-0.6).min(1.0);
        let amb = if interior { env.ambient * 0.28 + v(0.060, 0.045, 0.035) } else { env.ambient };
        let mut diffuse = amb * hemi;
        let mut specular = v(0.0, 0.0, 0.0);
        let org = p + n * 0.012;

        // Sol / luna
        let kd = env.key_dir;
        let ndl = n.dot(&kd);
        if ndl > 0.0 && env.key_col.norm() > 0.01 {
            let vis = self.visibility(org, kd, 120.0, None);
            if vis > 0.0 {
                diffuse = diffuse + env.key_col * (ndl * vis);
                let h = (kd - d).normalize();
                specular = specular + env.key_col * (n.dot(&h).max(0.0).powf(s.shin) * vis);
            }
        }
        // Luces puntuales (Fawkes, chimenea, candelabro, pensadero...)
        for l in self.lights.iter() {
            let to = l.pos - p;
            let d2 = to.dot(&to);
            let dist = d2.sqrt();
            if dist < 1e-3 {
                continue;
            }
            let ld = to / dist;
            let ndl = n.dot(&ld);
            if ndl <= 0.0 {
                continue;
            }
            let att = 1.0 / (1.0 + l.k * d2);
            let mut vis = 1.0;
            if l.shadow {
                vis = self.visibility(org, ld, dist - 0.03, l.ignore_char);
                if vis <= 0.0 {
                    continue;
                }
            }
            let c = l.col * (att * vis);
            diffuse = diffuse + c * ndl;
            let h = (ld - d).normalize();
            specular = specular + c * n.dot(&h).max(0.0).powf(s.shin);
        }
        s.color.component_mul(&diffuse) * s.diff + specular * s.spec + s.emit
    }

    fn refract_voxel(&self, p0: Vec3, cell0: (i32, i32, i32), mat: u8, s: &Surf, d: Vec3, n: Vec3, depth: u32) -> Vec3 {
        let cs = self.grid.cell;
        let mut dir = match refract(d, n, 1.0 / s.ior) {
            Some(r) => r,
            None => return self.trace(p0 + n * 0.012, reflect(d, n), depth + 1),
        };
        let mut p = p0;
        let mut cell = [cell0.0, cell0.1, cell0.2];
        let mut out_dir = dir;
        let mut axis_out = 0usize;
        let mut step_out: i32 = 1;
        for _ in 0..8 {
            let mut t_exit = f32::INFINITY;
            let mut axis = 0usize;
            for a in 0..3 {
                if dir[a].abs() < 1e-7 {
                    continue;
                }
                let bound = if dir[a] > 0.0 { (cell[a] + 1) as f32 * cs } else { cell[a] as f32 * cs };
                let t = (bound - p[a]) / dir[a];
                if t < t_exit {
                    t_exit = t;
                    axis = a;
                }
            }
            if !t_exit.is_finite() {
                break;
            }
            p = p + dir * t_exit.max(0.0);
            let step: i32 = if dir[axis] > 0.0 { 1 } else { -1 };
            let mut next = cell;
            next[axis] += step;
            axis_out = axis;
            step_out = step;
            if self.grid.get(next[0], next[1], next[2]) == mat {
                cell = next;
                continue;
            }
            let mut n_out = v(0.0, 0.0, 0.0);
            n_out[axis] = step as f32;
            match refract(dir, -n_out, s.ior) {
                Some(r) => {
                    out_dir = r;
                    break;
                }
                None => {
                    dir = reflect(dir, n_out);
                }
            }
        }
        let mut po = p;
        po[axis_out] += step_out as f32 * 0.004;
        self.trace(po, out_dir, depth + 1)
    }

    fn refract_sphere(&self, i: usize, p0: Vec3, s: &Surf, d: Vec3, n: Vec3, depth: u32) -> Vec3 {
        let sp = &self.spheres[i];
        let dir = match refract(d, n, 1.0 / s.ior) {
            Some(r) => r,
            None => return self.trace(p0 + n * 0.012, reflect(d, n), depth + 1),
        };
        let q = p0 - sp.center;
        let b = dir.dot(&q);
        let c = q.dot(&q) - sp.radius * sp.radius;
        let disc = (b * b - c).max(0.0);
        let t = -b + disc.sqrt();
        let pe = p0 + dir * t;
        let ne = (pe - sp.center) / sp.radius;
        let out = match refract(dir, -ne, s.ior) {
            Some(r) => r,
            None => dir,
        };
        self.trace(pe + ne * 0.005, out, depth + 1)
    }

    pub fn trace(&self, o: Vec3, d: Vec3, depth: u32) -> Vec3 {
        let hit = match self.closest(o, d, 400.0) {
            Some(h) => h,
            None => return sky_color(self.env, d),
        };
        let surf = self.surface(&hit);
        let mut n = hit.n;
        let inside = d.dot(&n) > 0.0;
        if inside {
            n = -n;
        }
        let p = hit.p;
        let local = self.local_shading(p, n, d, &surf);

        let refl = surf.refl;
        let transp = surf.transp;
        if depth >= MAX_DEPTH || (refl < 0.01 && transp < 0.01) {
            return local;
        }
        let cos_i = (-d.dot(&n)).max(0.0);
        let mut r_w = refl;
        let mut t_w = 0.0;
        if transp > 0.01 {
            let fres = schlick(cos_i, surf.ior);
            r_w = (refl + (1.0 - refl) * fres).min(1.0);
            t_w = transp * (1.0 - r_w);
        }
        let l_w = (1.0 - r_w - t_w).max(0.0);
        let mut col = local * l_w;
        if r_w > 0.01 {
            let rd = reflect(d, n);
            let rc = self.trace(p + n * 0.012, rd, depth + 1);
            col = col + rc.component_mul(&surf.rtint) * r_w;
        }
        if t_w > 0.01 {
            let rc = if inside {
                self.trace(p + d * 0.02, d, depth + 1)
            } else {
                match hit.obj {
                    HitObj::Voxel { cell, mat } => self.refract_voxel(p, cell, mat, &surf, d, n, depth),
                    HitObj::Sphere(i) => self.refract_sphere(i, p, &surf, d, n, depth),
                    HitObj::Char(..) => self.trace(p + d * 0.02, d, depth + 1),
                }
            };
            col = col + rc.component_mul(&surf.tint) * t_w;
        }
        col
    }
}

pub fn pack(c: Vec3) -> u32 {
    let f = |x: f32| -> u32 {
        let t = 1.0 - (-1.25 * x.max(0.0)).exp();
        (255.0 * t.powf(0.4545)).min(255.0) as u32
    };
    (f(c.x) << 16) | (f(c.y) << 8) | f(c.z)
}

pub fn render(ctx: &Ctx, cam: &Camera, buf: &mut [u32], w: usize, h: usize) {
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let mut buckets: Vec<Vec<(usize, &mut [u32])>> = (0..threads).map(|_| Vec::new()).collect();
    for (y, row) in buf.chunks_mut(w).enumerate() {
        buckets[y % threads].push((y, row));
    }
    std::thread::scope(|sc| {
        for bucket in buckets {
            sc.spawn(move || {
                for (y, row) in bucket {
                    for x in 0..w {
                        let dir = cam.ray(x, y, w, h);
                        let c = ctx.trace(cam.eye, dir, 0);
                        row[x] = pack(c);
                    }
                }
            });
        }
    });
}
