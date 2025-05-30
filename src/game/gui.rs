use crate::game::health::Health;
use crate::game::player::Player;
use bevy::prelude::*;
use bevy::text::FontStyle;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_gui)
            .add_systems(Update, update_health_text);
    }
}

#[derive(Component)]
struct HealthText;

fn setup_gui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        children![(
            HealthText,
            Node::default(),
            Text { ..default() },
            TextFont {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::BLACK)
        )],
    ));
}

fn update_health_text(
    player: Single<&Health, With<Player>>,
    mut text: Single<&mut Text, With<HealthText>>,
) {
    text.0 = format!("Leben: {:.0}", player.value);
}
