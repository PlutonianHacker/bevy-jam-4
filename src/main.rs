use std::f32::consts::PI;

use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
};
use bevy_gltf_components::ComponentsFromGltfPlugin;
use bevy_mod_picking::{
    backends::raycast::bevy_mod_raycast::{prelude::Raycast, primitives::Ray3d},
    prelude::{DefaultHighlightingPlugin, PointerLocation},
    DefaultPickingPlugins,
};
use bevy_xpbd_3d::{
    math::Quaternion,
    parry::math::Vector,
    prelude::{
        AsyncCollider, AsyncSceneCollider, Collider, ComputedCollider, PhysicsDebugPlugin,
        PhysicsPlugins, RigidBody, Sensor, ShapeCaster,
    },
};
use game::{
    beacon::{all_beacons_online, Beacon, BeaconPlugin, BeaconState},
    behavior::{BehaviorBundle, BehaviorPlugin, EnemySpawner, Waypoint, WaypointCache},
    camera::FpsCameraPlugin,
    cleanup,
    controller::{CharacterController, CharacterControllerBundle, CharacterControllerPlugin},
    door::{Door, DoorPlugin, DoorTrigger},
    game_over::GameOverPlugin,
    health::{Health, HealthPlugin},
    portal::{Portal, PortalPlugin, Projector},
    projectile::{ProjectilePlugin, Weapon},
    Enemy, GameState, InGame, KillCount,
};
use rand::Rng;

#[derive(Resource)]
pub struct Level(Handle<Gltf>, Vec<Handle<Gltf>>);

fn load_level(
    mut commands: Commands,
    level: Res<Level>,
    gltfs: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut waypoint_cache: ResMut<WaypointCache>,
    mut raycast: Raycast,
    server: Res<AssetServer>,
) {
    if let Some(gltf) = gltfs.get(&level.0) {
        //for (name, _handle) in &gltf.named_scenes {
        //    println!("{name:?}");
        //}

        commands.spawn((SceneBundle {
            scene: gltf.scenes[0].clone(), //server.load("models/The_Lab.glb#Scene0"),
            ..default()
        },));

        println!("{:?}", gltf.named_nodes);

        let mut waypoints = Vec::new();

        for (name, node) in gltf
            .named_nodes
            .iter()
            .map(|(name, node)| (name, gltf_nodes.get(node).unwrap()))
        {
            println!("{name:?}");

            match &name[..] {
                "Portal" => {
                    /*let translation = node.transform.translation;
                    commands.spawn((
                        SpatialBundle {
                            transform: Transform::from_xyz(
                                translation.x,
                                translation.y + 0.5,
                                -60.0, //translation.z,
                            ),
                            ..default()
                        },
                        EnemySpawner(Timer::from_seconds(1.0, TimerMode::Repeating)),
                    ));*/
                    commands.spawn((
                        SceneBundle {
                            scene: gltf.named_scenes["Scene.001"].clone(),
                            transform: node.transform,
                            ..default()
                        },
                        Portal,
                        //RigidBody::Static,
                        //AsyncCollider(ComputedCollider::TriMesh),
                    ));
                }
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
                    //EnemySpawner(Timer::from_seconds(1.0, TimerMode::Repeating)),
                    //AsyncSceneCollider::new(Some(ComputedCollider::ConvexHull)),
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

        next_state.set(GameState::Playing);

        for (current_offset, current_pos) in waypoints.iter().enumerate() {
            let mut waypoint = Waypoint {
                position: *current_pos,
                neighbors: Vec::new(),
            };

            for (other_offset, other_pos) in waypoints.iter().enumerate() {
                if other_offset != current_offset {
                    let dir = *other_pos - *current_pos;

                    let data = raycast.cast_ray(Ray3d::new(*current_pos, dir), &default());

                    //println!("{:?}", data.len());

                    let mut blocked = false;
                    'hit: for (_, hit) in data.iter() {
                        if hit.distance() < other_pos.distance(*current_pos) {
                            blocked = true;
                            break 'hit;
                        }
                    }

                    if !blocked {
                        waypoint.neighbors.push(other_offset);
                    }
                }
            }

            waypoint_cache.0.push(waypoint);
        }

        //println!("{:#?}", waypoint_cache.0);
    }
}

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Level(
        server.load("models/The_Lab.glb"),
        vec![
            server.load("models/red_creep.glb"),
            server.load("models/creep.glb"),
            server.load("models/portal_projector.glb"),
        ],
    ));

    let _: Handle<AudioSource> = server.load("audio/laser-zap.mp3");

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

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                shape::Capsule {
                    depth: 2.0,
                    radius: 0.6,
                    ..default()
                }
                .into(),
            ),
            material: materials.add(Color::AQUAMARINE.into()),
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
        PbrBundle {
            mesh: meshes.add(
                shape::Capsule {
                    depth: 2.0,
                    radius: 0.6,
                    ..default()
                }
                .into(),
            ),
            material: materials.add(Color::AQUAMARINE.into()),
            transform: Transform::from_xyz(-15.0, 2.0, -65.0),
            ..default()
        },
        Beacon {
            activation_radius: 5.0,
            safe_zone_radius: 10.0,
            heal_factor: 5.0,
        },
        BeaconState::Online,
    ));

    commands.insert_resource(AmbientLight {
        brightness: 1.0,
        color: Color::WHITE,
    });
}

fn go_to_gameover(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::GameOver);
}

fn pause_game(keyboard: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keyboard.just_released(KeyCode::Return) {
        println!("next state!");
        next_state.set(GameState::Paused);
    }
}

fn unpause_game(keyboard: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    //if keyboard.just_released(KeyCode::Return) {
    //    next_state.set(GameState::Playing);
    //}
}

fn main() {
    let mut app = App::new();

    app.add_state::<GameState>();
    app.init_resource::<KillCount>();

    app.add_plugins((
        DefaultPlugins,
        DefaultPickingPlugins
            .build()
            .disable::<DefaultHighlightingPlugin>(),
        PhysicsPlugins::default(),
        //PhysicsDebugPlugin::default(),
        //ComponentsFromGltfPlugin::default(),
        CharacterControllerPlugin,
        FpsCameraPlugin,
        ProjectilePlugin,
        BehaviorPlugin,
        HealthPlugin,
        DoorPlugin,
        BeaconPlugin,
        PortalPlugin,
        GameOverPlugin,
    ));
    app.add_systems(Startup, setup);
    app.add_systems(Update, (load_level,).run_if(in_state(GameState::Loading)));
    app.add_systems(
        Update,
        go_to_gameover
            .run_if(in_state(GameState::Playing))
            .run_if(all_beacons_online),
    );

    app.add_systems(PostUpdate, pause_game.run_if(in_state(GameState::Playing)));
    app.add_systems(Update, unpause_game.run_if(in_state(GameState::Paused)));

    app.add_systems(OnEnter(GameState::GameOver), cleanup::<InGame>);

    #[cfg(not(release))]
    {
        app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
    }

    app.run();
}
