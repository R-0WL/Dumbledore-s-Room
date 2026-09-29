use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::Vec3;

const EPSILON: f32 = 1e-4;

pub struct Cube {
    pub center: Vec3,
    pub size: f32,
    pub material: Material,
}

impl Cube {
    pub fn new(center: Vec3, size: f32, material: Material) -> Self {
        Self {
            center,
            size,
            material,
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let half = self.size / 2.0;

        let min_bound = self.center - Vec3::new(half, half, half);
        let max_bound = self.center + Vec3::new(half, half, half);

        let mut t_enter = f32::NEG_INFINITY;
        let mut t_exit = f32::INFINITY;

        let mut enter_normal = Vec3::new(0.0, 0.0, 0.0);
        let mut exit_normal = Vec3::new(0.0, 0.0, 0.0);

        for axis in 0..3 {
            let origin = ray_origin[axis];
            let direction = ray_direction[axis];

            if direction.abs() < EPSILON {
                if origin < min_bound[axis] || origin > max_bound[axis] {
                    return None;
                }

                continue;
            }

            let t1 = (min_bound[axis] - origin) / direction;
            let t2 = (max_bound[axis] - origin) / direction;

            let mut negative_normal = Vec3::new(0.0, 0.0, 0.0);
            negative_normal[axis] = -1.0;

            let mut positive_normal = Vec3::new(0.0, 0.0, 0.0);
            positive_normal[axis] = 1.0;

            let (near_t, far_t, near_normal, far_normal) = if t1 <= t2 {
                (t1, t2, negative_normal, positive_normal)
            } else {
                (t2, t1, positive_normal, negative_normal)
            };

            if near_t > t_enter {
                t_enter = near_t;
                enter_normal = near_normal;
            }

            if far_t < t_exit {
                t_exit = far_t;
                exit_normal = far_normal;
            }

            if t_enter > t_exit {
                return None;
            }
        }

        let (distance, normal) = if t_enter > EPSILON {
            (t_enter, enter_normal)
        } else if t_exit > EPSILON {
            (t_exit, exit_normal)
        } else {
            return None;
        };

        Some(Intersect {
            point: ray_origin + ray_direction * distance,
            normal,
            distance,
            material: self.material,
        })
    }
}
