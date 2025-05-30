use bevy::{
    ecs::component::Component,
    reflect::{Reflect, prelude::ReflectDefault},
};
use bytemuck::{Pod, Zeroable};

#[derive(Reflect, Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            red: 1.0,
            blue: 1.0,
            green: 1.0,
        }
    }
}

#[derive(Component, Reflect, Default, Debug, Clone, Copy, Zeroable, Pod)]
#[reflect(Default, Clone)]
#[repr(C)]
pub struct Material {
    pub color: Color,
}
