//! PausePlugin — owns GameState::Paused entry/exit + simulation-clock pause/resume.
//! Triggers: window focus loss (silent) and Escape key (with on-screen overlay).

use avian3d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;
use bevy::window::WindowFocused;

use crate::arena::cleanup_on_exit;
use crate::state::GameState;

pub struct PausePlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PauseSystems {
    Detect,
}

#[derive(Resource, Debug, Clone)]
pub struct PausedFrom(pub GameState);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseInitiator {
    FocusLoss,
    User,
}

#[derive(Component)]
pub struct PauseOverlayEntity;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, PauseSystems::Detect);
        app.add_systems(
            Update,
            (
                pause_on_focus_loss.run_if(in_state(GameState::Arena)),
                resume_on_focus_gain.run_if(in_state(GameState::Paused)),
                toggle_pause_on_escape
                    .run_if(in_state(GameState::Arena).or(in_state(GameState::Paused))),
            )
                .in_set(PauseSystems::Detect),
        );
        app.add_systems(
            OnEnter(GameState::Paused),
            (
                pause_simulation_clocks,
                spawn_pause_overlay_if_user_initiated,
            )
                .chain(),
        );
        app.add_systems(
            OnExit(GameState::Paused),
            (
                cleanup_on_exit::<PauseOverlayEntity>,
                resume_simulation_clocks,
            ),
        );
    }
}

pub fn pause_on_focus_loss(
    mut events: MessageReader<WindowFocused>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in events.read() {
        if !event.focused {
            commands.insert_resource(PausedFrom(GameState::Arena));
            commands.insert_resource(PauseInitiator::FocusLoss);
            next_state.set(GameState::Paused);
            info!("paused on focus loss (window {:?})", event.window);
            return;
        }
    }
}

pub fn resume_on_focus_gain(
    mut events: MessageReader<WindowFocused>,
    paused_from: Option<Res<PausedFrom>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in events.read() {
        if event.focused {
            let target = paused_from
                .as_deref()
                .map_or(GameState::Arena, |p| p.0.clone());
            info!("resumed from focus gain → {:?}", target);
            next_state.set(target);
            return;
        }
    }
}

pub fn toggle_pause_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    paused_from: Option<Res<PausedFrom>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    match current_state.get() {
        GameState::Arena => {
            commands.insert_resource(PausedFrom(GameState::Arena));
            commands.insert_resource(PauseInitiator::User);
            next_state.set(GameState::Paused);
            info!("paused via Escape (initiator: user)");
        }
        GameState::Paused => {
            let target = paused_from
                .as_deref()
                .map_or(GameState::Arena, |p| p.0.clone());
            info!("resumed via Escape → {:?}", target);
            next_state.set(target);
        }
        _ => {}
    }
}

pub fn pause_simulation_clocks(
    mut time_virtual: ResMut<Time<Virtual>>,
    mut time_physics: ResMut<Time<Physics>>,
) {
    time_virtual.pause();
    time_physics.pause();
    info!(
        "simulation clocks paused (virtual.is_paused={}, physics.is_paused={})",
        time_virtual.is_paused(),
        time_physics.is_paused()
    );
}

pub fn resume_simulation_clocks(
    mut time_virtual: ResMut<Time<Virtual>>,
    mut time_physics: ResMut<Time<Physics>>,
) {
    time_virtual.unpause();
    time_physics.unpause();
    info!("simulation clocks resumed");
}

pub fn spawn_pause_overlay_if_user_initiated(
    mut commands: Commands,
    initiator: Option<Res<PauseInitiator>>,
) {
    if initiator.as_deref().copied() != Some(PauseInitiator::User) {
        return;
    }
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        PauseOverlayEntity,
    ));
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            PauseOverlayEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED — Esc to resume"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paused_from_carries_state() {
        let p = PausedFrom(GameState::Arena);
        assert_eq!(p.0, GameState::Arena);
    }

    #[test]
    fn pause_initiator_variants_distinguishable() {
        assert_ne!(PauseInitiator::FocusLoss, PauseInitiator::User);
    }
}
