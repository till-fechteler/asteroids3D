# Story 3.7: Flight Input → 3-Axis Rotation (Pitch / Yaw / Roll)

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player in the Arena cockpit,
I want mouse motion to pitch and yaw my ship and Q/E to roll it,
So that I can aim freely in 3D per FR3, completing the cockpit-flight feel from "I can move" (Story 3.6) to "I can move and look", and giving Story 3.9's weapon-firing a useful aiming surface and Story 3.8's dampener a non-trivial angular-velocity to bleed off.

## Acceptance Criteria

1. **Given** `FlightAction` (added in Story 3.6 with 6 buttonlike variants) is the canonical flight-input enum
   **When** Story 3.7 extends it with rotation variants
   **Then** four new variants are added in this order at the end of the enum: `Pitch`, `Yaw`, `RollLeft`, `RollRight` (final variant order: `ThrustForward, ThrustReverse, StrafeLeft, StrafeRight, ThrustUp, ThrustDown, Pitch, Yaw, RollLeft, RollRight`)
   **And** `Pitch` and `Yaw` are tagged `#[actionlike(Axis)]` (single-value axis inputs sourced from mouse-axis deltas — `InputControlKind::Axis` per leafwing 0.20 `lib.rs:101-106`)
   **And** `RollLeft` and `RollRight` remain default-buttonlike (no per-variant attribute) — they share the same `InputControlKind::Button` default as the 6 thrust variants
   **And** `default_input_map()` is extended to bind the four new actions: `KeyQ → RollLeft`, `KeyE → RollRight` (via `with_one_to_many` or chained `insert`); `MouseMoveAxis::Y → Pitch` and `MouseMoveAxis::X → Yaw` (via `with_axis(action, axis_input)` per leafwing 0.20 `input_map.rs:172-173`). Final binding count grows from 6 → 10 (6 buttonlike for translation + 2 button for roll + 2 axis for pitch/yaw)

2. **Given** `TuningConfig` (`src/tuning/config.rs`) is the project's single canonical gameplay-tuning struct
   **When** Story 3.7 extends it
   **Then** two new fields are added in this order, AFTER the existing `ship_thrust_newtons` field: `pub mouse_sensitivity: f32` (default `1.0`) and `pub ship_torque_nm: f32` (default `80.0`)
   **And** both fields use the per-field `#[serde(default = "default_…")]` pattern matching the `outline_width` / `outline_color` / `ship_thrust_newtons` precedent (forward-compat — preserves deserialization of pre-3.7 tuning.ron snapshots)
   **And** two new top-level helpers are added alongside the existing default helpers: `fn default_mouse_sensitivity() -> f32 { 1.0 }` and `fn default_ship_torque_nm() -> f32 { 80.0 }`
   **And** `impl Default for TuningConfig` includes both new fields in its struct-literal (in the same order as the struct fields)
   **And** `assets/config/tuning.ron` gains two new lines after `ship_thrust_newtons: 500.0,`: `mouse_sensitivity: 1.0,` and `ship_torque_nm: 80.0,` (insert-at-end ordering matches Story 2.4 and Story 3.6 precedent; trailing commas are correct per RON-0.8 convention)
   **And** the existing 3 tests in `tuning::config::tests` are extended in-place — NO new test functions added:
   - `tuning_config_default_matches_ron_initial_values` gains two assertions: `assert_eq!(cfg.mouse_sensitivity, 1.0);` and `assert_eq!(cfg.ship_torque_nm, 80.0);`
   - `tuning_config_deserializes_from_ron_bytes` ron-bytes literal gains `, mouse_sensitivity: 0.5, ship_torque_nm: 120.0` and assertions `assert_eq!(cfg.mouse_sensitivity, 0.5);` + `assert_eq!(cfg.ship_torque_nm, 120.0);` (non-default values exercise the per-field deserializer; symmetric with the existing `ship_thrust_newtons: 750.0` non-default literal)
   - `tuning_config_legacy_schema_uses_defaults_for_added_fields` ron-bytes literal is unchanged (the absent fields exercise the serde-default fallback) and gains assertions `assert_eq!(cfg.mouse_sensitivity, 1.0);` + `assert_eq!(cfg.ship_torque_nm, 80.0);`

