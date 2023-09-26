use std::time::{Instant};
use nalgebra::Vector4;
use winit::event::VirtualKeyCode;

use crate::manifold::{Point, Manifold};

#[derive(Debug, Copy, Clone)]
pub struct Info {
    x: Vector4<f32>,
    y: Vector4<f32>,
    z: Vector4<f32>,
    p: Vector4<f32>,
    focal_length: f32,
    px_size: f32,
    width: f32,
    height: f32,
    delta: f32,
    max_iterations: f32,
    time: Instant
}

impl Info {
    pub fn print_position(&self) {
        println!("{}", self.p);
    }

    pub fn set_sizes(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        bytemuck::cast_slice(&[
            self.x.as_slice(),
            self.y.as_slice(),
            self.z.as_slice(),
            self.p.as_slice(),
            &[
                self.focal_length, self.px_size, self.width, self.height,
                self.delta, self.max_iterations, self.time.elapsed().as_millis() as f32
            ],
            &[0.0, 0.0, 0.0]
        ].concat()[..]).to_owned()
    }

    pub fn movement(&mut self, keycode: VirtualKeyCode, manifold: &impl Manifold) {
        use VirtualKeyCode as VKC;

        let self_info: (&mut Vector4<f32>, &mut Vector4<f32>, f32) = match keycode {
            VKC::D      => (&mut self.p, &mut self.x,  1.0),
            VKC::A      => (&mut self.p, &mut self.x, -1.0),
            VKC::Space  => (&mut self.p, &mut self.y,  1.0),
            VKC::LShift => (&mut self.p, &mut self.y, -1.0),
            VKC::W      => (&mut self.p, &mut self.z,  1.0),
            VKC::S      => (&mut self.p, &mut self.z, -1.0),
            _ => return
        };

        let advanced_point = manifold.advance_point(Point {
            pos: *self_info.0,
            ray: self_info.2 * (*self_info.1)
        }, 0.2);

        *self_info.0 = advanced_point.pos;
        *self_info.1 = self_info.2 * advanced_point.ray;
    }

    pub fn rotate_around_y(&mut self, xz_angle: f32, yz_angle: f32) {
        rotate_between(&mut self.x, &mut self.z, -xz_angle);

        // let xy_angle = self.x.y.atan2(self.x.remove_row(1).magnitude());
        // rotate_between(&mut self.x, &mut self.y, -xy_angle);

        rotate_between(&mut self.y, &mut self.z, -yz_angle);
    }

    pub fn reorient(&mut self) {
        self.x = self.x.normalize();
        self.y = self.y.normalize();
        self.z = self.z.normalize();

        self.y = self.y - self.x * self.x.dot(&self.y);
        self.y = self.y.normalize();

        self.z = self.z - self.x * self.x.dot(&self.z) - self.y * self.y.dot(&self.z);
        self.z = self.z.normalize();
    }
}

fn rotate_between(v1: &mut Vector4<f32>, v2: &mut Vector4<f32>, angle: f32) {
    (*v1, *v2) = (
        *v1 * angle.cos() + *v2 * angle.sin(),
        -*v1 * angle.sin() + *v2 * angle.cos(),
    );
}

impl Default for Info {
    fn default() -> Self {
        Self {
            x: Vector4::x(),
            y: Vector4::y(),
            z: Vector4::z(),
            p: Vector4::zeros(),
            focal_length: 0.5,
            px_size: 1.0,
            width: 0.0,
            height: 0.0,
            delta: 0.02,
            max_iterations: 2000.0,
            time: Instant::now()
        }
    }
}

pub enum Geometry {
    Function(u32)
}

impl Geometry {
    fn get_bytes(&self) -> Vec<u8> {
        bytemuck::cast_slice(&match self {
            Self::Function(id) => [0, *id, 0, 0]
        }).to_owned()
    }
}