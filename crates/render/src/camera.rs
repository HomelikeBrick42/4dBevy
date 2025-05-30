use bevy::{
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        system::{Query, Res},
        world::Ref,
    },
    reflect::{Reflect, prelude::ReflectDefault},
};
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

pub(super) fn upload_camera(
    state: Res<RenderState>,
    main_camera: Query<(Ref<GlobalTransform>, Ref<Camera>, Ref<MainCamera>)>,
) {
    let (transform, camera, main_camera) = main_camera
        .single()
        .expect("there should only be one MainCamera");

    if transform.is_changed() || camera.is_changed() || main_camera.is_changed() {
        // TODO: upload camera info
        _ = state;
    }
}
