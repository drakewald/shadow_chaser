// src/systems/player_control.rs

use specs::{System, Entities, Read, Write, ReadStorage, WriteStorage, Join};
use crate::{
    resources::{InputState, PhysicsWorld},
    components::{PhysicsBody, Player, Grounded, CharacterController},
};
use winit::keyboard::KeyCode;
use rapier2d::prelude::*;
use rapier2d::na::Vector2;
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
        WriteStorage<'a, CharacterController>,
    );

    fn run(&mut self, (entities, input, mut physics, _players, bodies, mut grounded_storage, mut controllers): Self::SystemData) {
        let dt = physics.integration_parameters.dt;

        for (entity, body, controller) in (&entities, &bodies, &mut controllers).join() {
            // --- 1. Get Initial State ---
            let move_speed = 400.0;
            let jump_velocity = 700.0;
            let gravity = physics.gravity;
            let filter = QueryFilter::new().exclude_rigid_body(body.rigid_body_handle);

            let is_grounded = grounded_storage.get(entity).is_some();

            // --- 2. Calculate Velocity ---
            // Apply gravity
            controller.velocity += gravity * dt;
            // If we are grounded, clamp vertical velocity to prevent gravity build-up
            if is_grounded {
                controller.velocity.y = f32::max(controller.velocity.y, -1.0);
            }

            // Horizontal velocity
            let pressing_left = input.pressed_keys.contains(&KeyCode::KeyA) || input.pressed_keys.contains(&KeyCode::ArrowLeft);
            let pressing_right = input.pressed_keys.contains(&KeyCode::KeyD) || input.pressed_keys.contains(&KeyCode::ArrowRight);
            let desired_x_vel = if pressing_left { -move_speed } else if pressing_right { move_speed } else { 0.0 };
            controller.velocity.x += (desired_x_vel - controller.velocity.x) * 0.2;

            // --- JUMP LOGIC ---
            // **THE FIX**: A jump command overrides horizontal movement for one frame.
            if is_grounded && input.jump_pressed {
                // Apply the full vertical jump velocity.
                controller.velocity.y = jump_velocity;
                // **CRITICAL**: Zero out horizontal velocity for this frame. This ensures
                // the character jumps straight up without interference from an adjacent wall.
                // Air control will resume on the next frame.
                controller.velocity.x = 0.0;
            }

            // --- 3. Separate Axis Movement ---
            let mut current_position = *physics.rigid_body_set.get(body.rigid_body_handle).unwrap().position();

            // --- MOVE HORIZONTALLY ---
            let horizontal_movement = Vector2::new(controller.velocity.x * dt, 0.0);
            if horizontal_movement.x.abs() > 1e-6 {
                let horizontal_collisions = controller.controller.move_shape(
                    dt, &physics.rigid_body_set, &physics.collider_set, &physics.query_pipeline,
                    physics.collider_set.get(body.collider_handle).unwrap().shape(),
                    &current_position, horizontal_movement, filter, &mut |_| {},
                );
                current_position.translation.vector += horizontal_collisions.translation;
            }

            // --- MOVE VERTICALLY ---
            let vertical_movement = Vector2::new(0.0, controller.velocity.y * dt);
            let vertical_collisions = controller.controller.move_shape(
                dt, &physics.rigid_body_set, &physics.collider_set, &physics.query_pipeline,
                physics.collider_set.get(body.collider_handle).unwrap().shape(),
                &current_position, vertical_movement, filter, &mut |_| {},
            );
            current_position.translation.vector += vertical_collisions.translation;

            // --- 4. Update State and Final Position ---
            log::info!("[Movement] Grounded: {}, Vel: ({:.2}, {:.2}), Final Pos: ({:.2}, {:.2})", 
                vertical_collisions.grounded,
                controller.velocity.x, controller.velocity.y, 
                current_position.translation.x, current_position.translation.y);

            if vertical_collisions.grounded {
                grounded_storage.insert(entity, Grounded).ok();
            } else {
                grounded_storage.remove(entity);
            }

            // Update our persistent velocity based on the actual movement that occurred.
            // This is crucial for the next frame's calculations.
            let actual_translation = current_position.translation.vector - physics.rigid_body_set.get(body.rigid_body_handle).unwrap().position().translation.vector;
            if dt > 0.0 {
                controller.velocity = actual_translation / dt;
            }

            if let Some(rb) = physics.rigid_body_set.get_mut(body.rigid_body_handle) {
                rb.set_next_kinematic_position(current_position);
            }
        }
    }
}
