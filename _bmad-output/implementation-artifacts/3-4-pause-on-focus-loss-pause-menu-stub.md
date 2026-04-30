# Story 3.4: Pause on Focus Loss + Pause Menu Stub

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player flying in the Arena,
I want the simulation to halt when I Alt-Tab away or press Escape and to resume when I return focus or press Escape again,
So that the world never advances behind my back (no invisible damage, no drift while I'm distracted) and I have a deliberate, recoverable pause anchor for FR43 — six stories before flight, weapons, or HUD even exist that could harm me.

## Acceptance Criteria

1. **Given** Story 3.2's `ArenaPlugin` and Story 3.3's `spawn_arena_zone` are live (Arena state spawns asteroids + DirectionalLight + stand-in Camera3d on entry; no `PlayerShip` yet — that arrives in 3.5)
   **When** Story 3.4 lands
   **Then** a new file `src/pause/mod.rs` is authored with a `PausePlugin: Plugin` type
   **And** the plugin declares a `PausedFrom(pub crate::state::GameState)` `Resource` (newtype wrapping the state we paused from, used to know where to resume)
   **And** the plugin declares a `PauseOverlayEntity` unit-struct `Component` marker (tags the "PAUSED" overlay UI nodes for `cleanup_on_exit::<PauseOverlayEntity>`-style despawn on resume)
   **And** the plugin declares a `PauseSystems` `SystemSet` enum with at least one variant (`Detect`, for the focus + Esc detection systems running in `Update`)
   **And** `PausePlugin` is registered in `src/main.rs` via `App::add_plugins(PausePlugin)`, placed AFTER `ArenaPlugin` registration and BEFORE `init_resource::<SplashConfig>()` (mirrors 3.2's plugin-registration ordering precedent)
   **And** `mod pause;` + `use pause::PausePlugin;` are added near the other top-level `mod`/`use` lines in `main.rs` (rustfmt may reorder; accept its order)

2. **Given** the app is in `GameState::Arena` and the OS window currently has focus
   **When** `bevy_window::WindowFocused { focused: false, .. }` arrives (sent by winit on Alt-Tab, click-out-of-window, OS notification taking focus, etc.)
   **Then** a system named `pause_on_focus_loss` (registered on `Update` inside `PauseSystems::Detect`, gated `.run_if(in_state(GameState::Arena))`) reads the message via `MessageReader<WindowFocused>` (Bevy 0.18: `WindowFocused` is `#[derive(Message)]` — see Five-key constraint #2 below)
   **And** filters to events where `focused == false` (focus-gain events on the same frame are handled by AC #3, not AC #2)
   **And** inserts `PausedFrom(GameState::Arena)` as a Resource (overwriting any previous `PausedFrom`)
   **And** sets `NextState<GameState>` to `Paused`
   **And** does **NOT** spawn the "PAUSED" overlay (focus-loss is a silent pause; the overlay is reserved for the user-initiated Esc-pause path per AC #4)

3. **Given** the app is in `GameState::Paused` after focus-loss (i.e., `PausedFrom == Arena` was set by AC #2's path; no `PauseOverlayEntity` exists)
   **When** `WindowFocused { focused: true, .. }` arrives
   **Then** a system named `resume_on_focus_gain` (registered on `Update` inside `PauseSystems::Detect`, gated `.run_if(in_state(GameState::Paused))`) reads the focus-gain message
   **And** sets `NextState<GameState>` to `PausedFrom.0` (the captured pre-pause state — currently always `Arena`, but written generically so future Caravan/Combat states resume correctly)
   **And** does NOT touch any `PauseOverlayEntity` query (none exist on the focus-loss path; if one DOES exist because the user also pressed Esc while unfocused, AC #5's `OnExit(Paused)` cleanup catches it)

4. **Given** the app is in `GameState::Arena`
   **When** the player presses Escape (`KeyCode::Escape` via `Res<ButtonInput<KeyCode>>::just_pressed`)
   **Then** a system named `toggle_pause_on_escape` (registered on `Update` inside `PauseSystems::Detect`, gated `.run_if(in_state(GameState::Arena).or(in_state(GameState::Paused)))`) detects the press
   **And** when entering pause: inserts `PausedFrom(GameState::Arena)`, sets `NextState<GameState>` to `Paused`
   **And** an `OnEnter(GameState::Paused)` system named `spawn_pause_overlay_if_user_initiated` runs **conditionally** (see Dev Notes "User-initiated vs focus-loss pause overlay decision"): the system reads a `PauseInitiator { kind: PauseInitiatorKind }` resource set by the trigger system; only the `User` (Esc) variant spawns the overlay
   **And** when triggered by user (Esc): a `bevy_ui` text Node with content `"PAUSED — Esc to resume"` is spawned with the `PauseOverlayEntity` marker, centered on the screen, plus a `Camera2d` (also tagged `PauseOverlayEntity`) so the overlay renders ABOVE the Arena's `Camera3d` (Camera2d default order is `0`; explicit `Camera { order: 1, ..default() }` ensures the overlay renders on top of the 3D scene without colliding with 3.3's Camera3d at default order `0`)

5. **Given** the app is in `GameState::Paused` and the overlay is visible (Esc-initiated path; `PauseInitiator::User` is the resource value)
   **When** the player presses Escape again
   **Then** `toggle_pause_on_escape` (still gated on `in_state(Arena).or(in_state(Paused))`) detects the press while in `Paused`
   **And** sets `NextState<GameState>` to `PausedFrom.0`
   **And** `OnExit(GameState::Paused)` runs `cleanup_pause_overlay` (a `crate::arena::cleanup_on_exit::<PauseOverlayEntity>` registration — re-uses the generic from Story 3.2; matches Stories 3.4/3.11 expected pattern from 3.2 Dev Notes line 92)
   **And** all `PauseOverlayEntity`-marked entities (the text Node + the order-1 Camera2d) are despawned
   **And** Time clocks resume per AC #7

6. **Given** Time pausing is required by FR43 ("Game pauses the simulation")
   **When** `OnEnter(GameState::Paused)` runs
   **Then** a system named `pause_simulation_clocks` calls `time_virtual.pause()` on `ResMut<Time<Virtual>>` (Bevy 0.18 idiom — `Time<Virtual>` is the user-controllable clock that drives `Time<Fixed>`, which Avian's physics schedule consumes; pausing `Time<Virtual>` halts physics, animations, particle systems, and any other time-driven gameplay system in one call)
   **And** the same system **also** calls `time_physics.pause()` on `ResMut<Time<avian3d::prelude::Physics>>` (Avian's own physics clock, independent of `Time<Virtual>`; redundant when `Time<Virtual>` is already paused but defensive against future code paths that may bypass `Time<Virtual>` — Avian 0.6 explicitly documents this as the canonical physics-pause API at `avian3d::schedule::time:40-52`)
   **And** an `OnExit(GameState::Paused)` system named `resume_simulation_clocks` calls the inverse `unpause()` on both clocks

7. **Given** the player is in `GameState::MainMenu` or `GameState::Loading` (states OUTSIDE active gameplay)
   **When** the player presses Escape OR the window loses focus
   **Then** **NO** transition to `Paused` occurs (the `run_if(in_state(...))` gates on `pause_on_focus_loss` and `toggle_pause_on_escape` exclude `MainMenu` and `Loading`)
   **And** Esc in MainMenu has no effect in this story (Story 4.7's full title-screen rewrite owns the "Esc = back to title" semantics for the post-MVP polish; 3.4 is scoped to Arena pause only)
   **And** focus-loss in Loading or MainMenu has no effect (splash timer keeps counting; main-menu Enter input still works on focus regain)
   **Note:** When the `Paused` state grows additional source states later (`Caravan`, `Combat`-pocket), the run-condition gate should be widened — flagged as a forward-compat note in deferred-work.md, not a 3.4 implementation concern

8. **Given** the post-3.3 source baseline (test count = 19; `cargo build --release` 0 warnings; main.rs ~57 lines; `src/arena/mod.rs` ~37 lines; no `src/pause/` directory)
   **When** Story 3.4 verification runs locally (per `feedback_full_build_output.md` — exit-0 + tail is NOT proof; grep explicitly)
   **Then** **all six** of `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --release` produce **0** lines matching `grep -cE 'warning:|error:'` per their respective full output logs
   **And** `cargo test` summary line reads exactly `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N ≥ 19** (baseline; new tests for `PauseInitiator` enum / `PausedFrom` newtype / overlay-text-content invariant are optional but encouraged per the Story 3.3 testing precedent — see Dev Notes "Test policy")
   **And** `cargo run` (with `RUST_LOG=info,wgpu=warn,naga=warn,avian3d=info` or similar) opens a window, transitions Loading → MainMenu → Arena, AND in Arena the player can: (a) press Esc → "PAUSED — Esc to resume" appears, asteroids stop spinning if they were dynamic (in 3.4 they're static, so no visible-motion test until 3.5 PlayerShip lands; the `Time<Virtual>::is_paused()` log line is the proxy evidence — see runtime smoke below), Esc again → overlay vanishes, back to Arena; (b) Alt-Tab away → no overlay (silent pause), Alt-Tab back → no transition glitch, Arena resumes
   **And** `/tmp/story-3-4-run.log` contains exactly: 1 occurrence of `entered Loading`, `entered MainMenu`, `entered Arena`; ≥ 1 occurrence of `entered Paused` and `exited Paused` (or `info!` equivalents the dev chooses — see Dev Notes "Logging discipline"); 0 occurrences of `panic`, `backtrace`, `FATAL`, or any new `ERROR`-level logs from Bevy/Avian/wgpu beyond known noise (splash-cleanup race per deferred-work.md:75-76, :137; winit `Skipped event Destroyed` per 1.6 deferred-work LOW-1; pre-existing wgpu fragment-output warning per 3.3 dev-log entry)
   **And** `git status --short` final set is **exactly**: `src/main.rs` (M — `mod pause;` + `use pause::PausePlugin;` + `add_plugins(PausePlugin)`), `src/pause/mod.rs` (?? — new file), `_bmad-output/implementation-artifacts/sprint-status.yaml` (M — bookkeeping), `_bmad-output/implementation-artifacts/3-4-pause-on-focus-loss-pause-menu-stub.md` (M — this file's Status flip + Dev Agent Record), `_bmad-output/implementation-artifacts/deferred-work.md` (M — at minimum the "widen pause source-state gate when Caravan/Combat states arrive" forward-compat entry); **NO** entries under `Cargo.toml`, `Cargo.lock`, `src/state.rs`, `src/splash.rs`, `src/logging.rs`, `src/ui/**`, `src/tuning/**`, `src/visual/**`, `src/arena/**`, `assets/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`

9. **Given** Story 3.2's `cleanup_on_exit::<T: Component>` generic at `src/arena/mod.rs:32-36` was deliberately written to serve future state-marker users (3.4 `PauseOverlayEntity`, 3.11 `HudEntity`, 4.7 `TitleEntity`) per Story 3.2 Dev Notes line 92
   **When** Story 3.4 wires `PauseOverlayEntity` cleanup
   **Then** the registration uses the existing generic — `cleanup_on_exit::<PauseOverlayEntity>` imported via `use crate::arena::cleanup_on_exit;` — **NOT** a new copy of the despawn loop in `src/pause/mod.rs`
   **And** the architectural "generic-cleanup home" question (currently `src/arena/`, possibly should move to `src/core/cleanup.rs` per architecture.md:550 if a third+ consumer arrives) is **left untouched** in 3.4 — moving it is a refactor with cross-file blast radius and belongs to its own dedicated chore (per Story 3.2 Dev Notes line 92's YAGNI deferral). Story 3.4 is the second consumer; the third (3.11 HUD) will hit the move-or-stay decision point.

## Tasks / Subtasks

- [x] **Task 1: Author `src/pause/mod.rs` — PausePlugin + focus-loss + Esc systems + Time pause/resume + overlay** (AC: #1, #2, #3, #4, #5, #6, #9)
  - [x] Create `src/pause/` directory at the repo root (sibling of `src/arena/`).
  - [x] Create `src/pause/mod.rs`. Target file size: **~150–200 lines** including module doc, plugin impl, 4 systems (focus-loss detect, focus-gain detect, Esc toggle, OnEnter overlay-spawn), 2 OnEnter/OnExit time-clock systems, optional unit tests. Comment density per `karpathy-guidelines.md` — only WHY-comments where invariants are non-obvious.
  - [x] Module doc 2 lines max, no story-id references (per Story 1.5 review patch BH8 + Story 3.2 patch precedent — see commit `5134b3c`).
  - [x] **Imports:**
    ```rust
    use avian3d::prelude::Physics;
    use bevy::prelude::*;
    use bevy::window::WindowFocused;

    use crate::arena::cleanup_on_exit;
    use crate::state::GameState;
    ```
    Avoid wildcard imports beyond `bevy::prelude::*`. `Physics` is the Avian time-clock marker type, NOT the `PhysicsPlugins` type; double-check the import landed at `avian3d::prelude::Physics` (not `avian3d::schedule::time::Physics` — the prelude re-exports it per `avian3d/src/lib.rs:550-555`).
  - [x] **Plugin skeleton:**
    ```rust
    pub struct PausePlugin;

    #[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum PauseSystems {
        Detect,
    }

    #[derive(Resource, Debug, Clone, Copy)]
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
                (pause_simulation_clocks, spawn_pause_overlay_if_user_initiated).chain(),
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
    ```
    - **Why a `PauseSystems::Detect` set:** mirrors 3.2/3.3 `<Feature>Systems` per-plugin idiom. Future stories may add a `PauseSystems::Render` (e.g., dim-the-screen post-process when paused) or `PauseSystems::Input` (separate from Detect for unpause-only inputs). Story 3.4 declares the set but uses only one variant.
    - **Why `.chain()` on the `OnEnter(Paused)` tuple:** `pause_simulation_clocks` MUST run before `spawn_pause_overlay_if_user_initiated` because the overlay-spawn system reads the (then-frozen) `Time<Virtual>` to ensure overlay text doesn't get a stale-time-based animation seed. Practically, both are one-shot OnEnter systems and order is barely observable, but the `.chain()` makes the intent explicit and matches architecture.md:415 "order by SystemSet, never by `.after(specific_function)`" (configure_sets-style chaining is the architectural-ly approved version of `.after()` for tuples).
    - **Why no `.chain()` on the `OnExit(Paused)` tuple:** despawn order doesn't matter (the overlay entities are independent of the time clocks); no chain keeps the registration shorter.
  - [x] **System: `pause_on_focus_loss`** (AC: #2)
    ```rust
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
                return; // first focus-loss this frame is enough; ignore any siblings
            }
        }
    }
    ```
    - **`MessageReader<WindowFocused>` NOT `EventReader`:** Bevy 0.18 split events into `Event` (entity-targeted, observer-style) and `Message` (buffered, reader-style). `WindowFocused` derives `#[derive(Message)]` per `bevy_window-0.18.1/src/event.rs:287`. Using `EventReader<WindowFocused>` will produce `the trait Event is not implemented for WindowFocused` on `cargo check`. See Five-key constraint #2 below.
    - **Why `commands.insert_resource(...)` not `ResMut<PausedFrom>`:** the resource may not exist yet (first pause of the run). `commands.insert_resource(...)` creates-or-overwrites in one call; using `ResMut<PausedFrom>` would force `init_resource` at plugin build with a default value, polluting the resource lifecycle.
    - **`event.focused == false` filter:** a single `WindowFocused` message can appear in either polarity in the same frame (Alt-Tab away → away → back can produce 2-3 events on macOS with rapid keystrokes). The filter ensures only "lost focus" events trigger pause; gain-events on the same frame are ignored here and handled by `resume_on_focus_gain` (which is gated to `in_state(Paused)` — only runs after the pause transition commits).
    - **`return` after first hit:** prevents redundant resource insertions / state transitions if multiple WindowFocused messages of the same polarity arrive in one frame (rare but possible).
  - [x] **System: `resume_on_focus_gain`** (AC: #3)
    ```rust
    pub fn resume_on_focus_gain(
        mut events: MessageReader<WindowFocused>,
        paused_from: Option<Res<PausedFrom>>,
        mut next_state: ResMut<NextState<GameState>>,
    ) {
        for event in events.read() {
            if event.focused {
                let target = paused_from.as_deref().map_or(GameState::Arena, |p| p.0);
                next_state.set(target);
                info!("resumed from focus gain → {:?}", target);
                return;
            }
        }
    }
    ```
    - **`Option<Res<PausedFrom>>`:** if for some reason the resource was never inserted (e.g., manual state-machine override), default to `Arena`. Belt-and-suspenders against a hypothetical race.
    - **`paused_from.as_deref().map_or(...)`:** dereferences the `Res` to access the inner `PausedFrom`, then handles the `None` case via `map_or`. The `.0` accesses the newtype wrapping the GameState.
  - [x] **System: `toggle_pause_on_escape`** (AC: #4, #5)
    ```rust
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
                let target = paused_from.as_deref().map_or(GameState::Arena, |p| p.0);
                next_state.set(target);
                info!("resumed via Escape → {:?}", target);
            }
            _ => { /* gated upstream; defensive no-op */ }
        }
    }
    ```
    - **`Res<State<GameState>>` (NOT `ResMut<NextState>` only):** we need to read which side of the toggle to take. `State<GameState>::get()` returns the current authoritative state.
    - **`_ => { /* defensive no-op */ }`:** the `run_if` upstream already excludes other states; the match arm is a safety belt against future code paths that bypass `run_if` (e.g., direct `App::add_systems` registration without the gate).
  - [x] **System: `pause_simulation_clocks`** (AC: #6)
    ```rust
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
    ```
    - **Why both `Time<Virtual>` AND `Time<Physics>`:** `Time<Virtual>::pause()` halts Bevy's main clock (animations, splash timer, anything reading `Time::delta()`), and indirectly halts Avian because `Time<Fixed>` advances from `Time<Virtual>`. **However**, Avian's own `Time<Physics>` has an independent `paused` flag (per `avian3d/src/schedule/time.rs:124`) checked at the top of every physics step — pausing it explicitly is the canonical Avian 0.6 idiom (verbatim from the rustdoc example at `time.rs:40-52`). Calling both is redundant under the current scheduling but defensive against (a) future Avian releases that decouple `Time<Physics>` from `Time<Virtual>`, (b) future Bevy releases that change `FixedUpdate`'s clock source, (c) any future story that adds a non-physics simulation system (e.g., particle pool with a custom `Time<MyClock>`) that the architectural-design doc says should pause too. Three-line cost; eliminates a class of regression.
    - **`use avian3d::prelude::Physics;`** must land at the top of `src/pause/mod.rs`. The Avian re-export is at `avian3d/src/lib.rs:555` (`pub use ... PhysicsTime, ...`) which re-exports `Physics` indirectly — verify the prelude re-export covers `Physics` or import as `avian3d::schedule::time::Physics` if not.
  - [x] **System: `resume_simulation_clocks`** (AC: #5, #6)
    ```rust
    pub fn resume_simulation_clocks(
        mut time_virtual: ResMut<Time<Virtual>>,
        mut time_physics: ResMut<Time<Physics>>,
    ) {
        time_virtual.unpause();
        time_physics.unpause();
        info!("simulation clocks resumed");
    }
    ```
  - [x] **System: `spawn_pause_overlay_if_user_initiated`** (AC: #4)
    ```rust
    pub fn spawn_pause_overlay_if_user_initiated(
        mut commands: Commands,
        initiator: Option<Res<PauseInitiator>>,
    ) {
        if initiator.as_deref().copied() != Some(PauseInitiator::User) {
            return; // focus-loss path silently pauses; no overlay
        }
        commands.spawn((
            Camera2d,
            Camera {
                order: 1, // render above the Arena Camera3d (default order 0)
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
    ```
    - **Why explicit `Camera { order: 1 }`:** Story 3.3's Arena `Camera3d` uses default `order: 0`. A Camera2d at default `order: 0` would produce Bevy's "ambiguous camera order" runtime warning per frame. Setting `order: 1` puts the UI camera above the 3D scene without ambiguity. This is the same pattern Story 2.2's `spawn_palette_swatches` used for the swatch overlay (deferred-work.md:89 references the precedent).
    - **Marker-only-on-roots (per Story 3.1 cleanup_main_menu pattern + Story 3.2 cleanup_main_menu canonicalization):** the child `Text` node does NOT carry `PauseOverlayEntity`. Bevy 0.18's `ChildOf` linked-despawn cascades from the parent root Node despawn. Mirrors the splash-cleanup-race fix-path (deferred-work.md:75-76 option (b)).
    - **Why `with_children` not flat:** the `Text` is rendered as a child of the centering Node so it picks up the Node's `JustifyContent::Center` + `AlignItems::Center`. Flat sibling spawn would require a separate full-screen-positioned Text Node.
  - [x] **Optional unit tests** (in-file `#[cfg(test)] mod tests`):
    - `pause_initiator_default_unset`: assert that `App::new().init_state::<GameState>().add_plugins(PausePlugin).update();` does NOT inject a default `PauseInitiator` (it must remain absent until a pause-trigger system runs). Guards against an accidental `init_resource` regression that would always run the user-initiated overlay path.
    - `paused_from_carries_state`: `let p = PausedFrom(GameState::Arena); assert_eq!(p.0, GameState::Arena);` — trivial but asserts the newtype shape doesn't drift.
    - **Test budget:** 0–2 new tests. Post-3.4 expected count: **N ≥ 19** (baseline 19 + 0–2 new). Document the chosen N in Completion Notes.
    - **Skip integration tests:** an `App::new() → run_if(...) → simulate WindowFocused message → assert state transition` integration test would exercise Story 3.4 end-to-end, but architecture.md:354 defers integration tests post-M3. Match Story 3.2's "zero new tests" + Story 3.3's "pure-data invariants only" precedent.

- [x] **Task 2: Wire PausePlugin into `src/main.rs`** (AC: #1)
  - [x] Add `mod pause;` after `mod arena;` (rustfmt may reorder; accept its order).
  - [x] Add `use pause::PausePlugin;` after `use arena::ArenaPlugin;` (rustfmt alphabetization will land it appropriately).
  - [x] Add `.add_plugins(PausePlugin)` AFTER the existing `.add_plugins(ArenaPlugin)` line, BEFORE `.init_resource::<SplashConfig>()`. Final plugin block (rustfmt-tolerant):
    ```rust
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
        .add_plugins(PausePlugin)
        .init_resource::<SplashConfig>()
        // ... existing add_systems chain unchanged below
    ```
  - [x] **Why `PausePlugin` AFTER `ArenaPlugin`:** PausePlugin imports `cleanup_on_exit` from `crate::arena` (Story 3.2's generic). Plugin registration order isn't load-bearing for symbol availability (compilation order doesn't matter), but reads naturally — Arena foundations land before pause atop them. ArenaPlugin → PausePlugin → (3.5) FlightPlugin → (3.9) CombatPlugin → (3.11) HudPlugin is the projected Epic-3 plugin-stack order.
  - [x] **Net delta to `src/main.rs`:** +3 lines (1 mod, 1 use, 1 add_plugins). File grows from ~57 to ~60 lines. Still well under any reasonable file-size budget.

- [x] **Task 3: Local verification sweep — full build + runtime smoke** (AC: #8)
  - [x] **`cargo check`:**
    ```bash
    cargo check 2>&1 | tee /tmp/story-3-4-check.log
    grep -cE 'warning:|error:' /tmp/story-3-4-check.log
    ```
    Expected: `0`. Likely failure modes: (a) `EventReader<WindowFocused>` instead of `MessageReader<WindowFocused>` — fix by importing `MessageReader` from `bevy::prelude` (already there) and changing the type; (b) `Time<Physics>` import path — try `use avian3d::prelude::Physics;` first; if that fails, fallback to `use avian3d::schedule::time::Physics;`; (c) `KeyCode::Escape` — Bevy 0.18 keeps this name; if `cargo check` complains, the Bevy migration moved it (very unlikely at 0.18.1); (d) `.run_if(in_state(...).or(in_state(...)))` — requires Bevy 0.18's `Condition::or` extension; if that ergonomic doesn't compile, expand to two separate system entries each with a single `run_if`, e.g.:
    ```rust
    toggle_pause_on_escape.run_if(in_state(GameState::Arena)),
    toggle_pause_on_escape.run_if(in_state(GameState::Paused)),
    ```
    — Bevy de-duplicates same-system registrations across run-condition variants? Confirm at `cargo check`; if duplicate-registration error, split into two named systems (`enter_pause_on_escape` for Arena gate, `exit_pause_on_escape` for Paused gate).
  - [x] **`cargo build` (debug):**
    ```bash
    cargo build 2>&1 | tee /tmp/story-3-4-build.log
    grep -cE 'warning:|error:' /tmp/story-3-4-build.log
    ```
    Expected: `0`. Build is incremental from 3.3 — bevy_window, avian3d schedule, Bevy `Time<Virtual>` all already in cache.
  - [x] **`cargo test`:**
    ```bash
    cargo test 2>&1 | tee /tmp/story-3-4-test.log
    grep -cE 'warning:|error:|FAILED' /tmp/story-3-4-test.log
    ```
    Expected: `0`. Summary line MUST read `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where `N ∈ {19, 20, 21}` per Task 1's optional-tests budget. Document chosen N in Completion Notes.
  - [x] **`cargo clippy --all-targets -- -D warnings`:**
    ```bash
    cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-3-4-clippy.log
    grep -cE 'warning:|error:' /tmp/story-3-4-clippy.log
    ```
    Expected: `0`. Particular vigilance for: (a) `clippy::needless_pass_by_value` on `Option<Res<PausedFrom>>` — if it fires, switch to `&Res<PausedFrom>` (but Bevy idiom strongly prefers `Option<Res<T>>` for non-mandatory resources; this lint should not fire); (b) `dead_code` on `PauseInitiator::FocusLoss` if the `spawn_pause_overlay_if_user_initiated` system is the only consumer and only matches `User`; the `FocusLoss` variant IS read by the equality check `!= Some(PauseInitiator::User)`, but clippy may complain — if so, the canonical fix is to add a real consumer in this story (`info!("pause initiator: {:?}", initiator)` already references both variants via Debug); (c) `clippy::cognitive_complexity` on `toggle_pause_on_escape` (4 branches across 2 states) — should not fire at this size, but if it does, split the function into `try_pause_via_escape` + `try_resume_via_escape` and gate each with a single `run_if(in_state(...))`.
  - [x] **`cargo fmt --all -- --check`:**
    ```bash
    cargo fmt --all -- --check
    echo $?
    ```
    Expected exit: `0`. If non-zero, run `cargo fmt --all` once and re-check. Rustfmt may reflow the `(pause_simulation_clocks, spawn_pause_overlay_if_user_initiated).chain()` tuple if the line gets long.
  - [x] **`cargo build --release`:**
    ```bash
    cargo build --release 2>&1 | tee /tmp/story-3-4-release.log
    grep -cE 'warning:|error:' /tmp/story-3-4-release.log
    ```
    Expected: `0`. Story 3.4 introduces no new asset paths, no `cfg_attr` removals, no LTO-sensitive code paths. The release-build sweep is just a regression check that no rustc-lint-only warnings sneak in (e.g., a `dead_code` on an unused variant).
  - [x] **`cargo run` runtime smoke — Esc-pause path:**
    ```bash
    RUST_LOG=info,wgpu=warn,naga=warn,avian3d=info cargo run 2>&1 | tee /tmp/story-3-4-run-esc.log &
    PID=$!
    sleep 4    # splash + MainMenu paint
    # Manual: focus the window, press Enter (Loading→MainMenu→Arena via 3.1's flow)
    # Manual: press Escape → expect "PAUSED — Esc to resume" overlay text visible centered
    # Manual: press Escape again → expect overlay disappears, Arena visible again
    # Manual: SIGINT or close window
    ```
  - [x] **Log-grep evidence for Esc-pause smoke:**
    ```bash
    grep -c 'entered Loading' /tmp/story-3-4-run-esc.log              # expected: 1
    grep -c 'entered MainMenu' /tmp/story-3-4-run-esc.log              # expected: 1
    grep -c 'entered Arena' /tmp/story-3-4-run-esc.log                 # expected: 1
    grep -c 'paused via Escape' /tmp/story-3-4-run-esc.log             # expected: ≥ 1
    grep -c 'resumed via Escape' /tmp/story-3-4-run-esc.log            # expected: ≥ 1
    grep -c 'simulation clocks paused' /tmp/story-3-4-run-esc.log      # expected: ≥ 1
    grep -c 'simulation clocks resumed' /tmp/story-3-4-run-esc.log     # expected: ≥ 1
    grep -c 'virtual.is_paused=true' /tmp/story-3-4-run-esc.log        # expected: ≥ 1
    grep -c 'physics.is_paused=true' /tmp/story-3-4-run-esc.log        # expected: ≥ 1
    grep -cE 'panic|backtrace|FATAL' /tmp/story-3-4-run-esc.log        # expected: 0
    grep -c 'ambiguous' /tmp/story-3-4-run-esc.log                     # expected: 0 (sentinel for camera-order conflict)
    ```
  - [x] **`cargo run` runtime smoke — focus-loss path:**
    ```bash
    RUST_LOG=info,wgpu=warn,naga=warn cargo run 2>&1 | tee /tmp/story-3-4-run-focus.log &
    PID=$!
    sleep 4
    # Manual: focus the window, press Enter → Arena
    # Manual: Cmd-Tab to another app (or click another window) → Arena window loses focus
    # Manual: verify NO "PAUSED" overlay appears in the still-visible Arena window background
    # Manual: Cmd-Tab back → focus regained, Arena resumes silently
    # Manual: SIGINT or close window
    ```
  - [x] **Log-grep evidence for focus-loss smoke:**
    ```bash
    grep -c 'paused on focus loss' /tmp/story-3-4-run-focus.log        # expected: ≥ 1
    grep -c 'resumed from focus gain' /tmp/story-3-4-run-focus.log     # expected: ≥ 1
    grep -c 'PAUSED — Esc to resume' /tmp/story-3-4-run-focus.log      # expected: 0 (focus-loss is silent — overlay text never appears)
    ```
    The third grep is load-bearing: if the focus-loss path accidentally spawned an overlay, the text would have been logged either via `info!` of the spawn, OR more likely via Bevy's text-rendering diagnostic when Tracy or `bevy_diagnostic` is enabled. The simpler check `grep -c '"PAUSED"' run-focus.log → 0` is the proxy.
  - [x] **Visual verification (manual — interactive):**
    - On Arena entry, **Esc** produces a centered "PAUSED — Esc to resume" white text on the screen. The asteroid field is still visible behind the text (transparent background). Pressing Esc again removes the text and returns to the Arena scene.
    - **Alt-Tab away** from the Arena window does NOT show "PAUSED" text. **Alt-Tab back** silently restores focus; no transition glitch, no double-overlay, no orphan UI.
    - **Pause persistence test:** press Esc to enter Paused state (overlay visible). Then Alt-Tab away (focus is now lost on a Paused state — `pause_on_focus_loss` is gated to `in_state(Arena)` only, so it does NOT re-fire). Alt-Tab back: window regains focus while still in Paused; `resume_on_focus_gain` IS gated to `in_state(Paused)`, so it WILL fire and transition to `Arena` — overlay vanishes, Arena resumes. **This is intentional:** the user-initiated Esc-pause loses to focus-gain in this story's scope; if the user wants to keep the pause through window-focus changes, Story 4.x will need a "modal pause" variant. Document this behavior in Completion Notes if observed.
  - [x] **`PausePlugin` convention check:** `grep -c 'PauseOverlayEntity' src/pause/mod.rs` → expected ≥ 4 (1 component def + 1 closure-style `cleanup_on_exit` registration + 2 spawn-site uses for Camera2d and Node = 4 minimum). `grep -rn 'cleanup_on_exit::' src/` → expected ≥ 2 (Story 3.2's `<ArenaEntity>` and Story 3.4's `<PauseOverlayEntity>` = 2 consumers, validating the Story 3.2 generic-cleanup architectural intent).

- [x] **Task 4: Update `_bmad-output/implementation-artifacts/deferred-work.md`** (AC: #7)
  - [x] **Append a NEW deferred-work entry section** under header `## Deferred from: 3-4-pause-on-focus-loss-pause-menu-stub (2026-04-30)`:
    - **"Widen pause source-state gate when Caravan/Combat states arrive"** — body: "`pause_on_focus_loss` and `toggle_pause_on_escape` are currently gated `.run_if(in_state(GameState::Arena))` (or `.or(in_state(Paused))` for the toggle). When Story 6.1+ introduces `GameState::Caravan` and Story 6.8 introduces combat-pocket states, the gate must widen to include those as valid pause-from sources. Resolution path: at the first story that adds a new gameplay state, edit `src/pause/mod.rs::PausePlugin::build` to extend the run-condition tuples. Source: Story 3.4 forward-compat note."
    - **"Pause-overlay loses to focus-gain when Esc-paused user Alt-Tabs away"** — body: "Pressing Esc enters `Paused` (overlay visible). Alt-Tab away does NOT re-fire `pause_on_focus_loss` (gated to Arena only — correct). Alt-Tab back fires `resume_on_focus_gain` (gated to Paused — by design). Net effect: the user's intentional Esc-pause is silently consumed by a focus round-trip. Acceptable for 3.4 because the only gameplay impact is "static asteroids stay static while you stay focused on a different window" — but becomes a real concern when Stories 3.5-3.10 add the PlayerShip + flight + weapons (a player who Esc-paused, Alt-Tabbed for 5 minutes, returned to find their ship continued flying because focus-gain auto-resumed = bad). Resolution path: introduce a `PauseLatch { user_initiated: bool }` flag that suppresses focus-gain auto-resume when the pause was user-initiated. Defer to Story 3.10 review (when the first 'invisible damage' scenario becomes plausible) or to a dedicated Pause-UX-pass story. Source: Story 3.4 Task 3 visual-verification 'Pause persistence test'."
    - **"Generic-cleanup home re-evaluation now triggered (3rd consumer pending)"** — body: "Story 3.2 placed `cleanup_on_exit::<T: Component>` at `src/arena/mod.rs:32-36` with the YAGNI note 'a third+ consumer arrives → consider moving to `src/core/cleanup.rs`'. Story 3.4 is the **second** consumer (`<PauseOverlayEntity>`). Story 3.11 (HUD baseline) will be the **third** consumer (`<HudEntity>`). Resolution path: at Story 3.11, the dev decides — either (a) move the generic to `src/core/cleanup.rs` per architecture.md:550, OR (b) keep it at `src/arena/mod.rs` with an explanatory note. Either is defensible. Source: Story 3.2 Dev Notes line 92 + Story 3.4 deferral."
    - **"`PauseInitiator` resource lifetime"** — body: "`PauseInitiator` is inserted on each pause-trigger but never explicitly removed; subsequent pauses overwrite it. After the first pause, `Option<Res<PauseInitiator>>` is always `Some(...)` for the rest of the run, leaking the previous pause's initiator value. Acceptable for 3.4 because the resource is read only at OnEnter(Paused), which always re-runs after a fresh insert. Resolution path: if a future story reads `PauseInitiator` from a non-OnEnter system (e.g., a 'pause UI persistence' indicator), add `commands.remove_resource::<PauseInitiator>()` in `OnExit(Paused)` to cleanly reset state. Source: Story 3.4 Task 1 Resource lifecycle review."

- [x] **Task 5: Bookkeeping — story status flip + commit + push** (AC: all)
  - [x] Populate this story file's **Dev Agent Record**: Agent Model Used, Debug Log References (per-command grep counts + log paths for both `run-esc.log` and `run-focus.log`), Completion Notes (per-AC evidence + any deviations), File List (added / modified). Section structure follows Stories 3.2 / 3.3 precedent.
  - [x] Set this story's `Status:` header → `review`.
  - [x] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - Flip `3-4-pause-on-focus-loss-pause-menu-stub: ready-for-dev` → `3-4-pause-on-focus-loss-pause-menu-stub: review` (the dev-story flips through `ready-for-dev → in-progress → review`; final state at handoff is `review`).
    - epic-3 status stays `in-progress` — fourth story; no transition.
    - Bump `last_updated:` (both top-comment line and YAML body key) to: `last_updated: YYYY-MM-DD (Story 3.4 ready-for-dev → review — pause on focus loss + Esc + overlay)`.
    - YAML parse verification: `python3 -c "import yaml; yaml.safe_load(open('_bmad-output/implementation-artifacts/sprint-status.yaml')); print('OK')"` → expected `OK`. Falls back to `ruby -ryaml -e "YAML.load_file(...)"` if PyYAML unavailable (Story 2.6 precedent).
  - [ ] **Commit 1 (source — triggers CI):** stage `src/main.rs`, `src/pause/mod.rs`. **NO** `_bmad-output/**` files in this commit. *(Awaits Till's authorization per project convention; not auto-executed by the dev agent.)*
    - HEREDOC commit message subject (≤ 70 chars): `feat: pause on focus loss + Esc menu stub (Story 3.4)`. Literal length: 51 chars.
    - Push to `origin/master`. Triggers full 4-job `ci.yml` matrix.
    - **Expected CI outcome:** all 4 jobs ✓. Wall time: **~5–10 m on warm cache** (no new dep additions; just consume avian3d's `Physics` time-clock + Bevy's `WindowFocused` message). msrv-check (Rust 1.89) MUST pass.
    - `gh run list --workflow=ci.yml -L 1` → capture run ID. Wait for completion. `gh run view <ID> --log | grep -cE 'warning:|error:'` → expected `0` (modulo `Free disk space` action ambient noise per 3.1/3.2/3.3 precedent).
  - [ ] **Commit 2 (bookkeeping — does NOT trigger CI):** stage `_bmad-output/implementation-artifacts/sprint-status.yaml`, `_bmad-output/implementation-artifacts/3-4-pause-on-focus-loss-pause-menu-stub.md`, `_bmad-output/implementation-artifacts/deferred-work.md`. *(Awaits Till's authorization.)*
    - HEREDOC commit message subject: `bmad: story 3.4 ready-for-dev → review (pause on focus loss + Esc stub)`. Matches Story 3.1 / 3.2 / 3.3 bookkeeping commit shape.
    - Push to `origin/master`. Does NOT trigger CI (`_bmad-output/**` is in `paths-ignore`).
  - [x] **Why two commits, not one:** matches Stories 3.1 / 3.2 / 3.3 / 2.4 / 2.5 / 2.6 precedent. Clean diff focus + CI cost focus + roll-back granularity.
  - [x] Story awaits code review. **Code review recommended via `bmad-code-review` skill, ideally with a different LLM than the implementer.** Diff surface is medium (~150 lines new in `src/pause/mod.rs`; ~3 lines modified in `src/main.rs`); a 3-agent review fits this scope. Specific review attention areas:
    - **(a) `Time<Virtual>` vs `Time<Physics>` redundancy:** is calling both `pause()` invocations defensible, or should we pick one (and which)? The Avian docs say `Time<Physics>` is the canonical physics-pause; the Bevy idiom is `Time<Virtual>::pause()` for engine-wide pause. Calling both is defensive but couples the pause logic to two clock APIs.
    - **(b) Esc-in-MainMenu silent no-op vs. explicit 'press Esc to back to splash' behavior:** Story 4.7 owns the title-screen Esc semantics. Should 3.4 register a no-op handler that logs "Esc in MainMenu — handled by future Story 4.7" for diagnostic clarity, or stay silent (current design)?
    - **(c) Pause-overlay-vs-focus-loss interaction:** the "Pause persistence test" hazard documented in Task 3 visual-verification is currently a deferred-work entry. Should it be a real AC instead (i.e., implement the `PauseLatch { user_initiated: bool }` suppression flag NOW)? Trade-off: scope creep vs. UX correctness.
    - **(d) Camera2d-at-order-1 vs. UI-only-no-camera:** the overlay spawns a Camera2d at order 1 to ensure rendering. Bevy 0.18's bevy_ui can render UI without an explicit Camera2d under specific conditions (UI Camera auto-spawned by `UiPlugin` in some versions). Verify whether the explicit Camera2d is necessary in 0.18.1, or whether removing it simplifies the overlay (with attendant complexity of "what if UiPlugin's auto-camera collides with Arena's Camera3d?").
    - **(e) `PauseSystems::Detect` set with one variant:** is the SystemSet declaration premature? Story 3.2's `ArenaSystems::Setup` had the same one-variant-with-no-current-need property and was justified by future expansion (3.3+ joined it). Story 3.4's `PauseSystems::Detect` may be similarly justified by future pause-related systems (dim-screen post-process, pause-menu UI scaffolding) but those stories don't exist yet. Trade-off: forward-compat hooks vs. YAGNI.

## Dev Notes

### Why this story exists

Story 3.4 is the **simulation-safety floor** for Epic 3. Stories 3.5–3.10 introduce the PlayerShip with thrusters, weapons, projectile collisions, and asteroid destruction — all systems that produce observable, gameplay-relevant change. Without 3.4 in place first, Alt-Tabbing away from the running game advances the simulation in the background: a player who steps away to answer the door for 5 minutes returns to find their ship has drifted into a wall, taken collision damage, fired projectiles into nothing, and possibly died — all invisible to them. **Pause is the architectural prerequisite for "I can leave the keyboard without consequence."**

Three concrete things land:

1. **Engine-time pause via `Time<Virtual>::pause()`.** Bevy 0.18's `Time<Virtual>` is the user-controllable clock that drives `Time<Fixed>` (which Avian's physics schedule consumes) and ALL `Time::delta()`-reading systems (animations, splash timer, future post-process effects). Pausing `Time<Virtual>` is one call that halts every time-driven gameplay system. This is the architectural choke-point FR43 needs.

2. **Avian-explicit physics pause via `Time<Physics>::pause()`.** Avian 0.6 maintains its own `Time<Physics>` clock with an independent `paused` flag (`avian3d/src/schedule/time.rs:124`). The architecture's "Avian in FixedUpdate at 60 Hz" decision (Story 3.2) means physics ticks are gated on `Time<Fixed>` — which IS frozen when `Time<Virtual>` is paused. So calling `Time<Physics>::pause()` is technically redundant under current scheduling. But Avian's docs explicitly recommend the call (`time.rs:40-52` rustdoc example), and it provides defense-in-depth against future Avian releases that decouple the clocks. Two-line cost; meaningful defensive value.

3. **A user-initiated overlay (Esc) distinct from a silent system pause (focus-loss).** Both pause the simulation identically; only the user-initiated path renders a "PAUSED" overlay. This separation is FR43-faithful (both paths satisfy "Game pauses the simulation") AND UX-correct (a user who Alt-Tabs away doesn't need to see a "PAUSED" message — they can't see the game window anyway; the message would render to a hidden surface and potentially leak through OS task-switcher previews).

[Source: [`epics/epic-3-arena-flight-first-combat-first-playable.md:82-108`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md) (Story 3.4 epic spec); [`prd.md:557`](../planning-artifacts/prd.md) (FR43 capability statement); [`architecture.md:210`](../planning-artifacts/architecture.md) (`GameState::Paused` declared as top-level state); [`architecture.md:707`](../planning-artifacts/architecture.md) (FR43 → `src/main.rs` location — superseded by epic spec line 90 which mandates `src/pause/mod.rs`); [`avian3d/src/schedule/time.rs:40-52`](Avian docs) (canonical pause/unpause example)]

### Inherited context from Stories 1.1, 2.1, 3.1, 3.2, 3.3

| Fact | Value | Source |
|---|---|---|
| Bevy version | `0.18` (resolved `0.18.1`) | `Cargo.toml:8` |
| Avian version | `avian3d = "0.6"` (resolved `0.6.1`) | `Cargo.toml:9` |
| Bevy events vs messages | Bevy 0.18 split: `Event` for entity-targeted/observer-style; `Message` for buffered/`MessageReader`-style. **`WindowFocused` is `#[derive(Message)]`** — must use `MessageReader<WindowFocused>` not `EventReader<WindowFocused>` | `bevy_window-0.18.1/src/event.rs:287`, `bevy_ecs-0.18.1/src/lib.rs:81` (prelude exports `MessageReader`) |
| `WindowFocused` shape | `pub struct WindowFocused { pub window: Entity, pub focused: bool }` | `bevy_window-0.18.1/src/event.rs:298-303` |
| `Time<Virtual>` API | `pause()`, `unpause()`, `is_paused()` via Bevy 0.18 prelude | Bevy 0.18 docs |
| `Time<Physics>` API | `pause()`, `unpause()`, `is_paused()` via `PhysicsTime` trait re-exported in `avian3d::prelude` | `avian3d/src/schedule/time.rs:138-225, 252-263`; prelude re-export at `avian3d/src/lib.rs:550-555` |
| `GameState::Paused` | enum variant in `src/state.rs:18` (already exists since Story 1.6 — declared but unused before 3.4); `#[expect(dead_code)]` annotation on the enum will lose its `Paused` consumer count when this story lands | `src/state.rs:7-19` |
| `KeyCode::Escape` | Bevy 0.18 keypress code; `Res<ButtonInput<KeyCode>>::just_pressed(KeyCode::Escape)` is the canonical edge-detection idiom (Story 3.1 uses identical pattern for Enter at `src/ui/main_menu.rs:56`) | `src/ui/main_menu.rs:56` |
| `cleanup_on_exit::<T>` | generic at `src/arena/mod.rs:32-36`; Story 3.2 declared it specifically anticipating Story 3.4's `<PauseOverlayEntity>` consumer (Story 3.2 Dev Notes line 92) | `src/arena/mod.rs:32-36`, `3-2-...-md` Dev Notes |
| Marker-only-on-roots cleanup | canonicalized by Story 3.1 `cleanup_main_menu`; Story 3.4 mirrors (Camera2d + parent Node tagged; child Text untagged, despawned via `ChildOf` linked-despawn cascade) | `src/ui/main_menu.rs:62-66` |
| Camera order convention | Arena Camera3d: default `order: 0`. PauseOverlay Camera2d: explicit `order: 1` to render on top without ambiguity warnings. | Story 3.3 spawn pattern; deferred-work.md:89 (Story 2.2 swatch precedent for order:1) |
| Focus-loss event source | winit emits `WindowFocused` on Alt-Tab, click-out-of-window, app-switch keyboard shortcuts on macOS/Windows/Linux | Bevy 0.18 winit integration |
| Test count post-3.3 | **19 passing** | `_bmad-output/implementation-artifacts/3-3-...-md` Dev Agent Record |
| Test count post-3.4 (expected) | **19–21** depending on Task 1's optional-test budget | this story |
| `tracing` + panic-hook | live since 1.8; `info!` is the `entered <State>` lifecycle convention | `src/state.rs:22-32`, `src/logging.rs` |
| Splash race re-deferred | non-deterministic; not a 3.4 regression | deferred-work.md:75-76, :137, :168 |
| Splash file location debt | `src/splash.rs` flat at `src/`; 3.4 does NOT touch | deferred-work.md:140 |
| `VisualSystems::Setup` cleanup chore | re-deferred by 3.3; 3.4 does NOT touch | deferred-work.md:158 |
| Commit style precedent | `feat:` for source, `bmad:` for bookkeeping; HEREDOC for multi-line; no `Co-Authored-By` trailer | `git log --oneline -n 15` |
| Two-commit pattern | source + bookkeeping; used by Stories 1.7/2.4/2.5/2.6/3.1/3.2/3.3 | `git log` |
| `paths-ignore` in CI | `.github/workflows/ci.yml` excludes `_bmad/**` and `_bmad-output/**` from triggers | deferred-work.md:5 |

### Five-key constraint summary (memorize these)

1. **`Time<Virtual>::pause()` is THE engine-wide pause mechanism in Bevy 0.18.** Pausing `Time<Virtual>` halts `Time<Fixed>` (which Avian consumes), animation systems, splash-style timers, and any system reading `Time::delta()`. **DO NOT** try to pause via `app.world.resource_mut::<State<GameState>>().pause()` (no such API on Bevy `States`) or via custom `Schedule` manipulation. The single Bevy-idiomatic call is `time_virtual.pause()` on `ResMut<Time<Virtual>>`.

2. **`MessageReader<WindowFocused>` NOT `EventReader<WindowFocused>`.** Bevy 0.18 split `Event` (observer-style, entity-targeted) and `Message` (buffered, reader-style). `WindowFocused` derives `#[derive(Message)]`. The `bevy::prelude::*` re-exports both `MessageReader` and `EventReader` — using the wrong one produces `the trait Event is not implemented for WindowFocused` on `cargo check`.

3. **`Time<Physics>::pause()` is the Avian-canonical second pause call** (defense in depth + matches Avian rustdoc example at `time.rs:40-52`). Both clocks pausing is redundant under current scheduling but defensive against future decoupling.

4. **PauseOverlay belongs to Esc-pause path ONLY.** `PauseInitiator::FocusLoss` does NOT spawn the overlay (Alt-Tabbed user can't see the window anyway; the overlay would render to a hidden surface). The two paths share `pause_simulation_clocks` but diverge on the overlay-spawn decision via `PauseInitiator` resource.

5. **`run_if(in_state(Arena))` gates focus-loss + Esc — not `(Arena | Caravan | Combat)`.** Story 3.4 only handles Arena pause because Caravan/Combat states don't exist yet. The deferred-work entry from Task 4 documents the gate-widening contract for Story 6.1+ when Caravan lands.

### Architecture compliance

- **`GameState::Paused` as top-level state** matches `architecture.md:210` ("Top-level states: `MainMenu`, `Arena`, `Caravan`, `PostRun`, `PhotoMode`, `Paused`"). The `Paused` variant has been declared in `src/state.rs:18` since Story 1.6, awaiting its first consumer — Story 3.4. [Source: architecture.md:210, src/state.rs:18]
- **`PausePlugin` per-feature plugin pattern** matches `architecture.md:343-350` ("each feature module exposes a `<Feature>Plugin` type ... a `SystemSet` enum (`<Feature>Systems`) for ordering"). `PausePlugin` owns pause-state lifecycle; placement at `src/pause/mod.rs` follows Story 3.2's `src/arena/mod.rs` precedent. [Source: architecture.md:343-350, epic-3 spec line 90]
- **FR43 location reconciliation:** architecture.md:707 maps FR43 to `src/main.rs` (window-focus event handler). Epic-3 spec line 90 mandates `src/pause/mod.rs::PausePlugin`. **Reconciliation:** the architecture's bulk FR-mapping line says "the FR's behavior is dispatched from main.rs" — true in the sense that `add_plugins(PausePlugin)` lives in main.rs. The actual systems live in `src/pause/`. This matches the ArenaPlugin precedent (`mod arena;` in main.rs; systems in `src/arena/`). No deviation from architectural intent. [Source: architecture.md:707 vs epic-3 spec line 90]
- **`cleanup_on_exit::<T>` re-use** honors Story 3.2's deliberate generic-design intent (3.2 Dev Notes line 92: "Story 3.4 needs `cleanup_on_exit::<PauseOverlayEntity>`"). Story 3.4 imports `crate::arena::cleanup_on_exit` and applies it to `<PauseOverlayEntity>` — no duplicated despawn loop. [Source: src/arena/mod.rs:32-36, 3-2-...-md Dev Notes line 92]
- **Marker-only-on-roots cleanup** matches the precedent canonicalized by Story 3.1's `cleanup_main_menu`: parent Node + Camera2d carry `PauseOverlayEntity`; the child Text node does NOT. Bevy 0.18's `ChildOf` linked-despawn cascade handles the child. [Source: src/ui/main_menu.rs:36-44, deferred-work.md:91 "the canonical state-exit cleanup pattern for all future UI surfaces"]
- **State-transition idiom** matches `architecture.md:417-418` ("Trigger transitions by `NextState<GameState>` resource mutation. Never mutate `State<GameState>` directly. `OnEnter(state)` / `OnExit(state)` systems are idempotent"). Story 3.4 uses `NextState<GameState>` exclusively; OnEnter/OnExit systems are idempotent (re-pausing a paused clock is a no-op via `Time::pause()`'s flag check). [Source: architecture.md:417-418]
- **Cross-cutting Resources read-only / cross-plugin discipline** matches `architecture.md:660-664`. `PausePlugin` reads `Res<State<GameState>>` (read-only — architectural-correct), `Res<ButtonInput<KeyCode>>` (read-only — Bevy-managed). It WRITES `NextState<GameState>` (the explicit transition mechanism, architectural-correct), `ResMut<Time<Virtual>>` and `ResMut<Time<Physics>>` (engine-managed Bevy/Avian Resources, correct usage of their public APIs). It INSERTS `PausedFrom` and `PauseInitiator` Resources owned by PausePlugin itself (intra-plugin state — correct). [Source: architecture.md:660-664]
- **No `.after(specific_system_function)`** — `PausePlugin` uses `configure_sets` for the `PauseSystems::Detect` set and `.chain()` on the OnEnter tuple. No function-name-based `.after()` calls. [Source: architecture.md:415]
- **No god-plugin** — `PausePlugin` owns ONLY pause-state lifecycle (focus-loss detection, Esc detection, Time clock control, overlay UI). Physics scheduling stays in main.rs. Arena lifecycle stays in `ArenaPlugin`. UI for non-pause surfaces stays in `UiPlugin`. Plugin boundaries match `architecture.md:643-657`. [Source: architecture.md:643-657]
- **Past-tense events** — none yet. Story 3.4 produces no events of its own; it consumes Bevy's `WindowFocused` Message (which is past-tense per the Bevy convention) and uses `Res<NextState>` for state-machine signaling (Bevy idiom, not a custom event). [Source: architecture.md:324]
- **Naming** — `pause_on_focus_loss`, `resume_on_focus_gain`, `toggle_pause_on_escape`, `pause_simulation_clocks`, `resume_simulation_clocks`, `spawn_pause_overlay_if_user_initiated` are all snake_case verb-phrases per `architecture.md:323`. `PausePlugin`, `PausedFrom`, `PauseInitiator`, `PauseOverlayEntity`, `PauseSystems` are PascalCase nouns/markers per `architecture.md:322`. [Source: architecture.md:322-323]

### Library / framework requirements

| Crate | Version | Change in Story 3.4 |
|---|---|---|
| `bevy` | `0.18` (resolved `0.18.1`) | unchanged — uses `MessageReader`, `WindowFocused`, `KeyCode`, `ButtonInput`, `NextState`, `State`, `Time<Virtual>`, `Camera2d`, `Camera`, `Node`, `Text`, `TextFont`, `TextColor` (all `bevy::prelude::*` + `bevy::window::WindowFocused`) |
| `avian3d` | `0.6` (resolved `0.6.1`) | unchanged — first-time consumption of `avian3d::prelude::Physics` (the `Time<Physics>` clock-marker type) and the `PhysicsTime` trait methods `pause()`/`unpause()` |
| All other pinned deps | unchanged | unchanged — `bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager`, etc. all untouched |
| `Cargo.toml` | unchanged | no feature additions, no version bumps, no new deps |
| `Cargo.lock` | unchanged (expected) | no dep tree change; should be byte-identical post-3.4 |

**Avian 0.6 imports needed in 3.4:** `Physics` from `avian3d::prelude::Physics` (the time-clock marker; the `PhysicsTime` trait is auto-imported via the prelude per `avian3d/src/lib.rs:550-555`).

**Bevy 0.18 imports needed in 3.4:** `MessageReader` (auto-prelude), `WindowFocused` from `bevy::window::WindowFocused`. All other types are in `bevy::prelude::*`.

### File structure changes

| Path | Action | Purpose |
|---|---|---|
| `src/pause/mod.rs` | **Add** | `PausePlugin` + `PausedFrom` + `PauseInitiator` + `PauseOverlayEntity` + 6 systems + optional unit tests; ~150-200 lines. |
| `src/main.rs` | **Modify** | +3 lines: `mod pause;`, `use pause::PausePlugin;`, `.add_plugins(PausePlugin)`. Net +3 lines (~57 → ~60). |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | **Modify** | 3-4 → review, last_updated bump |
| `_bmad-output/implementation-artifacts/3-4-...-md` (this file) | **Modify** | Tasks checked, Dev Agent Record populated, Status → review |
| `_bmad-output/implementation-artifacts/deferred-work.md` | **Modify** | Append "Deferred from: 3-4-..." section with 4 entries (gate-widening, focus-gain-overrides-Esc, generic-cleanup-3rd-consumer, PauseInitiator-lifetime) |
| `Cargo.toml`, `Cargo.lock` | **Do NOT touch** | No version bumps, no feature additions |
| `src/state.rs` | **Do NOT touch** | `GameState::Paused` already declared since Story 1.6; no new entry / no removal of `#[expect(dead_code)]` annotation (the annotation correctly stays — `Caravan`, `PostRun`, `PhotoMode` variants remain unused after 3.4) |
| `src/splash.rs` | **Do NOT touch** | Splash race + location debt re-deferred again |
| `src/logging.rs` | **Do NOT touch** | Out of scope |
| `src/ui/**` | **Do NOT touch** | Title screen + main-menu surfaces unchanged. **Note:** the pause overlay deliberately lives in `src/pause/mod.rs` NOT `src/ui/pause_overlay.rs` — pause UI is part of pause-state lifecycle (PausePlugin's responsibility), not a generic UI surface. If a future story adds a richer pause menu (settings, quit-to-title, etc.), the menu UI may move to `src/ui/pause_menu.rs` while pause-detection logic stays in `src/pause/`. |
| `src/visual/**` | **Do NOT touch** | Out of scope; no toon-shader / outline / palette changes |
| `src/arena/**` | **Do NOT touch** | The generic `cleanup_on_exit` is imported, not modified. Arena entry/exit paths are unchanged. |
| `src/tuning/**` | **Do NOT touch** | TuningConfig untouched (no new tunables — pause is an instantaneous boolean, no f32 to tune) |
| `assets/**` | **Do NOT touch** | No new assets; the "PAUSED" string is hardcoded (NOT in `en.ron` because 3.4 ships before the string-table feature lands per architecture.md:597; FR-RON-localization-target is post-MVP per PRD) |
| `docs/**` | **Do NOT touch** | Out of scope |
| `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.gitignore` | **Do NOT touch** | Out of scope |

### `src/pause/mod.rs` skeleton (rustfmt-tolerant, near-verbatim — ~170 lines)

```rust
//! PausePlugin — owns GameState::Paused entry/exit + simulation-clock pause/resume.
//! Triggers: window focus loss (silent) and Escape key (with on-screen overlay).

use avian3d::prelude::Physics;
use bevy::prelude::*;
use bevy::window::WindowFocused;

use crate::arena::cleanup_on_exit;
use crate::state::GameState;

pub struct PausePlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PauseSystems {
    Detect,
}

#[derive(Resource, Debug, Clone, Copy)]
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
            (pause_simulation_clocks, spawn_pause_overlay_if_user_initiated).chain(),
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
            let target = paused_from.as_deref().map_or(GameState::Arena, |p| p.0);
            next_state.set(target);
            info!("resumed from focus gain → {:?}", target);
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
            let target = paused_from.as_deref().map_or(GameState::Arena, |p| p.0);
            next_state.set(target);
            info!("resumed via Escape → {:?}", target);
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
```

### `src/main.rs` post-edit (rustfmt-tolerant — diff against current)

```rust
//! asteroids3D — app entry point.
//! Initializes tracing subscriber + panic-hook-to-file before Bevy startup.
//! Registers DefaultPlugins (minus LogPlugin), GameState, splash flow, and gameplay plugins.

use avian3d::prelude::{Gravity, PhysicsPlugins};
use bevy::prelude::*;

mod arena;
mod logging;
mod pause;
mod splash;
mod state;
mod tuning;
mod ui;
mod visual;

use arena::ArenaPlugin;
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
```

### User-initiated vs focus-loss pause overlay decision

The epic AC #4 mandates spawning the "PAUSED — Esc to resume" overlay when the player presses Escape. The epic AC #2 (focus-loss pause) says nothing about an overlay — only that "the simulation is paused per the applicable Bevy-0.18 / Avian-0.6 convention." Story 3.4's interpretation: focus-loss is a SILENT pause. Two reasons:

1. **Visibility:** the player who Alt-Tabbed away cannot see the game window. Spawning a "PAUSED" overlay renders to a hidden surface — pure work for nothing. Worse, on macOS, the OS task-switcher preview (Mission Control, Cmd-Tab thumbnails) WILL render the off-screen overlay if it captured a thumbnail of the game window after pause was triggered, leaking a "PAUSED" decoration into the OS UI for no benefit.

2. **UX intent:** Esc is a deliberate user input ("I want to pause, show me feedback"). Focus-loss is incidental ("I clicked away to check something"). The overlay communicates "your pause is acknowledged" — useful for the deliberate path; redundant for the incidental path.

The mechanism: `PauseInitiator::User | FocusLoss` resource is set by the trigger system; `spawn_pause_overlay_if_user_initiated` reads it and only proceeds when `User`. Cost: 1 extra Resource definition + 3-line guard at overlay-spawn entry. Trade-off: overlay-spawn becomes branching (3.4 reviewers: confirm this is justified or push back).

| Option | Pros | Cons |
|---|---|---|
| **Esc spawns overlay; focus-loss does NOT (selected)** | UX-clean (overlay only when user can see it); avoids macOS task-switcher preview leak | Two paths through OnEnter(Paused); requires `PauseInitiator` resource |
| Both paths spawn overlay | Single OnEnter(Paused) path (simpler code) | Macos preview leak; off-screen rendering work for no benefit |
| Neither path spawns overlay | Simplest code (no overlay system needed) | Violates epic AC #4's literal text ("text Node 'PAUSED — Esc to resume' is spawned with PauseOverlayEntity marker"); fails the spec |

**Decision:** Esc-only overlay. The complexity cost (Resource + guard) is small and the UX correctness is real.

### Test policy — why 0–2 new tests are reasonable here

Story 3.2's "zero new tests" was correct for a plugin-skeleton story. Story 3.3's "5 new tests" was correct for literal-data-invariant testing. Story 3.4's territory:

- **Plugin-skeleton aspect:** tests are not load-bearing (Bevy's `App::new() → init_state → add_plugins → update` machinery is the de-facto test, but architecture.md:354 defers integration tests post-M3).
- **Behavioral aspect:** the systems are reactive to Bevy-engine signals (`WindowFocused` Message, `KeyCode::Escape` press). Unit-testing them requires either a full `App::new()` harness (which falls under integration tests) or mocking Bevy's input/event machinery (which is overkill for this story's scope).
- **Pure-data invariant aspect:** there are no const arrays of layout coordinates to test (unlike Story 3.3 ASTEROIDS). The only testable data shapes are `PausedFrom(GameState::Arena).0 == GameState::Arena` (trivial) and `PauseInitiator::FocusLoss != PauseInitiator::User` (also trivial).

**Recommendation:** include the 2 trivial invariant tests for cheap regression-guard, OR skip with the YAGNI justification "the guarded properties are too tautological to drift." Either choice is defensible. Document chosen N (0 or 2 new) in Completion Notes.

### Logging discipline

The systems emit `info!` at every state-transition boundary. This is consistent with Story 1.6's `entered <State>` lifecycle convention but goes one step further by logging the *cause* (focus loss vs Escape):

- `paused on focus loss (window <Entity>)` — distinguishes the silent path
- `paused via Escape (initiator: user)` — distinguishes the user-deliberate path
- `resumed from focus gain → <GameState>` — focus regain
- `resumed via Escape → <GameState>` — explicit Esc-resume
- `simulation clocks paused (virtual.is_paused=<bool>, physics.is_paused=<bool>)` — proves both clocks engaged
- `simulation clocks resumed` — symmetric inverse

**Why log the `is_paused` boolean values:** Task 3 runtime smoke uses `grep -c 'virtual.is_paused=true'` as evidence that the pause actually engaged. Without the boolean in the message, the grep would have to be replaced by a manual stress test (e.g., adding a per-frame counter that asserts non-incrementation during pause), which is more invasive.

**No `entered Paused` / `exited Paused` logs from PausePlugin itself:** the `OnEnter(Paused)` / `OnExit(Paused)` schedule systems emit cause-specific logs (paused-on-focus-loss, paused-via-Escape, etc.), which is more informative than a generic "entered Paused". State-machine reviewers who need raw `entered/exited Paused` evidence can add temporary `add_systems(OnEnter(Paused), || info!("entered Paused"))` registrations during code review; the cause-specific logs subsume the need.

### Previous-story intelligence — what to learn from 3.1 / 3.2 / 3.3

**From Story 3.1 (UiPlugin + cleanup_main_menu + Camera2d at order 0):**
- The marker-only-on-roots cleanup pattern (parent Node + Camera2d tagged; child Text untagged, despawned via Bevy 0.18 `ChildOf` linked-despawn cascade) is Story 3.1's canonical contribution. Story 3.4 mirrors it exactly for the pause overlay.
- Story 3.1's MainMenu Camera2d uses default `order: 0`. After OnExit(MainMenu) the Camera2d is despawned. So in Arena there is only the Arena's Camera3d at default order 0 — and the pause overlay's Camera2d at explicit order 1 sits cleanly above without ambiguity.

**From Story 3.2 (ArenaPlugin + Avian foundation + cleanup_on_exit generic):**
- The `cleanup_on_exit::<T>` generic at `src/arena/mod.rs:32-36` was designed FOR Story 3.4's `<PauseOverlayEntity>` consumer (3.2 Dev Notes line 92). Story 3.4 is the second consumer; honors the design.
- The plugin-per-feature pattern (mod arena; arena::ArenaPlugin; add_plugins) is the template Story 3.4 follows verbatim with `mod pause;` + `pause::PausePlugin;`.
- Avian's `Time<Physics>` clock (vs Bevy's `Time<Virtual>`) was unintroduced in 3.2 — 3.4 is the first story to interact with it via `pause()` / `unpause()` calls.

**From Story 3.3 (zone-spawn + DirectionalLight + stand-in Camera3d + cfg_attr removal):**
- Story 3.3's stand-in Camera3d at default `order: 0` is what Story 3.4's Camera2d at `order: 1` renders above. Once Story 3.5 lands the cockpit camera (also Camera3d), the order-0/order-1 split persists.
- Story 3.3's review-finding "code review patches: warn! log when tuning.ron not loaded; .expect() on .ico(2)" set the precedent for in-flight review patches landing as separate `fix:` commits. Story 3.4 should expect similar review patches (e.g., a reviewer suggesting `EventReader → MessageReader` migration if the dev miswrote the import) and treat them as expected, not exceptional.
- Two-commit pattern (source + bookkeeping) is now the rock-solid project default — five consecutive stories (3.1, 3.2, 3.3, plus 2.5/2.6) used it.

### Forward compatibility — Story 3.5 (PlayerShip) hand-off

Story 3.5 will spawn a `PlayerShip` entity with `RigidBody::Dynamic` + `Collider`. **Story 3.4's pause behavior subsumes 3.5's PlayerShip motion:** when `Time<Virtual>` and `Time<Physics>` are paused, the PlayerShip's Avian-driven motion freezes. The Esc-pause overlay renders above the cockpit Camera3d (Story 3.5's first cockpit camera at default `order: 0`). No 3.4 work is needed for 3.5 compatibility — the pause is generic over what Arena state is rendering.

### Forward compatibility — Stories 3.6–3.8 (flight input, dampener) hand-off

3.6 (translation), 3.7 (rotation), 3.8 (dampener) attach `ExternalForce`/`ExternalTorque` to the `PlayerShip`. **Pausing freezes integration**, so flight inputs read during pause produce no motion. The flight-input systems should NOT be gated on `in_state(Arena)` only — they should be gated on `in_state(Arena).and(time_virtual_not_paused)` or equivalent. This is 3.6's concern; Story 3.4 only ensures the time clocks freeze on pause.

### Forward compatibility — Story 3.9 (projectiles) + Story 3.10 (collision) hand-off

Same logic: Pausing halts physics; projectiles in flight stop mid-trajectory; collision events that would have fired during pause-time accumulate or skip per Avian's pause semantics (Avian's `Time<Physics>` is the safe check). When Story 3.9 introduces the firing system, it should be gated on `Time<Virtual>::is_paused() == false` to prevent the player from holding the trigger during pause and dumping a buffered burst on resume. This is 3.9's concern.

### Forward compatibility — Story 3.11 (HUD baseline) hand-off

Story 3.11 spawns `HudEntity`-tagged screen-space UI on `OnEnter(Arena)`. **The HUD will be visible during pause** (paused state retains visual elements; Time<Virtual> pause doesn't despawn anything). The pause overlay (text-centered) should NOT obstruct the HUD's corner-anchored elements; the HUD's corners and the overlay's center don't overlap. **The Camera2d at order 1 issue:** Story 3.11 may add its own UI Camera2d. If 3.11's UI Camera2d also uses order > 0, there will be ambiguity between 3.4's order-1 Camera2d and 3.11's. Resolution: Story 3.11 should query for existing Camera2d in Arena and either reuse it or differentiate via `RenderLayers`. Flag this in Story 3.11's forward-compat hand-off.

### Forward compatibility — Stories 4.x and beyond — gate widening

The deferred-work entry at Task 4 documents the "widen the run-condition gate" contract for Story 6.1+. Specifically:
- Story 6.1 (CaravanPlugin skeleton) introduces `GameState::Caravan`. The pause systems' `run_if` must extend: `in_state(Arena).or(in_state(Caravan))` for `pause_on_focus_loss`; `in_state(Arena).or(in_state(Paused).or(in_state(Caravan)))` for `toggle_pause_on_escape`.
- Story 6.8 (combat pockets) may introduce a sub-state. If implemented as a sub-state of Caravan, the existing `in_state(Caravan)` gate covers it. If a top-level state, gate must extend further.
- Story 4.7 (full title screen) reclaims the Esc-in-MainMenu input: pressing Esc on the title screen should exit the app (or open settings, etc.). Story 4.7 adds its own Esc handler gated on `in_state(MainMenu)`; Story 3.4's `toggle_pause_on_escape` is still gated to `Arena | Paused` and does not conflict.

### Project structure notes

- **Path alignment:** `src/pause/mod.rs` is **NEW**. Architecture.md:589-597 lists `src/ui/` as the home for FR36–FR43 (which includes FR43 pause). Architecture.md:707 specifically maps FR43 to `src/main.rs`. Epic-3 spec line 90 mandates `src/pause/mod.rs::PausePlugin`. **Reconciliation:** the pause systems are state-lifecycle code (more akin to ArenaPlugin and 3.5's projected FlightPlugin than to UI surfaces). Living at `src/pause/mod.rs` matches Story 3.2's `src/arena/mod.rs` precedent and matches the epic-3 spec's explicit directive. Architecture.md's bulk FR mapping ("FR43 → src/main.rs") is interpreted loosely as "dispatched from main.rs" — true, since `add_plugins(PausePlugin)` lives there.
- **`src/main.rs` is MODIFIED** with +3 lines.
- **No path conflicts** introduced by Story 3.4.
- **Splash file location debt re-deferred** — `src/splash.rs` stays flat at `src/`. 3.4 does NOT touch splash.
- **`Cargo.toml`, `Cargo.lock`, `.gitignore`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `assets/**`, `docs/**` — UNTOUCHED.**

### LLM dev-agent guardrails — most-likely-to-go-wrong patterns

1. **Using `EventReader<WindowFocused>` instead of `MessageReader<WindowFocused>`.** Bevy 0.18's `WindowFocused` derives `Message`, not `Event`. The compile error is `the trait Event is not implemented for WindowFocused` — diagnose by checking the `#[derive(Message)]` annotation in `bevy_window-0.18.1/src/event.rs:287`.
2. **Forgetting `.run_if(in_state(...))` on the toggle system.** Without the gate, Esc presses in MainMenu/Loading/PostRun/PhotoMode would all fire `toggle_pause_on_escape`, which would attempt to enter `Paused` from a state that has no `OnEnter(Paused) → OnExit(Paused) → OnEnter(MainMenu/Loading/...)` round-trip wired. Bevy doesn't crash — it just transitions to `Paused` with a `PausedFrom(MainMenu)` (because `current_state.get()` returns whatever state was active), and then the resume returns to MainMenu. **But:** Camera2d order-1 layered above MainMenu's order-0 Camera2d → ambiguity warning. Esc-in-MainMenu must be a no-op in this story's scope (Story 4.7 owns it).
3. **Using `commands.entity(e).despawn_recursive()` instead of `despawn()` in the cleanup.** The generic `cleanup_on_exit::<T>` from `src/arena/mod.rs:32-36` uses `despawn()` (root-only, children cascade via `ChildOf`). Story 3.4's overlay is a Camera2d + parent Node (with Text child). The marker `PauseOverlayEntity` is on the Camera2d AND the parent Node, NOT on the Text. The cleanup walks both marked entities and despawns them; the Text auto-cascades from the Node despawn. **Do NOT** add `PauseOverlayEntity` to the Text — that produces a "tried to despawn missing entity" warning when the Text is cascaded-despawned and then iterated by the cleanup query (the Story 2.2 over-tagging hazard, deferred-work.md:92).
4. **Calling `Time<Virtual>::reset()` instead of `pause()`.** `reset()` rewinds to t=0 — for a paused-resumed cycle, you want `pause()` then `unpause()`. Reset would teleport time backwards and could break animation interpolation, splash timer accuracy, or any `since_startup` reasoning.
5. **Adding `mod pause;` BEFORE `mod arena;` in main.rs alphabetically.** Rustfmt may reorder. Either order works; the issue is when the dev hand-orders them and rustfmt reorders later, producing a fmt-check failure. Just let rustfmt do it.
6. **Spawning the overlay in `pause_simulation_clocks` instead of in a separate system.** Mixing concerns: clock systems should ONLY pause clocks; UI spawn systems should ONLY spawn UI. Two systems on `OnEnter(Paused)` is the architectural-clean factoring.
7. **`Time<Physics>` import path failure.** If `use avian3d::prelude::Physics;` doesn't compile (verify in your local Cargo registry — the prelude SHOULD re-export it), fall back to `use avian3d::schedule::time::Physics;`. Both are valid paths; the prelude is preferred per Avian convention.
8. **Forgetting to insert `PauseInitiator` on the focus-loss path.** Without it, `spawn_pause_overlay_if_user_initiated` reads `Option<Res<PauseInitiator>>` as `None` and skips overlay-spawn (correct behavior for focus-loss). BUT a leftover `PauseInitiator::User` from a previous Esc-pause would persist (resources aren't auto-removed on state transition), so a focus-loss after an Esc-resume would incorrectly read `User` and spawn the overlay. **Fix:** insert `PauseInitiator::FocusLoss` in `pause_on_focus_loss` (which the skeleton above already does — verify it's preserved during the implementation).
9. **Camera2d at default order 0 producing "ambiguous camera order" warnings.** Arena's Camera3d at default `order: 0` + new Camera2d at default `order: 0` = ambiguity. Always set `order: 1` (or higher) on the overlay Camera2d.
10. **Hardcoding "PAUSED — Esc to resume" in non-RON form.** Architecture.md:597 prescribes `assets/strings/en.ron` as the canonical string-table. **However**, the string-table feature is post-MVP per the PRD. Story 3.4 hardcodes the string in source as a **temporary measure**; when the string-table system lands (post-MVP), the literal moves to `en.ron` with key `ui.pause.message`. Document this in Completion Notes if the reviewer flags it.
11. **`run_if(in_state(Arena).or(in_state(Paused)))` not compiling.** Bevy 0.18's `in_state(...)` returns `impl Condition`; Bevy's `Condition::or` extension provides the combinator. If unavailable in 0.18.1, expand to two registrations (one per state). See Task 3 cargo-check fallback.
12. **Touching `src/state.rs` to remove `Paused` from the `#[expect(dead_code)]` annotation.** **Do NOT.** The annotation is per-enum-as-a-whole, not per-variant. `Paused` becoming live (consumed by `next_state.set(GameState::Paused)`) doesn't change the dead_code surface — the OTHER variants (`Caravan`, `PostRun`, `PhotoMode`) remain unused. Removing the annotation produces 3 dead_code warnings on the remaining variants. Leave it alone; the annotation is correct.
13. **`return` after first `WindowFocused` event hit.** This is intentional — it prevents redundant state-transitions if multiple events arrive in one frame. **Do NOT** remove it for "completeness" or to "process all events" — Bevy events are designed for single-fire reactivity, not batch processing.
14. **Touching `Cargo.toml`.** No new deps. Avian's `Physics` time-clock and Bevy's `Time<Virtual>` + `WindowFocused` are all already pinned.
15. **Adding a `pause_overlay.rs` sibling file when 150 lines fits in `mod.rs`.** YAGNI: keep everything in `src/pause/mod.rs` until the file exceeds ~250 lines. If a future story expands pause UI (settings menu accessible from pause), THAT story splits the file.
16. **Skipping the focus-loss runtime smoke (Task 3's `run-focus.log`).** Without it, the focus-loss path is unverified — only the Esc path proves the pause works at all. Both paths must be smoked.
17. **Spawning the overlay's Camera2d with `..default()` for the entire `Camera` struct.** `Camera2d` is the marker; `Camera { order: 1, ..default() }` is the explicit-order override. Spawning `(Camera2d, Camera::default(), ...)` would default `order: 0` and produce the ambiguity warning. The skeleton above is correct — verify the implementation matches.
18. **Touching `_bmad-output/planning-artifacts/**`.** Read-only from story-execution perspective.

### References

- [Source: [`_bmad-output/planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md:82-108`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md)] — Story 3.4 epic spec.
- [Source: [`_bmad-output/planning-artifacts/prd.md:557`](../planning-artifacts/prd.md)] — FR43 capability statement: "Game pauses the simulation when Player opens the in-run pause menu or when the application window loses focus."
- [Source: [`_bmad-output/planning-artifacts/architecture.md:210`](../planning-artifacts/architecture.md)] — `GameState::Paused` declared as top-level state.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:240`](../planning-artifacts/architecture.md)] — Avian `FixedUpdate` at 60 Hz (relevant: Time<Virtual> pause implies Time<Fixed> pause implies Avian no-tick).
- [Source: [`_bmad-output/planning-artifacts/architecture.md:343-350`](../planning-artifacts/architecture.md)] — Plugin-per-feature module pattern + `<Feature>Systems` SystemSet.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:415-420`](../planning-artifacts/architecture.md)] — `cleanup_on_exit::<T>` pattern via state-scoped markers.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:417-418`](../planning-artifacts/architecture.md)] — `NextState<GameState>` is the state-transition mechanism; idempotent OnEnter/OnExit semantics.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:660-664`](../planning-artifacts/architecture.md)] — Cross-cutting Resources read in main.rs (here: `State<GameState>`, `ButtonInput<KeyCode>`).
- [Source: [`_bmad-output/planning-artifacts/architecture.md:707`](../planning-artifacts/architecture.md)] — FR43 location mapping (reconciled with epic spec line 90 in Dev Notes "Architecture compliance").
- [Source: [`Cargo.toml:8-12`](../../Cargo.toml)] — bevy 0.18 + avian3d 0.6 + bevy_mod_outline 0.12 + bevy_kira_audio 0.25 + leafwing-input-manager 0.20 pinned versions.
- [Source: [`src/main.rs`](../../src/main.rs)] — current plugin-registration block (post-3.3; +3 lines for PausePlugin in 3.4).
- [Source: [`src/state.rs:7-19`](../../src/state.rs)] — `GameState` enum including `Paused` variant declared since 1.6.
- [Source: [`src/arena/mod.rs:32-36`](../../src/arena/mod.rs)] — `cleanup_on_exit::<T: Component>` generic from Story 3.2; reused by 3.4 for `<PauseOverlayEntity>`.
- [Source: [`src/ui/main_menu.rs:36-44, 62-66`](../../src/ui/main_menu.rs)] — `cleanup_main_menu` precedent (marker-only-on-roots cleanup pattern); 3.4 mirrors.
- [Source: [`src/ui/main_menu.rs:52-60`](../../src/ui/main_menu.rs)] — Enter-key edge-detection idiom (`Res<ButtonInput<KeyCode>>::just_pressed`); 3.4 mirrors for `KeyCode::Escape`.
- [Source: avian3d-0.6.1 source — `src/schedule/time.rs:40-52, 124-263`] — `Time<Physics>` clock, `PhysicsTime` trait, `pause()`/`unpause()`/`is_paused()` methods, rustdoc example.
- [Source: avian3d-0.6.1 source — `src/lib.rs:550-555`] — prelude re-exports `PhysicsTime` trait + `Physics` time-clock marker type.
- [Source: bevy_window-0.18.1 source — `src/event.rs:287-303`] — `WindowFocused` `#[derive(Message)]` declaration + struct shape.
- [Source: bevy_ecs-0.18.1 source — `src/lib.rs:81`] — prelude re-exports `MessageReader`, `MessageWriter`, `Message`, `Messages`, `MessageMutator`.
- [Source: [`_bmad-output/implementation-artifacts/3-2-avian-physics-foundation-arena-state-skeleton.md`](./3-2-avian-physics-foundation-arena-state-skeleton.md) Dev Notes line 92] — generic `cleanup_on_exit::<T>` was deliberately designed to serve Story 3.4's `<PauseOverlayEntity>` consumer.
- [Source: [`_bmad-output/implementation-artifacts/3-3-hand-designed-arena-zone-with-static-asteroid-field.md`](./3-3-hand-designed-arena-zone-with-static-asteroid-field.md) "Forward compatibility — Story 3.4 (pause overlay) hand-off"] — Story 3.3 forward-compat note that 3.4 freezes simulation but does NOT despawn arena entities (asteroids remain spawned during Paused).
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:75-76, 137, 139, 168`](./deferred-work.md)] — Splash race + location debt + VisualSystems::Setup empty-no-op (all re-deferred for 3.4 — out of scope).
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:148`](./deferred-work.md)] — Story 3.2 review finding "no enforcement that arena-spawned entities carry the ArenaEntity marker"; 3.4 honors the convention via `PauseOverlayEntity` marker on every spawn site.
- [Source: [`MEMORY.md` → `feedback_full_build_output.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_full_build_output.md)] — verification discipline: exit-0 + tail is NOT proof; grep explicitly.
- [Source: [`MEMORY.md` → `feedback_compact_review_style.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_compact_review_style.md)] — Till's compact-review style.
- [Source: [`MEMORY.md` → `feedback_staged_rollout.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_staged_rollout.md)] — staged-rollout preference; informs scope-bundling rationale (overlay UI + clock pause + state transition + focus-loss handling all in one story is justified because they are inseparable for FR43 satisfaction).

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Opus 4.7, 1M context)

### Debug Log References

Local verification sweep (all logs in `/tmp/`):

| Command | Log file | `grep -cE 'warning:\|error:'` | Notes |
|---|---|---|---|
| `cargo check` | `/tmp/story-3-4-check.log` | 0 | Initial pass produced 8 errors (E0204 `Copy` not derivable for `PausedFrom(GameState)`; E0599 trait `PhysicsTime` not in scope for `Time<Physics>::pause()`) — both fixed in-flight per Dev Notes "Five-key constraint #2/#3"; final pass clean |
| `cargo build` (debug) | `/tmp/story-3-4-build.log` | 0 | 2.36s; incremental build from 3.3 cache |
| `cargo test` | `/tmp/story-3-4-test.log` | 0 (also 0 `FAILED`) | `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Test count: 19 baseline + 2 new (`pause::tests::paused_from_carries_state`, `pause::tests::pause_initiator_variants_distinguishable`) |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-3-4-clippy.log` | 0 | 0.57s; no `dead_code` complaints on `PauseInitiator::FocusLoss` (consumed via the equality check `!= Some(PauseInitiator::User)`) |
| `cargo fmt --all -- --check` | exit code | 0 (after one `cargo fmt --all`) | Rustfmt re-flowed the `(pause_simulation_clocks, spawn_pause_overlay_if_user_initiated).chain()` tuple to multi-line form (line-length-driven; cosmetic only) |
| `cargo build --release` | `/tmp/story-3-4-release.log` | 0 | 4m 05s (LTO=fat + codegen-units=1, full re-link); regression check — no new warnings introduced |
| `cargo run` runtime smoke | `/tmp/story-3-4-run-esc.log` | n/a | Single run exercised both Esc-pause path AND focus-loss-pause path (Till drove the smoke manually). 39 lines total. |

**Runtime-smoke evidence** (per AC #8 grep harness — single combined run log):

| Marker | Count | Expected |
|---|---|---|
| `entered Loading` | 1 | 1 |
| `splash timer elapsed` | 1 | 1 |
| `entered MainMenu` | 1 | 1 |
| `entered Arena` | 4 | ≥ 1 (1 initial + 2 Esc-resume + 1 focus-resume) |
| `paused via Escape` | 2 | ≥ 1 |
| `resumed via Escape` | 2 | ≥ 1 |
| `paused on focus loss` | 1 | ≥ 1 |
| `resumed from focus gain` | 1 | ≥ 1 |
| `simulation clocks paused` | 3 | ≥ 1 (matches 2 Esc + 1 focus) |
| `simulation clocks resumed` | 3 | ≥ 1 |
| `virtual.is_paused=true` | 3 | ≥ 1 (proves `Time<Virtual>::pause()` engaged) |
| `physics.is_paused=true` | 3 | ≥ 1 (proves `Time<Physics>::pause()` engaged) |
| `panic\|backtrace\|FATAL` | 0 | 0 |
| `ambiguous` (camera order) | 0 | 0 |
| `ERROR.*avian` / `WARN.*Avian` | 0 / 0 | 0 / 0 |
| `PAUSED — Esc to resume` (in log) | 0 | 0 (overlay text isn't logged; visually verified by Till during smoke) |

**Documented (non-3.4-regression) WARNs in run log** — consistent with prior deferrals:

1. `bevy_ecs::error::handler: Encountered an error in command ... Entity despawned: ID 87v0 invalid; generation 1` at splash → MainMenu transition — splash-cleanup race per deferred-work.md:75-76, :137, :168 (re-deferred again for 3.4; not 3.4-introduced).
2. `wgpu_core::device::resource: The fragment stage "fragment" output @location(0) values are ignored` — pre-existing Story 2.3 ToonMaterial fragment shader output binding warning; not 3.4-introduced.
3. `bevy_winit::state: Skipped event Destroyed for unknown winit Window Id` at window close — known Bevy 0.18 winit-event race per Story 1.6 deferred-work LOW-1; not 3.4-introduced.

**`PauseOverlayEntity` convention check** (per deferred-work.md:148 resolution path):

```
$ grep -c 'PauseOverlayEntity' src/pause/mod.rs
5
$ grep -c 'commands.spawn(' src/pause/mod.rs
2
$ grep -rn 'cleanup_on_exit::' src/
src/arena/mod.rs:27:            cleanup_on_exit::<ArenaEntity>,
src/pause/mod.rs:54:                cleanup_on_exit::<PauseOverlayEntity>,
```

`PauseOverlayEntity` appears 5 times: 1 component def + 1 cleanup_on_exit registration + 2 spawn-site uses (Camera2d + parent Node) + 1 generic-import path. The 2 generic-cleanup consumers (`<ArenaEntity>` and `<PauseOverlayEntity>`) validate Story 3.2's design intent — Story 3.11 will be the third consumer per the deferred-work entry filed in Task 4.

### Completion Notes List

- **AC #1** ✓ — `src/pause/mod.rs` authored (196 lines including module doc + 2 unit tests). `PausePlugin` registered in `src/main.rs:42` via `App::add_plugins(PausePlugin)`, placed AFTER `ArenaPlugin` and BEFORE `init_resource::<SplashConfig>()` per the ordering precedent. `mod pause;` + `use pause::PausePlugin;` added at `main.rs:10` and `main.rs:18` (rustfmt alphabetized — landed naturally between `logging` and `splash`). All four type definitions present: `PausePlugin`, `PauseSystems::Detect` SystemSet, `PausedFrom(pub GameState)` Resource, `PauseInitiator { FocusLoss, User }` Resource, `PauseOverlayEntity` Component marker.

- **AC #2** ✓ — `pause_on_focus_loss` system at `src/pause/mod.rs:61-75` reads `MessageReader<WindowFocused>`, filters `!event.focused`, inserts `PausedFrom(GameState::Arena)` + `PauseInitiator::FocusLoss`, sets `NextState<GameState>` to `Paused`. Runtime smoke confirmed via `paused on focus loss (window 0v0)` log line (1 occurrence).

- **AC #3** ✓ — `resume_on_focus_gain` system at `src/pause/mod.rs:77-92` reads focus-gain message, sets `NextState` to `paused_from.0` (clones the inner `GameState` because `GameState` doesn't derive `Copy` — see Deviation #1 below), gated `.run_if(in_state(GameState::Paused))`. Runtime smoke confirmed via `resumed from focus gain → Arena` log line.

- **AC #4** ✓ — `toggle_pause_on_escape` at `src/pause/mod.rs:94-120` detects `KeyCode::Escape` `just_pressed`, branches on `current_state.get()`: Arena → set `PauseInitiator::User` + `PausedFrom(Arena)` + `NextState(Paused)`. `spawn_pause_overlay_if_user_initiated` at `src/pause/mod.rs:144-180` spawns the Camera2d (order: 1) + centered Node + child "PAUSED — Esc to resume" Text only when `initiator == Some(PauseInitiator::User)`. Camera order:1 prevents ambiguity with Arena's order:0 Camera3d. Runtime smoke confirmed via `paused via Escape (initiator: user)` log line (2 occurrences) + Till's visual verification.

- **AC #5** ✓ — Esc-while-`Paused` branch in `toggle_pause_on_escape` resumes via `NextState(paused_from.0)`. `OnExit(GameState::Paused)` registers `cleanup_on_exit::<PauseOverlayEntity>` (re-uses Story 3.2's generic from `crate::arena`) + `resume_simulation_clocks`. Runtime smoke confirmed via `resumed via Escape → Arena` (2 occurrences) — overlay despawned cleanly each cycle (no `tried to despawn missing entity` warnings).

- **AC #6** ✓ — `pause_simulation_clocks` (`src/pause/mod.rs:122-133`) calls `time_virtual.pause()` AND `time_physics.pause()` (both via `PhysicsTime` trait — required `use avian3d::prelude::{Physics, PhysicsTime};` per Deviation #2 below). `resume_simulation_clocks` calls the inverse `unpause()` on both. Runtime smoke confirmed via `simulation clocks paused (virtual.is_paused=true, physics.is_paused=true)` log line (3 occurrences across 2 Esc + 1 focus pause cycles).

- **AC #7** ✓ — `run_if(in_state(Arena))` and `run_if(in_state(Arena).or(in_state(Paused)))` gates correctly exclude `Loading`, `MainMenu`, `Caravan`, `PostRun`, `PhotoMode`. Verified by inspection of the registration block at `src/pause/mod.rs:33-49` + `src/pause/mod.rs:36-39`. The `Condition::or` combinator compiled cleanly on Bevy 0.18.1 — fallback to two-system-registration path (Task 3 cargo-check fallback) was not needed.

- **AC #8** ✓ — All 6 cargo commands report 0 warnings/errors per the per-command grep. Test count: 21 (= 19 baseline + 2 new pause invariant tests, within the AC #8 N ≥ 19 expected range). `cargo fmt --all -- --check` exit 0 after one `cargo fmt --all`. Runtime smoke evidence per the table above. Git status final delta exactly matches AC #8 expectations: `M src/main.rs`, `?? src/pause/`, `M sprint-status.yaml`, `M 3-4-...-md` (this file via Status flip + Dev Agent Record), `M deferred-work.md`. NO drift to forbidden paths (`Cargo.toml`, `Cargo.lock`, `src/state.rs`, `src/splash.rs`, `src/logging.rs`, `src/ui/**`, `src/tuning/**`, `src/visual/**`, `src/arena/**`, `assets/**`, `docs/**`, `.github/workflows/**`, etc.).

- **AC #9** ✓ — `cleanup_on_exit::<PauseOverlayEntity>` registered at `src/pause/mod.rs:54` reuses Story 3.2's `crate::arena::cleanup_on_exit` generic (imported at `src/pause/mod.rs:8`). No duplicate despawn loop in `src/pause/mod.rs`. The architectural "generic-cleanup home" decision left untouched — deferred to Story 3.11 (3rd consumer) per the deferred-work entry filed in Task 4.

**Deviations:**

1. **`PausedFrom` derive: `Clone` only, NOT `Clone + Copy`.** The story skeleton specified `#[derive(Resource, Debug, Clone, Copy)] pub struct PausedFrom(pub GameState);`. Empirically `cargo check` failed with E0204 "the trait `std::marker::Copy` cannot be implemented for this type" because `GameState` does NOT derive `Copy` (per `src/state.rs:5` it derives `Default, Debug, Clone, Eq, PartialEq, Hash` — Copy is absent). Adjusted `PausedFrom` to `#[derive(Resource, Debug, Clone)]` and re-wrote the `map_or` callsites in `resume_on_focus_gain` and `toggle_pause_on_escape` to clone the inner state: `paused_from.as_deref().map_or(GameState::Arena, |p| p.0.clone())`. Adding `Copy` to `GameState`'s derive would simplify these call sites, but the story explicitly forbids touching `src/state.rs` ("Do NOT touch"); the deviation honors that constraint and adds a deferred-work entry suggesting `Copy` be appended to `GameState` next time `state.rs` is legitimately touched.

2. **`use avian3d::prelude::PhysicsTime;` required (not just `Physics`).** The story skeleton specified `use avian3d::prelude::Physics;`. Empirically `cargo check` failed with E0599 "no method named `pause` found for struct `ResMut<Time<Physics>>`; items from traits can only be used if the trait is in scope" because `pause()` / `unpause()` / `is_paused()` are defined on the `PhysicsTime` extension trait, not inherent methods on `Time<Physics>`. Adjusted import to `use avian3d::prelude::{Physics, PhysicsTime};`. Both are re-exported from the prelude per `avian3d/src/lib.rs:550-555` — single-line fix. Story Five-key constraint #3 already documented this risk; the empirical failure validates the constraint.

3. **2 unit tests added (matches story spec's 0–2 budget — high end).** Both trivial-data invariants per Dev Notes "Test policy": `paused_from_carries_state` and `pause_initiator_variants_distinguishable`. Net post-3.4 test count: **21** (= 19 baseline + 2 new).

4. **Runtime smoke combined into a single `cargo run` invocation.** The story spec listed two separate `cargo run` smokes (one for Esc-path, one for focus-loss-path) with separate log files. In practice Till exercised both paths in a single run session: 2× Esc pause/resume cycles followed by 1× focus-loss pause/resume cycle, all captured in `/tmp/story-3-4-run-esc.log` (39 lines). Combined log makes the evidence harness simpler (one grep harness instead of two) without sacrificing coverage; both AC #2 and AC #4 paths fired with proper clock-pause evidence. The story's "two separate logs" prescription was guidance, not a hard AC requirement.

5. **Two-commit push — NOT YET EXECUTED.** Per project rules (Stories 3.1/3.2/3.3 precedent), commits and pushes await Till's explicit authorization. The Task 5 subtasks "Commit 1" + "Commit 2" remain unchecked deliberately; Dev Agent Record + Status flip + sprint-status update are saved without staging or pushing.

### File List

**Added:**

- `src/pause/mod.rs` (new file; 196 lines after rustfmt — `PausePlugin` + 6 systems + `PausedFrom` newtype + `PauseInitiator` enum + `PauseOverlayEntity` marker + 2 unit tests)

**Modified:**

- `src/main.rs` (+3 net lines: `mod pause;` declaration, `use pause::PausePlugin;` import, `.add_plugins(PausePlugin)` registration call)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (3-4 status flip ready-for-dev → in-progress → review; `last_updated` bump)
- `_bmad-output/implementation-artifacts/3-4-pause-on-focus-loss-pause-menu-stub.md` (this file: tasks/subtasks checked except Commit 1/Commit 2 awaiting authorization, Dev Agent Record populated, Status → review)
- `_bmad-output/implementation-artifacts/deferred-work.md` (appended new "Deferred from: 3-4-..." section with 5 entries — gate-widening for Caravan/Combat states, focus-gain-overrides-Esc UX hazard, generic-cleanup-3rd-consumer trigger, PauseInitiator-resource-lifetime, GameState-needs-Copy)

**Untouched (verified):** `Cargo.toml`, `Cargo.lock`, `src/state.rs`, `src/splash.rs`, `src/logging.rs`, `src/ui/**`, `src/visual/**`, `src/arena/**`, `src/tuning/**`, `assets/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.gitignore`.
