use bevy::prelude::*;
use bevy_mod_billboard::{BillboardTextBundle, BillboardTextureBundle};
use bevy_mod_outline::{OutlineBundle, OutlineVolume};

use crate::{
    controller::CharacterController,
    health::{Health, UpdateHealth},
    Enemy, GameState, InGame,
};

pub struct BeaconPlugin;

impl Plugin for BeaconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (
                    clear_enemies_in_safezone,
                    heal_player_in_safezone,
                    activate_beacon,
                    update_activation_timers,
                    show_beacons,
                    highlight_nearby_beacons,
                    update_beacon_ui,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
                change_beacon_color,
            ),
        )
        .add_systems(OnEnter(GameState::BeginGame), spawn_beacon_ui)
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
pub struct ActivationCountdown;

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
    server: Res<AssetServer>,
) {
    let player = players.single();

    if keyboard.pressed(KeyCode::Space) {
        for (entity, transform, _, mut state) in beacons
            .iter_mut()
            .filter(|(_, _, _, state)| **state == BeaconState::Offline)
        {
            if player.translation.distance(transform.translation) <= 8.0 {
                *state = BeaconState::Activated;

                commands.entity(entity).insert(ActivationTimer::default());

                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        BillboardTextBundle {
                            transform: Transform::from_xyz(0., 3.0, 0.)
                                .with_scale(Vec3::splat(0.0085)),
                            text: Text::from_section(
                                "15",
                                TextStyle {
                                    font_size: 40.,
                                    color: Color::BLACK,
                                    font: server.load("fonts/motion-control.bold.otf"),
                                },
                            )
                            .with_alignment(TextAlignment::Center),
                            ..default()
                        },
                        ActivationCountdown,
                    ));
                });

                info!("Beacon activated!");
            }
        }
    }
}

fn highlight_nearby_beacons(
    mut commands: Commands,
    players: Query<&Transform, (With<CharacterController>, Without<Beacon>)>,
    beacons_without_outlines: Query<
        (Entity, &Transform, &BeaconState),
        (With<Beacon>, Without<Outline>),
    >,
    beacons_with_outlines: Query<(Entity, &Transform, &BeaconState, &Children), (With<Beacon>)>,
    children: Query<&Children>,
    meshes: Query<Entity, With<Handle<Mesh>>>,
    outlines: Query<&mut OutlineVolume>,
) {
    let player = players.single();

    for (entity, transform, _) in beacons_without_outlines
        .iter()
        .filter(|(_, _, state)| **state == BeaconState::Offline)
    {
        if player.translation.distance(transform.translation) <= 8.0 {
            /*commands.entity(entity).insert(OutlineBundle {
                outline: OutlineVolume {
                    visible: true,
                    width: 2.0,
                    colour: Color::GREEN,
                },
                ..Default::default()
            });*/

            for descendant in children.iter_descendants(entity) {
                if let Ok(_) = meshes.get(descendant) {
                    commands.entity(descendant).insert(OutlineBundle {
                        outline: OutlineVolume {
                            visible: true,
                            width: 2.0,
                            colour: Color::GREEN,
                        },
                        ..Default::default()
                    });
                }
            }
        }
    }

    for (entity, transform, state, _) in beacons_with_outlines.iter()
    //.filter(|(_, _, state)| **state == BeaconState::Offline)
    {
        if player.translation.distance(transform.translation) > 8.0
            || *state != BeaconState::Offline
        {
            //commands.entity(entity).remove::<OutlineVolume>();

            for descendant in children.iter_descendants(entity) {
                if let Ok(_) = outlines.get(descendant) {
                    commands.entity(descendant).remove::<OutlineBundle>();
                }
            }
        }
    }
}

fn change_beacon_color(
    mut commands: Commands,
    beacons: Query<(Entity, &BeaconState), Or<(Changed<BeaconState>, Added<BeaconState>)>>,
    children: Query<&Children>,
    mat_query: Query<(&mut Handle<StandardMaterial>, &Name)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (beacon, state) in beacons.iter() {
        for descendant in children.iter_descendants(beacon) {
            if let Ok((_, name)) = mat_query.get(descendant) {
                if name.as_str().contains("Cube.001") {
                    let color = match state {
                        BeaconState::Offline => Color::RED,
                        BeaconState::Activated => Color::ORANGE,
                        BeaconState::Online => Color::GREEN,
                    };

                    let mat = materials.add((color * 7.0).into());

                    commands.entity(descendant).insert(mat);
                }
            }
        }
    }
}

fn update_activation_timers(
    time: Res<Time>,
    mut commands: Commands,
    mut beacons: Query<(
        Entity,
        &mut ActivationTimer,
        &mut BeaconState,
        &Transform,
        &Children,
    )>,
    mut countdown_displays: Query<(Entity, &mut Text), With<ActivationCountdown>>,
) {
    for (entity, mut timer, mut state, _, kiddos) in beacons.iter_mut() {
        //countdown_displays;

        for kiddo in kiddos.iter() {
            if let Ok((_, mut text)) = countdown_displays.get_mut(*kiddo) {
                text.sections[0].value =
                    ((timer.0.duration() - timer.0.elapsed()).as_secs() + 1).to_string();
            }
        }

        if timer.0.tick(time.delta()).finished() {
            commands.entity(entity).remove::<ActivationTimer>();

            for kiddo in kiddos.iter() {
                if let Ok((e, _)) = countdown_displays.get(*kiddo) {
                    commands.entity(e).despawn_recursive();
                }
            }

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

#[derive(Component)]
pub struct BeaconCounter;

fn spawn_beacon_ui(mut commands: Commands, server: Res<AssetServer>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                    ..default()
                },
                ..default()
            },
            InGame,
        ))
        .with_children(|parent| {
            let style = TextStyle {
                font_size: 24.,
                color: Color::rgb_u8(56, 193, 235),
                font: server.load("fonts/motion-control.bold.otf"),
            };

            parent.spawn((
                TextBundle::from_sections(vec![
                    TextSection::new(
                        "0",
                        TextStyle {
                            color: Color::rgb_u8(132, 219, 105),
                            ..style.clone()
                        },
                    ),
                    TextSection::new(" / ", style.clone()),
                    TextSection::new(
                        "0",
                        TextStyle {
                            color: Color::rgb_u8(250, 143, 55),
                            ..style.clone()
                        },
                    ), // 132, 219, 105
                    TextSection::new(" / ", style.clone()),
                    TextSection::new("0", style.clone()),
                ]),
                BeaconCounter,
            ));
        });
}

pub fn update_beacon_ui(
    mut query: Query<&mut Text, With<BeaconCounter>>,
    beacons: Query<&BeaconState>,
) {
    for mut text in query.iter_mut() {
        //let mut offline = 0;
        let mut activated = 0;
        let mut online = 0;
        let mut total = 0;

        for state in beacons.iter() {
            total += 1;

            match state {
                BeaconState::Activated => activated += 1,
                BeaconState::Online => online += 1,
                _ => {}
            }
        }

        text.sections[0].value = online.to_string();
        text.sections[2].value = activated.to_string();
        text.sections[4].value = total.to_string();
    }
}

pub fn all_beacons_online(beacons: Query<&BeaconState>) -> bool {
    beacons.iter().all(|state| *state == BeaconState::Online)
}
