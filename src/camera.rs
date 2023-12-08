use std::f32::consts::PI;

use bevy::{core_pipeline::{tonemapping::Tonemapping, bloom::BloomSettings}, prelude::*};

use crate::controller::CharacterController;

pub struct FpsCameraPlugin;

impl Plugin for FpsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(PostUpdate, follow_player);
    }
}

#[derive(Component)]
pub struct FpsCamera;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            //transform: Transform::from_xyz(0.0, 60.0, -60.0).with_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 2.0, 0.0, 0.0)),
            //.with_rotation(Quat::from_rotation_y(90.0 * (PI / 180.0))), //.looking_at(Vec3::new(), Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 45.0 * (PI / 180.0),
                ..default()
            }),
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::AcesFitted,
            ..default()
        },
        FpsCamera,
        BloomSettings {
            ..default()
        }
    ));
}

fn follow_player(
    controller: Query<&Transform, (With<CharacterController>, Changed<Transform>)>,
    mut camera: Query<&mut Transform, (With<FpsCamera>, Without<CharacterController>)>,
) {
    let Ok(controller_transform) = controller.get_single() else {
        return;
    };

    let mut camera_transform = camera.single_mut();
    *camera_transform = *controller_transform;
    camera_transform.translation.y += 1.0;
}   
