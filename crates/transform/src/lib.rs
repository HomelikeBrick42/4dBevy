use bevy::prelude::*;

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
            ),
        )
        .add_systems(
            PostUpdate,
            (
                flag_orphans.before(compute_global_transform_parents),
                compute_global_transform_children,
            ),
        );
    }
}

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Default, Clone)]
#[require(GlobalTransform)]
pub struct Transform;

impl Transform {
    #[must_use]
    pub fn then(self, #[expect(unused)] other: Self) -> Self {
        self
    }
}

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Default, Clone)]
pub struct GlobalTransform(pub Transform);

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
