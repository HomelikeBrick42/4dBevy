use crate::{PreRender, Render, RenderState, Rendering};
use bevy::{
    app::{App, Plugin},
    ecs::{
        change_detection::DetectChanges,
        entity::Entity,
        query::{Added, Changed},
        removal_detection::RemovedComponents,
        resource::Resource,
        system::{Query, Res, ResMut},
        world::Ref,
    },
};
use bytemuck::{Pod, Zeroable};
use std::{mem::offset_of, num::NonZero};
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

    main_texture: wgpu::Texture,
    main_texture_bind_group_layout: wgpu::BindGroupLayout,
    main_texture_bind_group: wgpu::BindGroup,
    rendering_main_texture_bind_group_layout: wgpu::BindGroupLayout,
    rendering_main_texture_bind_group: wgpu::BindGroup,

    ray_tracing_pipeline: wgpu::ComputePipeline,
    full_screen_quad_pipeline: wgpu::RenderPipeline,
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
            .add_systems(
                PreRender,
                (camera_upload, material_upload, hyper_spheres_upload),
            )
            .add_systems(Render, ray_trace);
    }

    fn finish(&self, app: &mut App) {
        let state = app.world().resource::<RenderState>();

        let ray_tracing_shader = state
            .device
            .create_shader_module(wgpu::include_wgsl!(concat!(
                env!("OUT_DIR"),
                "/shaders/ray_tracing.wgsl",
            )));
        let full_screen_quad_shader =
            state
                .device
                .create_shader_module(wgpu::include_wgsl!(concat!(
                    env!("OUT_DIR"),
                    "/shaders/full_screen_quad.wgsl",
                )));

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
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: NonZero::new(size_of::<GpuCamera>() as _),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
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

        let materials_buffer =
            create_materials_buffer(&state.device, size_of::<GpuMaterial>() as _);
        let hyper_spheres_buffer =
            create_hyper_spheres_buffer(&state.device, size_of::<GpuHyperSphere>() as _);
        let objects_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Objects Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: NonZero::new(size_of::<GpuMaterial>() as _),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
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

        let main_texture = create_main_texture(&state.device, 1, 1);

        let main_texture_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Main Texture Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba32Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });
        let main_texture_bind_group = create_main_texture_bind_group(
            &state.device,
            &main_texture_bind_group_layout,
            &main_texture,
        );

        let rendering_main_texture_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Rendering Main Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                    ],
                });
        let rendering_main_texture_bind_group = create_rendering_main_texture_bind_group(
            &state.device,
            &rendering_main_texture_bind_group_layout,
            &main_texture,
        );

        let ray_tracing_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Ray Tracing Pipeline Layout"),
                    bind_group_layouts: &[
                        &main_texture_bind_group_layout,
                        &info_bind_group_layout,
                        &objects_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
        let ray_tracing_pipeline =
            state
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Ray Tracing Pipeline"),
                    layout: Some(&ray_tracing_pipeline_layout),
                    module: &ray_tracing_shader,
                    entry_point: Some("ray_trace"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                });

        let full_screen_quad_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Full Screen Quad Pipeline Layout"),
                    bind_group_layouts: &[&rendering_main_texture_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let full_screen_quad_pipeline =
            state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Full Screen Quad Pipeline"),
                    layout: Some(&full_screen_quad_pipeline_layout),
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

            main_texture,
            main_texture_bind_group_layout,
            main_texture_bind_group,
            rendering_main_texture_bind_group_layout,
            rendering_main_texture_bind_group,

            ray_tracing_pipeline,
            full_screen_quad_pipeline,
        });
    }
}

fn create_materials_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Materials Buffer"),
        size,
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    })
}

fn create_hyper_spheres_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Hyper Spheres Buffer"),
        size,
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    })
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

fn create_main_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::wgt::TextureDescriptor {
        label: Some("Main Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn create_main_texture_bind_group(
    device: &wgpu::Device,
    main_texture_bind_group_layout: &wgpu::BindGroupLayout,
    main_texture: &wgpu::Texture,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Main Texture Bind Group"),
        layout: main_texture_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(
                &main_texture.create_view(&Default::default()),
            ),
        }],
    })
}

