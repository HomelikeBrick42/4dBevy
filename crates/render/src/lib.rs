use bevy::{
    app::{MainScheduleOrder, Plugin, PostUpdate},
    ecs::schedule::ScheduleLabel,
    log::info,
};

#[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Render;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_schedule(Render);
        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_after(PostUpdate, Render);

        app.add_systems(Render, render);
    }
}

fn render() {
    info!("rendering");
}
