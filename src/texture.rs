// Texturas procedurales estilo pixel-art (cada cubo muestra una textura de 32x32).
use crate::util::*;
use nalgebra_glm::Vec3;

pub const T_STONE: usize = 0;
pub const T_PLANKS: usize = 1;
pub const T_DARK: usize = 2;
pub const T_GLASS: usize = 3;
pub const T_BOOKS: usize = 4; // 4 variantes: 4..7
pub const T_CARPET: usize = 8;
pub const T_GOLD: usize = 9;
pub const T_FIRE: usize = 10;
pub const T_SLATE: usize = 11;
pub const T_ROCK: usize = 12;
pub const T_GRASS: usize = 13;
pub const T_WATER: usize = 14;
pub const T_CLOTH: usize = 15;
pub const T_SILVER: usize = 16;
pub const T_PORTRAIT: usize = 17; // 8 variantes: 17..24
pub const T_CARPET_B: usize = 25;

pub struct Texture {
    pub w: usize,
    pub h: usize,
    pub rgb: Vec<Vec3>,
    // Multiplicador de transparencia por texel (0 = opaco, p.ej. el emplomado del vitral).
    pub a: Vec<f32>,
}

impl Texture {
    fn build<F: Fn(i32, i32) -> (Vec3, f32)>(w: usize, h: usize, f: F) -> Texture {
        let mut rgb = Vec::with_capacity(w * h);
        let mut a = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                let (c, al) = f(x as i32, y as i32);
                rgb.push(c);
                a.push(al);
            }
        }
        Texture { w, h, rgb, a }
    }

    pub fn sample(&self, u: f32, vv: f32) -> (Vec3, f32) {
        let x = ((u * self.w as f32) as usize).min(self.w - 1);
        let y = ((vv * self.h as f32) as usize).min(self.h - 1);
        let i = y * self.w + x;
        (self.rgb[i], self.a[i])
    }
}

fn stone() -> Texture {
    Texture::build(32, 32, |x, y| {
        let row = y / 8;
        let off = if row % 2 == 0 { 0 } else { 8 };
        let bx = (x + off) % 16;
        let brick = (x + off) / 16;
        if y % 8 == 0 || bx == 0 {
            let n = 0.9 + 0.2 * hash3(x, y, 1);
            return (v(0.34, 0.33, 0.32) * n, 1.0);
        }
        let var = 0.82 + 0.3 * hash3(brick, row, 2);
        let n = 0.92 + 0.16 * hash3(x, y, 3);
        let hl = if y % 8 == 1 { 1.08 } else if y % 8 == 7 { 0.9 } else { 1.0 };
        (v(0.60, 0.58, 0.54) * (var * n * hl), 1.0)
    })
}

fn planks() -> Texture {
    Texture::build(32, 32, |x, y| {
        let plank = y / 8;
        let h = hash3(plank, 0, 4);
        if y % 8 == 0 {
            return (v(0.20, 0.12, 0.06), 1.0);
        }
        let joint = (x + (h * 32.0) as i32) % 32 == 0;
        if joint {
            return (v(0.22, 0.13, 0.07), 1.0);
        }
        let grain = 0.88 + 0.12 * (x as f32 * 0.55 + h * 20.0 + y as f32 * 0.3).sin();
        let n = 0.94 + 0.12 * hash3(x, y, 5);
        (v(0.58, 0.38, 0.19) * ((0.80 + 0.35 * h) * grain * n), 1.0)
    })
}

fn darkwood() -> Texture {
    Texture::build(32, 32, |x, y| {
        let plank = x / 8;
        let h = hash3(plank, 1, 6);
        if x % 8 == 0 {
            return (v(0.12, 0.07, 0.04), 1.0);
        }
        let grain = 0.86 + 0.14 * (y as f32 * 0.35 + h * 9.0 + x as f32 * 0.4).sin();
        let n = 0.93 + 0.14 * hash3(x, y, 7);
        (v(0.30, 0.18, 0.09) * ((0.8 + 0.4 * h) * grain * n), 1.0)
    })
}

fn glass() -> Texture {
    Texture::build(32, 32, |x, y| {
        let border = x < 2 || x >= 30 || y < 2 || y >= 30;
        let cross = x == 16 || y == 16;
        if border || cross {
            return (v(0.14, 0.13, 0.12), 0.0);
        }
        let g = 0.9 + 0.1 * hash3(x / 4, y / 4, 8);
        (v(0.80, 0.92, 1.0) * g, 1.0)
    })
}

