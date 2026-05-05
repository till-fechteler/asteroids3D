//! asteroids3D — app entry point.
//! Initializes tracing subscriber + panic-hook-to-file before Bevy startup.
//! Registers DefaultPlugins (minus LogPlugin), GameState, splash flow, and gameplay plugins.

use avian3d::prelude::{Gravity, PhysicsPlugins};
use bevy::prelude::*;

mod arena;
mod combat;
mod flight;
mod logging;
mod pause;
mod splash;
mod state;
mod tuning;
mod ui;
mod visual;

use arena::ArenaPlugin;
use combat::CombatPlugin;
use flight::FlightPlugin;
use logging::init_logging;
use pause::PausePlugin;
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
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(Gravity(Vec3::ZERO))
        .add_plugins(TuningPlugin)
        .add_plugins(VisualPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(ArenaPlugin)
        .add_plugins(FlightPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(PausePlugin)
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
