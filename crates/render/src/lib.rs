use bevy::{
    app::{App, MainScheduleOrder, Plugin, PostUpdate},
    ecs::{
        entity::Entity, event::EventReader, query::With, resource::Resource,
        schedule::ScheduleLabel, system::ResMut,
    },
    log::info,
    window::{PrimaryWindow, RawHandleWrapperHolder, WindowResized},
};
use bytemuck::{Pod, Zeroable};
use camera::GpuCamera;
use hyper_sphere::GpuHyperSphere;
use std::{cell::Cell, num::NonZeroU64, rc::Rc};
use wgpu::util::DeviceExt;

mod camera;
mod hyper_sphere;
mod material;

pub use camera::{Camera, MainCamera};
pub use hyper_sphere::HyperSphere;
pub use material::{Color, Material};

#[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreRender;

#[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Render;

struct RenderInitState {
    state: Rc<Cell<Option<RenderState>>>,
}

#[derive(Resource)]
pub struct RenderState {
    pub instance: wgpu::Instance,
    pub primary_window_entity: Entity,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    hyper_spheres_count: u32,
    hyper_spheres_buffer: wgpu::Buffer,

    objects_info_buffer: wgpu::Buffer,
    objects_bind_group_layout: wgpu::BindGroupLayout,
    objects_bind_group: wgpu::BindGroup,

    full_screen_quad_render_pipeline: wgpu::RenderPipeline,
}

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct GpuObjectsInfo {
    hyper_spheres_count: u32,
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let render_init_state = RenderInitState {
            state: Rc::new(Cell::new(None)),
        };
        let render_init_future = {
            let state = render_init_state.state.clone();

            let (primary_window_entity, primary_window) = app
                .world_mut()
                .query_filtered::<(Entity, &RawHandleWrapperHolder), With<PrimaryWindow>>()
                .single(app.world())
                .expect("there should be a primary window");
            let primary_window = primary_window.clone();

            async move {
                let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::all(),
                    flags: wgpu::InstanceFlags::from_env_or_default(),
                    backend_options: wgpu::BackendOptions::from_env_or_default(),
                });

                let surface = {
                    // this is horrible but just do an async spin loop until the primary window is available, this seems to work
                    let handle = loop {
                        let handle = primary_window.0.lock().unwrap();
                        if handle.is_some() {
                            break handle;
                        }
                        drop(handle);
                        bevy::tasks::futures_lite::future::yield_now().await;
                    };
                    let handle = handle.as_ref().unwrap();

                    // Safety: this async task is spawned on the main thread with spawn_local, and bevy should only be initialised on the main thread
                    let handle = unsafe { handle.get_handle() };

                    instance
                        .create_surface(handle)
                        .expect("wgpu surface should be created")
                };

                info!("Created the surface");

                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::HighPerformance,
                        force_fallback_adapter: false,
                        compatible_surface: Some(&surface),
                    })
                    .await
                    .expect("wgpu adapter should be found");

                info!("Using adapter {:?}", adapter.get_info());

                let (device, queue) = adapter
                    .request_device(&wgpu::DeviceDescriptor {
                        label: Some("Device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                        memory_hints: wgpu::MemoryHints::Performance,
                        trace: wgpu::Trace::Off,
                    })
                    .await
                    .expect("wgpu device and queue should be created");

                info!("Created device and queue");

                let surface_config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    width: 1,
                    height: 1,
                    present_mode: wgpu::PresentMode::AutoVsync,
                    desired_maximum_frame_latency: 2,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                };
                surface.configure(&device, &surface_config);

                let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Camera Buffer"),
                    size: size_of::<GpuCamera>() as _,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                    mapped_at_creation: false,
                });
                let camera_bind_group_layout =
                    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("Camera Bind Group Layout"),
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: NonZeroU64::new(size_of::<GpuCamera>() as _),
                            },
                            count: None,
                        }],
                    });
                let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Camera Bind Group"),
                    layout: &camera_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }],
                });

                let hyper_spheres_count = 0;
                let hyper_spheres_min_size = size_of::<GpuHyperSphere>() as wgpu::BufferAddress;
                let hyper_spheres_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("HyperSpheres Buffer"),
                    size: hyper_spheres_min_size,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                });

                let objects_info_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Objects Info Buffer"),
                        contents: bytemuck::bytes_of(&GpuObjectsInfo {
                            hyper_spheres_count,
                        }),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                    });
                let objects_bind_group_layout =
                    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("HyperSpheres Bind Group Layout"),
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: NonZeroU64::new(
                                        size_of::<GpuObjectsInfo>() as _
                                    ),
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                                    has_dynamic_offset: false,
                                    min_binding_size: NonZeroU64::new(hyper_spheres_min_size),
                                },
                                count: None,
                            },
                        ],
                    });
                let objects_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("HyperSpheres Bind Group"),
                    layout: &objects_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: objects_info_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: hyper_spheres_buffer.as_entire_binding(),
                        },
                    ],
                });

                let full_screen_quad_shader = device.create_shader_module(wgpu::include_wgsl!(
                    concat!(env!("OUT_DIR"), "/shaders/full_screen_quad.wgsl",)
                ));

                let full_screen_quad_render_pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Full Screen Quad Render Pipeline Layout"),
                        bind_group_layouts: &[
                            &camera_bind_group_layout,
                            &objects_bind_group_layout,
                        ],
                        push_constant_ranges: &[],
                    });
                let full_screen_quad_render_pipeline =
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Full Screen Quad Render Pipeline"),
                        layout: Some(&full_screen_quad_render_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &full_screen_quad_shader,
                            entry_point: Some("vertex"),
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                            buffers: &[],
                        },
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleStrip,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Cw,
                            cull_mode: None,
                            unclipped_depth: false,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false,
                        },
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &full_screen_quad_shader,
                            entry_point: Some("fragment"),
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: surface_config.format,
                                blend: None,
                                write_mask: wgpu::ColorWrites::all(),
                            })],
                        }),
                        multiview: None,
                        cache: None,
                    });

                info!("Initialized renderer");

                state.set(Some(RenderState {
                    primary_window_entity,
                    instance,
                    surface,
                    surface_config,
                    device,
                    queue,

                    camera_buffer,
                    camera_bind_group,

                    hyper_spheres_count,
                    hyper_spheres_buffer,

                    objects_info_buffer,
                    objects_bind_group_layout,
                    objects_bind_group,

                    full_screen_quad_render_pipeline,
                }));
            }
        };
        bevy::tasks::IoTaskPool::get()
            .spawn_local(render_init_future)
            .detach();

        app.insert_non_send_resource(render_init_state);

        app.init_schedule(PreRender).init_schedule(Render);
        let mut main_schedule = app.world_mut().resource_mut::<MainScheduleOrder>();
        main_schedule.insert_after(PostUpdate, PreRender);
        main_schedule.insert_after(PreRender, Render);

        app.register_type::<Camera>()
            .register_type::<MainCamera>()
            .register_type::<Material>()
            .register_type::<HyperSphere>()
            .add_systems(
                PreRender,
                (camera::upload_camera, hyper_sphere::upload_hyper_spheres),
            )
            .add_systems(Render, render);
    }

    fn ready(&self, app: &App) -> bool {
        let init_resource = app.world().non_send_resource::<RenderInitState>();
        let init_state = init_resource.state.take();
        let finished = init_state.is_some();
        init_resource.state.set(init_state);
        finished
    }

    fn finish(&self, app: &mut App) {
        let render_state = app
            .world_mut()
            .remove_non_send_resource::<RenderInitState>()
            .unwrap()
            .state
            .take()
            .expect("if RenderPlugin::ready returned true then RenderState has been created");
        app.insert_resource(render_state);
    }
}

