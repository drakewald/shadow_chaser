use specs::{System, Entities, Read, Write, ReadStorage, WriteStorage, Join};
use crate::{
    resources::{InputState, PhysicsWorld},
    components::{PhysicsBody, Player, Grounded},
};
use winit::keyboard::KeyCode;
use rapier2d::prelude::*;
use rapier2d::parry::query::ShapeCastOptions;
use log;

pub struct PlayerControlSystem;

impl<'a> System<'a> for PlayerControlSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, InputState>,
        Write<'a, PhysicsWorld>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, PhysicsBody>,
        WriteStorage<'a, Grounded>,
    );

    fn run(&mut self, (entities, input_state, mut physics_world, players, bodies, mut grounded_storage): Self::SystemData) {
        let pw = &mut *physics_world;

        for (entity, _, body) in (&entities, &players, &bodies).join() {
            let collider = pw.collider_set.get(body.collider_handle).unwrap();
            let rigid_body = pw.rigid_body_set.get(body.rigid_body_handle).unwrap();
            
            let was_grounded = grounded_storage.get(entity).is_some();
            let shape = collider.shape();
            let shape_pos = rigid_body.position();
            let shape_vel = vector![0.0, -0.1];
            let filter = QueryFilter::default().exclude_rigid_body(body.rigid_body_handle);

            let cast_options = ShapeCastOptions {
                max_time_of_impact: 0.2,
                target_distance: 0.0,
                stop_at_penetration: true,
                compute_impact_geometry_on_penetration: false,
            };

            let hit = pw.query_pipeline.cast_shape(
                &pw.rigid_body_set, &pw.collider_set, shape_pos, &shape_vel, shape, cast_options, filter
            );
            
            let is_grounded = hit.is_some();

            if is_grounded {
                if !was_grounded {
                    log::info!("Player has landed.");
                }
                grounded_storage.insert(entity, Grounded).unwrap();
            } else {
                if was_grounded {
                    log::info!("Player is airborne.");
                }
                grounded_storage.remove(entity);
            }

            if let Some(rb) = pw.rigid_body_set.get_mut(body.rigid_body_handle) {
                let move_speed = 500.0;
                let jump_impulse = 500.0;
                let current_vel = *rb.linvel();
                let mut desired_vel = vector![0.0, current_vel.y];

                if input_state.pressed_keys.contains(&KeyCode::KeyA) || input_state.pressed_keys.contains(&KeyCode::ArrowLeft) {
                    desired_vel.x = -move_speed;
                } else if input_state.pressed_keys.contains(&KeyCode::KeyD) || input_state.pressed_keys.contains(&KeyCode::ArrowRight) {
                    desired_vel.x = move_speed;
                } else {
                    desired_vel.x = 0.0;
                }

                if is_grounded && input_state.pressed_keys.contains(&KeyCode::Space) {
                    desired_vel.y = jump_impulse;
                    log::info!("Jump initiated! Setting Y velocity to {}", jump_impulse);
                }
                
                rb.set_linvel(desired_vel, true);
            }
        }
    }
}
