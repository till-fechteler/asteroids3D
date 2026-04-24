# Story 1.7: Splash Screen Shows "asteriods3D" and Transitions to MainMenu

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player launching the game,
I want to see "asteriods3D" displayed when the app opens,
So that I immediately know the app launched and I'm in the right program.

## Acceptance Criteria

1. **Given** the app is in `GameState::Loading`
   **When** `OnEnter(GameState::Loading)` runs
   **Then** a `bevy_ui` text Node is spawned with content `"asteriods3D"`
   **And** the Node uses centered flexbox layout that scales to window size
   **And** the text entity (or its parent Node) carries a `LoadingStateEntity` marker component

2. **Given** the splash is visible
   **When** a configurable splash-duration elapses (duration loaded from a `SplashConfig` resource, default 2.0 seconds)
   **Then** the app mutates `NextState<GameState>` to `MainMenu`

3. **Given** the state transitions from `Loading` to `MainMenu`
   **When** `OnExit(GameState::Loading)` runs
   **Then** all entities tagged with `LoadingStateEntity` are despawned
   **And** no orphaned splash text remains in the hierarchy

4. **Given** the app is now in `GameState::MainMenu`
   **When** the window is inspected visually
   **Then** the splash text is gone (MainMenu UI is a later epic's responsibility — this story ends at the transition)

## Tasks / Subtasks

- [x] **Task 1: Enable `bevy_ui` features in `Cargo.toml`** (AC: #1)
  - [x] `[dependencies].bevy.features`: `["3d", "png"]` → `["3d", "png", "bevy_ui", "default_font"]`.
  - [x] `[target.'cfg(target_os = "linux")'.dependencies.bevy].features`: `["3d", "png", "x11", "wayland"]` → `["3d", "png", "x11", "wayland", "bevy_ui", "default_font"]`.
  - [x] `cargo check` exit 0, zero warnings/errors.
  - [x] **Deviation from spec prediction:** `Cargo.lock` did NOT regenerate. `grep -c 'bevy_ui\|bevy_text' Cargo.lock` returned 20 pre-edit — both were already transitively pulled by `"3d"`'s feature graph, so enabling them at our crate level added no new resolved dep entries. Consequence: no CI cache invalidation, no need to commit `Cargo.lock`. `git diff` shows Cargo.toml only.

- [x] **Task 2: Create `src/splash.rs`** (AC: #1, #2, #3)
  - [x] New file `src/splash.rs`, 88 lines (69 code + tests + doc).
  - [x] `use bevy::prelude::*;` + `use crate::state::GameState;`.
  - [x] Constants `SPLASH_TEXT = "asteriods3D"` (typo preserved per spec), `SPLASH_DURATION_SECS = 2.0`.
  - [x] `#[derive(Resource)] pub struct SplashConfig { pub timer: Timer }` + `Default` impl returning `Timer::from_seconds(SPLASH_DURATION_SECS, TimerMode::Once)`.
  - [x] `#[derive(Component)] pub struct LoadingStateEntity;`.
  - [x] `spawn_splash` spawns `(Camera2d, LoadingStateEntity)` and the centered flex Node with `(Text::new, TextFont { font_size: 64.0 }, TextColor::WHITE)` child.
  - [x] `tick_splash_timer` ticks on `Time::delta()`, logs `splash timer elapsed, transitioning to MainMenu` on `just_finished()`, sets `NextState(MainMenu)`.
  - [x] `cleanup_loading_entities` iterates `Query<Entity, With<LoadingStateEntity>>` and `despawn()`s each.
  - [x] `#[cfg(test)] mod tests` with `splash_config_default_is_two_seconds` — `duration() == 2.0s`, `mode() == Once`.
  - [x] `//!` doc 2 lines, no story-id.

- [x] **Task 3: Add `log_mainmenu_entered` to `src/state.rs`** (AC: #2)
  - [x] Appended `pub fn log_mainmenu_entered() { info!("entered MainMenu"); }` after `log_loading_entered`.
  - [x] `#[expect(dead_code, reason = "...")]` on `GameState` preserved — `MainMenu` becomes live in this story via `NextState::set`, but 5 other variants remain unused → expectation still valid.

- [x] **Task 4: Wire splash + state-log systems into `src/main.rs`** (AC: #1, #2, #3, #4)
  - [x] `mod splash; mod state;` — two module declarations.
  - [x] Imports: `use splash::{SplashConfig, cleanup_loading_entities, spawn_splash, tick_splash_timer};` + `use state::{GameState, log_loading_entered, log_mainmenu_entered};` (rustfmt re-sorted alphabetically, uppercase-first, as predicted).
  - [x] `.init_state::<GameState>()` + `.init_resource::<SplashConfig>()`.
  - [x] `OnEnter(Loading)` tuple `(log_loading_entered, spawn_splash)`.
  - [x] `OnEnter(MainMenu)` registers `log_mainmenu_entered`.
  - [x] `Update` registers `tick_splash_timer.run_if(in_state(GameState::Loading))`.
  - [x] `OnExit(Loading)` registers `cleanup_loading_entities`.
  - [x] `fn main() -> AppExit` signature preserved from 1.6. `.run()` trailing expression, no `;`.
  - [x] `//!` doc updated to 2 lines: "asteroids3D — app entry point." / "Registers DefaultPlugins, GameState, and the Loading → MainMenu splash flow."

- [x] **Task 5: Local verification sweep** (AC: #1, #2, #3, #4)
  - [x] `cargo check` — `grep -E 'warning:|error:' /tmp/story-1-7-check.log` → **0** hits.
  - [x] `cargo build` — same → **0** hits. (Fast: incremental from `cargo check`, 0.97s.)
  - [x] `cargo test` — **2 passed, 0 failed** (`state::tests::default_state_is_loading`, `splash::tests::splash_config_default_is_two_seconds`). `grep -E 'warning:|error:|FAILED' /tmp/story-1-7-test.log` → **0** hits.
  - [x] `cargo clippy --all-targets -- -D warnings` → **0** hits.
  - [x] `cargo fmt --all -- --check` — initial run diffed `use` ordering in `main.rs`; applied `cargo fmt --all`; re-check exit 0.
  - [x] `cargo run &` in background; window opened; splash visible ~2 s; transitioned; window later closed by user.
  - [x] `grep 'entered Loading' /tmp/story-1-7-run.log` → **1** hit.
  - [x] `grep 'splash timer elapsed' /tmp/story-1-7-run.log` → **1** hit (at T+2.01 s from Loading entry).
  - [x] `grep 'entered MainMenu' /tmp/story-1-7-run.log` → **1** hit (same frame as splash-timer-elapsed log).
  - [x] `grep -E 'AdapterInfo|backend:' /tmp/story-1-7-run.log` → **1** hit, `backend: Metal` on Apple M5 Pro (parity with 1.5/1.6).
  - [x] Line counts + timestamps captured in Debug Log References below.

- [x] **Task 6: Scope guardrails — verify nothing else drifted** (AC: #1, #2, #3, #4)
  - [x] `git status --short`: only `Cargo.toml` (M), `src/main.rs` (M), `src/state.rs` (M), `src/splash.rs` (??), plus bookkeeping `sprint-status.yaml` (M) and this story file (??). `Cargo.lock` NOT modified (see Task 1 deviation note).
  - [x] `grep -nrE 'Arena|Caravan|PostRun|PhotoMode|Paused' src/ --include='*.rs' | grep -v 'state.rs'` → **0** hits.
  - [x] `grep -rn 'tracing_subscriber\|directories::\|panic::set_hook' src/` → **0** hits. Story 1.8 owns those.
  - [x] `grep -rn 'string.table\|en\.ron\|strings::' src/` → **0** hits. Epic 3+ owns string-table loader.
  - [x] `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` untouched.

- [x] **Task 7: Commit + CI observation** (AC: #1, #2, #3, #4)
  - [x] Staged: `Cargo.toml`, `src/main.rs`, `src/state.rs`, `src/splash.rs` (no `Cargo.lock` to stage — see Task 1 deviation).
  - [x] Commit `8914284`: `feat: splash screen — bevy_ui text + Loading → MainMenu transition (Story 1.7)`. Single-line, no `Co-Authored-By` trailer.
  - [x] Pushed `8e45679..8914284` to `origin/master`.
  - [x] CI run `24882136265` — all 4 jobs ✅: msrv-check (1m11s), macos (2m08s), ubuntu (3m46s), windows (10m43s). Total wall ≈ 10m47s. Cache warm because Cargo.lock unchanged.
  - [x] `gh run view 24882136265 --log | grep -cE 'warning:|error:'` → **0**.
  - [x] BMad bookkeeping commit will follow on Step 9 (story → review + sprint-status flip).

## Dev Notes

### Why this story exists

Story 1.7 completes Epic 1's M0 completion criterion: **`cargo run` opens a window showing "asteriods3D" on all three platforms, with CI passing**. [Source: architecture.md:994] Stories 1.1–1.6 compile the project, pin plugins, configure toolchain, run 3-OS CI, open a window, and install the `States` skeleton — but the window is still blank default-Bevy. Story 1.7 puts the project's name on screen and exercises the first `Loading → MainMenu` state transition, proving the `States` backbone from 1.6 works end-to-end.

It also installs the **first bevy_ui surface in the project**. Every subsequent UI story (FR36 title screen, FR11 HUD, FR38 post-run summary, FR41 photo mode) builds on the same pipeline. Getting Cargo features + Camera2d + Node/Text layout + state-scoped cleanup right once here removes the same ambiguity from every later UI story. [Source: architecture.md:226-230 hybrid HUD strategy; architecture.md:254 menu system]

### Context inherited from Stories 1.1–1.6

| Fact | Value | Source |
|---|---|---|
| Rust toolchain | `1.94.1` (stable, pinned) | `rust-toolchain.toml` |
| MSRV | `1.89` (CI-verified) | `Cargo.toml:5` |
| Bevy | `0.18` (resolved `0.18.1`) | `Cargo.toml:8` |
| Current `src/main.rs` body | 16 lines: module doc + `use bevy::prelude::*;` + `mod state;` + `use state::{...};` + `fn main() -> AppExit { App::new().add_plugins(DefaultPlugins).init_state::<GameState>().add_systems(OnEnter(GameState::Loading), log_loading_entered).run() }` | Post-1.6 |
| Current `src/state.rs` body | 32 lines: `GameState` enum (7 variants, `#[default]` on `Loading`, `#[expect(dead_code, reason = "...")]`) + `log_loading_entered` + 1 unit test | Post-1.6 |
| Tests in project | 1 (`state::tests::default_state_is_loading`) | Post-1.6 |
| CI workflow | 3-OS build matrix + msrv-check, all 4 green on `19ed03c` | `.github/workflows/ci.yml` |
| Commit convention | Single-line subject; `feat:` / `chore:` / `docs:` / `fix:` / `ci:` / `bmad:` prefixes; NO `Co-Authored-By` trailer | `git log --oneline -n 15` |
| Local dev machine | macOS 26.4.1 / arm64 (Apple M5 Pro) | Prior story Debug Logs |
| `cargo run` window behavior | Opens black window on Metal backend; closes cleanly; emits `bevy_winit Destroyed for unknown winit Window Id` WARN on close (known Bevy 0.18 winit race, deferred-work.md) | Story 1.6 `/tmp/story-1-6-run.log` |

### Bevy 0.18 UI pipeline — what the dev agent must know

**`bevy_ui` is a Cargo feature, not auto-included.** With `default-features = false` (Story 1.1's pin), the Bevy features we get are *only* what we list explicitly. Current list `["3d", "png"]` deliberately omits UI because prior stories didn't need it. Story 1.7 needs it → must add.

- Feature `bevy_ui`: enables the `bevy_ui` crate + `UiPlugin` in `DefaultPlugins`. Required for `Node`, `Val`, `JustifyContent`, `AlignItems`, `Camera2d`'s UI rendering path.
- Feature `default_font`: embeds Fira Sans Regular as a compile-time fallback. Without it, `Text::new(...)` renders nothing (no font, no glyphs) unless an asset font is loaded manually. For a splash screen, `default_font` avoids pulling a TTF into `assets/`.
- `bevy_text` feature is pulled transitively by `bevy_ui` in Bevy 0.18 — **probably**. If `cargo check` surfaces missing `Text`/`TextFont` types, add `"bevy_text"` explicitly. Empirical verification via `cargo check` is authoritative; the architecture doc doesn't pre-document the feature graph at this granularity.

**Component-based UI API (Bevy 0.15+ migration completed by 0.18):**
- `Node` component carries all layout styling (flex, grid, sizing, padding). It replaces `NodeBundle`.
- `Text::new("...")` creates a text component; required components auto-insert `TextFont` (default) and `TextLayout`. Explicit `TextFont { font_size, ..default() }` + `TextColor(Color::...)` customize appearance.
- Parent-child is established via `.with_children(|parent| parent.spawn(...))` or the `ChildOf` component. Use `.with_children` here — it's the most readable form.
- Despawn cascades to descendants by default since Bevy 0.16 (Relationship migration). `despawn_recursive()` is gone; plain `despawn()` is recursive. This means tagging only the parent Node with `LoadingStateEntity` suffices — the child Text is cleaned up with it.

**Camera2d for UI:** `bevy_ui` renders through a `Camera2d` (or any camera with a UI render target). Spawning `Camera2d` is sufficient; required components handle the rest. Tagging the camera with `LoadingStateEntity` scopes it to the Loading state; `OnExit(Loading)` removes it. Next-state camera will be spawned by whichever story introduces MainMenu UI (Epic 3+).

**Flexbox centering idiom:**
```rust
Node {
    width: Val::Percent(100.0),
    height: Val::Percent(100.0),
    justify_content: JustifyContent::Center,  // horizontal axis (row direction)
    align_items: AlignItems::Center,          // cross axis (column)
    ..default()
}
```
This fills the viewport and centers children in both axes. Matches AC #1's "centered flexbox layout that scales to window size."

### State-transition pattern (architecture-aligned)

Architecture.md:417-420 prescribes:
- `NextState<GameState>` mutation (never direct `State<GameState>` writes) — `tick_splash_timer` does this correctly.
- `OnEnter`/`OnExit` systems are idempotent — this story's systems are.
- State cleanup via marker component + `cleanup_on_exit::<T>`-style despawn in `OnExit` — `LoadingStateEntity` + `cleanup_loading_entities` implement this.

**Alternative considered and rejected:** `StateScoped<GameState::Loading>` (Bevy 0.18 canonical auto-despawn). Rejected because AC #1 and architecture.md:420 explicitly prescribe the marker-component + explicit-cleanup pattern. `StateScoped` could be a future refactor if the architecture is amended; not this story.

### `src/splash.rs` skeleton

The dev agent writes this near-verbatim. Rustfmt's canonical formatting is allowed to adjust whitespace, argument wrapping, import ordering, trailing commas.

```rust
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
```

### `src/state.rs` delta

Append after `log_loading_entered`:

```rust
pub fn log_mainmenu_entered() {
    info!("entered MainMenu");
}
```

Do NOT touch the `GameState` enum, its derives, or the `#[expect(dead_code, reason = "...")]` attribute. The dead-code expectation remains valid: `MainMenu` becomes live via `NextState::set(GameState::MainMenu)` in `splash.rs`, but `Arena`, `Caravan`, `PostRun`, `PhotoMode`, `Paused` remain unused. When those later stories land and exercise all variants, rustc's "lint expectation not fulfilled" warning will fire and force removal of the `#[expect]` — self-cleaning per Story 1.6 code-review patch MED-1.

### `src/main.rs` skeleton (post-edit)

```rust
//! asteroids3D — app entry point.
//! Registers DefaultPlugins, GameState, and the Loading → MainMenu splash flow.

use bevy::prelude::*;

mod splash;
mod state;

use splash::{
    cleanup_loading_entities, spawn_splash, tick_splash_timer, SplashConfig,
};
use state::{log_loading_entered, log_mainmenu_entered, GameState};

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
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
        .add_systems(OnExit(GameState::Loading), cleanup_loading_entities)
        .run()
}
```

Rustfmt is allowed to re-order the `use` items inside each `use` block (it sorts alphabetically within `{…}`). Accept its order.

### Typo preservation: "asteriods3D" vs "asteroids3D"

The Epic 1 spec (`epic-1-*.md:156`) literally reads `"asteriods3D"` — a transposed `ei` → `ie`. The *correct* brand (and Cargo package name after commit `113eebe`) is `asteroids3D`. The planning artifacts still carry the typo across `sprint-status.yaml`, `epics.md`, `architecture.md`, and others — tracked as a deferred-work chore ("fix project-name typo across planning + bookkeeping artifacts") from the Story 1.3 code review.

**This story preserves the literal typo.** Rationale:
1. The Story 1.7 AC #1 explicitly specifies `"asteriods3D"`. Dev agent follows spec literally.
2. A single-character fix here, without also fixing the rest of the typo cluster, creates inconsistency between splash text and planning artifacts.
3. The dedicated typo-rename chore will sweep `SPLASH_TEXT` together with the other occurrences in one coherent commit. That's the right place for the fix.

If Till decides to ship the correction in this story, he can change `SPLASH_TEXT` to `"asteroids3D"` in a one-line patch — but the default and spec-aligned behavior is to keep the typo.

[Source: `_bmad-output/implementation-artifacts/deferred-work.md` → "Deferred from: code review of 1-3" → `project: asteriods3D typo`]

### NFR-L3 deferral: hard-coded string, not RON

PRD NFR-L3 (`prd.md:612`) requires *all player-facing strings* to load from a RON string-table rather than be hard-coded. Architecture.md:597 houses the loader in `src/ui/strings.rs`. That infrastructure doesn't exist in M0 — it's an Epic 3+ piece tied to FR36 (title screen copy) and the first menu story.

`SPLASH_TEXT` is a `const &str` here because (a) the string-table loader doesn't exist yet, (b) the splash text is effectively a brand identifier, arguably borderline "player-facing string" vs "compile-time brand constant", (c) the typo-rename chore will cover this string alongside all other spellings. When the RON loader lands, a principled migration would: define `splash.text` in `en.ron`, load it at startup, pass it to `spawn_splash`. That's Epic 3+ work.

No code comment is needed on `SPLASH_TEXT` noting this — the dev notes suffice for audit trail.

### Scope boundaries — what belongs to later stories

| Concern | Story that owns it |
|---|---|
| MainMenu UI (title, start/settings/credits/quit) | **Epic 3 Story 3.1** — `ui/main_menu.rs`; FR36. |
| `tracing_subscriber` init + panic hook + per-OS log file | **Story 1.8** — until then, Bevy's `LogPlugin` handles stderr. |
| RON string-table loader + `t!()` macro / lookup helper | **Epic 3+** — `ui/strings.rs`; FR-L3 satisfaction. |
| Font asset (TTF in `assets/fonts/`) | **Not needed** — `default_font` feature embeds Fira Sans. Custom fonts arrive with UI polish (Epic 10). |
| Camera lifecycle across all states (3D for Arena/Caravan, 2D for menus) | **Epic 3+** — each state spawns its own camera on `OnEnter`. |
| Refactor splash to `src/ui/splash.rs` + `UiPlugin` | **Epic 3 Story 3.1** — first mover on `src/ui/` subtree moves splash in. |
| `StateScoped<S>` migration instead of marker + cleanup | **Architectural amendment** — out of story scope. |
| Fix `asteriods3D` typo in splash + planning artifacts | **Dedicated typo-rename chore** — not Story 1.7 scope. |

### Architecture Compliance

- **`bevy_ui` + `default_font` Cargo features** — `architecture.md:294` lists "bevy_ui splash screen" as an M0 deliverable but doesn't call out the feature-flag requirement; that's an implementation detail Story 1.1's `default-features = false` choice made load-bearing. Adding `"bevy_ui"` + `"default_font"` to `Cargo.toml` is the minimal change to satisfy the architecture without drifting from Story 1.1's "only the features you need" principle. [Source: architecture.md:294 + Cargo.toml:8]
- **Centered flexbox Node + bevy_ui Text** — matches architecture.md:227 ("`bevy_ui` screen-space: Shields, Hull, Ammo, Salvage-currency, Post-run summary, Menus, Settings") and architecture.md:254 ("`bevy_ui` + States-transitions"). Splash is the first screen-space UI surface in the project. [Source: architecture.md:227,254]
- **Marker component + `OnExit` cleanup** — `LoadingStateEntity` + `cleanup_loading_entities` match architecture.md:420 ("entities spawned for a state tag themselves with a marker component … despawned by a `cleanup_on_exit::<T>` system in `OnExit`"). [Source: architecture.md:420]
- **`NextState<GameState>` mutation, no direct `State` writes** — matches architecture.md:418. [Source: architecture.md:418]
- **`OnEnter(GameState::Loading)` spawn pattern** — matches architecture.md:432-433 ("`OnEnter(GameState::Loading)`: kick off asset loads, show splash"). Asset loading is a future-story concern; showing the splash is this story. [Source: architecture.md:432-433]
- **`src/splash.rs` flat vs `src/ui/splash.rs`** — `src/ui/` subtree doesn't exist yet. Architecture.md:589-597 reserves `src/ui/main_menu.rs` for Epic 3 Story 3.1 (title screen). A flat `src/splash.rs` today, promoted into `src/ui/splash.rs` when the `UiPlugin` skeleton lands, matches the staged rollout preference (MEMORY `feedback_staged_rollout.md`) and avoids scaffolding a one-file subdirectory. **This is a deliberate, time-bounded deviation from the final file-layout target**; it will align with architecture at Epic 3 Story 3.1. No architecture amendment needed.
- **`log_mainmenu_entered` parallel to `log_loading_entered`** — diagnostic logging convention from Story 1.6. Keeps `state.rs` the home of state-entry diagnostic logs; avoids scattering `info!("entered …")` across feature modules. [Source: Story 1.6 `src/state.rs` pattern]
- **Camera2d scoped to Loading** — no cross-state camera persistence in M0. Each state manages its own camera lifecycle per the state-scoped cleanup pattern. Epic 3+ stories that introduce persistent cameras (e.g., cockpit Camera3d) will negotiate lifecycle explicitly. [Source: architecture.md:420]

### Library/Framework Requirements

| Crate | Version | Change in Story 1.7 |
|---|---|---|
| `bevy` | `0.18` | **Features extended**: `["3d", "png"]` → `["3d", "png", "bevy_ui", "default_font"]` on both `[dependencies]` and `[target.'cfg(target_os = "linux")'.dependencies.bevy]` blocks. No version change. |
| `Cargo.lock` | — | Regenerates with transitive entries pulled by new features (`bevy_ui`, `bevy_text`, `taffy`, and any font/glyph deps). Commit the lock change. |
| All other pinned deps | unchanged | `avian3d`, `bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager`, `serde`, `serde_json`, `ron`, `thiserror`, `tracing`, `tracing-subscriber`, `directories` — still unused in code (Epic 2+). |

No new top-level crate dependency added. Manifest diff is a 2-character feature-list extension per block, plus `Cargo.lock` regeneration.

### File Structure Requirements

| Path | Add/Modify | Purpose |
|---|---|---|
| `Cargo.toml` | **Modify** | Extend `bevy` features with `"bevy_ui"` + `"default_font"` on both dep blocks. ~2 lines changed. |
| `Cargo.lock` | **Modify (auto)** | Regenerated by `cargo check`. Commit. |
| `src/splash.rs` | **Add** | ~65 lines: `SplashConfig` resource + `LoadingStateEntity` marker + `spawn_splash` / `tick_splash_timer` / `cleanup_loading_entities` systems + 1 unit test. Module doc ≤2 lines. |
| `src/main.rs` | **Modify** | Add `mod splash;` + splash imports; add `log_mainmenu_entered` to state imports; `.init_resource::<SplashConfig>()`; extend `OnEnter(Loading)` to tuple; add `OnEnter(MainMenu)`, `Update (tick_splash_timer run_if in_state Loading)`, `OnExit(Loading)` registrations. Net ~10 line additions. Update `//!` doc. |
| `src/state.rs` | **Modify** | Append `pub fn log_mainmenu_entered() { info!("entered MainMenu"); }` after `log_loading_entered`. Net +3 lines. Do NOT touch the enum or its attributes. |
| `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` | **Do NOT touch** | Out of scope. |
| `docs/plugin-compatibility.md` | **Do NOT touch** | No plugin added; only Bevy features extended. |
| `_bmad-output/implementation-artifacts/deferred-work.md` | **Do NOT touch in this story** | No new resolutions. The `.claude/scheduled_tasks.lock` gitignore defer remains open. The `bevy_winit` close-WARN defer remains open (may or may not reappear in `/tmp/story-1-7-run.log`; still not a regression signal). |

**Single-binary layout reminder:** the project is still a single-binary crate with no `lib.rs`. `src/splash.rs` is a direct child module declared via `mod splash;` in `main.rs`, same as Story 1.6's `src/state.rs`. `src/lib.rs` remains a future-refactor concern (post-M1 or when cross-plugin types emerge).

### Testing Requirements

- **Unit tests:** 1 new test (`splash_config_default_is_two_seconds`) in `src/splash.rs`. Pure function, zero I/O, CI-safe. Total tests after this story: **2**.
- **Manual test:** `cargo run` on macOS — visually verify splash text appears centered for ~2s, then disappears; verify three log lines (`entered Loading`, `splash timer elapsed, transitioning to MainMenu`, `entered MainMenu`) appear in the expected order.
- **Integration tests (App construction):** **deferred**. `App::new().add_plugins(DefaultPlugins)` boots wgpu/winit and breaks headless CI runners (per Story 1.6 Dev Notes § "Why a single unit test"). Integration-test coverage for state transitions + UI cleanup arrives at the first gameplay story (Epic 3). [Source: architecture.md:144-146]
- **Windows/Linux runtime verification:** same pattern as Stories 1.5/1.6 — Till runs `cargo run` on physical hardware, confirms splash visible + log lines appear. Can be done in parallel with the commit/CI flow or deferred to a future machine-access window. CI itself verifies compile parity on 3 OSes.

### Latest Technical Information

**Bevy 0.18 UI API — stable since 0.16.** The component-based UI (Node / Text / TextFont / TextColor + required components) has been the canonical pattern since Bevy 0.15 completed the bundle→component migration. No breaking changes in 0.18 relevant to this story. [Source: Bevy 0.15 and 0.16 migration guides]

**Bevy 0.18 despawn cascading — default behavior since 0.16.** `commands.entity(e).despawn()` removes the entity AND all descendants in the Children hierarchy. `despawn_recursive()` was removed as redundant. Writing the cleanup system to iterate matching entities and despawn each is robust against either behavior (orphaned children from an earlier-despawned parent become no-ops, not panics).

**Bevy 0.18 `default_font` feature.** Embeds Fira Sans Regular at compile time; `TextFont { font_size: ..., ..default() }` uses it implicitly when the `font: Handle<Font>` field is left default. Binary-size impact: ~170 KB. Acceptable for M0; asset-loaded custom fonts arrive with polish (Epic 10).

**Bevy 0.18 `run_if(in_state(...))`** is the canonical condition for state-scoped Update systems. No version-specific gotchas.

**`Timer::from_seconds(s, TimerMode::Once)`** — a Timer that fires `.just_finished() == true` on one frame (the frame its accumulated `tick()` crosses the threshold). `TimerMode::Once` ensures subsequent ticks don't re-fire. Stable across Bevy versions.

### Previous Story Intelligence

**From Story 1.6 (just closed, commit chain `19ed03c` → `8f5223c` → `e0ed6f0` → `432af5e`):**

- **Two-commit pattern (source + bookkeeping).** Story 1.7 follows: `feat:` commit with source + manifest + lock, then `bmad:` commit with story file + sprint-status.
- **`#[expect(dead_code, reason = "...")]` on `GameState` enum** (from 1.6 review patch MED-1). Stays in place. With Story 1.7, `MainMenu` becomes live but the other 5 variants still aren't — expectation stays satisfied.
- **Exit-0 is not proof.** MEMORY `feedback_full_build_output.md` explicitly rejects "cargo check exit 0 + tail looks clean" as verification. Task 5 uses `tee` + explicit `grep` for every command — paste line counts into DAR.
- **Module doc style:** ≤2 lines, no story-id references (honors 1.5 review-patch BH8). Story 1.7's `//!` docs in `splash.rs` and updated `main.rs` follow this.
- **CI cache sensitivity.** Cargo.lock change in Story 1.7 WILL invalidate `Swatinem/rust-cache` keys on first push, causing a cold-cache CI run (~30–60 m). Subsequent pushes will re-warm. Story 1.6 skipped this because it didn't touch Cargo.lock; Story 1.7 re-incurs the cost.
- **`bevy_winit Destroyed for unknown winit Window Id` WARN on close** — known Bevy 0.18 race (deferred-work.md LOW-1 from 1.6). Do NOT treat as a Story 1.7 regression if it reappears in `/tmp/story-1-7-run.log`.
- **Rustfmt may re-order `use` items.** Story 1.6 had rustfmt re-sort `use state::{...}`. Same thing will happen here with the longer import list. Accept rustfmt's order.
- **`tracing` via Bevy's `LogPlugin`** still works with zero subscriber init. `info!(...)` calls land in stderr via the default `DefaultPlugins` LogPlugin. Story 1.8 owns the custom subscriber + file output.
- **Scope guardrails via `git status --short` + targeted `grep`.** Story 1.7 Task 6 mirrors Story 1.6 Task 6.

### Git Intelligence

Recent relevant commits (last 5):
```
8e45679 bmad: retrofix — story 1.2 status review → done (forgotten in 48cedcd)
432af5e bmad: story 1.6 review complete — 1 patch applied, 1 defer logged
e0ed6f0 chore: apply code-review patch (Story 1.6 MED-1: #[allow] → #[expect])
8f5223c bmad: story 1.6 ready-for-dev → review (CI green, deferred AppExit resolved)
19ed03c feat: GameState enum + Bevy States skeleton (Story 1.6)
```

**Patterns reinforced:**
- Single-line commit subjects, sub-70-char. Story-ID in parens at end when semantically useful.
- Prefix vocabulary: `feat:` / `chore:` / `docs:` / `fix:` / `ci:` / `bmad:`. Story 1.7 uses `feat:`.
- No `Co-Authored-By` trailer.
- BMad bookkeeping is a separate commit from source changes.
- Light code-review pattern (single reviewer, 60–120 line diffs) is acceptable — Story 1.6 ran light. Story 1.7's diff will be larger (~90–120 lines including Cargo.toml + Cargo.lock + 3 code files); light review is still likely sufficient given the file-by-file simplicity, but a full 3-agent review is discretionary.

### Project Structure Notes

Alignment with architecture.md file layout:
- `src/main.rs` — ✅ `App::new()` assembly, plugin + state registration. [architecture.md:548]
- `src/state.rs` — ✅ GameState enum + (future) AppPhase SystemSets. No drift in Story 1.7 (only appends `log_mainmenu_entered`).
- `src/splash.rs` — ⚠️ **Time-bounded deviation from architecture.md:589-597.** Architecture reserves `src/ui/` for UI surfaces; splash fits there eventually. Story 1.7 puts splash flat at `src/` because the `src/ui/` subtree doesn't exist yet. Epic 3 Story 3.1 ("Title Screen Stub — MainMenu Arena transition") introduces `src/ui/mod.rs` and is the natural time to move `splash.rs` → `src/ui/splash.rs`. This deviation is documented in-story (Architecture Compliance section) and tracked implicitly by Epic 3.1's ownership.

No conflicts with the rest of the unified project structure. Future `src/core/`, `src/tuning/`, `src/flight/`, etc. are Epic 3+ and untouched here.

### References

- [Source: `_bmad-output/planning-artifacts/epics/epic-1-foundation-plugin-compatibility-gate.md:145-170`] — Story 1.7 spec with full AC list.
- [Source: `_bmad-output/planning-artifacts/architecture.md:226-230`] — Hybrid HUD strategy (splash is screen-space bevy_ui).
- [Source: `_bmad-output/planning-artifacts/architecture.md:254`] — Menu system decision: `bevy_ui` + States-transitions.
- [Source: `_bmad-output/planning-artifacts/architecture.md:294`] — M0 deliverable: "bevy_ui splash screen."
- [Source: `_bmad-output/planning-artifacts/architecture.md:417-420`] — State-transition patterns (NextState mutation + marker-component cleanup).
- [Source: `_bmad-output/planning-artifacts/architecture.md:432-434`] — `OnEnter(GameState::Loading)` startup sequencing.
- [Source: `_bmad-output/planning-artifacts/architecture.md:548-549`] — `src/main.rs` + `src/state.rs` placement + purpose.
- [Source: `_bmad-output/planning-artifacts/architecture.md:589-597`] — Target placement for `src/ui/` subtree (Epic 3+).
- [Source: `_bmad-output/planning-artifacts/architecture.md:994`] — M0 completion criterion: `"cargo run" opens a window showing "asteriods3D"`.
- [Source: `_bmad-output/planning-artifacts/prd.md:612`] — NFR-L3: external string table. Deferred to Epic 3+.
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md`] — `asteriods3D` typo sweep (from 1.3 review); `bevy_winit` close-WARN (from 1.6 review).
- [Source: `_bmad-output/implementation-artifacts/1-6-gamestate-enum-with-bevy-states-skeleton.md`] — Prior-story conventions: commit pattern, verification discipline, scope guardrails, `#[expect(dead_code)]` continuity.
- [Source: `Cargo.toml:8,23-26`] — current Bevy feature pins (to be extended).
- [Source: `MEMORY.md → feedback_full_build_output.md`] — verification discipline: exit-0 + tail is NOT proof; grep explicitly.
- [Source: `MEMORY.md → feedback_staged_rollout.md`] — staged rollout preference (flat `src/splash.rs` now, promoted later).

## Dev Agent Record

### Agent Model Used

claude-opus-4-7[1m] (Claude Code, 1M context)

### Debug Log References

All local verification logs captured at `/tmp/story-1-7-*.log` on macOS 26.4.1 / Apple M5 Pro / arm64.

| Command | Grep pattern | Hit count | Notes |
|---|---|---|---|
| `cargo check` | `warning:\|error:` | **0** | `Checking asteroids3D v0.1.0 ... Finished` in 0.19s |
| `cargo build` | `warning:\|error:` | **0** | `Finished dev profile ... in 0.97s` (incremental from check) |
| `cargo test` | `warning:\|error:\|FAILED` | **0** | `2 passed; 0 failed; 0 ignored` — `splash::tests::splash_config_default_is_two_seconds ... ok` + `state::tests::default_state_is_loading ... ok` |
| `cargo clippy --all-targets -- -D warnings` | `warning:\|error:` | **0** | Clean post-fmt |
| `cargo fmt --all -- --check` | (exit code) | **0** | initial check flagged `use` ordering in `main.rs` — `cargo fmt --all` applied; re-check exit 0 |
| `cargo run` → `grep 'entered Loading'` | `entered Loading` | **1** | `09:19:12.252 INFO asteroids3D::state: entered Loading` |
| `cargo run` → `grep 'splash timer elapsed'` | `splash timer elapsed` | **1** | `09:19:14.261 INFO asteroids3D::splash: splash timer elapsed, transitioning to MainMenu` — Δ = 2.009s (tick-lag within 1 frame) |
| `cargo run` → `grep 'entered MainMenu'` | `entered MainMenu` | **1** | `09:19:14.270 INFO asteroids3D::state: entered MainMenu` — Δ = 9 ms from splash-elapsed (same-frame transition via `NextState::set`) |
| `cargo run` → `grep 'backend:'` | `AdapterInfo\|backend:` | **1** | `AdapterInfo { name: "Apple M5 Pro", ..., backend: Metal }` — parity with 1.5/1.6 |
| `git status --short` pre-commit | (scope) | **5** entries | `Cargo.toml` (M), `src/main.rs` (M), `src/state.rs` (M), `src/splash.rs` (??), + bookkeeping files. `Cargo.lock` NOT modified. |
| `grep -nrE 'Arena\|Caravan\|PostRun\|PhotoMode\|Paused' src/ \| grep -v state.rs` | (guardrail) | **0** | Unused variants confined to enum declaration. |
| `grep -rn 'tracing_subscriber\|directories::\|panic::set_hook' src/` | (guardrail) | **0** | Story 1.8 scope untouched. |
| `grep -rn 'string.table\|en\.ron\|strings::' src/` | (guardrail) | **0** | Epic 3+ RON-string-table scope untouched. |

**CI run `24882136265`** (push `8914284` to `origin/master`):
- 4/4 jobs ✅: `msrv-check (rust 1.89, ubuntu-latest)` 1m11s, `build (macos-latest)` 2m08s, `build (ubuntu-latest)` 3m46s, `build (windows-latest)` 10m43s.
- Total wall ≈ 10m47s. Story Dev Notes predicted ~30–60 m on cold cache; actual was fast because Cargo.lock never changed (warm cache — see Task 1 deviation).
- `gh run view 24882136265 --log | grep -cE 'warning:|error:'` → **0** across combined job logs.

**Runtime observations (non-blocking):**
- `WARN bevy_winit::state: Skipped event Destroyed for unknown winit Window Id` emitted on window close — same known Bevy 0.18 winit race logged by Story 1.6 review LOW-1. Not a 1.7 regression.
- `INFO bevy_app::terminal_ctrl_c_handler: Skipping installing Ctrl+C handler as one was already installed` — first observation in this project, emitted once at startup. Bevy's default plugin defers to the existing handler (installed by our async runtime or shell). Informational, not a defect.

### Completion Notes List

- **AC #1 — splash text Node spawned on `OnEnter(Loading)`:** `spawn_splash` creates `(Camera2d, LoadingStateEntity)` + `(Node { Percent 100% + JustifyContent::Center + AlignItems::Center }, LoadingStateEntity)` with child `(Text::new("asteriods3D"), TextFont { font_size 64 }, TextColor::WHITE)`. Centered flexbox fills the viewport; marker scopes Node + Camera to Loading.
- **AC #2 — `SplashConfig` timer → `NextState(MainMenu)` after 2.0 s:** `SplashConfig { timer: Timer::from_seconds(2.0, Once) }` initialized via `init_resource`. `tick_splash_timer` runs `in_state(Loading)`; `just_finished()` → `info!("splash timer elapsed, ...")` + `NextState::set(MainMenu)`. Runtime log confirms 2.009 s splash duration (within one 60 Hz frame tolerance).
- **AC #3 — `OnExit(Loading)` despawns `LoadingStateEntity`:** `cleanup_loading_entities` iterates `Query<Entity, With<LoadingStateEntity>>` and `despawn()`s each. Both the Node parent and Camera2d carry the marker; despawn cascades to the Text child via Bevy 0.16+ Relationship-based recursion.
- **AC #4 — MainMenu visually empty after transition:** runtime log shows `entered MainMenu` followed by no further UI system output. Window stays open (user closed manually ≈ 8 s after transition); no orphan splash text. MainMenu UI is Epic 3+.
- **Cargo.lock non-regeneration (Task 1 deviation):** Story predicted `Cargo.lock` would gain transitive entries when `bevy_ui` + `default_font` features were enabled. Empirically: both features were already transitively pulled by `"3d"`'s feature graph (verified `grep -c 'bevy_ui\|bevy_text' Cargo.lock` = 20 pre-edit). Adding them to our crate's feature list activated the bevy crate's compilation paths without expanding the resolved dep set. Consequence: no `Cargo.lock` commit, no CI cache invalidation. Side-effect: CI wall-time matched the 1.6 warm-cache run.
- **Rustfmt import re-ordering (pre-commit):** Initial `cargo fmt --all -- --check` flagged the `use` ordering — rustfmt canonical ordering is alphabetical-uppercase-first (`SplashConfig, cleanup_loading_entities, spawn_splash, tick_splash_timer`, and `GameState, log_loading_entered, log_mainmenu_entered`). Applied via `cargo fmt --all` once; accepted as-is. Matches the Story 1.6 rustfmt pattern.
- **Typo preservation:** `SPLASH_TEXT = "asteriods3D"` (transposed `ei` → `ie`) preserved per Epic 1 AC literal. The corrected brand `asteroids3D` exists in `Cargo.toml:2` (post-commit `113eebe`). Dedicated typo-rename chore (deferred-work.md → 1.3 review) will sweep all occurrences, including `SPLASH_TEXT`, in one coherent commit.
- **No new tests fail-first needed beyond existing test:** Story's unit test (`splash_config_default_is_two_seconds`) is pure-function and deterministic; no "RED → GREEN" cycle needed for a Default-value assertion. Integration tests (App construction with states) remain deferred per architecture.md:144-146 and Story 1.6's Dev Notes precedent (`App::new()` boots wgpu, breaks headless CI).
- **Windows/Linux runtime verification deferred to Till's physical hardware** per the Story 1.5/1.6 model. CI 3-OS compile parity is the gating evidence for this hobby-project cadence.
- **`.claude/scheduled_tasks.lock` gitignore defer + `bevy_winit` close-WARN defer remain open** — not this story's resolution window.

### File List

**Added:**
- `src/splash.rs` — `SplashConfig` Resource + `LoadingStateEntity` Component + `spawn_splash` / `tick_splash_timer` / `cleanup_loading_entities` systems + 1 unit test. 88 lines.

**Modified:**
- `Cargo.toml` — extended `bevy` features on both `[dependencies]` and `[target.'cfg(target_os = "linux")'.dependencies.bevy]` blocks with `"bevy_ui"` + `"default_font"`. 2 lines changed.
- `src/main.rs` — added `mod splash;`; splash imports; `log_mainmenu_entered` import; `.init_resource::<SplashConfig>()`; `OnEnter(Loading)` → tuple `(log_loading_entered, spawn_splash)`; new `OnEnter(MainMenu)`, `Update + run_if in_state(Loading)`, `OnExit(Loading)` registrations; updated `//!` doc. Net +13 lines (16 → 29).
- `src/state.rs` — appended `pub fn log_mainmenu_entered() { info!("entered MainMenu"); }` after `log_loading_entered`. Net +4 lines. `GameState` enum + `#[expect(dead_code, reason = "...")]` untouched.
- `_bmad-output/implementation-artifacts/1-7-splash-screen-shows-asteriods3d-and-transitions-to-mainmenu.md` — this file: Tasks checked, Dev Agent Record / Completion Notes / File List populated, Status → review.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status flip `ready-for-dev → in-progress → review` + `last_updated` bump (staged for bookkeeping commit).

**Untouched (guardrail):** `Cargo.lock`, `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `docs/plugin-compatibility.md`, `_bmad-output/implementation-artifacts/deferred-work.md`.

## Change Log

| Date | Author | Change |
|---|---|---|
| 2026-04-24 | claude-opus-4-7 (create-story) | Story 1.7 drafted. Scope: `src/splash.rs` new file (SplashConfig + LoadingStateEntity + 3 systems + 1 unit test); `src/state.rs` appends `log_mainmenu_entered`; `src/main.rs` wires `init_resource`, `OnEnter(MainMenu)`, `Update` (timer run_if in_state Loading), `OnExit(Loading)` cleanup; `Cargo.toml` extends bevy features to include `"bevy_ui"` + `"default_font"` on both dep blocks; `Cargo.lock` regenerates. M0 completion criterion (`cargo run` shows "asteriods3D" → transitions to blank MainMenu) satisfied. `asteriods3D` typo preserved per epic spec — swept by dedicated chore story. Status: ready-for-dev. |
| 2026-04-24 | claude-opus-4-7 (dev-story) | Story 1.7 implemented. Source commit `8914284` — `src/splash.rs` added (88 lines), `src/main.rs` modified (+13 lines), `src/state.rs` modified (+4 lines), `Cargo.toml` features extended. One notable deviation: `Cargo.lock` did NOT regenerate — `bevy_ui` + `default_font` were already transitively pulled by `"3d"`'s feature graph, so the features activated compilation paths without expanding the resolved dep set. Side-effect: CI cache stayed warm, total wall ≈ 10m47s (vs. Story Dev Notes prediction of 30–60m cold-cache). Rustfmt auto-reordered `use` import lists alphabetically (anticipated, accepted). All 4 CI jobs green (run `24882136265`, 0 warning/error across full log). Second project unit test added (total: 2 passing). Runtime verification on macOS: 2.009 s splash duration, backend: Metal confirmed. Known `bevy_winit` close-WARN reappeared (1.6 defer; not a regression). Status: review. |
