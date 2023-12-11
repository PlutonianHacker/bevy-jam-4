use std::f32::consts::PI;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
};
use bevy_mod_billboard::prelude::BillboardPlugin;
use bevy_mod_outline::{AutoGenerateOutlineNormalsPlugin, OutlinePlugin};
use bevy_mod_picking::{
    backends::raycast::bevy_mod_raycast::{prelude::Raycast, primitives::Ray3d},
    prelude::{DefaultHighlightingPlugin, PointerLocation},
    DefaultPickingPlugins,
};
use bevy_xpbd_3d::prelude::{
    AsyncCollider, AsyncSceneCollider, Collider, ComputedCollider, PhysicsDebugPlugin,
    PhysicsPlugins, RigidBody, Sensor, ShapeCaster,
};
use game::{
    beacon::{all_beacons_online, Beacon, BeaconPlugin, BeaconState},
    behavior::{BehaviorBundle, BehaviorPlugin, EnemySpawner, Waypoint, WaypointCache},
    camera::FpsCameraPlugin,
    cleanup,
    controller::{CharacterController, CharacterControllerBundle, CharacterControllerPlugin},
    door::{Door, DoorPlugin, DoorTrigger},
    game_over::{GameOverPlugin, Outcome},
    health::{Health, HealthPlugin},
    levels::{Level, Levels, LevelsPlugin},
    loading::LoadingPlugin,
    portal::{Portal, PortalPlugin, PortalState, Projector},
    projectile::{ProjectilePlugin, Weapon, Damage, Speed},
    Enemy, GameState, InGame, KillCount, weapon::{AutoFire, FiringRate, WeaponsPlugin},
};

