use specs::{Component, VecStorage, NullStorage};
use specs_derive::Component;
use rapier2d::prelude::{RigidBodyHandle, ColliderHandle};
use rapier2d::na::Vector2;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position(pub Vector2<f32>);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Renderable {
    pub color: [f32; 4],
    pub width: f32,
    pub height: f32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct PhysicsBody {
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
}

#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct Player;

// A simple component to track if the player is on the ground.
#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct Grounded;
