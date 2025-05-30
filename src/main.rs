use bevy::{
    a11y::AccessibilityPlugin,
    app::{App, AppExit, PanicHandlerPlugin, Startup, TaskPoolPlugin, Update},
    asset::AssetPlugin,
    diagnostic::{
        DiagnosticsPlugin, DiagnosticsStore, FrameCountPlugin, FrameTimeDiagnosticsPlugin,
    },
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        event::EventReader,
        query::With,
        system::{Commands, Query, Res},
    },
    input::{
        ButtonInput, InputPlugin,
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion},
    },
    log::{LogPlugin, info},
    time::{Time, TimePlugin},
    window::{CursorGrabMode, PrimaryWindow, Window, WindowPlugin},
    winit::WinitPlugin,
};
use render::{Camera, MainCamera, RenderPlugin};
use transform::{Rotor, Transform, TransformPlugin};

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
    .add_systems(Update, (handle_cursor_locking, movement_controls));

    if PRINT_FPS {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Update, print_diagnostics);
    }

    app.run()
}

#[derive(Component, Default)]
pub struct MovementControl {
    main_transform: Transform,
    xy_rotation: Rotor,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Transform::translation(-3.0, 0.0, 0.0, 0.0),
        Camera::default(),
        MainCamera,
        MovementControl::default(),
    ));
}

fn handle_cursor_locking(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut primary_window = primary_window
        .single_mut()
        .expect("there should only be one primary window");
    if mouse_buttons.just_pressed(MouseButton::Left)
        || mouse_buttons.just_pressed(MouseButton::Right)
    {
        primary_window.cursor_options.visible = false;
        primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }
    if !primary_window.cursor_options.visible
        && !mouse_buttons.pressed(MouseButton::Left)
        && !mouse_buttons.pressed(MouseButton::Right)
    {
        primary_window.cursor_options.visible = true;
        primary_window.cursor_options.grab_mode = CursorGrabMode::None;
    }
}

fn movement_controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse: EventReader<MouseMotion>,
    mut transform: Query<(&mut Transform, &mut MovementControl)>,
) {
    let (mut out_transform, mut movement_control) = transform
        .single_mut()
        .expect("there should only be one entity with MovementControl");

    {
        const MOUSE_SENSITIVITY: f32 = 0.01;
        for event in mouse.read() {
            if mouse_buttons.pressed(MouseButton::Left) {
                if event.delta.x != 0.0 {
                    movement_control.main_transform = movement_control
                        .main_transform
                        .then(Transform::rotation_xz(event.delta.x * MOUSE_SENSITIVITY));
                }
                if event.delta.y != 0.0 {
                    movement_control.xy_rotation = movement_control
                        .xy_rotation
                        .then(Rotor::rotation_xy(-event.delta.y * MOUSE_SENSITIVITY));
                }
            }
            if mouse_buttons.pressed(MouseButton::Right) {
                if event.delta.x != 0.0 {
                    movement_control.main_transform = movement_control
                        .main_transform
                        .then(Transform::rotation_xw(event.delta.x * MOUSE_SENSITIVITY));
                }
                if event.delta.y != 0.0 {
                    movement_control.main_transform = movement_control
                        .main_transform
                        .then(Transform::rotation_zw(-event.delta.y * MOUSE_SENSITIVITY));
                }
            }
        }
    }

    {
        const MOVEMENT_SPEED: f32 = 1.0;
        let dt = time.delta_secs();
        let (mut x, mut y, mut z, mut w) = (0.0, 0.0, 0.0, 0.0);
        if keys.pressed(KeyCode::KeyW) {
            x += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            y += 1.0;
        }
        if keys.pressed(KeyCode::KeyQ) {
            z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyE) {
            z += 1.0;
        }
        if keys.pressed(KeyCode::KeyR) {
            w += 1.0;
        }
        if keys.pressed(KeyCode::KeyF) {
            w -= 1.0;
        }
        if x != 0.0 && y != 0.0 && z != 0.0 && w != 0.0 {
            movement_control.main_transform =
                movement_control.main_transform.then(Transform::translation(
                    x * MOVEMENT_SPEED * dt,
                    y * MOVEMENT_SPEED * dt,
                    z * MOVEMENT_SPEED * dt,
                    w * MOVEMENT_SPEED * dt,
                ));
        }
    }

    if movement_control.is_changed() {
        *out_transform = movement_control
            .main_transform
            .then(movement_control.xy_rotation.into());
    }
}

fn print_diagnostics(d: Res<DiagnosticsStore>) {
    if let Some(fps) = d.get_measurement(&FrameTimeDiagnosticsPlugin::FPS) {
        info!("FPS: {}", fps.value);
    }
}
