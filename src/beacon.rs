use bevy::prelude::*;

use crate::{
    controller::CharacterController,
    health::{Health, UpdateHealth},
    Enemy, GameState,
};

pub struct BeaconPlugin;

impl Plugin for BeaconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                clear_enemies_in_safezone,
                heal_player_in_safezone,
                activate_beacon,
                update_activation_timers,
                show_beacons,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::GameOver), crate::cleanup::<Beacon>);
    }
}

#[derive(Component, Default)]
pub struct Beacon {
    pub activation_radius: f32,
    pub safe_zone_radius: f32,
    pub heal_factor: f32,
}

#[derive(Component, PartialEq, Eq, Copy, Clone, Hash, Debug, Default)]
pub enum BeaconState {
    #[default]
    Offline,
    Activated,
    Online,
}

#[derive(Component)]
pub struct ActivationTimer(pub Timer);

impl Default for ActivationTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(15.0, TimerMode::Once))
    }
}

#[derive(Component, Default)]
pub struct BeaconBundle {
    pub beacon: Beacon,
    pub beacon_state: BeaconState,
}

fn clear_enemies_in_safezone(
    mut writer: EventWriter<UpdateHealth>,
    enemies: Query<(Entity, &Transform, &Health), With<Enemy>>,
    beacons: Query<(&Transform, &Beacon, &BeaconState), Without<Enemy>>,
) {
    for (transform, beacon, _) in beacons
        .iter()
        .filter(|(_, _, state)| **state == BeaconState::Online)
    {
        for (entity, enemy_transform, health) in enemies.iter() {
            let beacon_pos = transform.translation;
            let enemy_pos = enemy_transform.translation;

            if beacon_pos.distance(enemy_pos) <= beacon.safe_zone_radius {
                writer.send(UpdateHealth(entity, -health.amount));
            }
        }
    }
}

pub struct HealZoneTimer(pub Timer);

impl Default for HealZoneTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

fn heal_player_in_safezone(
    time: Res<Time>,
    players: Query<(Entity, &Transform), With<CharacterController>>,
    beacons: Query<(&Transform, &Beacon, &BeaconState), Without<CharacterController>>,
    mut timer: Local<HealZoneTimer>,
    mut writer: EventWriter<UpdateHealth>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (transform, beacon, _) in beacons
            .iter()
            .filter(|(_, _, state)| **state == BeaconState::Online)
        {
            for (entity, player_transform) in players.iter() {
                let beacon_pos = transform.translation;
                let player_pos = player_transform.translation;

                if beacon_pos.distance(player_pos) <= beacon.safe_zone_radius {
                    writer.send(UpdateHealth(entity, beacon.heal_factor));
                }
            }
        }
    }
}

fn activate_beacon(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    players: Query<&Transform, With<CharacterController>>,
    mut beacons: Query<
        (Entity, &Transform, &Beacon, &mut BeaconState),
        Without<CharacterController>,
    >,
) {
    let player = players.single();

    if keyboard.pressed(KeyCode::Space) {
        for (entity, transform, _, mut state) in beacons
            .iter_mut()
            .filter(|(_, _, _, state)| **state == BeaconState::Offline)
        {
            if player.translation.distance(transform.translation) <= 5.0 {
                *state = BeaconState::Activated;

                commands.entity(entity).insert(ActivationTimer::default());

                info!("Beacon activated!");
            }
        }
    }
}

fn update_activation_timers(
    time: Res<Time>,
    mut commands: Commands,
    mut beacons: Query<(Entity, &mut ActivationTimer, &mut BeaconState, &Transform)>,
) {
    for (entity, mut timer, mut state, _) in beacons.iter_mut() {
        if timer.0.tick(time.delta()).finished() {
            commands.entity(entity).remove::<ActivationTimer>();

            *state = BeaconState::Online;

            info!("Beacon online!");
        }
    }
}

fn show_beacons(beacons: Query<(&Transform, &Beacon, &BeaconState)>, mut gizmos: Gizmos) {
    for (transform, beacon, state) in beacons.iter() {
        match state {
            BeaconState::Offline => {}
            BeaconState::Activated => {
                gizmos.circle(
                    transform.translation,
                    Vec3::Y,
                    beacon.activation_radius,
                    Color::ORANGE,
                );
            }
            BeaconState::Online => {
                gizmos.circle(
                    transform.translation,
                    Vec3::Y,
                    beacon.safe_zone_radius,
                    Color::GREEN,
                );
            }
        }
    }
}

pub fn all_beacons_online(beacons: Query<&BeaconState>) -> bool {
    beacons.iter().all(|state| *state == BeaconState::Online)
}
