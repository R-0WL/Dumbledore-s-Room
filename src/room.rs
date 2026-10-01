// Construccion del diorama: torre circular flotante del despacho, hecha solo con cubos texturizados.
use crate::material::*;
use crate::tracer::Sphere;
use crate::util::*;
use crate::voxel::Grid;
use nalgebra_glm::Vec3;
use std::f32::consts::PI;

pub const NX: usize = 40;
pub const NY: usize = 44;
pub const NZ: usize = 40;
pub const CELL: f32 = 0.5;
pub const CX: f32 = 20.0;
pub const CZ: f32 = 20.0;
pub const FLOOR_Y: i32 = 14; // capa del piso (la superficie queda en y = 7.5 m)
pub const CEIL_Y: i32 = 28;
pub const R_IN: f32 = 9.2;
pub const R_OUT: f32 = 10.8;

pub struct WinDef {
    pub alpha: f32, // azimut desde +Z hacia +X
    pub hw: f32,    // semiancho en celdas
}

pub const WINDOWS: [WinDef; 3] = [
    WinDef { alpha: 0.0, hw: 2.6 },
    WinDef { alpha: 2.35, hw: 1.6 },
    WinDef { alpha: -2.35, hw: 1.6 },
];

pub struct Room {
    pub grid: Grid,
    pub spheres: Vec<Sphere>,
}

pub fn cell_center(x: i32, y: i32, z: i32) -> Vec3 {
    v((x as f32 + 0.5) * CELL, (y as f32 + 0.5) * CELL, (z as f32 + 0.5) * CELL)
}

pub fn center_world() -> Vec3 {
    v(CX * CELL, 0.0, CZ * CELL)
}

fn dxz(x: i32, z: i32) -> (f32, f32) {
    (x as f32 + 0.5 - CX, z as f32 + 0.5 - CZ)
}

fn rad(x: i32, z: i32) -> f32 {
    let (dx, dz) = dxz(x, z);
    (dx * dx + dz * dz).sqrt()
}

fn azim(x: i32, z: i32) -> f32 {
    let (dx, dz) = dxz(x, z);
    dx.atan2(dz)
}

fn fill(g: &mut Grid, x0: i32, x1: i32, y0: i32, y1: i32, z0: i32, z1: i32, m: u8) {
    for y in y0..=y1 {
        for z in z0..=z1 {
            for x in x0..=x1 {
                g.set(x, y, z, m);
            }
        }
    }
}

// Forma de una ventana arqueada en la pared: vitral, marco de madera o nada.
fn window_kind(w: &WinDef, dx: f32, dz: f32, y: i32) -> u8 {
    let (sa, ca) = w.alpha.sin_cos();
    let radial = dx * sa + dz * ca;
    if radial <= 0.0 {
        return 0;
    }
    let s = (dx * ca - dz * sa).abs();
    let yy = y as f32;
    let spring = 21.0;
    let glass = if yy < 17.0 {
        false
    } else if yy <= spring {
        s <= w.hw
    } else {
        s * s + (yy - spring) * (yy - spring) <= w.hw * w.hw
    };
    if glass {
        return M_GLASS;
    }
    let fw = w.hw + 1.2;
    let frame = if yy < 16.0 {
        false
    } else if yy <= spring {
        s <= fw
    } else {
        s * s + (yy - spring) * (yy - spring) <= fw * fw
    };
    if frame {
        M_DARK
    } else {
        0
    }
}

// Libreros, retratos, cortinas y repisas pegados a la pared interior.
fn ring_item(x: i32, y: i32, z: i32) -> u8 {
    if y < 15 || y > 27 {
        return 0;
    }
    let (dx, dz) = dxz(x, z);
    let a = dx.atan2(dz);
    for w in WINDOWS.iter() {
        let (sa, ca) = w.alpha.sin_cos();
        let radial = dx * sa + dz * ca;
        if radial > 0.0 {
            let s = (dx * ca - dz * sa).abs();
            if s < w.hw + 2.6 {
                if y == 16 && s < w.hw + 1.6 {
                    return M_DARK;
                }
                if s >= w.hw + 1.3 && y >= 16 && y <= 25 {
                    return M_CLOTH;
                }
                return 0;
            }
        }
    }
    if adiff(a, PI).abs() < 0.30 {
        return 0; // chimenea
    }
    if a > 0.50 && a < 1.98 {
        return 0; // escalera y puerta
    }
    match y {
        15 | 19 | 23 | 27 => M_DARK,
        16..=18 | 20..=22 => M_BOOKS,
        _ => M_PORTRAIT,
    }
}

