use bevy::prelude::*;
use bevy_xpbd_3d::prelude::Collisions;

use crate::{controller::CharacterController, GameState};

pub struct DoorPlugin;

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, trigger_door.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Door;

#[derive(Component)]
pub struct DoorTrigger(pub Entity);

fn trigger_door(
    mut commands: Commands,
    collisions: Res<Collisions>,
    character: Query<Entity, With<CharacterController>>,
    door_triggers: Query<(Entity, &DoorTrigger,)>,
) {
    let character_entity = character.single();

    for (entity, DoorTrigger(door)) in &door_triggers {
        if let Some(_) = collisions.get(entity, character_entity) {
            println!("Open the door");
            commands.entity(*door).despawn_recursive();
            commands.entity(entity).despawn_recursive();
        }
    }
}
