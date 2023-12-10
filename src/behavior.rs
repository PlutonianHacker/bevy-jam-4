use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_mod_picking::backends::raycast::bevy_mod_raycast::{prelude::Raycast, primitives::Ray3d};
use bevy_xpbd_3d::prelude::{Collider, LinearVelocity, RigidBody};
use rand::{seq::SliceRandom, Rng};

use crate::{
    controller::{CharacterController, DampingFactor},
    health::{Health, UpdateHealth},
    projectile::Damage,
    Enemy, GameState,
};

pub struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaypointCache>()
            .add_systems(
                Update,
                ((update_enemy_behavior, attack_player).chain(),)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::GameOver), crate::cleanup::<Enemy>);
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

#[derive(Component)]
pub struct AttackCooldownTimer(pub Timer);

#[derive(Bundle)]
pub struct BehaviorBundle {
    pub target: Target,
    pub threshold: ProximityThreshold,
    pub visibility_range: VisibiltyRange,
    pub damping_factor: DampingFactor,
    pub attack_timer: AttackCooldownTimer,
    pub damage: Damage,
}

impl Default for BehaviorBundle {
    fn default() -> Self {
        Self {
            target: Default::default(),
            threshold: ProximityThreshold(6.0),
            visibility_range: VisibiltyRange(25.0),
            damping_factor: DampingFactor(0.92),
            attack_timer: AttackCooldownTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
            damage: Damage(1.0),
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

    let mut rng = rand::thread_rng();

    for (transform, mut lin_vel, threshold, range, mut target) in enemies.iter_mut() {
        let enemy_pos = transform.translation;
        let player_pos = player_transform.translation;

        let dist_to_player = player_pos.distance(enemy_pos);

        //let mut dist;

        if dist_to_player <= 10.0 {
            if dist_to_player >= 6.0 {
                // move towards player.
                let d = player_pos - enemy_pos;
                let angle = d.z.atan2(d.x);

                lin_vel.x += angle.cos() * 40.0 * time.delta_seconds();
                lin_vel.z += angle.sin() * 40.0 * time.delta_seconds();
            } else {
                // move away from player.
                let d = player_pos - enemy_pos;
                let angle = d.z.atan2(d.x) + PI;

                lin_vel.x += angle.cos() * 40.0 * time.delta_seconds();
                lin_vel.z += angle.sin() * 40.0 * time.delta_seconds();
            }
        } else if let Some(waypoint) = target.0 {
            let dist_to_waypoint = waypoint.distance(enemy_pos);

            if dist_to_waypoint > threshold.0 {
                // move towards waypoint.
                let d = target.0.unwrap() - enemy_pos;
                let angle = d.z.atan2(d.x);

                lin_vel.x += angle.cos() * 40.0 * time.delta_seconds();
                lin_vel.z += angle.sin() * 40.0 * time.delta_seconds();
            } else {
                target.1 = target.0;
                target.0 = None;
            }
        } else {
            // pick a new waypoint to move towards.
            let nearby_waypoints = waypoints
                .0
                .iter()
                .filter(|point| {
                    point.position.distance(enemy_pos) <= range.0
                    //&& !(target.1.is_some() && target.1.unwrap() == point.position)
                })
                .collect::<Vec<_>>();

            target.0 = Some(
                nearby_waypoints
                    .choose(&mut rng)
                    .as_ref()
                    .unwrap_or(&&&waypoints.0[0]) // super cursed.
                    .position,
            );
        }

        /*if target.2.is_none() {
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
        } else if target.2.is_some() && player_transform.translation.distance(enemy_pos) > 5.0 {
            let d = player_transform.translation - enemy_pos;
            let angle = d.z.atan2(d.x);

            lin_vel.x += angle.cos() * 40.0 * time.delta_seconds();
            lin_vel.z += angle.sin() * 40.0 * time.delta_seconds();
        }*/
    }
}

fn attack_player(
    time: Res<Time>,
    player: Query<(Entity, &Transform), (With<CharacterController>, Without<Enemy>)>,
    mut enemies: Query<(&Transform, &Damage, &mut AttackCooldownTimer), With<Enemy>>,
    mut event_writer: EventWriter<UpdateHealth>,
) {
    let (player, player_transform) = player.single();

    for (enemy_transform, damage, mut timer) in enemies.iter_mut() {
        if player_transform
            .translation
            .distance(enemy_transform.translation)
            <= 7.0
        {
            if timer.0.tick(time.delta()).just_finished() {
                event_writer.send(UpdateHealth(player, -damage.0));
            }
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
