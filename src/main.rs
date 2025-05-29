use bevy::{
    a11y::AccessibilityPlugin,
    app::{
        App, AppExit, PanicHandlerPlugin, Startup, TaskPoolPlugin, TerminalCtrlCHandlerPlugin,
        Update,
    },
    asset::AssetPlugin,
    diagnostic::{
        DiagnosticsPlugin, DiagnosticsStore, FrameCountPlugin, FrameTimeDiagnosticsPlugin,
    },
    ecs::{
        event::EventReader,
        system::{Commands, Res},
    },
    input::{InputPlugin, keyboard::KeyboardInput},
    log::LogPlugin,
    time::TimePlugin,
    window::WindowPlugin,
    winit::WinitPlugin,
};
use tracing::info;
use transform::TransformPlugin;

const PRINT_FPS: bool = false;

fn main() -> AppExit {
    let mut app = App::new();

    app.add_plugins((
        PanicHandlerPlugin,
        LogPlugin::default(),
        TaskPoolPlugin::default(),
        FrameCountPlugin,
        TimePlugin,
        DiagnosticsPlugin,
        InputPlugin,
        WindowPlugin::default(),
        AccessibilityPlugin,
        TerminalCtrlCHandlerPlugin,
        AssetPlugin::default(),
        <WinitPlugin>::default(),
        TransformPlugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, print_key_presses);

    if PRINT_FPS {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Update, print_diagnostics);
    }

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
