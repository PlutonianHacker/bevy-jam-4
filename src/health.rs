use bevy::prelude::*;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateHealth>();
        app.add_systems(Update, update_healths);
    }
}

#[derive(Component)]
pub struct Health {
    pub max: f32,
    pub amount: f32,
}

impl Health {
    pub fn new(amount: f32) -> Self {
        Self {
            max: amount,
            amount,
        }
    }
}

#[derive(Event)]
pub struct UpdateHealth(pub Entity, pub f32);

fn update_healths(
    mut commands: Commands,
    mut healths: Query<&mut Health>,
    mut health_events: EventReader<UpdateHealth>,
) {
    for UpdateHealth(entity, amount) in health_events.read() {
        let mut health = healths.get_mut(*entity).unwrap();

        health.amount = (health.amount + amount).min(health.max);

        if health.amount <= 0.0 {
            commands.entity(*entity).despawn_recursive();
        }
    }
}
