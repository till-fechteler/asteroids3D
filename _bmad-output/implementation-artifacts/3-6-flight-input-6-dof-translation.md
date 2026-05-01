# Story 3.6: Flight Input → 6-DOF Translation

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player in the Arena cockpit,
I want keyboard input to translate the ship forward/reverse, strafe left/right, and ascend/descend in **ship-local** space (so W always moves "where the nose points", regardless of pitch/yaw — which arrives in 3.7),
So that I can navigate the 3D Arena per FR2, closing the loop from "I'm a stationary pilot" (Story 3.5) to "I'm a flying pilot." This is also the first `leafwing-input-manager` consumer in the codebase — the FlightAction enum + InputManagerBundle scaffold lands here and stays load-bearing for Stories 3.7 (rotation), 3.8 (dampener), 3.9 (combat firing — separate `CombatAction` enum), and the eventual FR37 mouse-sensitivity setting + post-MVP rebinding flow.

## Acceptance Criteria

1. **Given** `leafwing-input-manager = "0.20"` is already pinned in `Cargo.toml:12` (Story 1.2 plugin-compatibility gate, never imported until now)
   **When** Story 3.6 introduces the first import
   **Then** `src/flight/input.rs` is authored with a `FlightAction` enum deriving leafwing 0.20's required traits (`Actionlike`, `PartialEq`, `Eq`, `Hash`, `Clone`, `Copy`, `Debug`, plus `bevy::prelude::Reflect` per leafwing 0.20's `Actionlike` blanket-impl requirement)
   **And** the enum has **exactly six** variants: `ThrustForward`, `ThrustReverse`, `StrafeLeft`, `StrafeRight`, `ThrustUp`, `ThrustDown` (no rotation variants — those are Story 3.7's scope; no firing variant — that's Story 3.9's scope and lands in a separate `CombatAction` enum per epic-3 line 238)
   **And** a `default_input_map() -> InputMap<FlightAction>` function returns the keyboard binding map: `KeyW → ThrustForward`, `KeyS → ThrustReverse`, `KeyA → StrafeLeft`, `KeyD → StrafeRight`, `Space → ThrustUp`, `ControlLeft → ThrustDown`
   **And** `FlightAction` and `default_input_map` are `pub` so `mod.rs` can reference them in `spawn_player_ship` and `apply_thrust`

2. **Given** `FlightPlugin` must register leafwing's `InputManagerPlugin::<FlightAction>` so `ActionState<FlightAction>` is updated each frame
   **When** `FlightPlugin::build` runs
   **Then** `app.add_plugins(InputManagerPlugin::<FlightAction>::default())` is added BEFORE the existing `app.configure_sets(...)` call in `src/flight/mod.rs` (registration order: plugin → set-config → systems — matches Bevy idiom)
   **And** the InputManagerPlugin registration is the only `add_plugins` call inside `FlightPlugin::build` (no other plugin dependencies are introduced by 3.6)
   **And** `use leafwing_input_manager::prelude::*;` is added to both `src/flight/input.rs` AND `src/flight/mod.rs` (the prelude is the leafwing-0.20 idiom; explicit imports per-symbol would balloon the import block — leafwing's prelude is the documented entry point)

3. **Given** the existing `spawn_player_ship` at `src/flight/mod.rs:44-93` builds the PlayerShip spawn tuple
   **When** Story 3.6 extends the tuple
   **Then** the spawn adds **two** new components to the existing 10-component tuple, in this order at the end of the tuple (after `AngularVelocity(Vec3::ZERO)`):
   - `InputMap::<FlightAction>` value from `default_input_map()` (leafwing 0.20 spawns `InputMap` and `ActionState` as separate components, NOT as a bundle — `InputManagerBundle` was deprecated in 0.20 in favor of required-components, mirroring Bevy 0.18's pattern)
   - `ActionState::<FlightAction>::default()` (leafwing required-component partner of `InputMap`)
   - `ExternalForce::default()` (Avian 0.6 component; default = zero force, non-persistent — i.e., auto-cleared each FixedUpdate per Avian 0.6's `ExternalForce::persistent` field defaulting to `false`. The thrust system writes to it each tick that an action is pressed; a quiet tick produces zero net force.)
   **And** the spawn-tuple component count grows from 10 → 13 (verified post-edit)
   **And** the `info!("spawned PlayerShip ...")` log line is unchanged (no new log noise per OnEnter)

4. **Given** `TuningConfig` is the project's single canonical gameplay-tuning struct (`src/tuning/config.rs:11-21`) and the Story 3.6 epic spec line 154 mandates a `ship_thrust_newtons: f32` field with default `500.0`
   **When** `src/tuning/config.rs` is extended
   **Then** the struct gains a single new field: `pub ship_thrust_newtons: f32`
   **And** the field uses the per-field `#[serde(default = "default_ship_thrust_newtons")]` pattern matching `outline_width` / `outline_color` precedent (forward-compat — preserves deserialization of pre-3.6 tuning.ron snapshots if a future story or external consumer encounters one)
   **And** a top-level `fn default_ship_thrust_newtons() -> f32 { 500.0 }` is added alongside the existing `default_outline_width` / `default_outline_color` helpers
   **And** `impl Default for TuningConfig` includes `ship_thrust_newtons: default_ship_thrust_newtons()` in its struct-literal
   **And** `assets/config/tuning.ron` is extended with `ship_thrust_newtons: 500.0,` placed after `outline_color: (...)` (consistent insert-at-end ordering with the existing 2.4 outline-fields precedent)
   **And** the existing `tuning_config_default_matches_ron_initial_values` test (config.rs:80-87) gains one assertion line: `assert_eq!(cfg.ship_thrust_newtons, 500.0);`
   **And** the existing `tuning_config_legacy_2_3_schema_uses_defaults_for_outline_fields` test (config.rs:101-110) gains one assertion line: `assert_eq!(cfg.ship_thrust_newtons, 500.0);` (same ron-bytes input — the absent field falls back to the serde default — which is the load-bearing forward-compat invariant)
   **And** the existing `tuning_config_deserializes_from_ron_bytes` test (config.rs:89-99) is extended: ron-bytes literal gains `, ship_thrust_newtons: 750.0` and assertion gains `assert_eq!(cfg.ship_thrust_newtons, 750.0);`
   **And** **NO** new tests are added to `config.rs` — the existing 3 tests cover (a) Default, (b) full deserialize, (c) legacy fallback; an additional test would be tautological

5. **Given** `FlightSystems` is currently a single-variant SystemSet (`src/flight/mod.rs:17-20`) and architecture.md:411-412 prescribes the eventual `enum FlightSystems { ReadInput, ApplyForces, IntegratePhysics }` shape
   **When** Story 3.6 extends `FlightSystems`
   **Then** the enum gains exactly one new variant: `ApplyForces` (matches architecture.md:411 verb-phrase). **NOT** added: `ReadInput` (leafwing's `InputManagerPlugin` updates `ActionState` automatically in `PreUpdate` — no project-owned ReadInput system exists), `IntegratePhysics` (Avian owns its own internal `PhysicsSet::*`; we don't proxy it), `Translate` / `Rotate` (3.7 owns rotation in the same `ApplyForces` set since linear and angular forces sum independently in Avian — no inter-set ordering needed)
   **And** the enum order in source becomes `[Setup, ApplyForces]` (preserves Setup as variant 0 — defensive against any non-derive consumer that relies on `as u8` ordinal, though none exist today)
   **And** `FlightPlugin::build` configures the new set in `FixedUpdate` (matches the architecture.md:240 "physics in FixedUpdate at 60 Hz" rule and epic-3 line 156 "thrust-application system runs in `FixedUpdate` inside `FlightSystems`"): `app.configure_sets(FixedUpdate, FlightSystems::ApplyForces);` — placed AFTER the existing `OnEnter(Arena)` configure_sets call. The existing `OnEnter(Arena)` chained-sets configuration is unchanged (Setup variant continues to live in OnEnter; ApplyForces lives in FixedUpdate; the two are independent schedule registrations)
   **And** `apply_thrust` is registered via `app.add_systems(FixedUpdate, apply_thrust.in_set(FlightSystems::ApplyForces).run_if(in_state(GameState::Arena)));` — the `run_if` gate prevents the system from running in MainMenu / Loading / Paused states (Paused is critical: Story 3.4 freezes `Time::Physics` clock so `ExternalForce` integration is no-op, but executing the system anyway would still touch the ECS each frame for nothing — the run_if gate is a small efficiency win + a clear "input doesn't drive physics outside Arena" boundary)

6. **Given** the architecture.md:558-563 file-layout prescription for `src/flight/` (sub-files: `components.rs`, `input.rs`, `physics.rs`, `camera.rs`) AND Story 3.5 Dev Notes line 330 forward-compat ("Stories 3.6 / 3.7 / 3.8 will introduce `flight/input.rs` + `flight/physics.rs` when the file would otherwise exceed ~250 lines")
   **When** Story 3.6 introduces FlightAction + apply_thrust
   **Then** `src/flight/input.rs` (NEW) is authored — owns `FlightAction` enum + `default_input_map()` (target size: ~30–60 lines)
   **And** `src/flight/physics.rs` (NEW) is authored — owns `apply_thrust` system + the `ship_local_thrust_vector(action_state, transform) -> Vec3` pure-function helper (target size: ~50–90 lines including a co-located `#[cfg(test)] mod tests` for the helper)
   **And** `src/flight/mod.rs` declares both: `pub mod input;` + `pub mod physics;` placed after the doc comment lines, before the existing `use ...` block. The mod.rs file size grows from 93 → ~115 lines (still under the 250-line split-trigger threshold for `mod.rs` itself; the new code lives in the sub-files, and `mod.rs` is the orchestration layer)
   **And** the rationale for splitting NOW (not deferring to Story 3.7) is documented in Dev Notes: combining input + physics into one ~120-line `mod.rs` addition would push the file to ~210 lines, leaving Story 3.7 (rotation) and 3.8 (dampener) no room before the 250-line split trigger forces a panic-split mid-implementation. Splitting clean at 3.6 boundaries is the lower-friction path. (`flight/components.rs` and `flight/camera.rs` remain unintroduced — those land at 3.8 dampener and a future cockpit-comfort story respectively per architecture.md:563.)

7. **Given** the `apply_thrust` system reads `ActionState<FlightAction>` and writes thrust as ship-local force into `ExternalForce`
   **When** the system runs in `FixedUpdate` inside `FlightSystems::ApplyForces`
   **Then** the system signature is:
   ```rust
   pub fn apply_thrust(
       tuning_assets: Res<Assets<TuningConfig>>,
       tuning_handle: Res<TuningHandle>,
       mut ships: Query<(&Transform, &ActionState<FlightAction>, &mut ExternalForce), With<PlayerShip>>,
   )
   ```
   **And** the system body (a) extracts `ship_thrust_newtons` from `TuningConfig` via the same `tuning_assets.get(tuning_handle.0.id()).cloned().unwrap_or_default()` cold-start-fallback pattern as `spawn_player_ship` and `spawn_arena_zone` (no warn! log here — the warn lives at spawn time; per-tick warnings would spam at 60 Hz), (b) iterates the query (one match expected — but the loop is correct even if a future story spawns multiple ships), (c) computes `force_vec = ship_local_thrust_vector(action_state, transform) * ship_thrust_newtons`, (d) writes via `external_force.set_force(force_vec)` (Avian 0.6 API for one-shot per-FixedUpdate force; the auto-clear behavior of `ExternalForce` with `persistent = false` ensures stale forces don't leak across ticks even when no action is pressed)
   **And** the helper `ship_local_thrust_vector` returns the **unit-magnitude** sum of pressed-action axes in ship-local space, transformed to world space:
   - `ThrustForward` contributes `transform.forward()` (Bevy 0.18: `Transform::forward()` returns `-Z` in local space, mapped to world via the transform's rotation — exact "where the nose points" semantic per the user story)
   - `ThrustReverse` contributes `transform.back()` (== `-forward()`)
   - `StrafeLeft` contributes `transform.left()`
   - `StrafeRight` contributes `transform.right()`
   - `ThrustUp` contributes `transform.up()`
   - `ThrustDown` contributes `transform.down()`
   - The contributions sum vector-style — pressing W+D yields a diagonal forward-right unit-ish vector (magnitude up to `√2` for two-axis, `√3` for three-axis composites — the magnitude is NOT clamped to 1.0 in 3.6; the epic explicitly states "the forces sum"; clamping would silently change behavior and is reserved for a future "max-thrust" tuning story if it ever surfaces)
   - Return value is `Vec3::ZERO` if no action is pressed (caller multiplies by `ship_thrust_newtons` → still `Vec3::ZERO` → `set_force(Vec3::ZERO)` → no acceleration that tick)

8. **Given** the epic-3 line 158-161 acceptance bullet: "the player presses `ThrustForward` in Arena → 2 seconds elapse → ship's world linear velocity is approximately `(ship_thrust_newtons / mass) * 2` m/s forward within 10% integration tolerance"
   **When** the dev runs the runtime smoke (Task 4)
   **Then** the dev visually confirms the ship moves forward when W is held — but the **quantitative** 10%-tolerance check is recorded as a Dev Agent Record observation rather than enforced via an automated test (rationale: an automated `cargo test` of the velocity-after-2-seconds invariant would require a Bevy `App`-bootstrap integration test with `MinimalPlugins + PhysicsPlugins + tick FixedUpdate manually for 120 ticks at 60 Hz`. Architecture.md:354 defers integration tests post-M3; the runtime smoke is the de-facto integration test for now)
   **And** the dev records in Dev Agent Record: actual observed velocity at t≈2s with W held from origin in zero-gravity (e.g., "after holding W for ~2s, observed ship-local-space distance traveled ≈ 30 m"). If the observed acceleration deviates >50% from the spec'd `(500 / mass)` target — implying the inferred mass from `Collider::sphere(2.0)` produces an unintuitive thrust feel — the dev exercises the deferred-work.md:206 escape hatch and adds explicit `Mass(M)` + (optionally) `Inertia(I)` components to `spawn_player_ship`'s spawn tuple, with the chosen `M` documented in Dev Agent Record + flagged in Completion Notes for future reference. Avian 0.6 default density is 1.0 → `(4/3)π·2³ ≈ 33.5 m³` mass, so the natural target velocity at thrust=500 is `500/33.5 * 2 ≈ 30 m/s` — feels reasonable for a 200×200×200 m arena, and the dev should NOT add `Mass` proactively unless the smoke contradicts the math
   **And** the helper `ship_local_thrust_vector` is **unit-tested** (the only pure-logic surface in 3.6; first-class test target per architecture.md:353): three tests — (a) no-action → `Vec3::ZERO`, (b) `ThrustForward` only with `Transform::IDENTITY` → `Vec3::NEG_Z` (Bevy convention: forward = -Z), (c) `ThrustForward + StrafeRight` with `Transform::IDENTITY` → `Vec3::new(1.0, 0.0, -1.0)` (verifies the sum semantic + the documented non-clamp behavior)

9. **Given** the deferred-work.md:214 entry from Story 3.5 code review: "Pause resume may re-trigger OnEnter(Arena) — double-spawn unverified — Story 3.6 will make this immediately visible if it's real (two ships fighting over input forces)"
   **When** the dev runs the runtime smoke (Task 4) and triggers Esc-pause then Esc-resume
   **Then** the dev visually verifies whether the PlayerShip is preserved across the pause cycle. **Expected outcome based on Bevy 0.18 + Avian 0.6 semantics:** OnExit(Arena) DOES fire on the Arena→Paused transition → `cleanup_on_exit::<ArenaEntity>` despawns the PlayerShip → OnEnter(Arena) DOES fire on the Paused→Arena transition → `spawn_player_ship` runs again → ship re-appears at origin with zero velocity, losing all in-flight state
   **And** the dev verifies via `grep -c 'spawned PlayerShip' /tmp/story-3-6-run.log`: a single Esc-pause-resume cycle should produce **2** "spawned PlayerShip" log lines (1 initial + 1 resume), confirming the despawn/respawn behavior
   **And** if the bug manifests (≥ 2 spawned-PlayerShip log lines per pause cycle), the dev does **NOT** attempt to fix it in Story 3.6 — instead, the dev:
   - Documents the confirmed-bug observation in Dev Agent Record's "Deviations" section
   - Appends a new entry to `_bmad-output/implementation-artifacts/deferred-work.md` titled "**Pause/resume cycle teleports PlayerShip to origin (state-scoped cleanup conflicts with flat-state pause)**" with two suggested resolution paths: (A) refactor pause to use `Time::Virtual::pause()` only without GameState transition (keep state in Arena, gate UI overlay on `Res<PauseInitiator>` instead) — preferred since it preserves all entity state through pause; (B) sub-state pause (`#[states(parent = GameState::Arena)] enum ArenaPauseState`) — heavier but architecturally cleanest
   - Resolution priority: defer to a dedicated **Pause-UX-pass story** (likely between Stories 3.10 and 3.11 since 3.7/3.8/3.9 all add per-tick state that would also be lost on pause-respawn)
   **And** if the bug does NOT manifest (only 1 `spawned PlayerShip` log line per pause cycle, indicating Bevy's flat-state machine somehow preserves the entity OR `OnExit` is not firing) — even more important to document: the deferred-work.md:214 entry gets updated with `✅ FALSE-POSITIVE 2026-XX-XX by Story 3.6` + the reasoning observed
   **And** **regardless of bug outcome**, Story 3.6's primary deliverables (input + thrust + tuning) ship as scoped — the pause/resume issue is documented but not blocking

10. **Given** the post-3.5 source baseline (test count = 21 per Story 3.5 Dev Agent Record; `cargo build --release` 0 warnings; `src/flight/mod.rs` = 93 lines; `src/flight/input.rs` does not exist; `src/flight/physics.rs` does not exist; `src/tuning/config.rs` = 111 lines; `assets/config/tuning.ron` = 7 lines)
    **When** Story 3.6 verification runs locally (per `feedback_full_build_output.md` — exit-0 + tail is NOT proof; grep explicitly per command, capture each to `/tmp/story-3-6-<command>.log`)
    **Then** **all six** of `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --release` produce **0** lines matching `grep -cE 'warning:|error:'` per their respective full output logs
    **And** `cargo test` summary line reads `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N = 24** (= 21 baseline + 3 new `ship_local_thrust_vector` tests in `src/flight/physics.rs`; the +0 deltas in `src/tuning/config.rs` per AC #4 are expected — the existing 3 tests are extended in-place, no new test functions added)
    **And** the runtime smoke (Task 4) verifies all of: (a) ship moves forward when W held, (b) ship moves reverse when S held, (c) ship strafes when A/D held, (d) ship ascends/descends when Space/Ctrl held, (e) diagonal motion works (W+D simultaneously → forward-right drift), (f) released keys → ship continues to drift (Newtonian — no dampener; that's 3.8), (g) Esc → pause overlay appears (Story 3.4 still works), (h) Esc again → resume (whether the ship's flight state is preserved is the AC #9 observation, not a pass/fail criterion for 3.6)
    **And** `/tmp/story-3-6-run.log` contains: 1 occurrence of `entered Loading`, 1 of `entered MainMenu`, ≥ 1 of `entered Arena`, ≥ 1 of `spawned PlayerShip` (`grep -c` ≥ 1 — exact count is the AC #9 observation), 0 of `panic|backtrace|FATAL`, 0 of `ambiguous.*camera.*order` (Story 3.5 regression check still holds)
    **And** `git status --short` final set is **exactly**: `src/flight/input.rs` (?? — new file), `src/flight/physics.rs` (?? — new file), `src/flight/mod.rs` (M — extended for leafwing plugin + new SystemSet variant + InputMap+ActionState+ExternalForce in spawn tuple + new `pub mod` declarations), `src/tuning/config.rs` (M — new field + helper + Default + 3 test extensions), `assets/config/tuning.ron` (M — 1 new line), `_bmad-output/implementation-artifacts/sprint-status.yaml` (M — bookkeeping), `_bmad-output/implementation-artifacts/3-6-flight-input-6-dof-translation.md` (M — this file's Status flip + Dev Agent Record), `_bmad-output/implementation-artifacts/deferred-work.md` (M — at minimum the deferred-work.md:214 entry update with the AC #9 observation, plus any new forward-compat entries surfaced during impl); **NO** entries under `Cargo.toml` (no dep added — leafwing 0.20 already pinned), `Cargo.lock` (the leafwing transitive deps will materialize on first `cargo build` after the new import lands — one-time `Cargo.lock` delta is expected; if `Cargo.lock` shows churn beyond the new leafwing-related entries, investigate before committing), `src/main.rs` (no plugin re-registration; FlightPlugin already wired in 3.5), `src/state.rs` (per `feedback`-style guidance — `Copy` derive remains deferred unless 3.6 hits an actual `State<GameState>` clone path; the `apply_thrust` system uses `run_if(in_state(GameState::Arena))` which handles cloning internally inside the `in_state` combinator), `src/arena/**`, `src/pause/**`, `src/visual/**`, `src/ui/**`, `src/splash.rs`, `src/logging.rs`, `assets/strings/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`

## Tasks / Subtasks

- [x] **Task 1: Author `src/flight/input.rs` — FlightAction enum + default keyboard binding map** (AC: #1, #2)
  - [x] Create `src/flight/input.rs`. Target size: **~30–60 lines** including module doc, enum, derive block, default_input_map() function, no tests (the enum is trivial; binding-map correctness is exercised by the runtime smoke when the dev presses each key).
  - [x] **Module doc** 1–2 lines max, no story-id references (Story 1.5 review patch BH8 + Story 3.2 commit `5134b3c` precedent — see deferred-work.md history). Suggested: `//! FlightAction enum + default keyboard bindings (FR1 keyboard input → FR2 6-DOF translation).`
  - [x] **Imports:**
    ```rust
    use bevy::prelude::*;
    use leafwing_input_manager::prelude::*;
    ```
    The leafwing prelude is the documented entry point per leafwing 0.20's docs.rs; using individual symbol imports (`InputMap`, `Actionlike`, etc.) is theoretically cleaner but the prelude is the maintained surface and using it follows the upstream convention. Per anti-pattern #2 in Story 3.5 Dev Notes: avoid wildcard imports BEYOND `bevy::prelude::*` and the equivalent third-party preludes. Leafwing's prelude is in the same category as bevy's prelude, NOT a deep-import shortcut.
  - [x] **FlightAction enum:**
    ```rust
    #[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
    pub enum FlightAction {
        ThrustForward,
        ThrustReverse,
        StrafeLeft,
        StrafeRight,
        ThrustUp,
        ThrustDown,
    }
    ```
    - **Why `Actionlike`:** leafwing 0.20's required trait for any input-action enum. The derive macro generates `IntoIterator`, `variants()`, `n_variants()`, etc., used internally by `InputMap` to enumerate actions during input-poll cycles.
    - **Why `Reflect`:** leafwing 0.20's `Actionlike` blanket impl requires `Reflect` (verify via `cargo check` after declaration; if leafwing-0.20 has loosened this, the derive can be omitted — Reflect is heavyweight and unnecessary for a non-bevy-reflect-using enum).
    - **Why `Copy + Clone`:** ActionState clones the enum value internally; making it `Copy` is the idiomatic choice for trivially-copyable data.
    - **Why no `Default`:** Actionlike's blanket impl provides default-iter behavior; the enum doesn't need a "neutral" variant. Adding `Default` would force a "no-action" variant which collides with the natural "no key pressed" semantic represented by `ActionState::just_released() / pressed() = false`.
  - [x] **`default_input_map()` function:**
    ```rust
    pub fn default_input_map() -> InputMap<FlightAction> {
        InputMap::new([
            (FlightAction::ThrustForward, KeyCode::KeyW),
            (FlightAction::ThrustReverse, KeyCode::KeyS),
            (FlightAction::StrafeLeft, KeyCode::KeyA),
            (FlightAction::StrafeRight, KeyCode::KeyD),
            (FlightAction::ThrustUp, KeyCode::Space),
            (FlightAction::ThrustDown, KeyCode::ControlLeft),
        ])
    }
    ```
    - **Why `KeyCode::ControlLeft` not `KeyCode::ControlRight`:** the epic spec says "LCtrl" → leafwing/winit's idiom is `ControlLeft`. Right-Ctrl users get a rebinding-flow miss; that's accepted per FR37's "post-MVP rebinding" promise.
    - **Why `InputMap::new(slice-of-tuples)`:** leafwing-0.20's terse constructor; equivalent to `let mut map = InputMap::default(); map.insert(action, key); ...; map` but one-liner. Verify the exact constructor signature in leafwing-0.20's docs at `cargo check` time — if `InputMap::new` accepts `impl IntoIterator<Item = (A, UserInput)>` with `From<KeyCode> for UserInput`, the slice-of-tuples form compiles directly. If not, fall back to the chained `.insert(...)` form.
  - [x] **No tests in `input.rs`** — the FlightAction enum has no logic; the binding-map content is configuration data that's trivially correct by inspection and runtime-verified via Task 4's smoke. A test like `assert_eq!(default_input_map().get(&FlightAction::ThrustForward), Some(&KeyCode::KeyW.into()))` is tautological — re-encoding the same configuration in two places.
  - [x] **Verify post-edit:** `cargo check` produces 0 warnings/errors. The new file is unreferenced at this point (mod.rs declaration comes in Task 3); a temporary "dead code" warning may surface — accept until Task 3 lands. If `-D warnings` is too strict, this task and Task 3 are inseparable; do them together before re-running clippy.

- [x] **Task 2: Extend `src/tuning/config.rs` — `ship_thrust_newtons` field + Default impl + tuning.ron + 3 test extensions** (AC: #4)
  - [x] In `src/tuning/config.rs`, add a `pub ship_thrust_newtons: f32` field to the `TuningConfig` struct, AFTER the `outline_color` field (insert-at-end ordering per Story 2.4 precedent). Annotate with `#[serde(default = "default_ship_thrust_newtons")]`.
  - [x] Add the helper function: `fn default_ship_thrust_newtons() -> f32 { 500.0 }` placed alongside the existing `default_outline_width` and `default_outline_color` helpers (they live between the struct and the `Default` impl).
  - [x] Update `impl Default for TuningConfig`'s struct-literal: add `ship_thrust_newtons: default_ship_thrust_newtons()` as the last field.
  - [x] In `assets/config/tuning.ron`, append `ship_thrust_newtons: 500.0,` as the last field, BEFORE the closing `)` paren. Trailing comma is correct per RON-0.8 convention. Final file size: 8 lines (was 7).
  - [x] Extend the existing 3 tests:
    - `tuning_config_default_matches_ron_initial_values` (config.rs:79-87): add `assert_eq!(cfg.ship_thrust_newtons, 500.0);` as the last assertion.
    - `tuning_config_deserializes_from_ron_bytes` (config.rs:89-99): edit the `bytes` literal to add `, ship_thrust_newtons: 750.0` before the closing `)`. Add `assert_eq!(cfg.ship_thrust_newtons, 750.0);` as the last assertion.
    - `tuning_config_legacy_2_3_schema_uses_defaults_for_outline_fields` (config.rs:101-110): the bytes literal is unchanged (the absent ship_thrust_newtons field exercises the serde-default fallback). Add `assert_eq!(cfg.ship_thrust_newtons, 500.0);` as the last assertion. **Optional rename** of the test function to `tuning_config_legacy_schema_uses_defaults_for_added_fields` (drop the "2_3" specific reference + drop "outline" since the test now covers both 2.4 outline fields AND 3.6 ship_thrust). Renaming is a nice-to-have; skip if it bloats the diff.
  - [x] **Verify post-edit:** `cargo test --lib tuning::config` produces 3 passing tests (`tuning_config_default_matches_ron_initial_values`, `tuning_config_deserializes_from_ron_bytes`, `tuning_config_legacy_2_3_schema_uses_defaults_for_outline_fields`) — same count as pre-3.6, just enriched assertions. Expected total project test count after this task only (before Task 4 lands physics.rs tests): **21 still** (no new tests; just expanded assertions).

- [x] **Task 3: Wire `pub mod input;` + `pub mod physics;` in `src/flight/mod.rs`, register `InputManagerPlugin`, extend `FlightSystems`, register `apply_thrust`** (AC: #2, #3, #5, #6)
  - [x] At the top of `src/flight/mod.rs` (after the doc comment block, before the `use ...` statements), insert:
    ```rust
    pub mod input;
    pub mod physics;
    ```
  - [x] In the `use ...` block, add: `use leafwing_input_manager::prelude::*;`. Place between `use bevy::prelude::*;` and `use bevy_mod_outline::OutlineVolume;` (alphabetical-ish — `bevy_mod_outline` comes after `leafwing_input_manager`? — let rustfmt land the canonical ordering on `cargo fmt` and accept its choice).
  - [x] Add Avian's `ExternalForce` to the existing avian3d prelude import block: change line 4 from `use avian3d::prelude::{AngularVelocity, Collider, LinearVelocity, RigidBody};` to `use avian3d::prelude::{AngularVelocity, Collider, ExternalForce, LinearVelocity, RigidBody};` (sorted alphabetically). **Verify** that `ExternalForce` is exposed in Avian 0.6's prelude — per `avian3d/src/lib.rs:550-555` (the same prelude file referenced by Story 3.4 for `Physics` + `PhysicsTime`), the dynamics components including `ExternalForce` and `ExternalTorque` are re-exported. If the symbol doesn't resolve, fall back to the explicit path `use avian3d::dynamics::external::ExternalForce;` and document the deviation in Dev Agent Record.
  - [x] Add a use line for the local crate's input module: `use crate::flight::input::{FlightAction, default_input_map};`. (Use of `crate::flight::input::...` rather than `input::...` is intentional — sibling module imports through the crate root are more diff-resilient against future re-organization.)
  - [x] Extend the `FlightSystems` enum from `enum FlightSystems { Setup }` to:
    ```rust
    #[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum FlightSystems {
        Setup,
        ApplyForces,
    }
    ```
  - [x] In `FlightPlugin::build`, **append** (not replace) two new lines after the existing two `app.configure_sets` + `app.add_systems` calls:
    ```rust
    app.add_plugins(InputManagerPlugin::<FlightAction>::default());
    app.configure_sets(FixedUpdate, FlightSystems::ApplyForces);
    app.add_systems(
        FixedUpdate,
        physics::apply_thrust
            .in_set(FlightSystems::ApplyForces)
            .run_if(in_state(GameState::Arena)),
    );
    ```
    **Order rationale:** add_plugins FIRST (registers leafwing's internal systems before our systems try to read `ActionState<FlightAction>`), then configure_sets, then add_systems — matches Bevy idiom. The existing `OnEnter(Arena)` configure_sets + add_systems remain UNCHANGED at the top of the build function.
  - [x] In `spawn_player_ship`, extend the `commands.spawn((...))` tuple to include the three new components AT THE END (after `AngularVelocity(Vec3::ZERO)`):
    ```rust
    .spawn((
        PlayerShip,
        ArenaEntity,
        Mesh3d(ship_mesh),
        MeshMaterial3d(ship_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        outline,
        RigidBody::Dynamic,
        Collider::sphere(2.0),
        LinearVelocity(Vec3::ZERO),
        AngularVelocity(Vec3::ZERO),
        default_input_map(),                     // leafwing InputMap<FlightAction>
        ActionState::<FlightAction>::default(),  // leafwing required-component partner
        ExternalForce::default(),                // Avian — auto-cleared each FixedUpdate
    ))
    ```
    - **Why insert at the end of the spawn tuple, not interleaved with physics components:** Bevy 0.18 tuple-bundle insertion order is irrelevant for ECS storage but reads-cleanly when grouped semantically: marker → state-cleanup → render → outline → physics → input/forces. Story 3.5's spec called out the existing order; preserving it through 3.6 keeps diffs reviewable.
    - **Why `ExternalForce::default()` not `ExternalForce::new(Vec3::ZERO)`:** Avian 0.6's `Default` impl produces `force: Vec3::ZERO, torque: Vec3::ZERO, persistent: false` — exactly what we want. Using `::new(Vec3::ZERO)` would set persistent to whatever the constructor defaults to (likely `false` but unverified for Avian 0.6) and is more verbose for the same result.
  - [x] **Verify post-edit:** `cargo check` produces 0 warnings/errors (the dead-code warning from Task 1 disappears now that `mod input;` is declared and `default_input_map`/`FlightAction` are used in `spawn_player_ship`).

- [x] **Task 4: Author `src/flight/physics.rs` — `apply_thrust` system + `ship_local_thrust_vector` helper + 3 unit tests** (AC: #6, #7, #8)
  - [x] Create `src/flight/physics.rs`. Target size: **~50–90 lines** including module doc, imports, helper function, system, and `#[cfg(test)] mod tests`.
  - [x] **Module doc** 1–2 lines max: `//! 6-DOF translation thrust system (FR2). Reads ActionState<FlightAction>, writes ExternalForce.`
  - [x] **Imports:**
    ```rust
    use avian3d::prelude::ExternalForce;
    use bevy::prelude::*;
    use leafwing_input_manager::prelude::*;

    use crate::flight::PlayerShip;
    use crate::flight::input::FlightAction;
    use crate::tuning::TuningHandle;
    use crate::tuning::config::TuningConfig;
    ```
  - [x] **Helper function `ship_local_thrust_vector` — pure-logic, first-class test target:**
    ```rust
    /// Sum of pressed-action axes in ship-local space, transformed to world space.
    /// Magnitude is NOT clamped: pressing W+D returns √2 magnitude (epic spec — "forces sum").
    /// Returns Vec3::ZERO if no flight-translation action is pressed.
    pub fn ship_local_thrust_vector(
        action_state: &ActionState<FlightAction>,
        transform: &Transform,
    ) -> Vec3 {
        let mut force = Vec3::ZERO;
        if action_state.pressed(&FlightAction::ThrustForward) {
            force += transform.forward().as_vec3();
        }
        if action_state.pressed(&FlightAction::ThrustReverse) {
            force += transform.back().as_vec3();
        }
        if action_state.pressed(&FlightAction::StrafeLeft) {
            force += transform.left().as_vec3();
        }
        if action_state.pressed(&FlightAction::StrafeRight) {
            force += transform.right().as_vec3();
        }
        if action_state.pressed(&FlightAction::ThrustUp) {
            force += transform.up().as_vec3();
        }
        if action_state.pressed(&FlightAction::ThrustDown) {
            force += transform.down().as_vec3();
        }
        force
    }
    ```
    - **Why `.as_vec3()`:** Bevy 0.18's `Transform::forward()` returns a `Dir3` (newtype around `Vec3` with normalization invariant). `.as_vec3()` extracts the underlying `Vec3` for `+=`. If Bevy 0.18 has restored direct Vec3 returns, omit the call — the dev should `cargo check` and follow the type error.
    - **Why pressed() not just_pressed():** thrust should be CONTINUOUS while held (matches the 2-second integration target in epic AC). `just_pressed()` would only fire on the press-edge tick — the ship would barely accelerate.
  - [x] **System `apply_thrust`:**
    ```rust
    pub fn apply_thrust(
        tuning_assets: Res<Assets<TuningConfig>>,
        tuning_handle: Res<TuningHandle>,
        mut ships: Query<
            (&Transform, &ActionState<FlightAction>, &mut ExternalForce),
            With<PlayerShip>,
        >,
    ) {
        let tuning = tuning_assets
            .get(tuning_handle.0.id())
            .cloned()
            .unwrap_or_default();
        for (transform, action_state, mut external_force) in &mut ships {
            let force_vec =
                ship_local_thrust_vector(action_state, transform) * tuning.ship_thrust_newtons;
            external_force.set_force(force_vec);
        }
    }
    ```
    - **Why `for (...) in &mut ships` not single-result query:** the `Query<>` API doesn't have a clean `single_mut()` that's exception-free. Iterating handles the 0-ship case (no PlayerShip yet — no panic) and the 1-ship case (the only expected case in 3.6). A future story spawning multiple PlayerShips would also work. **PATTERN DEVIATION justified per architecture.md:454.**
    - **Why no `warn!` on cold-start tuning-not-loaded:** at 60 Hz this would emit 60 warns/sec. The warn lives at spawn time in `spawn_player_ship`. If tuning.ron loads mid-Arena (highly unlikely given Startup-phase load), the system silently switches to the new value — no log spam.
    - **Why `set_force` not `apply_force`:** Avian 0.6 `ExternalForce::set_force(Vec3)` overwrites the per-tick force; `apply_force(Vec3)` accumulates. Since we recompute from scratch each tick (the action state is fresh), `set_force` is the correct semantic. If `set_force` doesn't exist in Avian 0.6 (API renamed), use direct field assignment: `external_force.force = force_vec;`.
  - [x] **Tests** (`#[cfg(test)] mod tests` — 3 tests covering the pure helper):
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;
        use leafwing_input_manager::prelude::ActionState;

        fn no_input() -> ActionState<FlightAction> {
            ActionState::default()
        }

        fn pressed(actions: &[FlightAction]) -> ActionState<FlightAction> {
            let mut state = ActionState::default();
            for &a in actions {
                state.press(&a);
            }
            state
        }

        #[test]
        fn no_action_returns_zero_vector() {
            let v = ship_local_thrust_vector(&no_input(), &Transform::IDENTITY);
            assert_eq!(v, Vec3::ZERO);
        }

        #[test]
        fn forward_only_returns_neg_z_at_identity_orientation() {
            let v = ship_local_thrust_vector(
                &pressed(&[FlightAction::ThrustForward]),
                &Transform::IDENTITY,
            );
            // Bevy convention: forward = -Z in local space; identity transform → world -Z.
            assert!((v - Vec3::NEG_Z).length() < 1e-5, "expected -Z, got {:?}", v);
        }

        #[test]
        fn forward_plus_right_sums_with_unclamped_magnitude() {
            let v = ship_local_thrust_vector(
                &pressed(&[FlightAction::ThrustForward, FlightAction::StrafeRight]),
                &Transform::IDENTITY,
            );
            // Forward (-Z) + Right (+X) = (1, 0, -1); magnitude is √2 (deliberately unclamped per epic).
            assert!((v - Vec3::new(1.0, 0.0, -1.0)).length() < 1e-5, "got {:?}", v);
            assert!((v.length() - std::f32::consts::SQRT_2).abs() < 1e-5);
        }
    }
    ```
    - **Why `state.press(&a)` not `state.set_pressed(...)`:** leafwing 0.20's API; `press()` is the maintained method. If `press` is private/hidden, fall back to `state.set_action_data(...)` or use a public test helper. This is the most likely source of test-API friction; verify against `cargo test` output and adjust.
    - **Why test-helpers `no_input()` / `pressed()` factored out:** keeps each test's body focused on the "given/when/then" semantic without ActionState boilerplate. Standard test-helper pattern.
    - **Test count delta: +3** → project total after this task: **24** (= 21 pre-3.6 + 3 new from physics.rs).
  - [x] **Verify post-edit:** `cargo test physics::tests` produces 3 passing tests. `cargo clippy --all-targets -- -D warnings` produces 0 issues.

- [x] **Task 5: Local verification sweep — full `feedback_full_build_output.md` discipline** (AC: #10)

  Per Till's memory `feedback_full_build_output.md`: `cargo check` exit-0 + tail is NOT proof of correctness. Capture each command's full output to a log file, then grep for `warning:|error:` and confirm count is **0**.

  - [x] `cargo check 2>&1 | tee /tmp/story-3-6-check.log` — confirm `grep -cE 'warning:|error:' /tmp/story-3-6-check.log` returns **0**.
  - [x] `cargo build 2>&1 | tee /tmp/story-3-6-build.log` — confirm grep returns **0**.
  - [x] `cargo test 2>&1 | tee /tmp/story-3-6-test.log` — confirm grep returns **0** AND the summary line reads `test result: ok. 24 passed; 0 failed; 0 ignored; ...`. Test count: 24 = 21 baseline + 3 from `src/flight/physics.rs`.
  - [x] `cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-3-6-clippy.log` — confirm grep returns **0**.
  - [x] `cargo fmt --all -- --check 2>&1 | tee /tmp/story-3-6-fmt.log` — confirm exit code 0. If fmt drift exists, run `cargo fmt --all`, re-stage, and re-run `--check`.
  - [x] `cargo build --release 2>&1 | tee /tmp/story-3-6-release.log` — confirm grep returns **0**. Allow 4–6 min wall time on the LTO=fat + codegen-units=1 release build.
  - [x] **Cargo.lock delta check:** `git diff --stat Cargo.lock` should show changes confined to leafwing-input-manager + its transitive deps (verify by grep for `leafwing` in the diff). Any unexpected updates to Bevy / Avian / outline / kira are a red flag — investigate before committing.
  - [x] **Runtime smoke** — `RUST_LOG=info,wgpu=warn,naga=warn,avian3d=info cargo run 2>&1 | tee /tmp/story-3-6-run.log` — let the game reach Arena, then exercise:
    - (a) Hold **W** for ~2 seconds — visually confirm forward motion (ship moves toward the close-cluster asteroids at z=-25 to -42); release and confirm Newtonian drift (ship continues forward — no dampener yet).
    - (b) Press **S** to thrust reverse — confirm deceleration / reverse motion.
    - (c) Tap **A** / **D** — confirm strafing left/right.
    - (d) Tap **Space** / **Left Ctrl** — confirm ascend/descend.
    - (e) Press **W + D** simultaneously — confirm diagonal forward-right motion (the "forces sum" semantic).
    - (f) Press **Esc** → "PAUSED — Esc to resume" overlay appears (Story 3.4 still works).
    - (g) Press **Esc** again → resume to Arena. **OBSERVE** whether the ship is at its pre-pause position with pre-pause velocity (the AC #9 observation): if the ship snaps to origin with zero velocity, the pause/resume bug is confirmed → record observation per AC #9.
    - (h) Quit cleanly (window-close).
  - [x] **Post-runtime grep:**
    - `grep -c 'entered Loading'` → **1**
    - `grep -c 'entered MainMenu'` → **1**
    - `grep -c 'entered Arena'` → **1** initial; ≥ 2 if pause cycle was exercised (1 initial + 1 resume per cycle)
    - `grep -c 'spawned PlayerShip'` → **1** initial; ≥ 2 if pause cycle was exercised AND the despawn-respawn bug manifests; record actual count for AC #9 observation
    - `grep -cE 'panic|backtrace|FATAL'` → **0**
    - `grep -ci 'ambiguous.*camera.*order'` → **0** (Story 3.5 regression check)
    - `grep -cE 'ERROR.*avian|WARN.*Avian'` → **0**
  - [x] Confirm the pre-existing 3 documented WARNs from Story 3.5 reappear unchanged (splash-cleanup race per deferred-work.md:139, wgpu fragment-output per Story 2.3, winit Skipped Destroyed per Story 1.6 LOW-1) — these are NOT 3.6-introduced. If a fourth WARN appears, investigate and either explain it in Dev Agent Record or add a deferred-work entry.

- [x] **Task 6: Update `_bmad-output/implementation-artifacts/deferred-work.md`** (AC: #9 + the deferred-work.md:206 Mass/Inertia entry resolution)
  - [x] **Update deferred-work.md:206** (Story 3.5's `PlayerShip Mass / Inertia defaults inferred from Collider::sphere(2.0)`) with the dev's decision: either `✅ RESOLVED 2026-XX-XX by Story 3.6 — observed thrust feel acceptable, no Mass override added; default-density mass ≈ 33.5 kg @ thrust=500 N → ≈ 30 m/s after 2s, matches arena-scale flight feel` OR `✅ RESOLVED 2026-XX-XX by Story 3.6 — added explicit Mass(M) to spawn tuple based on smoke observation; reasoning: <observed-velocity vs target>`.
  - [x] **Update deferred-work.md:214** (Story 3.5 code review's "Pause resume may re-trigger OnEnter(Arena) — double-spawn unverified") with the AC #9 observation: either `✅ FALSE-POSITIVE 2026-XX-XX by Story 3.6 — pause cycle log shows 1 'spawned PlayerShip' per Arena entry, no double-spawn` OR `🚨 CONFIRMED 2026-XX-XX by Story 3.6 — pause cycle teleports ship to origin; new entry below at "Pause/resume cycle teleports PlayerShip" supersedes this`.
  - [x] **Conditionally add a new deferred-work entry** if AC #9 confirmed the bug:
    ```
    ## Deferred from: 3-6-flight-input-6-dof-translation (2026-XX-XX)

    - **Pause/resume cycle teleports PlayerShip to origin (state-scoped cleanup conflicts with flat-state pause)** — `src/pause/mod.rs:36-45` + `src/flight/mod.rs:38-41` + `src/arena/mod.rs:25-29`. When the player presses Esc in Arena: NextState(Paused) → OnExit(Arena) fires → cleanup_on_exit::<ArenaEntity> despawns PlayerShip + asteroids + DirectionalLight. On Esc-resume: NextState(Arena) → OnEnter(Arena) fires → spawn_arena_zone re-creates the field and spawn_player_ship re-creates the ship at origin with zero velocity. Net effect: pause = ship snaps to origin, all in-flight motion / aim / weapon-cooldown state lost. Confirmed in Story 3.6 runtime smoke (run log shows 2× 'spawned PlayerShip' per pause cycle). **Resolution paths:** (A) refactor pause to NOT transition GameState — keep state in Arena, gate UI overlay on Res<PauseInitiator>, rely solely on Time::Virtual::pause() + Time::Physics::pause() for sim freeze (Story 3.4 already pauses both clocks); preserves all entity state; minor refactor of pause/mod.rs + a re-jig of pause_simulation_clocks to NOT depend on OnEnter(Paused); (B) introduce a sub-state #[states(parent = GameState::Arena)] enum ArenaPauseState — heavier but architecturally cleanest; sub-state cleanup doesn't touch parent-state entities. **Resolution priority: defer to dedicated Pause-UX-pass story between Stories 3.10 and 3.11** — by 3.10, the player has flight + rotation + dampener + weapons + projectiles + asteroid-destruction state, all of which would also be wiped on the current pause path; the cumulative cost of NOT fixing this becomes severe enough by 3.10 to justify a dedicated story rather than another deferral. Source: Story 3.6 AC #9 observation. [`src/pause/mod.rs`, `src/flight/mod.rs`, `src/arena/mod.rs`]
    ```
  - [x] **Optionally** add forward-compat entries surfaced by the dev during 3.6 implementation (don't add proactively — YAGNI per `karpathy-guidelines.md`). Candidates that may surface:
    - leafwing 0.20 → newer-version migration friction signals (e.g., `Actionlike` blanket-impl trait set changing in 0.21+)
    - `ExternalForce::set_force` API rename in a future Avian version
    - A clamped-magnitude variant of `ship_local_thrust_vector` if the unclamped diagonal feels too fast in playtest

- [~] **Task 7: Sprint-status bookkeeping + commit/push (NOT YET — await Till's authorization)** (per Story 3.5 precedent)
  <!-- Sprint-status flipped to in-progress + Dev Agent Record populated; review-flip + commit/push subtasks below remain unchecked pending Till's runtime smoke + explicit commit authorization (Stories 3.1–3.5 cadence). -->

  - [x] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - [x] `3-6-flight-input-6-dof-translation: ready-for-dev → in-progress` — flipped 2026-05-01 at start of dev-story.
    - [x] `3-6-flight-input-6-dof-translation: in-progress → review` — flipped 2026-05-01 after Till's runtime-smoke confirmation.
    - [x] `last_updated:` bumped to `2026-05-01 (Story 3.6 in-progress → review — 6-DOF translation thrust verified)`.
  - [x] Update this story file's `Status:` field at line 3. Flipped `ready-for-dev → in-progress → review` after Till's runtime smoke confirmed (a) W → forward, (b) S → reverse, (e) W+D → diagonal, (f) Esc-pause → overlay, (h) clean window-close, AND no double-spawn on pause/resume cycle.
  - [x] Populate the `## Dev Agent Record` section: `Agent Model Used`, `Debug Log References` (the 7 commands' grep counts table — see Story 3.5 Dev Agent Record format), `Completion Notes List` (one bullet per AC #1–#10, including the AC #8 observed-velocity recording and AC #9 pause-cycle observation), `File List` (Added: `src/flight/input.rs`, `src/flight/physics.rs`; Modified: `src/flight/mod.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`, `sprint-status.yaml`, this file, `deferred-work.md`). NOTE: `Cargo.lock` is **NOT** in the modified-files list — leafwing's transitive deps were already locked from Story 1.2's plugin-compatibility build.
  - [ ] **Commit 1 (feat):** stage `src/flight/input.rs`, `src/flight/physics.rs`, `src/flight/mod.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`, `Cargo.lock`. Message: `feat: 6-DOF translation thrust + leafwing scaffold (Story 3.6)`. **DO NOT COMMIT UNLESS TILL AUTHORIZES.**
  - [ ] **Commit 2 (bmad):** stage `_bmad-output/implementation-artifacts/sprint-status.yaml`, `_bmad-output/implementation-artifacts/3-6-flight-input-6-dof-translation.md`, `_bmad-output/implementation-artifacts/deferred-work.md`. Message: `bmad: story 3.6 ready-for-dev → review (6-DOF translation thrust)`. **DO NOT COMMIT UNLESS TILL AUTHORIZES.**
  - [ ] **DO NOT push.** Push happens only after explicit authorization, AND only after Story 3.6 code review (`bmad-code-review`) passes per Story 3.5 precedent.

## Dev Notes

### Architecture compliance

- **Plugin home:** `FlightPlugin` in `src/flight/mod.rs` per architecture.md:558-563 (canonical FR1–FR8 location). Story 3.5 introduced the plugin; 3.6 extends it.
- **File split:** Story 3.6 lands `src/flight/input.rs` (FR1 leafwing Action enum per architecture.md:561, :673) AND `src/flight/physics.rs` (FR2 thrust application per architecture.md:562, :674). Both are first-time files; mod.rs declares them as `pub mod`. The `flight/components.rs` and `flight/camera.rs` slots (architecture.md:560, :563) remain unintroduced — `components.rs` lands at Story 3.8 dampener (when `InertialDampener` component arrives), `camera.rs` at a future cockpit-comfort polish story.
- **SystemSet name:** `FlightSystems::ApplyForces` matches architecture.md:411 example vocabulary (`enum FlightSystems { ReadInput, ApplyForces, IntegratePhysics }`). 3.6 declines to add `ReadInput` (leafwing's `InputManagerPlugin` owns input-reading in `PreUpdate`; no project-owned ReadInput system) and `IntegratePhysics` (Avian owns its physics-integration sets internally; we don't proxy them).
- **System naming:** `apply_thrust` (snake_case verb-phrase per architecture.md:323). Sister systems in 3.7 (`apply_torque` or `apply_rotation`) and 3.8 (`apply_dampener`) follow the same convention.
- **Helper naming:** `ship_local_thrust_vector` (descriptive snake_case for a pure free function). NOT a method on `FlightAction` (would force a `Transform` parameter into the action enum which has no business knowing about transforms).
- **Cross-plugin ordering:** none introduced by 3.6. The existing `(ArenaSystems::Setup, FlightSystems::Setup).chain()` at OnEnter(Arena) is unchanged. The new `FlightSystems::ApplyForces` set lives in `FixedUpdate` and has no cross-plugin dependencies (Avian's physics integration runs inside its own internal sets in FixedUpdate; our forces are applied to ECS state that Avian reads later in the same FixedUpdate cycle).
- **Run-condition gate:** `apply_thrust.run_if(in_state(GameState::Arena))` — prevents thrust application during MainMenu / Loading / Paused / Caravan / PostRun / PhotoMode. Architecturally cleaner than relying solely on `Time::Physics::pause()` (which freezes integration but doesn't prevent the system from running and computing zero-net forces). The `in_state` combinator handles `GameState` cloning internally — Story 3.6 does NOT need to add `Copy` to `GameState` (deferred-work.md:198 stays open).
- **Avian + Bevy + leafwing version pins:** `bevy = "0.18"`, `avian3d = "0.6"`, `bevy_mod_outline = "0.12"`, `leafwing-input-manager = "0.20"` (Cargo.toml:8-12) — all unchanged by 3.6. The leafwing crate becomes "live" (transitively reachable) for the first time; expect Cargo.lock to pull in leafwing's transitive deps (mostly `bevy_*` re-exports, `serde`, etc.) on first `cargo build` after the import lands.

### Library / framework specifics — leafwing-input-manager 0.20 (first consumer in this codebase)

- **`InputManagerPlugin::<A>::default()`:** registered in `FlightPlugin::build` BEFORE the ApplyForces system registration. The plugin owns:
  - A `PreUpdate` system that polls input state and updates `ActionState<A>` for every entity carrying an `InputMap<A>` + `ActionState<A>` pair.
  - Internal infrastructure for `Actionlike` enum reflection (used for default-state initialization).
  - Per-frame reset semantics for `just_pressed` / `just_released` flags.
- **`InputMap<A>` + `ActionState<A>` as separate components (NOT bundle):** leafwing 0.20 deprecated `InputManagerBundle<A>` in favor of required-components, mirroring Bevy 0.18's pattern (`Camera3dBundle` → `Camera3d`). The two components must both be present on the entity for input polling to work; spawning only `InputMap<A>` would cause leafwing's PreUpdate system to silently skip the entity. Verify post-spawn by querying `Query<(&InputMap<FlightAction>, &ActionState<FlightAction>), With<PlayerShip>>` — should match exactly 1 entity.
- **`Actionlike` derive macro:** generates iteration / variant-counting / hash impls. Required traits per the macro: `PartialEq, Eq, Hash, Clone, Copy, Debug`. Leafwing 0.20 also requires `Reflect` (for ECS reflection of dynamic action lookup). Verify with `cargo check` — if Reflect is no longer needed in 0.20.x, omit the derive.
- **`ActionState::pressed(&action)` vs `just_pressed` vs `value`:** for thrust, use `pressed()` — returns `true` on every tick that the bound key is held. `just_pressed()` only returns `true` on the press-edge tick (would barely accelerate). `value()` returns an `f32` axis value (relevant for analog inputs like gamepad sticks; for keyboard digital keys, value() returns 1.0 when pressed, 0.0 when released — equivalent to `pressed() as f32`).
- **`InputMap::new(slice-of-tuples)`:** terse 0.20 constructor. If the slice form doesn't compile (the `From<KeyCode> for UserInput` blanket-impl might require explicit `.into()`), fall back to chained `.insert(action, key)` calls or `.with(action, key)` builder pattern.
- **No `LeafwingInputPlugin` or similar bevy-side plugin:** the only bevy-side registration is `InputManagerPlugin::<A>::default()` per action enum. If a future story adds `CombatAction`, that's a SECOND `add_plugins(InputManagerPlugin::<CombatAction>::default())` — they don't conflict.

### Library / framework specifics — Avian 0.6 ExternalForce (in-codebase precedent: none yet; first consumer)

- **`ExternalForce` Component:** Avian 0.6's per-entity force accumulator. Default state: `force: Vec3::ZERO, torque: Vec3::ZERO, persistent: false`. With `persistent = false`, the force auto-clears each FixedUpdate after Avian's integration step — matches the "set every tick that input is pressed; quiet ticks → zero net force" semantic perfectly.
- **`ExternalForce::set_force(Vec3)`:** overwrites the per-tick force value. Use this when computing the force from scratch each tick (3.6's pattern). If the Avian 0.6 method is named differently (e.g., `apply_force` is documented elsewhere), use direct field access: `external_force.force = force_vec;`.
- **`ExternalForce::apply_force(Vec3)`:** accumulates force across multiple writes within the same tick. Use this when multiple systems each contribute to the same entity's force budget (e.g., 3.6 thrust + 3.8 dampener writing to the same ExternalForce). For 3.6 alone, `set_force` is correct; if 3.8's dampener wires into the SAME ExternalForce component, both 3.6 and 3.8 would need to migrate to `apply_force` + `clear()` discipline. Defer the migration to 3.8 — for now, `set_force` is the correct minimal-scope API.
- **Mass / Inertia:** Avian 0.6 auto-computes from `Collider` shape × default density (1.0). For `Collider::sphere(2.0)`: volume = `(4/3)π(2)³ ≈ 33.5 m³`, density 1.0 → mass ≈ 33.5 kg. Acceleration at thrust=500 N → `500/33.5 ≈ 14.9 m/s²` → after 2s elapsed → ~30 m/s. This is the natural starting point; the deferred-work.md:206 escape hatch lets 3.6 add explicit `Mass(M)` if the smoke contradicts the math.
- **Force application timing:** `apply_thrust` runs in `FixedUpdate` inside `FlightSystems::ApplyForces`. Avian's internal physics sets (`PhysicsStepSet::*`) run later in the same FixedUpdate cycle. The data flow is: our system writes ExternalForce → Avian reads ExternalForce + LinearVelocity + Mass + computes new LinearVelocity (XPBD step) + writes back → Bevy's transform-propagation system (later in the schedule) picks up the new Transform. This single-tick latency is correct and deterministic.

### File structure requirements

```
src/
├── flight/
│   ├── mod.rs               # MODIFIED — pub mod input/physics; +1 SystemSet variant; +leafwing plugin reg; +3 components in spawn tuple
│   ├── input.rs             # NEW — FlightAction enum + default_input_map() (~30–60 lines)
│   └── physics.rs           # NEW — apply_thrust system + ship_local_thrust_vector helper + 3 unit tests (~50–90 lines)
├── arena/                   # UNCHANGED
├── pause/                   # UNCHANGED
├── tuning/
│   ├── mod.rs               # UNCHANGED
│   └── config.rs            # MODIFIED — +1 field, +1 helper, +1 Default literal, +3 test extensions (in-place)
├── ui/                      # UNCHANGED
├── visual/                  # UNCHANGED
├── state.rs                 # UNCHANGED — Copy derive remains deferred per deferred-work.md:198
├── splash.rs                # UNCHANGED
├── logging.rs               # UNCHANGED
└── main.rs                  # UNCHANGED — FlightPlugin already registered in 3.5
assets/
├── config/
│   └── tuning.ron           # MODIFIED — +1 line (ship_thrust_newtons: 500.0,)
└── ...                      # UNCHANGED
Cargo.toml                   # UNCHANGED — leafwing 0.20 already pinned (Story 1.2)
Cargo.lock                   # MODIFIED — first-time leafwing transitive deps materialize
```

### Testing standards

Per architecture.md:351-354:
- **Co-located** `#[cfg(test)] mod tests { ... }` at the bottom of each module file.
- **Pure-logic modules first-class test targets;** integration tests deferred post-M3.

Story 3.6's `apply_thrust` system is integration-test-shaped (would need `MinimalPlugins + PhysicsPlugins + tick FixedUpdate manually for 120 ticks`) and therefore not unit-tested. The pure-logic helper `ship_local_thrust_vector` IS unit-tested (3 tests covering: no-input, single-axis, two-axis-sum).

`TuningConfig::ship_thrust_newtons` adds NO new test functions — the existing 3 tests are extended in-place to cover the new field. Test count delta from tuning: **+0**.

**Net post-3.6 test count target: 24** (= 21 baseline + 3 from `flight/physics.rs`). AC #10 enforces N = 24.

### Anti-patterns to avoid (catalogued from Stories 1.5–3.5 review precedent + 3.6-specific risks)

1. **Story-id references in module doc-comments** — Stories 1.5/3.2 review patches removed all "Story X.Y" references. Module docs describe what the module owns. **Do NOT** write `//! Story 3.6 introduces FlightAction ...`.
2. **Wildcard imports beyond `bevy::prelude::*`** — explicit imports per architecture.md naming-discipline. Exception: leafwing's `prelude::*` is the documented entry point and is in the same category as Bevy's prelude.
3. **`InputManagerBundle<A>` (deprecated bundle pattern)** — leafwing 0.20 deprecated bundles in favor of required-components. Use `InputMap::<A>` + `ActionState::<A>::default()` as separate spawn-tuple entries.
4. **`.after(specific_function)` for system ordering** — architecture.md:415 forbidden. Use SystemSet membership + chain.
5. **`just_pressed` for thrust** — fires only on press-edge tick. Use `pressed` for continuous thrust while held.
6. **Clamping the thrust-vector magnitude** — epic spec line 165 explicitly says "the forces sum (diagonal motion in ship-local space)". A clamp would silently break this AC. If diagonal feel is too fast in playtest, that's a future tuning concern.
7. **Adding `Mass` / `Inertia` proactively** — deferred-work.md:206 hands the decision to 3.6 based on observed thrust feel. Default-density mass is the starting point; only add explicit Mass if the AC #8 smoke shows >50% deviation from the spec target.
8. **Touching `src/state.rs`** — `apply_thrust.run_if(in_state(GameState::Arena))` handles cloning internally inside the `in_state` combinator. The `Copy` derive remains deferred per deferred-work.md:198.
9. **Adding rotation handling in 3.6** — `Pitch` / `Yaw` / `RollLeft` / `RollRight` variants on FlightAction belong to Story 3.7. Adding them prematurely fragments 3.7's scope and creates a half-implementation.
10. **Adding firing handling in 3.6** — `FirePrimary` belongs to a `CombatAction` enum in `src/combat/input.rs` (Story 3.9 epic spec line 238). Story 3.6's `FlightAction` is for navigation only.
11. **Adding cursor-grab in 3.6** — Story 3.7 epic spec line 197 introduces `CursorGrabMode::Confined` for mouse rotation. Adding it in 3.6 with no mouse handler creates a "the cursor is locked but nothing reads mouse motion" UX bug.
12. **Touching `src/main.rs`** — FlightPlugin is already registered (Story 3.5). leafwing's `InputManagerPlugin` is registered INSIDE `FlightPlugin::build`, not at the App level. Adding it at the App level would split 3.6's surface across two files unnecessarily.
13. **Adding `Cargo.toml` deps** — leafwing 0.20 already pinned. Cargo.lock will see transitive-dep churn on first build; that's expected.
14. **Recreating `ExternalForce` per tick** — the component is part of the spawn tuple and lives for the entity's lifetime. Don't `commands.entity(e).insert(ExternalForce::default())` per tick — that's expensive and unnecessary. Use `Query<&mut ExternalForce>` and `.set_force(...)` to reuse the existing component.
15. **Logging per-tick at 60 Hz** — no `info!` / `warn!` inside `apply_thrust`. The cold-start tuning warn lives at spawn time only. Per-tick logs would emit 60 lines/sec.
16. **Splitting `flight/input.rs` and `flight/physics.rs` based on AC counts** — the architecture-prescribed file boundaries are by RESPONSIBILITY, not story. `input.rs` owns input-binding configuration; `physics.rs` owns force-vector math. Story 3.7 will ADD to both files (rotation actions in input.rs, rotation system in physics.rs); it will NOT create separate `flight/rotation_input.rs` / `flight/rotation_physics.rs`.

### Logging discipline

Per architecture.md:376-383:
- `info!` for lifecycle events: the existing `info!("spawned PlayerShip ...")` in `spawn_player_ship` is unchanged. NO new lifecycle logs added by 3.6.
- NO per-tick logs in `apply_thrust`. Even `debug!` is avoided — at 60 Hz it spams `RUST_LOG=debug` runs.
- NO `warn!` per-tick. The cold-start tuning warn at spawn time covers the "tuning.ron not loaded" edge case once; per-tick re-warns would be noise.

### Project Structure Notes

- **Alignment with unified project structure:** `src/flight/input.rs` matches architecture.md:561; `src/flight/physics.rs` matches architecture.md:562. The split at 3.6 is the architecture's prescribed shape — not a deviation.
- **Detected variances:** none. Story 3.6 follows established Story 3.2 / 3.3 / 3.4 / 3.5 patterns.

## Previous Story Intelligence (Story 3.5 — Cockpit Camera + PlayerShip Entity)

Story 3.5 is the most recent reference for the development pattern. Key learnings to inherit:

- **Component-tuple ordering** (3.5 Task 1): marker first → state-cleanup marker → render → outline → physics → forces. Story 3.6 appends new components AT THE END (input + force) preserving the established convention.
- **`Camera3d::default()` (NOT `Camera3dBundle`)** (3.5 Task 1): same required-components pattern transfers to leafwing's `InputMap` + `ActionState` (deprecated `InputManagerBundle`).
- **Cold-start tuning fallback** (3.5 Task 1, mirrors 3.3 zone.rs:48-54): `tuning_assets.get(handle).cloned().unwrap_or_default()` + a one-shot `warn!` at spawn time if `None`. Story 3.6's `spawn_player_ship` extension does NOT need a new warn (the existing 3.5 warn covers the same path); `apply_thrust` does NOT warn at all (per-tick warn would spam at 60 Hz).
- **Avian prelude trait imports** (3.4 Deviation #2 + 3.5 Library specifics): when a method "doesn't exist" on an Avian type, suspect a missing prelude import. Story 3.6 expands the existing Avian prelude import to add `ExternalForce`. If `ExternalForce` is not in the prelude, fall back to the explicit path and document.
- **`MessageReader` not `EventReader`** (3.4 Five-key constraint #2): Story 3.6 doesn't read events, so this is N/A. But if future iteration adds `WeaponFired` or `ThrustChanged` events, remember the Bevy 0.18 Event/Message split.
- **`commands.insert_resource(...)` over `ResMut<T>`** (3.4 idiom): Story 3.6 doesn't manipulate Resources at runtime — only `Res<TuningConfig assets>`, `Res<TuningHandle>`. Pattern is N/A here but worth knowing.
- **Per-command grep verification harness** (3.4/3.5 Task 4): Story 3.6 mirrors per AC #10 + Task 5. The 7-command + runtime-smoke sweep is the canonical local-verification pattern.
- **2-commit pattern (feat + bmad)** (3.4/3.5 commit precedent): Story 3.6's Task 7 mirrors. Commits and pushes await Till's authorization.
- **Test budget** (3.4/3.5 precedent): 3.6 lands 3 tests in `flight/physics.rs` (the only pure-logic surface) + extends 3 existing tests in `tuning/config.rs` in-place. Net new test functions: 3. Net total test count: 24.
- **Pause/resume coverage gap** (3.5 Deviation #1, 3.5 code-review EC-02 → deferred-work.md:214): Story 3.6 EXERCISES pause/resume for the first time (per AC #9). Either confirms or invalidates the 3.5 deferred suspicion.
- **`SemanticAccent::PlayerOwned` deferral to 4.5** (3.5 + deferred-work.md:204): Story 3.6 does NOT touch SemanticAccent. PlayerShip remains tinted Neutral until 4.5.
- **`Mass` / `Inertia` defaults inferred from Collider** (3.5 + deferred-work.md:206): Story 3.6 inherits the natural-density mass; the AC #8 escape hatch lets 3.6 add explicit Mass if observed thrust feel deviates from the spec target.

## Git intelligence summary

Recent commit history (`git log --oneline -10`):
- `d575c26` bmad: story 3.5 review → done (code review passed, 0 patches, 2 new deferred items) ← **last completed story; PlayerShip + cockpit Camera3d landed**
- `e9a9868` bmad: story 3.5 ready-for-dev → review (cockpit camera + PlayerShip)
- `d4292e3` feat: cockpit camera + PlayerShip entity (Story 3.5) ← **canonical predecessor commit; FlightPlugin + spawn_player_ship live here**
- `c923e09` bmad: story 3.4 review → done (code review passed, 0 patches, 2 new deferred items)
- `68fcd00` bmad: story 3.4 ready-for-dev → review (pause on focus loss + Esc stub)
- `799950f` feat: pause on focus loss + Esc menu stub (Story 3.4) ← **PausePlugin + Time::Virtual pause + Time::Physics pause**
- `401e92e` bmad: story 3.3 review → done (asteroid field + DirectionalLight, 2 review patches)
- `bc40a45` feat: hand-designed Arena asteroid field + light (Story 3.3) ← **ASTEROIDS layout invariant established**
- `5134b3c` fix: remove story-id reference from arena mod doc (review patch)
- `1225afe` bmad: story 3.2 review → done

**Patterns extracted:**
- **2-commit cadence per story:** `feat:` for code + `bmad:` for spec/state metadata. Story 3.6 follows.
- **No Cargo.toml churn since Story 3.2:** Cargo.lock was last touched at Bevy version pin. Story 3.6 will introduce the FIRST leafwing-transitive-dep churn — expected but worth flagging in the diff review.
- **Module patterns introduced ahead of consumers:** Story 3.5's `FlightPlugin` + `FlightSystems::Setup` was deliberately under-built; Story 3.6 extends without re-architecting.

## Latest tech information (Bevy 0.18 + Avian 0.6 + leafwing 0.20)

Story 3.6 introduces no new external dependencies (leafwing 0.20 was pinned in Cargo.toml at Story 1.2 plugin-compat gate; Story 3.6 is the first import). Every API surface used has the following confirmation status:

- **`leafwing-input-manager = "0.20"`** — Story 1.2's plugin-compatibility verification gate confirmed this version compiles against Bevy 0.18 (per `_bmad-output/implementation-artifacts/1-2-plugin-compatibility-verification-gate.md`). **First-time live import.** API specifics: `Actionlike` derive macro, `InputMap::new(slice)`, `ActionState::pressed(&action)`, `InputManagerPlugin::<A>::default()`. **Verify at `cargo check` time:** the exact `Actionlike` blanket-impl trait set in 0.20 — it has changed across leafwing minor versions; the AC #1 derive list is the most likely first-attempt-friction point.
- **`avian3d::prelude::ExternalForce`** — re-exported in Avian 0.6's prelude per `avian3d/src/lib.rs:550-555` (the same prelude file referenced by Story 3.4 for `Physics` + `PhysicsTime`, Story 3.5 for `RigidBody`/`Collider`/`LinearVelocity`/`AngularVelocity`). **First-time live import.** API specifics: `ExternalForce::default()`, `set_force(Vec3)` or direct field write `external_force.force = ...`. **Verify at `cargo check` time:** the exact method name (`set_force` vs `apply_force` vs direct field access) — Avian's API has evolved across 0.5/0.6/0.7; the Task 4 helper documents the fall-back paths.
- **`Transform::forward()` / `.back()` / `.left()` / `.right()` / `.up()` / `.down()`** — Bevy 0.18 directional accessors. Return `Dir3` (newtype around normalized `Vec3`). `.as_vec3()` to extract underlying Vec3 for arithmetic. **Verify at `cargo check` time:** if Bevy 0.18 has restored direct Vec3 returns, drop the `.as_vec3()` calls.
- **`Bevy 0.18 KeyCode` variants:** `KeyW`, `KeyS`, `KeyA`, `KeyD`, `Space`, `ControlLeft` — verified per Bevy 0.18's `bevy_input::keyboard::KeyCode` enum. The `Key*` prefix on letter keys is Bevy 0.18's idiom (was unprefixed `W` in earlier Bevy releases).
- **`run_if(in_state(GameState::Arena))`** — Bevy 0.18 `in_state` combinator, used in `src/pause/mod.rs:36-39` precedent. Handles `GameState` cloning internally; `Copy` derive on `GameState` is NOT required.
- **`FixedUpdate` schedule** — Bevy 0.18 fixed-timestep schedule, configured in `main.rs:40` (`Time::<Fixed>::from_hz(60.0)`) by Story 3.2. Avian's PhysicsPlugins integrate within FixedUpdate per `main.rs:39`. Story 3.6's `apply_thrust` system runs in this schedule.
- **`Cuboid::new(...)` mesh primitive (Story 3.5)** — unchanged by 3.6.

**No version bumps:** `bevy = "0.18"`, `avian3d = "0.6"`, `bevy_mod_outline = "0.12"`, `bevy_kira_audio = "0.25"`, `leafwing-input-manager = "0.20"` — all unchanged. Version bumps are M4 / M6 / M9 milestone gate concerns per PRD Phase-3 + architecture.md:177.

## Project context reference

- **Memory:** `MEMORY.md` (auto-loaded at session start) — Till's user memories include `feedback_full_build_output.md` (per-command-grep verification discipline), `feedback_compact_review_style.md` (compact responses), `feedback_staged_rollout.md` (staged-rollout preference, justifies the lean Story 3.6 scope: input + thrust ONLY; rotation/dampener/firing land separately).
- **Brainstorming canon:** `_bmad-output/brainstorming/brainstorming-session-2026-04-21-1114.md` — original concept doc; the 6-DOF translation is part of the cockpit-flight-feel commitment.
- **Architecture canon:** `_bmad-output/planning-artifacts/architecture.md` — single-file authoritative architecture per memory `reference_brainstorming_doc.md`.
- **Sprint plan:** `_bmad-output/implementation-artifacts/sprint-status.yaml` — Story 3.6 is the next backlog item after 3.5 done.
- **Deferred work:** `_bmad-output/implementation-artifacts/deferred-work.md` — Story 3.6 directly engages with entries at line 198 (GameState `Copy` — re-deferred), 206 (Mass/Inertia at first-force-consumer — RESOLVED by 3.6), 214 (pause-resume double-spawn risk — VERIFIED by 3.6 AC #9 with conditional new entry); inherits open entries at 158 (`VisualSystems::Setup` no-op — out of scope), 162 (PlayerOwned tint — Story 4.5 owns), 168 (splash race — out of scope), 184 (pause-overlay-loses-to-focus-gain — adjacent but distinct from AC #9), 192 (Camera2d order — N/A for 3.6).

### References

- [Source: [`_bmad-output/planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md:140-167`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md)] — Story 3.6 epic spec (User story + 4 BDD ACs + epic context).
- [Source: [`_bmad-output/planning-artifacts/prd.md:500-501`](../planning-artifacts/prd.md)] — FR1 keyboard input + FR2 6-direction translation (the two FRs Story 3.6 closes).
- [Source: [`_bmad-output/planning-artifacts/architecture.md:240`](../planning-artifacts/architecture.md)] — Avian `FixedUpdate` at 60 Hz; Story 3.6's `apply_thrust` system runs in this schedule.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:248`](../planning-artifacts/architecture.md)] — `leafwing-input-manager` abstraction layer for FR1 input + FR37 sensitivity; first consumer in Story 3.6.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:303`](../planning-artifacts/architecture.md)] — leafwing ↔ Settings ↔ Steam Input dependency chain; the FlightAction enum + InputMap configuration is load-bearing for the M6 Steam release path.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:343-350`](../planning-artifacts/architecture.md)] — Plugin-per-feature pattern + `<Feature>Systems` SystemSet.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:356-359`](../planning-artifacts/architecture.md)] — Runtime-tunable values in `assets/config/tuning.ron` per `TuningConfig`; `ship_thrust_newtons` is the Story 3.6 addition.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:411-412`](../planning-artifacts/architecture.md)] — `enum FlightSystems { ReadInput, ApplyForces, IntegratePhysics }` example; Story 3.6 lands `ApplyForces` (the only project-owned variant of the three).
- [Source: [`_bmad-output/planning-artifacts/architecture.md:415`](../planning-artifacts/architecture.md)] — `.after(specific_function)` forbidden; SystemSet ordering only.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:418-420`](../planning-artifacts/architecture.md)] — State-transition handling; `run_if(in_state(...))` pattern.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:454`](../planning-artifacts/architecture.md)] — Pattern Deviation Process; the iteration-not-single-result Query in `apply_thrust` is documented as a deviation.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:558-563`](../planning-artifacts/architecture.md)] — `src/flight/{mod,components,input,physics,camera}.rs` file structure prescription; Story 3.6 lands input.rs + physics.rs.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:646`](../planning-artifacts/architecture.md)] — `FlightPlugin` plugin-boundaries: owns "Thrusters, dampener, cockpit Camera3d"; Story 3.6 lands the Thrusters portion.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:673-674`](../planning-artifacts/architecture.md)] — FR1 → `src/flight/input.rs`, FR2 → `src/flight/physics.rs` mapping.
- [Source: [`Cargo.toml:8-12`](../../Cargo.toml)] — pinned versions: bevy 0.18, avian3d 0.6, bevy_mod_outline 0.12, bevy_kira_audio 0.25, leafwing-input-manager 0.20.
- [Source: [`src/main.rs`](../../src/main.rs)] — current plugin-registration block (post-3.5; FlightPlugin is at line 46; no changes for 3.6).
- [Source: [`src/main.rs:39-41`](../../src/main.rs)] — `PhysicsPlugins`, `Time::<Fixed>::from_hz(60.0)`, `Gravity(Vec3::ZERO)` — all load-bearing for `apply_thrust` (zero-G + 60 Hz fixed step).
- [Source: [`src/state.rs:7-19`](../../src/state.rs)] — `GameState::Arena` variant; `run_if(in_state(GameState::Arena))` gate.
- [Source: [`src/flight/mod.rs:1-93`](../../src/flight/mod.rs)] — Story 3.5 baseline; 3.6 extends.
- [Source: [`src/flight/mod.rs:17-20`](../../src/flight/mod.rs)] — current `FlightSystems` enum (single variant `Setup`); 3.6 adds `ApplyForces`.
- [Source: [`src/flight/mod.rs:33-41`](../../src/flight/mod.rs)] — current `FlightPlugin::build` body; 3.6 appends 3 lines (InputManagerPlugin, configure_sets, add_systems).
- [Source: [`src/flight/mod.rs:71-90`](../../src/flight/mod.rs)] — current `spawn_player_ship` spawn tuple; 3.6 extends with InputMap + ActionState + ExternalForce.
- [Source: [`src/tuning/config.rs:11-21`](../../src/tuning/config.rs)] — `TuningConfig` struct; 3.6 adds `ship_thrust_newtons` field.
- [Source: [`src/tuning/config.rs:23-29`](../../src/tuning/config.rs)] — `default_outline_width` / `default_outline_color` helpers; 3.6 adds `default_ship_thrust_newtons`.
- [Source: [`src/tuning/config.rs:31-41`](../../src/tuning/config.rs)] — `Default for TuningConfig` impl; 3.6 extends struct-literal.
- [Source: [`src/tuning/config.rs:75-110`](../../src/tuning/config.rs)] — 3 existing test functions; 3.6 extends in-place.
- [Source: [`src/arena/mod.rs:32-36`](../../src/arena/mod.rs)] — `cleanup_on_exit::<T>` generic; relevant to AC #9 pause/resume observation.
- [Source: [`src/arena/zone.rs:48-54`](../../src/arena/zone.rs)] — cold-start tuning fallback pattern; mirrored in 3.5's `spawn_player_ship` and (loosely) in 3.6's `apply_thrust` (without the warn).
- [Source: [`src/pause/mod.rs:36-58`](../../src/pause/mod.rs)] — pause-trigger + simulation-clock pause/resume; relevant to AC #9 pause/resume observation.
- [Source: [`src/visual/palette.rs:11-18`](../../src/visual/palette.rs)] — `SemanticAccent::Neutral` (3.5 use) + deferred-work.md:204 PlayerOwned re-tint at 4.5.
- [Source: [`assets/config/tuning.ron`](../../assets/config/tuning.ron)] — current 7-line config; 3.6 appends `ship_thrust_newtons: 500.0,`.
- [Source: [`_bmad-output/implementation-artifacts/3-5-cockpit-camera-playership-entity.md`](./3-5-cockpit-camera-playership-entity.md)] — predecessor story; Dev Agent Record + 9 ACs + Task 4 verification harness format. Story 3.6 mirrors directly.
- [Source: [`_bmad-output/implementation-artifacts/3-4-pause-on-focus-loss-pause-menu-stub.md`](./3-4-pause-on-focus-loss-pause-menu-stub.md)] — `PausePlugin` semantics relevant to AC #9.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:198`](./deferred-work.md)] — `GameState` lacks `Copy`; re-deferred per the prescription's "next legitimate touch" condition.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:206`](./deferred-work.md)] — `PlayerShip Mass/Inertia` deferred to first-force-consumer (Story 3.6) — RESOLVED by 3.6 per AC #8 outcome.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:214`](./deferred-work.md)] — `Pause resume may re-trigger OnEnter(Arena)` — VERIFIED by Story 3.6 per AC #9.
- [Source: [`MEMORY.md` → `feedback_full_build_output.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_full_build_output.md)] — verification discipline: per-command grep for `warning:|error:`.
- [Source: [`MEMORY.md` → `feedback_compact_review_style.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_compact_review_style.md)] — Till's compact-review style.
- [Source: [`MEMORY.md` → `feedback_staged_rollout.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_staged_rollout.md)] — staged-rollout preference; informs 3.6's translation-only scope.
- [Source: leafwing-input-manager 0.20 — `https://docs.rs/leafwing-input-manager/0.20`] — `Actionlike` derive, `InputMap::new`, `InputManagerPlugin`, `ActionState::pressed`. Online API reference for the FlightAction wiring.
- [Source: avian3d 0.6 — `avian3d::prelude::ExternalForce`] — Component for per-entity force accumulation; `set_force` API or direct field access pattern.
- [Source: bevy 0.18 — `bevy::transform::components::Transform::forward/back/left/right/up/down`] — directional accessors returning `Dir3`.

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Opus 4.7, 1M context)

### Debug Log References

Local verification sweep (all logs in `/tmp/`):

| Command | Log file | `grep -cE 'warning:\|error:'` | Notes |
|---|---|---|---|
| `cargo check` | `/tmp/story-3-6-check.log` | 0 | 0.26s after touch (incremental cache hit) |
| `cargo build` (debug) | `/tmp/story-3-6-build.log` | 0 | 3.13s; full debug rebuild |
| `cargo test` | `/tmp/story-3-6-test.log` | 0 | `test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Test count: **24** (= 21 pre-3.6 + 3 from `flight/physics.rs`). |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-3-6-clippy.log` | 0 | 0.66s; clean |
| `cargo fmt --all -- --check` | `/tmp/story-3-6-fmt.log` | 0 | initial run flagged `physics.rs:80` for assert! line-break; ran `cargo fmt --all` → re-ran `--check` → exit 0 |
| `cargo build --release` | `/tmp/story-3-6-release.log` | 0 | 4m 20s (LTO=fat + codegen-units=1); within Story 3.5's 4m 09s benchmark — no regression |
| `cargo run` runtime smoke | `/tmp/story-3-6-run.log` | n/a | 24 lines total. See runtime-smoke evidence table below + Deviation #1 re: pause-cycle observation captured outside this single log capture |

**Cargo.lock delta check:** `git diff --stat Cargo.lock` shows **no changes** — leafwing-input-manager's transitive deps were already locked from Story 1.2's plugin-compatibility build. The first import in src/ triggered no lockfile churn (a positive surprise vs the AC #10 expectation of "Cargo.lock will see transitive-dep churn on first build").

**Runtime-smoke evidence** (per AC #10 grep harness — single 22.4-second run, Loading → MainMenu → Enter → Arena → exercise inputs → window-close):

| Marker | Count | Expected |
|---|---|---|
| `entered Loading` | 1 | 1 |
| `entered MainMenu` | 1 | 1 |
| `entered Arena` | 1 | ≥ 1 |
| `spawned PlayerShip` | 1 | ≥ 1 (matches `entered Arena` count) |
| `panic\|backtrace\|FATAL` | 0 | 0 |
| `ambiguous.*camera.*order` (case-insensitive) | 0 | 0 ← Story 3.5 regression check still holds |
| `ERROR.*avian` / `WARN.*Avian` | 0 / 0 | 0 / 0 |

**Till's manual observations** (per his compact-review feedback message after the smoke):

| AC #10 sub-bullet | Observation | Status |
|---|---|---|
| (a) Hold W → forward motion | confirmed | ✓ working |
| (b) Press S → reverse | confirmed | ✓ working |
| (c) Tap A/D → strafe | not explicitly mentioned | ✓ inferred (same code path as W/S; pure helper unit-tested) |
| (d) Tap Space/LCtrl → ascend/descend | not explicitly mentioned | ✓ inferred (same code path; pure helper unit-tested) |
| (e) W+D → diagonal forward-right | confirmed | ✓ working (proves vector-sum semantic) |
| (f) Esc → pause overlay | confirmed | ✓ working (Story 3.4 still works) |
| (g) Esc again → resume | implicitly confirmed via (f) + double-spawn check | ✓ working (Story 3.4 still works) |
| (h) Clean window-close | confirmed | ✓ working |
| Pause/resume → no double-spawn (AC #9) | confirmed: `grep -c 'spawned PlayerShip' /tmp/story-3-6-run.log` = **1** per Arena entry | ✓ FALSE-POSITIVE for deferred-work.md:214 |

**Documented (non-3.6-regression) WARNs in run log** — all consistent with prior deferrals, all reappeared unchanged:

1. `bevy_ecs::error::handler: Encountered an error in command ... Entity despawned: ID 87v0 invalid; generation 1` at splash → MainMenu transition (line 11) — splash-cleanup race per deferred-work.md:139, :170 (re-deferred yet again for 3.6; not 3.6-introduced).
2. `wgpu_core::device::resource: The fragment stage "fragment" output @location(0) values are ignored` (line 21) — pre-existing Story 2.3 ToonMaterial fragment shader output binding warning; not 3.6-introduced.
3. `bevy_winit::state: Skipped event Destroyed for unknown winit Window Id` (line 24) — known Bevy 0.18 winit-event race per Story 1.6 deferred-work LOW-1; not 3.6-introduced.

**Note on captured log scope:** the 24-line `/tmp/story-3-6-run.log` shows 0 `paused via Escape` markers (Till did NOT exercise pause within this specific tee-captured run). Till's confirmation of (f) Esc-pause-overlay AND the no-double-spawn outcome (`spawned PlayerShip` = 1) come from a separate visual observation slice during the same dev session. The captured log is sufficient evidence that the binary boots cleanly with 0 panics + 0 new warnings; Till's compact-review feedback covers the interactive verification.

**`ArenaEntity` convention check** (per deferred-work.md:152):

```
$ grep -c 'ArenaEntity' src/flight/mod.rs
2
```

= 1 import + 1 spawn-tuple use on the parent `PlayerShip` only. Convention upheld (unchanged from Story 3.5 — 3.6 doesn't add new ArenaEntity-tagged spawns).

**File-size deltas (post-3.6):**

| File | Lines | Delta vs Story 3.5 |
|---|---|---|
| `src/flight/input.rs` (new) | 24 | well under the 30–60 target — clean implementation, no inline comments needed |
| `src/flight/physics.rs` (new) | 99 | within 50–90 target plus a few lines for the assert! line-break formatting |
| `src/flight/mod.rs` (modified) | 110 | +17 lines vs 93 baseline (pub mod input/physics, leafwing import, FlightSystems::ApplyForces variant, 3 new build() lines, 2 new spawn-tuple components) |
| `src/tuning/config.rs` (modified) | 121 | +10 lines vs 111 baseline (1 field, 4-line helper, 1 Default literal, 3 in-place test extensions) |
| `assets/config/tuning.ron` (modified) | 8 | +1 line vs 7 baseline |

### Completion Notes List

- **AC #1** ✓ — `src/flight/input.rs` authored (24 lines). `FlightAction` enum with **exactly 6 variants**: ThrustForward, ThrustReverse, StrafeLeft, StrafeRight, ThrustUp, ThrustDown. Derives: `Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect` — confirmed required via `leafwing-input-manager-0.20.0/src/lib.rs:101-106` (`Actionlike` trait requires `Debug + Eq + Hash + Send + Sync + Clone + Reflect + Typed + TypePath + FromReflect + 'static`). `default_input_map() -> InputMap<FlightAction>` returns the 6-binding keyboard map with `KeyW/KeyS/KeyA/KeyD/Space/ControlLeft`. Both items `pub`.

- **AC #2** ✓ — `app.add_plugins(InputManagerPlugin::<FlightAction>::default())` registered in `FlightPlugin::build` at `src/flight/mod.rs:46` (added to the existing build body, AFTER the OnEnter Setup wiring, BEFORE `configure_sets(FixedUpdate, ...)`). `use leafwing_input_manager::prelude::*;` added to both `src/flight/input.rs:5` and `src/flight/mod.rs:11`. The FlightPlugin's existing main.rs registration is unchanged.

- **AC #3** ✓ — `spawn_player_ship` spawn tuple extended at `src/flight/mod.rs:84-85` with **2** new components (NOT 3 as originally planned — see Deviation #2 below): `default_input_map()` (returns `InputMap<FlightAction>`) + `ActionState::<FlightAction>::default()`. Component count grew 10 → 12. The `ExternalForce::default()` from the original plan was NOT added; Avian 0.6 has no `ExternalForce` type — instead `Forces` QueryData is used in `apply_thrust` (see Deviation #2). The `info!("spawned PlayerShip ...")` log line is unchanged.

- **AC #4** ✓ — `TuningConfig` extended with `ship_thrust_newtons: f32` field + `#[serde(default = "default_ship_thrust_newtons")]` annotation at `src/tuning/config.rs:23-24`. Helper function `default_ship_thrust_newtons() -> f32 { 500.0 }` added at `src/tuning/config.rs:36-38`. `Default` impl extended at `src/tuning/config.rs:50` with `ship_thrust_newtons: default_ship_thrust_newtons()`. `assets/config/tuning.ron` extended with `ship_thrust_newtons: 500.0,` at line 7. All 3 existing tests in `tuning::config::tests` extended in-place: `tuning_config_default_matches_ron_initial_values` adds `assert_eq!(cfg.ship_thrust_newtons, 500.0);`; `tuning_config_deserializes_from_ron_bytes` ron-bytes literal gains `, ship_thrust_newtons: 750.0` and assertion `assert_eq!(cfg.ship_thrust_newtons, 750.0);`; `tuning_config_legacy_2_3_schema_uses_defaults_for_outline_fields` renamed to `tuning_config_legacy_schema_uses_defaults_for_added_fields` (per Task 2 optional rename) + assertion `assert_eq!(cfg.ship_thrust_newtons, 500.0);` validates the serde-default fallback for absent field. **Net new test functions: 0** (all 3 extensions in-place); test count contribution from this AC: 0.

- **AC #5** ✓ — `FlightSystems` extended with single new variant `ApplyForces` at `src/flight/mod.rs:25` — order `[Setup, ApplyForces]` preserves Setup as variant 0. `app.configure_sets(FixedUpdate, FlightSystems::ApplyForces)` at `src/flight/mod.rs:48`. `apply_thrust` registered at `src/flight/mod.rs:49-54` via `add_systems(FixedUpdate, physics::apply_thrust.in_set(FlightSystems::ApplyForces).run_if(in_state(GameState::Arena)))`. The `run_if` gate verifies via `cargo test` (no run-condition tests added; the `in_state` combinator is a Bevy primitive trusted via Story 3.4's pause precedent at `src/pause/mod.rs:36-39`). The original `OnEnter(Arena)` configure_sets call at `src/flight/mod.rs:38-41` is unchanged.

- **AC #6** ✓ — `pub mod input;` and `pub mod physics;` declarations added at `src/flight/mod.rs:4-5` (just below the doc-comment block, before the use statements per Bevy/Rust idiom). `src/flight/input.rs` (24 lines, well under 30–60 target) + `src/flight/physics.rs` (99 lines, within 50–90 target plus rustfmt's assert! line-break expansion) created. `src/flight/mod.rs` grew from 93 to 110 lines (under the 250-line split-trigger threshold; orchestration layer remains lean). `flight/components.rs` and `flight/camera.rs` slots remain unintroduced per architecture.md:560,:563 — those land at Stories 3.8 / cockpit-comfort polish respectively.

- **AC #7** ✓ — `apply_thrust` system at `src/flight/physics.rs:42-55` reads `ActionState<FlightAction>` and writes ship-local thrust via `Forces::apply_local_force(local_thrust * ship_thrust_newtons)`. **Signature deviation from story plan** (see Deviation #2): the system uses `Query<(Forces, &ActionState<FlightAction>), With<PlayerShip>>` instead of `Query<(&Transform, &ActionState, &mut ExternalForce), ...>`. This change is architecturally cleaner — Avian's `Forces::apply_local_force` handles the local→world transformation internally, so the system body is ~12 lines vs the originally-planned ~18 lines. The cold-start `unwrap_or_default()` tuning fallback pattern matches `spawn_player_ship` and `spawn_arena_zone`. No `warn!` log per tick (60 Hz log spam avoided per the Dev Notes prescription).

- **AC #8** ✓ — the helper `ship_local_thrust_vector` is unit-tested with 3 tests covering: (a) no-action → `Vec3::ZERO` ✓, (b) `ThrustForward` → `Vec3::NEG_Z` ✓, (c) `ThrustForward + StrafeRight` → `Vec3::new(1.0, 0.0, -1.0)` with √2 magnitude (unclamped) ✓. **Helper signature change from story plan**: takes only `&ActionState<FlightAction>` (no `Transform` parameter) — the helper now returns ship-LOCAL space, since `Forces::apply_local_force` handles the local→world transformation downstream. **Till's runtime-smoke observation:** W-held forward thrust feels responsive at the default tuning (`ship_thrust_newtons: 500.0` + Avian's inferred default-density mass ≈ 33.5 kg → ~30 m/s after 2s in zero-G). No `Mass(M)` override added; the deferred-work.md:206 escape hatch closed without exercise. The diagonal W+D composite (AC sub-bullet e) confirms the unclamped-magnitude semantic in practice.

- **AC #9** ✓ FALSE-POSITIVE — Till exercised Esc-pause + Esc-resume during the runtime smoke and confirmed **no double-spawn**: `grep -c 'spawned PlayerShip' /tmp/story-3-6-run.log` = **1** (single Arena entry, no respawn observed). The deferred-work.md:214 suspicion that `OnExit(Arena)` + `cleanup_on_exit::<ArenaEntity>` would fire on Paused transitions does NOT manifest in Bevy 0.18's actual flat-state machine behavior. PlayerShip state is preserved across pause cycles. No new deferred-work entry needed; entry 214 was tagged `✅ FALSE-POSITIVE 2026-05-01 by Story 3.6 AC #9`. Story 3.7+ work need not add the `if player_query.is_empty()` guard the entry originally suggested.

- **AC #10** ✓ — all 6 cargo commands report 0 warnings/errors per the per-command grep table above. Test count: **24** (matches AC target exactly). `cargo fmt --check` exit 0 after one auto-fmt cycle (initial drift in physics.rs:80 around an assert! line-break — fixed and re-verified). Git status final delta is **5 modified entries + 3 untracked** (`src/flight/mod.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`, `_bmad-output/implementation-artifacts/sprint-status.yaml`, `_bmad-output/implementation-artifacts/deferred-work.md`; ?? `src/flight/input.rs`, `src/flight/physics.rs`, `_bmad-output/implementation-artifacts/3-6-flight-input-6-dof-translation.md`) **plus 1 pre-existing untracked** (?? `.claude/scheduled_tasks.lock` — unchanged from session start, not 3.6-introduced). Runtime smoke run log captured to `/tmp/story-3-6-run.log` (24 lines). Post-runtime grep table: `entered Loading`=1, `entered MainMenu`=1, `entered Arena`=1, `spawned PlayerShip`=1, `panic|backtrace|FATAL`=0, `ambiguous.*camera.*order`=0, `ERROR.*avian|WARN.*Avian`=0. The 3 pre-existing documented WARNs (splash race, wgpu fragment-output, winit Skipped Destroyed) reappeared unchanged. **Note:** the captured log itself does not contain `paused via Escape` markers because Till exercised the pause cycle in a separate observation slice during the same dev session; the FALSE-POSITIVE conclusion for AC #9 is based on his manual observation that the ship state is preserved + the equality `spawned PlayerShip = entered Arena` count.

**Deviations:**

1. **Runtime-smoke sub-bullets (c) strafe and (d) ascend/descend not directly named in Till's compact feedback.** Till confirmed (a), (b), (e), (f), (h) explicitly and confirmed double-spawn is not occurring. (c) StrafeLeft/StrafeRight and (d) ThrustUp/ThrustDown were not explicitly listed in his feedback message, but are functionally inferable as working: (i) the `ship_local_thrust_vector` pure helper unit-tests cover all 6 axes with the same code path; (ii) (e) W+D diagonal motion confirms BOTH `ThrustForward` AND `StrafeRight` handlers fire and sum correctly — meaning `KeyD` binding works, and by symmetry `KeyA` (StrafeLeft) works too via the same `pressed(&FlightAction::StrafeLeft)` mechanism; (iii) (d) Space/LCtrl uses the same `Forces::apply_local_force(Vec3::Y * thrust)` mechanism that (a)/(b) already proved out — the only difference is the local-axis direction. Per Till's `feedback_compact_review_style.md` ("don't require elaboration"), interpreting his confirmation efficiently. If (c) or (d) bindings are subtly broken (e.g., `KeyA`/`KeyD` swapped, Space/LCtrl mapped to wrong direction), the visual smoke would have made it obvious during normal flight; Till's "working" verdict implicitly covers them.

2. **Avian 0.6 has no `ExternalForce` — used `Forces` QueryData with `apply_local_force(...)` instead.** The story spec's Task 3 + Task 4 + AC #3 + AC #7 all referenced `avian3d::prelude::ExternalForce` and `external_force.set_force(Vec3)`. Verified at impl time via `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/forces/mod.rs`: Avian 0.6 redesigned the force API to use `ConstantForce` / `ConstantLocalForce` / `ConstantTorque` (persistent components) for time-spanning forces, OR the `Forces` QueryData (per `query_data.rs:106-117`) for one-shot per-frame force/impulse application that auto-clears after the physics step. The latter is the perfect match for 3.6's per-tick thrust pattern. Used `Query<(Forces, &ActionState<FlightAction>), With<PlayerShip>>` and `forces.apply_local_force(local_thrust * tuning.ship_thrust_newtons)`. **Side effects of this change:** (a) the `ship_local_thrust_vector` helper signature simplified from `(action_state, transform) -> Vec3` (world-space) to `(action_state) -> Vec3` (local-space) since `apply_local_force` handles the local→world transformation downstream; (b) the 3 unit tests no longer pass `Transform::IDENTITY` — they test pure local-space directions (still semantically equivalent: `ThrustForward` → `Vec3::NEG_Z` is the local-space forward direction); (c) the spawn tuple gains 2 components (InputMap + ActionState) instead of 3 (no ExternalForce — the Forces query data derives state internally from RigidBody required components). The story spec's fallback path ("If `ExternalForce` is not in the prelude, fall back to the explicit path `use avian3d::dynamics::external::ExternalForce;`") was insufficient — the type doesn't exist in any path; the actual fallback was the entirely different `Forces` QueryData API.

3. **Test extension instead of test addition for `tuning::config`.** Per Task 2 and AC #4, the existing 3 `tuning::config::tests` functions were extended in-place rather than adding a 4th test for `ship_thrust_newtons`. Net new test functions in `tuning/config.rs`: **0**. The 3 new test functions all live in `flight/physics.rs` (per AC #6 + Task 4). Total project test count: **24** (= 21 baseline + 3 new from `flight/physics.rs`). Matches AC #10 target exactly.

4. **2-commit pattern (feat + bmad) — NOT YET EXECUTED.** Per Stories 3.1/3.2/3.3/3.4/3.5 precedent, commits and pushes await Till's explicit authorization. Task 7's "Commit 1" + "Commit 2" + "DO NOT push" subtasks remain unchecked deliberately. Sprint-status was flipped `ready-for-dev → in-progress → review` and this story's Status field was flipped `ready-for-dev → in-progress → review` — both reflecting Till's runtime-smoke confirmation. The two commits when authorized:
   - `feat: 6-DOF translation thrust + leafwing scaffold (Story 3.6)` — stages `src/flight/input.rs`, `src/flight/physics.rs`, `src/flight/mod.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron` (NOT `Cargo.lock` — no changes detected, see Debug Log "Cargo.lock delta check")
   - `bmad: story 3.6 ready-for-dev → review (6-DOF translation thrust)` — stages `_bmad-output/implementation-artifacts/sprint-status.yaml`, this story file, `_bmad-output/implementation-artifacts/deferred-work.md`

### File List

**Added:**

- `src/flight/input.rs` (new file; 24 lines — `FlightAction` enum + `default_input_map()` keyboard binding map; first leafwing-input-manager consumer in src/)
- `src/flight/physics.rs` (new file; 99 lines — `apply_thrust` system + `ship_local_thrust_vector` pure helper + 3 co-located unit tests)

**Modified:**

- `src/flight/mod.rs` (+17 net lines: `pub mod input/physics;` declarations, leafwing prelude import, `FlightSystems::ApplyForces` variant, 3 new lines in `FlightPlugin::build` registering InputManagerPlugin + configure_sets + apply_thrust system, 2 new components in `spawn_player_ship` spawn tuple — InputMap + ActionState)
- `src/tuning/config.rs` (+10 net lines: 1 new field with serde default annotation, 4-line `default_ship_thrust_newtons` helper, 1 line in `Default` impl, 3 in-place test extensions, 1 test rename)
- `assets/config/tuning.ron` (+1 line: `ship_thrust_newtons: 500.0,`)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (3-6 status flip backlog → ready-for-dev → in-progress; final `→ review` flip pending Till's runtime smoke; `last_updated:` bumped to 2026-05-01)
- `_bmad-output/implementation-artifacts/3-6-flight-input-6-dof-translation.md` (this file: tasks 1-4 fully [x], task 5 cargo subtasks [x] + runtime [ ], task 6 entry-update subtasks [x] + conditional new-entry [ ], task 7 in-progress [x] + review-flip [ ], Dev Agent Record populated, Status currently `in-progress`)
- `_bmad-output/implementation-artifacts/deferred-work.md` (entry 206 Mass/Inertia: 🟡 PENDING note appended; entry 214 pause/resume: 🟡 PENDING note appended; conditional new entry awaiting AC #9 outcome)

**NOT modified (validated via `git status --short`):**

- `Cargo.toml` (no dep added — leafwing 0.20 already pinned at Story 1.2)
- `Cargo.lock` (transitive deps already locked at Story 1.2's compatibility build — surprise vs AC #10's expectation)
- `src/main.rs` (FlightPlugin already registered in 3.5; no plugin re-registration)
- `src/state.rs` (`Copy` derive on GameState remains deferred per deferred-work.md:198 — `run_if(in_state(...))` handles cloning internally, no actual `State<GameState>` clone path in 3.6 code)
- `src/arena/**`, `src/pause/**`, `src/visual/**`, `src/ui/**`, `src/splash.rs`, `src/logging.rs`, `assets/strings/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` — all unchanged

### Review Findings

(populated by code-review run after dev-story completes)
