//! M1 tech-spike screenshot capture (Story 2.5).
//! Activated by setting ASTEROIDS3D_CAPTURE_PNG=<path> in the environment.
//! When the env var is unset, this plugin's `build()` is never called — the binary
//! behaves byte-identically to the post-2.4 baseline (no extra plugins registered,
//! no extra log lines, no extra systems).
//!
//! Removal note: this entire module + its main.rs registration become dead code
//! once Story 3.1 replaces the reference scene with the Arena state. Capture mode
//! is M1-spike-only; no gameplay code may consume it.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy::window::PrimaryWindow;
use std::path::PathBuf;

pub const CAPTURE_ENV_VAR: &str = "ASTEROIDS3D_CAPTURE_PNG";

/// Returns Some(path) if capture is requested, None otherwise.
/// Uses `var_os` (not `var`) to tolerate non-UTF-8 paths (Windows UNC, etc.).
pub fn requested_capture_path() -> Option<PathBuf> {
    std::env::var_os(CAPTURE_ENV_VAR).map(PathBuf::from)
}

pub struct CapturePlugin {
    pub output_path: PathBuf,
}

#[derive(Resource)]
struct CaptureState {
    output_path: PathBuf,
    frames_observed: u32,
    capture_triggered: bool,
    capture_completed: bool,
}

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CaptureState {
            output_path: self.output_path.clone(),
            frames_observed: 0,
            capture_triggered: false,
            capture_completed: false,
        });
        app.add_systems(
            Update,
            force_skip_splash_in_capture.run_if(in_state(crate::state::GameState::Loading)),
        );
        app.add_observer(on_screenshot_captured);
        app.add_systems(Update, drive_capture);
    }
}

// Deterministic camera transform: reference_scene.rs:51 already spawns Camera3d
// at Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y). Capture
// mode reuses this — no separate "capture transform" exists. AC #1 (Story 2.5).

fn drive_capture(
    mut commands: Commands,
    mut state: ResMut<CaptureState>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if state.capture_completed {
        // One frame after on-disk save observer fires so wgpu's command queue
        // has flushed; then exit.
        app_exit.write(AppExit::Success);
        return;
    }
    state.frames_observed += 1;
    if state.capture_triggered {
        return;
    }
    // 60 frames at ~60Hz = ~1s — enough for tuning.ron asset-load + outline
    // propagator to settle. Splash is bypassed at Startup so we're already
    // post-MainMenu by frame 60.
    const CAPTURE_FRAME: u32 = 60;
    if state.frames_observed < CAPTURE_FRAME {
        return;
    }
    if primary_window.single().is_err() {
        // No primary window yet — defer until next frame.
        return;
    }
    let path = state.output_path.clone();
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
    state.capture_triggered = true;
}

fn on_screenshot_captured(_trigger: On<ScreenshotCaptured>, mut state: ResMut<CaptureState>) {
    state.capture_completed = true;
    info!(
        "Screenshot capture finished; exiting on next frame. Path: {}",
        state.output_path.display()
    );
}

/// Push GameState::MainMenu directly while still in Loading. Skips the 2-second
/// splash without modifying splash.rs (which is not capture-aware) and keeps the
/// splash flow intact for non-capture builds.
///
/// Why direct NextState set instead of the spec's Startup timer-tick bypass:
/// Bevy 0.18's `TimerMode::Once` early-returns from `tick()` once `finished == true`,
/// which means `just_finished()` only returns true on the tick that crossed the
/// duration boundary. A Startup pre-tick past duration sets `finished = true` on
/// frame 0; subsequent `tick_splash_timer` calls in Update see `finished && Once`,
/// hit the early-return path, and `just_finished()` is permanently false — so the
/// `next_state.set(MainMenu)` line in `tick_splash_timer` never fires. Pushing
/// NextState directly is robust against this and timer-duration changes.
fn force_skip_splash_in_capture(mut next_state: ResMut<NextState<crate::state::GameState>>) {
    next_state.set(crate::state::GameState::MainMenu);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_disabled_when_env_var_unset() {
        // SAFETY: cargo test runs tests in parallel by default; env var mutation
        // is process-global. This test snapshots+restores prior value. No other
        // test in this crate reads or writes ASTEROIDS3D_CAPTURE_PNG, so cross-test
        // races cannot occur. If a future test SETS this env var, it MUST gate on
        // serial_test (not currently in deps) or use a child-process harness.
        let prior = std::env::var_os(CAPTURE_ENV_VAR);
        unsafe {
            std::env::remove_var(CAPTURE_ENV_VAR);
        }
        let result = requested_capture_path();
        if let Some(value) = prior {
            unsafe {
                std::env::set_var(CAPTURE_ENV_VAR, value);
            }
        }
        assert!(result.is_none(), "capture must be inert when env var unset");
    }
}
