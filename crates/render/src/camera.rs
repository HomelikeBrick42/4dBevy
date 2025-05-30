use bevy::{
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        system::{Query, Res},
        world::Ref,
    },
    reflect::{Reflect, prelude::ReflectDefault},
};
use bytemuck::{Pod, Zeroable};
use transform::{GlobalTransform, Transform};

use crate::RenderState;

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Default, Clone)]
#[require(Transform, GlobalTransform)]
pub struct Camera {
    pub fov: f32,
    pub min_ray_distance: f32,
    pub max_distance: f32,
    pub max_bounces: u32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov: 90.0,
            min_ray_distance: 0.01,
            max_distance: 1000.0,
            max_bounces: 4,
        }
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Default)]
#[require(Camera)]
pub struct MainCamera;

#[derive(Zeroable, Pod, Clone, Copy)]
#[repr(C)]
pub(super) struct GpuCamera {
    transform: Transform,
    fov: f32,
    min_ray_distance: f32,
    max_distance: f32,
    max_bounces: u32,
}

pub(super) fn upload_camera(
    state: Res<RenderState>,
    main_camera: Query<(Ref<GlobalTransform>, Ref<Camera>, Ref<MainCamera>)>,
) {
    let (transform, camera, main_camera) = main_camera
        .single()
        .expect("there should only be one MainCamera");

    if transform.is_changed() || camera.is_changed() || main_camera.is_changed() {
        let transform = transform.0;
        let Camera {
            fov,
            min_ray_distance,
            max_distance,
            max_bounces,
        } = *camera;
        let gpu_camera = GpuCamera {
            transform,
            fov,
            min_ray_distance,
            max_distance,
            max_bounces,
        };

        state
            .queue
            .write_buffer(&state.camera_buffer, 0, bytemuck::bytes_of(&gpu_camera));
    }
}
