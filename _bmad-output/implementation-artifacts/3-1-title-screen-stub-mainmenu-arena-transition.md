# Story 3.1: Title Screen Stub — MainMenu → Arena Transition

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player launching the game,
I want a minimal title screen that lets me start a run with a single key press,
So that I reach gameplay from the first Epic-3 commit without dev hacks like default-to-Arena.

## Acceptance Criteria

1. **Given** the app is in `GameState::MainMenu` after Epic 1 Story 1.7's `Loading → MainMenu` transition
   **When** `OnEnter(GameState::MainMenu)` runs
   **Then** a `bevy_ui` text Node hierarchy is spawned with title `"asteroids3D"` plus subtitle `"Press Enter to start"`
   **And** the title and subtitle are visually centered (flexbox `JustifyContent::Center` + `AlignItems::Center` filling the viewport)
   **And** the title uses a larger font size than the subtitle (≥ 2× the subtitle size — sizing fixed in code, no asset font required; Bevy's `default_font` feature already shipped by Story 1.7 covers rendering)
   **And** every entity in the title-screen hierarchy that is a despawn root (`Camera2d` + parent `Node`) carries a `MainMenuEntity` marker component (children inherit cleanup via Bevy 0.18's `ChildOf` linked-despawn — child entities are NOT marked, avoiding the splash cleanup-iteration race documented in `deferred-work.md` lines 71–73)

2. **Given** the title screen is visible (state is `GameState::MainMenu`, MainMenu entities are alive)
   **When** the player presses Enter or the numpad-Enter key (`KeyCode::Enter` or `KeyCode::NumpadEnter`)
   **Then** an input-handling system running in `Update` with `run_if(in_state(GameState::MainMenu))` calls `NextState::set(GameState::Arena)`
   **And** an `info!("MainMenu: Enter pressed, transitioning to Arena")` log line is emitted on the same tick
   **And** the system uses `ButtonInput::just_pressed` (not `pressed`) so that holding Enter does not re-fire the transition every frame

3. **Given** the state transitions `MainMenu → Arena`
   **When** `OnExit(GameState::MainMenu)` runs
   **Then** every `Entity` matching `Query<Entity, With<MainMenuEntity>>` is despawned via `commands.entity(e).despawn()`
   **And** no orphaned title text, subtitle text, parent Node, or `Camera2d` remains in the hierarchy after the next `Update` tick
   **And** `info!("entered Arena")` is emitted from a new `log_arena_entered` system in `state.rs` registered on `OnEnter(GameState::Arena)` — symmetric with `log_loading_entered` and `log_mainmenu_entered`

4. **Given** Story 2.6's M2 Impact section ([`docs/tech-spike/m1-decision.md`](../../docs/tech-spike/m1-decision.md)) and the Story 2.5 deferred-work entry ([`deferred-work.md`](./deferred-work.md) lines 99–102) jointly assign the M1 capture-mode and reference-scene teardown to this story
   **When** Story 3.1 lands
   **Then** `src/visual/capture.rs` is deleted (`git rm`)
   **And** `src/visual/reference_scene.rs` is deleted (`git rm`)
   **And** `src/visual/mod.rs` is edited to remove (a) `pub mod capture;`, (b) `#[cfg(debug_assertions)] mod reference_scene;` plus its `app.add_plugins(reference_scene::ReferenceScenePlugin);` line, and (c) the doc-comment lines that referenced Story 2.1 / 2.5 reference-scene + capture artifacts
   **And** `src/main.rs` is edited to remove the `visual::capture::requested_capture_path()` call, the conditional `WindowPlugin` override on capture, the `capture_path`-gated `CapturePlugin` registration, AND the `bevy::asset::AssetPlugin` import + the explicit `AssetPlugin { watch_for_changes_override: ... }` setter (no remaining consumer needs the override; Bevy's default already enables file_watcher via the `file_watcher` Cargo feature in debug builds)
   **And** `.github/workflows/parity-capture.yml` is deleted (`git rm`)
   **And** `src/visual/palette.rs`'s two `#[cfg_attr(not(debug_assertions), allow(dead_code, ...))]` blocks are converted to unconditional `#[allow(dead_code, reason = "no current consumer; gameplay consumer arrives in Story 4.5 (SemanticAccent wiring on gameplay entities)")]` because the reference scene (the sole debug-build consumer) is now gone — the cfg-conditional release-only allow no longer matches reality
   **And** `cargo build --release` and `cargo build` (debug) both produce **0** lines matching `grep -cE 'warning:|error:'` after these edits
   **And** the `cargo test` count post-3.1 is **14** — the `capture::tests::capture_disabled_when_env_var_unset` test is removed with `capture.rs` (−1) and the new `ui::main_menu::tests::title_font_size_is_at_least_double_subtitle` test is added (+1); net change = 0

5. **Given** the post-Story-2.6 source baseline
   **When** Story 3.1 verification runs
   **Then** the local sweep produces **0** lines matching `grep -cE 'warning:|error:'` for each of: `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build --release`
   **And** `cargo fmt --all -- --check` exits 0
   **And** `cargo run` (no env var) opens a window, transitions through `Loading (~2 s splash) → MainMenu (visible title + subtitle)` without the M1 reference scene appearing, and `info!("entered Arena")` lands in `/tmp/story-3-1-run.log` after the dev presses Enter once
   **And** `cargo test` summary line reads exactly `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
   **And** `git status --short` final set after dev work is exactly: `Cargo.toml` (M, IFF a new file_watcher-gating change happened — see Task 8 dev note; expected `unchanged`), `src/main.rs` (M), `src/state.rs` (M), `src/visual/mod.rs` (M), `src/visual/palette.rs` (M), `src/visual/capture.rs` (D), `src/visual/reference_scene.rs` (D), `src/ui/mod.rs` (??), `src/ui/main_menu.rs` (??), `.github/workflows/parity-capture.yml` (D), plus bookkeeping `_bmad-output/implementation-artifacts/sprint-status.yaml` (M) and this story file (??/M). NO entries under `Cargo.lock`, `.gitignore`, `assets/`, `docs/`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`.

## Tasks / Subtasks

- [x] **Task 1: Author `src/ui/mod.rs` with `UiPlugin` skeleton** (AC: #1, #2, #3)
  - [x] Create `src/ui/` directory at the repo root.
  - [x] Create `src/ui/mod.rs`. ≤ 30 lines. Module doc 2 lines max, no story-id references (per Story 1.5 review patch BH8 + Story 1.7 dev-notes precedent).
  - [x] `UiPlugin` is a unit struct implementing `Plugin`. Body registers the `main_menu` submodule's `OnEnter(MainMenu)` spawn system, an Update input system (`run_if(in_state(MainMenu))`), and an `OnExit(MainMenu)` cleanup system. No `SystemSet` enum is introduced for `UiPlugin` in this story — the FR36 stub has only one logical phase per state-edge. A `UiSystems` enum can be added by Epic 4's full title-screen story (4-7) if cross-system ordering inside UI becomes load-bearing.
  - [x] Skeleton:
    ```rust
    //! UiPlugin — bevy_ui screen-space UI surfaces (Story 3.1: title-screen stub).
    //! Future home for HUD (Story 3.11), pause overlay (Story 3.4), settings (Story 4.8).

    use bevy::prelude::*;

    pub mod main_menu;

    pub struct UiPlugin;

    impl Plugin for UiPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(
                OnEnter(crate::state::GameState::MainMenu),
                main_menu::spawn_main_menu,
            )
            .add_systems(
                Update,
                main_menu::handle_main_menu_input
                    .run_if(in_state(crate::state::GameState::MainMenu)),
            )
            .add_systems(
                OnExit(crate::state::GameState::MainMenu),
                main_menu::cleanup_main_menu,
            );
        }
    }
    ```
  - [x] Rationale for `pub mod main_menu;` rather than inlining: future stories (3.4 pause overlay, 3.11 HUD baseline, 4.7 full title screen, 4.8 settings, 4.9 post-run summary) all add per-feature `*.rs` siblings; the `mod.rs` stays an orchestrator. This matches `architecture.md:589-598` UI subtree layout exactly.

- [x] **Task 2: Author `src/ui/main_menu.rs` — title-screen UI + input + cleanup** (AC: #1, #2, #3)
  - [x] Create `src/ui/main_menu.rs`. Target 80–100 lines including 1 unit test + module doc.
  - [x] Module doc 2 lines max, no story-id references.
  - [x] Constants:
    ```rust
    const TITLE_TEXT: &str = "asteroids3D";
    const SUBTITLE_TEXT: &str = "Press Enter to start";
    const TITLE_FONT_SIZE: f32 = 96.0;
    const SUBTITLE_FONT_SIZE: f32 = 32.0;
    const SUBTITLE_TOP_MARGIN_PX: f32 = 24.0;
    ```
    Constants live at module scope (matches `splash.rs` precedent); they are not tunable via `tuning.ron` because Story 3.1 is a stub and the full FR36 title screen (Epic 4 / Story 4.7) rewrites this entirely.
  - [x] Marker component:
    ```rust
    #[derive(Component)]
    pub struct MainMenuEntity;
    ```
    `pub` because `cleanup_main_menu` queries it. The component has no fields — pure tag.
  - [x] `spawn_main_menu` system signature:
    ```rust
    pub fn spawn_main_menu(mut commands: Commands) { ... }
    ```
    Spawns:
    1. `(Camera2d, MainMenuEntity)` — camera order defaults to `0`. No `order:` override needed; the splash `Camera2d` (also order 0) was despawned by `cleanup_loading_entities` on `OnExit(Loading)`, so there is no camera collision.
    2. Parent root Node — `Node { width: 100%, height: 100%, justify_content: Center, align_items: Center, flex_direction: Column, row_gap: Val::Px(SUBTITLE_TOP_MARGIN_PX), ..default() }`, tagged `MainMenuEntity`.
    3. Inside `.with_children(|parent| ...)`: two `Text`-component children, **NOT marked** with `MainMenuEntity` (rely on Bevy 0.18 `ChildOf` linked-despawn cascade). The two children are:
       - title: `(Text::new(TITLE_TEXT), TextFont { font_size: TITLE_FONT_SIZE, ..default() }, TextColor(Color::WHITE))`
       - subtitle: `(Text::new(SUBTITLE_TEXT), TextFont { font_size: SUBTITLE_FONT_SIZE, ..default() }, TextColor(Color::srgb(0.7, 0.7, 0.7)))` (slightly dimmer than title for visual hierarchy)
    4. **Why no child marker:** the `LoadingStateEntity` marker on the splash child Text (Story 1.7 review patch F2) is what produces the `Entity despawned … invalid` WARN observed during Story 2.1 dev-verification ([`deferred-work.md`](./deferred-work.md) lines 71–73). The deferred-work entry's option (b) — "remove the now-redundant marker" — is the exact pattern adopted here for `main_menu.rs`. Marking only despawn-roots is the canonical Bevy 0.18 approach.
  - [x] `handle_main_menu_input` system signature:
    ```rust
    pub fn handle_main_menu_input(
        keys: Res<ButtonInput<KeyCode>>,
        mut next_state: ResMut<NextState<crate::state::GameState>>,
    ) {
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
            info!("MainMenu: Enter pressed, transitioning to Arena");
            next_state.set(crate::state::GameState::Arena);
        }
    }
    ```
    `just_pressed` (NOT `pressed`) so holding Enter is a single transition event. Both `KeyCode::Enter` and `KeyCode::NumpadEnter` covered for keyboard-layout robustness (matches Bevy convention; full keyboard rebinding is leafwing-input-manager territory in Story 3.6 onward).
  - [x] `cleanup_main_menu` system signature:
    ```rust
    pub fn cleanup_main_menu(
        mut commands: Commands,
        query: Query<Entity, With<MainMenuEntity>>,
    ) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }
    ```
    Mirror of `cleanup_loading_entities` in `src/splash.rs` — same despawn pattern. Children cascade via Bevy 0.18 linked-despawn since they have no marker (cleanest path; sidesteps the splash race entirely).
  - [x] Unit test (one):
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn title_font_size_is_at_least_double_subtitle() {
            assert!(
                TITLE_FONT_SIZE >= 2.0 * SUBTITLE_FONT_SIZE,
                "AC #1 hierarchy guarantee: title must be ≥2× subtitle"
            );
        }
    }
    ```
    Rationale: AC #1 requires "the title uses a larger font size than the subtitle (≥ 2×)". A static-constant assertion is the cheapest way to make that an automatic-regression target. No App construction needed; CI-safe.

- [x] **Task 3: Add `log_arena_entered` to `src/state.rs`** (AC: #3)
  - [x] Append after `log_mainmenu_entered`:
    ```rust
    pub fn log_arena_entered() {
        info!("entered Arena");
    }
    ```
  - [x] **Do NOT** touch the `GameState` enum or the `#[expect(dead_code, reason = "...")]` attribute. With `Arena` becoming live in this story, and `MainMenu` already live since 1.7, the dead-code expectation is still valid — `Caravan`, `PostRun`, `PhotoMode`, `Paused` remain unused. (When Story 3.4 lands `Paused`, the rustc lint expectation will start to drift; either Story 3.4 or a planning-sweep story removes the `#[expect]` then. Not Story 3.1 scope.)
  - [x] Net delta to `state.rs`: +3 lines.

- [x] **Task 4: Wire UiPlugin + `log_arena_entered` into `src/main.rs` AND remove M1 capture wiring** (AC: #2, #3, #4)
  - [x] Add `mod ui;` after `mod tuning;`. Add `use ui::UiPlugin;` after `use tuning::TuningPlugin;`. Rustfmt may reorder these alphabetically inside their respective groups — accept its order.
  - [x] Add `use state::{GameState, log_arena_entered, log_loading_entered, log_mainmenu_entered};` (rustfmt will sort alphabetically; current order is `GameState, log_loading_entered, log_mainmenu_entered`).
  - [x] Add `.add_plugins(UiPlugin)` after the existing `.add_plugins(VisualPlugin)`.
  - [x] Add `.add_systems(OnEnter(GameState::Arena), log_arena_entered)` after the existing `OnEnter(MainMenu)` registration.
  - [x] **Remove** the entire capture-mode block from `main.rs`:
    - Delete `let capture_path = visual::capture::requested_capture_path();`
    - Delete `let default_plugins = if capture_path.is_some() { ... } else { default_plugins };` reassignment (the conditional WindowPlugin override is M1-spike-only).
    - Delete `if let Some(path) = capture_path { app.add_plugins(visual::capture::CapturePlugin { output_path: path }); }` block.
    - Delete `use bevy::asset::AssetPlugin;` import IF the resulting `default_plugins.build().disable::<...>().set(AssetPlugin { ... })` chain is also being simplified. **Decision:** keep the `disable::<bevy::log::LogPlugin>()` (Story 1.8's logging override depends on it) but DROP the `set(AssetPlugin { watch_for_changes_override: cfg!(debug_assertions).then_some(true) })` setter — the `file_watcher` Cargo feature already enables hot-reload in debug builds (`Cargo.toml:8` lists `file_watcher` in the bevy features). The override is now redundant. With the setter gone, the `AssetPlugin` import is also unused and can be deleted.
    - **NOTE:** if removing the `AssetPlugin` setter breaks `tuning.ron` hot-reload in Story 2.3's manual smoke (see Task 8 verification), restore the setter and keep the `AssetPlugin` import. The Cargo `file_watcher` feature *should* be sufficient by itself per Bevy 0.18 docs, but empirical verification is authoritative.
  - [x] Resulting `main.rs` body (post-edit, modulo rustfmt):
    ```rust
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
    ```
  - [x] Net delta to `main.rs`: removes ~20 lines (capture wiring + AssetPlugin import + WindowPlugin conditional + AssetPlugin setter), adds ~5 lines (`mod ui;`, `use ui::UiPlugin;`, `add_plugins(UiPlugin)`, `add_systems(OnEnter(Arena), log_arena_entered)`, `log_arena_entered` import). File shrinks from 71 to ~50 lines.

- [x] **Task 5: Delete `src/visual/capture.rs` + edit `src/visual/mod.rs` to drop capture module declaration** (AC: #4)
  - [x] `git rm src/visual/capture.rs`. The 138-line file has 1 unit test (`capture_disabled_when_env_var_unset`) which goes with the file — `cargo test` count drops from 14 to 13.
  - [x] Edit `src/visual/mod.rs`:
    - Remove `pub mod capture;` (line 10).
    - Remove the doc-comment line `//! Story 2.5 adds opt-in screenshot capture (...)` (line 6).
  - [x] Verify post-edit: `grep -rn 'capture' src/` returns ZERO hits. `grep -rn 'CapturePlugin\|ASTEROIDS3D_CAPTURE_PNG\|requested_capture_path' src/` returns ZERO hits.

- [x] **Task 6: Delete `src/visual/reference_scene.rs` + edit `src/visual/mod.rs` to drop reference-scene plugin** (AC: #4)
  - [x] `git rm src/visual/reference_scene.rs`. The 236-line file is debug-only (`#[cfg(debug_assertions)] mod reference_scene;` in `mod.rs:60`), no tests, was the M1 dev scaffold for vector-aesthetic verification.
  - [x] Edit `src/visual/mod.rs`:
    - Remove the entire `#[cfg(debug_assertions)] app.add_plugins(reference_scene::ReferenceScenePlugin);` block in `VisualPlugin::build` (lines 41–42).
    - Remove the `#[cfg(debug_assertions)] mod reference_scene;` declaration (lines 59–60).
    - Remove the doc-comment lines that referenced reference-scene work — keep the FR49/FR50/Story 2.3/2.4 lines (still relevant), drop the Story 2.1 (reference scene) line.
  - [x] Verify post-edit: `grep -rn 'reference_scene\|ReferenceScene' src/` returns ZERO hits.
  - [x] Resulting `src/visual/mod.rs` (post-edit, ~30 lines, doc-comment trimmed):
    ```rust
    //! Visual presentation plugin: toon shader, outlines, palette.
    //! Story 2.3 adds the WGSL `ToonMaterial` (FR49) wired through `MaterialPlugin`.
    //! Story 2.4 adds `bevy_mod_outline::OutlinePlugin` wiring + outline hot-reload propagation (FR49).

    use bevy::prelude::*;

    pub mod outline;
    pub mod palette;
    pub mod toon_material;

    pub struct VisualPlugin;

    #[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum VisualSystems {
        Setup,
    }

    impl Plugin for VisualPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins(MaterialPlugin::<toon_material::ToonMaterial>::default());
            app.add_plugins(bevy_mod_outline::OutlinePlugin);

            app.configure_sets(
                OnEnter(crate::state::GameState::Loading),
                VisualSystems::Setup,
            );

            app.add_systems(
                Update,
                (
                    apply_tuning_to_toon_materials,
                    outline::apply_tuning_to_outlines,
                )
                    .in_set(crate::tuning::TuningSystems::Reload),
            );
        }
    }

    fn apply_tuning_to_toon_materials(
        mut events: MessageReader<crate::tuning::TuningReloaded>,
        mut materials: ResMut<Assets<toon_material::ToonMaterial>>,
    ) {
        for event in events.read() {
            for (_, material) in materials.iter_mut() {
                material.steps = event.0.toon_steps;
                material.rim_power = event.0.toon_rim_power;
                material.rim_intensity = event.0.toon_rim_intensity;
            }
        }
    }
    ```
  - [x] **Note on `VisualSystems::Setup` SystemSet:** the set is currently configured on `OnEnter(Loading)` but no system lives in it post-cleanup (the only consumer was `spawn_reference_scene`). The set is preserved for Story 3.3's `spawn_arena_zone` system (epic spec at [`epics/epic-3-...md:65-66`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md) — `spawn_arena_zone` runs on `OnEnter(GameState::Arena)`). **However**, Story 3.3 is on `OnEnter(Arena)`, not `OnEnter(Loading)`, so the existing `configure_sets(OnEnter(Loading), VisualSystems::Setup)` is mismatched for that future use. **Decision:** leave `VisualSystems::Setup` configured on `OnEnter(Loading)` for now (no behavioural drift in Story 3.1 — empty set is a no-op), and Story 3.3 will reconfigure to `OnEnter(Arena)` when it actually uses the set. Documenting the mismatch here so 3.3's author doesn't read the unused set as load-bearing for `Loading`.

- [x] **Task 7: Update `src/visual/palette.rs` cfg_attr → unconditional `allow(dead_code)`** (AC: #4)
  - [x] With the reference scene removed, `SemanticAccent` and `color_for` have NO consumer in either debug or release builds. The current `#[cfg_attr(not(debug_assertions), allow(dead_code, reason = "..."))]` blocks (palette.rs:7-13 and palette.rs:23-29) only suppress the warning in release; debug builds will now emit `dead_code` warnings.
  - [x] Replace both `#[cfg_attr(not(debug_assertions), allow(dead_code, reason = "..."))]` attributes with unconditional:
    ```rust
    #[allow(dead_code, reason = "no current consumer; gameplay consumer arrives in Story 4.5 (SemanticAccent wiring on gameplay entities)")]
    ```
    Apply to both the `pub enum SemanticAccent { ... }` block AND the `pub fn color_for(...) -> Color` function.
  - [x] **Reason text update rationale:** the prior `cfg_attr` reason was `"reference_scene (debug-only consumer) is cfg-gated; release consumer arrives in Story 4.5"`. With the reference scene gone, that reason is stale (no debug-only consumer either). New reason names Story 4.5 directly as the consumer.
  - [x] Verify post-edit: `cargo build` (debug) → 0 warnings; `cargo build --release` → 0 warnings.
  - [x] Update the deferred-work entry at [`deferred-work.md` (Story 2.2 cfg_attr block, around lines 94–96)](./deferred-work.md): no edit needed in this story — Story 4.5 (the rescheduled consumer landing) will perform the cfg_attr-removal verification per its existing deferred-work assignment. (Leaving the deferred-work entry unchanged maintains the single source of truth: the cfg_attr blocks ARE in the code; Story 4.5 removes them when it adds release-path consumers.)

- [x] **Task 8: Delete `.github/workflows/parity-capture.yml`** (AC: #4)
  - [x] `git rm .github/workflows/parity-capture.yml`. The 133-line workflow is M1-spike-only (manual `workflow_dispatch` trigger; not on push/PR). Last successful runs: `25111626273` and `25113263165` (Story 2.5 baseline).
  - [x] **Removal validity check:** the workflow's three jobs (`linux-vulkan`, `macos-metal`, `windows-dx12`) all spawn the binary with `ASTEROIDS3D_CAPTURE_PNG=...` set. With the binary's capture-handling code gone (Task 4), even if the workflow were dispatched, the env var would be ignored and the screenshot step would fail. Removing the workflow file before that would-be-broken state matters for repo hygiene + future maintainer cognitive load.
  - [x] **Audit-trail preservation:** the M1 evidence (`docs/tech-spike/m1-backends/{metal,vulkan,dx12}.png` + `parity-report.md` + diff heatmaps + `m1-decision.md`) STAYS — those files are committed and not part of this Task. They are the auditable evidence the M1 gate was satisfied.
  - [x] **Re-introduction at M4 Bevy bump:** the `m1-decision.md` Risks Accepted #3 ("Bevy version-bump risk") notes that the parity-capture workflow can be re-introduced in a 1-day chore at the M4 bump window if cross-backend re-validation is wanted. Cross-link tracked in `deferred-work.md` (Story 2.6 entry).

- [x] **Task 9: Local verification sweep — full build + runtime smoke** (AC: #5)
  - [x] **`cargo check`:**
    ```bash
    cargo check 2>&1 | tee /tmp/story-3-1-check.log
    grep -cE 'warning:|error:' /tmp/story-3-1-check.log
    ```
    Expected: `0`. If non-zero, the most likely culprit is a missing `pub` on `MainMenuEntity` or a wrong `crate::` path in `src/ui/mod.rs`. Read the error verbatim and patch.
  - [x] **`cargo build` (debug):**
    ```bash
    cargo build 2>&1 | tee /tmp/story-3-1-build.log
    grep -cE 'warning:|error:' /tmp/story-3-1-build.log
    ```
    Expected: `0`. **Particular vigilance** for `dead_code` on `SemanticAccent` / `color_for` — if Task 7's cfg_attr-to-allow conversion was missed, this is where it surfaces.
  - [x] **`cargo test`:**
    ```bash
    cargo test 2>&1 | tee /tmp/story-3-1-test.log
    grep -cE 'warning:|error:|FAILED' /tmp/story-3-1-test.log
    ```
    Expected: `0`. The summary line MUST read exactly `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Pre-3.1 count is **14** (post-2.6 baseline). Story 3.1 removes 1 test (`capture::tests::capture_disabled_when_env_var_unset`, gone with `capture.rs`) and adds 1 test (`ui::main_menu::tests::title_font_size_is_at_least_double_subtitle`), so net change = 0 → post-3.1 count = **14**.
  - [x] **`cargo clippy --all-targets -- -D warnings`:**
    ```bash
    cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-3-1-clippy.log
    grep -cE 'warning:|error:' /tmp/story-3-1-clippy.log
    ```
    Expected: `0`.
  - [x] **`cargo fmt --all -- --check`:**
    ```bash
    cargo fmt --all -- --check
    echo $?
    ```
    Expected exit: `0`. If non-zero, run `cargo fmt --all` once and re-check (rustfmt will canonicalize the `use` ordering in `src/main.rs` and `src/ui/mod.rs`; accept its order without further edits).
  - [x] **`cargo build --release`:**
    ```bash
    cargo build --release 2>&1 | tee /tmp/story-3-1-release.log
    grep -cE 'warning:|error:' /tmp/story-3-1-release.log
    ```
    Expected: `0`. **Particularly important** for catching dead_code regressions in palette.rs after the cfg_attr conversion (Task 7).
  - [x] **`cargo run` runtime smoke (foreground or short-running background):**
    ```bash
    RUST_LOG=info,wgpu=warn,naga=warn cargo run 2>&1 | tee /tmp/story-3-1-run.log &
    PID=$!
    # wait ~3 s for splash (2 s) + MainMenu paint
    sleep 3
    # send Enter via xdotool / AppleScript / leave to manual press
    # ... after ~5–8 s total, close window manually or send SIGINT
    ```
    The background launch + manual Enter is acceptable; alternative is a foreground run with manual Enter press. The runtime sweep MUST capture all four log signals below. *(If running on macOS, a manual Enter on the window in focus is the simplest path.)*
  - [x] **Log-grep evidence for runtime smoke:**
    ```bash
    grep -c 'entered Loading' /tmp/story-3-1-run.log     # expected: 1
    grep -c 'splash timer elapsed' /tmp/story-3-1-run.log  # expected: 1
    grep -c 'entered MainMenu' /tmp/story-3-1-run.log    # expected: 1
    grep -c 'MainMenu: Enter pressed, transitioning to Arena' /tmp/story-3-1-run.log  # expected: 1 (after Enter press)
    grep -c 'entered Arena' /tmp/story-3-1-run.log       # expected: 1 (same tick or +1 frame)
    grep -cE 'panic|backtrace|FATAL' /tmp/story-3-1-run.log  # expected: 0
    grep -E 'AdapterInfo|backend:' /tmp/story-3-1-run.log    # expected: backend: Metal on Apple M5 Pro
    ```
    Each of the first 5 expected counts MUST be 1 (or up to 1 if Enter is pressed multiple times — but `just_pressed` fires once per press, so over-counting indicates user pressed twice).
  - [x] **Visual verification (manual):** the MainMenu screen shows "asteroids3D" centered with "Press Enter to start" below it, no M1 reference scene meshes, no swatch UI. After pressing Enter, the screen goes blank (Arena state has no rendering yet — that's Story 3.3). Window stays open until manually closed.
  - [x] **Tuning hot-reload non-regression check:** run `cargo run` in debug, then in another terminal edit `assets/config/tuning.ron` (e.g., bump `toon_steps` from 4 to 6), save, and observe `/tmp/story-3-1-run.log` for `TuningReloaded: toon_steps=6 ...`. Expected: 1 hit, confirming hot-reload still works without the explicit `AssetPlugin { watch_for_changes_override: ... }` setter (relying on the `file_watcher` Cargo feature alone). **If no `TuningReloaded` event is logged within 2 s of the file save**, restore the `set(AssetPlugin { ... })` and `use bevy::asset::AssetPlugin;` in `main.rs` and re-verify. Document the empirical outcome in Dev Agent Record. *(This check is the ONLY load-bearing regression risk from Task 4's `AssetPlugin` setter removal; it's worth the 2-minute manual confirmation.)*

- [x] **Task 10: Scope guardrails — verify nothing else drifted** (AC: #5)
  - [x] `git status --short` final inspection. Expected file set:
    - `src/main.rs` (M) — Task 4.
    - `src/state.rs` (M) — Task 3.
    - `src/visual/mod.rs` (M) — Tasks 5, 6.
    - `src/visual/palette.rs` (M) — Task 7.
    - `src/visual/capture.rs` (D) — Task 5.
    - `src/visual/reference_scene.rs` (D) — Task 6.
    - `src/ui/mod.rs` (??) — Task 1.
    - `src/ui/main_menu.rs` (??) — Task 2.
    - `.github/workflows/parity-capture.yml` (D) — Task 8.
    - `_bmad-output/implementation-artifacts/sprint-status.yaml` (M) — Task 12.
    - `_bmad-output/implementation-artifacts/3-1-title-screen-stub-mainmenu-arena-transition.md` (M) — Task 12.
    - **NO** `Cargo.toml` (M), `Cargo.lock` (M), `.gitignore` (M), `.github/workflows/ci.yml` (M), `assets/**` (M), `docs/**` (M), `rust-toolchain.toml` (M), `rustfmt.toml` (M), `clippy.toml` (M).
  - [x] **Conditional Cargo.toml exception:** if Task 9's tuning hot-reload non-regression check fails AND restoration of the `AssetPlugin` setter fixes it, then `Cargo.toml` is unchanged but `src/main.rs` retains the `AssetPlugin` import + setter. If the check ALSO requires bumping/changing the bevy `file_watcher` feature, then `Cargo.toml` (M) is acceptable IFF documented in Dev Agent Record with empirical justification. Default expectation: `Cargo.toml` unchanged.
  - [x] `grep -rn 'capture\|CapturePlugin\|reference_scene\|ReferenceScene\|ASTEROIDS3D_CAPTURE_PNG' src/ --include='*.rs'` → expected: **0 hits**. (All capture and reference-scene references gone from source.)
  - [x] `grep -rn 'MainMenuEntity\|UiPlugin\|spawn_main_menu\|cleanup_main_menu\|handle_main_menu_input' src/ --include='*.rs'` → expected: **6+ hits** (definitions + uses across `src/ui/mod.rs`, `src/ui/main_menu.rs`, `src/main.rs`).
  - [x] `grep -rn 'GameState::Arena' src/ --include='*.rs'` → expected: **2 hits** (the `next_state.set(GameState::Arena)` call in `main_menu.rs` + the `OnEnter(GameState::Arena)` registration in `main.rs`).
  - [x] **Files NOT touched (must remain byte-identical):** `Cargo.toml` (modulo Task 9 conditional), `Cargo.lock`, `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `assets/**`, `docs/**` (including `docs/tech-spike/m1-backends/**` — audit-trail evidence stays), all `_bmad-output/planning-artifacts/**`.

- [x] **Task 11: Update `deferred-work.md` — acknowledge cleanup completion** (AC: #4)
  - [x] Edit `_bmad-output/implementation-artifacts/deferred-work.md`:
    - Append a `> **✅ RESOLVED 2026-XX-XX by Story 3.1**` annotation to the Story 2.5 deferral block (currently at lines 99–102, the entry titled "Story 3.1 cleanup: remove M1-spike capture mode"). Format matches the precedent at line 19 (Story 1.3 typo-sweep resolution annotation) and line 56 (Story 1.5 → 1.6 AppExit resolution).
    - Append a `> **✅ RESOLVED 2026-XX-XX by Story 3.1**` annotation to the Story 2.1 deferral block (currently at lines 67–69, "spawn_reference_scene is not idempotent on OnEnter(Loading) re-entry") — Story 3.1 removes `reference_scene.rs` entirely, which makes the idempotency concern moot.
    - Append a `> **✅ RESOLVED 2026-XX-XX by Story 3.1**` annotation to the Story 2.2 deferral block on swatch-UI cleanup (currently at line 87, "No despawn-on-state-exit for swatch UI") — Story 3.1 removes `reference_scene.rs` (the source of swatches), so the leak hazard is gone.
    - Append a `> **✅ RESOLVED 2026-XX-XX by Story 3.1**` annotation to the Story 2.2 deferral block on swatch over-tagging (line 88) — same reason.
    - **Do NOT** annotate the Story 2.6 deferral block on Story 2.7 disposition (line 112) — that entry is a definitive "not-needed" assertion, not a deferral that 3.1 resolves.
    - **Do NOT** annotate the splash cleanup-iteration race entry (lines 71–73) — Story 3.1 does NOT touch `src/splash.rs`. The race remains deferred. Story 3.1's `cleanup_main_menu` system AVOIDS the race by tagging only despawn roots (the deferred-work entry's option (b)), but does not retroactively patch `src/splash.rs`.
    - **Do NOT** annotate the Story 2.5 capture-mode contingency entries (lines 101–102, "Contingency: WGPU_ADAPTER_NAME override" and "Contingency: re-evaluate VK_ICD_FILENAMES") — those become moot in the same removal but are conditional contingencies that never triggered, so a "RESOLVED" annotation would over-claim. They naturally die with the workflow file.
  - [x] Replace `2026-XX-XX` with the actual dev-story execution date.
  - [x] **Append a new section** at the bottom of `deferred-work.md`:
    ```markdown
    ## Deferred from: 3-1-title-screen-stub-mainmenu-arena-transition (YYYY-MM-DD)

    - **Splash cleanup-iteration race remains** — `src/splash.rs:67-73` (the `Entity despawned … invalid` WARN observed during Story 2.1 dev-verification). Story 3.1 did NOT touch `src/splash.rs`; the race entry at `deferred-work.md:71-73` stays open and re-deferred. Story 3.1's new `cleanup_main_menu` system AVOIDS the race by tagging only despawn roots (parent Node + Camera2d, not child Text), so the new code is race-free; the splash code is unchanged. **Resolution path:** the next story that touches `src/splash.rs` (likely Story 4.7 full title screen, or a dedicated splash-refactor chore) should apply option (b) from the deferred-work entry — remove the `LoadingStateEntity` marker from the child Text spawn at `src/splash.rs:42-50`, mirroring the main-menu pattern Story 3.1 establishes.
    - **`src/splash.rs` location vs. `architecture.md:589-597` UI subtree** — `src/splash.rs` is still flat at `src/`, not `src/ui/splash.rs` as architecture mandates. Story 1.7 deliberately deferred the move to "when the UiPlugin skeleton lands"; Story 3.1 lands UiPlugin but does NOT move splash. **Resolution path:** mechanical move (`git mv src/splash.rs src/ui/splash.rs` + update import in `src/main.rs` + add `pub mod splash;` to `src/ui/mod.rs`) at any future story that touches `src/splash.rs` OR a dedicated 1-task chore story. Bundling the move with Story 3.1 was rejected to keep this story's surface focused on the FR36 stub + M1 cleanup; the splash file's location is purely organizational and has no functional impact.
    - **`VisualSystems::Setup` SystemSet configured on `OnEnter(Loading)` is now empty** — `src/visual/mod.rs` (post-3.1). The set was introduced by Story 2.1 to host `spawn_reference_scene`; with reference_scene gone, the set is configured on `OnEnter(Loading)` but contains no systems. **Resolution path:** Story 3.3 (Hand-Designed Arena Zone with Static Asteroid Field) is the next consumer — its `spawn_arena_zone` system on `OnEnter(GameState::Arena)` should reuse `VisualSystems::Setup` AND reconfigure the set to `OnEnter(Arena)` instead of `OnEnter(Loading)`. Until 3.3 lands, the empty set is a no-op. Flagged here so 3.3's author doesn't read the existing `OnEnter(Loading)` configuration as load-bearing.
    ```
  - [x] Replace `YYYY-MM-DD` with the dev-story execution date.

- [x] **Task 12: Bookkeeping commit + CI observation** (AC: all)
  - [x] Populate this story file's **Dev Agent Record**: Agent Model Used, Debug Log References (per-command grep counts + log paths), Completion Notes (per-AC evidence + any deviations), File List (added / modified / deleted).
  - [x] Set this story's `Status:` header → `review`.
  - [x] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - Flip `epic-3: backlog` → `epic-3: in-progress` (3.1 is the first story in Epic 3 — matches the precedent at `epic-1` line 47 and `epic-2` line 59 transitions).
    - Flip `3-1-title-screen-stub-mainmenu-arena-transition: backlog` → `3-1-title-screen-stub-mainmenu-arena-transition: review` (the dev-story flips through `ready-for-dev → in-progress → review`; final state at handoff is `review`; the post-code-review bookkeeping commit will flip `review → done`).
    - Bump `last_updated:` (both top-comment line and YAML key) to: `last_updated: YYYY-MM-DD (Story 3.1 ready-for-dev → review — title screen stub, M1 capture+reference-scene teardown; epic-3 → in-progress)`.
    - YAML parse verification: `python3 -c "import yaml; yaml.safe_load(open('_bmad-output/implementation-artifacts/sprint-status.yaml')); print('OK')"` → expected `OK`. Falls back to Ruby (`ruby -ryaml -e "YAML.load_file(...)"`) if PyYAML unavailable, per Story 2.6 precedent.
  - [x] **Commit 1 (source — triggers CI):** stage `src/main.rs`, `src/state.rs`, `src/visual/mod.rs`, `src/visual/palette.rs`, the deletions of `src/visual/capture.rs` and `src/visual/reference_scene.rs`, `src/ui/mod.rs`, `src/ui/main_menu.rs`, and the deletion of `.github/workflows/parity-capture.yml`. **NO** `_bmad-output/**` files in this commit.
    - HEREDOC commit message subject: `feat: title-screen stub + M1 capture/reference-scene teardown (Story 3.1)`. Single-line, target ≤ 70 chars (literal length: 73 — *acceptable margin* per Till's precedent of 60–80; if rejected by hooks, shorten to `feat: title-screen stub + M1 teardown (Story 3.1)`).
    - Push to `origin/master`. Triggers full 4-job `ci.yml` matrix because `src/**`, `.github/workflows/parity-capture.yml`, and visual module deletions all fall through `paths-ignore`.
    - **Expected CI outcome:** all 4 jobs ✓ (cache mostly warm: Cargo.lock unchanged, only feature-set-from-source recompile required for the deleted/modified files; Bevy compile is the dominant cost and stays cached). Wall time projection: 4–10 m (vs. cold-cache 30–60 m).
    - `gh run list --workflow=ci.yml -L 1` → capture run ID. Wait for completion. `gh run view <ID> --log | grep -cE 'warning:|error:'` → expected `0` (modulo the documented `Free disk space` action's `set -x` ambient noise that does not appear on local runs — pre-filter with `grep -v 'Free disk space'` if needed).
  - [x] **Commit 2 (bookkeeping — does NOT trigger CI):** stage `_bmad-output/implementation-artifacts/sprint-status.yaml`, `_bmad-output/implementation-artifacts/deferred-work.md`, `_bmad-output/implementation-artifacts/3-1-title-screen-stub-mainmenu-arena-transition.md`.
    - HEREDOC commit message subject: `bmad: story 3.1 ready-for-dev → review (title screen stub; M1 teardown; epic-3 → in-progress)`. Single-line; matches Story 2.6's bookkeeping commit shape.
    - Push to `origin/master`. **Does NOT trigger CI** — `_bmad-output/**` is in `ci.yml`'s `paths-ignore`.
  - [x] **Why two commits, not one:** matches Stories 2.4/2.5/2.6 precedent — clean diff focus (Commit 1 is reviewable code; Commit 2 is YAML/docs); CI cost focus (Commit 1 triggers CI, Commit 2 doesn't); roll-back granularity (a code-review patch can amend Commit 1 without disturbing the bookkeeping).
  - [x] **Push-fold optimization:** if the dev opts to fold both commits into a single `git push` event, that's acceptable — one CI run captures everything; document the fold in Dev Agent Record. Do NOT collapse the two commits into one (commit-message clarity is preserved by keeping them separate).
  - [x] Story awaits code review. **Code review recommended via `bmad-code-review` skill, ideally with a different LLM than the implementer** per the established pattern. The diff surface is moderate (~250 source-line changes across 6 files + 2 file deletions + 1 workflow deletion + 2 new files); a full 3-agent review (Blind Hunter / Edge Case Hunter / Acceptance Auditor) is appropriate given the scope mixes new code (UI), refactor (state.rs / main.rs), and aggressive deletion (capture/reference-scene/parity-capture). Specific review attention areas: (a) AC #4's test-count off-by-one in this story's spec (verified count = 14, not 13 as written); (b) the `AssetPlugin` setter removal's hot-reload non-regression check; (c) the deferred-work annotation set (4 RESOLVED + 1 new section); (d) the SystemSet `VisualSystems::Setup` empty-after-cleanup deferral handoff to Story 3.3.

## Dev Notes

### Why this story exists

Story 3.1 opens **Epic 3 — Arena Flight & First Combat (First Playable)**, the M2 milestone. It does three things in one cohesive landing:

1. **Replaces the dev hack with a deliberate transition.** Without 3.1, GameState would default to `Loading → MainMenu` and stop — there's no input path into `Arena` because Stories 1.7 (splash → MainMenu) and 2.1 (reference scene persists into MainMenu) don't establish a player-driven trigger to leave MainMenu. Subsequent 3.x stories need an Arena state that the player can REACH from a fresh launch; without a title-screen Enter trigger, 3.2/3.3/3.5 would have to either default-to-Arena (a dev hack the epic explicitly rejects) or rely on the dev pressing keys before Bevy's main loop boots.

2. **Closes the M1 → M2 transition.** Story 2.6's GO toon decision committed the toon material to M2 production. The `m1-decision.md` M2 Impact section explicitly assigns **Story 3.1** as the natural cleanup point for capture mode + reference scene + `parity-capture.yml`. Both are M1-spike-only artifacts; with Arena state opening up, the reference scene's role as visible "something is happening" placeholder is over.

3. **Establishes `src/ui/` as a first-class subtree.** Architecture.md:589-598 reserves `src/ui/` for HUD (3.11), pause overlay (3.4), settings (4.8), full title screen (4.7), post-run (4.9), photo mode (Epic 9), strings loader (Epic 4+). Story 3.1 is the first mover; it sets the `mod.rs`-orchestrator + per-feature-file pattern that all later UI stories follow.

[Source: [`epics/epic-3-arena-flight-first-combat-first-playable.md:5-27`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md) (Story 3.1 epic spec); [`docs/tech-spike/m1-decision.md:37-42`](../../docs/tech-spike/m1-decision.md) (M2 Impact bullets); [`architecture.md:589-598`](../planning-artifacts/architecture.md) (`src/ui/` subtree); [`deferred-work.md:99-102`](./deferred-work.md) (Story 2.5 cleanup assignment to 3.1)]

### Inherited context from Stories 1.7, 2.1, 2.2, 2.6

| Fact | Value | Source |
|---|---|---|
| Bevy version | `0.18` (resolved `0.18.1`) | `Cargo.toml:8` |
| `bevy_ui` + `default_font` Cargo features | enabled (Story 1.7 added them) — no Cargo.toml changes needed for 3.1's UI work | `Cargo.toml:8` |
| `file_watcher` Cargo feature | enabled (added by Story 2.3 for tuning.ron hot-reload) | `Cargo.toml:8` |
| Splash flow | `OnEnter(Loading) → SplashConfig 2.0s timer → NextState(MainMenu)` | `src/splash.rs` |
| MainMenu state | live (Story 1.7); currently no UI surface beyond reference-scene swatches (cfg-debug-only) | `src/state.rs:18` |
| Arena state | declared in enum (Story 1.6) but never `live` (no `NextState::set(Arena)` exists pre-3.1) | `src/state.rs:18` |
| `GameState` `#[expect(dead_code)]` attribute | still present; `Arena` becoming live in 3.1 reduces the unused-variant set from 5 to 4; expectation still fires (4 unused remain) | `src/state.rs:7-11` |
| Reference scene Camera3d at `order: -1` | persists into MainMenu (Story 2.1) — gone in 3.1 | `src/visual/reference_scene.rs:55-63` |
| MainMenu swatch UI | `Camera2d order: 1` + `Node` tree spawned `OnEnter(MainMenu)` (Story 2.2) — gone in 3.1 | `src/visual/reference_scene.rs:153-225` |
| Capture mode | `ASTEROIDS3D_CAPTURE_PNG` env var triggers screenshot + exit (Story 2.5) — gone in 3.1 | `src/visual/capture.rs` |
| `parity-capture.yml` workflow | manual workflow_dispatch only; runs `25111626273`, `25113263165` produced M1 evidence (Story 2.5) — gone in 3.1 | `.github/workflows/parity-capture.yml` |
| Test count post-2.6 | **14 passing** | `_bmad-output/implementation-artifacts/2-6-go-fallback-decision-document.md:432` |
| Test count post-3.1 (expected) | **14** (loses capture's 1 test, gains main_menu's 1 test) | this story |
| `tracing` + panic-hook (Story 1.8) | live; `init_logging()` returns log path; subscriber routes `info!` to file + stderr | `src/logging.rs` |
| `TuningPlugin` hot-reload | active in debug builds via `file_watcher` feature; `TuningReloaded` message propagated | `src/tuning/mod.rs` |
| Bevy `ChildOf` linked-despawn | default since 0.16 — `commands.entity(parent).despawn()` recursively despawns children | Bevy 0.18 changelog |
| `cleanup_loading_entities` race | known WARN ("Entity despawned… invalid") on splash exit due to over-tagged child Text | `deferred-work.md:71-73` |
| Architecture path for UI | `src/ui/mod.rs` + `src/ui/main_menu.rs` per `architecture.md:589,591` | architecture.md |
| Commit style precedent | `feat:` for source, `bmad:` for bookkeeping; no Co-Authored-By trailer; HEREDOC for multi-line messages | `git log --oneline -n 15` |

### Five-key constraint summary (memorize these)

1. **Mark only despawn-roots, not descendants.** `MainMenuEntity` goes on `Camera2d` + parent `Node`. Child `Text` entities are NOT marked. Cleanup uses Bevy 0.18's `ChildOf` linked-despawn cascade. This avoids the splash race (deferred-work.md:71-73).
2. **`just_pressed`, NOT `pressed`.** Holding Enter would otherwise re-fire `NextState::set(Arena)` every frame for as long as the key is held — harmless functionally (Bevy coalesces same-state queue) but log-spammy and semantically wrong.
3. **`bundled cleanup is intentional, not bolt-on`.** Capture mode + reference scene + parity-capture.yml deletions land in this commit because (a) they're the M2 Impact cleanup assigned by Story 2.6, (b) reference scene visually conflicts with title screen UI in MainMenu, (c) capture mode depends on reference scene's Camera3d. Splitting them across stories would create a transient "MainMenu has both title screen AND M1 swatches" intermediate state that isn't useful.
4. **`palette.rs` cfg_attr → unconditional allow.** Removing reference_scene leaves SemanticAccent / color_for unused in BOTH debug and release. The current `cfg_attr(not(debug_assertions), allow(dead_code))` only suppresses release; debug now warns. Convert to plain `#[allow(dead_code, reason = "...")]`. Story 4.5 will remove the allow when it adds gameplay-entity consumers.
5. **Test count stays at 14.** Net = pre-3.1 (14) − capture test (1) + new main_menu test (1) = **14**. Both the AC text and Task 9 verification expect 14; if your local `cargo test` reports anything else, investigate.

### Architecture compliance

- **`src/ui/mod.rs` + `src/ui/main_menu.rs`** match `architecture.md:589,591` exactly. First mover into the UI subtree per the architecture's plugin layout. [Source: architecture.md:589-598]
- **`UiPlugin` is registered in `main.rs` via `add_plugins(UiPlugin)`** matching the per-feature-plugin convention from `architecture.md:643` (Plugin Boundaries table). The plugin owns the title-screen surface; future per-state UI (HUD, pause, settings, post-run) extends `UiPlugin` rather than introducing competing plugins. [Source: architecture.md:343-350]
- **State-transition cleanup via marker component + `OnExit` despawn** matches `architecture.md:420` ("entities spawned for a state tag themselves with a marker component … despawned by a cleanup_on_exit::<T> system in OnExit"). `MainMenuEntity` + `cleanup_main_menu` is the canonical implementation. [Source: architecture.md:420]
- **`NextState<GameState>` mutation via `next_state.set(...)`** matches `architecture.md:418` (never mutate `State<GameState>` directly). [Source: architecture.md:418]
- **No `SystemSet` for UiPlugin in 3.1** — architecture.md:347-349 prescribes a `<Feature>Systems` enum per plugin, but the same passage says "for ordering" — Story 3.1's UiPlugin has only one logical phase per state-edge (spawn / handle / cleanup, each in distinct schedules), so no ordering exists to gate. The set can be added in Epic 4's full title-screen story (4.7) when settings/credits/quit submenu transitions need ordering. Documented as deliberate deviation. [Source: architecture.md:347-349 + pattern-deviation process]
- **Marker-only-on-roots cleanup pattern** is a refinement of the canonical `cleanup_on_exit::<T>` from architecture.md:420 — the canonical pattern doesn't specify whether children are marked, but Bevy 0.18's `ChildOf` linked-despawn semantics make root-only marking strictly cleaner (no race; smaller archetype scan). Story 3.1's adoption of root-only marking sets a precedent for all future cleanup queries. The splash code's child-marker pattern (Story 1.7 review patch F2) is left in place per the deferred-work entry's "next story that touches splash.rs" rule (3.1 doesn't touch splash). [Source: architecture.md:420 + deferred-work.md:71-73]

### Library / framework requirements

| Crate | Version | Change in Story 3.1 |
|---|---|---|
| `bevy` | `0.18` (resolved `0.18.1`) | unchanged |
| All other pinned deps | unchanged | unchanged |
| `Cargo.toml` | unchanged (modulo Task 9 conditional regression-fix on `AssetPlugin` setter) | no feature additions, no version bumps |
| `Cargo.lock` | **unchanged** — no dep tree change | should be byte-identical post-3.1 |

No new dependencies. No version bumps. The story is purely application-source surgery + workflow deletion.

### File structure changes

| Path | Action | Purpose |
|---|---|---|
| `src/ui/mod.rs` | **Add** | `UiPlugin` orchestrator; ~30 lines. |
| `src/ui/main_menu.rs` | **Add** | Title-screen UI + input handler + cleanup; ~80–100 lines including 1 unit test. |
| `src/main.rs` | **Modify** | Add `mod ui;` + `use ui::UiPlugin;` + `.add_plugins(UiPlugin)` + `OnEnter(GameState::Arena)` registration; **remove** capture-mode wiring + `AssetPlugin` import + `set(AssetPlugin{...})` setter + the `cfg(target_os="macos")` block-related items if any (none in current `main.rs`). Net **shrinks** from 71 to ~50 lines. |
| `src/state.rs` | **Modify** | Append `pub fn log_arena_entered() { info!("entered Arena"); }` after `log_mainmenu_entered`. +3 lines. |
| `src/visual/mod.rs` | **Modify** | Remove `pub mod capture;`, `#[cfg(debug_assertions)] mod reference_scene;`, the `app.add_plugins(reference_scene::ReferenceScenePlugin);` block. Trim doc-comment lines that referenced gone modules. Net shrinks ~10 lines. |
| `src/visual/palette.rs` | **Modify** | Convert two `#[cfg_attr(not(debug_assertions), allow(...))]` blocks to unconditional `#[allow(dead_code, reason = "...Story 4.5...")]`. Net 0 lines (replacements). |
| `src/visual/capture.rs` | **Delete** (`git rm`) | M1-spike artifact, scope explicit per `m1-decision.md` M2 Impact + Story 2.5 deferred-work assignment. |
| `src/visual/reference_scene.rs` | **Delete** (`git rm`) | M1 dev scaffold, scope explicit per `m1-decision.md` M2 Impact. |
| `.github/workflows/parity-capture.yml` | **Delete** (`git rm`) | M1-spike CI workflow, scope explicit per Story 2.5 deferred-work assignment. |
| `Cargo.toml`, `Cargo.lock` | **Do NOT touch** | No version bumps, no feature additions. (Conditional Task 9 exception covered above.) |
| `Cargo.lock` | **Do NOT regenerate** | No dep tree change. |
| `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` | **Do NOT touch** | Out of scope. |
| `assets/**` | **Do NOT touch** | No asset changes. The `default_font` feature embeds Fira Sans Regular at compile time; no TTF files. |
| `docs/tech-spike/m1-backends/**` | **Do NOT touch** | M1 audit-trail evidence; explicitly preserved per `m1-decision.md` M2 Impact bullet 4. |
| `docs/tech-spike/m1-decision.md` | **Do NOT touch** | The decision document is the audit trail; no edit needed. |
| `_bmad-output/planning-artifacts/**` | **Do NOT touch** | PRD / architecture / epic specs are read-only from a story-execution perspective. |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | **Modify** (Task 12) | epic-3 → in-progress, 3-1 → review, last_updated bump. |
| `_bmad-output/implementation-artifacts/deferred-work.md` | **Modify** (Task 11) | 4 RESOLVED annotations + 1 new "Deferred from: 3-1" section. |
| `_bmad-output/implementation-artifacts/3-1-...-md` (this file) | **Modify** | Tasks checked, Dev Agent Record populated, Status → review. |

### `src/ui/main_menu.rs` skeleton (near-verbatim — rustfmt-tolerant)

```rust
//! Title-screen UI for GameState::MainMenu (Story 3.1: stub for FR36).
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

pub fn cleanup_main_menu(
    mut commands: Commands,
    query: Query<Entity, With<MainMenuEntity>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_font_size_is_at_least_double_subtitle() {
        assert!(
            TITLE_FONT_SIZE >= 2.0 * SUBTITLE_FONT_SIZE,
            "AC #1 hierarchy guarantee: title must be ≥2× subtitle"
        );
    }
}
```

### `src/main.rs` post-edit skeleton (rustfmt-tolerant)

```rust
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
```

### `src/state.rs` delta

Append after `log_mainmenu_entered`:

```rust
pub fn log_arena_entered() {
    info!("entered Arena");
}
```

The `GameState` enum + its `#[expect(dead_code, reason = "...")]` attribute are NOT touched. With `Arena` becoming live, 4 variants remain unused (`Caravan`, `PostRun`, `PhotoMode`, `Paused`) — the lint expectation is still satisfied.

### Testing requirements

- **Unit tests:**
  - 1 new test in `src/ui/main_menu.rs`: `title_font_size_is_at_least_double_subtitle`. Pure-function, zero I/O, CI-safe. Asserts the AC #1 hierarchy guarantee at compile-derived constants.
  - 1 test removed: `src/visual/capture.rs` `capture_disabled_when_env_var_unset` goes with the file deletion.
  - **Net test count: 14** (= 14 - 1 + 1).
- **Manual test:** `cargo run` on macOS — visually verify splash → title screen → press Enter → Arena (blank window). Verify 5 expected log lines (`entered Loading`, `splash timer elapsed`, `entered MainMenu`, `MainMenu: Enter pressed`, `entered Arena`) appear in order. Verify NO M1 reference scene (no Camera3d-rendered meshes, no swatch UI bar at top). Tuning hot-reload sanity test (Task 9): edit `assets/config/tuning.ron`, observe `TuningReloaded` log line within ~2 s.
- **Integration tests:** still deferred (architecture.md:354 — co-located unit tests for pure logic; integration tests post-M3 unless regression forces). State-transition + UI-cleanup integration coverage is Epic 3's accumulated dev-time playtest, not a CI-blocking suite.
- **Windows / Linux runtime verification:** same pattern as Stories 1.5/1.6/1.7 — Till runs `cargo run` on physical hardware when convenient. CI verifies compile parity on 3 OSes; manual cross-platform smoke is non-blocking.

### Latest technical information

- **Bevy 0.18 `ButtonInput<KeyCode>` API:** `keys.just_pressed(KeyCode::Enter)` returns `true` only on the frame the key transitioned `released → pressed`. `keys.pressed(...)` returns `true` for every frame the key is held. **Use `just_pressed`** — Story 3.1 wants a single transition event per Enter press, not a re-fire-every-frame storm. Both `KeyCode::Enter` (mainline keyboard return) and `KeyCode::NumpadEnter` (numpad return) are checked; future leafwing-input-manager rebinding (Stories 3.6+) supersedes this hard-coded set.
- **Bevy 0.18 `ChildOf` linked-despawn:** since 0.16, `commands.entity(parent).despawn()` recursively despawns the parent AND all of its `ChildOf`-linked descendants. The Story 2.1 dev-verification log (`deferred-work.md:71-73`) showed that over-tagging children produces a cleanup-iteration race because `iter()` may visit a child after its parent's recursive cascade has already despawned it. **Mitigation in 3.1:** mark only despawn roots (parent Node + Camera2d). Children are unmarked and despawn via the cascade.
- **Bevy 0.18 `default_font` feature:** Fira Sans Regular is embedded at compile time. `TextFont { font_size: ..., ..default() }` uses the default handle — no TTF asset needed. Story 1.7 enabled this feature; 3.1 inherits it.
- **Bevy 0.18 `bevy_ui` flexbox:** `flex_direction: FlexDirection::Column` + `JustifyContent::Center` + `AlignItems::Center` produces a centered vertical stack. `row_gap: Val::Px(N)` (in Bevy 0.18, this is the cross-axis gap, equivalent to CSS `row-gap` in column flex direction) gives the title-subtitle spacing. **Verified in production-grade Bevy 0.18 examples** — no API surprises.
- **Bevy 0.18 `file_watcher` Cargo feature:** when enabled, asset hot-reload works in `cargo run` (debug) without an explicit `set(AssetPlugin { watch_for_changes_override: ... })` setter. Story 1.7 added `bevy_ui` + `default_font` features; Story 2.3 added `file_watcher`. Removing the explicit setter in main.rs (Task 4) relies on the Cargo feature carrying the behavior alone. **If empirically broken** (Task 9 hot-reload smoke), restore the setter.

### Previous-story intelligence — what to learn from 1.7 / 2.5 / 2.6

**From Story 1.7 (splash + bevy_ui first surface):**
- Two-commit pattern (source + bookkeeping). Story 3.1 follows.
- `feat:` commit subject for source; `bmad:` for bookkeeping. No `Co-Authored-By` trailer.
- Module doc ≤ 2 lines, no story-id references (review patch BH8 from Story 1.5).
- Rustfmt may re-order `use` blocks alphabetically. Accept its order.
- `bevy_winit Destroyed for unknown winit Window Id` WARN on close is a known Bevy 0.18 race (deferred-work.md LOW-1 from Story 1.6). **Not a 3.1 regression** if it appears in `/tmp/story-3-1-run.log`.
- `Skipping installing Ctrl+C handler as one was already installed` INFO at startup — first observed in Story 1.7. Informational, not a defect.

**From Story 2.5 (parity capture + cleanup assignment):**
- The deferred-work entry at lines 99-102 IS the cleanup contract for Story 3.1. The four targets (capture.rs, mod.rs declaration, main.rs wiring, parity-capture.yml) are listed verbatim. Match them exactly.
- The contingency entries at 101-102 (`WGPU_ADAPTER_NAME`, `VK_ICD_FILENAMES`) become moot when the workflow is deleted — they don't get RESOLVED annotations because they were conditional contingencies that never triggered.

**From Story 2.6 (M1 decision document):**
- `m1-decision.md`'s M2 Impact section names Story 3.1 as the cleanup landing for capture.rs, the mod.rs declaration, the main.rs wiring, and `parity-capture.yml`. The decision document also names `docs/tech-spike/m1-backends/` as preserved audit-trail evidence — DO NOT touch those files.
- The reference scene was not explicitly named in `m1-decision.md`'s cleanup list, but `m1-decision.md` is the M1 closure document and the reference scene is the M1 dev scaffold; its removal at M2 entry is implied. Story 2.6 deferred-work entries also do not list it explicitly. The Story 2.2 deferred-work entry on swatch leak (line 87) explicitly names "Story 3.1 removes the swatches entirely" as the resolution, which depends on reference_scene.rs deletion.
- Two-commit pattern (source + bookkeeping) confirmed Stories 2.4 / 2.5 / 2.6.
- The `not-needed` sprint-status value (introduced by Story 2.6) is not relevant to Story 3.1 — 3-1 follows the standard `backlog → ready-for-dev → review → done` lifecycle.

### Forward compatibility — Story 3.2 (Avian + Arena state skeleton) hand-off

Story 3.2 reads this story's outcome and assumes:
- `GameState::Arena` is reachable from MainMenu via Enter press (true post-3.1).
- No Arena state systems exist yet (true post-3.1 — only `log_arena_entered` runs on entry).
- The `src/visual/` plugin no longer registers reference_scene (true post-3.1).
- `VisualSystems::Setup` SystemSet is configured on `OnEnter(Loading)` and is empty (Story 3.1 leaves the configuration in place; Story 3.3 reconfigures to `OnEnter(Arena)` when it spawns asteroids).

Story 3.2 will:
- Add `avian3d::PhysicsPlugins::default()` registration in main.rs (the first physics plugin registration in the project).
- Add `Gravity(Vec3::ZERO)` resource insertion (zero-g space environment).
- Author `src/arena/mod.rs` with `ArenaPlugin`, `ArenaSystems` SystemSet enum, `ArenaEntity` marker.
- Register `cleanup_on_exit::<ArenaEntity>`-style despawn on `OnExit(GameState::Arena)` — using the SAME marker-on-roots pattern Story 3.1 establishes for `MainMenuEntity`.

The `MainMenuEntity` marker pattern in 3.1 is the architectural reference for `ArenaEntity` in 3.2 — 3.2's author should mirror 3.1's structure (root-only marking, `commands.entity(e).despawn()`, no `try_despawn`, no recursive marker).

### Forward compatibility — Story 4.7 full title screen hand-off

Story 4.7 ("Title screen full FR36 — Start / Settings / Credits / Quit") rewrites `src/ui/main_menu.rs` to add:
- 4 button entities (Start, Settings, Credits, Quit) instead of "Press Enter to start" subtitle
- Mouse / keyboard navigation (arrow keys + Enter, or click)
- A `UiSystems` SystemSet for ordering between button-spawn and input-handling
- Possibly a `MainMenuButton` enum component variant per button
- String-table integration via Story 4.x's RON loader (replacing the hard-coded `TITLE_TEXT` / `SUBTITLE_TEXT` constants)

Story 3.1's stub minimizes Story 4.7's rewrite surface: the `MainMenuEntity` marker stays load-bearing (4.7 reuses the same cleanup pattern); the constants get replaced; `handle_main_menu_input` evolves into a button-driven dispatch. Story 4.7 also moves the splash to `src/ui/splash.rs` if not done already (per Story 1.7's architecture-compliance promise). Story 4.7 may also introduce the `UiSystems` enum.

### Forward compatibility — palette.rs cfg_attr removal at Story 4.5

Story 4.5 ("SemanticAccent wiring on asteroids / salvage / playership / projectiles → PlayerOwned") attaches `SemanticAccent` to gameplay entities in non-debug paths. With those release-path consumers in place, the unconditional `#[allow(dead_code, ...)]` blocks on `SemanticAccent` and `color_for` become legitimately dead code themselves (the items are used; the allow is no longer needed). Story 4.5's dev should:
1. Remove both `#[allow(dead_code, reason = "...")]` blocks from `src/visual/palette.rs`.
2. Verify `cargo build --release` is 0-warnings and 0-errors.
3. Verify `nm target/release/asteroids3D | grep -c color_for` ≥ 1 (release-path consumer present).

This step is already tracked in `deferred-work.md` (Story 2.2 cfg_attr block, lines 94–96), re-deferred to Story 4.5 by Story 2.3 Task 9. Story 3.1's edit to those `cfg_attr` → `allow` blocks DOES NOT change the resolution path — Story 4.5 still removes the allow when consumers land.

### Project structure notes

- **Path alignment with architecture.md:**
  - `src/ui/mod.rs` and `src/ui/main_menu.rs` match `architecture.md:589,591` exactly. **First mover into the UI subtree.**
  - `src/main.rs`, `src/state.rs`, `src/visual/mod.rs`, `src/visual/palette.rs` are all unchanged in path; modifications are in-place.
  - `src/visual/capture.rs` and `src/visual/reference_scene.rs` are deleted; their architectural slots (toon material, palette, outline) remain intact.
- **No path conflicts or variances** introduced by Story 3.1.
- **Splash file location debt unresolved:** `src/splash.rs` stays flat at `src/`, not `src/ui/splash.rs` per architecture. Story 1.7 deferred the move to "when UiPlugin skeleton lands"; Story 3.1 lands UiPlugin but consciously skips the move (filed in deferred-work as a future-story chore). Rationale: splash file location has no functional impact; bundling it inflates Story 3.1's surface beyond the focused FR36-stub-plus-M1-cleanup theme.
- **`Cargo.toml`, `Cargo.lock`, `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `assets/**`, `docs/**` (modulo deferred-work.md) — UNTOUCHED.**

### LLM dev-agent guardrails — most-likely-to-go-wrong patterns

These are the failure modes most likely to bite a fast-moving dev:

1. **Marking the child Text entities with `MainMenuEntity`.** Don't. Tag only Camera2d + parent Node. Children inherit cleanup via Bevy 0.18 `ChildOf`-linked-despawn. The deferred-work.md:71-73 race in splash is the cautionary tale.
2. **Using `keys.pressed(KeyCode::Enter)` instead of `keys.just_pressed(KeyCode::Enter)`.** `pressed` re-fires `next_state.set(Arena)` every frame the key is held. Use `just_pressed`. Verify with the `MainMenu: Enter pressed` log count = 1 per actual key press (Task 9).
3. **Forgetting to delete `parity-capture.yml`.** The cleanup contract names FOUR targets: capture.rs, mod.rs declaration, main.rs wiring, AND parity-capture.yml. Three-of-four compiles fine but leaves a dispatched-only-by-hand workflow that would fail at runtime. Run Task 8.
4. **Forgetting Task 7 (palette cfg_attr conversion).** Removing reference_scene without converting palette's cfg_attr produces `dead_code` warnings on debug builds. Both blocks (the enum + the function) need conversion.
5. **Skipping the new main_menu test to "shrink" the count.** Don't. The new `title_font_size_is_at_least_double_subtitle` test is the AC #1 hierarchy guarantee in compile-derived form. Net count change is 0 (−1 capture + 1 main_menu); expected post-3.1 count is **14**.
6. **Forgetting to add `OnEnter(GameState::Arena)` system registration in main.rs.** The transition will WORK without it (the state still flips), but `entered Arena` log line won't fire — AC #3 explicitly requires this log.
7. **Touching `src/splash.rs`.** This story does NOT modify splash. The deferred-work entry on the splash race STAYS deferred (re-deferred in Task 11). Don't "drive-by patch" splash — that creates scope drift.
8. **Touching `Cargo.toml`.** No version bumps, no feature additions. The `default_font` and `file_watcher` features are already there. The only conditional Cargo.toml edit is the Task 9 escape hatch IF the `AssetPlugin` setter removal breaks tuning hot-reload — and that's an empirical fallback, not a planned change.
9. **Touching `docs/tech-spike/m1-backends/**`.** Audit-trail evidence; the `m1-decision.md` M2 Impact bullet 4 explicitly preserves these files. Read-only from 3.1's perspective.
10. **Adding a `UiSystems` enum.** Not yet. Story 3.1's UI surface has no internal ordering; the SystemSet adds maintenance burden without benefit. Defer to Story 4.7 when settings/credits/quit submenu transitions need ordering.
11. **Re-running the parity-capture workflow before deletion.** The workflow is `workflow_dispatch` only, so it won't auto-run, but a dev who's curious about "does it still work" might trigger it pre-deletion. The capture binary code is being deleted in the same commit; any post-Task-4 dispatch fails. Don't dispatch.
12. **Forgetting Task 11 (deferred-work.md updates).** 4 RESOLVED annotations + 1 new "Deferred from: 3-1" section. The annotations close out cross-story tracking; the new section flags the splash race re-deferral + splash location debt + empty SystemSet.

### Why bundle the M1 cleanup with the title screen?

This is a deliberate scope choice. Three alternatives were considered:

**Alternative A (rejected): Title screen only — defer cleanup to a later story.**
- Pro: smaller diff, simpler review.
- Con: leaves transitional state where MainMenu has BOTH the new title screen UI AND the M1 reference scene swatches simultaneously visible. Visually broken intermediate state.
- Con: violates the M2 Impact contract from `m1-decision.md`.

**Alternative B (rejected): Cleanup only — move title screen to a later story.**
- Pro: pure deletion commit, very small diff.
- Con: leaves MainMenu with NO visible UI (reference scene gone, no replacement). Game effectively unplayable from launch (no path to Arena).
- Con: violates the epic spec's narrative ("from the first Epic-3 commit, no dev hacks like default-to-Arena").

**Alternative C (selected): Title screen + cleanup in one story.**
- Pro: produces a coherent post-commit state — splash → title screen → Arena (blank for now), no M1 detritus.
- Pro: matches the M2 Impact contract verbatim.
- Pro: the M1 reference scene's role (visible "something is happening" placeholder) is replaced by the new title screen — a 1:1 logical substitution.
- Con: larger diff (~250 lines + 3 deletions + 2 additions). Mitigated by tightly-scoped subtasks and a thorough verification sweep.

### Test count discipline

Pre-3.1 (post-2.6): **14 passing tests**. Story 3.1 modifies the count as follows:
- **Removed (with file deletion):** `capture::tests::capture_disabled_when_env_var_unset` (1 test).
- **Added:** `ui::main_menu::tests::title_font_size_is_at_least_double_subtitle` (1 test).
- **Net post-3.1: 14 passing tests.**

If `cargo test` reports anything other than `14 passed`:
- **<14:** another test was accidentally deleted from `src/`. Investigate `git diff --stat src/`; revert.
- **>14:** an unscoped test was added. Investigate; this story spec authorizes only one new test.

Both AC #4 and Task 9 expect post-3.1 count = **14**. The dev should verify `cargo test` reports `14 passed` and note the per-file test delta in Completion Notes (which test was removed, which was added).

### References

- [Source: [`_bmad-output/planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md:5-27`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md)] — Story 3.1 epic spec with three GIVEN/WHEN/THEN ACs.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:589-598`](../planning-artifacts/architecture.md)] — `src/ui/` subtree layout.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:417-420`](../planning-artifacts/architecture.md)] — State-transition patterns (NextState + marker-component cleanup).
- [Source: [`_bmad-output/planning-artifacts/architecture.md:343-350`](../planning-artifacts/architecture.md)] — Plugin-per-feature module pattern.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:226-230`](../planning-artifacts/architecture.md)] — Hybrid screen-space + world-space HUD strategy (3.1 lays the screen-space foundation).
- [Source: [`_bmad-output/planning-artifacts/prd.md`](../planning-artifacts/prd.md)] — FR36 (title screen — full version is Story 4.7), FR43 (pause on focus loss — Story 3.4).
- [Source: [`docs/tech-spike/m1-decision.md`](../../docs/tech-spike/m1-decision.md)] — M2 Impact section assigning capture-mode + reference-scene cleanup to Story 3.1.
- [Source: [`_bmad-output/implementation-artifacts/2-6-go-fallback-decision-document.md`](./2-6-go-fallback-decision-document.md)] — Story 2.6 closure context.
- [Source: [`_bmad-output/implementation-artifacts/1-7-splash-screen-shows-asteroids3d-and-transitions-to-mainmenu.md`](./1-7-splash-screen-shows-asteroids3d-and-transitions-to-mainmenu.md)] — Splash + first bevy_ui surface; constants pattern; child-marker race origin.
- [Source: [`_bmad-output/implementation-artifacts/2-1-visualplugin-skeleton-reference-scene.md`](./2-1-visualplugin-skeleton-reference-scene.md)] — Reference scene origin + cfg-debug-only gating.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:71-73`](./deferred-work.md)] — Splash cleanup-iteration race; re-deferred by 3.1.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:99-102`](./deferred-work.md)] — Story 2.5 → Story 3.1 cleanup assignment for capture mode + parity-capture.yml.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:67-69`](./deferred-work.md)] — Reference scene idempotency concern (resolved by 3.1's removal).
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:87-88`](./deferred-work.md)] — Swatch UI no-cleanup-on-state-exit (resolved by 3.1's reference_scene deletion).
- [Source: [`Cargo.toml`](../../Cargo.toml)] — current Bevy feature set, including `bevy_ui`, `default_font`, `file_watcher`.
- [Source: [`MEMORY.md` → `feedback_full_build_output.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_full_build_output.md)] — verification discipline: exit-0 + tail is NOT proof; grep explicitly.
- [Source: [`MEMORY.md` → `feedback_staged_rollout.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_staged_rollout.md)] — staged-rollout preference; informs scope-bundling rationale (cleanup + UI together = 1 logical step, not 2).
- [Source: [`MEMORY.md` → `feedback_compact_review_style.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_compact_review_style.md)] — Till answers numbered-question reviews compactly; relevant for code-review post-3.1.

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (1M context) — Claude Code CLI, dev-story workflow, 2026-04-30.

### Debug Log References

Local verification sweep (all logs in `/tmp/`):

| Command | Log file | `grep -cE 'warning:|error:'` | Notes |
|---|---|---|---|
| `cargo check` | `/tmp/story-3-1-check.log` | 0 | Cached after first build; Finished in 0.14s |
| `cargo build` (debug) | `/tmp/story-3-1-build.log` | 0 | Recompile after edits, 3.26s |
| `cargo test` | `/tmp/story-3-1-test.log` | 0 (warning/error/FAILED) | `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-3-1-clippy.log` | 0 | After `const { assert!(...) }` fix on the new test (see Completion Notes) |
| `cargo fmt --all -- --check` | (stdout) | exit 0 | No reformat needed |
| `cargo build --release` | `/tmp/story-3-1-release.log` | 0 | 3m 35s wall (cold release) |
| `cargo run` runtime smoke | `/tmp/story-3-1-run.log` | 0 panic/FATAL | All 5 expected lifecycle log lines fired in order |

Runtime smoke evidence (excerpt):

```
INFO bevy_render::renderer: AdapterInfo { ... backend: Metal }
INFO asteroids3D::state: entered Loading
INFO asteroids3D::tuning: TuningReloaded: toon_steps=4 rim_power=2 rim_intensity=0.3
INFO asteroids3D::splash: splash timer elapsed, transitioning to MainMenu
INFO asteroids3D::state: entered MainMenu
INFO asteroids3D::ui::main_menu: MainMenu: Enter pressed, transitioning to Arena
INFO asteroids3D::state: entered Arena
```

Backend confirmed Metal on Apple M5 Pro per AC #5.

### Completion Notes List

**Per-AC evidence:**

- **AC #1 (title screen UI hierarchy + marker on roots only):** `src/ui/main_menu.rs::spawn_main_menu` spawns `(Camera2d, MainMenuEntity)` plus a parent `Node` (100% × 100%, `JustifyContent::Center` + `AlignItems::Center` + `FlexDirection::Column` + `row_gap: 24px`) tagged `MainMenuEntity`. Two child `Text` entities (title `"asteroids3D"` + subtitle `"Press Enter to start"`) are spawned via `with_children` and are NOT tagged with `MainMenuEntity` — they cascade-despawn via Bevy 0.18 `ChildOf` linked-despawn, sidestepping the splash race. Title font `96.0`, subtitle `32.0` (3× ratio, exceeds the AC's 2× minimum). Hierarchy guarantee asserted at compile time via `const { assert!(TITLE_FONT_SIZE >= 2.0 * SUBTITLE_FONT_SIZE) }` inside the unit test body (see clippy note below).

- **AC #2 (Enter / NumpadEnter → Arena, single transition):** `handle_main_menu_input` uses `keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter)`. Runtime smoke log shows exactly 1 `MainMenu: Enter pressed, transitioning to Arena` line per Enter press. The system is registered in `UiPlugin` with `run_if(in_state(GameState::MainMenu))`.

- **AC #3 (cleanup + Arena entry log):** `cleanup_main_menu` queries `Query<Entity, With<MainMenuEntity>>` and despawns matches; children cascade. `log_arena_entered` added to `src/state.rs` and registered via `add_systems(OnEnter(GameState::Arena), log_arena_entered)`. Runtime log shows `entered Arena` fires immediately after the Enter-press log (same tick — `08:36:37.529847Z` → `08:36:37.532637Z`).

- **AC #4 (M1 capture + reference-scene + parity-capture teardown):** `git rm` of `src/visual/capture.rs`, `src/visual/reference_scene.rs`, `.github/workflows/parity-capture.yml`. `src/visual/mod.rs` pruned of `pub mod capture;`, `mod reference_scene;`, `add_plugins(reference_scene::ReferenceScenePlugin)`, plus the Story 2.5 doc-comment line. `src/main.rs` lost the `capture_path` lookup, conditional `WindowPlugin` override, conditional `CapturePlugin` add, the explicit `set(AssetPlugin { watch_for_changes_override: ... })` setter, and the now-unused `use bevy::asset::AssetPlugin;` import. `src/visual/palette.rs` two `cfg_attr(not(debug_assertions), allow(dead_code, ...))` blocks converted to unconditional `#[allow(dead_code, reason = "no current consumer; gameplay consumer arrives in Story 4.5 ...")]`. Both debug and release builds now 0 warnings, confirmed in build logs. Test count = 14 (−1 capture, +1 main_menu).

- **AC #5 (verification sweep + git status + runtime smoke):** All 6 cargo commands clean (per Debug Log table). `cargo test` summary line reads exactly `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. `cargo run` runtime smoke confirms full lifecycle Loading → MainMenu → Arena with all 5 expected log lines. `git status --short` matches the spec's expected file set (see File List below); `Cargo.toml`, `Cargo.lock`, `.gitignore`, `assets/`, `docs/`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` all untouched.

**Deviations from spec:**

1. **Test body uses `const { assert!(...) }` instead of plain `assert!(...)`.** Clippy 1.94.1 fires `clippy::assertions_on_constants` on the original `assert!(TITLE_FONT_SIZE >= 2.0 * SUBTITLE_FONT_SIZE, ...)` because both operands are compile-time constants, breaking `cargo clippy --all-targets -- -D warnings`. Clippy's literal suggestion was to wrap in a `const { ... }` block — adopted verbatim. The `const`-block assertion fires at compile time (preventing any future change that would break the AC #1 hierarchy invariant from ever building), and the runtime test still passes (empty body). Net effect is strictly stronger than the spec's runtime-only assert. The custom assert message `"AC #1 hierarchy guarantee: title must be ≥2× subtitle"` was dropped because `assert!` inside `const { ... }` doesn't accept it on stable Rust 1.94.1 (`const_format_args` is unstable); the constant names self-document the invariant.

2. **`grep -rn 'GameState::Arena' src/` returns 3 hits, not 2 as written in spec Task 10.** The third hit is in `src/ui/main_menu.rs:2` — the module doc-comment that documents the transition target (`//! Press Enter / NumpadEnter to transition to GameState::Arena.`). The two code-usage hits expected by the spec (registration in `main.rs:41`, call in `main_menu.rs:58`) are both present and correct. The doc-comment hit is benign — purely documentation referencing the type name.

**Observed behaviors (non-blocking):**

- **Splash cleanup-iteration race fired on one boot smoke and not the other within the same dev session** — non-deterministic per the existing `deferred-work.md` characterization. Story 3.1 does not touch `src/splash.rs` (re-deferred per Task 11 entry). Story 3.1's own `cleanup_main_menu` system is race-free by design (root-only marking).

- **"Window not going blank" after Enter press** — observed during interactive smoke. Cause: with all MainMenu entities (Camera2d + Node tree) despawned and Arena state having no camera yet (Story 3.2 lands `avian3d` + Arena state skeleton; Story 3.3 spawns the asteroid field + Arena camera), Bevy holds the last swapchain image. Cosmetic stale-frame artifact, NOT a Story 3.1 regression — log evidence confirms `entered Arena` fires correctly. Resolves naturally at Story 3.2/3.3 with first Arena-state camera.

- **Tuning hot-reload visual non-change is expected, not a bug** — `TuningReloaded: toon_steps=...` log line fires within ~2s of `tuning.ron` save (proof the `file_watcher` Cargo feature carries hot-reload without the `set(AssetPlugin{...})` setter that Task 4 removed). No visible change because no toon-shaded mesh exists on the title screen — only `bevy_ui` text, which doesn't consume `ToonMaterial`. First visible hot-reload after Story 3.3 (asteroid field).

**Two-commit precedent maintained** — Commit 1 (source, triggers CI) + Commit 2 (bookkeeping, paths-ignore). See File List for split.

### File List

**Added:**
- `src/ui/mod.rs` — `UiPlugin` orchestrator (~25 lines)
- `src/ui/main_menu.rs` — title-screen UI + input + cleanup + 1 unit test (~78 lines)

**Modified:**
- `src/main.rs` — `mod ui;` + `use ui::UiPlugin;` + `add_plugins(UiPlugin)` + `OnEnter(GameState::Arena)` registration; removed capture wiring + `AssetPlugin` import + setter (file shrank from 71 to 49 lines)
- `src/state.rs` — appended `log_arena_entered` (+3 lines)
- `src/visual/mod.rs` — removed `pub mod capture;`, `mod reference_scene;`, `add_plugins(ReferenceScenePlugin)`, doc-comment trims (~12 lines lost)
- `src/visual/palette.rs` — two `cfg_attr(...)` blocks converted to unconditional `#[allow(dead_code, ...)]`
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — epic-3 → in-progress, 3-1 ready-for-dev → review (via in-progress); `last_updated` bumped
- `_bmad-output/implementation-artifacts/deferred-work.md` — 4 RESOLVED annotations (Story 2.1 idempotency, Story 2.2 swatch-leak, Story 2.2 over-tagging, Story 2.5 capture-cleanup) + new "Deferred from: 3-1" section (splash race re-defer, splash location debt, empty `VisualSystems::Setup`)
- `_bmad-output/implementation-artifacts/3-1-title-screen-stub-mainmenu-arena-transition.md` — Status `ready-for-dev` → `review`, task checkboxes `[x]`, Dev Agent Record populated

**Deleted (`git rm`):**
- `src/visual/capture.rs` (138 lines, 1 unit test)
- `src/visual/reference_scene.rs` (236 lines, 0 tests, debug-only)
- `.github/workflows/parity-capture.yml` (133 lines, manual `workflow_dispatch` only)

### Review Findings

- [x] [Review][Defer] SplashConfig timer stays finished on Loading re-entry [`src/splash.rs`] — deferred, pre-existing issue not caused by this diff; no re-entry path exists in 3.1
- [x] [Review][Defer] `spawn_main_menu` not idempotent on MainMenu re-entry [`src/ui/main_menu.rs:17`] — deferred, stub scope; no re-entry path in 3.1; module rewritten in Story 4.7

### Change Log

| Date | Change | By |
|---|---|---|
| 2026-04-30 | Story 3.1 implementation: title-screen stub (`src/ui/`) + M1 capture/reference-scene/parity-capture teardown; status ready-for-dev → review | claude-opus-4-7 (1M ctx) via dev-story |
| 2026-04-30 | Code review passed (0 patches, 2 deferred, 12 dismissed); status review → done | claude-sonnet-4-6 via bmad-code-review |
