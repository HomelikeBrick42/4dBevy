use bevy::{
    a11y::AccessibilityPlugin,
    app::{App, AppExit, PanicHandlerPlugin, Startup, TaskPoolPlugin, Update},
    asset::AssetPlugin,
    diagnostic::{
        DiagnosticsPlugin, DiagnosticsStore, FrameCountPlugin, FrameTimeDiagnosticsPlugin,
    },
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res},
    },
    input::InputPlugin,
    log::{LogPlugin, info},
    time::{Time, TimePlugin},
    window::WindowPlugin,
    winit::WinitPlugin,
};
use chunks::ChunksPlugin;
use movement_control::{MovementControl, handle_cursor_locking, movement_controls};
use render::{
    RenderPlugin,
    ray_tracing::{Camera, Color, HyperSphere, MainCamera, Material},
};
use transform::{Rotor, Transform, TransformPlugin};

mod movement_control;

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
        ChunksPlugin,
        RenderPlugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, (handle_cursor_locking, movement_controls, orbit));

    if PRINT_FPS {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Update, print_diagnostics);
    }

    app.run()
}

#[derive(Component)]
struct Orbit;

fn setup(mut commands: Commands) {
    commands.spawn((
        MovementControl {
            main_transform: Transform::translation(-3.0, 0.0, 0.0, 0.0),
            xy_rotation: Rotor::IDENTITY,
        },
        Camera::default(),
        MainCamera,
    ));
    commands.spawn((
        Transform::IDENTITY,
        Material {
            base_color: Color {
                red: 0.8,
                green: 0.3,
                blue: 0.2,
            },
        },
        HyperSphere { radius: 1.0 },
    ));
    commands.spawn((
        Transform::IDENTITY,
        Material {
            base_color: Color {
                red: 0.2,
                green: 0.3,
                blue: 0.8,
            },
        },
        HyperSphere { radius: 0.3 },
        Orbit,
    ));
}

fn orbit(time: Res<Time>, mut transforms: Query<&mut Transform, With<Orbit>>) {
    transforms.par_iter_mut().for_each(|mut transform| {
        let (sin, cos) = (time.elapsed_secs() * core::f32::consts::TAU * 0.25).sin_cos();
        *transform = Transform::translation(sin * 3.0, 0.0, cos * 3.0, 0.0);
    });
}

fn print_diagnostics(d: Res<DiagnosticsStore>) {
    if let Some(fps) = d.get_measurement(&FrameTimeDiagnosticsPlugin::FPS) {
        info!("FPS: {}", fps.value);
    }
}
