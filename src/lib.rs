pub mod behavior;
pub mod camera;
pub mod controller;
pub mod door;
pub mod health;
pub mod projectile;

use bevy::prelude::*;

#[derive(Debug, States, Eq, PartialEq, Clone, Default, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

#[derive(Component)]
pub struct Enemy;
