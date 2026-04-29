//! asteroids3D — app entry point.
//! Initializes tracing subscriber + panic-hook-to-file before Bevy startup.
//! Registers DefaultPlugins (minus LogPlugin), GameState, and the Loading → MainMenu splash flow.
//! Registers TuningPlugin (hot-reloadable gameplay knobs) and VisualPlugin (toon material + reference scene).

use bevy::asset::AssetPlugin;
use bevy::prelude::*;

mod logging;
mod splash;
mod state;
mod tuning;
mod visual;

use logging::init_logging;
use splash::{SplashConfig, cleanup_loading_entities, spawn_splash, tick_splash_timer};
use state::{GameState, log_loading_entered, log_mainmenu_entered};
use tuning::TuningPlugin;
use visual::VisualPlugin;

fn main() -> AppExit {
    let log_path = init_logging();
    if let Some(path) = &log_path {
        info!("file logging active at {}", path.display());
    }
    let capture_path = visual::capture::requested_capture_path();

    let default_plugins = DefaultPlugins
        .build()
        .disable::<bevy::log::LogPlugin>()
        .set(AssetPlugin {
            watch_for_changes_override: cfg!(debug_assertions).then_some(true),
            ..default()
        });
    let default_plugins = if capture_path.is_some() {
        default_plugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: bevy::window::WindowResolution::new(1920, 1080),
                resizable: false,
                decorations: false,
                title: String::from("asteroids3D capture"),
                ..default()
            }),
            ..default()
        })
    } else {
        default_plugins
    };

    let mut app = App::new();
    app.add_plugins(default_plugins)
        .init_state::<GameState>()
        .add_plugins(TuningPlugin)
        .add_plugins(VisualPlugin)
        .init_resource::<SplashConfig>()
        .add_systems(
            OnEnter(GameState::Loading),
            (log_loading_entered, spawn_splash),
        )
        .add_systems(OnEnter(GameState::MainMenu), log_mainmenu_entered)
        .add_systems(
            Update,
            tick_splash_timer.run_if(in_state(GameState::Loading)),
        )
        .add_systems(OnExit(GameState::Loading), cleanup_loading_entities);

    if let Some(path) = capture_path {
        app.add_plugins(visual::capture::CapturePlugin { output_path: path });
    }
    app.run()
}
