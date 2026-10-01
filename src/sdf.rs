// Modelos procedurales (SDF) de Dumbledore, Fawkes y Harry+Buckbeak.
// La geometria sale de los main.rs originales (ver model_data.rs); aqui se evalua
// con las mismas formulas (elipsoide, capsula, pluma, tunica) y se coloca en la escena.
use crate::util::*;
use nalgebra_glm::Vec3;

pub enum PrimDef {
    E([f32; 3], [f32; 3], f32, u8),
    C([f32; 3], [f32; 3], f32, u8),
    P([f32; 3], [f32; 3], f32, f32, u8),
}

#[derive(Clone, Copy)]
pub enum Shape {
    Ellipsoid { c: Vec3, r: Vec3, s: f32, co: f32, rmin: f32 },
    Capsule { a: Vec3, ba: Vec3, inv_len2: f32, radius: f32 },
    Plume { c: Vec3, side: Vec3, axis: Vec3, back: Vec3, r: Vec3, rmin: f32 },
}

#[derive(Clone, Copy)]
pub struct Prim {
    pub shape: Shape,
    pub mat: usize,
    pub bc: Vec3,
    pub br: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum WingRule {
    None,
    Fawkes,
    Buckbeak,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CharKind {
    Dumbledore,
    Fawkes,
    Harry,
}

fn arr(a: &[f32; 3]) -> Vec3 {
    v(a[0], a[1], a[2])
}

// Rota (en el plano XY, alrededor del eje Z que pasa por `pivot`) los puntos de las alas.
fn flap_point(p: Vec3, pivot: Vec3, angle: f32) -> Vec3 {
    let (s, c) = angle.sin_cos();
    let dx = p.x - pivot.x;
    let dy = p.y - pivot.y;
    v(pivot.x + dx * c - dy * s, pivot.y + dx * s + dy * c, p.z)
}

fn is_wing(rule: WingRule, a: Vec3, b: Vec3) -> bool {
    let wide = a.x.abs().max(b.x.abs()) > 0.3;
    match rule {
        WingRule::None => false,
        WingRule::Fawkes => wide && a.y >= 2.0 && a.y <= 3.0,
        WingRule::Buckbeak => wide && a.y >= 2.3 && a.y <= 2.8,
    }
}

pub struct Model {
    pub prims: Vec<Prim>,
    pub robe: bool,
    pub bc: Vec3,
    pub bh: Vec3,
}

impl Model {
    pub fn build(
        defs: &[PrimDef],
        plume_ref_y: bool,
        wing: WingRule,
        flap: f32,
        pivot: Vec3,
        robe: bool,
        bc: Vec3,
        bh: Vec3,
    ) -> Model {
        let mut prims: Vec<Prim> = Vec::with_capacity(defs.len());
        for d in defs.iter() {
            match d {
                PrimDef::E(c, r, ang, m) => {
                    let (s, co) = ang.sin_cos();
                    let rv = arr(r);
                    let rmin = rv.x.min(rv.y).min(rv.z);
                    prims.push(Prim {
                        shape: Shape::Ellipsoid { c: arr(c), r: rv, s, co, rmin },
                        mat: *m as usize,
                        bc: arr(c),
                        br: rv.x.max(rv.y).max(rv.z),
                    });
                }
                PrimDef::C(a, b, rad, m) => {
                    let av = arr(a);
                    let bv = arr(b);
                    let ba = bv - av;
                    prims.push(Prim {
                        shape: Shape::Capsule { a: av, ba, inv_len2: 1.0 / ba.dot(&ba), radius: *rad },
                        mat: *m as usize,
                        bc: (av + bv) * 0.5,
                        br: ba.norm() * 0.5 + *rad,
                    });
                }
                PrimDef::P(a, b, w, t, m) => {
                    let mut av = arr(a);
                    let mut bv = arr(b);
                    if flap != 0.0 && is_wing(wing, av, bv) {
                        let sgn = if av.x + bv.x >= 0.0 { 1.0 } else { -1.0 };
                        av = flap_point(av, pivot, sgn * flap);
                        bv = flap_point(bv, pivot, sgn * flap);
                    }
                    let axis = (bv - av).normalize();
                    let reference = if plume_ref_y {
                        if axis.y.abs() < 0.95 { v(0.0, 1.0, 0.0) } else { v(0.0, 0.0, 1.0) }
                    } else if axis.z.abs() < 0.95 {
                        v(0.0, 0.0, 1.0)
                    } else {
                        v(0.0, 1.0, 0.0)
                    };
                    let side = axis.cross(&reference).normalize();
                    let back = side.cross(&axis);
                    let half = (bv - av).norm() * 0.5;
                    let rr = v(*w, half, *t);
                    let rmin = rr.x.min(rr.y).min(rr.z);
                    prims.push(Prim {
                        shape: Shape::Plume { c: (av + bv) * 0.5, side, axis, back, r: rr, rmin },
                        mat: *m as usize,
                        bc: (av + bv) * 0.5,
                        br: rr.x.max(rr.y).max(rr.z),
                    });
                }
            }
        }
        Model { prims, robe, bc, bh }
    }

    // Distancia (cota conservadora) y material mas cercano, en el espacio del modelo.
    pub fn dist(&self, p: Vec3) -> (f32, usize) {
        let mut best = (1.0e9f32, 0usize);
        if self.robe {
            let bd = (p - v(0.0, 1.56, 0.0)).norm() - 1.9;
            if bd < best.0 {
                let d = robe_dist(p);
                if d < best.0 {
                    best = (d, 0);
                }
            }
        }
        for pr in self.prims.iter() {
            let bd = (p - pr.bc).norm() - pr.br;
            if bd >= best.0 {
                continue;
            }
            let d = match pr.shape {
                Shape::Ellipsoid { c, r, s, co, rmin } => {
                    let q = p - c;
                    let local = v(co * q.x + s * q.y, -s * q.x + co * q.y, q.z);
                    (local.component_div(&r).norm() - 1.0) * rmin
                }
                Shape::Capsule { a, ba, inv_len2, radius } => {
                    let pa = p - a;
                    let h = (pa.dot(&ba) * inv_len2).max(0.0).min(1.0);
                    (pa - ba * h).norm() - radius
                }
                Shape::Plume { c, side, axis, back, r, rmin } => {
                    let q = p - c;
                    let local = v(q.dot(&side), q.dot(&axis), q.dot(&back));
                    (local.component_div(&r).norm() - 1.0) * rmin
                }
            };
            if d < best.0 {
                best = (d, pr.mat);
            }
        }
        best
    }
}

// Perfil cerrado de la tunica (copiado del Dumbledore original).
fn robe_dist(p: Vec3) -> f32 {
    const PROFILE: [(f32, f32); 9] = [
        (0.17, 0.0),
        (0.17, 0.83),
        (0.28, 0.84),
        (0.70, 0.77),
        (1.50, 0.63),
        (2.35, 0.46),
        (2.85, 0.40),
        (3.12, 0.25),
        (3.12, 0.0),
    ];
    let x = (p.x * p.x + (p.z / 0.85).powi(2)).sqrt();
    let y = p.y;
    let mut distance2 = f32::INFINITY;
    let mut inside = false;
    for i in 0..PROFILE.len() {
        let (ay, ax) = PROFILE[i];
        let (by, bx) = PROFILE[(i + 1) % PROFILE.len()];
        let dx = bx - ax;
        let dy = by - ay;
        let t = (((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)).max(0.0).min(1.0);
        distance2 = distance2.min((x - ax - t * dx).powi(2) + (y - ay - t * dy).powi(2));
        if (ay > y) != (by > y) && x < ax + (y - ay) * dx / dy {
            inside = !inside;
        }
    }
    distance2.sqrt() * 0.85 * if inside { -1.0 } else { 1.0 }
}

pub struct CharHit {
    pub t: f32,
    pub p: Vec3,
    pub mat: usize,
}

pub struct Character {
    pub model: Model,
    pub kind: CharKind,
    pub pos: Vec3,
    pub scale: f32,
    pub s: f32,
    pub c: f32,
}

impl Character {
    pub fn new(model: Model, kind: CharKind, pos: Vec3, yaw: f32, scale: f32) -> Character {
        let (s, c) = yaw.sin_cos();
        Character { model, kind, pos, scale, s, c }
    }

    pub fn to_local_point(&self, p: Vec3) -> Vec3 {
        let q = p - self.pos;
        v(q.x * self.c - q.z * self.s, q.y, q.x * self.s + q.z * self.c) / self.scale
    }

    pub fn to_local_dir(&self, d: Vec3) -> Vec3 {
        v(d.x * self.c - d.z * self.s, d.y, d.x * self.s + d.z * self.c)
    }

    pub fn dir_to_world(&self, d: Vec3) -> Vec3 {
        v(d.x * self.c + d.z * self.s, d.y, -d.x * self.s + d.z * self.c)
    }

    pub fn to_world(&self, p: Vec3) -> Vec3 {
        self.dir_to_world(p) * self.scale + self.pos
    }

    fn aabb(&self, o: Vec3, d: Vec3) -> Option<(f32, f32)> {
        let mut t0 = f32::NEG_INFINITY;
        let mut t1 = f32::INFINITY;
        for a in 0..3 {
            let lo = self.model.bc[a] - self.model.bh[a];
            let hi = self.model.bc[a] + self.model.bh[a];
            if d[a].abs() < 1e-8 {
                if o[a] < lo || o[a] > hi {
                    return None;
                }
            } else {
                let inv = 1.0 / d[a];
                let mut ta = (lo - o[a]) * inv;
                let mut tb = (hi - o[a]) * inv;
                if ta > tb {
                    std::mem::swap(&mut ta, &mut tb);
                }
                t0 = t0.max(ta);
                t1 = t1.min(tb);
                if t0 > t1 {
                    return None;
                }
            }
        }
        if t1 < 0.0 {
            None
        } else {
            Some((t0.max(0.0), t1))
        }
    }

    // Ray marching dentro de la caja del modelo. Devuelve t en unidades del mundo.
    pub fn intersect(&self, o: Vec3, d: Vec3, tmax: f32, max_steps: usize) -> Option<CharHit> {
        let ol = self.to_local_point(o);
        let dl = self.to_local_dir(d);
        let (t0, t1) = self.aabb(ol, dl)?;
        let tmax_l = tmax / self.scale;
        if t0 > tmax_l {
            return None;
        }
        let t1 = t1.min(tmax_l);
        let mut t = t0;
        for _ in 0..max_steps {
            let p = ol + dl * t;
            let (dd, m) = self.model.dist(p);
            if dd < 0.003 {
                return Some(CharHit { t: t * self.scale, p, mat: m });
            }
            t += dd.max(0.002) * 0.9;
            if t > t1 {
                break;
            }
        }
        None
    }

    // Normal en el espacio del mundo (gradiente por tetraedro).
    pub fn normal_world(&self, p_local: Vec3) -> Vec3 {
        let e = 0.004;
        let k0 = v(1.0, -1.0, -1.0);
        let k1 = v(-1.0, -1.0, 1.0);
        let k2 = v(-1.0, 1.0, -1.0);
        let k3 = v(1.0, 1.0, 1.0);
        let n = k0 * self.model.dist(p_local + k0 * e).0
            + k1 * self.model.dist(p_local + k1 * e).0
            + k2 * self.model.dist(p_local + k2 * e).0
            + k3 * self.model.dist(p_local + k3 * e).0;
        let len = n.norm();
        if len < 1e-9 {
            return v(0.0, 1.0, 0.0);
        }
        self.dir_to_world(n / len)
    }
}
