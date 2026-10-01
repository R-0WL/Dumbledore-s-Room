use nalgebra_glm::Vec3;
use crate::util::v;

pub struct Camera {
    pub eye: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub tan_half: f32,
    pub aspect: f32,
}

impl Camera {
    pub fn orbit(target: Vec3, yaw: f32, pitch: f32, dist: f32, fov_deg: f32, aspect: f32) -> Camera {
        let eye = target + v(yaw.sin() * pitch.cos(), pitch.sin(), yaw.cos() * pitch.cos()) * dist;
        let forward = (target - eye).normalize();
        let right = forward.cross(&v(0.0, 1.0, 0.0)).normalize();
        let up = right.cross(&forward);
        Camera { eye, forward, right, up, tan_half: (fov_deg * 0.5).to_radians().tan(), aspect }
    }

    pub fn ray(&self, x: usize, y: usize, w: usize, h: usize) -> Vec3 {
        let sx = (2.0 * (x as f32 + 0.5) / w as f32 - 1.0) * self.aspect * self.tan_half;
        let sy = (1.0 - 2.0 * (y as f32 + 0.5) / h as f32) * self.tan_half;
        (self.forward + self.right * sx + self.up * sy).normalize()
    }
}
