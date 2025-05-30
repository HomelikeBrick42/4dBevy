use bevy::{
    a11y::AccessibilityPlugin,
    app::{App, AppExit, PanicHandlerPlugin, Startup, TaskPoolPlugin, Update},
    asset::AssetPlugin,
    diagnostic::{
        DiagnosticsPlugin, DiagnosticsStore, FrameCountPlugin, FrameTimeDiagnosticsPlugin,
    },
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        system::{Commands, Query, Res},
    },
    input::{InputPlugin, keyboard::KeyboardInput},
    log::{LogPlugin, info},
    time::{Time, TimePlugin},
    window::WindowPlugin,
    winit::WinitPlugin,
};
use render::{Camera, MainCamera, RenderPlugin};
use transform::{Transform, TransformPlugin};

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
        AssetPlugin::default(),
        <WinitPlugin>::default(),
        TransformPlugin,
        RenderPlugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, (print_key_presses, rotate_y));

    if PRINT_FPS {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Update, print_diagnostics);
    }

    app.run()
}

#[derive(Component)]
struct RotateXY;

fn setup(mut commands: Commands) {
    commands.spawn((
        Transform::translation(-3.0, 0.0, 0.0, 0.0),
        Camera::default(),
        MainCamera,
        RotateXY,
    ));
}

fn rotate_y(time: Res<Time>, mut transforms: Query<&mut Transform, With<RotateXY>>) {
    transforms.par_iter_mut().for_each(|mut transform| {
        let transform = &mut *transform;
        *transform = transform.then(Transform::rotation_xy(
            time.delta_secs() * core::f32::consts::TAU,
        ));
    })
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
