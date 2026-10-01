// Tabla de materiales de los cubos. Cada material tiene su propia textura y sus parametros:
// albedo = [difuso, especular, reflectividad, transparencia], brillo, indice de refraccion, emision.
use crate::texture::*;
use nalgebra_glm::Vec3;

pub const M_STONE: u8 = 1;
pub const M_PLANKS: u8 = 2;
pub const M_DARK: u8 = 3;
pub const M_GLASS: u8 = 4;
pub const M_BOOKS: u8 = 5;
pub const M_CARPET: u8 = 6;
pub const M_CARPET_B: u8 = 7;
pub const M_GOLD: u8 = 8;
pub const M_FIRE: u8 = 9;
pub const M_SLATE: u8 = 10;
pub const M_ROCK: u8 = 11;
pub const M_GRASS_ROCK: u8 = 12;
pub const M_LEAF: u8 = 13;
pub const M_WATER: u8 = 14;
pub const M_CLOTH: u8 = 15;
pub const M_SILVER: u8 = 16;
pub const M_PORTRAIT: u8 = 17;

#[derive(Clone)]
pub struct VoxMat {
    pub name: &'static str,
    pub tex: usize,
    pub tex_top: usize,
    pub variants: usize,
    pub albedo: [f32; 4],
    pub shininess: f32,
    pub ior: f32,
    pub emission: f32,
    pub tint: Vec3,
}

fn m(
    name: &'static str,
    tex: usize,
    albedo: [f32; 4],
    shininess: f32,
) -> VoxMat {
    VoxMat {
        name,
        tex,
        tex_top: tex,
        variants: 1,
        albedo,
        shininess,
        ior: 1.0,
        emission: 0.0,
        tint: Vec3::new(1.0, 1.0, 1.0),
    }
}

pub fn build_materials() -> Vec<VoxMat> {
    let mut t: Vec<VoxMat> = Vec::new();
    t.push(m("aire", 0, [0.0, 0.0, 0.0, 0.0], 1.0)); // 0
    t.push(m("piedra", T_STONE, [0.95, 0.06, 0.0, 0.0], 8.0)); // 1
    t.push(m("tarima", T_PLANKS, [0.90, 0.30, 0.10, 0.0], 40.0)); // 2 (barnizada: refleja un poco)
    t.push(m("madera oscura", T_DARK, [0.90, 0.30, 0.04, 0.0], 32.0)); // 3

    let mut glass = m("vitral", T_GLASS, [0.06, 0.95, 0.10, 0.88], 140.0); // 4
    glass.ior = 1.5;
    glass.tint = Vec3::new(0.88, 0.96, 1.0);
    t.push(glass);

    let mut books = m("libros", T_BOOKS, [0.95, 0.10, 0.0, 0.0], 10.0); // 5
    books.variants = 4;
    t.push(books);

    t.push(m("alfombra", T_CARPET, [1.0, 0.0, 0.0, 0.0], 1.0)); // 6
    t.push(m("alfombra borde", T_CARPET_B, [1.0, 0.0, 0.0, 0.0], 1.0)); // 7
    t.push(m("oro", T_GOLD, [0.55, 0.95, 0.40, 0.0], 90.0)); // 8

    let mut fire = m("fuego", T_FIRE, [0.0, 0.0, 0.0, 0.0], 1.0); // 9
    fire.emission = 2.4;
    t.push(fire);

    t.push(m("pizarra", T_SLATE, [0.95, 0.30, 0.04, 0.0], 30.0)); // 10
    t.push(m("roca", T_ROCK, [1.0, 0.0, 0.0, 0.0], 1.0)); // 11

    let mut gr = m("roca con pasto", T_ROCK, [1.0, 0.0, 0.0, 0.0], 1.0); // 12
    gr.tex_top = T_GRASS;
    t.push(gr);

    t.push(m("hojas", T_GRASS, [1.0, 0.02, 0.0, 0.0], 4.0)); // 13

    let mut water = m("pensadero", T_WATER, [0.30, 0.85, 0.25, 0.55], 120.0); // 14
    water.ior = 1.33;
    water.emission = 0.85;
    water.tint = Vec3::new(0.75, 0.88, 1.0);
    t.push(water);

    t.push(m("terciopelo", T_CLOTH, [1.0, 0.05, 0.0, 0.0], 6.0)); // 15
    t.push(m("plata", T_SILVER, [0.40, 0.95, 0.55, 0.0], 110.0)); // 16

    let mut pt = m("retratos", T_PORTRAIT, [0.95, 0.15, 0.0, 0.0], 14.0); // 17
    pt.variants = 8;
    t.push(pt);
    t
}
