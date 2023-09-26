use nalgebra::{Vector3, Vector4};
use winit::event::VirtualKeyCode;
use crate::manifold::EPSILON;
use super::Manifold;

pub trait Shape3D: std::fmt::Debug {
    fn description(&self) -> String {
        String::from("3D shape")
    }

    fn project_onto_wgsl(&self) -> String;

    fn project_onto(&self, pos: Vector3<f32>) -> Vector3<f32>;

    fn is_on_curve(&self, pos: Vector3<f32>) -> bool {
        panic!("{} does not currently impl is_on_curve", self.description())
    }

    fn get_data(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>, f32, f32, f32);
}

#[derive(Debug)]
pub struct ExtrudedShape<SHAPE: Shape3D>(pub SHAPE);

impl<SHAPE: Shape3D> Manifold for ExtrudedShape<SHAPE> {
    fn description(&self) -> String {
        format!("Extruded: {}", self.0.description())
    }
    
    fn project_onto_wgsl(&self) -> String {
        format!("\
            let pos2 = project_onto_3d_part(vec3(pos.x, pos.y, pos.w)); \
      \n    return vec4(pos2.x, pos2.y, pos.z, pos2.z);\
      \n}}\
      \n\
      \nfn project_onto_3d_part(pos: vec3<f32>) -> vec3<f32> {{\
      \n    {}\
        ", self.0.project_onto_wgsl())
    }

    fn project_onto(&self, pos: Vector4<f32>) -> Vector4<f32> {
        let proj_pos = self.0.project_onto(Vector3::new(pos.x, pos.y, pos.w));
        
        Vector4::new(proj_pos.x, proj_pos.y, pos.z, proj_pos.z)
    }

    fn is_on_curve(&self, pos: Vector4<f32>) -> bool {
        self.0.is_on_curve(Vector3::new(pos.x, pos.y, pos.w))
    }

    fn get_bytes(&self) -> Vec<u8> {
        let (
            v1, v2, v3,
            r1, r2, r3
        ) = self.0.get_data();

        bytemuck::cast_slice(&[
            v1.fixed_resize::<4, 1>(0.0).as_slice(),
            v2.fixed_resize::<4, 1>(0.0).as_slice(),
            v3.fixed_resize::<4, 1>(0.0).as_slice(),
            &[r1, r2, r3]
        ].concat()[..]).to_owned()
    }
}
#[derive(Debug)]
pub struct Sphere {
    radius: f32,
    center: Vector3<f32>
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        Self {
            radius, center: radius * Vector3::z()
        }
    }
}

impl Shape3D for Sphere {
    fn description(&self) -> String {
        String::from("Sphere")
    }

    fn get_data(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>, f32, f32, f32) {
        (
            self.center,
            Vector3::zeros(),
            Vector3::zeros(),

            self.radius,
            0.0,
            0.0
        )
    }

    fn project_onto_wgsl(&self) -> String {
        String::from("\
            let center = vec3(manifold_info.v1.x, manifold_info.v1.y, manifold_info.v1.z);\
      \n    return manifold_info.r1 * normalize(pos - center) + center;\
        ")
    }

    fn project_onto(&self, pos: Vector3<f32>) -> Vector3<f32> {
        self.radius * (pos - self.center).normalize() + self.center
    }

    fn is_on_curve(&self, mut pos: Vector3<f32>) -> bool {
        pos -= self.center;

        pos.x.powi(2) +
            pos.y.powi(2) +
            pos.z.powi(2) -
            self.radius.powi(2) < EPSILON
    }
}

#[derive(Debug)]
pub struct Torus {
    radius_major: f32,
    radius_minor: f32,
    center: Vector3<f32>
}

impl Torus {
    pub fn new(radius_major: f32, radius_minor: f32) -> Self {
        Self {
            radius_major, radius_minor,
            center: (radius_major + radius_minor) * Vector3::z()
        }
    }
}

impl Shape3D for Torus {
    fn description(&self) -> String {
        String::from("Torus")
    }

    fn get_data(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>, f32, f32, f32) {
        (
            self.center,
            Vector3::zeros(),
            Vector3::zeros(),

            self.radius_major,
            self.radius_minor,
            0.0
        )
    }

    fn project_onto_wgsl(&self) -> String {
        String::from("\
            let center = vec3(manifold_info.v1.x, manifold_info.v1.y, manifold_info.v1.z);\
            let pos2 = pos - center;\
      \n    var new_pos: vec3<f32> = vec3(0.0, 0.0, 0.0);\
      \n    \
      \n    new_pos += manifold_info.r1 * normalize(vec3(pos2.x, 0.0, pos.z));\
      \n    new_pos += manifold_info.r2 * normalize(pos2 - new_pos);\
      \n    \
      \n    return new_pos + center;\
        ")
    }

    fn project_onto(&self, mut pos: Vector3<f32>) -> Vector3<f32> {
        pos -= self.center;
        let mut new_pos = Vector3::zeros();

        if pos.x == 0.0 && pos.z == 0.0 {
            pos.x += 0.0001;
        }

        new_pos += self.radius_major *
            Vector3::new(pos.x, 0.0, pos.z)
                .normalize();

        if (pos - new_pos).magnitude() == 0.0 {
            pos.y += 0.0001;
        }

        new_pos += self.radius_minor *
            (pos - new_pos)
                .normalize();

        new_pos + self.center
    }
}