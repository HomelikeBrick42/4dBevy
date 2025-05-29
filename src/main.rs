use bevy::prelude::*;
use transform::TransformPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, TransformPlugin))
        .add_systems(Startup, setup)
        .run()
}

fn setup(commands: Commands) {
    _ = commands;
}
