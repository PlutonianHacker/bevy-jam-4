use bevy::{prelude::*, gltf::Gltf};

pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Resource)]
pub struct Levels {
    pub current_level: usize,
    pub data: Vec<Level>,
}

pub struct Level {
    pub name: String,
    pub scene: Handle<Gltf>,
}