/*
#[derive(Resource)]
pub struct LevelData(Handle<Gltf>, Vec<Handle<Gltf>>);

fn load_level(
    mut commands: Commands,
    level: Res<LevelData>,
    gltfs: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut waypoint_cache: ResMut<WaypointCache>,
    //mut raycast: Raycast,
    mut meshes: ResMut<Assets<Mesh>>,
    server: Res<AssetServer>,
) {
    if let Some(gltf) = gltfs.get(&level.0) {
        commands.spawn((SceneBundle {
            scene: gltf.scenes[0].clone(),
            ..default()
        },));

        let mut waypoints = Vec::new();

        for (name, node) in gltf
            .named_nodes
            .iter()
            .map(|(name, node)| (name, gltf_nodes.get(node).unwrap()))
        {
            println!("{name:?}");

            match &name[..] {
                "Door" => {
                    let gltf_mesh = gltf_meshes.get(node.mesh.as_ref().unwrap()).unwrap();

                    let entity = commands
                        .spawn((
                            PbrBundle {
                                mesh: gltf_mesh.primitives[0].mesh.clone(),
                                material: materials.add(Color::NONE.into()), //gltf_mesh.primitives[0].material.clone().unwrap(), //gltf_mesh.primitives[0].mesh.clone(),
                                transform: node.transform,
                                ..default()
                            },
                            Door,
                            RigidBody::Static,
                            AsyncCollider(ComputedCollider::TriMesh),
                        ))
                        .id();

                    commands.spawn((
                        Sensor,
                        RigidBody::Static,
                        DoorTrigger(entity),
                        Collider::cuboid(3.0, 3.0, 3.0),
                        SpatialBundle::from_transform(node.transform),
                    ));
                }
                "Collider" => {
                    let gltf_mesh = gltf_meshes.get(node.mesh.as_ref().unwrap()).unwrap();

                    commands.spawn((
                        PbrBundle {
                            mesh: gltf_mesh.primitives[0].mesh.clone(),
                            material: materials.add(Color::NONE.into()), //gltf_mesh.primitives[0].mesh.clone(),
                            ..default()
                        },
                        RigidBody::Static,
                        AsyncCollider(ComputedCollider::TriMesh),
                    ));
                }
                "SpawnPoint" => {
                    let mut collider = Collider::capsule(2.0, 0.4);
                    collider.set_scale(Vec3::ONE * 0.99, 10);

                    let entity = commands
                        .spawn((
                            PbrBundle {
                                mesh: meshes.add(
                                    shape::Capsule {
                                        radius: 0.4,
                                        depth: 2.0,
                                        ..default()
                                    }
                                    .into(),
                                ),
                                material: materials.add(Color::ALICE_BLUE.into()),
                                transform: Transform::from_xyz(
                                    node.transform.translation.x,
                                    3.0,
                                    node.transform.translation.z,
                                ),
                                ..default()
                            },
                            CharacterControllerBundle {
                                ..Default::default()
                            },
                            Health::new(350.0),
                            Collider::capsule(1.0, 0.5),
                            RigidBody::Kinematic,
                            ShapeCaster::new(collider, Vec3::ZERO, Quat::default(), Vec3::NEG_Y)
                                .with_max_time_of_impact(0.2),
                            //SpatialListener::new(4.0),
                            InGame,
                        ))
                        .id();

                    commands
                        .spawn((
                            PbrBundle {
                                mesh: meshes.add(shape::Box::new(0.25, 0.25, 2.0).into()),
                                material: materials.add(Color::BLACK.into()),
                                transform: Transform::from_xyz(1.0, 0.0, -2.5),
                                ..default()
                            },
                            Weapon,
                            Name::new("Yon weapon of choice"),
                        ))
                        .set_parent(entity);
                }
                _ => {}
            }

            if name.contains("Waypoint") {
                waypoints.push(node.transform.translation);
            }

            if name.contains("Projector") {
                commands.spawn((
                    SceneBundle {
                        scene: server.load("models/portal_projector.glb#Scene0"),
                        transform: node.transform,
                        ..default()
                    },
                    Projector,
                    RigidBody::Static,
                    Collider::capsule(2.6, 2.0),
                    //EnemySpawner(Timer::from_seconds(1.5, TimerMode::Repeating)),
                    //AsyncSceneCollider::new(Some(ComputedCollider::ConvexHull)),
                ));
            }

            if name.contains("Portal") {
                commands.spawn((
                    SceneBundle {
                        scene: server.load("models/portal.glb#Scene0"), //.named_scenes["Scene.001"].clogltfne(),
                        transform: node.transform,
                        ..default()
                    },
                    Portal,
                    PortalState::default(),
                    EnemySpawner(Timer::from_seconds(1.5, TimerMode::Repeating)),
                ));
            }

            if name.contains("Beacon") {
                commands.spawn((
                    SceneBundle {
                        scene: server.load("models/beacon.glb#Scene0"),
                        transform: Transform::from_xyz(
                            node.transform.translation.x,
                            2.0,
                            node.transform.translation.z,
                        ),
                        ..default()
                    },
                    Beacon {
                        activation_radius: 5.0,
                        safe_zone_radius: 10.0,
                        heal_factor: 5.0,
                    },
                    BeaconState::Offline,
                ));
            }

            //println!("{:?}", waypoints.0);

            /*if name.contains("Lamp") {
                commands.spawn((PointLightBundle {
                    point_light: PointLight {
                        intensity: 200.0,
                        radius: 1.0,
                        color: Color::RED,
                        shadows_enabled: true,
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        node.transform.translation.x,
                        node.transform.translation.y - 2.5,
                        node.transform.translation.z,
                    ),
                    ..default()
                },));
            }*/
        }

        next_state.set(GameState::BeginGame);

        for (current_offset, current_pos) in waypoints.iter().enumerate() {
            let mut waypoint = Waypoint {
                position: *current_pos,
                neighbors: Vec::new(),
            };

            for (other_offset, other_pos) in waypoints.iter().enumerate() {
                if other_offset != current_offset {
                    let dir = *other_pos - *current_pos;

                    //let data = raycast.cast_ray(Ray3d::new(*current_pos, dir), &default());

                    //println!("{:?}", data.len());

                    /*let mut blocked = false;
                    'hit: for (_, hit) in data.iter() {
                        if hit.distance() < other_pos.distance(*current_pos) {
                            blocked = true;
                            break 'hit;
                        }
                    }*/

                    //if !blocked {
                        waypoint.neighbors.push(other_offset);
                    //}
                }
            }

            waypoint_cache.0.push(waypoint);
        }

        //println!("{:#?}", waypoint_cache.0);
    }
}

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(LevelData(
        server.load("models/Level_1.glb"),
        vec![
            server.load("models/red_creep.glb"),
            server.load("models/creep.glb"),
            server.load("models/portal_projector.glb"),
        ],
    ));

    let _: Handle<AudioSource> = server.load("audio/laser-zap.mp3");

    commands.insert_resource(AmbientLight {
        brightness: 1.0,
        color: Color::WHITE,
    });

    commands.insert_resource(Levels {
        current_level: 0,
        data: vec![Level {
            name: "Level 1".to_string(),
            scene: server.load("models/Level_1.glb#Scene0"),
        }],
    });
}

