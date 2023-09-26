use nalgebra::Vector4;
use winit::event::VirtualKeyCode;
use crate::manifold::{EPSILON, Manifold};

#[derive(Debug)]
pub struct Hyperplane;

impl Manifold for Hyperplane {
    fn description(&self) -> String {
        String::from("Hyperplane")
    }

    fn get_bytes(&self) -> Vec<u8> {
        bytemuck::cast_slice(&[
            Vector4::zeros().as_slice(),
            Vector4::zeros().as_slice(),
            Vector4::zeros().as_slice(),
            &[ 0.0, 0.0, 0.0 ]
        ].concat()[..]).to_owned()
    }

    fn project_onto_wgsl(&self) -> String {
        String::from("return vec4(pos.x, pos.y, pos.z, 0.0);")
    }

    fn project_onto(&self, pos: Vector4<f32>) -> Vector4<f32> {
        pos.xyz().fixed_resize::<4, 1>(0.0)
    }

    fn is_on_curve(&self, pos: Vector4<f32>) -> bool {
        pos.w.abs() < EPSILON
    }
}

#[derive(Debug)]
pub struct Hypersphere {
    radius: f32,
    center: Vector4<f32>
}

impl Hypersphere {
    pub fn new(radius: f32) -> Self {
        Self {
            radius, center: radius * Vector4::w()
        }
    }
}

impl Manifold for Hypersphere {
    fn description(&self) -> String {
        String::from("Hypersphere")
    }

    fn get_bytes(&self) -> Vec<u8> {
        bytemuck::cast_slice(&[
            self.center.as_slice(),
            Vector4::zeros().as_slice(),
            Vector4::zeros().as_slice(),
            &[ self.radius, 0.0, 0.0 ]
        ].concat()[..]).to_owned()
    }

    fn project_onto_wgsl(&self) -> String {
        String::from("return manifold_info.r1 * normalize(pos - manifold_info.v1) + manifold_info.v1;")
    }

    fn project_onto(&self, pos: Vector4<f32>) -> Vector4<f32> {
        self.radius * (pos - self.center).normalize() + self.center
    }

    fn is_on_curve(&self, mut pos: Vector4<f32>) -> bool {
        pos -= self.center;

        pos.x.powi(2) +
            pos.y.powi(2) +
            pos.z.powi(2) +
            pos.w.powi(2) -
            self.radius.powi(2) < EPSILON
    }
}

#[derive(Debug)]
pub struct Hypersphube {
    exponent: f32,
    radius: f32,
    center: Vector4<f32>
}

impl Hypersphube {
    pub fn new(radius: f32, exponent: f32) -> Self {
        Self {
            exponent: exponent, radius, center: radius * Vector4::w()
        }
    }
}

impl Manifold for Hypersphube {
    fn description(&self) -> String {
        String::from("Hypersphube")
    }

    fn project_onto_wgsl(&self) -> String {
        String::from("\
            let new_pos: vec4<f32> = pos - manifold_info.v1;\
      \n    \
      \n    return manifold_info.v1 + manifold_info.r1 * (new_pos / pow(\
      \n        pow(abs(new_pos.x), manifold_info.r2) + \
      \n        pow(abs(new_pos.y), manifold_info.r2) + \
      \n        pow(abs(new_pos.z), manifold_info.r2) + \
      \n        pow(abs(new_pos.w), manifold_info.r2)\
      \n    , 1.0 / manifold_info.r2));\
        ")
    }

    fn project_onto(&self, pos: Vector4<f32>) -> Vector4<f32> {
        let new_pos = pos - self.center;

        self.center + self.radius * (new_pos / (
            new_pos.x.abs().powf(self.exponent) +
            new_pos.y.abs().powf(self.exponent) +
            new_pos.z.abs().powf(self.exponent) +
            new_pos.w.abs().powf(self.exponent)
        ).powf(1.0 / (self.exponent as f32)))
    }

    fn is_on_curve(&self, mut pos: Vector4<f32>) -> bool {
        pos -= self.center;

        pos.x.abs().powf(self.exponent) +
            pos.y.abs().powf(self.exponent) +
            pos.z.abs().powf(self.exponent) +
            pos.w.abs().powf(self.exponent) -
            self.radius.abs().powf(self.exponent) < EPSILON
    }

