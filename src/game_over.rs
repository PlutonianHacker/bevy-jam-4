use bevy::{app::AppExit, prelude::*};
use bevy_mod_picking::prelude::*;

use crate::{cleanup, levels::Levels, GameState};

#[derive(Debug, Resource, PartialEq, Eq, Copy, Clone)]
pub enum Outcome {
    Won,
    Lost,
}

#[derive(Event)]
pub struct EndGame(pub Outcome);

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EndGame>();

        app.add_systems(OnEnter(GameState::GameOver), spawn_gameover_menu)
            .add_systems(OnExit(GameState::GameOver), cleanup::<Root>);

        app.add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            .add_systems(OnExit(GameState::Paused), cleanup::<Root>);

        app.add_systems(Update, update_button_colors);
    }
}

#[derive(Component)]
pub struct Root;

#[derive(Component)]
pub struct Button;

fn spawn_gameover_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    outcome: Res<Outcome>,
    levels: ResMut<Levels>,
) {
    let wrapper = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    //flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::DARK_GRAY.with_a(0.8).into(),
                ..default()
            },
            Pickable::IGNORE,
            Root,
        ))
        .id();

    let root = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                max_height: Val::Percent(100.0),
                margin: UiRect::axes(Val::Percent(10.0), Val::Px(50.0)),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: Color::DARK_GRAY.into(),
            border_color: Color::BLACK.into(),
            ..default()
        })
        .set_parent(wrapper)
        .id();

    commands
        .spawn((
            TextBundle::from_section(
                "GAME OVER",
                TextStyle {
                    font_size: 72.,
                    color: Color::WHITE,
                    font: asset_server.load("fonts/motion-control.bold.otf"),
                },
            )
            .with_style(Style {
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Px(42.0),
                    bottom: Val::Px(10.0),
                },
                ..default()
            }),
            Pickable::IGNORE,
        ))
        .set_parent(root);

    commands
        .spawn((
            TextBundle::from_section(
                match *outcome {
                    Outcome::Won => "YOU WON!",
                    Outcome::Lost => "YOU LOST!",
                },
                TextStyle {
                    font_size: 36.,
                    color: Color::WHITE,
                    font: asset_server.load("fonts/motion-control.bold.otf"),
                },
            )
            .with_style(Style {
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Px(2.0),
                    bottom: Val::Px(25.0),
                },
                ..default()
            }),
            Pickable::IGNORE,
        ))
        .set_parent(root);

    let outcome = *outcome;

    commands
        .spawn((
            ButtonBundle {
                background_color: Color::rgb_u8(209, 191, 29).into(),
                style: Style {
                    padding: UiRect::all(Val::Px(10.0)),
                    margin: UiRect::axes(Val::Auto, Val::Px(5.0)),
                    max_height: Val::Px(100.0),
                    width: Val::Px(242.0),
                    ..default()
                },
                ..default()
            },
            On::<Pointer<Click>>::run(
                move |mut next_state: ResMut<NextState<GameState>>, mut levels: ResMut<Levels>| {
                    levels.current_level = if levels.current_level == levels.data.len() - 1 {
                        1
                    } else if outcome == Outcome::Lost {
                        levels.current_level
                    } else {
                        levels.current_level + 1
                    };

                    next_state.set(GameState::LoadGame)
                },
            ),
            Button,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    match outcome {
                        Outcome::Won => {
                            if levels.current_level == levels.data.len() - 1 {
                                "PLAY AGAIN"
                            } else {
                                "NEXT LEVEL"
                            }
                        }
                        Outcome::Lost => "TRY AGAIN",
                    },
                    TextStyle {
                        font_size: 42.,
                        color: Color::WHITE,
                        font: asset_server.load("fonts/motion-control.bold.otf"),
                    },
                )
                .with_style(Style {
                    margin: UiRect::horizontal(Val::Auto),
                    ..default()
                }),
                Pickable::IGNORE,
            ));
        })
        .set_parent(root);

    commands
        .spawn((
            ButtonBundle {
                background_color: Color::rgb_u8(209, 191, 29).into(),
                style: Style {
                    padding: UiRect::all(Val::Px(10.0)),
                    margin: UiRect::axes(Val::Auto, Val::Px(5.0)),
                    width: Val::Px(242.0),
                    max_height: Val::Px(100.0),
                    ..default()
                },
                ..default()
            },
            On::<Pointer<Click>>::run(|mut app_exit_events: EventWriter<AppExit>| {
                app_exit_events.send(AppExit);
            }),
            Button,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "QUIT",
                    TextStyle {
                        font_size: 42.,
                        color: Color::WHITE,
                        font: asset_server.load("fonts/motion-control.bold.otf"),
                    },
                )
                .with_style(Style {
                    margin: UiRect::horizontal(Val::Auto),
                    ..default()
                }),
                Pickable::IGNORE,
            ));
        })
        .set_parent(root);
}

