// src/main.rs

use std::iter;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, ElementState},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::KeyCode,
    keyboard::PhysicalKey,
    window::Window,
};
use specs::{World, WorldExt, Builder, Dispatcher, DispatcherBuilder, System, Write};
use rapier2d::prelude::*;
use rapier2d::na::Vector2;
// **FIXED: ADDED THE MISSING IMPORT**
use rapier2d::control::KinematicCharacterController;
use log;

mod components;
mod resources;
mod systems;

use components::*;
use resources::*;
use systems::{physics::PhysicsSystem, player_control::PlayerControlSystem, rendering::RenderingSystem};

// A simple system to reset single-frame input flags
pub struct InputResetSystem;
impl<'a> System<'a> for InputResetSystem {
    type SystemData = Write<'a, InputState>;
    fn run(&mut self, mut input_state: Self::SystemData) {
        input_state.jump_pressed = false;
    }
}


#[derive(Default)]
struct App<'a> {
    state: Option<State<'a>>,
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    
    ecs_world: World,
    dispatcher: Dispatcher<'a, 'a>,

    last_update: Instant,
    accumulator: f32,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (std::mem::size_of::<Vertex>() * 6 * 1024) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // --- Specs ECS Setup ---
        let mut ecs_world = World::new();
        ecs_world.register::<Position>();
        ecs_world.register::<Renderable>();
        ecs_world.register::<PhysicsBody>();
        ecs_world.register::<Player>();
        ecs_world.register::<Grounded>();
        ecs_world.register::<CharacterController>();


        ecs_world.insert(PhysicsWorld::default());
        ecs_world.insert(RenderData::default());
        ecs_world.insert(InputState::default());
        ecs_world.insert(ScreenDimensions { width: size.width as f32, height: size.height as f32 });

        let dispatcher = DispatcherBuilder::new()
            .with(PlayerControlSystem, "player_control", &[])
            .with(PhysicsSystem, "physics_system", &["player_control"])
            .with(RenderingSystem, "rendering_system", &["physics_system"])
            .with_thread_local(InputResetSystem) 
            .build();

        // --- Create Entities ---
        create_level(&mut ecs_world);
        create_player(&mut ecs_world, 0.0, 100.0);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            num_vertices: 0,
            ecs_world,
            dispatcher,
            last_update: Instant::now(),
            accumulator: 0.0,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            let mut screen_dim = self.ecs_world.write_resource::<ScreenDimensions>();
            screen_dim.width = new_size.width as f32;
            screen_dim.height = new_size.height as f32;
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state,
                        ..
                    },
                ..
            } => {
                let mut input_state = self.ecs_world.write_resource::<InputState>();
                match state {
                    ElementState::Pressed => {
                        if input_state.pressed_keys.insert(*keycode) && *keycode == KeyCode::Space {
                            input_state.jump_pressed = true;
                        }
                    }
                    ElementState::Released => {
                        input_state.pressed_keys.remove(keycode);
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        let dt = self.ecs_world.read_resource::<PhysicsWorld>().integration_parameters.dt;
        let now = Instant::now();
        self.accumulator += now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        while self.accumulator >= dt {
            self.dispatcher.dispatch(&self.ecs_world);
            self.ecs_world.maintain();
            self.accumulator -= dt;
        }
        
        let render_data = self.ecs_world.read_resource::<RenderData>();
        self.num_vertices = render_data.0.len() as u32;
        self.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&render_data.0));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }
        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn create_wall(world: &mut World, x: f32, y: f32, width: f32, height: f32) {
    let (rb_handle, col_handle) = {
        let mut pw = world.write_resource::<PhysicsWorld>();
        let pw = &mut *pw;
        let rigid_body_set = &mut pw.rigid_body_set;
        let collider_set = &mut pw.collider_set;

        let rigid_body = RigidBodyBuilder::fixed().translation(vector![x, y]).build();
        let collider = ColliderBuilder::cuboid(width / 2.0, height / 2.0).build();
        let rb_handle = rigid_body_set.insert(rigid_body);
        let col_handle = collider_set.insert_with_parent(collider, rb_handle, rigid_body_set);
        (rb_handle, col_handle)
    };
    
    world.create_entity()
        .with(Position(Vector2::new(x, y)))
        .with(Renderable { color: [0.2, 0.2, 0.2, 1.0], width, height })
        .with(PhysicsBody { rigid_body_handle: rb_handle, collider_handle: col_handle })
        .build();
}

fn create_level(world: &mut World) {
    create_wall(world, 0.0, -250.0, 500.0, 20.0);
    create_wall(world, 200.0, -150.0, 200.0, 20.0);
    create_wall(world, -200.0, 0.0, 200.0, 20.0);
    create_wall(world, -200.0, -150.0, 20.0, 200.0);
}

fn create_player(world: &mut World, x: f32, y: f32) {
    let (rb_handle, col_handle) = {
        let mut pw = world.write_resource::<PhysicsWorld>();
        let pw = &mut *pw;
        let rigid_body_set = &mut pw.rigid_body_set;
        let collider_set = &mut pw.collider_set;

        let rigid_body = RigidBodyBuilder::kinematic_position_based()
            .translation(vector![x, y])
            .build();
        
        let collider = ColliderBuilder::capsule_y(10.0, 10.0)
            .build();
        
        let rb_handle = rigid_body_set.insert(rigid_body);
        let col_handle = collider_set.insert_with_parent(collider, rb_handle, rigid_body_set);
        (rb_handle, col_handle)
    };

    let character_controller = CharacterController {
        controller: KinematicCharacterController::default(),
        velocity: Vector2::zeros(),
    };

    world.create_entity()
        .with(Position(Vector2::new(x, y)))
        .with(Renderable { color: [1.0, 0.5, 0.0, 1.0], width: 20.0, height: 40.0 })
        .with(PhysicsBody { rigid_body_handle: rb_handle, collider_handle: col_handle })
        .with(Player)
        .with(character_controller)
        .build();
}


impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes()
            .with_title("Shadow Chaser");
        let window = Box::leak(Box::new(event_loop.create_window(attributes).unwrap()));
        self.state = Some(pollster::block_on(State::new(window)));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        if let Some(state) = self.state.as_mut() {
            if !state.input(&event) {
                match event {
                    WindowEvent::CloseRequested => event_loop.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                         state.update();
                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logging::log_to_file("debug.log", log::LevelFilter::Info)?;
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