fn render(
    mut state: ResMut<RenderState>,
    mut resize_events: EventReader<WindowResized>,
) -> bevy::ecs::error::Result {
    let RenderState {
        instance: _,
        primary_window_entity,
        ref surface,
        ref mut surface_config,
        ref device,
        ref queue,

        camera_buffer: _,
        ref camera_bind_group,

        hyper_spheres_count: _,
        hyper_spheres_buffer: _,

        objects_info_buffer: _,
        objects_bind_group_layout: _,
        ref objects_bind_group,

        ref full_screen_quad_render_pipeline,
    } = *state;

    if let Some(resize_event) = resize_events
        .read()
        .filter(|event| event.window == primary_window_entity)
        .last()
    {
        surface_config.width = resize_event.width as _;
        surface_config.height = resize_event.height as _;
        surface.configure(device, surface_config);

        info!(
            "resized surface to {}, {}",
            surface_config.width, surface_config.height
        );
    }

    let texture = match surface.get_current_texture() {
        Ok(texture) => texture,
        Err(wgpu::SurfaceError::Outdated) => return Ok(()),
        Err(wgpu::SurfaceError::Timeout) => return Ok(()),
        r => r?,
    };

    let mut encoder = device.create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture.texture.create_view(&Default::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(full_screen_quad_render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, objects_bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    texture.present();

    Ok(())
}
