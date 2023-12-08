use bevy::prelude::*;
use bevy_mod_picking::backends::raycast::bevy_mod_raycast::{prelude::Raycast, primitives::Ray3d};
use bevy_xpbd_3d::prelude::{Collider, LinearVelocity, RigidBody};
use rand::{seq::SliceRandom, Rng};

use crate::{
    controller::{CharacterController, DampingFactor},
    health::Health,
    Enemy, GameState,
};

pub struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaypointCache>().add_systems(
            Update,
            (update_enemy_behavior, update_enemy_spawners, show_waypoints)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct EnemySpawner(pub Timer);

#[derive(Component, Default)]
pub struct Target(pub Option<Vec3>, pub Option<Vec3>, pub Option<Entity>);

impl Target {
    pub fn current(&self) -> Option<Vec3> {
        self.0
    }

    pub fn prev(&self) -> Option<Vec3> {
        self.1
    }
}

/// The maximum distance an enemy can be from a target and be consider next to it.
#[derive(Component)]
pub struct ProximityThreshold(pub f32);

#[derive(Component)]
pub struct VisibiltyRange(pub f32);

#[derive(Bundle)]
pub struct BehaviorBundle {
    pub target: Target,
    pub threshold: ProximityThreshold,
    pub visibility_range: VisibiltyRange,
    pub damping_factor: DampingFactor,
}

impl Default for BehaviorBundle {
    fn default() -> Self {
        Self {
            target: Default::default(),
            threshold: ProximityThreshold(5.0),
            visibility_range: VisibiltyRange(14.0),
            damping_factor: DampingFactor(0.92),
        }
    }
}

#[derive(Resource, Default)]
pub struct WaypointCache(pub Vec<Waypoint>);

#[derive(Debug)]
pub struct Waypoint {
    pub position: Vec3,
    pub neighbors: Vec<usize>,
}

fn update_enemy_behavior(
    time: Res<Time>,
    waypoints: Res<WaypointCache>,
    mut enemies: Query<
        (
            &Transform,
            &mut LinearVelocity,
            &ProximityThreshold,
            &VisibiltyRange,
            &mut Target,
        ),
        With<Enemy>,
    >,
    player: Query<(Entity, &Transform), With<CharacterController>>,
    mut raycast: Raycast,
    mut gizmos: Gizmos,
) {
    let (entity, player_transform) = player.single();

    for (transform, mut lin_vel, threshold, range, mut target) in enemies.iter_mut() {
        let enemy_pos = transform.translation;

        if target.2.is_none() {
            let distance = player_transform.translation.distance(enemy_pos);

            let mut rng = rand::thread_rng();

            if distance <= 10.0 {
                if rng.gen_ratio(10 - distance as u32, 10) {
                    //target.2 = Some(entity);
                }
            }
        }

        if target.2.is_none()
            && target.0.is_some()
            && enemy_pos.distance(target.0.unwrap()) <= threshold.0
        {
            target.1 = target.0;
            target.0 = None;
        }

        if target.0.is_none() && target.2.is_none() {
            let nearby_waypoints = waypoints
                .0
                .iter()
                .filter(|point| {
                    point.position.distance(enemy_pos) <= range.0
                        && !(target.1.is_some() && target.1.unwrap() == point.position)
                })
                .filter(|_pos| {
                    //let data = raycast.debug_cast_ray(Ray3d::new(enemy_pos, **pos - enemy_pos), &default(), &mut gizmos);

                    //data.is_empty() || data[0].1.distance() > pos.distance(enemy_pos)
                    true
                })
                .collect::<Vec<_>>();

            let mut rng = rand::thread_rng();

            target.0 = Some(nearby_waypoints.choose(&mut rng).as_ref().unwrap().position);
        }

        if target.2.is_none() {
            raycast.debug_cast_ray(
                Ray3d::new(enemy_pos, target.0.unwrap() - enemy_pos),
                &default(),
                &mut gizmos,
            );

            let d = target.0.unwrap() - enemy_pos;
            let angle = d.z.atan2(d.x);

            lin_vel.x += angle.cos() * 40.0 * time.delta_seconds();
            lin_vel.z += angle.sin() * 40.0 * time.delta_seconds();
        } else {
            raycast.debug_cast_ray(
                Ray3d::new(enemy_pos, player_transform.translation - enemy_pos),
                &default(),
                &mut gizmos,
            );

            let d = player_transform.translation - enemy_pos;
            let angle = d.z.atan2(d.x);

            lin_vel.x += angle.cos() * 40.0 * time.delta_seconds();
            lin_vel.z += angle.sin() * 40.0 * time.delta_seconds();
        }
    }
}

fn update_enemy_spawners(
    time: Res<Time>,
    mut commands: Commands,
    mut spawners: Query<(&mut EnemySpawner, &Transform)>,
    server: Res<AssetServer>,
) {
    for (mut spawner, transform) in spawners.iter_mut() {
        if spawner.tick(time.delta()).just_finished() {
            let mut rng = rand::thread_rng();
            let rand_y_offset = rng.gen_range(0.0..2.0);
            let rand_x_offset = rng.gen_range(-1.5..1.5);

            commands.spawn((
                SceneBundle {
                    scene: server.load("models/creep.glb#Scene0"),
                    transform: Transform::from_xyz(
                        transform.translation.x + rand_x_offset,
                        transform.translation.y + rand_y_offset + 3.0,
                        transform.translation.z,
                    ),
                    ..default()
                },
                Enemy,
                Health::new(100.0),
                Collider::capsule(0.8, 0.6),
                RigidBody::Kinematic,
                BehaviorBundle::default(),
            ));
        }
    }
}

fn show_waypoints(waypoints: Res<WaypointCache>, mut gizmos: Gizmos) {
    for waypoint in waypoints.0.iter() {
        gizmos.circle(
            Vec3::new(waypoint.position.x, 1.0, waypoint.position.z),
            Vec3::Y,
            15.0,
            Color::GREEN,
        );
    }
}
