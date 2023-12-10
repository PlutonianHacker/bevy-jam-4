pub mod beacon;
pub mod behavior;
pub mod camera;
pub mod controller;
pub mod door;
pub mod health;
pub mod levels;
pub mod portal;
pub mod projectile;
pub mod weapon;

pub mod game_over;
pub mod loading;

use bevy::prelude::*;

#[derive(Debug, States, Eq, PartialEq, Clone, Default, Hash)]
pub enum GameState {
    #[default]
    Loading, // load assets
    MainMenu, // main menu, boring stuff.
    LoadGame, // load any extra game related stuff.
    BeginGame, // one frame to set up game stuff
    Playing,
    Paused,
    GameOver,
}

#[derive(Component)]
pub struct Enemy;

#[derive(Resource, Default)]
pub struct KillCount(pub usize);

#[derive(Component)]
pub struct InGame;

pub fn cleanup<T: Component + 'static>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
