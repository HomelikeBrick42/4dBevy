use crate::{PreRender, Render, RenderState, Rendering};
use bevy::{
    app::{App, Plugin},
    ecs::{
        change_detection::DetectChanges,
        query::{Changed, Or},
        resource::Resource,
        system::{Query, Res, ResMut},
        world::Ref,
    },
};
use bytemuck::{Pod, Zeroable};
use std::num::NonZero;
use transform::GlobalTransform;
use wgpu::util::DeviceExt;

mod camera;
mod hyper_spheres;
mod materials;

pub use camera::*;
pub use hyper_spheres::*;
pub use materials::*;

#[derive(Resource)]
struct RayTracing {
    camera_buffer: wgpu::Buffer,
    objects_info_buffer: wgpu::Buffer,
    info_bind_group: wgpu::BindGroup,

    materials_buffer: wgpu::Buffer,
    hyper_spheres_buffer: wgpu::Buffer,
    objects_bind_group_layout: wgpu::BindGroupLayout,
    objects_bind_group: wgpu::BindGroup,

    ray_tracing_pipeline: wgpu::RenderPipeline,
}

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct GpuObjectsInfo {
    hyper_spheres_count: u32,
}

pub(super) struct RayTracingPlugin;

impl Plugin for RayTracingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MaterialAllocator>()
            .add_systems(PreRender, (camera_upload, material_upload))
            .add_systems(Render, ray_trace);
    }

    fn finish(&self, app: &mut App) {
        let state = app.world().resource::<RenderState>();

        let camera_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: size_of::<GpuCamera>() as _,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let objects_info_buffer =
            state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Objects Info Buffer"),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                    contents: bytemuck::bytes_of(&GpuObjectsInfo {
                        hyper_spheres_count: 0,
                    }),
                });

        let info_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Info Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: NonZero::new(size_of::<GpuCamera>() as _),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: NonZero::new(size_of::<GpuObjectsInfo>() as _),
                            },
                            count: None,
                        },
                    ],
                });
        let info_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Info Bind Group"),
            layout: &info_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: objects_info_buffer.as_entire_binding(),
                },
            ],
        });

        let materials_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Materials Buffer"),
            size: size_of::<GpuMaterial>() as _,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let hyper_spheres_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hyper Spheres Buffer"),
            size: size_of::<GpuHyperSphere>() as _,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let objects_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Objects Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: NonZero::new(size_of::<GpuMaterial>() as _),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: NonZero::new(size_of::<GpuHyperSphere>() as _),
                            },
                            count: None,
                        },
                    ],
                });
        let objects_bind_group = create_objects_bind_group(
            &state.device,
            &objects_bind_group_layout,
            &materials_buffer,
            &hyper_spheres_buffer,
        );

        let ray_tracing_shader = state
            .device
            .create_shader_module(wgpu::include_wgsl!(concat!(
                env!("OUT_DIR"),
                "/shaders/ray_tracing.wgsl",
            )));
        let ray_tracing_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Ray Tracing Pipeline Layout"),
                    bind_group_layouts: &[&info_bind_group_layout, &objects_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let ray_tracing_pipeline =
            state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Ray Tracing Pipeline"),
                    layout: Some(&ray_tracing_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &ray_tracing_shader,
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
                        module: &ray_tracing_shader,
                        entry_point: Some("fragment"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: state.surface_config.format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    }),
                    multiview: None,
                    cache: None,
                });

        app.insert_resource(RayTracing {
            camera_buffer,
            objects_info_buffer,
            info_bind_group,

            materials_buffer,
            hyper_spheres_buffer,
            objects_bind_group_layout,
            objects_bind_group,

            ray_tracing_pipeline,
        });
    }
}

fn create_objects_bind_group(
    device: &wgpu::Device,
    objects_bind_group_layout: &wgpu::BindGroupLayout,
    materials_buffer: &wgpu::Buffer,
    hyper_spheres_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Objects Bind Group"),
        layout: &objects_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: materials_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: hyper_spheres_buffer.as_entire_binding(),
            },
        ],
    })
}

