use crate::mantis::CenterOfMass;
use crate::{MovementMode, WorldOptions};
use bevy::prelude::*;

pub fn lemniscate(t: f32) -> Vec3 {
    return Vec3::new(
        ((2.0 * t).cos()).sqrt() * t.cos(),
        0.0,
        ((2.0 * t).cos()).sqrt() * t.sin(),
    );
}

pub fn controls_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            keyboard_controls,
            mouse_controls,
            switch_movement_mode,
            legacy_controls,
        ),
    );
}

pub fn keyboard_controls(
    mut mantis: Single<(&mut Transform, &CenterOfMass)>,
    input: Res<ButtonInput<KeyCode>>,
    mode: Res<WorldOptions>,
    time: Res<Time>,
) {
    if mode.movement_mode != MovementMode::Keyboard {
        return;
    }
    let mut transform = (0.0, 0.0);
    if input.pressed(KeyCode::KeyW) {
        transform.1 += 1.0;
    } else if input.pressed(KeyCode::KeyS) {
        transform.1 -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        transform.0 -= 1.0;
    } else if input.pressed(KeyCode::KeyD) {
        transform.0 += 1.0;
    }
    let speed = mantis.1.speed;
    mantis.0.translation.x += transform.0 * speed * time.delta_secs();
    mantis.0.translation.z += transform.1 * speed * time.delta_secs();
}

pub fn mouse_controls(
    mut mantis: Single<(&mut Transform, &CenterOfMass)>,
    world_options: Res<WorldOptions>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    input: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    if world_options.movement_mode != MovementMode::Mouse {
        return;
    }

    let camera = camera_query.0;
    let camera_transform: &GlobalTransform = camera_query.1;
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    let plane = InfinitePlane3d::new(Vec3::Y);
    if let Some(distance) = ray.intersect_plane(Vec3::ZERO, plane) {
        if input.pressed(MouseButton::Left) {
            let speed = mantis.1.speed;
            let mut world_pos = ray.origin + ray.direction * distance;
            let mantis_current_pos = mantis.0.translation;
            world_pos.y = mantis_current_pos.y;
            let dir = (world_pos - mantis_current_pos).normalize();

            mantis.0.translation += dir * speed * time.delta_secs();
            mantis.0.look_to(dir, Vec3::Y);
        }
    }
}

pub fn auto_movement(
    mut mantis: Single<&mut Transform, With<CenterOfMass>>,
    mut world_options: ResMut<WorldOptions>,
    time: Res<Time>,
) {
    if world_options.movement_mode != MovementMode::Auto {
        return;
    }
}

pub fn switch_movement_mode(mut mode: ResMut<WorldOptions>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyM) {
        match mode.movement_mode {
            MovementMode::Mouse => {
                mode.movement_mode = MovementMode::Keyboard;
            }
            MovementMode::Keyboard => {
                mode.movement_mode = MovementMode::Auto;
            }
            MovementMode::Auto => {
                mode.movement_mode = MovementMode::Legacy;
            }
            MovementMode::Legacy => {
                mode.movement_mode = MovementMode::Mouse;
            }
        }
    }
}

pub fn legacy_controls(
    mode: Res<WorldOptions>,
    input: Res<ButtonInput<KeyCode>>,
    mut mantis_params: Single<(&mut Transform, &CenterOfMass)>,
    time: Res<Time>,
) {
    if mode.movement_mode != MovementMode::Legacy {
        return;
    }
    let speed = mantis_params.1.speed;

    //rotators
    if input.pressed(KeyCode::KeyD) {
        mantis_params.0.rotation *= Quat::from_rotation_y(-speed * time.delta_secs());
    } else if input.pressed(KeyCode::KeyA) {
        mantis_params.0.rotation *= Quat::from_rotation_y(speed * time.delta_secs());
    }

    //movement
    let forward_vec = *mantis_params.0.forward();
    if input.pressed(KeyCode::KeyW) {
        mantis_params.0.translation += forward_vec * speed * time.delta_secs();
    } else if input.pressed(KeyCode::KeyS) {
        mantis_params.0.translation -= forward_vec * speed * time.delta_secs();
    }
}