fn setup_game(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /*let mut collider = Collider::capsule(2.0, 0.4);
    collider.set_scale(Vec3::ONE * 0.99, 10);

    let entity = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(
                    shape::Capsule {
                        radius: 0.4,
                        depth: 2.0,
                        ..default()
                    }
                    .into(),
                ),
                material: materials.add(Color::ALICE_BLUE.into()),
                transform: Transform::from_xyz(0.0, 3.0, -40.0),
                ..default()
            },
            CharacterControllerBundle {
                ..Default::default()
            },
            Health::new(350.0),
            Collider::capsule(1.0, 0.5),
            RigidBody::Kinematic,
            ShapeCaster::new(collider, Vec3::ZERO, Quat::default(), Vec3::NEG_Y)
                .with_max_time_of_impact(0.2),
            //SpatialListener::new(4.0),
            InGame,
        ))
        .id();

    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(shape::Box::new(0.25, 0.25, 2.0).into()),
                material: materials.add(Color::BLACK.into()),
                transform: Transform::from_xyz(1.0, 0.0, -2.5),
                ..default()
            },
            Weapon,
            Name::new("Yon weapon of choice"),
        ))
        .set_parent(entity);*/

    /*commands.spawn((
        SceneBundle {
            scene: server.load("models/beacon.glb#Scene0"),
            transform: Transform::from_xyz(0.0, 2.0, -30.0),
            ..default()
        },
        Beacon {
            activation_radius: 5.0,
            safe_zone_radius: 10.0,
            heal_factor: 5.0,
        },
        BeaconState::Offline,
    ));

    commands.spawn((
        SceneBundle {
            scene: server.load("models/beacon.glb#Scene0"),
            transform: Transform::from_xyz(-15.0, 2.0, -65.0),
            ..default()
        },
        Beacon {
            activation_radius: 5.0,
            safe_zone_radius: 10.0,
            heal_factor: 5.0,
        },
        BeaconState::Offline,
    ));*/
}*/

fn go_to_gameover(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::GameOver);

    commands.insert_resource(Outcome::Won);
}

fn pause_game(keyboard: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    //println!("next state!");
    if keyboard.pressed(KeyCode::Return) {
        next_state.set(GameState::Paused);
    }
}

fn unpause_game(keyboard: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    //if keyboard.just_released(KeyCode::Return) {
    //    next_state.set(GameState::Playing);
    //}
}

fn start_gameplay(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}

