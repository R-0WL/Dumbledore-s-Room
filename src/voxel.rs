// Rejilla de cubos (diorama) con recorrido DDA (Amanatides-Woo).
use crate::util::*;
use nalgebra_glm::Vec3;

pub struct VoxelHit {
    pub t: f32,
    pub cell: (i32, i32, i32),
    pub normal: Vec3,
    pub mat: u8,
}

pub struct Grid {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub cell: f32,
    pub data: Vec<u8>,
    // Transmitancia por material (para sombras a traves de vidrio/liquido).
    pub transmit: Vec<f32>,
}

impl Grid {
    pub fn new(nx: usize, ny: usize, nz: usize, cell: f32) -> Grid {
        Grid { nx, ny, nz, cell, data: vec![0u8; nx * ny * nz], transmit: vec![0.0; 64] }
    }

    fn idx(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        if x < 0 || y < 0 || z < 0 || x >= self.nx as i32 || y >= self.ny as i32 || z >= self.nz as i32 {
            None
        } else {
            Some((y as usize * self.nz + z as usize) * self.nx + x as usize)
        }
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> u8 {
        match self.idx(x, y, z) {
            Some(i) => self.data[i],
            None => 0,
        }
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, m: u8) {
        if let Some(i) = self.idx(x, y, z) {
            self.data[i] = m;
        }
    }

    pub fn set_if_air(&mut self, x: i32, y: i32, z: i32, m: u8) {
        if let Some(i) = self.idx(x, y, z) {
            if self.data[i] == 0 {
                self.data[i] = m;
            }
        }
    }

    pub fn size(&self) -> Vec3 {
        v(self.nx as f32 * self.cell, self.ny as f32 * self.cell, self.nz as f32 * self.cell)
    }

    // Recorre las celdas atravesadas por el rayo; `f` devuelve true para detener.
    fn walk<F: FnMut(i32, i32, i32, u8, f32, Vec3) -> bool>(&self, o: Vec3, d: Vec3, tmax: f32, mut f: F) {
        let size = self.size();
        let mut t0 = 0.0f32;
        let mut t1 = tmax;
        let mut enter_axis: i32 = -1;
        for a in 0..3 {
            if d[a].abs() < 1e-9 {
                if o[a] < 0.0 || o[a] > size[a] {
                    return;
                }
            } else {
                let inv = 1.0 / d[a];
                let mut ta = (0.0 - o[a]) * inv;
                let mut tb = (size[a] - o[a]) * inv;
                if ta > tb {
                    std::mem::swap(&mut ta, &mut tb);
                }
                if ta > t0 {
                    t0 = ta;
                    enter_axis = a as i32;
                }
                if tb < t1 {
                    t1 = tb;
                }
                if t0 > t1 {
                    return;
                }
            }
        }
        let ps = o + d * (t0 + 1e-4);
        let dims = [self.nx as i32, self.ny as i32, self.nz as i32];
        let mut c = [0i32; 3];
        let mut step = [0i32; 3];
        let mut tm = [f32::INFINITY; 3];
        let mut td = [f32::INFINITY; 3];
        for a in 0..3 {
            c[a] = ((ps[a] / self.cell).floor() as i32).max(0).min(dims[a] - 1);
            if d[a] > 0.0 {
                step[a] = 1;
                tm[a] = ((c[a] + 1) as f32 * self.cell - o[a]) / d[a];
                td[a] = self.cell / d[a];
            } else if d[a] < 0.0 {
                step[a] = -1;
                tm[a] = (c[a] as f32 * self.cell - o[a]) / d[a];
                td[a] = -self.cell / d[a];
            }
        }
        let mut normal = if enter_axis >= 0 {
            let mut n = v(0.0, 0.0, 0.0);
            let a = enter_axis as usize;
            n[a] = if d[a] > 0.0 { -1.0 } else { 1.0 };
            n
        } else {
            -d
        };
        let mut t_cur = t0;
        loop {
            let m = self.get(c[0], c[1], c[2]);
            if m != 0 && f(c[0], c[1], c[2], m, t_cur, normal) {
                return;
            }
            let mut a = 0;
            if tm[1] < tm[a] {
                a = 1;
            }
            if tm[2] < tm[a] {
                a = 2;
            }
            t_cur = tm[a];
            if t_cur > t1 {
                return;
            }
            c[a] += step[a];
            if c[a] < 0 || c[a] >= dims[a] {
                return;
            }
            tm[a] += td[a];
            normal = v(0.0, 0.0, 0.0);
            normal[a] = -(step[a] as f32);
        }
    }

    pub fn trace(&self, o: Vec3, d: Vec3, tmax: f32) -> Option<VoxelHit> {
        let mut res: Option<VoxelHit> = None;
        self.walk(o, d, tmax, |x, y, z, m, t, n| {
            res = Some(VoxelHit { t, cell: (x, y, z), normal: n, mat: m });
            true
        });
        res
    }

    // Visibilidad hacia una luz: 0 = bloqueada, entre 0 y 1 si atraviesa vidrio/liquido.
    pub fn shadow(&self, o: Vec3, d: Vec3, tmax: f32) -> f32 {
        let mut vis = 1.0f32;
        let tr = &self.transmit;
        self.walk(o, d, tmax, |_x, _y, _z, m, _t, _n| {
            let k = tr[m as usize];
            if k <= 0.0 {
                vis = 0.0;
                true
            } else {
                vis *= k;
                vis < 0.01
            }
        });
        vis
    }
}
