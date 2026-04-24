//! Splash screen for GameState::Loading.
//! Timer-driven transition to GameState::MainMenu.

use bevy::prelude::*;

use crate::state::GameState;

const SPLASH_TEXT: &str = "asteriods3D";
const SPLASH_DURATION_SECS: f32 = 2.0;

#[derive(Resource)]
pub struct SplashConfig {
    pub timer: Timer,
}

impl Default for SplashConfig {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(SPLASH_DURATION_SECS, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct LoadingStateEntity;

pub fn spawn_splash(mut commands: Commands) {
    commands.spawn((Camera2d, LoadingStateEntity));
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            LoadingStateEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(SPLASH_TEXT),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn tick_splash_timer(
    time: Res<Time>,
    mut config: ResMut<SplashConfig>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    config.timer.tick(time.delta());
    if config.timer.just_finished() {
        info!("splash timer elapsed, transitioning to MainMenu");
        next_state.set(GameState::MainMenu);
    }
}

pub fn cleanup_loading_entities(
    mut commands: Commands,
    query: Query<Entity, With<LoadingStateEntity>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splash_config_default_is_two_seconds() {
        let config = SplashConfig::default();
        assert_eq!(
            config.timer.duration(),
            std::time::Duration::from_secs_f32(SPLASH_DURATION_SECS)
        );
        assert_eq!(config.timer.mode(), TimerMode::Once);
    }
}