fn spawn_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let wrapper = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    //flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::DARK_GRAY.with_a(0.8).into(),
                ..default()
            },
            Pickable::IGNORE,
            Root,
        ))
        .id();

    let root = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                max_height: Val::Percent(100.0),
                margin: UiRect::axes(Val::Percent(100.0), Val::Px(100.0)),
                padding: UiRect::horizontal(Val::Px(50.0)),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: Color::DARK_GRAY.into(),
            border_color: Color::BLACK.into(),
            ..default()
        })
        .set_parent(wrapper)
        .id();

    commands
        .spawn((
            ButtonBundle {
                background_color: Color::rgb_u8(209, 191, 29).into(),
                style: Style {
                    padding: UiRect::all(Val::Px(10.0)),
                    margin: UiRect::axes(Val::Auto, Val::Px(5.0)),
                    max_height: Val::Px(100.0),
                    width: Val::Px(242.0),
                    ..default()
                },
                ..default()
            },
            On::<Pointer<Click>>::run(|mut next_state: ResMut<NextState<GameState>>| {
                next_state.set(GameState::Playing)
            }),
            Button,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "RESUME",
                    TextStyle {
                        font_size: 42.,
                        color: Color::WHITE,
                        font: asset_server.load("fonts/motion-control.bold.otf"),
                    },
                )
                .with_style(Style {
                    margin: UiRect::horizontal(Val::Auto),
                    ..default()
                }),
                Pickable::IGNORE,
            ));
        })
        .set_parent(root);

    commands
        .spawn((
            ButtonBundle {
                background_color: Color::rgb_u8(209, 191, 29).into(),
                style: Style {
                    padding: UiRect::all(Val::Px(10.0)),
                    margin: UiRect::axes(Val::Auto, Val::Px(5.0)),
                    width: Val::Px(242.0),
                    max_height: Val::Px(100.0),
                    ..default()
                },
                ..default()
            },
            On::<Pointer<Click>>::run(|mut app_exit_events: EventWriter<AppExit>| {
                app_exit_events.send(AppExit);
            }),
            Button,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "QUIT",
                    TextStyle {
                        font_size: 42.,
                        color: Color::WHITE,
                        font: asset_server.load("fonts/motion-control.bold.otf"),
                    },
                )
                .with_style(Style {
                    margin: UiRect::horizontal(Val::Auto),
                    ..default()
                }),
                Pickable::IGNORE,
            ));
        })
        .set_parent(root);
}

fn spawn_help_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let wrapper = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    //flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::DARK_GRAY.with_a(0.8).into(),
                ..default()
            },
            Pickable::IGNORE,
            Root,
        ))
        .id();

    let root = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                max_height: Val::Percent(100.0),
                margin: UiRect::axes(Val::Percent(100.0), Val::Px(100.0)),
                padding: UiRect::horizontal(Val::Px(50.0)),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: Color::DARK_GRAY.into(),
            border_color: Color::BLACK.into(),
            ..default()
        })
        .set_parent(wrapper)
        .id();

    commands
        .spawn((
            TextBundle::from_section(
                "YOU WON!",
                TextStyle {
                    font_size: 36.,
                    color: Color::WHITE,
                    font: asset_server.load("fonts/motion-control.bold.otf"),
                },
            )
            .with_style(Style {
                margin: UiRect::left(Val::Px(20.0)),
                ..default()
            }),
            Pickable::IGNORE,
        ))
        .set_parent(root);

    commands
        .spawn((
            ButtonBundle {
                background_color: Color::rgb_u8(209, 191, 29).into(),
                style: Style {
                    padding: UiRect::all(Val::Px(10.0)),
                    margin: UiRect::axes(Val::Auto, Val::Px(5.0)),
                    width: Val::Px(242.0),
                    max_height: Val::Px(100.0),
                    ..default()
                },
                ..default()
            },
            On::<Pointer<Click>>::run(|mut app_exit_events: EventWriter<AppExit>| {
                app_exit_events.send(AppExit);
            }),
            Button,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "QUIT",
                    TextStyle {
                        font_size: 42.,
                        color: Color::WHITE,
                        font: asset_server.load("fonts/motion-control.bold.otf"),
                    },
                )
                .with_style(Style {
                    margin: UiRect::horizontal(Val::Auto),
                    ..default()
                }),
                Pickable::IGNORE,
            ));
        })
        .set_parent(root);
}

fn update_button_colors(
    mut buttons: Query<(Option<&PickingInteraction>, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, mut button_color) in &mut buttons {
        *button_color = match interaction {
            Some(PickingInteraction::Pressed) => Color::rgb(0.35, 0.75, 0.35),
            Some(PickingInteraction::Hovered) => Color::rgb(0.25, 0.25, 0.25),
            Some(PickingInteraction::None) | None => Color::rgb(0.15, 0.15, 0.15),
        }
        .into();
    }
}
