use bevy::{
    DefaultPlugins,
    app::{App, AppExit, Startup, Update},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    ecs::{
        event::EventReader,
        system::{Commands, Res},
    },
    input::keyboard::KeyboardInput,
};
use tracing::info;
use transform::TransformPlugin;

const PRINT_FPS: bool = false;

fn main() -> AppExit {
    let mut app = App::new();

    if PRINT_FPS {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    }

    app.add_plugins((DefaultPlugins, TransformPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (print_diagnostics, print_key_presses));

    app.run()
}

fn setup(commands: Commands) {
    _ = commands;
}

fn print_diagnostics(d: Res<DiagnosticsStore>) {
    if let Some(fps) = d.get_measurement(&FrameTimeDiagnosticsPlugin::FPS) {
        info!("FPS: {}", fps.value);
    }
}

fn print_key_presses(mut inputs: EventReader<KeyboardInput>) {
    for input in inputs.read() {
        info!("{:?} {:?}", input.key_code, input.state);
    }
}