fn camera_upload(
    state: Res<RenderState>,
    ray_tracing: Res<RayTracing>,
    camera: Query<(Ref<GlobalTransform>, Ref<Camera>, Ref<MainCamera>)>,
) {
    let (transform, camera, main_camera) =
        camera.single().expect("there should be one main camera");

    if state.is_changed()
        || transform.is_changed()
        || camera.is_changed()
        || main_camera.is_changed()
    {
        let transform = transform.0;
        let rotation = transform.rotor_part();

        let position = transform.transform((0.0, 0.0, 0.0, 0.0)).into();
        let forward = rotation.rotate((1.0, 0.0, 0.0, 0.0)).into();
        let right = rotation.rotate((0.0, 0.0, 1.0, 0.0)).into();
        let up = rotation.rotate((0.0, 1.0, 0.0, 0.0)).into();

        let aspect = state.surface_config.width as f32 / state.surface_config.height as f32;

        let Camera {
            min_distance,
            max_distance,
        } = *camera;

        let gpu_camera = GpuCamera {
            position,
            forward,
            right,
            up,
            aspect,
            min_distance,
            max_distance,
            _padding: Default::default(),
        };

        state.queue.write_buffer(
            &ray_tracing.camera_buffer,
            0,
            bytemuck::bytes_of(&gpu_camera),
        );
    }
}

fn material_upload(
    state: Res<RenderState>,
    mut ray_tracing: ResMut<RayTracing>,
    materials: Query<(&Material, &MaterialId), Or<(Changed<Material>, Changed<MaterialId>)>>,
) {
    let max_id = materials
        .iter()
        .map(|(_, &MaterialId(id))| id)
        .max()
        .unwrap_or(0);

    let required_space =
        (max_id as wgpu::BufferAddress + 1) * size_of::<GpuMaterial>() as wgpu::BufferAddress;
    let old_size = ray_tracing.materials_buffer.size();
    if required_space > old_size {
        let new_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Materials Buffer"),
            size: required_space,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Materials Buffer Realloc"),
            });
        encoder.copy_buffer_to_buffer(&ray_tracing.materials_buffer, 0, &new_buffer, 0, old_size);
        state.queue.submit(std::iter::once(encoder.finish()));

        ray_tracing.materials_buffer = new_buffer;
        ray_tracing.objects_bind_group = create_objects_bind_group(
            &state.device,
            &ray_tracing.objects_bind_group_layout,
            &ray_tracing.materials_buffer,
            &ray_tracing.hyper_spheres_buffer,
        );
    }

    for (material, &MaterialId(id)) in materials {
        let offset = id as wgpu::BufferAddress * size_of::<GpuMaterial>() as wgpu::BufferAddress;
        let Material { base_color } = *material;
        let gpu_material = GpuMaterial { base_color };
        state.queue.write_buffer(
            &ray_tracing.materials_buffer,
            offset,
            bytemuck::bytes_of(&gpu_material),
        );
    }
}

fn ray_trace(state: Res<RenderState>, rendering: Res<Rendering>, ray_tracing: Res<RayTracing>) {
    if let Some(surface_texture) = &rendering.surface_texture {
        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Ray Tracing Command Encoder"),
            });
        {
            let mut ray_tracing_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Ray Tracing Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_texture.texture.create_view(&Default::default()),
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

            ray_tracing_pass.set_pipeline(&ray_tracing.ray_tracing_pipeline);
            ray_tracing_pass.set_bind_group(0, &ray_tracing.info_bind_group, &[]);
            ray_tracing_pass.set_bind_group(1, &ray_tracing.objects_bind_group, &[]);
            ray_tracing_pass.draw(0..4, 0..1);
        }
        state.queue.submit(std::iter::once(encoder.finish()));
    }
}