fn books(seed: i32) -> Texture {
    let mut spine = vec![0i32; 32];
    let mut x = 0i32;
    let mut idx = 0i32;
    while x < 32 {
        let w = 3 + (hash3(idx, seed, 11) * 3.0) as i32;
        for k in 0..w {
            if x + k < 32 {
                spine[(x + k) as usize] = idx;
            }
        }
        x += w;
        idx += 1;
    }
    let pal: [(f32, f32, f32); 8] = [
        (0.45, 0.07, 0.10),
        (0.08, 0.30, 0.14),
        (0.10, 0.16, 0.40),
        (0.40, 0.24, 0.10),
        (0.62, 0.45, 0.12),
        (0.12, 0.10, 0.10),
        (0.10, 0.36, 0.38),
        (0.62, 0.58, 0.45),
    ];
    Texture::build(32, 32, move |x, y| {
        let b = spine[x as usize];
        let hgt = 18 + (hash3(b, seed, 12) * 12.0) as i32;
        let top = 32 - hgt;
        let edge = x == 0 || spine[(x - 1) as usize] != b;
        if y < top {
            return (v(0.07, 0.05, 0.04), 1.0);
        }
        let ci = ((hash3(b, seed, 13) * 8.0) as usize).min(7);
        let c = pal[ci];
        let mut col = v(c.0, c.1, c.2) * (0.85 + 0.15 * hash3(x, y, 14));
        let dy = y - top;
        if dy == 2 || dy == 4 || y == 29 {
            col = v(0.85, 0.68, 0.25);
        }
        if edge {
            col = col * 0.55;
        }
        (col, 1.0)
    })
}

fn carpet() -> Texture {
    Texture::build(32, 32, |x, y| {
        let px = x % 16;
        let py = y % 16;
        let d = (px - 8).abs() + (py - 8).abs();
        let n = 0.9 + 0.2 * hash3(x, y, 15);
        let col = if d == 8 || d == 7 {
            v(0.78, 0.62, 0.22)
        } else if d < 3 {
            v(0.70, 0.52, 0.18)
        } else {
            v(0.46, 0.07, 0.10)
        };
        (col * n, 1.0)
    })
}

fn carpet_border() -> Texture {
    Texture::build(32, 32, |x, y| {
        let n = 0.9 + 0.2 * hash3(x, y, 16);
        let stripe = (x + y) % 8 < 3;
        let col = if stripe { v(0.80, 0.64, 0.24) } else { v(0.20, 0.05, 0.08) };
        (col * n, 1.0)
    })
}

fn gold() -> Texture {
    Texture::build(32, 32, |x, y| {
        let n = 0.9 + 0.2 * hash3(x, y, 8);
        let edge = if x == 0 || y == 0 {
            1.2
        } else if x == 31 || y == 31 {
            0.75
        } else {
            1.0
        };
        (v(0.95, 0.75, 0.28) * (n * edge), 1.0)
    })
}

fn fire() -> Texture {
    Texture::build(32, 32, |x, y| {
        let fy = y as f32 / 31.0;
        let n = hash3(x / 2, y / 2, 9);
        let heat = clamp01(fy + (n - 0.5) * 0.6);
        let col = mixv(v(0.95, 0.18, 0.02), v(1.0, 0.85, 0.30), heat);
        (col * (0.8 + 0.4 * heat), 1.0)
    })
}

fn slate() -> Texture {
    Texture::build(32, 32, |x, y| {
        let row = y / 8;
        let off = if row % 2 == 0 { 0 } else { 8 };
        let tx = (x + off) % 16;
        let tile = (x + off) / 16;
        let var = 0.8 + 0.4 * hash3(tile, row, 10);
        let n = 0.93 + 0.14 * hash3(x, y, 11);
        let base = v(0.20, 0.24, 0.32) * (var * n);
        if y % 8 == 7 || tx == 0 {
            (base * 0.55, 1.0)
        } else if y % 8 == 0 {
            (base * 1.2, 1.0)
        } else {
            (base, 1.0)
        }
    })
}

fn rock() -> Texture {
    Texture::build(32, 32, |x, y| {
        let n1 = noise3(v(x as f32 * 0.18, y as f32 * 0.18, 3.0));
        let n2 = hash3(x, y, 12);
        let mut col = v(0.43, 0.37, 0.31) * ((0.55 + 0.5 * n1) * (0.9 + 0.2 * n2));
        if hash3(x / 3, y / 3, 13) < 0.08 {
            col = col * 0.7;
        }
        (col, 1.0)
    })
}

fn grass() -> Texture {
    Texture::build(32, 32, |x, y| {
        let mut col = v(0.25, 0.55, 0.15) * (0.75 + 0.5 * hash3(x, y, 14));
        if hash3(x, y, 15) > 0.93 {
            col = v(0.50, 0.75, 0.25);
        }
        (col, 1.0)
    })
}

