use std::{mem::offset_of, num::NonZeroU64};

use crate::{GpuObjectsInfo, Material, RenderState};
use bevy::{
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        system::{Query, ResMut},
        world::Ref,
    },
    reflect::{Reflect, prelude::ReflectDefault},
};
use bytemuck::{Pod, Zeroable};
use transform::{GlobalTransform, Transform};

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Default, Clone)]
#[require(Transform, Material)]
pub struct HyperSphere {
    pub radius: f32,
}

impl Default for HyperSphere {
    fn default() -> Self {
        Self { radius: 1.0 }
    }
}

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub(super) struct GpuHyperSphere {
    transform: Transform,
    material: Material,
    radius: f32,
}

pub(super) fn upload_hyper_spheres(
    mut state: ResMut<RenderState>,
    hyper_spheres: Query<(Ref<GlobalTransform>, Ref<HyperSphere>, Ref<Material>)>,
) {
    let mut any_changed = false;
    let mut count = 0u32;
    for (transform, hyper_sphere, material) in &hyper_spheres {
        any_changed = any_changed
            || transform.is_changed()
            || hyper_sphere.is_changed()
            || material.is_changed();
        count += 1;
    }
    any_changed |= count > state.hyper_spheres_count;

    let size_for_upload = (count as usize * size_of::<GpuHyperSphere>()) as wgpu::BufferAddress;
    if size_for_upload > state.hyper_spheres_buffer.size() {
        assert!(any_changed);
        state.hyper_spheres_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("HyperSpheres Buffer"),
            size: size_for_upload,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        state.objects_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("HyperSpheres Bind Group"),
            layout: &state.objects_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: state.objects_info_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: state.hyper_spheres_buffer.as_entire_binding(),
                },
            ],
        });
    }

    if count != state.hyper_spheres_count {
        state.hyper_spheres_count = count;
        state.queue.write_buffer(
            &state.objects_info_buffer,
            offset_of!(GpuObjectsInfo, hyper_spheres_count) as _,
            &state.hyper_spheres_count.to_ne_bytes(),
        );
    }

    // TODO: find some way to avoid re-uploading *all* if only some have changed
    if any_changed {
        let mut buffer = state
            .queue
            .write_buffer_with(
                &state.hyper_spheres_buffer,
                0,
                NonZeroU64::new(size_for_upload).unwrap(),
            )
            .unwrap();

        for (index, (transform, hyper_sphere, material)) in hyper_spheres.iter().enumerate() {
            let transform = transform.0;
            let HyperSphere { radius } = *hyper_sphere;
            let material = *material;
            let gpu_hyper_sphere = GpuHyperSphere {
                transform,
                material,
                radius,
            };

            let offset = index * size_of::<GpuHyperSphere>();
            buffer[offset..][..size_of::<GpuHyperSphere>()]
                .copy_from_slice(bytemuck::bytes_of(&gpu_hyper_sphere));
        }
    }
}
