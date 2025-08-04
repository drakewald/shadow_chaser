use specs::{System, Write, WriteStorage, ReadStorage, Join};
use crate::{
    resources::PhysicsWorld,
    components::{Position, PhysicsBody},
};

pub struct PhysicsSystem;

impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (
        Write<'a, PhysicsWorld>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, PhysicsBody>,
    );

    fn run(&mut self, (mut physics_world, mut positions, bodies): Self::SystemData) {
        let pw = &mut *physics_world;

        pw.physics_pipeline.step(
            &pw.gravity,
            &pw.integration_parameters,
            &mut pw.island_manager,
            &mut pw.broad_phase,
            &mut pw.narrow_phase,
            &mut pw.rigid_body_set,
            &mut pw.collider_set,
            &mut pw.impulse_joint_set,
            &mut pw.multibody_joint_set,
            &mut pw.ccd_solver,
            None,
            &pw.physics_hooks,
            &pw.event_handler,
        );

        // Update our Position component from the physics world's state
        for (pos, body) in (&mut positions, &bodies).join() {
            if let Some(rigid_body) = pw.rigid_body_set.get(body.rigid_body_handle) {
                let translation = rigid_body.translation();
                pos.0.x = translation.x;
                pos.0.y = translation.y;
            }
        }
    }
}
