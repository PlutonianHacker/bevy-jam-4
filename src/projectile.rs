use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_xpbd_3d::{prelude::*, SubstepSchedule, SubstepSet};

use crate::{controller::CharacterController, health::UpdateHealth, Enemy, GameState};

#[derive(Component)]
pub struct Projectile;

#[derive(Component)]
pub struct Damage(pub f32);

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Bundle)]
pub struct ProjectileBundle {
    pub damage: Damage,
    pub speed: Speed,
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_projectile, update_projectiles)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            SubstepSchedule,
            handle_projectile_collisions.in_set(SubstepSet::SolveUserConstraints),
        );
    }
}

#[derive(Component)]
pub struct Weapon;

fn spawn_projectile(
    mut commands: Commands,
    //pointers: Query<&PointerLocation>,
    buttons: Res<Input<MouseButton>>,
    controllers: Query<(Entity, &Transform), With<CharacterController>>,
    mut weapons: Query<&Transform, (With<Weapon>, Without<CharacterController>)>,
    //cameras: Query<&Transform, With<FpCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    children_query: Query<&Children>,
    asset_server: Res<AssetServer>,
) {
    let (entity, transform) = controllers.single();

    if buttons.just_pressed(MouseButton::Left) {
        //let Ok(pointer) = pointers.get_single() else {
        //    return;
        //};

        for c in children_query.iter_descendants(entity) {
            let Ok(weapon_transform) = weapons.get_mut(c) else {
                continue;
            };

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
                ProjectileBundle {
                    damage: Damage(115.0),
                    speed: Speed(30.0),
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

fn update_projectiles(time: Res<Time>, mut projectiles: Query<(&Speed, &mut Transform)>) {
    for (speed, mut transform) in projectiles.iter_mut() {
        transform.translation =
            transform.transform_point(Vec3::new(0.0, -time.delta_seconds() * speed.0, 0.0));
    }
}

fn handle_projectile_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Speed, &Damage)>,
    enemies: Query<Entity, With<Enemy>>,
    mut collision_events: EventReader<CollisionStarted>,
    mut update_health_events: EventWriter<UpdateHealth>,
) {
    for CollisionStarted(a, b) in collision_events.read() {
        let projectile = if projectiles.get(*a).is_ok() {
            *a
        } else if projectiles.get(*b).is_ok() {
            *b
        } else {
            continue;
        };

        let enemy = if enemies.get(*a).is_ok() {
            *a
        } else if enemies.get(*b).is_ok() {
            *b
        } else {
            continue;
        };

        //println!("collision between {projectile:?} and {enemy:?}");
        commands.entity(projectile).despawn_recursive();

        let (_, _, damage) = projectiles.get(projectile).unwrap();

        update_health_events.send(UpdateHealth(enemy, -damage.0));
    }
}
