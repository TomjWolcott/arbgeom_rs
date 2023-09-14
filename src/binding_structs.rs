use nalgebra::{Vector4, Matrix4};

#[derive(Debug)]
pub struct Info {
    orientation: Matrix4<f32>,
    p: Vector4<f32>,
    focal_length: f32,
    px_size: f32,
    width: f32,
    height: f32,
    delta: f32,
    max_iterations: f32,
    pub time: f32
}

impl Info {
    pub fn set_sizes(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn get_bytes<'a>(&'a self) -> Vec<u8> {
        bytemuck::cast_slice(&[
            self.orientation.remove_column(3).as_slice(),
            self.p.as_slice(),
            &[self.focal_length, self.px_size, self.width, self.height, self.delta, self.max_iterations, self.time],
            &[0.0, 0.0, 0.0]
        ].concat()[..]).to_owned()
    }

    pub fn rotate_between(&mut self, i0: usize, i1: usize, angle: f32) {
        let mut rotation = Matrix4::zeros();

        rotation[(i0, i0)] = angle.cos();
        rotation[(i0, i1)] = angle.sin();
        rotation[(i1, i0)] = -angle.sin();
        rotation[(i1, i1)] = angle.cos();

        println!("rot1: {:?}", rotation);

        for i in 0..4 {
            if i != i0 && i != i1 {
                rotation[(i, i)] = 1.0;
            }
        }
        println!("rot2: {:?}", rotation);

        self.orientation *= rotation;
    }

    pub fn rotate_around_y(&mut self, xz_angle: f32, yz_angle: f32) {
        println!("{:?}", self.orientation);

        self.rotate_between(0, 2, xz_angle);
        self.rotate_between(1, 2, yz_angle);

        // self.p = -1.0 * self.orientation.column(2);

        // let xy_angle = (self.orientation.m12).atan2(self.orientation.m22);
        // self.rotate_between(0, 1, xy_angle);

        println!("{:?}", self.orientation);
    }
}

impl Default for Info {
    fn default() -> Self {
        Self {
            orientation: Matrix4::identity(),
            p: -5.0f32 * Vector4::w() + Vector4::y(),
            focal_length: 4.0,
            px_size: 1.0,
            width: 0.0,
            height: 0.0,
            delta: 0.01,
            max_iterations: 10000.0,
            time: 0.0
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