fn water() -> Texture {
    Texture::build(32, 32, |x, y| {
        let w = 0.5 + 0.5 * (x as f32 * 0.35 + (y as f32 * 0.25).sin() * 2.0).sin();
        let n = hash3(x, y, 16) * 0.2;
        (v(0.55, 0.72, 0.95) * (0.7 + 0.5 * w) + v(n, n, n), 1.0)
    })
}

fn cloth() -> Texture {
    Texture::build(32, 32, |x, y| {
        let weave = if (x + y) % 2 == 0 { 1.0 } else { 0.85 };
        let fold = 0.85 + 0.15 * (x as f32 * 0.8).sin();
        (v(0.38, 0.07, 0.22) * (weave * fold * (0.9 + 0.2 * hash3(x, y, 17))), 1.0)
    })
}

fn silver() -> Texture {
    Texture::build(32, 32, |x, y| {
        let n = 0.9 + 0.2 * hash3(x, y, 18);
        let edge = if x == 0 || y == 0 { 1.15 } else if x == 31 || y == 31 { 0.8 } else { 1.0 };
        (v(0.80, 0.83, 0.90) * (n * edge), 1.0)
    })
}

fn portrait(i: i32) -> Texture {
    let bgp: [(f32, f32, f32); 8] = [
        (0.20, 0.10, 0.10),
        (0.08, 0.16, 0.12),
        (0.10, 0.12, 0.22),
        (0.25, 0.18, 0.08),
        (0.18, 0.08, 0.18),
        (0.08, 0.18, 0.20),
        (0.22, 0.12, 0.06),
        (0.12, 0.12, 0.14),
    ];
    let robes: [(f32, f32, f32); 8] = [
        (0.45, 0.10, 0.12),
        (0.10, 0.35, 0.20),
        (0.15, 0.20, 0.55),
        (0.55, 0.40, 0.12),
        (0.40, 0.12, 0.45),
        (0.12, 0.45, 0.50),
        (0.60, 0.25, 0.10),
        (0.30, 0.30, 0.36),
    ];
    let hairs: [(f32, f32, f32); 4] = [
        (0.88, 0.88, 0.84),
        (0.62, 0.62, 0.62),
        (0.35, 0.22, 0.12),
        (0.10, 0.09, 0.09),
    ];
    Texture::build(32, 32, move |x, y| {
        if x < 2 || x >= 30 || y < 2 || y >= 30 {
            return (v(0.62, 0.45, 0.16) * (0.9 + 0.2 * hash3(x, y, 20)), 1.0);
        }
        if x == 2 || x == 29 || y == 2 || y == 29 {
            return (v(0.12, 0.08, 0.05), 1.0);
        }
        let bg = bgp[(i % 8) as usize];
        let rb = robes[(i % 8) as usize];
        let hc = hairs[((i / 2) % 4) as usize];
        let xf = x as f32 - 16.0;
        let yf = y as f32 - 12.0;
        let face = (xf / 6.0) * (xf / 6.0) + (yf / 8.0) * (yf / 8.0) < 1.0;
        let hair = (xf / 9.0) * (xf / 9.0) + (yf / 11.0) * (yf / 11.0) < 1.0 && !face && y < 14;
        let beard = y >= 14 && y < 25 && xf.abs() < 5.0 - (y as f32 - 14.0) * 0.35 && (i % 3 != 0);
        let robe = y >= 22 && xf.abs() < 4.0 + (y as f32 - 22.0) * 0.9;
        let n = 0.92 + 0.16 * hash3(x, y, 21);
        if y == 11 && (x == 14 || x == 18) {
            return (v(0.05, 0.04, 0.04), 1.0);
        }
        let col = if beard {
            v(hc.0, hc.1, hc.2)
        } else if face {
            v(0.85, 0.65, 0.50)
        } else if hair {
            v(hc.0, hc.1, hc.2)
        } else if robe {
            v(rb.0, rb.1, rb.2)
        } else {
            v(bg.0, bg.1, bg.2)
        };
        (col * n, 1.0)
    })
}

pub fn build_textures() -> Vec<Texture> {
    let mut t: Vec<Texture> = Vec::new();
    t.push(stone()); // 0
    t.push(planks()); // 1
    t.push(darkwood()); // 2
    t.push(glass()); // 3
    for i in 0..4 {
        t.push(books(i)); // 4..7
    }
    t.push(carpet()); // 8
    t.push(gold()); // 9
    t.push(fire()); // 10
    t.push(slate()); // 11
    t.push(rock()); // 12
    t.push(grass()); // 13
    t.push(water()); // 14
    t.push(cloth()); // 15
    t.push(silver()); // 16
    for i in 0..8 {
        t.push(portrait(i)); // 17..24
    }
    t.push(carpet_border()); // 25
    t
}
