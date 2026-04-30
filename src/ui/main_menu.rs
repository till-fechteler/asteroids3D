//! Title-screen UI for GameState::MainMenu.
//! Press Enter / NumpadEnter to transition to GameState::Arena.

use bevy::prelude::*;

use crate::state::GameState;

const TITLE_TEXT: &str = "asteroids3D";
const SUBTITLE_TEXT: &str = "Press Enter to start";
const TITLE_FONT_SIZE: f32 = 96.0;
const SUBTITLE_FONT_SIZE: f32 = 32.0;
const SUBTITLE_TOP_MARGIN_PX: f32 = 24.0;

#[derive(Component)]
pub struct MainMenuEntity;

pub fn spawn_main_menu(mut commands: Commands) {
    commands.spawn((Camera2d, MainMenuEntity));
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(SUBTITLE_TOP_MARGIN_PX),
                ..default()
            },
            MainMenuEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(TITLE_TEXT),
                TextFont {
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(SUBTITLE_TEXT),
                TextFont {
                    font_size: SUBTITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

pub fn handle_main_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        info!("MainMenu: Enter pressed, transitioning to Arena");
        next_state.set(GameState::Arena);
    }
}

pub fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_font_size_is_at_least_double_subtitle() {
        const { assert!(TITLE_FONT_SIZE >= 2.0 * SUBTITLE_FONT_SIZE) };
    }
}
