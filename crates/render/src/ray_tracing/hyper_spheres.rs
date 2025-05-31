use super::Material;
use bevy::{
    ecs::component::Component,
    reflect::{Reflect, prelude::ReflectDefault},
};
use bytemuck::{Pod, Zeroable};
use transform::Transform;

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
    pub position: [f32; 4],
    pub material_id: u32,
    pub radius: f32,
    pub _padding: [u8; 8],
}
