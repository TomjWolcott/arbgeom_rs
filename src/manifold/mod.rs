pub mod shapes4D;
pub mod shapes3D;

use nalgebra::Vector4;
use winit::event::VirtualKeyCode;

const EPSILON: f32 = 0.01;

pub struct Point {
    pub pos: Vector4<f32>,
    pub ray: Vector4<f32>
}

pub trait Manifold: std::fmt::Debug {
    fn description(&self) -> String {
        String::from("Manifold")
    }

    // fn starting_position(&self) -> (Vector4<f32>) {
    //
    // }

    fn project_onto_wgsl(&self) -> String;

    fn project_onto(&self, pos: Vector4<f32>) -> Vector4<f32>;

    fn advance_point(&self, point: Point, delta: f32) -> Point {
        let new_pos = self.project_onto(
            point.pos + delta * point.ray
        );

        Point {
            pos: new_pos,
            ray: (new_pos - point.pos).normalize()
        }
    }

    fn is_on_curve(&self, pos: Vector4<f32>) -> bool {
        panic!("{} does not currently impl is_on_curve", self.description())
    }

    fn get_bytes(&self) -> Vec<u8>;

    fn insert_into_wgsl(&self, mut wgsl: String) -> Option<String> {
        const FN_HEADER: &'static str = "fn project_onto_curve(pos: vec4<f32>) -> vec4<f32> {";

        if let Some(index) = wgsl.find(FN_HEADER) {
            wgsl.insert_str(index + FN_HEADER.len(), format!("\n    {}", self.project_onto_wgsl()).as_str());

            Some(wgsl)
        } else {
            None
        }
    }

    fn change_on_keybinds(&mut self, _key_code: &VirtualKeyCode) {

    }
}

#[cfg(test)]
mod tests {
    use crate::manifold::shapes4D::*;
    use super::*;

    #[test]
    fn test_all_manifolds() {
        let manifolds: Vec<Box<dyn Manifold>> = vec![
            Box::new(Hyperplane),
            Box::new(Hypersphere::new(3.0)),
            Box::new(Hypersphube::new(3.0, 6.0)),
            Box::new(Ditorus::new(4.0, 2.0, 1.0)),
        ];
        for manifold in manifolds {
            println!("\n{}:", manifold.description());

            for i in -5..5 {
                for j in -5..5 {
                    for k in -5..5 {
                        let pos = Vector4::new(i as f32, j as f32, k as f32, 0.0);

                        let pos2 = manifold.project_onto(pos);

                        println!("    {:?} => {:?}", pos, pos2);

                        assert!(manifold.is_on_curve(pos2));
                    }
                }
            }
        }
    }
}
