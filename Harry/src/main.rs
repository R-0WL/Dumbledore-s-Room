// Harry y Buckbeak simplificado, basado en la estructura del archivo modeladors(1).rs.
// Reemplaza src/main.rs; conserva minifb y nalgebra-glm en Cargo.toml.
// Geometria procedural: no necesita scene.bin, modelos ni texturas externas.
// Buckbeak: frente +Z, alas a ambos lados de X, vertical +Y.
// Reconstruccion simplificada desde la imagen adjunta: no carga modelos externos.
// Ejecutar con cargo run --release (el ray marching es costoso en modo debug).
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nalgebra_glm::{cross, dot, Vec3};
use std::f32::consts::PI;

const WIDTH: usize = 1080;
const HEIGHT: usize = 760;
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
    let reference = if axis.y.abs()<0.95 {v(0.0,1.0,0.0)} else {v(0.0,0.0,1.0)};
    let side = cross(&axis,&reference).normalize();
    let back = cross(&side,&axis);
    let q = p-(a+b)*0.5;
    let local = v(dot(&q,&side),dot(&q,&axis),dot(&q,&back));
    ellipsoid(local,v(0.0,0.0,0.0),v(width,(b-a).norm()*0.5,thickness),0.0)
}

fn objects(p: Vec3, mut hit: (f32,usize)) -> (f32,usize) {
    choose(&mut hit,ellipsoid(p,v(0.0,2.08,-0.34),v(0.49,0.54,1.06),0.0),2);
    choose(&mut hit,ellipsoid(p,v(0.0,2.08,-1.04),v(0.52,0.53,0.54),0.0),2);
    choose(&mut hit,ellipsoid(p,v(0.0,2.16,0.49),v(0.48,0.66,0.52),0.0),0);
    choose(&mut hit,ellipsoid(p,v(0.0,2.89,1.35),v(0.31,0.32,0.43),0.0),0);
    choose(&mut hit,ellipsoid(p,v(0.0,2.82,1.58),v(0.29,0.22,0.3),0.0),0);
    choose(&mut hit,ellipsoid(p,v(0.0,2.8,1.9),v(0.205,0.14,0.26),0.0),3);
    choose(&mut hit,ellipsoid(p,v(0.0,2.68,2.085),v(0.115,0.19,0.11),0.0),3);
    choose(&mut hit,ellipsoid(p,v(0.0,2.54,2.1),v(0.049,0.085,0.052),0.0),3);
    choose(&mut hit,ellipsoid(p,v(-0.253,2.96,1.6),v(0.064,0.075,0.064),0.0),5);
    choose(&mut hit,ellipsoid(p,v(-0.275,2.966,1.626),v(0.039,0.048,0.035),0.0),4);
    choose(&mut hit,ellipsoid(p,v(-0.293,2.966,1.643),v(0.015,0.033,0.017),0.0),5);
    choose(&mut hit,ellipsoid(p,v(-0.293,2.989,1.664),v(0.008,0.011,0.007),0.0),13);
    choose(&mut hit,ellipsoid(p,v(-0.45,1.47,-1.71),v(0.13,0.13,0.18),0.0),12);
    choose(&mut hit,ellipsoid(p,v(0.253,2.96,1.6),v(0.064,0.075,0.064),0.0),5);
    choose(&mut hit,ellipsoid(p,v(0.275,2.966,1.626),v(0.039,0.048,0.035),0.0),4);
    choose(&mut hit,ellipsoid(p,v(0.293,2.966,1.643),v(0.015,0.033,0.017),0.0),5);
    choose(&mut hit,ellipsoid(p,v(0.293,2.989,1.664),v(0.008,0.011,0.007),0.0),13);
    choose(&mut hit,ellipsoid(p,v(0.45,1.47,-1.71),v(0.13,0.13,0.18),0.0),12);
    choose(&mut hit,ellipsoid(p,v(0.0,2.79,-0.4),v(0.27,0.2,0.25),0.0),11);
    choose(&mut hit,ellipsoid(p,v(0.0,3.22,-0.42),v(0.31,0.46,0.23),0.0),7);
    choose(&mut hit,ellipsoid(p,v(0.0,3.25,-0.211),v(0.18,0.34,0.031),0.0),8);
    choose(&mut hit,ellipsoid(p,v(0.0,3.7,-0.41),v(0.105,0.17,0.1),0.0),6);
    choose(&mut hit,ellipsoid(p,v(0.0,3.99,-0.39),v(0.224,0.29,0.205),0.0),6);
    choose(&mut hit,ellipsoid(p,v(-0.222,3.99,-0.39),v(0.047,0.078,0.05),0.0),6);
    choose(&mut hit,ellipsoid(p,v(-0.62,3.32,-0.41),v(0.17,0.21,0.13),0.0),7);
    choose(&mut hit,ellipsoid(p,v(-1.105,3.4,-0.25),v(0.13,0.054,0.085),0.0),6);
    choose(&mut hit,ellipsoid(p,v(-0.48,1.98,0.23),v(0.13,0.11,0.22),0.0),12);
    choose(&mut hit,ellipsoid(p,v(0.222,3.99,-0.39),v(0.047,0.078,0.05),0.0),6);
    choose(&mut hit,ellipsoid(p,v(0.62,3.32,-0.41),v(0.17,0.21,0.13),0.0),7);
    choose(&mut hit,ellipsoid(p,v(1.105,3.4,-0.25),v(0.13,0.054,0.085),0.0),6);
    choose(&mut hit,ellipsoid(p,v(0.48,1.98,0.23),v(0.13,0.11,0.22),0.0),12);
    choose(&mut hit,ellipsoid(p,v(0.0,4.17,-0.42),v(0.235,0.14,0.21),0.0),5);
    choose(&mut hit,ellipsoid(p,v(0.0,4.04,-0.535),v(0.22,0.23,0.105),0.0),5);
    choose(&mut hit,ellipsoid(p,v(0.0,3.97,-0.16),v(0.041,0.072,0.062),0.0),6);
    choose(&mut hit,ellipsoid(p,v(-0.089,4.013,-0.204),v(0.043,0.029,0.015),0.0),13);
    choose(&mut hit,ellipsoid(p,v(-0.089,4.013,-0.19),v(0.016,0.022,0.008),0.0),5);
    choose(&mut hit,ellipsoid(p,v(0.089,4.013,-0.204),v(0.043,0.029,0.015),0.0),13);
    choose(&mut hit,ellipsoid(p,v(0.089,4.013,-0.19),v(0.016,0.022,0.008),0.0),5);
    choose(&mut hit,capsule(p,v(-0.35,1.49,0.82),v(-0.34,1.16,1.01),0.072),3);
    choose(&mut hit,capsule(p,v(-0.34,1.16,1.01),v(-0.34,1.02,1.22),0.059),3);
    choose(&mut hit,capsule(p,v(-0.34,1.04,1.2),v(-0.48,0.96,1.475),0.034),3);
    choose(&mut hit,capsule(p,v(-0.48,0.96,1.475),v(-0.48,1.015,1.555),0.023),5);
    choose(&mut hit,capsule(p,v(-0.34,1.04,1.2),v(-0.34,0.96,1.51),0.034),3);
    choose(&mut hit,capsule(p,v(-0.34,0.96,1.51),v(-0.34,1.015,1.59),0.023),5);
    choose(&mut hit,capsule(p,v(-0.34,1.04,1.2),v(-0.2,0.96,1.475),0.034),3);
    choose(&mut hit,capsule(p,v(-0.2,0.96,1.475),v(-0.2,1.015,1.555),0.023),5);
    choose(&mut hit,capsule(p,v(-0.43,1.58,-0.94),v(-0.44,1.25,-1.26),0.105),2);
    choose(&mut hit,capsule(p,v(-0.44,1.25,-1.26),v(-0.45,1.47,-1.64),0.081),2);
    choose(&mut hit,capsule(p,v(0.35,1.49,0.82),v(0.34,1.16,1.01),0.072),3);
    choose(&mut hit,capsule(p,v(0.34,1.16,1.01),v(0.34,1.02,1.22),0.059),3);
    choose(&mut hit,capsule(p,v(0.34,1.04,1.2),v(0.2,0.96,1.475),0.034),3);
    choose(&mut hit,capsule(p,v(0.2,0.96,1.475),v(0.2,1.015,1.555),0.023),5);
    choose(&mut hit,capsule(p,v(0.34,1.04,1.2),v(0.34,0.96,1.51),0.034),3);
    choose(&mut hit,capsule(p,v(0.34,0.96,1.51),v(0.34,1.015,1.59),0.023),5);
    choose(&mut hit,capsule(p,v(0.34,1.04,1.2),v(0.48,0.96,1.475),0.034),3);
    choose(&mut hit,capsule(p,v(0.48,0.96,1.475),v(0.48,1.015,1.555),0.023),5);
    choose(&mut hit,capsule(p,v(0.43,1.58,-0.94),v(0.44,1.25,-1.26),0.105),2);
    choose(&mut hit,capsule(p,v(0.44,1.25,-1.26),v(0.45,1.47,-1.64),0.081),2);
    choose(&mut hit,capsule(p,v(0.0,2.25,-1.43),v(0.09,2.12,-1.9),0.11),2);
    choose(&mut hit,capsule(p,v(0.09,2.12,-1.9),v(0.25,1.91,-2.3),0.095),5);
    choose(&mut hit,capsule(p,v(-0.2,3.49,-0.43),v(-0.65,3.44,-0.38),0.16),7);
    choose(&mut hit,capsule(p,v(-0.6,3.44,-0.38),v(-1.0,3.4,-0.29),0.13),7);
    choose(&mut hit,capsule(p,v(-1.12,3.4,-0.23),v(-1.23,3.39,-0.2),0.018),6);
    choose(&mut hit,capsule(p,v(-1.12,3.4,-0.195),v(-1.23,3.39,-0.165),0.018),6);
    choose(&mut hit,capsule(p,v(-1.12,3.4,-0.16),v(-1.23,3.39,-0.13),0.018),6);
    choose(&mut hit,capsule(p,v(-0.21,2.79,-0.4),v(-0.51,2.37,-0.17),0.135),11);
    choose(&mut hit,capsule(p,v(-0.51,2.38,-0.17),v(-0.48,2.01,0.14),0.11),11);
    choose(&mut hit,capsule(p,v(0.2,3.49,-0.43),v(0.65,3.44,-0.38),0.16),7);
    choose(&mut hit,capsule(p,v(0.6,3.44,-0.38),v(1.0,3.4,-0.29),0.13),7);
    choose(&mut hit,capsule(p,v(1.12,3.4,-0.23),v(1.23,3.39,-0.2),0.018),6);
    choose(&mut hit,capsule(p,v(1.12,3.4,-0.195),v(1.23,3.39,-0.165),0.018),6);
    choose(&mut hit,capsule(p,v(1.12,3.4,-0.16),v(1.23,3.39,-0.13),0.018),6);
    choose(&mut hit,capsule(p,v(0.21,2.79,-0.4),v(0.51,2.37,-0.17),0.135),11);
    choose(&mut hit,capsule(p,v(0.51,2.38,-0.17),v(0.48,2.01,0.14),0.11),11);
    choose(&mut hit,capsule(p,v(-0.053,3.876,-0.195),v(0.045,3.876,-0.192),0.01),9);
    choose(&mut hit,capsule(p,v(-0.021,4.011,-0.17),v(-0.026176,4.034726,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.026176,4.034726,-0.17),v(-0.040917,4.054841,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.040917,4.054841,-0.17),v(-0.062978,4.068281,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.062978,4.068281,-0.17),v(-0.089,4.073,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.089,4.073,-0.17),v(-0.115022,4.068281,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.115022,4.068281,-0.17),v(-0.137083,4.054841,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.137083,4.054841,-0.17),v(-0.151824,4.034726,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.151824,4.034726,-0.17),v(-0.157,4.011,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.157,4.011,-0.17),v(-0.151824,3.987274,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.151824,3.987274,-0.17),v(-0.137083,3.967159,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.137083,3.967159,-0.17),v(-0.115022,3.953719,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.115022,3.953719,-0.17),v(-0.089,3.949,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.089,3.949,-0.17),v(-0.062978,3.953719,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.062978,3.953719,-0.17),v(-0.040917,3.967159,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.040917,3.967159,-0.17),v(-0.026176,3.987274,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.026176,3.987274,-0.17),v(-0.021,4.011,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.157,4.011,-0.17),v(-0.217,4.011,-0.365),0.008),5);
    choose(&mut hit,capsule(p,v(0.157,4.011,-0.17),v(0.151824,4.034726,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.151824,4.034726,-0.17),v(0.137083,4.054841,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.137083,4.054841,-0.17),v(0.115022,4.068281,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.115022,4.068281,-0.17),v(0.089,4.073,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.089,4.073,-0.17),v(0.062978,4.068281,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.062978,4.068281,-0.17),v(0.040917,4.054841,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.040917,4.054841,-0.17),v(0.026176,4.034726,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.026176,4.034726,-0.17),v(0.021,4.011,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.021,4.011,-0.17),v(0.026176,3.987274,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.026176,3.987274,-0.17),v(0.040917,3.967159,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.040917,3.967159,-0.17),v(0.062978,3.953719,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.062978,3.953719,-0.17),v(0.089,3.949,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.089,3.949,-0.17),v(0.115022,3.953719,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.115022,3.953719,-0.17),v(0.137083,3.967159,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.137083,3.967159,-0.17),v(0.151824,3.987274,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.151824,3.987274,-0.17),v(0.157,4.011,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(0.157,4.011,-0.17),v(0.217,4.011,-0.365),0.008),5);
    choose(&mut hit,capsule(p,v(-0.021,4.019,-0.17),v(0.021,4.019,-0.17),0.009),5);
    choose(&mut hit,capsule(p,v(-0.028,3.4,-0.145),v(0.027,3.42,-0.145),0.011),10);
    choose(&mut hit,capsule(p,v(-0.025,3.3,-0.145),v(0.025,3.32,-0.145),0.011),10);
    choose(&mut hit,capsule(p,v(0.083,4.16,-0.198),v(0.06,4.13,-0.19),0.005),9);
    choose(&mut hit,capsule(p,v(0.06,4.13,-0.19),v(0.082,4.13,-0.19),0.005),9);
    choose(&mut hit,capsule(p,v(0.082,4.13,-0.19),v(0.059,4.1,-0.185),0.005),9);
    choose(&mut hit,plume(p,v(0.0,2.18,0.62),v(0.0,2.84,1.07),0.35,0.31),0);
    choose(&mut hit,plume(p,v(-0.19,3.04,1.71),v(-0.31,3.07,1.4),0.047,0.041),0);
    choose(&mut hit,plume(p,v(-0.16,3.02,1.17),v(-0.24,3.32,0.93),0.09,0.055),0);
    choose(&mut hit,plume(p,v(-0.215,3.02,1.17),v(-0.31,3.37,0.93),0.09,0.055),1);
    choose(&mut hit,plume(p,v(-0.35,2.38,0.45),v(-1.75,2.64,0.28),0.39,0.19),0);
    choose(&mut hit,plume(p,v(-1.55,2.62,0.28),v(-3.15,2.66,0.07),0.32,0.14),0);
    choose(&mut hit,plume(p,v(-1.6,2.63,0.12),v(-2.02,2.55,-1.38),0.16,0.066),0);
    choose(&mut hit,plume(p,v(-1.8814,2.5904,-0.885),v(-2.02,2.564,-1.38),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(-1.74,2.624,0.108),v(-2.265,2.525,-1.28),0.16,0.066),0);
    choose(&mut hit,plume(p,v(-2.09175,2.57167,-0.82196),v(-2.265,2.539,-1.28),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(-1.88,2.618,0.096),v(-2.51,2.5,-1.18),0.16,0.066),0);
    choose(&mut hit,plume(p,v(-2.3021,2.55294,-0.75892),v(-2.51,2.514,-1.18),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(-2.02,2.612,0.084),v(-2.755,2.475,-1.08),0.16,0.066),0);
    choose(&mut hit,plume(p,v(-2.51245,2.53421,-0.69588),v(-2.755,2.489,-1.08),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(-2.16,2.606,0.072),v(-3.0,2.45,-0.98),0.16,0.066),0);
    choose(&mut hit,plume(p,v(-2.7228,2.51548,-0.63284),v(-3.0,2.464,-0.98),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(-2.3,2.6,0.06),v(-3.245,2.425,-0.88),0.16,0.066),0);
    choose(&mut hit,plume(p,v(-2.93315,2.49675,-0.5698),v(-3.245,2.439,-0.88),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(-2.44,2.594,0.048),v(-3.49,2.4,-0.78),0.16,0.066),0);
    choose(&mut hit,plume(p,v(-3.1435,2.47802,-0.50676),v(-3.49,2.414,-0.78),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(-2.58,2.588,0.036),v(-3.735,2.375,-0.68),0.16,0.066),0);
    choose(&mut hit,plume(p,v(-3.35385,2.45929,-0.44372),v(-3.735,2.389,-0.68),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(-2.72,2.582,0.024),v(-3.98,2.35,-0.58),0.16,0.066),0);
    choose(&mut hit,plume(p,v(-3.5642,2.44056,-0.38068),v(-3.98,2.364,-0.58),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(-2.86,2.576,0.012),v(-4.225,2.325,-0.48),0.16,0.066),0);
    choose(&mut hit,plume(p,v(-3.77455,2.42183,-0.31764),v(-4.225,2.339,-0.48),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(-0.6,2.48,0.23),v(-0.72,2.4,-1.13),0.17,0.07),0);
    choose(&mut hit,plume(p,v(-0.6804,2.4404,-0.6812),v(-0.72,2.414,-1.13),0.1071,0.0595),1);
    choose(&mut hit,plume(p,v(-0.82,2.498,0.23),v(-1.0,2.41,-1.13),0.17,0.07),0);
    choose(&mut hit,plume(p,v(-0.9406,2.45304,-0.6812),v(-1.0,2.424,-1.13),0.1071,0.0595),1);
    choose(&mut hit,plume(p,v(-1.04,2.516,0.23),v(-1.28,2.42,-1.13),0.17,0.07),0);
    choose(&mut hit,plume(p,v(-1.2008,2.46568,-0.6812),v(-1.28,2.434,-1.13),0.1071,0.0595),1);
    choose(&mut hit,plume(p,v(-1.26,2.534,0.23),v(-1.56,2.43,-1.13),0.17,0.07),0);
    choose(&mut hit,plume(p,v(-1.461,2.47832,-0.6812),v(-1.56,2.444,-1.13),0.1071,0.0595),1);
    choose(&mut hit,plume(p,v(-1.48,2.552,0.23),v(-1.84,2.44,-1.13),0.17,0.07),0);
    choose(&mut hit,plume(p,v(-1.7212,2.49096,-0.6812),v(-1.84,2.454,-1.13),0.1071,0.0595),1);
    choose(&mut hit,plume(p,v(-1.7,2.57,0.23),v(-2.12,2.45,-1.13),0.17,0.07),0);
    choose(&mut hit,plume(p,v(-1.9814,2.5036,-0.6812),v(-2.12,2.464,-1.13),0.1071,0.0595),1);
    choose(&mut hit,plume(p,v(-0.66,2.67,0.4),v(-0.98,2.69,-0.12),0.15,0.085),1);
    choose(&mut hit,plume(p,v(-0.99,2.67,0.38),v(-1.31,2.69,-0.145),0.15,0.085),1);
    choose(&mut hit,plume(p,v(-1.32,2.67,0.36),v(-1.64,2.69,-0.17),0.15,0.085),1);
    choose(&mut hit,plume(p,v(-1.65,2.67,0.34),v(-1.97,2.69,-0.195),0.15,0.085),1);
    choose(&mut hit,plume(p,v(-1.98,2.67,0.32),v(-2.3,2.69,-0.22),0.15,0.085),1);
    choose(&mut hit,plume(p,v(-2.31,2.67,0.3),v(-2.63,2.69,-0.245),0.15,0.085),1);
    choose(&mut hit,plume(p,v(-2.64,2.67,0.28),v(-2.96,2.69,-0.27),0.15,0.085),1);
    choose(&mut hit,plume(p,v(-0.3,1.96,0.71),v(-0.35,1.45,0.82),0.18,0.18),0);
    choose(&mut hit,plume(p,v(-0.36,1.98,-1.03),v(-0.43,1.55,-0.94),0.2,0.2),2);
    choose(&mut hit,plume(p,v(0.19,3.04,1.71),v(0.31,3.07,1.4),0.047,0.041),0);
    choose(&mut hit,plume(p,v(0.16,3.02,1.17),v(0.24,3.32,0.93),0.09,0.055),0);
    choose(&mut hit,plume(p,v(0.215,3.02,1.17),v(0.31,3.37,0.93),0.09,0.055),1);
    choose(&mut hit,plume(p,v(0.35,2.38,0.45),v(1.75,2.64,0.28),0.39,0.19),0);
    choose(&mut hit,plume(p,v(1.55,2.62,0.28),v(3.15,2.66,0.07),0.32,0.14),0);
    choose(&mut hit,plume(p,v(1.6,2.63,0.12),v(2.02,2.55,-1.38),0.16,0.066),0);
    choose(&mut hit,plume(p,v(1.8814,2.5904,-0.885),v(2.02,2.564,-1.38),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(1.74,2.624,0.108),v(2.265,2.525,-1.28),0.16,0.066),0);
    choose(&mut hit,plume(p,v(2.09175,2.57167,-0.82196),v(2.265,2.539,-1.28),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(1.88,2.618,0.096),v(2.51,2.5,-1.18),0.16,0.066),0);
    choose(&mut hit,plume(p,v(2.3021,2.55294,-0.75892),v(2.51,2.514,-1.18),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(2.02,2.612,0.084),v(2.755,2.475,-1.08),0.16,0.066),0);
    choose(&mut hit,plume(p,v(2.51245,2.53421,-0.69588),v(2.755,2.489,-1.08),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(2.16,2.606,0.072),v(3.0,2.45,-0.98),0.16,0.066),0);
    choose(&mut hit,plume(p,v(2.7228,2.51548,-0.63284),v(3.0,2.464,-0.98),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(2.3,2.6,0.06),v(3.245,2.425,-0.88),0.16,0.066),0);
    choose(&mut hit,plume(p,v(2.93315,2.49675,-0.5698),v(3.245,2.439,-0.88),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(2.44,2.594,0.048),v(3.49,2.4,-0.78),0.16,0.066),0);
    choose(&mut hit,plume(p,v(3.1435,2.47802,-0.50676),v(3.49,2.414,-0.78),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(2.58,2.588,0.036),v(3.735,2.375,-0.68),0.16,0.066),0);
    choose(&mut hit,plume(p,v(3.35385,2.45929,-0.44372),v(3.735,2.389,-0.68),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(2.72,2.582,0.024),v(3.98,2.35,-0.58),0.16,0.066),0);
    choose(&mut hit,plume(p,v(3.5642,2.44056,-0.38068),v(3.98,2.364,-0.58),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(2.86,2.576,0.012),v(4.225,2.325,-0.48),0.16,0.066),0);
    choose(&mut hit,plume(p,v(3.77455,2.42183,-0.31764),v(4.225,2.339,-0.48),0.1008,0.0561),1);
    choose(&mut hit,plume(p,v(0.6,2.48,0.23),v(0.72,2.4,-1.13),0.17,0.07),0);
    choose(&mut hit,plume(p,v(0.6804,2.4404,-0.6812),v(0.72,2.414,-1.13),0.1071,0.0595),1);
    choose(&mut hit,plume(p,v(0.82,2.498,0.23),v(1.0,2.41,-1.13),0.17,0.07),0);
    choose(&mut hit,plume(p,v(0.9406,2.45304,-0.6812),v(1.0,2.424,-1.13),0.1071,0.0595),1);
    choose(&mut hit,plume(p,v(1.04,2.516,0.23),v(1.28,2.42,-1.13),0.17,0.07),0);
    choose(&mut hit,plume(p,v(1.2008,2.46568,-0.6812),v(1.28,2.434,-1.13),0.1071,0.0595),1);
    choose(&mut hit,plume(p,v(1.26,2.534,0.23),v(1.56,2.43,-1.13),0.17,0.07),0);
    choose(&mut hit,plume(p,v(1.461,2.47832,-0.6812),v(1.56,2.444,-1.13),0.1071,0.0595),1);
    choose(&mut hit,plume(p,v(1.48,2.552,0.23),v(1.84,2.44,-1.13),0.17,0.07),0);
    choose(&mut hit,plume(p,v(1.7212,2.49096,-0.6812),v(1.84,2.454,-1.13),0.1071,0.0595),1);
    choose(&mut hit,plume(p,v(1.7,2.57,0.23),v(2.12,2.45,-1.13),0.17,0.07),0);
    choose(&mut hit,plume(p,v(1.9814,2.5036,-0.6812),v(2.12,2.464,-1.13),0.1071,0.0595),1);
    choose(&mut hit,plume(p,v(0.66,2.67,0.4),v(0.98,2.69,-0.12),0.15,0.085),1);
    choose(&mut hit,plume(p,v(0.99,2.67,0.38),v(1.31,2.69,-0.145),0.15,0.085),1);
    choose(&mut hit,plume(p,v(1.32,2.67,0.36),v(1.64,2.69,-0.17),0.15,0.085),1);
    choose(&mut hit,plume(p,v(1.65,2.67,0.34),v(1.97,2.69,-0.195),0.15,0.085),1);
    choose(&mut hit,plume(p,v(1.98,2.67,0.32),v(2.3,2.69,-0.22),0.15,0.085),1);
    choose(&mut hit,plume(p,v(2.31,2.67,0.3),v(2.63,2.69,-0.245),0.15,0.085),1);
    choose(&mut hit,plume(p,v(2.64,2.67,0.28),v(2.96,2.69,-0.27),0.15,0.085),1);
    choose(&mut hit,plume(p,v(0.3,1.96,0.71),v(0.35,1.45,0.82),0.18,0.18),0);
    choose(&mut hit,plume(p,v(0.36,1.98,-1.03),v(0.43,1.55,-0.94),0.2,0.2),2);
    choose(&mut hit,plume(p,v(-0.24,2.78,0.99),v(-0.2256,2.56,1.04),0.08,0.045),0);
    choose(&mut hit,plume(p,v(-0.12,2.78,0.99),v(-0.1128,2.56,1.04),0.08,0.045),0);
    choose(&mut hit,plume(p,v(0.0,2.78,0.99),v(0.0,2.56,1.04),0.08,0.045),1);
    choose(&mut hit,plume(p,v(0.12,2.78,0.99),v(0.1128,2.56,1.04),0.08,0.045),0);
    choose(&mut hit,plume(p,v(0.24,2.78,0.99),v(0.2256,2.56,1.04),0.08,0.045),0);
    choose(&mut hit,plume(p,v(-0.24,2.57,0.935),v(-0.2256,2.35,0.985),0.08,0.045),0);
    choose(&mut hit,plume(p,v(-0.12,2.57,0.935),v(-0.1128,2.35,0.985),0.08,0.045),1);
    choose(&mut hit,plume(p,v(0.0,2.57,0.935),v(0.0,2.35,0.985),0.08,0.045),0);
    choose(&mut hit,plume(p,v(0.12,2.57,0.935),v(0.1128,2.35,0.985),0.08,0.045),0);
    choose(&mut hit,plume(p,v(0.24,2.57,0.935),v(0.2256,2.35,0.985),0.08,0.045),1);
    choose(&mut hit,plume(p,v(-0.24,2.36,0.88),v(-0.2256,2.14,0.93),0.08,0.045),1);
    choose(&mut hit,plume(p,v(-0.12,2.36,0.88),v(-0.1128,2.14,0.93),0.08,0.045),0);
    choose(&mut hit,plume(p,v(0.0,2.36,0.88),v(0.0,2.14,0.93),0.08,0.045),0);
    choose(&mut hit,plume(p,v(0.12,2.36,0.88),v(0.1128,2.14,0.93),0.08,0.045),1);
    choose(&mut hit,plume(p,v(0.24,2.36,0.88),v(0.2256,2.14,0.93),0.08,0.045),0);
    choose(&mut hit,plume(p,v(0.08,2.13,-1.92),v(0.17,1.72,-2.52),0.065,0.055),5);
    choose(&mut hit,plume(p,v(0.11,2.13,-1.92),v(0.25,1.76,-2.52),0.065,0.055),5);
    choose(&mut hit,plume(p,v(0.14,2.13,-1.92),v(0.33,1.8,-2.52),0.065,0.055),5);
    choose(&mut hit,plume(p,v(0.17,2.13,-1.92),v(0.41,1.84,-2.52),0.065,0.055),5);
    choose(&mut hit,plume(p,v(-0.22,3.03,-0.42),v(-0.4,2.46,-0.47),0.21,0.1),7);
    choose(&mut hit,plume(p,v(0.22,3.03,-0.42),v(0.4,2.46,-0.47),0.21,0.1),7);
    choose(&mut hit,plume(p,v(-0.13,4.23,-0.26),v(-0.173,4.09,-0.204),0.065,0.045),5);
    choose(&mut hit,plume(p,v(-0.065,4.23,-0.26),v(-0.099,4.045,-0.204),0.065,0.045),5);
    choose(&mut hit,plume(p,v(0.0,4.23,-0.26),v(-0.025,4.09,-0.204),0.065,0.045),5);
    choose(&mut hit,plume(p,v(0.065,4.23,-0.26),v(0.049,4.045,-0.204),0.065,0.045),5);
    choose(&mut hit,plume(p,v(0.13,4.23,-0.26),v(0.123,4.09,-0.204),0.065,0.045),5);
    choose(&mut hit,plume(p,v(0.0,3.55,-0.168),v(0.0,3.19,-0.166),0.043,0.018),9);
    hit
}

fn scene(p: Vec3) -> (f32,usize) {
    let mut hit = (p.y,14);
    let q = (p-v(0.0,2.60,-0.20)).map(|x| x.abs())-v(4.60,1.80,2.50);
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
    let target = v(0.0,2.56,-0.15);
    let eye = target + v(yaw.sin()*elevation.cos(), elevation.sin(), yaw.cos()*elevation.cos())*9.3;
    let forward = (target-eye).normalize();
    let right = cross(&forward,&v(0.0,1.0,0.0)).normalize();
    let up = cross(&right,&forward);
    let scale = (PI/8.0).tan();
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
                    0 => v(0.47,0.52,0.55), // Plumas grises.
                    1 => v(0.72,0.76,0.75), // Bordes y cobertoras claras.
                    2 => v(0.28,0.30,0.32), // Cuerpo equino.
                    3 => v(0.64,0.54,0.35), // Pico y patas de ave.
                    4 => v(0.95,0.57,0.08), // Iris ambar.
                    5 => v(0.018,0.020,0.023), // Pelo, lentes y pupilas.
                    6 => v(0.74,0.47,0.31), // Piel de Harry.
                    7 => v(0.045,0.052,0.065), // Tunica.
                    8 => v(0.81,0.81,0.74), // Camisa.
                    9 => v(0.46,0.025,0.035), // Rojo de corbata y cicatriz.
                    10 => v(0.85,0.56,0.10), // Dorado.
                    11 => v(0.16,0.18,0.21), // Pantalon.
                    12 => v(0.055,0.039,0.030), // Botas y cascos.
                    13 => v(0.93,0.95,0.94), // Blanco de ojos.
                    _ => v(0.095,0.14,0.18), // Piso.
                };
                let ambient = if night {0.075} else {0.30};
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
    render(&mut buffer,0.55,0.20,false);
    let mut file = std::fs::File::create("harry_buckbeak.ppm").expect("No se pudo guardar la imagen");
    write!(file,"P6\n{} {}\n255\n",WIDTH,HEIGHT).unwrap();
    for pixel in buffer {
        file.write_all(&[(pixel>>16) as u8,(pixel>>8) as u8,pixel as u8]).unwrap();
    }
    println!("Imagen guardada en harry_buckbeak.ppm");
}

fn main() {
    if std::env::args().any(|arg| arg == "--preview") { save_preview(); return; }
    let mut window = Window::new("Harry y Buckbeak | Flechas: girar | N: noche | R: reiniciar | Esc: salir",WIDTH,HEIGHT,WindowOptions::default())
        .expect("No se pudo crear la ventana");
    let mut buffer = vec![0;WIDTH*HEIGHT];
    let mut yaw: f32 = 0.55;
    let mut elevation: f32 = 0.27;
    let mut night = false;
    let mut dirty = true;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::N,KeyRepeat::No) { night = !night; dirty = true; }
        if window.is_key_pressed(Key::R,KeyRepeat::No) {
            yaw = 0.55; elevation = 0.27; night = false; dirty = true;
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
