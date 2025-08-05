// src/resources.rs

use rapier2d::prelude::*;
use std::collections::HashSet;
use winit::keyboard::KeyCode;
use crossbeam::channel::{unbounded, Receiver};

/// A resource that holds the entire rapier2d physics simulation state.
pub struct PhysicsWorld {
    pub gravity: nalgebra::Vector2<f32>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    // **ADD THE QUERY PIPELINE**
    // This is essential for casting shapes and rays for our controller.
    pub query_pipeline: QueryPipeline,
    pub physics_hooks: (),
    pub event_handler: ChannelEventCollector,
    pub _collision_event_receiver: Receiver<CollisionEvent>,
    pub _contact_force_event_receiver: Receiver<ContactForceEvent>,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        let (collision_sender, collision_receiver) = unbounded();
        let (contact_force_sender, contact_force_receiver) = unbounded();
        let event_handler = ChannelEventCollector::new(collision_sender, contact_force_sender);

        Self {
            gravity: nalgebra::vector![0.0, -2000.0], // Gravity can be tuned now
            integration_parameters: IntegrationParameters { dt: 1.0 / 60.0, ..Default::default() },
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(), // Initialize the query pipeline
            physics_hooks: (),
            event_handler,
            _collision_event_receiver: collision_receiver,
            _contact_force_event_receiver: contact_force_receiver,
        }
    }
}

/// A resource to hold the vertex data that needs to be rendered each frame.
#[derive(Default)]
pub struct RenderData(pub Vec<Vertex>);

/// The vertex structure that is sent to the GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    /// Describes the memory layout of the vertex buffer to wgpu.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// A resource to hold the current state of user input.
#[derive(Default)]
pub struct InputState {
    pub pressed_keys: HashSet<KeyCode>,
    /// A flag that is true only for the single frame a jump is initiated.
    pub jump_pressed: bool,
}

/// A resource to hold the current dimensions of the window.
#[derive(Default)]
pub struct ScreenDimensions {
    pub width: f32,
    pub height: f32,
}