    fn get_bytes(&self) -> Vec<u8> {
        bytemuck::cast_slice(&[
            self.center.as_slice(),
            Vector4::zeros().as_slice(),
            Vector4::zeros().as_slice(),
            &[ self.radius, self.exponent, 0.0 ]
        ].concat()[..]).to_owned()
    }
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct Ditorus {
    radius_major_major: f32,
    radius_major_minor: f32,
    radius_minor_minor: f32,
    center: Vector4<f32>
}

impl Ditorus {
    #[allow(non_snake_case)]
    pub fn new(radius_major_major: f32, radius_major_minor: f32, radius_minor_minor: f32) -> Self {
        Self {
            radius_major_major,
            radius_major_minor,
            radius_minor_minor,
            center: (radius_major_major + radius_major_minor + radius_minor_minor) * Vector4::w()
        }
    }
}

impl Manifold for Ditorus {
    fn description(&self) -> String {
        String::from("Ditorus")
    }

    fn project_onto_wgsl(&self) -> String {
        String::from("\
            let pos2 = pos - manifold_info.v1;\
      \n    var new_pos: vec4<f32> = vec4(0.0, 0.0, 0.0, 0.0);\
      \n    \
      \n    new_pos += manifold_info.r1 * normalize(vec4(pos2.x, 0.0, 0.0, pos2.w));\
      \n    new_pos += manifold_info.r2 * normalize(vec4(pos2.x, 0.0, pos2.z, pos2.w) - new_pos);\
      \n    new_pos += manifold_info.r3 * normalize(pos2 - new_pos);\
      \n    \
      \n    return new_pos + manifold_info.v1;\
        ")
    }

    fn project_onto(&self, mut pos: Vector4<f32>) -> Vector4<f32> {
        pos -= self.center;
        let mut new_pos = Vector4::zeros();

        if pos.x == 0.0 && pos.w == 0.0 {
            pos.x += 0.0001;
        }

        new_pos += self.radius_major_major *
            Vector4::new(pos.x, 0.0, 0.0, pos.w)
                .normalize();

        if pos.x == new_pos.x && pos.z == 0.0 && pos.w == new_pos.w {
            pos.x += 0.0001;
        }

        new_pos += self.radius_major_minor *
            (Vector4::new(pos.x, 0.0, pos.z, pos.w) - new_pos)
                .normalize();

        if (pos - new_pos).magnitude() == 0.0 {
            pos.w += 0.0001;
        }

        new_pos += self.radius_minor_minor *
            (pos - new_pos)
                .normalize();

        new_pos + self.center
    }

    fn is_on_curve(&self, mut pos: Vector4<f32>) -> bool {
        pos -= self.center;

        ((
            ((
                (pos.x.powi(2) + pos.w.powi(2)).sqrt() - self.radius_major_major).powi(2) +
                pos.z.powi(2)
            ).sqrt() - self.radius_major_minor
        ).powi(2) + pos.y.powi(2) - self.radius_minor_minor.powi(2)).abs() < EPSILON
    }

    fn get_bytes(&self) -> Vec<u8> {
        bytemuck::cast_slice(&[
            self.center.as_slice(),
            Vector4::zeros().as_slice(),
            Vector4::zeros().as_slice(),
            &[self.radius_major_major, self.radius_major_minor, self.radius_minor_minor]
        ].concat()[..]).to_owned()
    }

