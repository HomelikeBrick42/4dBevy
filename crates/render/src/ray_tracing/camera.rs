use bevy::{
    ecs::component::Component,
    reflect::{Reflect, prelude::ReflectDefault},
};
use bytemuck::{Pod, Zeroable};
use transform::Transform;

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Default, Clone)]
#[require(Transform)]
pub struct Camera {
    pub min_distance: f32,
    pub max_distance: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            min_distance: 0.01,
            max_distance: 1000.0,
        }
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Default)]
#[require(Camera)]
pub struct MainCamera;

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub(super) struct GpuCamera {
    pub position: [f32; 4],
    pub forward: [f32; 4],
    pub right: [f32; 4],
    pub up: [f32; 4],
    pub aspect: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub _padding: [u8; 4],
}
