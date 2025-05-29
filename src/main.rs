use bevy::{input::keyboard::KeyboardInput, prelude::*};
use transform::TransformPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, TransformPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, print_key_presses)
        .run()
}

fn setup(commands: Commands) {
    _ = commands;
}

fn print_key_presses(mut inputs: EventReader<KeyboardInput>) {
    for input in inputs.read() {
        info!("{:?} {:?}", input.key_code, input.state);
    }
}