    fn change_on_keybinds(&mut self, key_code: &VirtualKeyCode) {
        const AMOUNT: f32 = 0.1;

        match key_code {
            VirtualKeyCode::T => self.radius_major_major += AMOUNT,
            VirtualKeyCode::G => self.radius_major_major -= AMOUNT,
            VirtualKeyCode::Y => self.radius_major_minor += AMOUNT,
            VirtualKeyCode::H => self.radius_major_minor -= AMOUNT,
            VirtualKeyCode::U => self.radius_minor_minor += AMOUNT,
            VirtualKeyCode::J => self.radius_minor_minor -= AMOUNT,
            _ => {}
        }
    }
}
/*
pub enum TigerPairing {
    XyZw,
    XzWy,
    XwZy
}

impl TigerPairing {
    pub fn get_ordered_radii(&self, radius_major1: f32, radius_major2: f32) -> (f32, f32) {
        match self {
            Self::XwZy => (radius_major1, radius_major2),
            _          => (radius_major2, radius_major1)
        }
    }
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct Tiger {
    radius_major1: f32,
    radius_major2: f32,
    radius_minor: f32,
    tiger_pairing: TigerPairing,
    center: Vector4<f32>
}

impl Tiger {
    #[allow(non_snake_case)]
    pub fn new(
        radius_major1: f32,
        radius_major2: f32,
        radius_minor: f32,
        tiger_pairing: TigerPairing,
        start_on_inside: bool
    ) -> Self {
        let (
            radius_major_w,
            radius_major_nw
        ) = tiger_pairing.get_ordered_radii(radius_major1, radius_major2);

        Self {
            radius_major1,
            radius_major2,
            radius_minor,
            tiger_pairing,
            center: (
                radius_major_w + if start_on_inside { -1.0 } else { 1.0 } * (
                    radius_minor.powi(2) - radius_major_nw.powi(2)
                ).sqrt()
            ) * Vector4::w()
        }
    }
}

impl Manifold for Tiger {
    fn description(&self) -> String {
        String::from("Tiger")
    }

    fn project_onto_wgsl(&self) -> String {
        String::from("\
            let pos2 = pos - manifold_info.v1;
      \n    var new_pos: vec4<f32> = vec4(0.0, 0.0, 0.0, 0.0);\
      \n    \
      \n    new_pos += manifold_info.r1 * normalize(vec4(pos2.x, 0.0, 0.0, pos2.w));\
      \n    new_pos += manifold_info.r2 * normalize(vec4(pos2.x, 0.0, pos2.z, pos2.w) - new_pos);\
      \n    new_pos += manifold_info.r3 * normalize(pos2 - new_pos);\
      \n    \
      \n    return new_pos + manifold_info.v1;\
        ")
    }

    fn project_onto(&self, mut pos: Vector4<f32>) -> Vector4<f32> {
        pos -= self.center;
        let mut new_pos = Vector4::zeros();

        if pos.x == 0.0 && pos.w == 0.0 {
            pos.x += 0.0001;
        }

        new_pos += self.radius_major_major *
            Vector4::new(pos.x, 0.0, 0.0, pos.w)
                .normalize();

        if pos.x == new_pos.x && pos.z == 0.0 && pos.w == new_pos.w {
            pos.x += 0.0001;
        }

        new_pos += self.radius_major_minor *
            (Vector4::new(pos.x, 0.0, pos.z, pos.w) - new_pos)
                .normalize();

        if (pos - new_pos).magnitude() == 0.0 {
            pos.w += 0.0001;
        }

        new_pos += self.radius_minor_minor *
            (pos - new_pos)
                .normalize();

        new_pos + self.center
    }

    fn is_on_curve(&self, mut pos: Vector4<f32>) -> bool {
        pos -= self.center;

        match self.tiger_pairing {
            TigerPairing::XyZw =>
                ((pos.x.powi(2) + pos.y.powi(2)).sqrt() - self.radius_major1).powi(2) +
                    ((pos.z.powi(2) + pos.w.powi(2)).sqrt() - self.radius_major2).powi(2),
            TigerPairing::XzWy =>
                ((pos.x.powi(2) + pos.z.powi(2)).sqrt() - self.radius_major1).powi(2) +
                    ((pos.w.powi(2) + pos.y.powi(2)).sqrt() - self.radius_major2).powi(2),
            TigerPairing::XwZy =>
                ((pos.x.powi(2) + pos.w.powi(2)).sqrt() - self.radius_major1).powi(2) +
                    ((pos.z.powi(2) + pos.y.powi(2)).sqrt() - self.radius_major2).powi(2),
        } - self.radius_minor.powi(2) < EPSILON
    }

    fn get_bytes(&self) -> Vec<u8> {
        bytemuck::cast_slice(&[
            self.center.as_slice(),
            Vector4::zeros().as_slice(),
            Vector4::zeros().as_slice(),
            &[ self.radius_major1, self.radius_major2, self.radius_minor]
        ].concat()[..]).to_owned()
    }
}*/