use specs::{System, Entities, Read, Write, ReadStorage, WriteStorage, Join};
use crate::{
    resources::{InputState, PhysicsWorld},
    components::{PhysicsBody, Player, Grounded},
};
use winit::keyboard::KeyCode;
use rapier2d::prelude::*;
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
            
            // --- Ground Detection (Corrected Method) ---
            let was_grounded = grounded_storage.get(entity).is_some();
            let mut is_grounded = false;

            // We iterate through all contact pairs involving the player's collider.
            for contact_pair in pw.narrow_phase.contact_pairs_with(body.collider_handle) {
                // The contact normal tells us the direction of the collision.
                for manifold in &contact_pair.manifolds {
                    // The normal points "out" of the surface. To check if we are on the ground,
                    // we need to see if the contact normal on the player's body is pointing downwards.
                    let normal = if contact_pair.collider1 == body.collider_handle {
                        manifold.local_n1
                    } else {
                        manifold.local_n2
                    };

                    // A normal pointing down (a large negative Y value) means the contact is on the player's feet.
                    if normal.y < -0.7 {
                        is_grounded = true;
                        break; // Found ground, no need to check other manifolds
                    }
                }
                if is_grounded {
                    break; // Found ground, no need to check other contact pairs
                }
            }

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

            // --- Movement Control ---
            if let Some(rb) = pw.rigid_body_set.get_mut(body.rigid_body_handle) {
                let move_speed = 500.0;
                let jump_impulse = 400.0;
                let current_vel = *rb.linvel();
                let mut desired_vel = vector![0.0, current_vel.y];

                // Horizontal movement
                if input_state.pressed_keys.contains(&KeyCode::KeyA) || input_state.pressed_keys.contains(&KeyCode::ArrowLeft) {
                    desired_vel.x = -move_speed;
                } else if input_state.pressed_keys.contains(&KeyCode::KeyD) || input_state.pressed_keys.contains(&KeyCode::ArrowRight) {
                    desired_vel.x = move_speed;
                } else {
                    desired_vel.x = 0.0;
                }

                // Jumping
                if is_grounded && input_state.pressed_keys.contains(&KeyCode::Space) {
                    desired_vel.y = jump_impulse;
                    log::info!("Jump initiated! Setting Y velocity to {}", jump_impulse);
                }
                
                rb.set_linvel(desired_vel, true);
            }
        }
    }
}
