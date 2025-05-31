use bevy::{
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        event::EventReader,
        query::With,
        system::{Query, Res},
    },
    input::{
        ButtonInput,
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion},
    },
    time::Time,
    window::{CursorGrabMode, PrimaryWindow, Window},
};
use transform::{Rotor, Transform};

#[derive(Component, Default)]
#[require(Transform)]
pub struct MovementControl {
    pub main_transform: Transform,
    pub xy_rotation: Rotor,
}

pub fn handle_cursor_locking(
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

pub fn movement_controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse: EventReader<MouseMotion>,
    mut transform: Query<(&mut Transform, &mut MovementControl)>,
) {
    let (mut out_transform, mut movement_control) = transform
        .single_mut()
        .expect("there should only be one entity with MovementControl");

    let mut moved_or_rotated = false;

    {
        const MOUSE_SENSITIVITY: f32 = 0.01;

        for event in mouse.read() {
            if mouse_buttons.pressed(MouseButton::Left) {
                if event.delta.x != 0.0 {
                    movement_control.main_transform = movement_control
                        .main_transform
                        .then(Transform::rotation_xz(event.delta.x * MOUSE_SENSITIVITY));
                    moved_or_rotated = true;
                }
                if event.delta.y != 0.0 {
                    movement_control.xy_rotation = movement_control
                        .xy_rotation
                        .then(Rotor::rotation_xy(-event.delta.y * MOUSE_SENSITIVITY));
                    moved_or_rotated = true;
                }
            }
            if mouse_buttons.pressed(MouseButton::Right) {
                if event.delta.x != 0.0 {
                    movement_control.main_transform = movement_control
                        .main_transform
                        .then(Transform::rotation_zw(event.delta.x * MOUSE_SENSITIVITY));
                    moved_or_rotated = true;
                }
                if event.delta.y != 0.0 {
                    movement_control.main_transform = movement_control
                        .main_transform
                        .then(Transform::rotation_xw(-event.delta.y * MOUSE_SENSITIVITY));
                    moved_or_rotated = true;
                }
            }
        }
    }

    {
        const MOVEMENT_SPEED: f32 = 3.0;

        let dt = time.delta_secs();
        let (mut x, mut y, mut z, mut w) = (0.0, 0.0, 0.0, 0.0);
        if keys.pressed(KeyCode::KeyW) {
            x += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            z += 1.0;
        }
        if keys.pressed(KeyCode::KeyQ) {
            y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyE) {
            y += 1.0;
        }
        if keys.pressed(KeyCode::KeyR) {
            w += 1.0;
        }
        if keys.pressed(KeyCode::KeyF) {
            w -= 1.0;
        }
        if x != 0.0 || y != 0.0 || z != 0.0 || w != 0.0 {
            movement_control.main_transform =
                movement_control.main_transform.then(Transform::translation(
                    x * MOVEMENT_SPEED * dt,
                    y * MOVEMENT_SPEED * dt,
                    z * MOVEMENT_SPEED * dt,
                    w * MOVEMENT_SPEED * dt,
                ));
            moved_or_rotated = true;
        }
    }

    if moved_or_rotated || movement_control.is_changed() {
        *out_transform = movement_control
            .main_transform
            .then(movement_control.xy_rotation.into());
    }
}
