use bevy::prelude::*;

mod rotor;
mod transform;

pub use rotor::Rotor;
pub use transform::Transform;

pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Transform>()
            .register_type::<GlobalTransform>();

        app.add_systems(
            PostStartup,
            (
                flag_orphans.before(compute_global_transform_parents),
                compute_global_transform_children,
            )
                .after(normalise_transforms),
        )
        .add_systems(
            PostUpdate,
            (
                flag_orphans.before(compute_global_transform_parents),
                compute_global_transform_children,
            )
                .after(normalise_transforms),
        );
    }
}

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Default, Clone)]
#[require(Transform)]
pub struct GlobalTransform(pub Transform);

fn normalise_transforms(mut transforms: Query<&mut Transform, Changed<Transform>>) {
    transforms
        .par_iter_mut()
        .for_each(|mut transform| *transform = transform.normalised());
}

fn flag_orphans(
    mut global_transforms: Query<&mut GlobalTransform>,
    mut orphaned: RemovedComponents<ChildOf>,
) {
    // set the changed flag on all orphaned entities so their transforms will be updated
    let mut orphaned = global_transforms.iter_many_mut(orphaned.read());
    while let Some(mut global_transform) = orphaned.fetch_next() {
        global_transform.set_changed();
    }
}

fn compute_global_transform_parents(
    mut global_transforms: Query<
        (&Transform, &mut GlobalTransform),
        (
            Or<(Changed<Transform>, Changed<GlobalTransform>)>,
            Without<ChildOf>,
        ),
    >,
) {
    global_transforms
        .par_iter_mut()
        .for_each(|(transform, mut global_transform)| {
            *global_transform = GlobalTransform(*transform);
        })
}

// TODO: improve this to not run at all on entities which havent changed
fn compute_global_transform_children(
    mut global_transforms: Query<(Entity, &mut GlobalTransform), With<ChildOf>>,
    transforms: Query<(Option<Ref<Transform>>, Option<Ref<ChildOf>>)>,
) {
    global_transforms
        .par_iter_mut()
        .for_each(|(entity, mut global_transform)| {
            let mut computed_transform = Transform::default();

            let mut changed = global_transform.is_changed();
            let mut next = entity;
            loop {
                let Ok((transform, child_of)) = transforms.get(next) else {
                    unreachable!()
                };

                if let Some(transform) = transform {
                    changed |= transform.is_changed();
                    computed_transform = transform.then(computed_transform);
                }

                let Some(child_of) = child_of else {
                    break;
                };
                changed |= child_of.is_changed();
                next = child_of.0;
            }

            if changed {
                *global_transform = GlobalTransform(computed_transform);
            }
        })
}
