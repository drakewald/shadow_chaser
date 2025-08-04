use specs::{System, Write, ReadStorage, Join};
use crate::{
    components::{Player, Position},
    resources::CameraUniform,
};
use rapier2d::na::{Vector3, Matrix4};

// A simple camera struct to hold its state
pub struct Camera {
    pub position: Vector3<f32>,
    projection: Matrix4<f32>,
}

// CORRECTED: Implement Default for the Camera struct
impl Default for Camera {
    fn default() -> Self {
        Self::new(1, 1) // Default to 1x1, will be overwritten immediately
    }
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 1.0),
            projection: Matrix4::new_orthographic(
                - (width as f32 / 2.0),
                width as f32 / 2.0,
                - (height as f32 / 2.0),
                height as f32 / 2.0,
                0.1,
                100.0,
            ),
        }
    }
}

pub struct CameraSystem;

impl<'a> System<'a> for CameraSystem {
    type SystemData = (
        Write<'a, Camera>,
        Write<'a, CameraUniform>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, (mut camera, mut camera_uniform, players, positions): Self::SystemData) {
        for (_, pos) in (&players, &positions).join() {
            let target_pos = Vector3::new(pos.0.x, pos.0.y, 1.0);
            let lerp_factor = 0.1;
            camera.position = camera.position * (1.0 - lerp_factor) + target_pos * lerp_factor;
        }

        camera_uniform.update_view_proj(&camera.position, &camera.projection);
    }
}
