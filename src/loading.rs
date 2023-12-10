use bevy::{gltf::Gltf, prelude::*};

use crate::{
    levels::{Level, Levels},
    GameState,
};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetLoader>().add_systems(
            Update,
            (load_game_assets, get_next_state.run_if(is_done_loading))
                .run_if(in_state(GameState::Loading)),
        );
    }
}

fn is_done_loading(assets: Res<AssetLoader>) -> bool {
    assets.is_done
}

fn get_next_state(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::LoadGame);
}

#[derive(Default, Resource)]
pub struct AssetLoader {
    items: Vec<Handle<Gltf>>,
    levels: Vec<Handle<Gltf>>,
    is_done: bool,
}

fn load_game_assets(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut assets: ResMut<AssetLoader>,
    gltfs: Res<Assets<Gltf>>,
    mut initialized: Local<bool>,
) {
    if !*initialized {
        info!("Begin loading assets!");

        assets.items.push(server.load("models/beacon.glb"));
        assets.items.push(server.load("models/portal.glb"));
        assets.items.push(server.load("models/creep.glb"));

        assets.levels.push(server.load("models/Level_0.glb"));
        assets.levels.push(server.load("models/Level_1.glb"));
        assets.levels.push(server.load("models/Level_2.glb"));
        assets.levels.push(server.load("models/Level_3.glb"));
        assets.levels.push(server.load("models/Level_4.glb"));
        assets.levels.push(server.load("models/The_Lab.glb"));

        let levels = Levels {
            current_level: 0,
            data: vec![
                Level {
                    name: "Level 0".to_string(),
                    scene: server.load("models/Level_0.glb"),
                },
                Level {
                    name: "Level 1".to_string(),
                    scene: server.load("models/Level_1.glb"),
                },
                Level {
                    name: "Level 2".to_string(),
                    scene: server.load("models/Level_2.glb"),
                },
                Level {
                    name: "Level 3".to_string(),
                    scene: server.load("models/Level_3.glb"),
                },
                Level {
                    name: "Level 4".to_string(),
                    scene: server.load("models/Level_4.glb"),
                },
            ],
        };

        commands.insert_resource(levels);

        *initialized = true;
    }

    for item in assets.items.iter() {
        if gltfs.get(item).is_none() {
            return;
        }
    }

    info!("finished loading assets!");

    assets.is_done = true;
}