fn blob(g: &mut Grid, cx: f32, cy: f32, cz: f32, r: f32) {
    let ri = r.ceil() as i32 + 1;
    for y in (cy as i32 - ri)..=(cy as i32 + ri) {
        for z in (cz as i32 - ri)..=(cz as i32 + ri) {
            for x in (cx as i32 - ri)..=(cx as i32 + ri) {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let dz = z as f32 + 0.5 - cz;
                if (dx * dx + dy * dy + dz * dz).sqrt() < r {
                    g.set_if_air(x, y, z, M_ROCK);
                }
            }
        }
    }
    // pasto en la cara superior
    for y in (cy as i32 - ri)..=(cy as i32 + ri) {
        for z in (cz as i32 - ri)..=(cz as i32 + ri) {
            for x in (cx as i32 - ri)..=(cx as i32 + ri) {
                if g.get(x, y, z) == M_ROCK && g.get(x, y + 1, z) == 0 {
                    g.set(x, y, z, M_GRASS_ROCK);
                }
            }
        }
    }
}

fn tree(g: &mut Grid, x: i32, z: i32) {
    fill(g, x, x, FLOOR_Y, FLOOR_Y + 3, z, z, M_DARK);
    let (cx, cy, cz) = (x as f32 + 0.5, (FLOOR_Y + 5) as f32, z as f32 + 0.5);
    for y in (cy as i32 - 3)..=(cy as i32 + 3) {
        for zz in (z - 3)..=(z + 3) {
            for xx in (x - 3)..=(x + 3) {
                let dx = xx as f32 + 0.5 - cx;
                let dy = (y as f32 + 0.5 - cy) * 1.2;
                let dz = zz as f32 + 0.5 - cz;
                if (dx * dx + dy * dy + dz * dz).sqrt() < 2.4 {
                    g.set_if_air(xx, y, zz, M_LEAF);
                }
            }
        }
    }
}

