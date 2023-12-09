pub mod behavior;
pub mod camera;
pub mod controller;
pub mod door;
pub mod health;
pub mod projectile;
pub mod portal;
pub mod beacon;

pub mod game_over; 

use bevy::prelude::*;

#[derive(Debug, States, Eq, PartialEq, Clone, Default, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Tutorial,
    Playing,
    Paused,
    GameOver,
}

pub enum GameSet {}

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
