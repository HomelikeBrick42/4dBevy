use bevy::{
    ecs::{
        component::{Component, HookContext},
        resource::Resource,
        world::DeferredWorld,
    },
    reflect::{Reflect, prelude::ReflectDefault},
};
use bytemuck::{Pod, Zeroable};

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Zeroable, Pod)]
#[reflect(Default, Clone)]
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
            green: 1.0,
            blue: 1.0,
        }
    }
}

#[derive(Component, Reflect, Debug, Default, Clone, Copy, PartialEq)]
#[component(
    immutable,
    on_insert = material_insert,
    on_replace = material_replace,
)]
#[reflect(Default, Clone)]
pub struct Material {
    pub base_color: Color,
}

#[derive(Component)]
pub(crate) struct MaterialId(pub(crate) u32);

#[derive(Resource, Default)]
pub(super) struct MaterialAllocator {
    materials: Vec<Option<(usize, Material)>>,
    free_start: usize,
}

fn material_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let material = *world.get::<Material>(entity).unwrap();
    let MaterialAllocator {
        materials,
        free_start,
    } = &mut *world.resource_mut::<MaterialAllocator>();

    if let Some(id) = materials
        .iter()
        .position(|m| m.as_ref().is_some_and(|(_, m)| *m == material))
    {
        materials[id].as_mut().unwrap().0 += 1;
        world.commands().entity(entity).insert(MaterialId(id as _));
        return;
    }

    if let Some(id) = materials[*free_start..].iter().position(|m| m.is_none()) {
        *free_start = id + 1;
        materials[id] = Some((1, material));
        world.commands().entity(entity).insert(MaterialId(id as _));
        return;
    }

    let id = materials.len();
    *free_start = id + 1;
    materials.push(Some((1, material)));
    world.commands().entity(entity).insert(MaterialId(id as _));
}

fn material_replace(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let id = world.get::<MaterialId>(entity).unwrap().0 as usize;
    let MaterialAllocator {
        materials,
        free_start,
    } = &mut *world.resource_mut::<MaterialAllocator>();

    let material = materials[id].as_mut().unwrap();
    material.0 -= 1;
    if material.0 == 0 {
        materials[id] = None;
        if *free_start > id {
            *free_start = id;
        }
    }
}

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub(super) struct GpuMaterial {
    pub base_color: Color,
}
