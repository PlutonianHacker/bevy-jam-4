use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_xpbd_3d::prelude::{Collider, RigidBody};

use crate::{
    controller::CharacterController,
    projectile::{Damage, Projectile, ProjectileBundle, Speed, Weapon},
    GameState,
};

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fire_weapon.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct AutoFire;

/// A weapon's firing rate in bullets per minute.
#[derive(Component)]
pub struct FiringRate(pub f32);

//pub struct Ammo;

#[derive(Component)]
pub struct AutoFireTimer(pub Timer);

fn fire_weapon(
    time: Res<Time>,
    mut commands: Commands,
    inactive_weapons: Query<
        (
            Entity,
            &Transform,
            &FiringRate,
            &Damage,
            &Speed,
            Option<&AutoFire>,
        ),
        (With<Weapon>, Without<AutoFireTimer>),
    >,
    mut active_weapons: Query<
        (
            Entity,
            &Transform,
            &FiringRate,
            &Damage,
            &Speed,
            &mut AutoFireTimer,
            Option<&AutoFire>,
        ),
        With<Weapon>,
    >,
    inputs: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    controllers: Query<(Entity, &Transform), With<CharacterController>>,
    asset_server: Res<AssetServer>,
) {
    let (_, transform) = controllers.single();

    if inputs.just_pressed(MouseButton::Left) {
        for (entity, weapon_transform, firing_rate, damage, speed, _) in inactive_weapons
            .iter()
            .filter(|(_, _, _, _, _, can_auto_fire)| can_auto_fire.is_some())
        {
            let mut timer = Timer::from_seconds(60.0 / firing_rate.0, TimerMode::Repeating);

            commands.entity(entity).insert(AutoFireTimer(timer));

            let mut new_transform = *transform;

            new_transform.translation = new_transform
                .transform_point(weapon_transform.translation + Vec3::new(0.0, 0.0, -1.0)); // + Vec3::new(3.0, 0.0, -1.0));
            new_transform.rotate_local_x(PI / 2.0);

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(
                        shape::Capsule {
                            depth: 0.9,
                            radius: 0.05,
                            ..default()
                        }
                        .into(),
                    ),
                    material: materials.add(Color::rgb(10.0, 0.0, 0.0).into()),
                    transform: new_transform,
                    ..default()
                },
                Projectile,
                ProjectileBundle {
                    damage: Damage(damage.0),
                    speed: Speed(speed.0),
                },
                Collider::capsule(0.25, 0.1),
                RigidBody::Kinematic,
            ));

            commands.spawn(AudioBundle {
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    //volume: Volume::Relative(VolumeLevel::new(5.0)),
                    spatial: true,
                    paused: false,
                    ..default()
                },
                //source: asset_server.load("audio/epic-orchestra-transition.wav"),
                source: asset_server.load("audio/laser-zap.ogg"),
                ..default()
            });
        }
    }

    for (entity, weapon_transform, FiringRate(rate_of_fire), speed, damage, mut timer, auto_fire) in
        active_weapons.iter_mut()
    {
        if inputs.just_released(MouseButton::Left) {
            commands.entity(entity).remove::<AutoFireTimer>();
        }

        for _ in 0..timer.0.tick(time.delta()).times_finished_this_tick() {
            println!("Fire!");
            let mut new_transform = *transform;

            new_transform.translation = new_transform
                .transform_point(weapon_transform.translation + Vec3::new(0.0, 0.0, -1.0)); // + Vec3::new(3.0, 0.0, -1.0));
            new_transform.rotate_local_x(PI / 2.0);

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(
                        shape::Capsule {
                            depth: 0.9,
                            radius: 0.05,
                            ..default()
                        }
                        .into(),
                    ),
                    material: materials.add(Color::rgb(10.0, 0.0, 0.0).into()),
                    transform: new_transform,
                    ..default()
                },
                Projectile,
                ProjectileBundle {
                    damage: Damage(damage.0),
                    speed: Speed(speed.0),
                },
                Collider::capsule(0.25, 0.1),
                RigidBody::Kinematic,
            ));

            commands.spawn(AudioBundle {
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    //volume: Volume::Relative(VolumeLevel::new(5.0)),
                    spatial: true,
                    paused: false,
                    ..default()
                },
                //source: asset_server.load("audio/epic-orchestra-transition.wav"),
                source: asset_server.load("audio/laser-zap.ogg"),
                ..default()
            });
        }
    }
}