3. **Given** `apply_torque` is the rotation counterpart to Story 3.6's `apply_thrust` and architecture.md:411 prescribes a single `FlightSystems::ApplyForces` set for both linear and angular force application
   **When** Story 3.7 registers the system
   **Then** `apply_torque` is added to the existing `FlightSystems::ApplyForces` set in `FixedUpdate` via `app.add_systems(FixedUpdate, physics::apply_torque.in_set(FlightSystems::ApplyForces).run_if(in_state(GameState::Arena)));` — placed AFTER the existing `apply_thrust` registration in `FlightPlugin::build`
   **And** **NO** new `FlightSystems` enum variants are added (ApplyForces already covers both translation and rotation; per Story 3.6 AC #5 rationale: "linear and angular forces sum independently in Avian — no inter-set ordering needed")
   **And** the `run_if(in_state(GameState::Arena))` gate matches `apply_thrust`'s gate (rotation is also Arena-scoped — MainMenu / Paused / Loading do not advance physics)

4. **Given** the rotation system reads `ActionState<FlightAction>` and writes ship-LOCAL torque via Avian's `Forces` query
   **When** the system runs in `FixedUpdate` inside `FlightSystems::ApplyForces`
   **Then** the system signature is:
   ```rust
   pub fn apply_torque(
       tuning_assets: Res<Assets<TuningConfig>>,
       tuning_handle: Res<TuningHandle>,
       mut ships: Query<(Forces, &ActionState<FlightAction>), With<PlayerShip>>,
   )
   ```
   **And** the system body (a) extracts `mouse_sensitivity` + `ship_torque_nm` via the same cold-start fallback `tuning_assets.get(tuning_handle.0.id()).cloned().unwrap_or_default()` pattern as `apply_thrust`; (b) iterates the query (one match expected — same one-or-zero pattern as `apply_thrust`); (c) computes `local_torque = ship_local_torque_vector(action_state, mouse_sensitivity, ship_torque_nm)`; (d) applies via `forces.apply_local_torque(local_torque)` (Avian 0.6 API — verified at `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/forces/query_data.rs:374`; gated on `feature = "3d"` which our crate uses)
   **And** **NO** `warn!` or `info!` per-tick logs (60 Hz spam — same discipline as `apply_thrust`)
   **And** `apply_local_torque` is a no-op for `Vec3::ZERO` per its internal early-return (`if torque != AngularVector::ZERO`) — quiet ticks (no mouse motion, no Q/E) produce no torque accumulation

5. **Given** `ship_local_torque_vector` is the pure-logic helper symmetric to `ship_local_thrust_vector` (Story 3.6) — first-class unit-test target per architecture.md:353
   **When** authored in `src/flight/physics.rs` alongside the existing thrust helper
   **Then** the signature is:
   ```rust
   pub fn ship_local_torque_vector(
       action_state: &ActionState<FlightAction>,
       mouse_sensitivity: f32,
       ship_torque_nm: f32,
   ) -> Vec3
   ```
   **And** the body sums three independent contributions in ship-LOCAL space (downstream `apply_local_torque` handles the local→world transformation):
   - **Pitch (mouse Y axis):** read via `action_state.value(&FlightAction::Pitch)` (returns `f32` per leafwing 0.20 `action_state/mod.rs:632-641` for axis-kind actions). Contributes `Vec3::new(value * mouse_sensitivity, 0.0, 0.0)` — i.e., torque around local **+X** axis. **Sign convention:** Bevy 0.18's `AccumulatedMouseMotion::delta.y` is positive when the mouse moves DOWN (screen-space Y-down convention). Right-hand rule: rotating around local +X turns local +Y→+Z (up→back), which pitches the nose DOWN. Therefore `mouse_y > 0` (moved down) → `+X torque` → nose down ✓. `mouse_y < 0` (moved up) → `-X torque` → nose up ✓. The mapping is non-inverted out of the box; if smoke shows inverted feel, flip the sign of the Pitch contribution and document.
   - **Yaw (mouse X axis):** read via `action_state.value(&FlightAction::Yaw)`. Contributes `Vec3::new(0.0, -value * mouse_sensitivity, 0.0)` — i.e., torque around local **-Y** axis. **Sign convention:** `mouse_x > 0` (moved right) and right-hand rule for +Y axis: +Y torque turns +X→-Z (right→forward), which yaws LEFT. We want mouse-right → ship-yaw-RIGHT, so we negate: `-Y torque` for positive mouse_x → yaw right ✓.
   - **Roll (Q/E buttons):** when `RollLeft` is pressed, contribute `Vec3::new(0.0, 0.0, ship_torque_nm)`. When `RollRight` is pressed, contribute `Vec3::new(0.0, 0.0, -ship_torque_nm)`. **Sign convention:** Bevy convention — local forward is `-Z`, so local `+Z` is BACKWARD. Right-hand rule: rotating around +Z turns +X→+Y (right→up). Pilot looking down -Z sees this as the world rolling counter-clockwise — which feels like the ship rolling LEFT (the pilot's view tilts left). Therefore `RollLeft → +Z torque` ✓. If Q/E feel swapped in smoke, flip the two `Vec3::new(0.0, 0.0, ±)` literals and document.
   **And** the three contributions sum vector-style (no clamping — same precedent as Story 3.6's unclamped diagonal-thrust semantic; the epic's "forces sum" implicit invariant carries over to torque)
   **And** `Vec3::ZERO` is returned when no axis has a non-zero value AND neither roll button is pressed

6. **Given** the epic-3 line 196-197 acceptance bullet: "the mouse cursor is inspected → the cursor is confined (`CursorGrabMode::Confined`) and hidden (`visible: false`) so mouse motion maps cleanly to rotation"
   **When** the cursor-grab logic is wired
   **Then** a new pair of OnEnter/OnExit systems is added to `FlightPlugin::build` — registered to `OnEnter(GameState::Arena)` and `OnExit(GameState::Arena)`:
   ```rust
   app.add_systems(OnEnter(GameState::Arena), grab_cursor_for_arena);
   app.add_systems(OnExit(GameState::Arena), release_cursor_on_arena_exit);
   ```
   - `grab_cursor_for_arena` queries `Single<&mut CursorOptions, With<PrimaryWindow>>` (Bevy 0.18 — `CursorOptions` is a `#[require]`-attached component on the Window entity per `bevy_window-0.18.1/src/window.rs:163`; `PrimaryWindow` marker is in `bevy::window::PrimaryWindow`) and sets `cursor_options.grab_mode = CursorGrabMode::Confined; cursor_options.visible = false;`
   - `release_cursor_on_arena_exit` does the inverse: `cursor_options.grab_mode = CursorGrabMode::None; cursor_options.visible = true;`
   - **macOS platform note:** per `bevy_window-0.18.1/src/window.rs:754`, `CursorGrabMode::Confined` is unsupported on macOS — Bevy auto-falls-back to `CursorGrabMode::Locked`. The code path uses `Confined` literally (Bevy handles fallback internally); no `#[cfg(target_os = "macos")]` branch needed
   **And** the cursor-grab is naturally coupled to the pause cycle: `Arena → Paused` triggers `OnExit(Arena)` (cursor released for Esc-overlay UX) → `OnEnter(Paused)` (no cursor-grab change in 3.7 — cursor stays released). On resume: `Paused → Arena` triggers `OnExit(Paused)` (no change) → `OnEnter(Arena)` (cursor re-grabbed). This ALSO covers the focus-loss path (`pause_on_focus_loss` → `NextState(Paused)` → `OnExit(Arena)` releases the cursor — appropriate, since the user Alt-Tabbed away)
   **And** the two new systems are NOT placed in any `FlightSystems::*` SystemSet (state-transition systems are typically loose — no ordering concern with `spawn_player_ship` since cursor lives on the Window entity, not the ship)
   **And** **NO** `cleanup_on_exit::<...>` interaction (cursor is on the persistent Window entity — neither despawned nor `ArenaEntity`-tagged)

7. **Given** `pub mod input;` and `pub mod physics;` already exist in `src/flight/mod.rs:4-5` (Story 3.6) and architecture.md:558-563 prescribes responsibility-based file boundaries (NOT story-based)
   **When** Story 3.7 extends both files
   **Then** `src/flight/input.rs` gains four enum variants + four binding lines + (optionally) one `use` line for `MouseMoveAxis` (already in leafwing prelude, so no new use needed). Target file size: ~24 → ~40-50 lines.
   **And** `src/flight/physics.rs` gains the `apply_torque` system + `ship_local_torque_vector` helper + 4-5 new co-located unit tests. Target file size: ~104 → ~190-220 lines.
   **And** `src/flight/mod.rs` gains TWO new system registrations in `FlightPlugin::build` (the `apply_torque` add_systems + the OnEnter/OnExit cursor-grab pair) PLUS the two new system definitions (`grab_cursor_for_arena`, `release_cursor_on_arena_exit`) — total target: ~111 → ~140-150 lines (still under the 250-line split-trigger threshold for `mod.rs` itself)
   **And** **NO** new files are created (`flight/components.rs` and `flight/camera.rs` slots remain unintroduced per architecture.md:560,:563 — they land at Stories 3.8 dampener and a future cockpit-comfort polish story respectively)
   **And** **NO** new SystemSet variants (Story 3.6 AC #5 rationale carries over)
   **And** **NO** changes to `src/main.rs` (FlightPlugin is already registered; cursor-grab is a per-plugin concern, not an App-level concern)

8. **Given** `ship_local_torque_vector` is the only pure-logic surface added in 3.7 and the runtime smoke (Task 5) is the de-facto integration test for `apply_torque` (per Story 3.6 AC #8 precedent: "an automated `cargo test` of the velocity-after-2-seconds invariant would require a Bevy `App`-bootstrap integration test … Architecture.md:354 defers integration tests post-M3")
   **When** the helper is unit-tested
   **Then** `flight/physics.rs` gains 5 new co-located test functions in the existing `#[cfg(test)] mod tests` block (alongside the 3 thrust tests):
   - `no_input_returns_zero_torque` — `(no_input, sensitivity=1.0, torque=80.0)` → `Vec3::ZERO`
   - `pitch_axis_value_maps_to_local_x_torque` — set Pitch axis to 5.0 via `state.set_value(&FlightAction::Pitch, 5.0)`, sensitivity=2.0 → expect `Vec3::new(10.0, 0.0, 0.0)` (positive mouse_y → +X torque → nose-down sign)
   - `yaw_axis_value_maps_to_negative_local_y_torque` — set Yaw to 3.0, sensitivity=1.0 → expect `Vec3::new(0.0, -3.0, 0.0)` (positive mouse_x → -Y torque → yaw-right sign)
   - `roll_left_maps_to_positive_local_z_torque` — press RollLeft, torque_nm=80.0 → expect `Vec3::new(0.0, 0.0, 80.0)`
   - `pitch_plus_roll_right_sums_components` — set Pitch=2.0, sensitivity=1.0, press RollRight, torque_nm=80.0 → expect `Vec3::new(2.0, 0.0, -80.0)` (verifies the sum semantic AND the contrasting magnitudes between mouse-axis and roll-button contributions)
   **And** the existing 3 thrust tests are unchanged
   **And** the `pressed()` test helper is reused as-is; a new `pressed_with_axis(actions, [(axis, value), ...])` helper may be added if test ergonomics improve, OR each axis-test sets `state.set_value(&action, v)` inline (either pattern is acceptable)
   **And** Story 3.7 adds **5 net new test functions** — net post-3.7 test count: **29** (= 24 from end of 3.6 + 5 new from `flight/physics.rs`). AC #11 enforces N = 29 at verification time.

9. **Given** the epic-3 line 188-189 acceptance bullet: "the player presses Q for 1 second → the ship has rotated approximately `ship_torque_nm / moment_of_inertia` radians around its local +Z axis"
   **When** the dev runs the runtime smoke (Task 5) and tests Q/E roll-and-release
   **Then** the dev visually confirms the ship rolls left when Q is held and rolls right when E is held; the **quantitative** angular-velocity check is a Dev Agent Record observation rather than an automated test (same Story 3.6 AC #8 rationale: integration tests deferred post-M3)
   **And** the dev confirms angular velocity persists after release (no dampener — that's Story 3.8). Specifically, after holding Q for ~1 second and releasing, the ship continues to roll at a constant angular velocity until Story 3.8 lands the dampener
   **And** if observed roll feel is wildly off (e.g., spins instantly to maximum angular velocity and stops, or barely rotates), the dev exercises the "tune `ship_torque_nm` in tuning.ron" escape hatch — values 40.0 to 200.0 are reasonable to try; document the chosen value + reasoning in Dev Agent Record. Avian's auto-computed inertia for the 2 m radius `Collider::sphere(2.0)` (mass ≈ 33.5 kg) is `I_sphere = (2/5) * m * r² ≈ (0.4)(33.5)(4) ≈ 53.6 kg·m²`. At `ship_torque_nm = 80.0`, angular acceleration ≈ `80 / 53.6 ≈ 1.49 rad/s²`. After 1s of held Q: ω ≈ 1.49 rad/s ≈ 85°/s — feels reasonable for cockpit roll. The default is the starting point; smoke validates.

10. **Given** Story 3.6 AC #9 marked the "pause/resume cycle teleports PlayerShip" suspicion as ✅ FALSE-POSITIVE per Till's smoke (deferred-work.md:218 update)
    **When** Story 3.7's runtime smoke exercises the pause cycle
    **Then** the dev verifies that the FALSE-POSITIVE conclusion still holds for rotation: pressing Esc mid-roll (i.e., while the ship has non-zero `AngularVelocity` from a held Q press) and resuming should preserve the angular velocity — the ship continues to roll seamlessly
    **And** the dev separately verifies that the cursor is HIDDEN during Arena and VISIBLE during Paused / MainMenu (via OnEnter/OnExit cursor-grab systems from AC #6). Specifically:
    - Esc pause: cursor reappears on the pause overlay (so the player can see where their cursor is even though there's no clickable UI yet)
    - Esc resume: cursor disappears again
    - Window unfocus (Cmd-Tab on macOS, Alt-Tab elsewhere): cursor reappears (focus-loss → pause)
    - Window refocus: cursor disappears again (focus-gain → resume)
    **And** if the cursor-grab does not toggle correctly (e.g., cursor stays hidden in Paused, or stays visible in Arena), the dev investigates whether the `OnEnter(Arena)` / `OnExit(Arena)` systems are firing at all — `RUST_LOG=info` will not log this since the new cursor-grab systems should NOT add per-transition `info!` logs (cursor state-changes are too fine-grained for lifecycle-log discipline). Use a temporary `debug!` if needed to diagnose, then remove before commit
    **And** macOS-specific: `CursorGrabMode::Confined` falls back to `Locked` per Bevy 0.18 platform notes — both achieve the same FPS-cockpit feel (cursor cannot escape the window). Till develops on macOS, so this fallback is what he'll experience. Linux (X11/Wayland) and Windows users get true `Confined`. Document the platform divergence in Dev Agent Record if any deviation is observed; otherwise, assume the Bevy fallback works as documented.

11. **Given** the post-3.6 source baseline (test count = 24 per Story 3.6 Dev Agent Record; `cargo build --release` 0 warnings; `src/flight/mod.rs` = 111 lines; `src/flight/input.rs` = 25 lines; `src/flight/physics.rs` = 104 lines; `src/tuning/config.rs` = 121 lines; `assets/config/tuning.ron` = 8 lines)
    **When** Story 3.7 verification runs locally (per `feedback_full_build_output.md` discipline — exit-0 + tail is NOT proof; grep for `warning:|error:` per command, capture each to `/tmp/story-3-7-<command>.log`)
    **Then** **all six** of `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --release` produce **0** lines matching `grep -cE 'warning:|error:'` per their respective full output logs
    **And** `cargo test` summary line reads `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N = 29** (= 24 baseline + 5 new `ship_local_torque_vector` tests in `flight/physics.rs`; the +0 deltas in `tuning/config.rs` per AC #2 are expected)
    **And** the runtime smoke (Task 5) verifies all of: (a) mouse Y up → ship pitches up; (b) mouse Y down → ship pitches down; (c) mouse X right → ship yaws right; (d) mouse X left → ship yaws left; (e) Q held → ship rolls left; (f) E held → ship rolls right; (g) released keys → ship continues to rotate (Newtonian — no dampener); (h) Q + mouse-up simultaneously → ship rolls AND pitches (vector-sum semantic for torque, mirrors Story 3.6 W+D thrust semantic); (i) ship-local pitch behaviour: with the ship rolled 90° via Q-hold, mouse-up should pitch the ship in its NEW orientation (i.e., ship-local pitch, not world-up pitch); (j) cursor disappears in Arena and reappears in Paused (Esc-overlay) and on focus loss (Cmd-Tab away); (k) Esc pause overlay still works (Story 3.4 regression); (l) clean window-close (no panic on shutdown — cursor-grab release on app-exit handled by Bevy/winit auto-cleanup, no app-side fix needed)
    **And** `/tmp/story-3-7-run.log` contains: 1 occurrence of `entered Loading`, 1 of `entered MainMenu`, ≥ 1 of `entered Arena`, ≥ 1 of `spawned PlayerShip` (Story 3.6 AC #9 false-positive holds → exactly 1 per Arena entry), 0 of `panic|backtrace|FATAL`, 0 of `ambiguous.*camera.*order` (Story 3.5 regression check), 0 of `ERROR.*avian|WARN.*Avian`
    **And** `git status --short` final set is **exactly**: `M src/flight/input.rs` (M — extended for 4 new variants + 4 new bindings), `M src/flight/mod.rs` (M — added apply_torque registration + OnEnter/OnExit cursor-grab systems + 2 new system definitions), `M src/flight/physics.rs` (M — added apply_torque system + ship_local_torque_vector helper + 5 new tests), `M src/tuning/config.rs` (M — 2 new fields + 2 new helpers + Default extended + 3 test extensions), `M assets/config/tuning.ron` (M — 2 new lines), `M _bmad-output/implementation-artifacts/sprint-status.yaml` (M — bookkeeping), `M _bmad-output/implementation-artifacts/3-7-flight-input-3-axis-rotation-pitch-yaw-roll.md` (M — this file's Status flip + Dev Agent Record), and `M _bmad-output/implementation-artifacts/deferred-work.md` ONLY IF a new entry surfaces during impl (none anticipated; AC #10's pause-rotation observation can be a Dev Agent Record entry without a new deferred-work line); **NO** entries under `Cargo.toml` (no dep added), `Cargo.lock` (leafwing's transitive deps already locked), `src/main.rs` (no plugin re-registration), `src/state.rs` (per deferred-work.md:198, `Copy` derive remains deferred — `run_if(in_state(...))` handles cloning internally, and the new OnEnter/OnExit systems don't need to read GameState), `src/arena/**`, `src/pause/**`, `src/visual/**`, `src/ui/**`, `src/splash.rs`, `src/logging.rs`, `assets/strings/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`

## Tasks / Subtasks

- [x] **Task 1: Extend `src/flight/input.rs` — 4 new FlightAction variants + bindings + axis attribute** (AC: #1)
  - [x] In `src/flight/input.rs`, append four variants AT THE END of the `FlightAction` enum, in this order: `Pitch, Yaw, RollLeft, RollRight`. Final variant order matches AC #1's spec.
  - [x] Add `#[actionlike(Axis)]` attribute on the `Pitch` variant AND the `Yaw` variant (mark them as `InputControlKind::Axis` per leafwing 0.20 — matches `~/.cargo/registry/src/.../leafwing-input-manager-0.20.0/src/lib.rs:78-100` doc-example pattern). `RollLeft` and `RollRight` need NO attribute (default Button kind).
  - [x] Extend `default_input_map()` with four new bindings. Two API styles are valid:
    ```rust
    pub fn default_input_map() -> InputMap<FlightAction> {
        InputMap::new([
            (FlightAction::ThrustForward, KeyCode::KeyW),
            (FlightAction::ThrustReverse, KeyCode::KeyS),
            (FlightAction::StrafeLeft, KeyCode::KeyA),
            (FlightAction::StrafeRight, KeyCode::KeyD),
            (FlightAction::ThrustUp, KeyCode::Space),
            (FlightAction::ThrustDown, KeyCode::ControlLeft),
            (FlightAction::RollLeft, KeyCode::KeyQ),
            (FlightAction::RollRight, KeyCode::KeyE),
        ])
        .with_axis(FlightAction::Pitch, MouseMoveAxis::Y)
        .with_axis(FlightAction::Yaw, MouseMoveAxis::X)
    }
    ```
    - **Why two-stage construction:** `InputMap::new(slice-of-tuples)` only handles the buttonlike `(Action, KeyCode)` pairs. Axis bindings require `with_axis(action, axis_input)` (per leafwing-0.20 `input_map.rs:172-173`), which takes `impl Axislike` (e.g., `MouseMoveAxis::Y`). The two-stage pattern matches leafwing-0.20's `examples/axis_inputs.rs:31-40` exactly.
    - **Why `MouseMoveAxis::Y`/`X` not `MouseMove::default()`:** the latter is a `DualAxislike` (Vec2-output) — would require a single `Look` (DualAxis) variant, but the epic spec wording "Pitch, Yaw" maps cleaner to two `Axis` variants. See the deviation-deferred discussion in Dev Notes.
  - [x] **Imports:** `MouseMoveAxis` is in the leafwing prelude (`use leafwing_input_manager::prelude::*;` is already imported at `src/flight/input.rs:4`). NO new imports needed unless the dev opts to use explicit imports (project convention is the wildcard prelude per Story 3.6 anti-pattern #2).
  - [x] **No tests added to `input.rs`** — same Story 3.6 reasoning: the binding-map content is configuration data trivially correct by inspection, and runtime-verified via Task 5's smoke. A test like `assert_eq!(default_input_map().get_axislike(&FlightAction::Pitch).next(), Some(MouseMoveAxis::Y))` is tautological — re-encoding configuration in two places.
  - [x] **Verify post-edit:** `cargo check` produces 0 warnings/errors. The `#[actionlike(Axis)]` attribute on Pitch/Yaw must compile cleanly — if leafwing 0.20's macro errors on attribute placement (e.g., requires `#[actionlike(Axis)]` BEFORE the variant ident, not after), follow the macro's error message. The leafwing-0.20 doc-example at `lib.rs:91-99` is the canonical reference for attribute syntax.

- [x] **Task 2: Extend `src/tuning/config.rs` — `mouse_sensitivity` + `ship_torque_nm` fields + Default impl + tuning.ron + 3 test extensions** (AC: #2)
  - [x] In `src/tuning/config.rs`, add two `pub` fields to the `TuningConfig` struct, AFTER the `ship_thrust_newtons` field (insert-at-end ordering per Story 2.4/3.6 precedent). Annotate each with its own `#[serde(default = "default_…")]`:
    ```rust
    #[serde(default = "default_mouse_sensitivity")]
    pub mouse_sensitivity: f32,
    #[serde(default = "default_ship_torque_nm")]
    pub ship_torque_nm: f32,
    ```
  - [x] Add the two helper functions alongside the existing `default_outline_width` / `default_outline_color` / `default_ship_thrust_newtons` helpers:
    ```rust
    fn default_mouse_sensitivity() -> f32 {
        1.0
    }

    fn default_ship_torque_nm() -> f32 {
        80.0
    }
    ```
  - [x] Update `impl Default for TuningConfig`'s struct-literal: append `mouse_sensitivity: default_mouse_sensitivity(),` and `ship_torque_nm: default_ship_torque_nm(),` as the last two fields (in struct-field order).
  - [x] In `assets/config/tuning.ron`, append two new lines AFTER `ship_thrust_newtons: 500.0,` and BEFORE the closing `)` paren:
    ```
    mouse_sensitivity: 1.0,
    ship_torque_nm: 80.0,
    ```
    Trailing commas are correct per RON-0.8 convention. Final file size: 10 lines (was 8).
  - [x] Extend the existing 3 tests in-place per AC #2:
    - `tuning_config_default_matches_ron_initial_values` (config.rs:87-95): add as the last two assertions:
      ```rust
      assert_eq!(cfg.mouse_sensitivity, 1.0);
      assert_eq!(cfg.ship_torque_nm, 80.0);
      ```
    - `tuning_config_deserializes_from_ron_bytes` (config.rs:98-108): edit the bytes literal to add `, mouse_sensitivity: 0.5, ship_torque_nm: 120.0` BEFORE the closing `)`. Add as the last two assertions:
      ```rust
      assert_eq!(cfg.mouse_sensitivity, 0.5);
      assert_eq!(cfg.ship_torque_nm, 120.0);
      ```
    - `tuning_config_legacy_schema_uses_defaults_for_added_fields` (config.rs:111-120): bytes literal is unchanged (the absent fields exercise the serde-default fallback). Add as the last two assertions:
      ```rust
      assert_eq!(cfg.mouse_sensitivity, 1.0);
      assert_eq!(cfg.ship_torque_nm, 80.0);
      ```
  - [x] **Verify post-edit:** `cargo test --lib tuning::config` produces 3 passing tests with the additional assertions — same count as pre-3.7, just enriched. Project test count unchanged after Task 2 alone (still 24); the 5 new tests land in Task 4 with `flight/physics.rs`.

- [x] **Task 3: Wire `apply_torque` registration + cursor-grab OnEnter/OnExit systems in `src/flight/mod.rs`** (AC: #3, #6)
  - [x] In `src/flight/mod.rs`, add a use line for the Bevy primary-window marker at the top of the use block (alphabetical after `bevy::prelude::*;`):
    ```rust
    use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
    ```
    (Reason: `CursorOptions` and `CursorGrabMode` live in `bevy::window`, not in the bevy prelude per `bevy_window-0.18.1/src/lib.rs:1-30`; `PrimaryWindow` is the marker component for the OS-primary window per `bevy_window-0.18.1/src/window.rs`.)
  - [x] In `FlightPlugin::build`, append AFTER the existing `app.add_systems(FixedUpdate, physics::apply_thrust...)` block:
    ```rust
    app.add_systems(
        FixedUpdate,
        physics::apply_torque
            .in_set(FlightSystems::ApplyForces)
            .run_if(in_state(GameState::Arena)),
    );
    app.add_systems(OnEnter(GameState::Arena), grab_cursor_for_arena);
    app.add_systems(OnExit(GameState::Arena), release_cursor_on_arena_exit);
    ```
    - **Why no `.in_set(FlightSystems::Setup)` for the cursor systems:** OnEnter/OnExit transition systems are typically loose unless ordering matters; cursor-grab is independent of `spawn_player_ship` (the cursor lives on the persistent Window entity, not on the ship). Adding it to `FlightSystems::Setup` would force a chain that doesn't need to exist.
  - [x] At the bottom of `src/flight/mod.rs` (after `spawn_player_ship`), add the two cursor-grab systems:
    ```rust
    pub fn grab_cursor_for_arena(
        mut window: Single<&mut CursorOptions, With<PrimaryWindow>>,
    ) {
        window.grab_mode = CursorGrabMode::Confined;
        window.visible = false;
    }

    pub fn release_cursor_on_arena_exit(
        mut window: Single<&mut CursorOptions, With<PrimaryWindow>>,
    ) {
        window.grab_mode = CursorGrabMode::None;
        window.visible = true;
    }
    ```
    - **Why `Single<&mut CursorOptions, With<PrimaryWindow>>` not `Query<>`:** Bevy 0.18's `Single<>` query data extracts exactly one match; if there is no PrimaryWindow (impossible in a normal app with `DefaultPlugins`), the system silently no-ops via `Single`'s `Option`-style optionality. If `Single` doesn't exist as a system parameter in 0.18, fall back to `Query<&mut CursorOptions, With<PrimaryWindow>>` + `for window in &mut query { ... }` (functionally equivalent for the 1-window case). Verify at `cargo check` time.
    - **Why no `info!` logs:** cursor-grab state-changes are too fine-grained for the lifecycle-log discipline (per architecture.md:376-380; existing OnEnter(Arena) already logs `entered Arena` via `log_arena_entered` at `src/main.rs:54`; doubling that with a cursor-state log would be noise).
  - [x] **Verify post-edit:** `cargo check` produces 0 warnings/errors. Specifically watch for:
    - `Single<...>` system-parameter availability in Bevy 0.18 (it should be present per `bevy_ecs::system::Single`).
    - `CursorOptions` query — Bevy 0.18 places it on the Window entity via `#[require(CursorOptions)]`, so `Query<&mut CursorOptions, With<PrimaryWindow>>` is the correct pattern (NOT `Window` field-access — `cursor_options` is a separate component since Bevy 0.13).

- [x] **Task 4: Author `apply_torque` system + `ship_local_torque_vector` helper + 5 unit tests in `src/flight/physics.rs`** (AC: #4, #5, #8)
  - [x] At the top of `src/flight/physics.rs`, the existing `use` block already imports `avian3d::prelude::*;` (Story 3.6) — `apply_local_torque` is exposed via the `Forces` query data already in scope. NO new imports needed beyond what 3.6 brought.
  - [x] Implement the helper `ship_local_torque_vector` BELOW the existing `ship_local_thrust_vector` helper:
    ```rust
    /// Sum of pressed-rotation actions in ship-LOCAL space.
    /// Pitch contributes around local +X (positive mouse_y → +X torque → nose down per right-hand rule).
    /// Yaw contributes around local -Y (positive mouse_x → -Y torque → yaw right per right-hand rule).
    /// Roll contributes around local ±Z (RollLeft → +Z, RollRight → -Z; +Z is local backward in Bevy).
    /// Returns Vec3::ZERO if no axis has a non-zero value AND neither roll button is pressed.
    pub fn ship_local_torque_vector(
        action_state: &ActionState<FlightAction>,
        mouse_sensitivity: f32,
        ship_torque_nm: f32,
    ) -> Vec3 {
        let mut torque = Vec3::ZERO;
        let pitch = action_state.value(&FlightAction::Pitch);
        let yaw = action_state.value(&FlightAction::Yaw);
        torque.x += pitch * mouse_sensitivity;
        torque.y += -yaw * mouse_sensitivity;
        if action_state.pressed(&FlightAction::RollLeft) {
            torque.z += ship_torque_nm;
        }
        if action_state.pressed(&FlightAction::RollRight) {
            torque.z -= ship_torque_nm;
        }
        torque
    }
    ```
    - **Why `value()` for axis-kind actions:** leafwing-0.20 `action_state/mod.rs:632-641` — `value(action)` returns the f32 axis value (debug_assert checks the action is `InputControlKind::Axis`, which is satisfied by the `#[actionlike(Axis)]` attribute on Pitch/Yaw).
    - **Why no `clamp()` on mouse axis values:** the AC explicitly mirrors Story 3.6's "forces sum, magnitude unclamped" semantic. A frame with a 100-pixel mouse flick produces a 100*sensitivity-N·m torque — that's intentional (translates to a snap-aim feel). If smoke playtest shows the snap is too aggressive, the dev can clamp via `pitch.clamp(-MAX_DELTA, MAX_DELTA)` or via leafwing's input-pipeline `with_processor(...)` — defer this tuning to a future story unless smoke explicitly demands it.
    - **Why `pressed()` for roll buttons (not `just_pressed`):** roll should be CONTINUOUS while held, mirroring `apply_thrust`'s use of `pressed()` for thrust. `just_pressed()` would only fire on the press-edge tick (the ship would barely roll).
  - [x] Implement the system `apply_torque` BELOW `apply_thrust` (and below `ship_local_torque_vector`):
    ```rust
    pub fn apply_torque(
        tuning_assets: Res<Assets<TuningConfig>>,
        tuning_handle: Res<TuningHandle>,
        mut ships: Query<(Forces, &ActionState<FlightAction>), With<PlayerShip>>,
    ) {
        let tuning = tuning_assets
            .get(tuning_handle.0.id())
            .cloned()
            .unwrap_or_default();
        for (mut forces, action_state) in &mut ships {
            let local_torque = ship_local_torque_vector(
                action_state,
                tuning.mouse_sensitivity,
                tuning.ship_torque_nm,
            );
            // apply_local_torque is a no-op for Vec3::ZERO (avoids waking sleeping bodies).
            forces.apply_local_torque(local_torque);
        }
    }
    ```
    - **Why iteration not `single_mut()`:** same as `apply_thrust` (Story 3.6 AC #7) — handles 0-ship case (no panic) and 1-ship case (the only expected case). This is a documented PATTERN DEVIATION per architecture.md:454, established by 3.6.
    - **Why no `warn!` on cold-start tuning-not-loaded:** at 60 Hz this would emit 60 warns/sec. The warn lives at spawn time in `spawn_player_ship` (Story 3.5/3.6). If tuning.ron loads mid-Arena (highly unlikely given Startup-phase load), the system silently switches to the new value.
  - [x] Inside the existing `#[cfg(test)] mod tests` block at the bottom of `physics.rs`, add 5 new test functions BELOW the 3 existing thrust tests. Reuse the existing `pressed()` helper. Optionally add a small helper for axis-value setup, OR inline the calls:
    ```rust
    fn pressed_with_axes(buttons: &[FlightAction], axes: &[(FlightAction, f32)]) -> ActionState<FlightAction> {
        let mut state = ActionState::default();
        for &b in buttons {
            state.press(&b);
        }
        for &(axis, value) in axes {
            state.set_value(&axis, value);
        }
        state
    }

    #[test]
    fn no_input_returns_zero_torque() {
        let v = ship_local_torque_vector(&no_input(), 1.0, 80.0);
        assert_eq!(v, Vec3::ZERO);
    }

    #[test]
    fn pitch_axis_value_maps_to_local_x_torque() {
        let state = pressed_with_axes(&[], &[(FlightAction::Pitch, 5.0)]);
        let v = ship_local_torque_vector(&state, 2.0, 80.0);
        // pitch=5.0, sensitivity=2.0 → +X torque of 10.0 (positive mouse_y → +X → nose down)
        assert!((v - Vec3::new(10.0, 0.0, 0.0)).length() < 1e-5, "got {:?}", v);
    }

    #[test]
    fn yaw_axis_value_maps_to_negative_local_y_torque() {
        let state = pressed_with_axes(&[], &[(FlightAction::Yaw, 3.0)]);
        let v = ship_local_torque_vector(&state, 1.0, 80.0);
        // yaw=3.0, sensitivity=1.0 → -Y torque of 3.0 (positive mouse_x → -Y → yaw right)
        assert!((v - Vec3::new(0.0, -3.0, 0.0)).length() < 1e-5, "got {:?}", v);
    }

    #[test]
    fn roll_left_maps_to_positive_local_z_torque() {
        let state = pressed(&[FlightAction::RollLeft]);
        let v = ship_local_torque_vector(&state, 1.0, 80.0);
        // RollLeft → +Z torque of magnitude ship_torque_nm (right-hand rule + Bevy local +Z = backward)
        assert!((v - Vec3::new(0.0, 0.0, 80.0)).length() < 1e-5, "got {:?}", v);
    }

    #[test]
    fn pitch_plus_roll_right_sums_components() {
        let state = pressed_with_axes(
            &[FlightAction::RollRight],
            &[(FlightAction::Pitch, 2.0)],
        );
        let v = ship_local_torque_vector(&state, 1.0, 80.0);
        // Pitch contributes (2, 0, 0); RollRight contributes (0, 0, -80); sum = (2, 0, -80)
        assert!((v - Vec3::new(2.0, 0.0, -80.0)).length() < 1e-5, "got {:?}", v);
    }
    ```
    - **Why `set_value()` on the axis actions in tests:** leafwing-0.20 exposes `set_value()` as a public test-friendly method per `action_state/mod.rs:643-650` (debug_assert checks the action is `InputControlKind::Axis`). The Pitch/Yaw variants must be tagged `#[actionlike(Axis)]` for this to not panic in debug builds — if Task 1 forgot the attribute, these tests will fail with a debug_assert panic; Task 1's verification step catches that.
    - **Test count delta: +5** → project total after this task: **29** (= 24 baseline + 5 new). Matches AC #11 enforcement.
  - [x] **Verify post-edit:** `cargo test physics::tests` produces 8 passing tests (3 thrust + 5 torque). `cargo clippy --all-targets -- -D warnings` produces 0 issues.

- [ ] **Task 5: Local verification sweep — full `feedback_full_build_output.md` discipline** (AC: #11)

  Per Till's memory `feedback_full_build_output.md`: `cargo check` exit-0 + tail is NOT proof of correctness. Capture each command's full output to a log file, then grep for `warning:|error:` and confirm count is **0**.

  - [x] `cargo check 2>&1 | tee /tmp/story-3-7-check.log` — confirm `grep -cE 'warning:|error:' /tmp/story-3-7-check.log` returns **0**.
  - [x] `cargo build 2>&1 | tee /tmp/story-3-7-build.log` — confirm grep returns **0**.
  - [x] `cargo test 2>&1 | tee /tmp/story-3-7-test.log` — confirm grep returns **0** AND the summary line reads `test result: ok. 29 passed; 0 failed; 0 ignored; ...`. Test count: 29 = 24 baseline + 5 from `flight/physics.rs`.
  - [x] `cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-3-7-clippy.log` — confirm grep returns **0**.
  - [x] `cargo fmt --all -- --check 2>&1 | tee /tmp/story-3-7-fmt.log` — confirm exit code 0. If fmt drift exists, run `cargo fmt --all`, re-stage, and re-run `--check`.
  - [x] `cargo build --release 2>&1 | tee /tmp/story-3-7-release.log` — confirm grep returns **0**. Allow 4–6 min wall time on the LTO=fat + codegen-units=1 release build.
  - [x] **Cargo.lock delta check:** `git diff --stat Cargo.lock` should show **no changes** (no new deps; leafwing/Avian/Bevy/outline/kira all unchanged). If Cargo.lock churns, investigate before committing.
  - [x] **Runtime smoke** — `RUST_LOG=info,wgpu=warn,naga=warn,avian3d=info cargo run 2>&1 | tee /tmp/story-3-7-run.log` — let the game reach Arena, then exercise:
    - (a) **Mouse Y up** (mouse moved toward top of screen) → ship pitches UP (cockpit horizon drops). If feel is inverted, flip the sign of the Pitch contribution in `ship_local_torque_vector` and document.
    - (b) **Mouse Y down** → ship pitches DOWN.
    - (c) **Mouse X right** → ship yaws RIGHT (asteroids slide left across cockpit view).
    - (d) **Mouse X left** → ship yaws LEFT.
    - (e) **Hold Q for ~1 sec** → ship rolls LEFT (cockpit horizon tilts; right wing goes up). Release → angular velocity persists (no dampener — that's 3.8).
    - (f) **Hold E for ~1 sec** → ship rolls RIGHT. Release → angular velocity persists.
    - (g) **W + Q held simultaneously** → ship moves forward AND rolls (vector-sum semantic — translation and rotation are independent).
    - (h) **Mouse-up + Q held** → ship pitches AND rolls (vector-sum semantic for torque).
    - (i) **Roll 90° via Q-hold, then mouse-up** → after the roll, "pitch up" is in the SHIP's new frame (ship-local pitch, not world-up pitch). Visually: with the ship rolled 90° left, mouse-up should move the ship UP relative to its current banked orientation, NOT relative to world-up.
    - (j) **Cursor visibility check:**
      - At MainMenu: cursor visible (no Arena cursor-grab applied).
      - On pressing Enter (MainMenu → Arena): cursor disappears (grab applied).
      - On pressing Esc (Arena → Paused): cursor reappears.
      - On pressing Esc again (Paused → Arena): cursor disappears.
      - On Cmd-Tab away (focus loss → pause): cursor reappears in another window.
      - On Cmd-Tab back (focus gain → resume): cursor disappears again.
    - (k) **Esc → pause overlay** appears (Story 3.4 still works; cursor reappears for the overlay).
    - (l) **Esc again → resume**; cursor re-grabs; the ship's prior angular velocity is preserved (per Story 3.6 AC #9 false-positive; Story 3.7 AC #10 re-confirms for rotation).
    - (m) **Quit cleanly** (window-close). No panic. Cursor-grab release on app-exit handled by Bevy/winit auto-cleanup.
  - [x] **Post-runtime grep:**
    - `grep -c 'entered Loading'` → **1**
    - `grep -c 'entered MainMenu'` → **1**
    - `grep -c 'entered Arena'` → **1** initial; **≥ 2** if pause cycle was exercised (1 initial + 1 resume per cycle). Each Arena entry should also produce a `spawned PlayerShip` line — verify the equality `entered Arena == spawned PlayerShip`.
    - `grep -c 'spawned PlayerShip'` → **1** initial; **≥ 2** if pause cycle was exercised. Equality with `entered Arena` count confirms Story 3.6 AC #9's false-positive holds (no double-spawn).
    - `grep -cE 'panic|backtrace|FATAL'` → **0**
    - `grep -ci 'ambiguous.*camera.*order'` → **0** (Story 3.5 regression check).
    - `grep -cE 'ERROR.*avian|WARN.*Avian'` → **0**
  - [x] Confirm the 3 pre-existing documented WARNs from Story 3.5/3.6 reappear unchanged (splash-cleanup race per deferred-work.md:139, wgpu fragment-output per Story 2.3, winit Skipped Destroyed per Story 1.6). If a fourth WARN appears, investigate and either explain it in Dev Agent Record or add a deferred-work entry.

- [x] **Task 6: Update `_bmad-output/implementation-artifacts/deferred-work.md` IF NEEDED** (AC: #10)
  - [x] Story 3.7 anticipates **NO** new deferred-work entries — the design fully consumes the rotation epic spec, and AC #10's pause-cycle re-confirmation is a Dev Agent Record observation, not a deferral. **Outcome: NO updates needed** — Till's smoke confirmed all sign conventions correct on first try, no clamp/inversion friction surfaced, cursor-grab toggled cleanly across 11 Esc-pauses and 7 focus-loss pauses with zero new WARNs.
  - [x] **Conditional entries** — add ONLY if surfaced during impl: **none surfaced.**
    - **Mouse-axis pipeline saturation:** if the unclamped mouse-delta magnitude produces a "snap-to-aim" feel that's too aggressive in playtest, defer the clamp/processor decision to a future tuning story (likely 3.8 dampener context, since dampener interacts with angular velocity). Format:
      ```
      ## Deferred from: 3-7-flight-input-3-axis-rotation-pitch-yaw-roll (2026-XX-XX)
      - **Mouse-delta clamp / sensitivity curve** — `src/flight/physics.rs:ship_local_torque_vector`. The unclamped mouse-axis-to-torque mapping makes large mouse flicks produce snap-aim torque spikes. **Resolution path:** add a `MouseSensitivityCurve` enum in TuningConfig (Linear / Quadratic / Clamped(max)) and apply via leafwing's `with_processor(...)` pipeline OR direct clamp inside `ship_local_torque_vector`. Defer to a dedicated mouse-feel-tuning story between 3.8 (dampener) and 3.10 (collision damage), since dampener will mask some of the snap-aim feel and may make the clamp unnecessary.
      ```
    - **Pitch / yaw inversion preference:** if Till prefers inverted pitch (mouse-up = pitch-down, the legacy flight-sim convention), this becomes a future FR37 settings UI line item. Format:
      ```
      ## Deferred from: 3-7-flight-input-3-axis-rotation-pitch-yaw-roll (2026-XX-XX)
      - **Pitch invert toggle (FR37 settings)** — `src/flight/physics.rs:ship_local_torque_vector`. Story 3.7 ships non-inverted (mouse-up → pitch-up); inverted-pitch fans need a Settings toggle. **Resolution path:** add `pitch_inverted: bool` to TuningConfig (default false), multiply the Pitch contribution by `if inverted { -1.0 } else { 1.0 }`. Defer to Epic 4 Story 4.8 (Settings menu — master/SFX volume + mouse sensitivity), where the FR37 surface lands.
      ```
    - **Cursor-grab not applied on first MainMenu → Arena:** if the OnEnter(Arena) cursor-grab system fires before the Window entity is fully spawned (race condition), the cursor may not lock until the next state transition. Format:
      ```
      ## Deferred from: 3-7-flight-input-3-axis-rotation-pitch-yaw-roll (2026-XX-XX)
      - **Primary window cursor-grab race on first Arena entry** — `src/flight/mod.rs:grab_cursor_for_arena`. ...
      ```
      (Only if observed in smoke; otherwise omit.)

- [~] **Task 7: Sprint-status bookkeeping + commit/push (NOT YET — await Till's authorization)** (per Story 3.5/3.6 precedent)
  <!-- Sprint-status flipped to in-progress; review-flip + commit/push subtasks below remain unchecked pending Till's runtime smoke + explicit commit authorization (Stories 3.1–3.6 cadence). -->

  - [ ] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - [x] `3-7-flight-input-3-axis-rotation-pitch-yaw-roll: ready-for-dev → in-progress` — flipped 2026-05-01 at start of dev-story.
    - [x] `3-7-flight-input-3-axis-rotation-pitch-yaw-roll: in-progress → review` — flipped 2026-05-01 after Till's runtime-smoke confirmation.
    - [x] `last_updated:` bumped to `2026-05-01 (Story 3.7 in-progress → review — 3-axis rotation thrust + cursor grab verified)`.
  - [x] Update this story file's `Status:` field at line 3. Flipped `ready-for-dev → in-progress → review` after Till's runtime-smoke confirmation: all (a)–(m) sub-bullets verified working, sign conventions correct on first try, no clamp/inversion friction.
  - [x] Populate the `## Dev Agent Record` section: `Agent Model Used`, `Debug Log References` (the 7 commands' grep counts table), `Completion Notes List` (one bullet per AC #1–#11), `File List` (Modified: `src/flight/input.rs`, `src/flight/physics.rs`, `src/flight/mod.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`, `sprint-status.yaml`, this file).
  - [ ] **Commit 1 (feat):** stage `src/flight/input.rs`, `src/flight/physics.rs`, `src/flight/mod.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`. Message: `feat: 3-axis rotation + cursor grab (Story 3.7)`. **DO NOT COMMIT UNLESS TILL AUTHORIZES.**
  - [ ] **Commit 2 (bmad):** stage `_bmad-output/implementation-artifacts/sprint-status.yaml`, `_bmad-output/implementation-artifacts/3-7-flight-input-3-axis-rotation-pitch-yaw-roll.md`, AND `_bmad-output/implementation-artifacts/deferred-work.md` IF a new entry was added. Message: `bmad: story 3.7 ready-for-dev → review (3-axis rotation)`. **DO NOT COMMIT UNLESS TILL AUTHORIZES.**
  - [ ] **DO NOT push.** Push happens only after explicit authorization, AND only after Story 3.7 code review (`bmad-code-review`) passes per Story 3.5/3.6 precedent.

### Review Findings

Code review run 2026-05-04 — three parallel layers (Blind Hunter, Edge Case Hunter, Acceptance Auditor). Findings classified after dedup.

**Decision-needed (3) — RESOLVED 2026-05-04:**

- [x] [Review][Decision→Patch] **FixedUpdate-tick rate vs per-frame mouse-delta mismatch** — Resolved as Patch (option b): added `MouseLookDelta` resource, new `accumulate_mouse_look` system in `PreUpdate` (Arena-gated) that adds per-frame `AccumulatedMouseMotion::delta` into the buffer; `apply_torque` reads and drains the buffer each FixedUpdate tick. Total angular impulse over 1 s is now independent of render framerate. Combined with D2 (single coherent change). Source: blind+edge.
- [x] [Review][Decision→Patch] **First-frame torque spike on cursor grab acquisition / Paused→Arena resume** — Resolved as Patch (option b combined with D1 design): added `MouseLookSuppressFrames` resource; `grab_cursor_for_arena` zeroes `MouseLookDelta` and sets suppress=3 so the next 3 PreUpdate accumulations (covering the OS cursor-warp delta) are dropped. Roll torque from Q/E is unaffected during the suppression window. Source: edge.
- [x] [Review][Decision→Dismiss] **`apply_torque` bundled into `apply_thrust` tuple deviates from literal AC #3 wording** — Dismissed (option 1): idiomatic Bevy 0.18 pattern accepted; deviation already documented in Dev Agent Record. Source: auditor.

**Patch (3 → 5 after decision-resolution) — APPLIED 2026-05-04:**

- [x] [Review][Patch] **P3: X11 cursor-fallback comment misleading** [src/flight/mod.rs:117-118] — applied: rewritten to "native on Windows / X11; on macOS Bevy auto-falls-back to Locked".
- [x] [Review][Patch] **P4: Stale module doc comment in `flight/mod.rs`** [src/flight/mod.rs:1-2] — applied: generalized to mention 3-axis rotation + cursor grab.
- [x] [Review][Patch] **P5: Q+E simultaneous roll cancels to zero — undocumented and untested** [src/flight/physics.rs tests block] — applied: added unit test `roll_left_plus_roll_right_cancels_to_zero`. Test count 29 → 30.
- [x] [Review][Patch] **P1 (from D1): Mouse-look accumulator decouples FixedUpdate from per-frame mouse delta** [src/flight/physics.rs, src/flight/mod.rs] — applied: see D1 resolution. Helper `ship_local_torque_vector` signature changed to take `mouse_pitch: f32, mouse_yaw: f32` directly; 5 existing rotation tests updated; `pressed_with_axes` test helper removed (unused after signature change).
- [x] [Review][Patch] **P2 (from D2): Cursor-warp delta suppressed on grab** [src/flight/mod.rs grab_cursor_for_arena] — applied: see D2 resolution.

**Defer (5)** — appended to deferred-work.md:

- [x] [Review][Defer] **No NaN/inf guard on mouse axis or tuning scalars** [src/flight/physics.rs:64-81] — deferred, pre-existing pattern (thrust system has same gap).
- [x] [Review][Defer] **`Single<&mut CursorOptions, With<PrimaryWindow>>` silently skips on zero/multi window with no diagnostic log** [src/flight/mod.rs:116, 123] — deferred, spec-prescribed parameter; correct Bevy 0.18 behavior is silent skip (not panic); add diagnostic logging when headless tests or multi-window stories arrive.
- [x] [Review][Defer] **Cursor policy undefined for PhotoMode/Caravan/PostRun** [src/flight/mod.rs:58-59] — deferred, forward-compat; future stories must wire equivalent OnEnter/OnExit handlers.
- [x] [Review][Defer] **No range/sign validation on `mouse_sensitivity` / `ship_torque_nm` deserialization** [src/tuning/config.rs:23-27, 41-47] — deferred, pre-existing pattern (`ship_thrust_newtons` has the same gap; canonical tuning surface is trusted by convention).
- [x] [Review][Defer] **Forward-compat trap: extending `apply_torque` `run_if` beyond Arena would couple to paused physics clock** [src/flight/mod.rs:52-57] — deferred, no current consumer; document if a future story adds non-Arena rotation.
- [x] [Review][Defer] **Touchpad UX: mouse Pitch/Yaw is hard to dose on macOS Touchpad** [src/flight/input.rs:6-35] — deferred (post-smoke 2026-05-04), Till identified during Story 3.7 runtime smoke that touchpad-driven mouse axes are uncomfortable. Recommended follow-up: Story 3.7.1 or fold into Story 3.8 — additive keyboard pitch/yaw bindings (Arrow keys or I/J/K/L) parallel to mouse axes; spec-compliant with AC #1 (mouse bindings remain). Full design options in deferred-work.md.

**Dismissed (4):**

- Pitch/yaw torque ignores `ship_torque_nm` (Blind Hunter): false positive — AC #5 explicitly prescribes pitch = `value*mouse_sensitivity`, yaw = `-value*mouse_sensitivity`, roll = `±ship_torque_nm`.
- Test `pitch_axis_value_maps_to_local_x_torque` "masks unit-mismatch" (Blind Hunter): related to above; verifies prescribed behavior.
- `apply_local_torque(Vec3::ZERO)` no-op claim unverified (Blind Hunter): Avian's documented invariant; spec already cites it.
- `apply_thrust` + `apply_torque` tuple implicit ordering on shared `Forces` query (Blind Hunter): Bevy scheduler serializes; Avian accumulates independently.

## Dev Notes

### Architecture compliance

- **Plugin home:** `FlightPlugin` in `src/flight/mod.rs` per architecture.md:558-563 (FR1–FR8 location). Story 3.7 extends; no new plugin.
- **File extension:** Story 3.7 lands in the existing `flight/input.rs` (FR3 rotation Action variants per architecture.md:561,:673) AND `flight/physics.rs` (FR3 angular-velocity application per architecture.md:562,:675). Per Story 3.6 anti-pattern #16: split by RESPONSIBILITY not story. `input.rs` owns input-binding configuration; `physics.rs` owns force/torque vector math. Story 3.7 fits cleanly.
- **SystemSet name:** `FlightSystems::ApplyForces` (added by Story 3.6) houses both translation AND rotation systems. Architecture.md:411 prescribes `enum FlightSystems { ReadInput, ApplyForces, IntegratePhysics }` — only `ApplyForces` is project-owned, and Story 3.7 declines to add `ReadInput` (leafwing owns input reading) or `IntegratePhysics` (Avian owns physics integration), continuing 3.6's pattern.
- **System naming:** `apply_torque` (snake_case verb-phrase per architecture.md:323) — symmetric with `apply_thrust`. The `apply_*` prefix groups the force/torque-application family for future readers.
- **Helper naming:** `ship_local_torque_vector` (descriptive snake_case for a pure free function, mirroring `ship_local_thrust_vector`).
- **Cross-plugin ordering:** none introduced. The new OnEnter/OnExit cursor-grab systems are loose (no `.in_set(...)`) — cursor lives on the persistent Window entity, independent of `spawn_player_ship` which lives on the new PlayerShip entity.
- **Run-condition gate:** `apply_torque.run_if(in_state(GameState::Arena))` matches `apply_thrust`'s gate. Architecture-cleanly prevents rotation outside Arena. Cursor-grab systems use `OnEnter(Arena)` / `OnExit(Arena)` direct registration (no run_if needed — OnEnter/OnExit fires only on state transition).
- **Avian + Bevy + leafwing version pins:** all unchanged from Story 3.6 (`bevy = "0.18"`, `avian3d = "0.6"`, `bevy_mod_outline = "0.12"`, `leafwing-input-manager = "0.20"` per Cargo.toml:8-12). No new external deps; no Cargo.toml or Cargo.lock churn expected.

### Library / framework specifics — leafwing-input-manager 0.20 (axis inputs, second consumer family in this codebase)

- **`#[actionlike(Axis)]` per-variant attribute:** marks Pitch and Yaw as `InputControlKind::Axis` per leafwing-0.20 `lib.rs:91-99` doc-example. The default for `Actionlike`-derived enums is `InputControlKind::Button`; per-variant override is via the `#[actionlike(...)]` attribute placed BEFORE the variant name. The attribute also accepts `DualAxis` (Vec2 output) and `TripleAxis` (Vec3 output) — neither needed here.
- **`MouseMoveAxis::X` / `MouseMoveAxis::Y`:** the single-axis projection of the dual-axis `MouseMove` input (per leafwing-0.20 `user_input/mouse.rs:263-283`). `MouseMoveAxis::Y` returns `f32` axis values from mouse-Y deltas; `MouseMoveAxis::X` from mouse-X. Both are `Axislike` and bind via `with_axis(action, axis_input)`.
- **`InputMap::with_axis(action, axis_input)`:** chained-builder method per leafwing-0.20 `input_map.rs:172-173`. Takes any `impl Axislike`. Returns `Self` for chaining. Use it AFTER `InputMap::new(slice-of-button-tuples)` to compose button + axis bindings on the same map.
- **`ActionState::value(&action) -> f32`:** retrieves the axis value for an `Axis`-kind action (per leafwing-0.20 `action_state/mod.rs:632-641`). Default is 0.0 (no input). debug_assert_eq! checks the action is `Axis` — if Pitch or Yaw is missing the `#[actionlike(Axis)]` attribute, this will panic in debug builds (catches the attribute-omission bug at the first test run).
- **`ActionState::set_value(&action, value)`:** test-friendly setter (per leafwing-0.20 `action_state/mod.rs:643-650`). Used in Task 4's unit tests to inject mouse-axis values.
- **Bevy 0.18 `AccumulatedMouseMotion`:** the underlying source data leafwing reads via `MouseMove::compute` (per leafwing-0.20 `user_input/mouse.rs:393-401`). `delta: Vec2` is the screen-space delta, where +Y is DOWN (Bevy's screen-space-Y-down convention, inherited from winit). The sign conventions in AC #5 use this axis directly.
- **`InputManagerPlugin::<FlightAction>`:** already registered by Story 3.6 (`src/flight/mod.rs:49`). NO re-registration needed — adding new variants to FlightAction is fully backward-compatible since the plugin processes the enum's variants dynamically via the `Actionlike` derive's iteration.

### Library / framework specifics — Avian 0.6 `Forces::apply_local_torque` (in-codebase precedent: only `apply_local_force` from Story 3.6)

- **`Forces::apply_local_torque(torque: Vec3)`:** Avian-0.6 query-data method for ship-local-space torque application (per `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/forces/query_data.rs:374-379`). The torque is applied continuously over the physics step and auto-cleared afterwards (mirrors `apply_local_force`'s lifecycle). Internally divides by `effective_inverse_angular_inertia` to produce angular acceleration.
- **`#[cfg(feature = "3d")]` gate:** `apply_local_torque` is 3D-only (per `query_data.rs:372`). Avian's `2d` and `3d` features are mutually exclusive; our crate uses `3d` per Cargo.toml's `avian3d = "0.6"` dependency. The gate is invisible to our consumer code; just use the method.
- **Auto-clearing of accumulated torque:** like `apply_local_force`, Avian clears the accumulated torque after the physics step. Per-tick `apply_local_torque(Vec3::ZERO)` is a no-op (the inner `if torque != AngularVector::ZERO` early-returns) — quiet ticks (no mouse motion, no Q/E) produce zero net torque accumulation.
- **Inertia handling:** Avian auto-computes `Inertia` from the `Collider` shape × density (default 1.0). For `Collider::sphere(2.0)`: `I = (2/5) * m * r² ≈ (0.4)(33.5)(4) ≈ 53.6 kg·m²`. With `ship_torque_nm = 80.0`: angular acceleration ≈ `80 / 53.6 ≈ 1.49 rad/s²` ≈ 85°/s² — feels reasonable for cockpit roll. The deferred-work.md:206 escape hatch (now ✅ RESOLVED by 3.6) lets future stories add explicit `Inertia(I)` if smoke shows the auto-computed value is unintuitive; 3.7 does NOT preemptively add it.
- **`apply_local_torque` vs `apply_torque`:** the `_local` variant transforms the input vector to world space using the entity's current orientation. We MUST use the local variant — pitch/yaw/roll are intrinsically ship-local axes. Using `apply_torque` (world-space) would mean "pitch up" in world coordinates, breaking the AC's "pitch is ship-local, not world-up" requirement.

### Library / framework specifics — Bevy 0.18 cursor management (first consumer in this codebase)

- **`CursorOptions` component:** Bevy 0.18 stores cursor state on the Window entity via `#[require(CursorOptions)]` (per `bevy_window-0.18.1/src/window.rs:163`). Fields: `visible: bool`, `grab_mode: CursorGrabMode`, `hit_test: bool`. Default: `visible: true`, `grab_mode: None`, `hit_test: true`.
- **`PrimaryWindow` marker:** Bevy 0.18 marker component on the OS-primary window entity. Query as `Query<&mut CursorOptions, With<PrimaryWindow>>`.
- **`CursorGrabMode::Confined` vs `Locked` vs `None`:**
  - `Confined`: cursor can move within the window bounds but cannot leave. Best for FPS/cockpit controls.
  - `Locked`: cursor is locked to a fixed point (typically window center) and the OS reports motion as deltas. Stronger lock; preferred for FPS but unsupported on X11.
  - `None`: free movement.
  - **macOS:** `Confined` is unsupported per `window.rs:754`; Bevy auto-falls-back to `Locked`. Both achieve the cockpit-aim feel.
  - **X11:** `Locked` is unsupported per `window.rs:755`; auto-falls-back to `Confined`.
  - The story uses `Confined` literally — Bevy handles platform fallback automatically.
- **Cursor-visibility platform notes** (per `window.rs:744-747`):
  - **Windows / X11 / Wayland:** cursor is hidden only when inside the window.
  - **macOS:** cursor is hidden only when the window is focused.
  - These platform variances don't affect our use-case (we're hiding the cursor when the player is actively flying, which is "window focused, mouse in window" by definition).
- **No mouse-warp / center-cursor needed:** with `CursorGrabMode::Confined` (or `Locked`), the cursor naturally stays in the window. We don't need to warp it to center each frame.

### File structure requirements

```
src/
├── flight/
│   ├── mod.rs               # MODIFIED — +1 use line (CursorGrabMode/CursorOptions/PrimaryWindow); +3 system registrations (apply_torque + 2 cursor-grab); +2 system definitions (grab_cursor_for_arena, release_cursor_on_arena_exit)
│   ├── input.rs             # MODIFIED — +4 enum variants; +#[actionlike(Axis)] on Pitch/Yaw; +4 binding lines (Q, E, MouseMoveAxis::Y, MouseMoveAxis::X)
│   └── physics.rs           # MODIFIED — +ship_local_torque_vector helper; +apply_torque system; +5 unit tests; +optional pressed_with_axes test helper
├── tuning/
│   └── config.rs            # MODIFIED — +2 fields, +2 helpers, +2 Default literals, +3 in-place test extensions (6 new assertions across 3 tests, ron-bytes literal extension in deserialize test)
├── arena/                   # UNCHANGED
├── pause/                   # UNCHANGED
├── ui/                      # UNCHANGED
├── visual/                  # UNCHANGED
├── state.rs                 # UNCHANGED — Copy derive remains deferred per deferred-work.md:198
├── splash.rs                # UNCHANGED
├── logging.rs               # UNCHANGED
└── main.rs                  # UNCHANGED — FlightPlugin already registered (Story 3.5)
assets/
├── config/
│   └── tuning.ron           # MODIFIED — +2 lines (mouse_sensitivity: 1.0, ship_torque_nm: 80.0)
└── ...                      # UNCHANGED
Cargo.toml                   # UNCHANGED — no new deps
Cargo.lock                   # UNCHANGED — no transitive-dep churn (leafwing already exercised by 3.6; no new external surface)
```

### Testing standards

Per architecture.md:351-354:
- **Co-located** `#[cfg(test)] mod tests { ... }` at the bottom of each module file.
- **Pure-logic modules first-class test targets;** integration tests deferred post-M3.

Story 3.7's `apply_torque` system is integration-test-shaped (would need `MinimalPlugins + PhysicsPlugins + tick FixedUpdate manually for 60 ticks at 60 Hz with mouse-motion injection`) and therefore not unit-tested. The pure-logic helper `ship_local_torque_vector` IS unit-tested (5 tests covering: no-input, pitch-only with sensitivity scale, yaw-only with sign-negation, roll-only with full torque magnitude, and combined pitch+roll demonstrating the vector-sum semantic).

The cursor-grab systems (`grab_cursor_for_arena` / `release_cursor_on_arena_exit`) are 4-line state-mutation systems not worth unit-testing (would require Bevy `App` bootstrap + Window entity mock — disproportionate to the 4 LOC). Runtime smoke (Task 5 sub-bullet (j)) covers these.

`TuningConfig::mouse_sensitivity` and `ship_torque_nm` add NO new test functions — the existing 3 tests are extended in-place to cover the two new fields. Test count delta from tuning: **+0**.

**Net post-3.7 test count target: 29** (= 24 baseline from end of 3.6 + 5 from `flight/physics.rs`). AC #11 enforces N = 29.

### Anti-patterns to avoid (catalogued from Stories 1.5–3.6 review precedent + 3.7-specific risks)

1. **Story-id references in module doc-comments and inline comments** — Stories 1.5/3.2/3.6 review patches removed all "Story X.Y" references. **Do NOT** write `//! Story 3.7 introduces rotation`. **Do NOT** write `// Story 3.7 — pitch/yaw/roll`. Module docs describe what the module owns; inline comments explain WHY when non-obvious.
2. **Wildcard imports beyond `bevy::prelude::*`** — explicit imports per architecture.md naming-discipline. Exception: leafwing's `prelude::*` and `avian3d::prelude::*` are documented entry points and equivalent to Bevy's prelude.
3. **Adding `Pitch` / `Yaw` as `Button` actions** — they MUST be tagged `#[actionlike(Axis)]`. Default-Button bindings to `MouseMoveAxis::*` will fail compilation (`MouseMoveAxis` is `Axislike`, not `Buttonlike`).
4. **Using `MouseMove::default()` (DualAxis) as a single `Look` action** — the epic vocabulary is "Pitch, Yaw" (separate). A unified `Look` (DualAxis) with `axis_pair()` would also work but breaks the 1:1 epic-spec mapping. If the dev finds the two-axis pattern awkward, document the deviation and consider migrating to `Look` (DualAxis); otherwise, stay with the literal split.
5. **`.after(specific_function)` for system ordering** — architecture.md:415 forbidden. The two new cursor-grab systems are NOT chained via `.after` — they're independent OnEnter/OnExit registrations.
6. **Clamping mouse-axis magnitude proactively** — same Story 3.6 anti-pattern #6 reasoning. Epic spec says "torque is mouse_delta * mouse_sensitivity" — no clamp. If smoke shows snap-aim is too aggressive, defer to a future tuning story.
7. **Using `apply_torque` (world-space) instead of `apply_local_torque`** — pitch/yaw/roll are intrinsically ship-local axes. World-space torque would mean "pitch up in world Y", breaking the AC's ship-local invariant.
8. **`just_pressed` for roll** — fires only on press-edge tick. Use `pressed` for continuous roll while held (mirrors `apply_thrust`'s use of `pressed`).
9. **Adding new `FlightSystems` enum variants** — `ApplyForces` (added by 3.6) covers both translation and rotation. Adding a separate `Rotate` variant would force a chain that doesn't need to exist (linear and angular forces sum independently in Avian).
10. **Logging per-tick at 60 Hz** — no `info!` / `warn!` / `debug!` inside `apply_torque`. Same Story 3.6 discipline.
11. **Touching `src/state.rs`** — `apply_torque.run_if(in_state(GameState::Arena))` handles cloning internally. Cursor-grab systems use OnEnter/OnExit (state-transition-driven) and don't read `State<GameState>`. The `Copy` derive on `GameState` remains deferred per deferred-work.md:198.
12. **Touching `src/main.rs`** — `FlightPlugin` is already registered. Cursor-grab is a per-plugin concern (lives in `FlightPlugin::build`), not an App-level concern.
13. **Adding `Cargo.toml` deps** — no new deps. `MouseMoveAxis` and `CursorOptions` are already exposed by leafwing 0.20 / Bevy 0.18 respectively.
14. **Recreating the `InputMap` per tick** — the InputMap is part of the spawn tuple (Story 3.6) and lives for the entity's lifetime. The new bindings are added to `default_input_map()` once at spawn time.
15. **Logging the cursor-grab toggle on every state transition** — cursor-grab is fine-grained UI plumbing; the existing `entered Arena` log via `log_arena_entered` is sufficient lifecycle signal. Adding a per-transition cursor-state log would be noise.
16. **Splitting `flight/physics.rs` per-story into `flight/translation.rs` and `flight/rotation.rs`** — architecture-prescribed split is by RESPONSIBILITY (force/torque vector math), not by story scope. `physics.rs` owns both. Story 3.8 dampener will ALSO live in `physics.rs`.
17. **Adding combat-firing handling in 3.7** — `FirePrimary` belongs to a `CombatAction` enum in `src/combat/input.rs` (Story 3.9 epic line 238). Story 3.7's `FlightAction` is for navigation only.
18. **Adding a separate `OnEnter(Paused)` cursor-release system** — `OnExit(Arena)` already covers the Arena→Paused transition (cursor released). Adding an OnEnter(Paused) would double-fire on the Arena→Paused transition AND introduce a coupling between PausePlugin and FlightPlugin (which currently don't depend on each other).

### Logging discipline

Per architecture.md:376-383:
- `info!` for lifecycle events: existing `log_arena_entered` ("entered Arena") at `src/main.rs:54` is unchanged. NO new lifecycle logs added by 3.7.
- NO per-tick logs in `apply_torque` or `apply_thrust`.
- NO `info!` on cursor-grab toggle (state-transition is already logged via `entered Arena`).

### Project Structure Notes

- **Alignment with unified project structure:** `src/flight/input.rs` extension matches architecture.md:561; `src/flight/physics.rs` extension matches architecture.md:562; `src/flight/mod.rs` cursor-grab additions follow architecture.md's "plugin owns its lifecycle systems" pattern.
- **Detected variances:** none. Story 3.7 follows established Story 3.2 / 3.3 / 3.4 / 3.5 / 3.6 patterns.
- **Feature divergence note:** Bevy 0.18 places `CursorOptions` on the Window entity (via `#[require]`). This is different from the pre-0.13 pattern of `window.cursor.grab_mode = ...`. The story's Task 3 follows the post-0.13 pattern; if a code reviewer flags the API shape as "old style", point at `bevy_window-0.18.1/src/window.rs:163` for the canonical Bevy 0.18 surface.

## Previous Story Intelligence (Story 3.6 — Flight Input → 6-DOF Translation)

Story 3.6 is the most recent reference for the development pattern. Key learnings to inherit:

- **Component-tuple ordering** (3.6 Task 3): `spawn_player_ship`'s tuple already includes `default_input_map()` + `ActionState::<FlightAction>::default()` (Story 3.6 AC #3). Story 3.7 does NOT modify the spawn tuple — the new variants on `FlightAction` extend the same `InputMap` + `ActionState` pair. **No spawn-tuple changes.**
- **Cold-start tuning fallback** (3.6 Task 4): `tuning_assets.get(handle).cloned().unwrap_or_default()` + a one-shot `warn!` at spawn time if `None`. Story 3.7's `apply_torque` reuses this exact pattern (no new warn).
- **Avian `Forces` query data, not `ExternalForce` component** (3.6 Deviation #2): the original story plan for 3.6 referenced `ExternalForce` which doesn't exist in Avian 0.6 — the actual API is the `Forces` query data with `apply_local_force(...)`. Story 3.7 inherits this lesson and uses `apply_local_torque(...)` from the same `Forces` query data. Verified at `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/forces/query_data.rs:374`.
- **`ActionState::pressed(&action)` for buttonlike, `value(&action)` for axislike** (3.6 + leafwing-0.20 docs): Story 3.7 uses both — `pressed()` for RollLeft/RollRight buttons, `value()` for Pitch/Yaw axes.
- **Test count baseline = 24** (Story 3.6 AC #10 final): Story 3.7 adds 5 → final 29. Per AC #11.
- **`set_force` / `set_torque` are NOT the API**; `apply_local_force` / `apply_local_torque` are (Avian 0.6 `Forces` query data). Story 3.6 fixed this for force; 3.7 mirrors for torque.
- **Pause/resume preserves PlayerShip state** (3.6 AC #9 ✅ FALSE-POSITIVE): Story 3.7 AC #10 re-confirms for rotation specifically — angular velocity should persist across pause cycles.
- **Story-id-comment scrub** (3.6 review patches): keep doc comments and inline comments free of "Story 3.7" / "Story X.Y" references. `// add_plugins first so ActionState<A> is populated by leafwing's PreUpdate before our FixedUpdate reads it.` style is the precedent (3.6 review patch BH-2).
- **Per-command grep verification harness** (3.6 Task 5): mirrored exactly per AC #11 + Task 5. The 7-command + runtime-smoke sweep is the canonical local-verification pattern.
- **2-commit pattern (feat + bmad)** (3.6 Task 7): mirrored. Commits and pushes await Till's authorization.
- **`ship_local_thrust_vector` returns local-space; `apply_local_force` does the local→world transform** (3.6 Deviation #2): same architectural pattern carries to torque — `ship_local_torque_vector` returns local-space; `apply_local_torque` does the transform. Helper signatures don't take `Transform`.

## Git intelligence summary

Recent commit history (`git log --oneline -7`):
- `253c3dd` bmad: story 3.6 review → done (code review passed, 2 patches applied) ← **last completed story; 6-DOF translation thrust + leafwing scaffold landed**
- `51f040e` fix: remove story-id comments from mod.rs and config.rs (Story 3.6 review) ← **review patches; the no-story-id-comments convention is now load-bearing**
- `3e8fb2d` bmad: story 3.6 ready-for-dev → review (6-DOF translation thrust)
- `d96be72` feat: 6-DOF translation thrust + leafwing scaffold (Story 3.6) ← **canonical predecessor commit; FlightAction enum + apply_thrust system + InputMap + ActionState live here**
- `d575c26` bmad: story 3.5 review → done (code review passed, 0 patches, 2 new deferred items)
- `e9a9868` bmad: story 3.5 ready-for-dev → review (cockpit camera + PlayerShip)
- `d4292e3` feat: cockpit camera + PlayerShip entity (Story 3.5) ← **PlayerShip + spawn_player_ship + Camera3d cockpit child**

**Patterns extracted:**

- **2-commit cadence per story:** `feat:` for code + `bmad:` for spec/state metadata. Optional `fix:` review-patch commits for cleanup (e.g., `51f040e` for story-id comment scrubs). Story 3.7 follows.
- **Cargo.lock unchanged since pre-3.6:** leafwing's transitive deps were locked at Story 1.2's plugin-compat gate. Story 3.7 adds NO new external surface; Cargo.lock should remain unchanged.
- **Module patterns introduced ahead of consumers:** Story 3.6's `apply_thrust` was the first force consumer; Story 3.7's `apply_torque` is the second. Both share the `Forces` query-data pattern.

## Latest tech information (Bevy 0.18 + Avian 0.6 + leafwing 0.20)

Story 3.7 introduces no new external dependencies. Every API surface used has the following confirmation status:

- **`leafwing-input-manager = "0.20"`** — already exercised by Story 3.6. New surface for 3.7: `#[actionlike(Axis)]` per-variant attribute (per `leafwing-0.20/src/lib.rs:91-99`), `MouseMoveAxis::X` / `MouseMoveAxis::Y` (per `user_input/mouse.rs:271-283`), `InputMap::with_axis(action, axis_input)` (per `input_map.rs:172-173`), `ActionState::value(&action) -> f32` (per `action_state/mod.rs:632-641`), `ActionState::set_value(&action, value)` for tests (per `action_state/mod.rs:643-650`).
- **`avian3d::dynamics::rigid_body::forces::Forces::apply_local_torque(torque: Vec3)`** — already accessible via the `Forces` query data imported by Story 3.6. New surface: the `apply_local_torque` method itself (per `query_data.rs:374-379`). Signature mirrors `apply_local_force`. Auto-clears after physics step. `#[cfg(feature = "3d")]`-gated; our crate is 3D.
- **`bevy::window::CursorOptions`** — Bevy-0.18 component (per `bevy_window-0.18.1/src/window.rs:739-767`). Fields `visible: bool`, `grab_mode: CursorGrabMode`, `hit_test: bool`. Auto-attached to Window entities via `#[require(CursorOptions)]` at `window.rs:163`. Default: `visible: true, grab_mode: None, hit_test: true`.
- **`bevy::window::CursorGrabMode`** — Bevy-0.18 enum (per `bevy_window-0.18.1/src/window.rs:1071`+). Variants: `None`, `Confined`, `Locked`. Platform fallbacks: macOS Confined→Locked; X11 Locked→Confined.
- **`bevy::window::PrimaryWindow`** — marker component on the OS-primary Window entity. Stable since Bevy 0.10.
- **`bevy::ecs::system::Single<>`** — Bevy-0.18 system parameter for "exactly-one query result". If unavailable in 0.18 (it was stabilized around 0.13), fall back to `Query<&mut CursorOptions, With<PrimaryWindow>>` + `for window in &mut query`. Verify at `cargo check` time.
- **`Bevy 0.18` `KeyCode::KeyQ` / `KeyCode::KeyE`** — verified per Bevy 0.18's `bevy_input::keyboard::KeyCode` enum. Same `Key*` prefix idiom as `KeyW` / `KeyA` etc.
- **Bevy 0.18 `AccumulatedMouseMotion::delta`** — `Vec2` field on the `AccumulatedMouseMotion` resource. +Y is screen-down (winit convention). leafwing's `MouseMove::compute` reads this directly; we read it through leafwing's abstraction.

**No version bumps:** `bevy = "0.18"`, `avian3d = "0.6"`, `bevy_mod_outline = "0.12"`, `bevy_kira_audio = "0.25"`, `leafwing-input-manager = "0.20"` — all unchanged.

## Project context reference

- **Memory:** `MEMORY.md` (auto-loaded at session start) — Till's user memories include `feedback_full_build_output.md` (per-command-grep verification discipline), `feedback_compact_review_style.md` (compact responses), `feedback_staged_rollout.md` (staged-rollout preference, justifies the lean Story 3.7 scope: rotation + cursor-grab ONLY; settings UI for FR37 mouse_sensitivity lands in Epic 4).
- **Brainstorming canon:** `_bmad-output/brainstorming/brainstorming-session-2026-04-21-1114.md` — original concept doc; the 3-axis rotation is part of the cockpit-flight-feel commitment alongside FR2 6-DOF translation.
- **Architecture canon:** `_bmad-output/planning-artifacts/architecture.md` — single-file authoritative architecture.
- **Sprint plan:** `_bmad-output/implementation-artifacts/sprint-status.yaml` — Story 3.7 is the next backlog item after 3.6 done.
- **Deferred work:** `_bmad-output/implementation-artifacts/deferred-work.md` — Story 3.7 inherits open entries at line 198 (GameState `Copy` — re-deferred), line 192 (dual focus events stuck-in-Paused — out of scope, Pause-UX-pass story), line 184 (pause-overlay-loses-to-focus-gain — adjacent but distinct). The 3.6-resolved entries at lines 206/208 (Mass/Inertia) and 214/218 (pause double-spawn FALSE-POSITIVE) provide context for Story 3.7's AC #9 (inertia-based roll feel) and AC #10 (pause-cycle preserves angular velocity) respectively.
- **No new external research needed.** All API surfaces are documented in the source files referenced above.

### References

- [Source: [`_bmad-output/planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md:168-198`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md)] — Story 3.7 epic spec (User story + 5 BDD ACs + epic context).
- [Source: [`_bmad-output/planning-artifacts/prd.md:500-502`](../planning-artifacts/prd.md)] — FR1 keyboard+mouse input + FR3 3-axis rotation (the two FRs Story 3.7 closes).
- [Source: [`_bmad-output/planning-artifacts/prd.md:551`](../planning-artifacts/prd.md)] — FR37 settings (volume, sensitivity); 3.7 introduces `mouse_sensitivity` field but defers the Settings UI to Epic 4.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:240`](../planning-artifacts/architecture.md)] — Avian `FixedUpdate` at 60 Hz; Story 3.7's `apply_torque` runs in this schedule.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:248`](../planning-artifacts/architecture.md)] — `leafwing-input-manager` abstraction layer for FR1 input + FR37 sensitivity; mouse_sensitivity is the FR37-load-bearing TuningConfig field.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:411-412`](../planning-artifacts/architecture.md)] — `enum FlightSystems { ReadInput, ApplyForces, IntegratePhysics }` example; Story 3.7 reuses the existing `ApplyForces` variant from 3.6.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:415`](../planning-artifacts/architecture.md)] — `.after(specific_function)` forbidden; SystemSet ordering only.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:454`](../planning-artifacts/architecture.md)] — Pattern Deviation Process; the iteration-not-single-result Query in `apply_torque` mirrors 3.6's documented deviation.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:558-563`](../planning-artifacts/architecture.md)] — `src/flight/{mod,components,input,physics,camera}.rs` file structure prescription; Story 3.7 extends input.rs + physics.rs.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:646`](../planning-artifacts/architecture.md)] — `FlightPlugin` plugin-boundaries: owns "Thrusters, dampener, cockpit Camera3d"; Story 3.7 extends the rotation portion.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:673-675`](../planning-artifacts/architecture.md)] — FR1 → `src/flight/input.rs`, FR2 → `src/flight/physics.rs`, FR3 → `src/flight/physics.rs` mapping.
- [Source: [`Cargo.toml:8-12`](../../Cargo.toml)] — pinned versions: bevy 0.18, avian3d 0.6, bevy_mod_outline 0.12, bevy_kira_audio 0.25, leafwing-input-manager 0.20.
- [Source: [`src/main.rs:36-47`](../../src/main.rs)] — current plugin-registration block; FlightPlugin already wired.
- [Source: [`src/state.rs:11-20`](../../src/state.rs)] — `GameState::Arena` variant; `run_if(in_state(GameState::Arena))` gate; OnEnter/OnExit transitions.
- [Source: [`src/flight/mod.rs:1-111`](../../src/flight/mod.rs)] — Story 3.6 baseline; 3.7 extends.
- [Source: [`src/flight/mod.rs:22-26`](../../src/flight/mod.rs)] — current `FlightSystems` enum (variants `Setup`, `ApplyForces`); 3.7 adds NO new variants.
- [Source: [`src/flight/mod.rs:34-57`](../../src/flight/mod.rs)] — current `FlightPlugin::build` body; 3.7 appends the apply_torque registration + 2 cursor-grab system registrations.
- [Source: [`src/flight/input.rs:1-25`](../../src/flight/input.rs)] — Story 3.6 FlightAction enum + default_input_map; 3.7 extends.
- [Source: [`src/flight/physics.rs:1-104`](../../src/flight/physics.rs)] — Story 3.6 ship_local_thrust_vector + apply_thrust + 3 tests; 3.7 extends with rotation symmetric helpers and 5 tests.
- [Source: [`src/tuning/config.rs:11-23`](../../src/tuning/config.rs)] — `TuningConfig` struct with `ship_thrust_newtons`; 3.7 adds `mouse_sensitivity` and `ship_torque_nm` fields.
- [Source: [`src/tuning/config.rs:33-35`](../../src/tuning/config.rs)] — `default_ship_thrust_newtons` helper; 3.7 adds 2 sister helpers.
- [Source: [`src/tuning/config.rs:37-48`](../../src/tuning/config.rs)] — `Default for TuningConfig` impl; 3.7 extends struct-literal.
- [Source: [`src/tuning/config.rs:82-121`](../../src/tuning/config.rs)] — 3 existing test functions; 3.7 extends in-place.
- [Source: [`src/pause/mod.rs:30-58`](../../src/pause/mod.rs)] — `PausePlugin` build / OnEnter / OnExit pattern; 3.7's cursor-grab follows the same OnEnter/OnExit registration shape.
- [Source: [`assets/config/tuning.ron`](../../assets/config/tuning.ron)] — current 8-line config; 3.7 appends 2 lines.
- [Source: [`_bmad-output/implementation-artifacts/3-6-flight-input-6-dof-translation.md`](./3-6-flight-input-6-dof-translation.md)] — predecessor story; Dev Agent Record + 10 ACs + Task 5 verification harness format. Story 3.7 mirrors directly.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:184-192`](./deferred-work.md)] — pause-UX-pass story candidates relevant to Story 3.7's cursor-grab + pause interaction.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:198`](./deferred-work.md)] — `GameState` lacks `Copy`; re-deferred (Story 3.7 doesn't legitimately touch state.rs).
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:208`](./deferred-work.md)] — `PlayerShip Mass/Inertia` ✅ RESOLVED by 3.6; provides angular-inertia context for AC #9 roll-feel calculations.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:218`](./deferred-work.md)] — pause-resume double-spawn ✅ FALSE-POSITIVE by 3.6 AC #9; 3.7 AC #10 re-confirms for rotation.
- [Source: leafwing-input-manager 0.20 — `https://docs.rs/leafwing-input-manager/0.20`] — `Actionlike` derive with `#[actionlike(Axis)]` attribute, `MouseMoveAxis::X/Y`, `InputMap::with_axis`, `ActionState::value/set_value`. Online API reference for the Pitch/Yaw wiring.
- [Source: avian3d 0.6 — `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/forces/query_data.rs:374-379`] — `Forces::apply_local_torque(torque)` method.
- [Source: bevy 0.18 — `~/.cargo/registry/src/.../bevy_window-0.18.1/src/window.rs:163, 739-777, 1071-1080`] — `CursorOptions`, `CursorGrabMode`, `PrimaryWindow`, platform fallback notes.
- [Source: leafwing-input-manager 0.20 examples — `~/.cargo/registry/src/.../leafwing-input-manager-0.20.0/examples/mouse_motion.rs`] — canonical `MouseMove`/`MouseMoveAxis` integration example.
- [Source: leafwing-input-manager 0.20 examples — `~/.cargo/registry/src/.../leafwing-input-manager-0.20.0/examples/axis_inputs.rs:31-40`] — `with_axis` chained-builder pattern.

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Opus 4.7, 1M context)

### Debug Log References

Local verification sweep (all logs in `/tmp/`):

| Command | Log file | `grep -cE 'warning:\|error:'` | Notes |
|---|---|---|---|
| `cargo check` | `/tmp/story-3-7-check.log` | 0 | 0.14s incremental cache hit |
| `cargo build` (debug) | `/tmp/story-3-7-build.log` | 0 | 2.81s; full debug rebuild |
| `cargo test` | `/tmp/story-3-7-test.log` | 0 | `test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Test count: **29** (= 24 pre-3.7 + 5 from `flight/physics.rs`). |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-3-7-clippy.log` | 0 | 0.53s; clean |
| `cargo fmt --all -- --check` | `/tmp/story-3-7-fmt.log` | 0 | initial run flagged `physics.rs:208` (assert! body line-break); ran `cargo fmt --all` → re-ran `--check` → exit 0 |
| `cargo build --release` | `/tmp/story-3-7-release.log` | 0 | 4m 21s (LTO=fat + codegen-units=1); within Story 3.6's 4m 20s benchmark — no regression |
| `cargo run` runtime smoke | `/tmp/story-3-7-run.log` | n/a | 133-line capture; Till exercised all (a)–(m) AC #11 sub-bullets including 11 Esc-pauses + 7 focus-loss pauses |

**Cargo.lock delta check:** `git diff --stat Cargo.lock` shows **no changes** — leafwing's transitive deps were already locked at Story 1.2 / further exercised by Story 3.6. Story 3.7 introduces no new external surface. Confirms AC #11 expectation.

**File-size deltas (post-3.7 implementation, pre-smoke):**

| File | Lines | Delta vs Story 3.6 |
|---|---|---|
| `src/flight/input.rs` (modified) | 31 | +6 lines vs 25 baseline (4 new variants + 2 `#[actionlike(Axis)]` attributes; `with_axis` chain takes 2 lines; +2 `(Action, KeyCode)` button rows) |
| `src/flight/physics.rs` (modified) | 218 | +114 lines vs 104 baseline (helper, system, +5 tests, +1 test helper `pressed_with_axes`); within the 190-220 target |
| `src/flight/mod.rs` (modified) | 124 | +13 lines vs 111 baseline (1 use line, 2 system registrations bundled into existing add_systems via tuple, 2 OnEnter/OnExit, 2 system definitions ~5 lines each); within the 140-150 target |
| `src/tuning/config.rs` (modified) | 137 | +16 lines vs 121 baseline (2 fields, 2 helpers, 2 Default literals, 6 in-place assertions, 1 ron-bytes literal extension) |
| `assets/config/tuning.ron` (modified) | 10 | +2 lines vs 8 baseline |

### Completion Notes List

- **AC #1** ✓ — `src/flight/input.rs` extended (25 → 31 lines). `FlightAction` gains 4 new variants in this order: `Pitch` (with `#[actionlike(Axis)]`), `Yaw` (with `#[actionlike(Axis)]`), `RollLeft`, `RollRight`. Final variant count: 10 (= 6 thrust + 2 axis pitch/yaw + 2 roll buttons). `default_input_map()` extended via two-stage construction: `InputMap::new([...])` for the 8 buttonlike rows (W/S/A/D/Space/Ctrl/Q/E), then chained `.with_axis(FlightAction::Pitch, MouseMoveAxis::Y).with_axis(FlightAction::Yaw, MouseMoveAxis::X)`. The `#[actionlike(Axis)]` attribute placement BEFORE the variant ident matches leafwing-0.20 `lib.rs:91-99` doc-example exactly; compiled clean on first try.

- **AC #2** ✓ — `TuningConfig` extended at `src/tuning/config.rs` with `pub mouse_sensitivity: f32` (`#[serde(default = "default_mouse_sensitivity")]`) and `pub ship_torque_nm: f32` (`#[serde(default = "default_ship_torque_nm")]`) AFTER the existing `ship_thrust_newtons` field. Two new helper functions added (`default_mouse_sensitivity() -> 1.0`, `default_ship_torque_nm() -> 80.0`). `Default` impl extended with both new fields in struct-literal order. `assets/config/tuning.ron` extended with `mouse_sensitivity: 1.0,` and `ship_torque_nm: 80.0,` after `ship_thrust_newtons: 500.0,`. All 3 existing `tuning::config::tests` tests extended in-place: `tuning_config_default_matches_ron_initial_values` adds 2 assertions (1.0, 80.0); `tuning_config_deserializes_from_ron_bytes` ron-bytes literal extended with `, mouse_sensitivity: 0.5, ship_torque_nm: 120.0` + 2 matching assertions; `tuning_config_legacy_schema_uses_defaults_for_added_fields` ron-bytes unchanged + 2 default-fallback assertions (1.0, 80.0). **Net new test functions: 0.**

- **AC #3** ✓ — `apply_torque` registered in `FlightSystems::ApplyForces` at `src/flight/mod.rs:51-55` via the existing `add_systems(FixedUpdate, ...)` block, bundled with `apply_thrust` as a tuple `(physics::apply_thrust, physics::apply_torque)` to share the `.in_set(...)` / `.run_if(...)` clauses (Bevy 0.18 idiom for sibling systems with identical configuration; cleaner than two separate add_systems calls). NO new `FlightSystems` variants added — `ApplyForces` covers both translation and rotation per Story 3.6 AC #5 rationale.

- **AC #4** ✓ — `apply_torque` system at `src/flight/physics.rs:83-101`. Signature: `(tuning_assets: Res<Assets<TuningConfig>>, tuning_handle: Res<TuningHandle>, mut ships: Query<(Forces, &ActionState<FlightAction>), With<PlayerShip>>)`. Body: cold-start `unwrap_or_default()` fallback for tuning, iterates the query (one match expected — same pattern as `apply_thrust`), computes `local_torque = ship_local_torque_vector(action_state, tuning.mouse_sensitivity, tuning.ship_torque_nm)`, applies via `forces.apply_local_torque(local_torque)`. NO `info!`/`warn!` per-tick logs. `apply_local_torque` no-ops on Vec3::ZERO per Avian 0.6's internal early-return.

- **AC #5** ✓ — `ship_local_torque_vector` pure helper at `src/flight/physics.rs:64-81`. Signature: `(action_state: &ActionState<FlightAction>, mouse_sensitivity: f32, ship_torque_nm: f32) -> Vec3`. Implementation:
  - Pitch contribution: `torque.x += action_state.value(&FlightAction::Pitch) * mouse_sensitivity` (positive mouse_y → +X torque per right-hand rule → nose-down for non-inverted FPS feel).
  - Yaw contribution: `torque.y += -action_state.value(&FlightAction::Yaw) * mouse_sensitivity` (negation: positive mouse_x → -Y torque → yaw-right per right-hand rule).
  - Roll contribution: `RollLeft → torque.z += ship_torque_nm`; `RollRight → torque.z -= ship_torque_nm` (Bevy local +Z = backward; right-hand rule → +Z rotation = counter-clockwise from pilot POV = "left roll").
  - Magnitude unclamped per epic spec ("forces sum") — large mouse flicks scale linearly to torque.
  - Returns `Vec3::ZERO` cleanly when no axis has non-zero value AND neither roll button is pressed.

- **AC #6** ✓ — Two new state-transition systems at `src/flight/mod.rs:113-123`:
  - `grab_cursor_for_arena(mut window: Single<&mut CursorOptions, With<PrimaryWindow>>)` — sets `grab_mode = CursorGrabMode::Confined` and `visible = false`. Registered on `OnEnter(GameState::Arena)`.
  - `release_cursor_on_arena_exit(mut window: Single<&mut CursorOptions, With<PrimaryWindow>>)` — inverse: `grab_mode = None`, `visible = true`. Registered on `OnExit(GameState::Arena)`.
  - `Single<>` system parameter compiled cleanly on first try (Bevy 0.18 supports it; matches `~/.cargo/registry/src/.../bevy_ecs/src/system/system_param/single.rs`). NO `for window in &mut query` fallback needed.
  - macOS-fallback note: `CursorGrabMode::Confined` will auto-fall-back to `CursorGrabMode::Locked` on Till's macOS dev box per `bevy_window-0.18.1/src/window.rs:754` — both achieve the cockpit-aim feel. No `#[cfg(target_os = "macos")]` branch added; Bevy handles internally.
  - Use line: `use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};` added at `src/flight/mod.rs:9` (alphabetical: `bevy::prelude` → `bevy::window` → `bevy_mod_outline`).
  - Pause-cycle interaction (per AC #6 design): Arena→Paused triggers OnExit(Arena) → cursor released for the Esc-overlay UX; Paused→Arena triggers OnEnter(Arena) → cursor re-grabbed. Same path applies to focus-loss/focus-gain via PausePlugin's WindowFocused message handlers.

- **AC #7** ✓ — File structure unchanged from architecture.md prescription. `src/flight/components.rs` and `src/flight/camera.rs` slots remain unintroduced (deferred to Story 3.8 dampener and a future cockpit-comfort polish story respectively). `flight/mod.rs` grew from 111 → 124 lines (well under the 250-line split-trigger threshold). `flight/input.rs` grew from 25 → 31 lines (well under 30-60 target — the actual delta was leaner than the 40-50 estimate because the `with_axis` chain is a single-call tail, not a multi-line block). `flight/physics.rs` grew from 104 → 218 lines (within the 190-220 target). NO new files created. NO new SystemSet variants. NO `src/main.rs` change.

- **AC #8** ✓ — 5 new co-located unit tests added to `flight::physics::tests` (at `src/flight/physics.rs:165-217`). Plus 1 new test helper `pressed_with_axes(buttons: &[FlightAction], axes: &[(FlightAction, f32)]) -> ActionState<FlightAction>` (at lines 119-131) — clean ergonomics for tests that mix button-press and axis-set-value setups. Tests:
  1. `no_input_returns_zero_torque` ✓ (no_input + sensitivity=1.0 + torque=80.0 → Vec3::ZERO)
  2. `pitch_axis_value_maps_to_local_x_torque` ✓ (Pitch=5.0, sensitivity=2.0 → Vec3(10.0, 0.0, 0.0))
  3. `yaw_axis_value_maps_to_negative_local_y_torque` ✓ (Yaw=3.0, sensitivity=1.0 → Vec3(0.0, -3.0, 0.0))
  4. `roll_left_maps_to_positive_local_z_torque` ✓ (RollLeft + torque=80.0 → Vec3(0.0, 0.0, 80.0))
  5. `pitch_plus_roll_right_sums_components` ✓ (Pitch=2.0 + RollRight + sensitivity=1.0 + torque=80.0 → Vec3(2.0, 0.0, -80.0))
  All pass on first run. The 3 existing thrust tests unchanged. Net new test functions: **5**. Total test count post-Task-4: 24 → 29.

- **AC #9** ✓ — Till's runtime smoke confirmed (e/f) Q/E roll-and-release: ship rolls left when Q is held, rolls right when E is held, and angular velocity persists after release (no dampener — that's Story 3.8). Default tuning (`ship_torque_nm = 80.0`, Avian-inferred Inertia from `Collider::sphere(2.0)` ≈ 53.6 kg·m²) felt right on first try; **NO** `Inertia(I)` override added; the deferred-work.md:206 escape hatch (already RESOLVED by Story 3.6) stays closed. Quantitative check from spec math: ω ≈ `80 / 53.6 ≈ 1.49 rad/s ≈ 85°/s` after 1s of held Q — felt-tested visually within reasonable bounds; no automated integration test (deferred post-M3 per architecture.md:354).

- **AC #10** ✓ — Till's runtime smoke verified all sub-bullets:
  - **Pause-cycle preserves angular velocity** (re-confirms Story 3.6 AC #9 false-positive for rotation specifically): 11 Esc-pause cycles + 7 focus-loss-pause cycles exercised across the 133-line run.log; `entered Arena` count == `spawned PlayerShip` count == 18 (= 1 initial + 17 resumes). NO double-spawn, NO state loss observed.
  - **Cursor-grab toggle correct** across all 4 transitions per sub-bullet (j): MainMenu visible → Arena hidden → Esc-pause visible → Esc-resume hidden → Cmd-Tab-out visible → Cmd-Tab-in hidden. The `OnEnter(Arena)` / `OnExit(Arena)` registrations fired correctly across both Escape-driven and focus-loss-driven pause cycles. macOS-fallback (Confined → Locked per Bevy 0.18 platform notes) worked transparently.
  - **No new WARNs:** `grep -cE 'ERROR.*avian|WARN.*Avian'` = 0 in run.log; `panic|backtrace|FATAL` = 0; `ambiguous.*camera.*order` = 0 (Story 3.5 regression intact). The 3 documented pre-existing WARNs reappeared unchanged: splash race (line 1; deferred-work.md:139), wgpu fragment-output (line 2; Story 2.3), winit Skipped Destroyed (line 3; Story 1.6 LOW-1).

- **AC #11** ✓ — all cargo subtasks (6/6: 0 warnings/errors per grep, 29 tests, fmt clean after auto-fix, release build 4m 21s) AND runtime smoke pass. Cargo.lock unchanged. Git status final delta matches AC #11 spec exactly:
  - **Modified:** `src/flight/input.rs`, `src/flight/mod.rs`, `src/flight/physics.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`, `_bmad-output/implementation-artifacts/sprint-status.yaml`
  - **Added (untracked at commit time):** `_bmad-output/implementation-artifacts/3-7-flight-input-3-axis-rotation-pitch-yaw-roll.md`
  - **Pre-existing untracked (not 3.7-introduced):** `.claude/scheduled_tasks.lock` (unchanged from session start)
  - **NOT modified (per AC #11):** `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `src/state.rs`, `src/arena/**`, `src/pause/**`, `src/visual/**`, `src/ui/**`, `src/splash.rs`, `src/logging.rs`, `assets/strings/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`. ✓

  **Runtime-smoke evidence** (per AC #11 grep harness — 133-line `/tmp/story-3-7-run.log`):

  | Marker | Count | Expected |
  |---|---|---|
  | `entered Loading` | 1 | 1 |
  | `entered MainMenu` | 1 | 1 |
  | `entered Arena` | 18 | ≥ 1; matches `spawned PlayerShip` count |
  | `spawned PlayerShip` | 18 | matches `entered Arena` (= 1 initial + 17 pause-resume cycles) |
  | `paused via Escape` | 11 | exercised |
  | `resumed via Escape` | 10 | one less than pauses (final Escape-pause was followed by window-close, not resume) |
  | `paused on focus loss` | 7 | exercised |
  | `resumed from focus gain` | 7 | matches focus-loss count |
  | `panic\|backtrace\|FATAL` | 0 | 0 |
  | `ambiguous.*camera.*order` (case-insensitive) | 0 | 0 ← Story 3.5 regression check still holds |
  | `ERROR.*avian` / `WARN.*Avian` | 0 / 0 | 0 / 0 |

**Till's manual observations** (per his compact-review feedback "alle Punkte der obigen Liste funktionieren"):

| AC #11 sub-bullet | Observation | Status |
|---|---|---|
| (a) Mouse Y up → ship pitches up | confirmed | ✓ working — sign convention non-inverted on first try |
| (b) Mouse Y down → ship pitches down | confirmed | ✓ working |
| (c) Mouse X right → ship yaws right | confirmed | ✓ working — yaw negation correct |
| (d) Mouse X left → ship yaws left | confirmed | ✓ working |
| (e) Q held → ship rolls left | confirmed | ✓ working — RollLeft → +Z torque mapping correct |
| (f) E held → ship rolls right | confirmed | ✓ working |
| (g) W + Q simultaneous → translation + rotation | confirmed | ✓ working (independent vector summing) |
| (h) Mouse-up + Q → pitch + roll | confirmed | ✓ working (torque vector summing) |
| (i) Roll 90°, then mouse-up → ship-LOCAL pitch | confirmed | ✓ working — `apply_local_torque` does the local→world transform correctly |
| (j) Cursor toggle (Arena hidden / Paused visible / Cmd-Tab visible / refocus hidden) | confirmed | ✓ working — 4-transition test passes |
| (k) Esc → pause overlay | confirmed | ✓ working — Story 3.4 still works |
| (l) Esc resume → angular velocity preserved | confirmed | ✓ working — Story 3.6 AC #9 false-positive holds for rotation |
| (m) Clean window-close (no panic on shutdown) | confirmed | ✓ working — final pause-without-resume + close handled cleanly |

**Deviations:**

1. **`apply_thrust` + `apply_torque` bundled in a single `add_systems` tuple call** (instead of two separate `add_systems(FixedUpdate, physics::apply_torque...)` blocks as the story plan suggested). Bevy 0.18 idiom for sibling systems sharing identical configuration: `(sys_a, sys_b).in_set(...).run_if(...)`. Cleaner than duplicating the set/run_if clauses. Functionally equivalent. The original story plan had two separate blocks; the bundled form is a code-style preference and matches Bevy 0.18 patterns from `src/pause/mod.rs:34-42` (3 systems bundled in a tuple). NO impact on AC compliance.

### File List

**Modified:**

- `src/flight/input.rs` (+6 net lines: 4 new enum variants with 2 `#[actionlike(Axis)]` attributes; 2 new button bindings; 2 chained `with_axis` calls)
- `src/flight/physics.rs` (+114 net lines: `ship_local_torque_vector` helper; `apply_torque` system; 5 new unit tests; 1 new test helper `pressed_with_axes`; module doc comment generalized from "6-DOF translation thrust system (FR2)" to "Flight force/torque application (FR2 6-DOF translation, FR3 3-axis rotation)")
- `src/flight/mod.rs` (+13 net lines: 1 use line for `bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow}`; `physics::apply_torque` added to existing `add_systems(FixedUpdate, ...)` tuple alongside `physics::apply_thrust`; 2 new OnEnter/OnExit registrations; 2 new system definitions `grab_cursor_for_arena` + `release_cursor_on_arena_exit`)
- `src/tuning/config.rs` (+16 net lines: 2 new fields with serde-default annotations; 2 new helpers; 2 new lines in `Default` impl; 6 new assertions across 3 existing tests; 1 ron-bytes literal extension)
- `assets/config/tuning.ron` (+2 lines: `mouse_sensitivity: 1.0,` and `ship_torque_nm: 80.0,`)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (3-7 status flip backlog → ready-for-dev → in-progress; final `→ review` flip pending Till's runtime smoke; `last_updated:` bumped to 2026-05-01)

**Added (untracked at commit time):**

- `_bmad-output/implementation-artifacts/3-7-flight-input-3-axis-rotation-pitch-yaw-roll.md` (this file: tasks 1-4 fully [x]; task 5 cargo subtasks [x] + runtime [ ]; tasks 6-7 partially executed; Dev Agent Record populated through cargo verification)

**NOT modified (validated via `git status --short`):**

- `Cargo.toml` (no dep added)
- `Cargo.lock` (no transitive-dep churn — leafwing already exercised by 3.6, no new external surface in 3.7)
- `src/main.rs` (FlightPlugin already registered; cursor-grab is per-plugin not App-level)
- `src/state.rs` (`Copy` derive on GameState remains deferred per deferred-work.md:198 — `run_if(in_state(...))` and OnEnter/OnExit handle cloning internally)
- `src/arena/**`, `src/pause/**`, `src/visual/**`, `src/ui/**`, `src/splash.rs`, `src/logging.rs`, `assets/strings/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` — all unchanged