pub fn build_room(mats: &[VoxMat]) -> Room {
    let mut g = Grid::new(NX, NY, NZ, CELL);
    for (i, m) in mats.iter().enumerate() {
        let t = m.albedo[3];
        g.transmit[i] = if t > 0.3 { t * 0.9 } else { 0.0 };
    }

    // 1) Isla flotante (cono invertido irregular de roca, con pasto arriba)
    for y in 0..FLOOR_Y {
        let f = y as f32 / (FLOOR_Y - 1) as f32;
        let base = 2.2 + 12.0 * f.powf(0.75);
        for z in 0..NZ as i32 {
            for x in 0..NX as i32 {
                let n = noise3(v(x as f32 * 0.32, y as f32 * 0.40, z as f32 * 0.32));
                let rr = base + (n - 0.5) * 3.4;
                if rad(x, z) < rr {
                    g.set(x, y, z, if y == FLOOR_Y - 1 { M_GRASS_ROCK } else { M_ROCK });
                }
            }
        }
    }

    // 2) Piso, alfombra, techo y vigas
    for z in 0..NZ as i32 {
        for x in 0..NX as i32 {
            let r = rad(x, z);
            if r < R_OUT {
                let m = if r < 5.0 {
                    M_CARPET
                } else if r < 5.7 {
                    M_CARPET_B
                } else {
                    M_PLANKS
                };
                g.set(x, FLOOR_Y, z, m);
                g.set(x, CEIL_Y, z, M_DARK);
            }
            if r >= R_IN && r < R_OUT {
                for y in (FLOOR_Y + 1)..CEIL_Y {
                    g.set(x, y, z, M_STONE);
                }
            }
            if r < R_IN {
                let beam = x == 19 || x == 20 || z == 19 || z == 20 || r >= R_IN - 1.0;
                if beam {
                    g.set(x, CEIL_Y - 1, z, M_DARK);
                }
            }
        }
    }

    // 3) Techo conico de pizarra
    for y in (CEIL_Y + 1)..=(CEIL_Y + 13) {
        let rr = 12.0 * (1.0 - (y - CEIL_Y - 1) as f32 / 13.5);
        for z in 0..NZ as i32 {
            for x in 0..NX as i32 {
                if rad(x, z) < rr {
                    g.set(x, y, z, M_SLATE);
                }
            }
        }
    }
    fill(&mut g, 19, 20, CEIL_Y + 14, CEIL_Y + 14, 19, 20, M_GOLD);
    g.set(20, CEIL_Y + 15, 20, M_GOLD);

    // 4) Ventanas arqueadas con vitral
    for z in 0..NZ as i32 {
        for x in 0..NX as i32 {
            let r = rad(x, z);
            if r < R_IN || r >= R_OUT {
                continue;
            }
            let (dx, dz) = dxz(x, z);
            for y in (FLOOR_Y + 1)..CEIL_Y {
                for w in WINDOWS.iter() {
                    let k = window_kind(w, dx, dz, y);
                    if k != 0 {
                        g.set(x, y, z, k);
                    }
                }
            }
        }
    }

    // 5) Libreros, retratos, cortinas
    for z in 0..NZ as i32 {
        for x in 0..NX as i32 {
            let r = rad(x, z);
            if r >= 8.2 && r < R_IN {
                for y in (FLOOR_Y + 1)..CEIL_Y {
                    let m = ring_item(x, y, z);
                    if m != 0 {
                        g.set(x, y, z, m);
                    }
                }
            }
        }
    }

    // 6) Escalera de caracol, descanso y puerta
    let a0 = 0.50f32;
    let da = 0.19f32;
    for z in 0..NZ as i32 {
        for x in 0..NX as i32 {
            let r = rad(x, z);
            let a = azim(x, z);
            if r >= 7.0 && r < R_IN && a >= a0 && a < 1.91 {
                let k = (((a - a0) / da).floor() as i32).min(5);
                let top = FLOOR_Y + 1 + k;
                for y in (FLOOR_Y + 1)..=top {
                    g.set(x, y, z, if y == top { M_DARK } else { M_STONE });
                }
            }
            if r >= R_IN && r < R_OUT && a >= 1.72 && a <= 1.94 {
                fill(&mut g, x, x, 21, 24, z, z, M_DARK);
            }
        }
    }

    // 7) Chimenea con fuego, repisa y retrato
    fill(&mut g, 17, 22, 15, 22, 11, 12, M_STONE);
    fill(&mut g, 18, 21, 15, 18, 11, 12, 0);
    fill(&mut g, 19, 20, 15, 15, 11, 11, M_DARK);
    fill(&mut g, 19, 20, 16, 17, 11, 11, M_FIRE);
    fill(&mut g, 16, 23, 23, 23, 11, 13, M_DARK);
    fill(&mut g, 18, 21, 24, 27, 11, 11, M_STONE);
    fill(&mut g, 18, 21, 24, 26, 12, 12, M_PORTRAIT);
    fill(&mut g, 16, 23, FLOOR_Y, FLOOR_Y, 13, 14, M_STONE);

    // 8) Escritorio de Dumbledore con objetos
    fill(&mut g, 16, 21, 16, 16, 16, 18, M_DARK);
    fill(&mut g, 16, 16, 15, 15, 16, 18, M_DARK);
    fill(&mut g, 21, 21, 15, 15, 16, 18, M_DARK);
    fill(&mut g, 16, 21, 15, 15, 16, 16, M_DARK);
    g.set(18, 17, 17, M_GOLD); // base de la esfera de cristal
    g.set(21, 17, 17, M_GOLD);
    g.set(21, 18, 17, M_FIRE); // vela
    g.set(16, 17, 16, M_GOLD);
    g.set(16, 18, 16, M_FIRE);
    g.set(19, 17, 17, M_BOOKS);
    g.set(19, 18, 17, M_BOOKS);
    g.set(17, 17, 18, M_SILVER);
    g.set(17, 18, 18, M_SILVER);

    // 9) Sillon alto detras del escritorio y dos sillas al frente
    fill(&mut g, 18, 19, 15, 15, 14, 15, M_CLOTH);
    fill(&mut g, 18, 19, 16, 18, 13, 13, M_CLOTH);
    fill(&mut g, 17, 17, 16, 16, 14, 15, M_CLOTH);
    fill(&mut g, 20, 20, 16, 16, 14, 15, M_CLOTH);
    g.set(18, 19, 13, M_GOLD);
    g.set(19, 19, 13, M_GOLD);
    for sx in [18, 20].iter() {
        fill(&mut g, *sx, *sx + 1, 15, 15, 20, 21, M_CLOTH);
        fill(&mut g, *sx, *sx + 1, 16, 17, 22, 22, M_CLOTH);
    }

    // 10) Pensadero
    fill(&mut g, 13, 17, 15, 15, 20, 24, M_DARK);
    fill(&mut g, 13, 17, 16, 18, 20, 24, M_STONE);
    fill(&mut g, 14, 16, 16, 18, 21, 23, 0);
    fill(&mut g, 14, 16, 16, 17, 21, 23, M_WATER);
    for (px, pz) in [(13, 20), (17, 20), (13, 24), (17, 24)].iter() {
        g.set(*px, 18, *pz, M_GOLD);
    }

    // 11) Percha de Fawkes
    fill(&mut g, 23, 25, 15, 15, 16, 18, M_DARK);
    fill(&mut g, 24, 24, 16, 18, 17, 17, M_GOLD);

    // 12) Candelabro
    for z in 17..23 {
        for x in 17..23 {
            let r = rad(x, z);
            if (r - 2.6).abs() < 0.55 {
                g.set(x, 24, z, M_GOLD);
            }
        }
    }
    fill(&mut g, 19, 19, 24, 26, 19, 19, M_GOLD);
    fill(&mut g, 19, 20, 24, 24, 19, 19, M_GOLD);
    for k in 0..6 {
        let a = k as f32 * PI / 3.0 + 0.3;
        let cx = (CX + 2.6 * a.sin()).floor() as i32;
        let cz = (CZ + 2.6 * a.cos()).floor() as i32;
        g.set(cx, 24, cz, M_GOLD);
        g.set(cx, 25, cz, M_FIRE);
    }

    // 13) Arboles y rocas flotantes alrededor
    tree(&mut g, 13, 9);
    tree(&mut g, 29, 27);
    let blobs: [(f32, f32, f32, f32); 6] = [
        (33.0, 8.0, 30.0, 2.0),
        (7.0, 10.0, 31.0, 1.6),
        (34.0, 18.0, 10.0, 1.4),
        (6.0, 24.0, 8.0, 1.8),
        (9.0, 30.0, 33.0, 1.3),
        (31.0, 36.0, 26.0, 1.5),
    ];
    for b in blobs.iter() {
        blob(&mut g, b.0, b.1, b.2, b.3);
    }

    // Esferas de cristal (refraccion real): una sobre el escritorio y un recuerdo sobre el pensadero
    let mut spheres: Vec<Sphere> = Vec::new();
    spheres.push(Sphere {
        center: v(9.25, 9.2, 8.75),
        radius: 0.2,
        color: v(1.0, 1.0, 1.0),
        albedo: [0.02, 0.9, 0.05, 0.92],
        shininess: 200.0,
        ior: 1.5,
        emission: v(0.0, 0.0, 0.0),
        tint: v(0.95, 0.98, 1.0),
    });
    spheres.push(Sphere {
        center: v(7.75, 9.95, 11.25),
        radius: 0.17,
        color: v(0.5, 0.7, 1.0),
        albedo: [0.1, 0.9, 0.1, 0.6],
        shininess: 120.0,
        ior: 1.3,
        emission: v(0.22, 0.40, 0.80),
        tint: v(0.7, 0.85, 1.0),
    });
    Room { grid: g, spheres }
}
