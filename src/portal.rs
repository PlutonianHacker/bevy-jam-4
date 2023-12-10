use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use rand::Rng;

use crate::{
    beacon::{Beacon, BeaconState},
    behavior::{BehaviorBundle, EnemySpawner},
    health::Health,
    Enemy, GameState,
};

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Portal>()
            .register_type::<PortalId>()
            .add_systems(OnEnter(GameState::Playing), position_projectors)
            .add_systems(
                Update,
                (
                    (close_portals, change_portal_color).chain(),
                    update_enemy_spawners,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Portal;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct PortalId(pub u32);

#[derive(Component, Default, PartialEq, Eq)]
pub enum PortalState {
    #[default]
    Open,
    Closed,
}

#[derive(Component)]
pub struct Projector;

#[derive(Component)]
pub struct HitTimer(pub Timer, pub Handle<StandardMaterial>);

fn position_projectors(
    portals: Query<&Transform, With<Portal>>,
    mut projectors: Query<&mut Transform, (With<Projector>, Without<Portal>)>,
) {
    //let portal_positions = portals.iter().map(|t| t.translation.distance());

    for (mut proj_transform) in projectors.iter_mut() {
        let nearest_portal = portals
            .iter()
            .filter(|t| t.translation.distance(proj_transform.translation) <= 20.0)
            .collect::<Vec<_>>()[0];

        let d = nearest_portal.translation - proj_transform.translation;
        let angle = d.z.atan2(d.x);

        proj_transform.look_at(
            Vec3::new(
                nearest_portal.translation.x,
                0.0,
                nearest_portal.translation.y,
            ),
            Vec3::Y,
        );
    }
}

fn update_enemy_spawners(
    time: Res<Time>,
    mut commands: Commands,
    mut spawners: Query<(&mut EnemySpawner, &Transform, &PortalState)>,
    server: Res<AssetServer>,
) {
    for (mut spawner, transform, _) in spawners
        .iter_mut()
        .filter(|(_, _, state)| **state == PortalState::Open)
    {
        if spawner.tick(time.delta()).just_finished() {
            let mut rng = rand::thread_rng();
            let rand_y_offset = rng.gen_range(0.0..2.0);
            let rand_x_offset = rng.gen_range(-1.5..1.5);

            commands.spawn((
                SceneBundle {
                    scene: server.load("models/creep.glb#Scene0"),
                    transform: Transform::from_xyz(
                        transform.translation.x + rand_x_offset,
                        transform.translation.y + rand_y_offset + 1.5,
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

fn close_portals(
    mut portals: Query<(&Transform, &mut PortalState), With<Portal>>,
    beacons: Query<(&Transform, &BeaconState), (With<Beacon>, Without<Portal>)>,
) {
    for (portal_transform, mut portal_state) in portals
        .iter_mut()
        .filter(|(_, state)| **state == PortalState::Open)
    {
        if beacons
            .iter()
            .filter(|(transform, _)| {
                portal_transform.translation.distance(transform.translation) <= 30.0
            })
            .all(|(_, state)| *state == BeaconState::Online)
        {
            *portal_state = PortalState::Closed;
        }
    }
}

fn change_portal_color(
    mut commands: Commands,
    portals: Query<(Entity, &PortalState), Or<(Added<PortalState>, Changed<PortalState>)>>,
    children: Query<&Children>,
    mat_query: Query<(&mut Handle<StandardMaterial>, &Name)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (beacon, state) in portals.iter() {
        for descendant in children.iter_descendants(beacon) {
            if let Ok((_, name)) = mat_query.get(descendant) {
                info!("{:?}", name.as_str());
                if name.as_str().contains("Portal") {
                    // <- hard coded, cause why not.
                    let color = match state {
                        PortalState::Closed => Color::GREEN,
                        PortalState::Open => Color::RED,
                    };

                    let mat = materials.add((color * 7.0).into());

                    commands.entity(descendant).insert(mat);
                }
            }
        }
    }
}
