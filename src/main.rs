//! asteroids3D — app entry point.
//! Initializes tracing subscriber + panic-hook-to-file before Bevy startup.
//! Registers DefaultPlugins (minus LogPlugin), GameState, splash flow, and gameplay plugins.

use bevy::prelude::*;

mod logging;
mod splash;
mod state;
mod tuning;
mod ui;
mod visual;

use logging::init_logging;
use splash::{SplashConfig, cleanup_loading_entities, spawn_splash, tick_splash_timer};
use state::{GameState, log_arena_entered, log_loading_entered, log_mainmenu_entered};
use tuning::TuningPlugin;
use ui::UiPlugin;
use visual::VisualPlugin;

fn main() -> AppExit {
    let log_path = init_logging();
    if let Some(path) = &log_path {
        info!("file logging active at {}", path.display());
    }

    let default_plugins = DefaultPlugins.build().disable::<bevy::log::LogPlugin>();

    App::new()
        .add_plugins(default_plugins)
        .init_state::<GameState>()
        .add_plugins(TuningPlugin)
        .add_plugins(VisualPlugin)
        .add_plugins(UiPlugin)
        .init_resource::<SplashConfig>()
        .add_systems(
            OnEnter(GameState::Loading),
            (log_loading_entered, spawn_splash),
        )
        .add_systems(OnEnter(GameState::MainMenu), log_mainmenu_entered)
        .add_systems(OnEnter(GameState::Arena), log_arena_entered)
        .add_systems(
            Update,
            tick_splash_timer.run_if(in_state(GameState::Loading)),
        )
        .add_systems(OnExit(GameState::Loading), cleanup_loading_entities)
        .run()
}