fn setup_game(
    mut commands: Commands,
    levels: Res<Levels>,
    gltfs: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut waypoint_cache: ResMut<WaypointCache>,
    server: Res<AssetServer>,
) {
    //info!("AGGGHHHH!");
    if let Some(gltf) = gltfs.get(&levels.data[levels.current_level].scene) {
        commands.spawn((
            SceneBundle {
                scene: gltf.scenes[0].clone(),
                ..default()
            },
            InGame,
        ));

        let mut waypoints = Vec::new();

        for (name, node) in gltf
            .named_nodes
            .iter()
            .map(|(name, node)| (name, gltf_nodes.get(node).unwrap()))
        {
            println!("{name:?}");

            match &name[..] {
                "Door" => {
                    let gltf_mesh = gltf_meshes.get(node.mesh.as_ref().unwrap()).unwrap();

                    let entity = commands
                        .spawn((
                            PbrBundle {
                                mesh: gltf_mesh.primitives[0].mesh.clone(),
                                material: materials.add(Color::NONE.into()),
                                transform: node.transform,
                                ..default()
                            },
                            Door,
                            RigidBody::Static,
                            AsyncCollider(ComputedCollider::TriMesh),
                        ))
                        .id();

                    commands.spawn((
                        Sensor,
                        RigidBody::Static,
                        DoorTrigger(entity),
                        Collider::cuboid(3.0, 3.0, 3.0),
                        SpatialBundle::from_transform(node.transform),
                    ));
                }
                "Collider" => {
                    let gltf_mesh = gltf_meshes.get(node.mesh.as_ref().unwrap()).unwrap();

                    commands.spawn((
                        PbrBundle {
                            mesh: gltf_mesh.primitives[0].mesh.clone(),
                            material: materials.add(Color::NONE.into()),
                            ..default()
                        },
                        RigidBody::Static,
                        AsyncCollider(ComputedCollider::TriMesh),
                        InGame,
                    ));
                }
                "SpawnPoint" => {
                    let mut collider = Collider::capsule(2.0, 0.4);
                    collider.set_scale(Vec3::ONE * 0.99, 10);

                    let entity = commands
                        .spawn((
                            PbrBundle {
                                mesh: meshes.add(
                                    shape::Capsule {
                                        radius: 0.4,
                                        depth: 2.0,
                                        ..default()
                                    }
                                    .into(),
                                ),
                                material: materials.add(Color::ALICE_BLUE.into()),
                                transform: Transform::from_xyz(
                                    node.transform.translation.x,
                                    3.0,
                                    node.transform.translation.z,
                                )
                                .with_rotation(node.transform.rotation),
                                ..default()
                            },
                            CharacterControllerBundle {
                                ..Default::default()
                            },
                            Health::new(350.0),
                            Collider::capsule(1.0, 0.5),
                            RigidBody::Kinematic,
                            ShapeCaster::new(collider, Vec3::ZERO, Quat::default(), Vec3::NEG_Y)
                                .with_max_time_of_impact(0.2),
                            InGame,
                        ))
                        .id();

                    commands
                        .spawn((
                            PbrBundle {
                                mesh: meshes.add(shape::Box::new(0.25, 0.25, 2.0).into()),
                                material: materials.add(Color::BLACK.into()),
                                transform: Transform::from_xyz(1.0, 0.0, -2.5),
                                ..default()
                            },
                            Weapon,
                            AutoFire,
                            FiringRate(240.0),
                            Damage(60.0),
                            Speed(25.0),
                            Name::new("Yon weapon of choice"),
                            InGame,
                        ))
                        .set_parent(entity);
                }
                _ => {}
            }

            if name.contains("Waypoint") {
                waypoints.push(node.transform.translation);
            }

            if name.contains("Projector") {
                commands.spawn((
                    SceneBundle {
                        scene: server.load("models/portal_projector.glb#Scene0"),
                        transform: node.transform,
                        ..default()
                    },
                    Projector,
                    RigidBody::Static,
                    Collider::capsule(2.6, 2.0),
                    InGame,
                ));
            }

            if name.contains("Portal") {
                commands.spawn((
                    SceneBundle {
                        scene: server.load("models/portal.glb#Scene0"),
                        transform: node.transform,
                        ..default()
                    },
                    Portal,
                    PortalState::default(),
                    EnemySpawner(Timer::from_seconds(1.0, TimerMode::Repeating)),
                    InGame,
                ));
            }

            if name.contains("Beacon") {
                commands.spawn((
                    SceneBundle {
                        scene: server.load("models/beacon.glb#Scene0"),
                        transform: Transform::from_xyz(
                            node.transform.translation.x,
                            2.0,
                            node.transform.translation.z,
                        )
                        .with_rotation(node.transform.rotation),
                        ..default()
                    },
                    Beacon {
                        activation_radius: 5.0,
                        safe_zone_radius: 10.0,
                        heal_factor: 5.0,
                    },
                    BeaconState::Offline,
                ));
            }
        }

        commands.insert_resource(AmbientLight {
            brightness: 1.0,
            color: Color::WHITE,
        });

        next_state.set(GameState::BeginGame);

        for (current_offset, current_pos) in waypoints.iter().enumerate() {
            let mut waypoint = Waypoint {
                position: *current_pos,
                neighbors: Vec::new(),
            };

            for (other_offset, _other_pos) in waypoints.iter().enumerate() {
                if other_offset != current_offset {
                    waypoint.neighbors.push(other_offset);
                }
            }

            waypoint_cache.0.push(waypoint);
        }
    }
}

fn main() {
    let mut app = App::new();

    app.add_state::<GameState>();
    app.init_resource::<KillCount>();

    app.add_plugins((
        DefaultPlugins,
        (
            DefaultPickingPlugins
                .build()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Liminal Combat".to_string(),
                        canvas: Some("#bevy".to_string()),
                        //prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<DefaultHighlightingPlugin>(),
            PhysicsPlugins::default(),
            BillboardPlugin,
            OutlinePlugin,
            AutoGenerateOutlineNormalsPlugin,
        ),
        //PhysicsDebugPlugin::default(),
        CharacterControllerPlugin,
        FpsCameraPlugin,
        ProjectilePlugin,
        BehaviorPlugin,
        HealthPlugin,
        DoorPlugin,
        BeaconPlugin,
        PortalPlugin,
        GameOverPlugin,
        LevelsPlugin,
        LoadingPlugin,
        WeaponsPlugin,
    ));

    //app.add_plugins((LevelsPlugin, LoadingPlugin));
    //app.add_systems(Startup, setup);
    app.add_systems(Update, (setup_game,).run_if(in_state(GameState::LoadGame)));
    //app.add_systems(OnEnter(GameState::BeginGame), setup_game);
    app.add_systems(
        Update,
        go_to_gameover
            .run_if(in_state(GameState::Playing))
            .run_if(all_beacons_online),
    );

    app.add_systems(
        Update,
        start_gameplay.run_if(in_state(GameState::BeginGame)),
    );

    app.add_systems(Update, pause_game.run_if(in_state(GameState::Playing)));
    app.add_systems(Update, unpause_game.run_if(in_state(GameState::Paused)));

    app.add_systems(OnEnter(GameState::GameOver), cleanup::<InGame>);

    #[cfg(not(release))]
    {
        app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
    }

    app.run();
}
