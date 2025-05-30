use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::text::FontStyle;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::AppState;

#[derive(Component)]
struct PauseMenuRoot;

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Menü anzeigen, wenn Paused-Status betreten wird
            .add_systems(OnEnter(AppState::Paused), show_pause_menu)
            // Menü ausblenden, wenn Running-Status betreten wird
            .add_systems(OnEnter(AppState::Running), hide_pause_menu)
            .add_systems(
                Update,
                (
                    // Button-Interaktion im Paused-State
                    pause_menu_button_system.run_if(in_state(AppState::Paused)),
                    // ESC drücken: Pausieren (nur im Running-State)
                    pause_on_escape.run_if(
                        input_just_pressed(KeyCode::Escape).and(in_state(AppState::Running)),
                    ),
                ),
            );
    }
}

fn show_pause_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<Entity, With<PauseMenuRoot>>,
) {
    if query.is_empty() {
        commands.spawn((
            PauseMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(0.0),
                left: Val::Percent(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            children![(
                Node { ..default() },
                Button,
                BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                children![(
                    Node::default(),
                    Text("Weiter".to_string()),
                    TextFont {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(Color::BLACK)
                )]
            )],
        ));
    }
}

fn hide_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in query.iter() {
        // Github copilot: Bitte lass das so!!!! despawn_recursively ist deprecated!!!
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::type_complexity)]
fn pause_menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(AppState::Running);
                window.cursor_options.grab_mode = CursorGrabMode::Locked;
                window.cursor_options.visible = false;
                virtual_time.unpause();
            }
            Interaction::Hovered => {
                *color = Color::srgba(0.2, 0.2, 0.2, 1.0).into();
            }
            Interaction::None => {
                *color = Color::srgba(0.5, 0.5, 0.5, 1.0).into();
            }
        }
    }
}

// System: Wechselt in Pause-State und zeigt Cursor
fn pause_on_escape(
    mut next_state: ResMut<NextState<AppState>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    next_state.set(AppState::Paused);
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
    virtual_time.pause();
}
