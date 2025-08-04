use rapier2d::prelude::*;
use std::collections::HashSet;
use winit::keyboard::KeyCode;
use crossbeam::channel::{unbounded, Receiver};

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
    // RE-ADDED: This is required for shape-casting.
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
            gravity: nalgebra::vector![0.0, -981.0],
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
            query_pipeline: QueryPipeline::new(), // RE-ADDED
            physics_hooks: (),
            event_handler,
            _collision_event_receiver: collision_receiver,
            _contact_force_event_receiver: contact_force_receiver,
        }
    }
}

#[derive(Default)]
pub struct RenderData(pub Vec<Vertex>);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
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

#[derive(Default)]
pub struct InputState {
    pub pressed_keys: HashSet<KeyCode>,
}

#[derive(Default)]
pub struct ScreenDimensions {
    pub width: f32,
    pub height: f32,
}