fn create_rendering_main_texture_bind_group(
    device: &wgpu::Device,
    rendering_main_texture_bind_group_layout: &wgpu::BindGroupLayout,
    main_texture: &wgpu::Texture,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Rendering Main Texture Bind Group"),
        layout: rendering_main_texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &main_texture.create_view(&Default::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&device.create_sampler(
                    &wgpu::SamplerDescriptor {
                        label: Some("Rendering Main Texture Sampler"),
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Nearest,
                        min_filter: wgpu::FilterMode::Nearest,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        ..Default::default()
                    },
                )),
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
    materials: Query<(&Material, &MaterialId), Changed<MaterialId>>,
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
        let new_buffer = create_materials_buffer(&state.device, required_space);

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

fn hyper_spheres_upload(
    state: Res<RenderState>,
    mut ray_tracing: ResMut<RayTracing>,
    hyper_spheres_: Query<(Ref<GlobalTransform>, Ref<MaterialId>, Ref<HyperSphere>)>,
    added_hyper_spheres: Query<(), Added<HyperSphere>>,
    mut removed_hyper_spheres: RemovedComponents<HyperSphere>,
) {
    let was_removed = !removed_hyper_spheres.is_empty();
    if was_removed {
        removed_hyper_spheres.clear();
    }

    let was_added = !added_hyper_spheres.is_empty();

    let hyper_spheres = hyper_spheres_.iter().sort::<Entity>();
    let mut hyper_sphere_count = 0;
    if was_added {
        hyper_sphere_count = hyper_spheres_.iter().count() as _;

        let required_space = hyper_sphere_count as wgpu::BufferAddress
            * size_of::<GpuHyperSphere>() as wgpu::BufferAddress;
        let old_size = ray_tracing.hyper_spheres_buffer.size();
        if required_space > old_size {
            ray_tracing.hyper_spheres_buffer =
                create_hyper_spheres_buffer(&state.device, required_space);
            ray_tracing.objects_bind_group = create_objects_bind_group(
                &state.device,
                &ray_tracing.objects_bind_group_layout,
                &ray_tracing.materials_buffer,
                &ray_tracing.hyper_spheres_buffer,
            );
        }

        let mut buffer = state
            .queue
            .write_buffer_with(
                &ray_tracing.hyper_spheres_buffer,
                0,
                NonZero::new(required_space).unwrap(),
            )
            .unwrap();
        for (index, (transform, material, hyper_sphere)) in hyper_spheres.enumerate() {
            let offset = index * size_of::<GpuHyperSphere>();
            let position = transform.0.transform((0.0, 0.0, 0.0, 0.0)).into();
            let material_id = material.0;
            let HyperSphere { radius } = *hyper_sphere;
            let gpu_hyper_sphere = GpuHyperSphere {
                position,
                material_id,
                radius,
                _padding: Default::default(),
            };
            buffer[offset..][..size_of::<GpuHyperSphere>()]
                .copy_from_slice(bytemuck::bytes_of(&gpu_hyper_sphere));
        }
    } else {
        for (index, (transform, material, hyper_sphere)) in hyper_spheres.enumerate() {
            if was_removed
                || transform.is_changed()
                || material.is_changed()
                || hyper_sphere.is_changed()
            {
                let offset = index as wgpu::BufferAddress
                    * size_of::<GpuHyperSphere>() as wgpu::BufferAddress;
                let position = transform.0.transform((0.0, 0.0, 0.0, 0.0)).into();
                let material_id = material.0;
                let HyperSphere { radius } = *hyper_sphere;
                let gpu_hyper_sphere = GpuHyperSphere {
                    position,
                    material_id,
                    radius,
                    _padding: Default::default(),
                };
                state.queue.write_buffer(
                    &ray_tracing.hyper_spheres_buffer,
                    offset,
                    bytemuck::bytes_of(&gpu_hyper_sphere),
                );
            }
            hyper_sphere_count += 1;
        }
    }

    if was_added || was_removed {
        state.queue.write_buffer(
            &ray_tracing.objects_info_buffer,
            offset_of!(GpuObjectsInfo, hyper_spheres_count) as _,
            &u32::to_ne_bytes(hyper_sphere_count),
        );
    }
}

fn ray_trace(
    state: Res<RenderState>,
    rendering: Res<Rendering>,
    mut ray_tracing: ResMut<RayTracing>,
) {
    if let Some(surface_texture) = &rendering.surface_texture {
        {
            let surface_size = surface_texture.texture.size();
            let main_texture_size = ray_tracing.main_texture.size();
            if surface_size != main_texture_size {
                ray_tracing.main_texture =
                    create_main_texture(&state.device, surface_size.width, surface_size.height);
                ray_tracing.main_texture_bind_group = create_main_texture_bind_group(
                    &state.device,
                    &ray_tracing.main_texture_bind_group_layout,
                    &ray_tracing.main_texture,
                );
                ray_tracing.rendering_main_texture_bind_group =
                    create_rendering_main_texture_bind_group(
                        &state.device,
                        &ray_tracing.rendering_main_texture_bind_group_layout,
                        &ray_tracing.main_texture,
                    );
            }
        }

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Ray Tracing Command Encoder"),
            });
        {
            let mut ray_tracing_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Ray Tracing Pass"),
                timestamp_writes: None,
            });

            let main_texture_size = ray_tracing.main_texture.size();

            ray_tracing_pass.set_pipeline(&ray_tracing.ray_tracing_pipeline);
            ray_tracing_pass.set_bind_group(0, &ray_tracing.main_texture_bind_group, &[]);
            ray_tracing_pass.set_bind_group(1, &ray_tracing.info_bind_group, &[]);
            ray_tracing_pass.set_bind_group(2, &ray_tracing.objects_bind_group, &[]);
            ray_tracing_pass.dispatch_workgroups(
                main_texture_size.width.div_ceil(16),
                main_texture_size.height.div_ceil(16),
                1,
            );
        }
        {
            let mut rendering_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Rendering Pass"),
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

            rendering_pass.set_pipeline(&ray_tracing.full_screen_quad_pipeline);
            rendering_pass.set_bind_group(0, &ray_tracing.rendering_main_texture_bind_group, &[]);
            rendering_pass.draw(0..4, 0..1);
        }
        state.queue.submit(std::iter::once(encoder.finish()));
    }
}
