// src/components.rs

use specs::{Component, VecStorage, NullStorage};
use specs_derive::Component;
use rapier2d::prelude::{RigidBodyHandle, ColliderHandle};
use rapier2d::na::Vector2;
// **NEW IMPORT**
use rapier2d::control::KinematicCharacterController;

/// A component representing an entity's position in the game world.
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position(pub Vector2<f32>);

/// A component that makes an entity renderable as a colored quad.
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Renderable {
    pub color: [f32; 4],
    pub width: f32,
    pub height: f32,
}

/// A component that holds handles to the entity's physics bodies in the rapier2d world.
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct PhysicsBody {
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
}

/// A marker component to identify the player entity.
#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct Player;

/// A marker component to track if the player is on the ground.
#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct Grounded;

/// **NEW DEFINITION**
/// This component now holds the actual Rapier KinematicCharacterController
/// and the player's current velocity.
#[derive(Component)]
#[storage(VecStorage)]
pub struct CharacterController {
    pub controller: KinematicCharacterController,
    pub velocity: Vector2<f32>,
}
