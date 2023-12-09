use bevy::{ecs::system::SystemId, prelude::*};

use crate::{cleanup, controller::CharacterController, Enemy, GameState, InGame, KillCount};

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateHealth>();
        app.add_systems(OnEnter(GameState::Playing), spawn_player_health);
        app.add_systems(
            Update,
            (update_healths, update_player_health)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );

        //app.add_systems(OnExit(GameState::Playing), cleanup::<>);
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

//pub struct OnDeath(pub SystemId);

#[derive(Event)]
pub struct UpdateHealth(pub Entity, pub f32);

fn update_healths(
    mut commands: Commands,
    mut healths: Query<(&mut Health, Option<&Enemy>, Option<&CharacterController>)>,
    mut health_events: EventReader<UpdateHealth>,
    mut kill_count: ResMut<KillCount>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for UpdateHealth(entity, amount) in health_events.read() {
        let Ok((mut health, maybe_enemy, maybe_player)) = healths.get_mut(*entity) else { 
            continue 
        };

        health.amount = (health.amount + amount).min(health.max);

        if health.amount <= 0.0 {
            commands.entity(*entity).despawn_recursive();

            if maybe_player.is_some() {
                println!("Game Over!");

                next_state.set(GameState::GameOver);

                return;
            } else if maybe_enemy.is_some() {
                kill_count.0 += 1;
            }
        }
    }
}

#[derive(Component)]
pub struct HealthBar;

fn spawn_player_health(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(40.0),
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
            InGame,
        ))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    background_color: Color::rgb_u8(55, 170, 219).into(),
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(20.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    border_color: Color::BLACK.into(),
                    ..default()
                },
                HealthBar,
            ));
        });
}

fn update_player_health(
    mut health_bars: Query<&mut Style, With<HealthBar>>,
    player: Query<&Health, With<CharacterController>>,
) {
    let mut health_bar = health_bars.single_mut();
    let player_health = player.single();

    health_bar.width = Val::Percent((player_health.amount / player_health.max) * 100.0);
}
