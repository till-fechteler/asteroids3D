# Story 3.8: Inertial Dampener Toggle

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player flying the Arena cockpit ship,
I want a toggleable inertial dampener that bleeds my linear and angular velocity toward zero when active,
So that I can modulate between Newtonian drift (Stories 3.6/3.7 baseline) and arcade-tight stop-on-release control per FR5 — without which 6-DOF flight feels like ice-skating and aiming at Story 3.9's projectile targets becomes punishing.

## Acceptance Criteria

1. **Given** `FlightAction` (extended in Stories 3.6/3.7 with 10 variants — 6 thrust + 2 axis pitch/yaw + 2 roll buttons) is the canonical flight-input enum
   **When** Story 3.8 extends it
   **Then** one new variant is appended at the end of the enum: `ToggleDampener` (final variant order: `ThrustForward, ThrustReverse, StrafeLeft, StrafeRight, ThrustUp, ThrustDown, Pitch, Yaw, RollLeft, RollRight, ToggleDampener`)
   **And** `ToggleDampener` is a default-buttonlike variant — NO `#[actionlike(...)]` attribute (it shares `InputControlKind::Button` default with the 8 thrust/roll buttons)
   **And** `default_input_map()` is extended with one new binding: `KeyX → ToggleDampener` (insert into the buttonlike `InputMap::new([...])` slice BEFORE the trailing `.with_axis(...)` chain — keeping all button bindings inside the `new(...)` slice and all axis bindings in the chained tail). Final binding count grows from 10 → 11 (9 buttonlike + 2 axislike)

2. **Given** Story 3.7 dev notes explicitly punt the `src/flight/components.rs` slot to Story 3.8 (per architecture.md:560 — `Thrusters, InertialDampener, Boost, TractorEmitter` live there) and `DampenerState` is the first such marker/state component
   **When** Story 3.8 introduces `DampenerState`
   **Then** a new file `src/flight/components.rs` is created containing exactly:
   ```rust
   //! Marker / state components for FlightPlugin entities.
   //! Initial occupant is DampenerState (FR5); future stories add Boost (FR6),
   //! Thrusters (FR2 visual marker), and TractorEmitter (FR7) per architecture.md:560.

   use bevy::prelude::*;

   /// Toggleable inertial-dampener state on the PlayerShip entity. When `active`,
   /// `apply_dampener` (in `flight/physics.rs`) bleeds linear + angular velocity
   /// toward zero each FixedUpdate tick. Default `active = true` per Epic 3 spec.
   #[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
   pub struct DampenerState {
       pub active: bool,
   }

   impl Default for DampenerState {
       fn default() -> Self {
           Self { active: true }
       }
   }

   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn dampener_state_default_is_active() {
           assert!(DampenerState::default().active);
       }
   }
   ```
   **And** `src/flight/mod.rs` declares the module via `pub mod components;` adjacent to the existing `pub mod input;` and `pub mod physics;` declarations (alphabetical order: components → input → physics)
   **And** `DampenerState::default()` is appended to the `spawn_player_ship` component-tuple in `src/flight/mod.rs` AFTER `ActionState::<FlightAction>::default()` (insert-at-end ordering matches Story 3.6/3.7 precedent for spawn-tuple growth)

3. **Given** `TuningConfig` (`src/tuning/config.rs`) is the project's single canonical gameplay-tuning struct (extended by Stories 2.3/2.4/3.6/3.7)
   **When** Story 3.8 extends it
   **Then** two new fields are added in this order, AFTER the existing `ship_torque_nm` field: `pub dampener_linear_strength: f32` (default `2.0`) and `pub dampener_angular_strength: f32` (default `3.0`)
   **And** both fields use the per-field `#[serde(default = "default_…")]` pattern matching the precedent set by `outline_width` / `ship_thrust_newtons` / `mouse_sensitivity` / `ship_torque_nm` (forward-compat — preserves deserialization of pre-3.8 tuning.ron snapshots)
   **And** two new top-level helpers are added alongside the existing default helpers: `fn default_dampener_linear_strength() -> f32 { 2.0 }` and `fn default_dampener_angular_strength() -> f32 { 3.0 }`
   **And** `impl Default for TuningConfig` includes both new fields in its struct-literal (in the same order as the struct fields)
   **And** `assets/config/tuning.ron` gains two new lines after `ship_torque_nm: 80.0,`: `dampener_linear_strength: 2.0,` and `dampener_angular_strength: 3.0,` (insert-at-end ordering matches Story 2.4/3.6/3.7 precedent; trailing commas correct per RON-0.8 convention)
   **And** the existing 3 tests in `tuning::config::tests` are extended in-place — NO new test functions added:
   - `tuning_config_default_matches_ron_initial_values` gains two assertions: `assert_eq!(cfg.dampener_linear_strength, 2.0);` and `assert_eq!(cfg.dampener_angular_strength, 3.0);`
   - `tuning_config_deserializes_from_ron_bytes` ron-bytes literal gains `, dampener_linear_strength: 4.0, dampener_angular_strength: 6.0` and assertions `assert_eq!(cfg.dampener_linear_strength, 4.0);` + `assert_eq!(cfg.dampener_angular_strength, 6.0);` (non-default values exercise the per-field deserializer; symmetric with the existing `ship_torque_nm: 120.0` non-default literal from 3.7)
   - `tuning_config_legacy_schema_uses_defaults_for_added_fields` ron-bytes literal is unchanged (the absent fields exercise the serde-default fallback) and gains assertions `assert_eq!(cfg.dampener_linear_strength, 2.0);` + `assert_eq!(cfg.dampener_angular_strength, 3.0);`

4. **Given** Avian's `Forces` query data exposes `linear_velocity()` / `angular_velocity()` reads (world-space, per `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/forces/query_data.rs:204-214`) AND `apply_linear_acceleration(Vec3)` / `apply_angular_acceleration(Vec3)` writes (per `query_data.rs:482-487` and `:539-544`) that bypass the mass/inertia divisor used by `apply_force` / `apply_torque`
   **When** Story 3.8 implements the dampener-application system
   **Then** the system signature is:
   ```rust
   pub fn apply_dampener(
       tuning_assets: Res<Assets<TuningConfig>>,
       tuning_handle: Res<TuningHandle>,
       mut ships: Query<(Forces, &DampenerState), With<PlayerShip>>,
   )
   ```
   **And** the system body (a) extracts `dampener_linear_strength` + `dampener_angular_strength` via the same cold-start fallback `tuning_assets.get(tuning_handle.0.id()).cloned().unwrap_or_default()` pattern as `apply_thrust` / `apply_torque`; (b) iterates the query (one match expected — same one-or-zero pattern as `apply_thrust` / `apply_torque`); (c) early-returns per-iteration if `!state.active`; (d) reads `forces.linear_velocity()` and `forces.angular_velocity()` (both world-space); (e) computes and applies via `forces.apply_linear_acceleration(linear_accel)` and `forces.apply_angular_acceleration(angular_accel)` where the accelerations come from the `dampener_acceleration` pure helper (AC #5)
   **And** **NO** `warn!` or `info!` per-tick logs (60 Hz spam — same discipline as `apply_thrust` / `apply_torque`)
   **And** `apply_linear_acceleration` / `apply_angular_acceleration` are no-ops for `Vec3::ZERO` per their internal `if accel != ZERO` early-return — quiet ticks (zero velocity OR `state.active == false`) produce no acceleration accumulation and do not wake sleeping bodies

5. **Given** `dampener_acceleration` is the pure-logic helper symmetric to `ship_local_thrust_vector` (Story 3.6) and `ship_local_torque_vector` (Story 3.7) — first-class unit-test target per architecture.md:353
   **When** authored in `src/flight/physics.rs` alongside the existing thrust + torque helpers
   **Then** the signature is:
   ```rust
   pub fn dampener_acceleration(
       state: DampenerState,
       linear_velocity: Vec3,
       angular_velocity: Vec3,
       linear_strength: f32,
       angular_strength: f32,
   ) -> (Vec3, Vec3)
   ```
   **And** the body returns `(Vec3::ZERO, Vec3::ZERO)` if `!state.active` — early-return covers the dampener-off case before any arithmetic
   **And** when `state.active`, returns `(-linear_velocity * linear_strength, -angular_velocity * angular_strength)` — both contributions independent (linear vs. angular axes do not couple)
   **And** the helper performs **NO** clamping or NaN guarding (consistent with the unclamped `ship_local_thrust_vector` / `ship_local_torque_vector` precedent — input-hardening lives in TuningConfig deserialization, deferred per deferred-work.md:222 + :228)
   **And** **PATTERN DEVIATION (per architecture.md:454):** AC literal text says "ExternalForce equal to `-linear_velocity * dampener_linear_strength * mass`" and "ExternalTorque equal to `-angular_velocity * dampener_angular_strength * moment_of_inertia`". The implementation uses `apply_linear_acceleration(-v * strength)` and `apply_angular_acceleration(-ω * strength)` instead — mathematically identical (`accel = force / mass = (-v * strength * mass) / mass = -v * strength`) but avoids redundant mass/inertia tensor multiplication. Document deviation as a top-of-system comment: `// PATTERN DEVIATION: Avian's apply_*_acceleration bypasses the mass/inertia divisor; mathematically equivalent to applying force = -velocity * strength * mass per the AC, but skips a redundant query of ComputedMass / ComputedAngularInertia.`

6. **Given** the `ToggleDampener` action fires once per X-key press (button input, edge-triggered semantics)
   **When** Story 3.8 wires the toggle handler
   **Then** a new system `toggle_dampener` is authored in `src/flight/physics.rs` BELOW the existing systems with signature:
   ```rust
   pub fn toggle_dampener(
       mut ships: Query<(&ActionState<FlightAction>, &mut DampenerState), With<PlayerShip>>,
   )
   ```
   **And** the system body iterates the query and, when `action_state.just_pressed(&FlightAction::ToggleDampener)`, flips `state.active = !state.active` AND emits exactly one `info!("dampener {}", if state.active { "engaged" } else { "disengaged" });` log per toggle (per AC #2 of the Epic spec: "an info! log records the new state for dev feedback"). The log is acceptable here because `just_pressed` fires at most once per X-press (~once per second under realistic player input — orders of magnitude below the 60 Hz per-tick anti-pattern)
   **And** `toggle_dampener` is registered in `Update` (NOT `FixedUpdate`) with `.run_if(in_state(GameState::Arena))` — input-handling cadence pattern matches `pause::toggle_pause_on_escape` at `src/pause/mod.rs:38-41`. NO SystemSet placement (Update-phase input handlers are loose per pause-plugin precedent)
   **And** the `apply_dampener` system is registered in `FixedUpdate` inside the EXISTING `FlightSystems::ApplyForces` set, BUNDLED into the existing 2-system tuple via:
   ```rust
   (physics::apply_thrust, physics::apply_torque, physics::apply_dampener)
       .in_set(FlightSystems::ApplyForces)
       .run_if(in_state(GameState::Arena))
   ```
   This grows the tuple from 2 → 3 systems sharing the set + run_if (Bevy 0.18 idiom; matches `src/pause/mod.rs:34-42` 3-system tuple precedent)
   **And** **NO** new `FlightSystems` enum variants are added (per Story 3.6 AC #5 / 3.7 AC #3 rationale: "linear and angular forces sum independently in Avian — no inter-set ordering needed"; dampener is just another force/torque contributor in the same accumulation phase)
   **And** **NO** changes to `src/main.rs` (FlightPlugin already registered)

7. **Given** the dampener convergence math: `dv/dt = -strength * v` integrates to `v(t) = v(0) * exp(-strength * t)`
   **When** the dev runs the runtime smoke (Task 5) and tests dampener engagement
   **Then** at default `dampener_linear_strength = 2.0` after 3 seconds: `exp(-2.0 * 3) ≈ 0.0025` → linear velocity is ≈ 0.25% of initial → well within the AC's "5% of zero" tolerance
   **And** at default `dampener_angular_strength = 3.0` after 3 seconds: `exp(-3.0 * 3) ≈ 0.00012` → angular velocity is ≈ 0.012% of initial → well within tolerance
   **And** if observed convergence is wildly off (e.g., overshoots zero and oscillates, OR converges in < 1 s feeling overly snappy, OR converges in > 8 s feeling sludgy), the dev exercises the "tune `dampener_*_strength` in tuning.ron" escape hatch — values 1.0–5.0 (linear) and 1.5–6.0 (angular) are reasonable to try; document the chosen value + reasoning in Dev Agent Record

8. **Given** `dampener_acceleration` is the only pure-logic surface added in 3.8 and the runtime smoke (Task 5) is the de-facto integration test for `apply_dampener` + `toggle_dampener` (per Story 3.6 AC #8 / 3.7 AC #8 precedent: integration tests deferred post-M3 per architecture.md:354)
   **When** the helper is unit-tested
   **Then** `flight/physics.rs` gains 5 new co-located test functions in the existing `#[cfg(test)] mod tests` block (alongside the 3 thrust + 6 rotation tests):
   - `dampener_inactive_returns_zero_acceleration` — `(DampenerState{active: false}, lin=(2,0,0), ang=(0,3,0), 2.0, 3.0)` → `(Vec3::ZERO, Vec3::ZERO)` (verifies the early-return; non-zero velocities prove the gate isn't accidentally bypassed)
   - `dampener_active_zero_velocity_returns_zero_acceleration` — `(DampenerState{active: true}, lin=ZERO, ang=ZERO, 2.0, 3.0)` → `(Vec3::ZERO, Vec3::ZERO)` (verifies the active-but-quiet case; co-equal with the no-op early-return inside `apply_linear_acceleration`/`apply_angular_acceleration`)
   - `dampener_active_linear_velocity_returns_negative_proportional_acceleration` — `(active, lin=(2,0,0), ang=ZERO, 2.0, 3.0)` → `(Vec3::new(-4.0, 0.0, 0.0), Vec3::ZERO)` (verifies linear-only contribution + sign + scalar product)
   - `dampener_active_angular_velocity_returns_negative_proportional_acceleration` — `(active, lin=ZERO, ang=(0,3,0), 2.0, 3.0)` → `(Vec3::ZERO, Vec3::new(0.0, -9.0, 0.0))` (verifies angular-only contribution + sign + scalar product)
   - `dampener_combines_linear_and_angular_independently` — `(active, lin=(1,2,3), ang=(4,5,6), 2.0, 3.0)` → `(Vec3::new(-2.0, -4.0, -6.0), Vec3::new(-12.0, -15.0, -18.0))` (verifies the two contributions sum vector-wise and that linear strength applies only to linear, angular only to angular)
   **And** `flight/components.rs` gains 1 new test (per AC #2): `dampener_state_default_is_active` — `assert!(DampenerState::default().active)` — guards the default-active invariant against accidental future flips
   **And** the existing 3 thrust + 6 rotation tests in `flight/physics.rs` are unchanged
   **And** Story 3.8 adds **6 net new test functions** (5 in `flight/physics.rs` + 1 in `flight/components.rs`) — net post-3.8 test count: **36** (= 30 from end of 3.7 + 5 dampener tests + 1 components test). AC #11 enforces N = 36 at verification time

9. **Given** the toggle-flip semantics in AC #2 of the Epic spec ("`DampenerState.active` flips" + "an `info!` log records the new state")
   **When** the dev runs the runtime smoke (Task 5)
   **Then** the dev verifies that pressing X exactly once flips the state from active → inactive (smoke-confirmed via the `info!("dampener disengaged")` log line in `/tmp/story-3-8-run.log`); pressing X again flips back to active (`info!("dampener engaged")`)
   **And** the dev verifies that holding X (key auto-repeat) does NOT spam toggles — `just_pressed` fires only on press-edge, not while held; X-held should produce exactly 1 toggle per logical press, NOT N toggles where N is the OS auto-repeat count (this is leafwing's `just_pressed` contract — verify via log-line count of `dampener (engaged|disengaged)` matches manual press count)
   **And** the dev verifies that the dampener-off case behaves IDENTICALLY to pre-3.8 Newtonian drift: thrust release → ship continues to drift (Story 3.6 baseline); rotation release → ship continues to rotate (Story 3.7 baseline). The dampener-off case is the regression check — Story 3.8 must NOT alter pre-3.8 behavior when `state.active = false`

10. **Given** the dampener convergence-toward-zero behavior interacts with active thrust/rotation input (a thrust-against-dampener tug-of-war should produce a bounded terminal velocity, not pin to zero)
    **When** the dev runs the runtime smoke (Task 5) with dampener active AND held thrust
    **Then** the dev verifies the tug-of-war terminal-velocity model: with `apply_thrust` adding `+ship_thrust_newtons / mass` per second (acceleration) AND `apply_dampener` adding `-strength * v` per second, the steady state is `v_terminal = (ship_thrust_newtons / mass) / strength`. With defaults (`ship_thrust_newtons = 500.0`, mass ≈ 33.5 kg from `Collider::sphere(2.0)`, `strength = 2.0`): `v_terminal ≈ (500/33.5) / 2.0 ≈ 7.5 m/s` forward. Visually: holding W with dampener-on produces a *slow forward drift* that never accelerates beyond ~7.5 m/s, vs. the dampener-off case where W produces unbounded acceleration toward Story 3.6's `ship_thrust_newtons / mass * 2 ≈ 30 m/s` after 2 seconds
    **And** the dev verifies the rotation tug-of-war: holding Q with dampener-on produces a slow constant-angular-velocity roll (terminal `ω ≈ ship_torque_nm / inertia_tensor / angular_strength ≈ 80.0 / 53.6 / 3.0 ≈ 0.5 rad/s ≈ 28°/s`) that never accelerates further, vs. dampener-off case from Story 3.7 (`ω → 1.49 rad/s ≈ 85°/s` after 1s)
    **And** the dev verifies that release of input + dampener-on produces velocities approaching zero per AC #4 of the Epic ("3 seconds elapse → velocities within 5% of zero"): convergence math from AC #7 confirms 0.25% (linear) / 0.012% (angular) at defaults — well inside the 5% bound

11. **Given** Story 3.7's pause-cycle preservation invariant (AC #10) — angular velocity persists across Esc/focus-loss pauses
    **When** the dev runs the runtime smoke (Task 5) and tests dampener-state preservation across pause cycles
    **Then** the dev verifies that `DampenerState.active` persists across Arena → Paused → Arena cycles (the component lives on the persistent PlayerShip entity; pause does not despawn the ship per Story 3.6/3.7 false-positive ✅; therefore the dampener state survives by construction)
    **And** the dev verifies that velocities decaying mid-pause-cycle resume their decay correctly: pause WITH dampener active and non-zero velocity → resume → decay continues from the paused velocity (because Story 3.4's `pause_simulation_clocks` halts `Time<Physics>`, so `apply_dampener` produces zero net delta during the pause; on resume, decay restarts from the same velocity it had at pause-time)
    **And** the dev verifies the toggle-during-pause case: press X while paused → leafwing's `just_pressed` is consumed in the next Update tick (pause does NOT halt `Update`, only `Time<Virtual>` and `Time<Physics>`). Per the AC #6 design: `toggle_dampener.run_if(in_state(GameState::Arena))` — the run_if gate suppresses the toggle while paused, so X-press during Paused is a no-op (this is the intended behavior; toggling mid-pause would be UX-confusing). Smoke-verify: pause via Esc, mash X 5×, resume → expect dampener state unchanged (zero `dampener (engaged|disengaged)` log lines emitted during the paused window)

12. **Given** the post-3.7 source baseline (test count = 30 per `cargo test` 2026-05-04 measurement; `cargo build --release` 0 warnings; `src/flight/mod.rs` = 142 lines; `src/flight/input.rs` = 35 lines; `src/flight/physics.rs` = 248 lines; `src/tuning/config.rs` = 141 lines; `assets/config/tuning.ron` = 10 lines; NO `src/flight/components.rs` exists)
    **When** Story 3.8 verification runs locally (per `feedback_full_build_output.md` discipline — exit-0 + tail is NOT proof; grep for `warning:|error:` per command, capture each to `/tmp/story-3-8-<command>.log`)
    **Then** **all six** of `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --release` produce **0** lines matching `grep -cE 'warning:|error:'` per their respective full output logs
    **And** `cargo test` summary line reads `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N = 36** (= 30 baseline + 5 new `dampener_acceleration` tests in `flight/physics.rs` + 1 new `dampener_state_default_is_active` test in `flight/components.rs`; the +0 deltas in `tuning/config.rs` per AC #3 are expected)
    **And** the runtime smoke (Task 5) verifies all of: (a) X-press → dampener engages/disengages with one `info!` log per toggle; (b) X-held → exactly 1 toggle per logical press (no auto-repeat spam); (c) dampener-on + W release → ship decelerates to ≈ 0 within 3 s; (d) dampener-on + Q release → ship angular-decelerates to ≈ 0 within 3 s; (e) dampener-on + W held → terminal velocity ~7.5 m/s (NOT unbounded acceleration); (f) dampener-on + Q held → terminal angular velocity ~0.5 rad/s (NOT unbounded); (g) dampener-off → Story 3.6/3.7 baseline behavior identical (regression check); (h) Esc-pause + dampener-on mid-decay → resume → decay continues from paused velocity (no jump); (i) X-press during Paused → no-op (toggle gated by `in_state(Arena)`); (j) Cmd-Tab pause → resume → dampener state unchanged; (k) clean window-close (no panic on shutdown)
    **And** `/tmp/story-3-8-run.log` contains: 1 occurrence of `entered Loading`, 1 of `entered MainMenu`, ≥ 1 of `entered Arena`, ≥ 1 of `spawned PlayerShip` (matches `entered Arena` count per Story 3.6 AC #9 false-positive), ≥ 2 of `dampener (engaged|disengaged)` (at least one toggle exercised), 0 of `panic|backtrace|FATAL`, 0 of `ambiguous.*camera.*order` (Story 3.5 regression check), 0 of `ERROR.*avian|WARN.*Avian`
    **And** `git status --short` final set is **exactly**: `M src/flight/input.rs` (M — extended for `ToggleDampener` variant + `KeyX` binding), `M src/flight/mod.rs` (M — `pub mod components;` declaration + `DampenerState::default()` in spawn tuple + `apply_dampener` added to ApplyForces tuple + `toggle_dampener` registered in Update), `M src/flight/physics.rs` (M — added `dampener_acceleration` helper + `apply_dampener` system + `toggle_dampener` system + 5 new tests), `?? src/flight/components.rs` (?? — NEW FILE per AC #2: DampenerState component + Default + 1 test), `M src/tuning/config.rs` (M — 2 new fields + 2 new helpers + Default extended + 3 test extensions), `M assets/config/tuning.ron` (M — 2 new lines), `M _bmad-output/implementation-artifacts/sprint-status.yaml` (M — bookkeeping), `?? _bmad-output/implementation-artifacts/3-8-inertial-dampener-toggle.md` (?? — NEW FILE: this story spec; ?? at story-creation time, becomes M after dev flips Status), and `M _bmad-output/implementation-artifacts/deferred-work.md` ONLY IF a new entry surfaces during impl (none anticipated; AC #10 tug-of-war terminal-velocity observation is a Dev Agent Record entry without a new deferred-work line); **NO** entries under `Cargo.toml` (no dep added — DampenerState uses `bevy::prelude::Component`; Avian's `Forces`/acceleration APIs already in scope from 3.6/3.7), `Cargo.lock` (no transitive-dep churn — leafwing/Avian/Bevy/outline/kira all unchanged), `src/main.rs` (no plugin re-registration), `src/state.rs` (per deferred-work.md:198, `Copy` derive remains deferred — `run_if(in_state(...))` handles cloning internally), `src/arena/**`, `src/pause/**`, `src/visual/**`, `src/ui/**`, `src/splash.rs`, `src/logging.rs`, `assets/strings/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`

## Tasks / Subtasks

- [x] **Task 1: Create `src/flight/components.rs` — DampenerState marker component + Default + 1 test** (AC: #2)
  - [x] Create new file `src/flight/components.rs` (verify file does not exist beforehand: `ls src/flight/components.rs` → expected `No such file`).
  - [x] Author the file content per AC #2 verbatim: module doc-comment generalized to "Marker / state components for FlightPlugin entities" (NO story-id reference per anti-pattern #1); `use bevy::prelude::*;`; `DampenerState { pub active: bool }` component with `#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]`; `impl Default` returns `Self { active: true }`; co-located `#[cfg(test)] mod tests` with one `#[test] fn dampener_state_default_is_active() { assert!(DampenerState::default().active); }`.
  - [x] In `src/flight/mod.rs`, add `pub mod components;` to the existing module declarations (alphabetical placement: BEFORE `pub mod input;` and `pub mod physics;` — final order: `components`, `input`, `physics`).
  - [x] **Verify post-edit:** `cargo check` produces 0 warnings/errors. The component derive must compile cleanly; `Component` is in `bevy::prelude::*`. `DampenerState` is not yet referenced by any other code in this task — that wiring lands in Tasks 3 and 4. **Note:** `cargo check` reports `dead_code` warning on `DampenerState` (expected — first consumer lands in Task 4 `apply_dampener` query and Task 5 spawn tuple). Test `flight::components::tests::dampener_state_default_is_active` passes (1 test, 30 filtered out — total 31 = 30 baseline + 1 new).

- [x] **Task 2: Extend `src/flight/input.rs` — ToggleDampener variant + KeyX binding** (AC: #1)
  - [x] In `src/flight/input.rs`, append ONE variant AT THE END of the `FlightAction` enum: `ToggleDampener` (default-buttonlike — NO `#[actionlike(...)]` attribute). Final variant order matches AC #1's spec (11 variants total: 6 thrust + 2 axis pitch/yaw + 2 roll + 1 dampener-toggle).
  - [x] Extend `default_input_map()` with one new binding row inside the `InputMap::new([...])` slice, AFTER the `RollRight → KeyE` row and BEFORE the closing `])` of the slice:
    ```rust
    (FlightAction::ToggleDampener, KeyCode::KeyX),
    ```
    The trailing `.with_axis(...).with_axis(...)` chain stays intact AFTER the closing `])` (button bindings inside the slice; axis bindings in the chain — same partition as Story 3.7 AC #1).
  - [x] **No new imports needed** — `KeyCode::KeyX` is in the same family as `KeyCode::KeyW/A/S/D/Q/E` already in scope (per `bevy::input::keyboard::KeyCode`).
  - [x] **No tests added to `input.rs`** — same Stories 3.6/3.7 reasoning: the binding-map content is configuration data trivially correct by inspection, and runtime-verified via Task 5's smoke. A test like `assert_eq!(default_input_map().get_buttonlike(&FlightAction::ToggleDampener).next(), Some(KeyCode::KeyX))` is tautological — re-encoding configuration in two places.
  - [x] **Verify post-edit:** `cargo check` produces 0 warnings/errors. The new variant compiles cleanly; the InputMap construction stays valid (the `.with_axis(...)` chain still type-checks against the extended FlightAction enum). **Note:** only the expected `dead_code` warning on `DampenerState` remains (clears in Task 5).

- [x] **Task 3: Extend `src/tuning/config.rs` — `dampener_linear_strength` + `dampener_angular_strength` fields + Default impl + tuning.ron + 3 in-place test extensions** (AC: #3)
  - [x] In `src/tuning/config.rs`, add two `pub` fields to the `TuningConfig` struct, AFTER the `ship_torque_nm` field (insert-at-end ordering per Story 2.4/3.6/3.7 precedent). Annotate each with its own `#[serde(default = "default_…")]`:
    ```rust
    #[serde(default = "default_dampener_linear_strength")]
    pub dampener_linear_strength: f32,
    #[serde(default = "default_dampener_angular_strength")]
    pub dampener_angular_strength: f32,
    ```
  - [x] Add the two helper functions alongside the existing `default_outline_width` / `default_outline_color` / `default_ship_thrust_newtons` / `default_mouse_sensitivity` / `default_ship_torque_nm` helpers:
    ```rust
    fn default_dampener_linear_strength() -> f32 {
        2.0
    }

    fn default_dampener_angular_strength() -> f32 {
        3.0
    }
    ```
  - [x] Update `impl Default for TuningConfig`'s struct-literal: append `dampener_linear_strength: default_dampener_linear_strength(),` and `dampener_angular_strength: default_dampener_angular_strength(),` as the last two fields (in struct-field order).
  - [x] In `assets/config/tuning.ron`, append two new lines AFTER `ship_torque_nm: 80.0,` and BEFORE the closing `)` paren:
    ```
    dampener_linear_strength: 2.0,
    dampener_angular_strength: 3.0,
    ```
    Trailing commas correct per RON-0.8 convention. Final file size: 12 lines (was 10).
  - [x] Extend the existing 3 tests in-place per AC #3:
    - `tuning_config_default_matches_ron_initial_values` (config.rs:101-111): add as the last two assertions:
      ```rust
      assert_eq!(cfg.dampener_linear_strength, 2.0);
      assert_eq!(cfg.dampener_angular_strength, 3.0);
      ```
    - `tuning_config_deserializes_from_ron_bytes` (config.rs:114-126): edit the bytes literal to add `, dampener_linear_strength: 4.0, dampener_angular_strength: 6.0` BEFORE the closing `)`. Add as the last two assertions:
      ```rust
      assert_eq!(cfg.dampener_linear_strength, 4.0);
      assert_eq!(cfg.dampener_angular_strength, 6.0);
      ```
    - `tuning_config_legacy_schema_uses_defaults_for_added_fields` (config.rs:129-140): bytes literal is unchanged (the absent fields exercise the serde-default fallback). Add as the last two assertions:
      ```rust
      assert_eq!(cfg.dampener_linear_strength, 2.0);
      assert_eq!(cfg.dampener_angular_strength, 3.0);
      ```
  - [x] **Verify post-edit:** `cargo test tuning::config` produces 3 passing tests with the additional assertions (28 filtered out → total = 31, matches Task 1's +1 components test). Project test count unchanged after Task 3 alone (still 31 = 30 baseline + 1 components); the 5 new dampener-helper tests land in Task 4.

- [x] **Task 4: Author `dampener_acceleration` helper + `apply_dampener` system + `toggle_dampener` system + 5 unit tests in `src/flight/physics.rs`** (AC: #4, #5, #6, #8)
  - [x] At the top of `src/flight/physics.rs`, the existing `use` block already imports `avian3d::prelude::*;`, `bevy::prelude::*;`, `leafwing_input_manager::prelude::*;`, `crate::flight::PlayerShip`, `crate::flight::input::FlightAction`, `crate::tuning::TuningHandle`, `crate::tuning::config::TuningConfig`. Story 3.8 needs one additional import: `use crate::flight::components::DampenerState;` — add it BELOW `use crate::flight::PlayerShip;` (alphabetical placement keeps `components` → `input` ordering).
  - [x] Implement the helper `dampener_acceleration` BELOW the existing `ship_local_torque_vector` helper:
    ```rust
    /// Linear and angular acceleration to bleed velocity toward zero when
    /// `state.active`. Returns (Vec3::ZERO, Vec3::ZERO) when inactive — the
    /// early return covers the dampener-off case before any arithmetic.
    /// Linear strength scales linear-velocity-opposing acceleration; angular
    /// strength scales angular-velocity-opposing acceleration. Contributions
    /// are independent (linear vs. angular axes do not couple).
    pub fn dampener_acceleration(
        state: DampenerState,
        linear_velocity: Vec3,
        angular_velocity: Vec3,
        linear_strength: f32,
        angular_strength: f32,
    ) -> (Vec3, Vec3) {
        if !state.active {
            return (Vec3::ZERO, Vec3::ZERO);
        }
        (
            -linear_velocity * linear_strength,
            -angular_velocity * angular_strength,
        )
    }
    ```
    - **Why no clamping / NaN guarding:** consistent with the unclamped `ship_local_thrust_vector` / `ship_local_torque_vector` precedent (Stories 3.6/3.7); input-hardening lives in TuningConfig deserialization (deferred per deferred-work.md:222 + :228).
  - [x] Implement the system `apply_dampener` BELOW `dampener_acceleration` (and below the existing `apply_torque`):
    ```rust
    pub fn apply_dampener(
        tuning_assets: Res<Assets<TuningConfig>>,
        tuning_handle: Res<TuningHandle>,
        mut ships: Query<(Forces, &DampenerState), With<PlayerShip>>,
    ) {
        // PATTERN DEVIATION: Avian's apply_*_acceleration bypasses the mass/inertia
        // divisor; mathematically equivalent to applying force = -velocity * strength * mass
        // per the AC, but skips a redundant query of ComputedMass / ComputedAngularInertia.
        let tuning = tuning_assets
            .get(tuning_handle.0.id())
            .cloned()
            .unwrap_or_default();
        for (mut forces, state) in &mut ships {
            let (linear_accel, angular_accel) = dampener_acceleration(
                *state,
                forces.linear_velocity(),
                forces.angular_velocity(),
                tuning.dampener_linear_strength,
                tuning.dampener_angular_strength,
            );
            // apply_*_acceleration are no-ops for Vec3::ZERO (avoids waking sleeping bodies).
            forces.apply_linear_acceleration(linear_accel);
            forces.apply_angular_acceleration(angular_accel);
        }
    }
    ```
    - **Why iteration not `single_mut()`:** same as `apply_thrust` / `apply_torque` (Story 3.6/3.7 documented PATTERN DEVIATION) — handles 0-ship case (no panic) and 1-ship case (the only expected case).
    - **Why `forces.linear_velocity()` reads world-space:** Avian's `Forces` query exposes `linear_velocity() -> Vector` and `angular_velocity() -> AngularVector` (per `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/forces/query_data.rs:204-214`). Both return WORLD-space values. This pairs naturally with `apply_linear_acceleration` / `apply_angular_acceleration` (also world-space) — no local↔world transform needed.
    - **Why no `warn!` on cold-start tuning-not-loaded:** at 60 Hz this would emit 60 warns/sec. The warn lives at spawn time in `spawn_player_ship` (Story 3.5/3.6/3.7 precedent).
    - **Why dereferenced `*state`:** `DampenerState` is `Copy`; passing by value to the helper avoids a borrow chain through the closure.
  - [x] Implement the system `toggle_dampener` BELOW `apply_dampener`:
    ```rust
    pub fn toggle_dampener(
        mut ships: Query<(&ActionState<FlightAction>, &mut DampenerState), With<PlayerShip>>,
    ) {
        for (action_state, mut dampener) in &mut ships {
            if action_state.just_pressed(&FlightAction::ToggleDampener) {
                dampener.active = !dampener.active;
                info!(
                    "dampener {}",
                    if dampener.active { "engaged" } else { "disengaged" }
                );
            }
        }
    }
    ```
    - **Why `just_pressed` not `pressed`:** dampener should toggle ON the press-edge, NOT continuously while held. `just_pressed` fires exactly once per physical X-press regardless of OS auto-repeat; `pressed` would spam toggles every Update tick while held (per leafwing-0.20 `action_state/mod.rs:589-599` doc-example).
    - **Why an `info!` log here (vs. anti-pattern #10 forbidding per-tick logs):** `just_pressed` fires at most once per X-key-press (~order of magnitude ≤ 1 Hz under realistic player input). This is several orders of magnitude below the 60 Hz per-tick anti-pattern. The log is REQUIRED by AC #2 of the Epic spec ("an `info!` log records the new state for dev feedback"). The Dev Notes anti-patterns section explicitly carves out this exception.
    - **Why NO `Update` SystemSet placement:** per `src/pause/mod.rs:38-41` precedent (`toggle_pause_on_escape` runs in Update without a SystemSet), input-handling Update systems are loose unless ordering matters. `toggle_dampener` has no inter-system ordering concern (DampenerState is read by `apply_dampener` in FixedUpdate, after Update completes).
  - [x] Inside the existing `#[cfg(test)] mod tests` block at the bottom of `physics.rs`, add 5 new test functions BELOW the 9 existing thrust + rotation tests (3 thrust + 6 rotation including the 3.7 review-patch `roll_left_plus_roll_right_cancels_to_zero`). Use the existing pattern (no new test helpers needed — the helper takes plain primitives, no `ActionState` setup required):
    ```rust
    #[test]
    fn dampener_inactive_returns_zero_acceleration() {
        // Inactive dampener with non-zero velocities returns zero — verifies the early-return gate.
        let (lin, ang) = dampener_acceleration(
            DampenerState { active: false },
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 3.0, 0.0),
            2.0,
            3.0,
        );
        assert_eq!(lin, Vec3::ZERO);
        assert_eq!(ang, Vec3::ZERO);
    }

    #[test]
    fn dampener_active_zero_velocity_returns_zero_acceleration() {
        // Active dampener with zero velocities returns zero — verifies the no-op-quiet case.
        let (lin, ang) = dampener_acceleration(
            DampenerState { active: true },
            Vec3::ZERO,
            Vec3::ZERO,
            2.0,
            3.0,
        );
        assert_eq!(lin, Vec3::ZERO);
        assert_eq!(ang, Vec3::ZERO);
    }

    #[test]
    fn dampener_active_linear_velocity_returns_negative_proportional_acceleration() {
        // lin=(2,0,0), strength=2.0 → linear accel of (-4,0,0); angular zero (no coupling).
        let (lin, ang) = dampener_acceleration(
            DampenerState { active: true },
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::ZERO,
            2.0,
            3.0,
        );
        assert!((lin - Vec3::new(-4.0, 0.0, 0.0)).length() < 1e-5, "got {:?}", lin);
        assert_eq!(ang, Vec3::ZERO);
    }

    #[test]
    fn dampener_active_angular_velocity_returns_negative_proportional_acceleration() {
        // ang=(0,3,0), strength=3.0 → angular accel of (0,-9,0); linear zero (no coupling).
        let (lin, ang) = dampener_acceleration(
            DampenerState { active: true },
            Vec3::ZERO,
            Vec3::new(0.0, 3.0, 0.0),
            2.0,
            3.0,
        );
        assert_eq!(lin, Vec3::ZERO);
        assert!((ang - Vec3::new(0.0, -9.0, 0.0)).length() < 1e-5, "got {:?}", ang);
    }

    #[test]
    fn dampener_combines_linear_and_angular_independently() {
        // Both axes non-zero: linear strength scales linear only, angular strength scales angular only.
        let (lin, ang) = dampener_acceleration(
            DampenerState { active: true },
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            2.0,
            3.0,
        );
        assert!((lin - Vec3::new(-2.0, -4.0, -6.0)).length() < 1e-5, "got {:?}", lin);
        assert!((ang - Vec3::new(-12.0, -15.0, -18.0)).length() < 1e-5, "got {:?}", ang);
    }
    ```
    - **Test count delta in `flight/physics.rs`: +5** → physics.rs total: 12 tests (3 thrust + 6 rotation + 1 cancellation + 5 dampener; wait — actually 3 thrust + 6 rotation = 9; the 7th rotation test is the 3.7 review patch `roll_left_plus_roll_right_cancels_to_zero` which counts as part of rotation → 9 rotation+thrust + 5 dampener = 14 in physics.rs; combined with the 1 new test in components.rs = +6 total).
    - **Test count delta total: +6** → project total: **36** (= 30 baseline + 5 new in physics + 1 new in components). Matches AC #11 enforcement.
  - [x] **Verify post-edit:** `cargo test flight::physics` produces 14 passing tests (3 thrust + 6 rotation + 5 dampener — the 6 rotation includes the 3.7-review `roll_left_plus_roll_right_cancels_to_zero`). All 5 new dampener tests pass on first try. `cargo test flight::components` produces 1 passing test (`dampener_state_default_is_active`). Total project test count: 36 (verified later in Task 7 full sweep). Three `dead_code` warnings remain on `dampener_acceleration` / `apply_dampener` / `toggle_dampener` — expected, cleared in Task 5 wiring.

- [x] **Task 5: Wire `apply_dampener` registration + `toggle_dampener` registration + DampenerState in spawn tuple in `src/flight/mod.rs`** (AC: #2, #6)
  - [x] In `src/flight/mod.rs`, add a use line for the new `DampenerState` component (placed alphabetically next to the existing `use crate::arena::{ArenaEntity, ArenaSystems};` block: `components` → `input` → `physics`):
    ```rust
    use crate::flight::components::DampenerState;
    ```
  - [x] Add the module declaration `pub mod components;` to the existing module declarations at the top of the file (alphabetical placement: BEFORE `pub mod input;` and `pub mod physics;` — final order: `components`, `input`, `physics`). DONE in Task 1.
  - [x] In `FlightPlugin::build`, modify the existing `add_systems(FixedUpdate, ...)` block to extend the system tuple from 2 → 3 systems by appending `physics::apply_dampener`:
    ```rust
    app.add_systems(
        FixedUpdate,
        (physics::apply_thrust, physics::apply_torque, physics::apply_dampener)
            .in_set(FlightSystems::ApplyForces)
            .run_if(in_state(GameState::Arena)),
    );
    ```
  - [x] Add a new `add_systems(Update, ...)` block AFTER the FixedUpdate block (and BEFORE the OnEnter/OnExit cursor-grab registrations):
    ```rust
    app.add_systems(
        Update,
        physics::toggle_dampener.run_if(in_state(GameState::Arena)),
    );
    ```
    - **Why no SystemSet:** per AC #6 — input-handling Update systems are loose; matches `pause::toggle_pause_on_escape` precedent at `src/pause/mod.rs:38-41`.
  - [x] In `spawn_player_ship`, append `DampenerState::default()` to the component tuple AFTER `ActionState::<FlightAction>::default()`:
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
        default_input_map(),
        ActionState::<FlightAction>::default(),
        DampenerState::default(),
    ))
    ```
    - **Why insert-at-end:** matches Story 3.6/3.7 precedent for spawn-tuple growth; `DampenerState::default()` returns `{ active: true }` per AC #2 ("initial `active = true`").
  - [x] **Verify post-edit:** `cargo check` produces **0 warnings, 0 errors** — all dead_code warnings on `DampenerState`, `dampener_*_strength`, and the 3 dampener systems cleared once consumed by `spawn_player_ship` and `apply_dampener` / `toggle_dampener` registrations. The 3-system tuple compiles cleanly. The 13-component spawn tuple is well within Bevy's 15-tuple bundle limit.

- [x] **Task 6: Update generalized doc-comments — sprint header + module docs** (AC: #2, anti-pattern #1)
  - [x] In `src/flight/mod.rs:1-2`, generalize the module doc-comment to mention dampener:
    ```rust
    //! FlightPlugin — owns PlayerShip + CockpitCamera spawn, 6-DOF translation, 3-axis
    //! rotation, inertial dampener toggle, and Arena cursor-grab. Weapons land in subsequent stories.
    ```
    Replaces the prior "Dampener and weapons land in subsequent stories" line which is partially obsolete after 3.8.
  - [x] In `src/flight/physics.rs:1-3`, generalize the module doc-comment to mention dampener:
    ```rust
    //! Flight force/torque/acceleration application (FR2 6-DOF translation, FR3 3-axis
    //! rotation, FR5 inertial dampener). Reads ActionState<FlightAction>, applies
    //! ship-local force/torque/acceleration via Avian's Forces query (auto-cleared each FixedUpdate).
    ```
    Replaces the prior "Flight force/torque application (FR2 6-DOF translation, FR3 3-axis rotation)" line. NO story-id reference per anti-pattern #1.
  - [x] **Verify post-edit:** doc-comments compile (they are `//!` doc comments, so any cargo command that touches them will catch syntax errors). `cargo check` produces 0 warnings/errors.

- [x] **Task 7: Local verification sweep — full `feedback_full_build_output.md` discipline** (AC: #11)

  Per Till's memory `feedback_full_build_output.md`: `cargo check` exit-0 + tail is NOT proof of correctness. Capture each command's full output to a log file, then grep for `warning:|error:` and confirm count is **0**.

  - [x] `cargo check 2>&1 | tee /tmp/story-3-8-check.log` — grep returned **0** (0.14s incremental).
  - [x] `cargo build 2>&1 | tee /tmp/story-3-8-build.log` — grep returned **0** (3.12s).
  - [x] `cargo test 2>&1 | tee /tmp/story-3-8-test.log` — grep returned **0**; summary: `test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Test count: 36 = 30 baseline + 5 from `flight/physics.rs` + 1 from `flight/components.rs`.
  - [x] `cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-3-8-clippy.log` — grep returned **0** (0.69s).
  - [x] `cargo fmt --all -- --check 2>&1 | tee /tmp/story-3-8-fmt.log` — exit code 0; no fmt drift.
  - [x] `cargo build --release 2>&1 | tee /tmp/story-3-8-release.log` — grep returned **0**; 4m 18s (within Story 3.7's 4m 21s benchmark — no regression).
  - [x] **Cargo.lock delta check:** `git diff --stat Cargo.lock` shows no changes — no new transitive deps.
  - [x] **Runtime smoke** — `RUST_LOG=info,wgpu=warn,naga=warn,avian3d=info cargo run 2>&1 | tee /tmp/story-3-8-run.log` — let the game reach Arena, then exercise:
    - (a) **Press X once** → `info!("dampener disengaged")` log line; ship enters Newtonian-drift mode (matches Story 3.6/3.7 baseline).
    - (b) **Press X again** → `info!("dampener engaged")` log line; ship enters dampened mode (default state per AC #2).
    - (c) **Hold X for ~1 sec** → exactly 1 toggle log line emitted (verify `just_pressed` semantic; OS key-repeat must NOT spam toggles).
    - (d) **Dampener-ON + W press-and-release** → ship accelerates while W held, decelerates to ≈ 0 within ~3 s after release (convergence math: `exp(-2.0 * 3) ≈ 0.0025` → 0.25% of terminal velocity).
    - (e) **Dampener-ON + Q press-and-release** → ship rolls while Q held, angular-decelerates to ≈ 0 within ~3 s after release (convergence: `exp(-3.0 * 3) ≈ 0.00012` → 0.012% of terminal angular velocity).
    - (f) **Dampener-ON + W held for ~5 sec** → ship reaches terminal velocity ~7.5 m/s forward and STOPS accelerating (NOT unbounded). Compare visually to dampener-OFF case which would reach ~30 m/s after 2 s and keep going.
    - (g) **Dampener-ON + Q held for ~3 sec** → ship reaches terminal angular velocity ~0.5 rad/s ≈ 28°/s and STOPS angular-accelerating. Compare visually to dampener-OFF (Story 3.7 baseline) which would reach ~85°/s after 1 s and keep accelerating.
    - (h) **Dampener-OFF regression check:** press X to disengage; W press-and-release → ship continues to drift forever (Story 3.6 baseline). Q press-and-release → ship continues to rotate forever (Story 3.7 baseline). The dampener-OFF case MUST NOT alter pre-3.8 behavior.
    - (i) **Esc-pause + dampener-on mid-decay:** with dampener-ON and the ship mid-decay (e.g., right after releasing W from terminal velocity), press Esc to pause. The decay halts (per `pause_simulation_clocks`). Press Esc again to resume. Decay continues from the paused velocity (no jump, no reset).
    - (j) **X-press during Paused** → no toggle log line emitted (the `run_if(in_state(Arena))` gate suppresses the toggle handler while paused). Mash X 5× while paused → expect zero `dampener (engaged|disengaged)` log lines during the paused window.
    - (k) **Cmd-Tab focus-loss pause + resume → dampener state unchanged** (DampenerState lives on persistent PlayerShip; pause does not despawn the ship).
    - (l) **Quit cleanly** (window-close). No panic.
  - [x] **Post-runtime grep:**
    - `grep -c 'entered Loading'` → **1**
    - `grep -c 'entered MainMenu'` → **1**
    - `grep -c 'entered Arena'` → **≥ 1** initial; matches `spawned PlayerShip` count (Story 3.6 AC #9 false-positive holds).
    - `grep -c 'spawned PlayerShip'` → matches `entered Arena` count.
    - `grep -cE 'dampener (engaged|disengaged)'` → **≥ 2** (at least one engage + one disengage exercised).
    - `grep -cE 'panic|backtrace|FATAL'` → **0**
    - `grep -ci 'ambiguous.*camera.*order'` → **0** (Story 3.5 regression check).
    - `grep -cE 'ERROR.*avian|WARN.*Avian'` → **0**
  - [x] Confirm the 3 pre-existing documented WARNs from Story 3.5/3.6/3.7 reappear unchanged (splash-cleanup race per deferred-work.md:139, wgpu fragment-output per Story 2.3, winit Skipped Destroyed per Story 1.6). If a fourth WARN appears, investigate and either explain it in Dev Agent Record or add a deferred-work entry.

- [x] **Task 8: Update `_bmad-output/implementation-artifacts/deferred-work.md` IF NEEDED** (AC: #10)
  - [x] Story 3.8 anticipates **NO** new deferred-work entries — the design fully consumes the dampener epic spec, AC #10 tug-of-war terminal-velocity is a Dev Agent Record observation (not a deferral), and AC #5's NaN/clamp gap is already covered by deferred-work.md:222 (no duplicate entry). **Smoke confirmed all-green — no surprises surfaced; deferred-work.md unchanged.**
  - [x] **Conditional entries** — add ONLY if surfaced during impl: **None added (smoke clean).**
    - **Dampener feels too snappy / too sludgy:** if Till's smoke shows defaults (linear=2.0, angular=3.0) feel wrong (e.g., decay too fast for "graceful spaceship feel" or too slow for "arcade snap"), defer the tuning curve / non-linear-strength experimentation to a future polish story (likely Epic 5 ship-subsystem polish or Epic 10 final-polish). Format:
      ```
      ## Deferred from: 3-8-inertial-dampener-toggle (2026-XX-XX)
      - **Dampener strength tuning curve** — `src/flight/physics.rs:dampener_acceleration`. Defaults felt [too snappy / too sludgy] in smoke. **Resolution path:** experiment with non-linear strength (e.g., `-v.normalize() * fixed_strength` for constant-magnitude deceleration regardless of speed, OR `-v * (strength + |v| * coefficient)` for velocity-proportional damping). Defer to Epic 5 ship-subsystem polish or Epic 10 final-polish.
      ```
    - **Dampener UI feedback missing:** if Till wants an HUD indicator ("DAMPENER: ON/OFF") for the player rather than just an `info!` log, defer to Story 3.11 (HUD baseline) or Story 5.4 (HUD wiring for shields/hull). Format:
      ```
      ## Deferred from: 3-8-inertial-dampener-toggle (2026-XX-XX)
      - **Dampener HUD indicator** — `src/ui/hud.rs` (Story 3.11 / 5.4). Currently only `info!` log signals state changes. Players in release builds (no log-tail) won't know dampener state. **Resolution path:** add a `HudPlaceholder { field: HudField::Dampener }` slot in Story 3.11's HUD baseline, wire to `DampenerState` query in Story 5.4 HUD wiring.
      ```
    - **Pause-during-toggle race:** if the smoke reveals that an X-press WHILE the pause-state-transition is in flight (rare race) gets dropped or double-fires, defer investigation to a future stability story. Format:
      ```
      ## Deferred from: 3-8-inertial-dampener-toggle (2026-XX-XX)
      - **Pause-during-toggle X-press race** — `src/flight/physics.rs:toggle_dampener`. ...
      ```
      (Only if observed in smoke; otherwise omit.)

- [~] **Task 9: Sprint-status bookkeeping + commit/push (NOT YET — await Till's authorization)** (per Story 3.5/3.6/3.7 precedent)
  - [x] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - [x] `3-8-inertial-dampener-toggle: ready-for-dev → in-progress` — flip at start of dev-story.
    - [x] `3-8-inertial-dampener-toggle: in-progress → review` — flip after Till's runtime-smoke confirmation. **Done 2026-05-05.**
    - [x] `last_updated:` bumped to current date with brief note (e.g., `2026-XX-XX (Story 3.8 in-progress → review — inertial dampener toggle verified)`). **Done 2026-05-05.**
  - [x] Update this story file's `Status:` field at line 3. Flip `ready-for-dev → in-progress → review` after Till's runtime-smoke confirmation. **Done 2026-05-05.**
  - [x] Populate the `## Dev Agent Record` section: `Agent Model Used`, `Debug Log References` (the 7 commands' grep counts table), `Completion Notes List` (one bullet per AC #1–#12), `File List` (Modified: `src/flight/input.rs`, `src/flight/physics.rs`, `src/flight/mod.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`, `sprint-status.yaml`, this file; Added: `src/flight/components.rs`). **Done.**
  - [ ] **Commit 1 (feat):** stage `src/flight/components.rs`, `src/flight/input.rs`, `src/flight/physics.rs`, `src/flight/mod.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`. Message: `feat: inertial dampener toggle (Story 3.8)`. **DO NOT COMMIT UNLESS TILL AUTHORIZES.**
  - [ ] **Commit 2 (bmad):** stage `_bmad-output/implementation-artifacts/sprint-status.yaml`, `_bmad-output/implementation-artifacts/3-8-inertial-dampener-toggle.md`, AND `_bmad-output/implementation-artifacts/deferred-work.md` IF a new entry was added. Message: `bmad: story 3.8 ready-for-dev → review (inertial dampener toggle)`. **DO NOT COMMIT UNLESS TILL AUTHORIZES.**
  - [ ] **DO NOT push.** Push happens only after explicit authorization, AND only after Story 3.8 code review (`bmad-code-review`) passes per Story 3.5/3.6/3.7 precedent.

## Dev Notes

### Architecture compliance

- **Plugin home:** `FlightPlugin` in `src/flight/mod.rs` per architecture.md:558-563 (FR1–FR8 location). Story 3.8 extends; no new plugin.
- **File creation:** Story 3.8 introduces `src/flight/components.rs` per architecture.md:560 ("`Thrusters, InertialDampener, Boost, TractorEmitter`"). Story 3.7 explicitly punted this slot to 3.8 ("the `flight/components.rs` slot remains unintroduced ... they land at Stories 3.8 dampener"). `DampenerState` is the first occupant; future stories add `Boost` (FR6, Epic 6 Story 6.12) and others.
- **File extension:** `flight/input.rs` (FR1 binding for the new ToggleDampener action), `flight/physics.rs` (FR5 dampener system + helper per architecture.md:677), and `flight/mod.rs` (plugin wiring).
- **SystemSet name:** `FlightSystems::ApplyForces` houses `apply_thrust` (3.6), `apply_torque` (3.7), AND `apply_dampener` (3.8) — all force/torque/acceleration contributors in the same FixedUpdate phase. Architecture.md:411 prescribes `enum FlightSystems { ReadInput, ApplyForces, IntegratePhysics }` — only `ApplyForces` is project-owned; Story 3.8 declines to add `ReadInput` (leafwing owns) or `IntegratePhysics` (Avian owns), continuing 3.6/3.7's pattern.
- **System naming:** `apply_dampener` (snake_case verb-phrase per architecture.md:323) — symmetric with `apply_thrust` / `apply_torque`. The `apply_*` prefix groups the force/torque/acceleration-application family for future readers. `toggle_dampener` follows the `toggle_*` verb-phrase pattern (mirrors `toggle_pause_on_escape` at `src/pause/mod.rs:94`).
- **Helper naming:** `dampener_acceleration` (descriptive snake_case for a pure free function — returns `(linear, angular)` tuple, naming describes the OUTPUT not the input dimension; mirrors `ship_local_thrust_vector` / `ship_local_torque_vector` precedent of "what does this return" naming).
- **Component naming:** `DampenerState` (PascalCase, single-responsibility per architecture.md:322 — one boolean field; no god-struct anti-pattern). The component-name-vs-state-name distinction: `DampenerState` is the component (Bevy convention), with `active: bool` as the state field. `InertialDampener` from architecture.md:560 was a working name; `DampenerState` is the implementation name (the architecture line is descriptive, not prescriptive — see anti-pattern #19 below).
- **Cross-plugin ordering:** none introduced. `apply_dampener` joins the existing `FlightSystems::ApplyForces` set; `toggle_dampener` runs in `Update` (loose, no SystemSet). No interaction with `PausePlugin` beyond the shared `run_if(in_state(GameState::Arena))` gate.
- **Run-condition gate:** `apply_dampener.run_if(in_state(GameState::Arena))` matches `apply_thrust` / `apply_torque` gates. `toggle_dampener.run_if(in_state(GameState::Arena))` ensures X-press while Paused/MainMenu/Loading is a no-op (per AC #11 design — toggling mid-pause is UX-confusing).
- **Avian + Bevy + leafwing version pins:** all unchanged from Stories 3.6/3.7 (`bevy = "0.18"`, `avian3d = "0.6"`, `bevy_mod_outline = "0.12"`, `leafwing-input-manager = "0.20"` per Cargo.toml:8-12). No new external deps; no Cargo.toml or Cargo.lock churn expected.

### Library / framework specifics — Avian 0.6 `Forces::apply_linear_acceleration` + `apply_angular_acceleration` (in-codebase precedent: only `apply_local_force` from 3.6, `apply_local_torque` from 3.7)

- **`Forces::linear_velocity() -> Vector` (read):** returns world-space linear velocity (per `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/forces/query_data.rs:204-208`). NOT `Res<Time>`-dependent; reads the current cached `LinearVelocity` component.
- **`Forces::angular_velocity() -> AngularVector` (read):** returns world-space angular velocity (per `query_data.rs:210-214`). For 3D, `AngularVector = Vec3` (axis-angle representation).
- **`Forces::apply_linear_acceleration(acceleration: Vector)` (write):** applies a continuous linear acceleration (m/s²) for one physics step, then auto-clears (per `query_data.rs:482-487`). **KEY DIFFERENCE FROM `apply_force`:** this method bypasses the `inverse_mass()` divisor — accel goes straight into the integration data without mass scaling. For `dv/dt = -strength * v`, this is exactly what we want (no need to read mass).
- **`Forces::apply_angular_acceleration(acceleration: AngularVector)` (write):** applies a continuous angular acceleration (rad/s²) for one physics step, then auto-clears (per `query_data.rs:539-544`). **KEY DIFFERENCE FROM `apply_torque`:** bypasses the `effective_inverse_angular_inertia()` tensor multiplication. For `dω/dt = -strength * ω`, this avoids redundant tensor work.
- **PATTERN DEVIATION justification:** AC #4 of the Epic spec says "ExternalForce equal to `-linear_velocity * dampener_linear_strength * mass`" and "ExternalTorque equal to `-angular_velocity * dampener_angular_strength * moment_of_inertia`". The `apply_*_acceleration` API is mathematically equivalent (`accel = force / mass = (-v * mass * strength) / mass = -v * strength`) but semantically cleaner: we don't conceptually want to scale by mass and then unscale by mass; we want a velocity-proportional deceleration. The deviation is documented inline in `apply_dampener` per architecture.md:454.
- **`apply_*_acceleration` no-op on `Vec3::ZERO`:** the inner `if accel != Vector::ZERO` early-return at `query_data.rs:483, 540` ensures inactive-dampener case (which returns `(Vec3::ZERO, Vec3::ZERO)` from `dampener_acceleration`) produces zero net work and does not wake sleeping bodies.
- **`Forces` is a `QueryData(mutable)` struct:** the write methods (`apply_*`) require `&mut Forces`, hence `Query<(Forces, ...)>` is implicit-mut on the Forces field per `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/forces/query_data.rs:106-120`. The query `Query<(Forces, &DampenerState), With<PlayerShip>>` is correct — `Forces` is mutable, `&DampenerState` is read-only (we don't toggle inside `apply_dampener`; toggling lives in `toggle_dampener`).

### Library / framework specifics — leafwing-input-manager 0.20 `just_pressed` (in-codebase precedent: only `pressed` from 3.6/3.7)

- **`ActionState::just_pressed(&action) -> bool`:** edge-triggered button state — returns `true` exactly once per press-edge transition (per leafwing-0.20 `action_state/mod.rs:589-599`). The internal state-tracking de-bounces OS auto-repeat: holding X for 1 second produces exactly 1 `just_pressed` true, NOT N where N is the auto-repeat count.
- **`just_pressed` vs `pressed`:** `pressed` returns `true` continuously while held (used for thrust/roll); `just_pressed` returns `true` only on the press-edge tick (used for toggle). Story 3.6's `apply_thrust` uses `pressed` for continuous thrust; Story 3.8's `toggle_dampener` uses `just_pressed` for one-shot toggle. The choice is semantic, not stylistic.
- **`just_pressed` and `Update` schedule:** leafwing's `InputManagerPlugin::<FlightAction>` (registered in 3.6 at `src/flight/mod.rs:51`) updates `ActionState` in `PreUpdate` each frame (per leafwing-0.20 `plugin.rs:77-85`). `toggle_dampener` runs in `Update`, so it observes the `ActionState` *after* leafwing's PreUpdate refresh — the `just_pressed` semantic works correctly.
- **`InputManagerPlugin::<FlightAction>`:** already registered by Story 3.6; adding `ToggleDampener` to `FlightAction` is fully backward-compatible (the plugin processes enum variants dynamically via the `Actionlike` derive's iteration, per the Story 3.7 dev-notes precedent).

### Library / framework specifics — Bevy 0.18 `Time<Physics>` + `Time<Virtual>` interaction with dampener

- **`pause_simulation_clocks` (Story 3.4 at `src/pause/mod.rs:122-133`)** halts both `Time<Virtual>` and `Time<Physics>`. When the pause is active, `FixedUpdate` does NOT advance — `apply_thrust` / `apply_torque` / `apply_dampener` are NOT called. Therefore the dampener's exponential decay HALTS during pause and resumes from the paused velocity on `resume_simulation_clocks`. This is the intended behavior per AC #11 (verified via runtime smoke).
- **`Update` is NOT halted by the pause:** `Update` runs every render frame regardless of `Time<Virtual>` state. `toggle_dampener` would still fire if not gated. The `run_if(in_state(GameState::Arena))` gate on `toggle_dampener` (NOT on FixedUpdate's `apply_dampener` which already has the gate) prevents X-press during Paused from flipping the state. Both gates are necessary: `apply_dampener`'s gate prevents accidental dampening of zero-velocity (a no-op anyway) outside Arena; `toggle_dampener`'s gate prevents the UX-confusing case where X-press during Paused changes invisible state.

### File structure requirements

```
src/
├── flight/
│   ├── mod.rs               # MODIFIED — +1 module decl (`pub mod components;`); +1 use line (DampenerState); +1 system in ApplyForces tuple (apply_dampener); +1 add_systems(Update, ...) block (toggle_dampener); +1 component in spawn tuple (DampenerState::default()); doc-comment generalized (Task 6)
│   ├── components.rs        # NEW FILE — DampenerState component + Default + 1 unit test (~25 lines)
│   ├── input.rs             # MODIFIED — +1 enum variant (ToggleDampener); +1 binding row (KeyX → ToggleDampener)
│   └── physics.rs           # MODIFIED — +1 use line (DampenerState); +dampener_acceleration helper; +apply_dampener system; +toggle_dampener system; +5 unit tests; doc-comment generalized (Task 6)
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
│   └── tuning.ron           # MODIFIED — +2 lines (dampener_linear_strength: 2.0, dampener_angular_strength: 3.0)
└── ...                      # UNCHANGED
Cargo.toml                   # UNCHANGED — no new deps
Cargo.lock                   # UNCHANGED — no transitive-dep churn
```

**Target file size deltas:**

| File | Pre-3.8 | Post-3.8 target | Delta |
|---|---|---|---|
| `src/flight/components.rs` | (does not exist) | ~25 lines | NEW FILE |
| `src/flight/input.rs` | 35 | ~38 | +3 (1 variant + 1 binding + 1 line of formatting) |
| `src/flight/physics.rs` | 248 | ~330 | +82 (1 use line + helper + apply_dampener + toggle_dampener + 5 tests + 2 doc-comment lines) |
| `src/flight/mod.rs` | 142 | ~155 | +13 (module decl + use line + 1 system in tuple + 1 add_systems block + 1 spawn-tuple line + doc-comment edit) |
| `src/tuning/config.rs` | 141 | ~157 | +16 (2 fields + 2 helpers + 2 Default literals + 6 assertions + 1 ron-bytes literal extension) |
| `assets/config/tuning.ron` | 10 | 12 | +2 |

### Testing standards

Per architecture.md:351-354:
- **Co-located** `#[cfg(test)] mod tests { ... }` at the bottom of each module file.
- **Pure-logic modules first-class test targets;** integration tests deferred post-M3.

Story 3.8's `apply_dampener` and `toggle_dampener` systems are integration-test-shaped (would need `MinimalPlugins + PhysicsPlugins + leafwing + tick FixedUpdate manually for 60 ticks at 60 Hz with input injection`) and therefore not unit-tested. The pure-logic helper `dampener_acceleration` IS unit-tested (5 tests covering: inactive-with-velocity, active-zero-velocity, active-linear-only, active-angular-only, active-both-axes-with-independent-strengths). The `DampenerState::default()` invariant is unit-tested in `flight/components.rs` (1 test).

`TuningConfig::dampener_linear_strength` and `dampener_angular_strength` add NO new test functions — the existing 3 tests are extended in-place to cover the two new fields. Test count delta from tuning: **+0**.

**Net post-3.8 test count target: 36** (= 30 baseline from end of 3.7 + 5 from `flight/physics.rs` + 1 from `flight/components.rs`). AC #11 enforces N = 36.

### Anti-patterns to avoid (catalogued from Stories 1.5–3.7 review precedent + 3.8-specific risks)

1. **Story-id references in module doc-comments and inline comments** — Stories 1.5/3.2/3.6/3.7 review patches removed all "Story X.Y" references. **Do NOT** write `//! Story 3.8 introduces dampener`. **Do NOT** write `// Story 3.8 — toggle_dampener`. Module docs describe what the module owns; inline comments explain WHY when non-obvious.
2. **Wildcard imports beyond `bevy::prelude::*`, `avian3d::prelude::*`, `leafwing_input_manager::prelude::*`** — explicit imports per architecture.md naming-discipline.
3. **Adding `ToggleDampener` as an axis-kind action** — it MUST be default-buttonlike (no `#[actionlike(...)]` attribute). Toggle is press-edge-driven; an axis would be nonsense semantically.
4. **Using `pressed` instead of `just_pressed` for toggle** — `pressed` would flip the state every Update tick while X is held (60+ flips per second), making the dampener oscillate. `just_pressed` is the correct edge-triggered semantic.
5. **`.after(specific_function)` for system ordering** — architecture.md:415 forbidden. The new `apply_dampener` and `toggle_dampener` systems use SystemSet placement and run_if gates only.
6. **Clamping dampener-acceleration magnitude proactively** — same Story 3.6/3.7 anti-pattern reasoning. Epic spec says velocity-proportional damping; no clamp. If smoke shows an overshoot or oscillation (would require negative strength or huge dt — neither expected), defer to a future tuning story.
7. **Using `apply_local_*` instead of world-space `apply_*_acceleration`** — `LinearVelocity` and `AngularVelocity` are stored in WORLD-space; the dampener acts in world-space. `apply_local_*` would multiply by ship orientation, which is wrong (decelerating along the ship's *current* local axes does not bleed world-space velocity to zero). Use `apply_linear_acceleration` and `apply_angular_acceleration` (no `_local` suffix).
8. **Using `apply_force` / `apply_torque` instead of `apply_*_acceleration`** — `apply_force` divides by mass; `apply_torque` divides by inertia tensor. Both work correctly if you pre-multiply by mass/inertia, but require an extra query for `ComputedMass` / `ComputedAngularInertia`. Per the PATTERN DEVIATION (AC #5), `apply_*_acceleration` is the cleaner API for velocity-proportional damping.
9. **Adding new `FlightSystems` enum variants** — `ApplyForces` (added by 3.6) covers thrust + torque + acceleration. Adding a separate `Dampen` variant would force a chain that doesn't need to exist (force/torque/acceleration sum independently in Avian per the QueryData semantics).
10. **Per-tick `info!` / `warn!` / `debug!` logs in `apply_dampener`** — same 3.6/3.7 discipline. The ONE log allowed in this story is in `toggle_dampener` (gated by `just_pressed`, fires ≤ 1 Hz under realistic input).
11. **Touching `src/state.rs`** — `apply_dampener.run_if(in_state(GameState::Arena))` and `toggle_dampener.run_if(in_state(GameState::Arena))` handle cloning internally. The `Copy` derive on `GameState` remains deferred per deferred-work.md:198.
12. **Touching `src/main.rs`** — `FlightPlugin` is already registered. Dampener wiring is per-plugin (lives in `FlightPlugin::build`), not App-level.
13. **Adding `Cargo.toml` deps** — no new deps. `Component` is from `bevy::prelude`; `Forces` / `apply_*_acceleration` are from `avian3d::prelude` (already in scope from 3.6); `just_pressed` is from `leafwing_input_manager::prelude` (already in scope from 3.6).
14. **Recreating the `InputMap` per tick** — the InputMap is part of the spawn tuple and lives for the entity's lifetime. The new ToggleDampener binding is added to `default_input_map()` once at spawn time.
15. **Putting `toggle_dampener` in `FixedUpdate`** — input handling belongs in `Update` (matches `pause::toggle_pause_on_escape` precedent). FixedUpdate is for physics; one-shot input handlers should respond at render-frame cadence to feel responsive.
16. **Making `DampenerState` a Resource instead of a Component** — `DampenerState` is per-PlayerShip (multiple ships = multiple states). A Resource would not generalize to a future co-op or AI-pilot scenario. Component is the correct ECS shape.
17. **Splitting `flight/physics.rs` per-story into `flight/dampener.rs`** — architecture-prescribed split is by RESPONSIBILITY (force/torque/acceleration math), not by story scope. `physics.rs` owns thrust + torque + dampener.
18. **Adding visual UI for the dampener state in 3.8** — HUD work belongs to Story 3.11 (HUD baseline) or Story 5.4 (HUD wiring). Story 3.8's only player-facing signal is the `info!` log per AC #2 of the Epic. UI HUD wiring is deferred per AC #11 git-status spec ("NO entries under `src/ui/**`").
19. **Naming the component `InertialDampener` per architecture.md:560 verbatim** — `InertialDampener` is a working/descriptive name in the architecture document. The implementation name is `DampenerState`, which more accurately describes the component's role (carrying state, not BEING the dampener). The dampener IS the system + helper + tuning fields; the component is just its on/off flag. This is a benign rename consistent with the "renames to conform require no justification" rule (architecture.md:448).
20. **Adding a `prev_active: bool` field for "just toggled" detection** — the `info!` log is emitted from inside `toggle_dampener`'s `just_pressed` branch (the single source of truth for state changes). No need for a separate edge-detection field on the component.
21. **Treating "dampener disengaged" as a regression of Stories 3.6/3.7** — the dampener-OFF case is INTENTIONAL Newtonian-drift behavior (per FR5: "modulate between Newtonian drift and arcade tightness"). Smoke verification (h) tests this explicitly.

### Logging discipline

Per architecture.md:376-383:
- `info!` for lifecycle events: existing logs from `log_arena_entered` ("entered Arena") at `src/main.rs:54`, `pause::pause_on_focus_loss` ("paused on focus loss") and `pause::toggle_pause_on_escape` ("paused via Escape") at `src/pause/mod.rs:71, 109`, etc. all unchanged.
- ONE NEW `info!` in `toggle_dampener` — `info!("dampener {}", if state.active { "engaged" } else { "disengaged" })` — gated by `just_pressed` (≤ 1 Hz). Required by AC #2 of the Epic spec.
- NO per-tick logs in `apply_dampener`, `apply_thrust`, or `apply_torque`.

### Project Structure Notes

- **Alignment with unified project structure:** `src/flight/components.rs` creation matches architecture.md:560 prescription; `src/flight/input.rs` extension matches architecture.md:561; `src/flight/physics.rs` extension matches architecture.md:562, :677. `flight/mod.rs` wiring follows architecture.md's "plugin owns its lifecycle systems" pattern.
- **Detected variances:**
  - PATTERN DEVIATION (per architecture.md:454): `apply_dampener` uses `Forces::apply_*_acceleration` instead of `apply_force` / `apply_torque` per the AC's literal force/torque wording. Mathematically equivalent; documented inline.
  - Component name `DampenerState` instead of `InertialDampener` (architecture.md:560 working name) per anti-pattern #19 above.
- **Feature divergence note:** Avian 0.6's `Forces` query data exposes `apply_*_acceleration` methods that bypass the mass/inertia divisor — a 0.6-specific API; older Avian versions used `LinearAcceleration` / `AngularAcceleration` components instead. Story 3.8 follows the post-0.6 pattern.

### Library / framework specifics — Bevy 0.18 `Time<Physics>` + `Time<Virtual>` interaction with dampener

(Cross-referenced above — see "Library / framework specifics — Bevy 0.18 `Time<Physics>` + `Time<Virtual>` interaction with dampener" earlier in Dev Notes.)

## Previous Story Intelligence (Story 3.7 — Flight Input → 3-Axis Rotation)

Story 3.7 is the most recent reference for the development pattern. Key learnings to inherit:

- **Component-tuple ordering** (3.6/3.7 spawn pattern): `spawn_player_ship`'s tuple grows by appending new components at the end. Story 3.8 adds `DampenerState::default()` as the 13th component (was 12 in 3.7).
- **Cold-start tuning fallback** (3.6/3.7 pattern): `tuning_assets.get(handle).cloned().unwrap_or_default()` + a one-shot `warn!` at spawn time if `None`. Story 3.8's `apply_dampener` reuses this exact pattern (no new warn).
- **Avian `Forces` query data, not `ExternalForce` component** (3.6 Deviation #2): the `Forces` query data is the canonical Avian-0.6 API. Story 3.8 extends usage to `apply_linear_acceleration` and `apply_angular_acceleration` (in addition to the existing `apply_local_force` and `apply_local_torque` consumers from 3.6/3.7).
- **`ActionState::just_pressed(&action)` for edge-triggered, `pressed(&action)` for continuous** (3.6/3.7 + leafwing-0.20 docs): Story 3.8 uses `just_pressed` for the dampener toggle (one-shot semantic).
- **Test count baseline = 30** (`cargo test` 2026-05-04 measurement — 24 from end of 3.6 + 5 new in 3.7 + 1 from 3.7 review patch P5): Story 3.8 adds 6 → final 36.
- **Pause/resume preserves PlayerShip state** (3.6 AC #9 ✅ FALSE-POSITIVE, 3.7 AC #10 re-confirmed for rotation): Story 3.8 AC #11 re-confirms for `DampenerState` (component lives on persistent ship — pause does not despawn).
- **Story-id-comment scrub** (3.6/3.7 review patches): keep doc comments and inline comments free of "Story 3.8" / "Story X.Y" references.
- **Per-command grep verification harness** (3.6/3.7 Task 5/Task 7): mirrored exactly per AC #11 + Task 7. The 7-command + runtime-smoke sweep is the canonical local-verification pattern.
- **2-commit pattern (feat + bmad)** (3.6/3.7 final task): mirrored. Commits and pushes await Till's authorization.
- **Three-system tuple in `add_systems(FixedUpdate, ...)`** (Story 3.7's `(apply_thrust, apply_torque)` 2-tuple pattern, generalized): Story 3.8 grows the tuple to 3 systems `(apply_thrust, apply_torque, apply_dampener)` — Bevy 0.18 idiom for sibling systems sharing identical `.in_set` and `.run_if` configuration.
- **`Update`-schedule input handler precedent** (`pause::toggle_pause_on_escape` at `src/pause/mod.rs:38-41`): Story 3.8's `toggle_dampener` mirrors this — input handlers run in Update, not FixedUpdate.

## Git intelligence summary

Recent commit history (`git log --oneline -8`):
- `baf057e` bmad: epic 10 add Story 10.13 — final mesh assets in MVP (NOT a code change; epic-list update)
- `31954f4` bmad: story 3.7 review → done (code review passed, 5 patches applied) ← **last completed story; 3-axis rotation + cursor grab + accumulator landed**
- `541a6d7` fix: Story 3.7 review — mouse-look accumulator + cursor-warp suppression ← **review patches; introduced MouseLookDelta resource + suppress-frames pattern**
- `33151c0` bmad: story 3.7 ready-for-dev → review (3-axis rotation)
- `108b381` feat: 3-axis rotation + cursor grab (Story 3.7) ← **canonical predecessor commit; FlightAction extensions, apply_torque, ship_local_torque_vector, cursor-grab**
- `253c3dd` bmad: story 3.6 review → done
- `51f040e` fix: remove story-id comments from mod.rs and config.rs (Story 3.6 review) ← **load-bearing convention: no-story-id-comments**
- `3e8fb2d` bmad: story 3.6 ready-for-dev → review (6-DOF translation thrust)

**Patterns extracted:**

- **2-commit cadence per story:** `feat:` for code + `bmad:` for spec/state metadata. Optional `fix:` review-patch commits for cleanup. Story 3.8 follows.
- **Cargo.lock unchanged since pre-3.6:** leafwing's transitive deps were locked at Story 1.2's plugin-compat gate; 3.6/3.7 added no new transitives. Story 3.8 adds NO new external surface; Cargo.lock should remain unchanged.
- **Module patterns introduced ahead of consumers:** `flight/components.rs` is created in 3.8 with one occupant (`DampenerState`); future stories add `Boost`, `Thrusters`, `TractorEmitter` as their consumers land.
- **Files touched by 3.7's review patches (`src/flight/mod.rs`, `src/flight/physics.rs`):** these are the same files Story 3.8 will modify. Verify post-edit that the 3.7 review patches (MouseLookDelta resource, MouseLookSuppressFrames, accumulate_mouse_look system, cursor-warp suppression in `grab_cursor_for_arena`) remain intact and functional.

## Latest tech information (Bevy 0.18 + Avian 0.6 + leafwing 0.20)

Story 3.8 introduces no new external dependencies. Every API surface used has the following confirmation status:

- **`leafwing-input-manager = "0.20"`** — already exercised by Stories 3.6/3.7. New surface for 3.8: `ActionState::just_pressed(&action) -> bool` (per leafwing-0.20 `action_state/mod.rs:589-599`). All other input surfaces (Actionlike derive, InputMap construction, ActionState component) unchanged from 3.6/3.7.
- **`avian3d::dynamics::rigid_body::forces::Forces`** — already accessible via the `Forces` query data imported by Stories 3.6/3.7. New surfaces:
  - `Forces::linear_velocity() -> Vector` (per `query_data.rs:204-208`) — read; world-space.
  - `Forces::angular_velocity() -> AngularVector` (per `query_data.rs:210-214`) — read; world-space.
  - `Forces::apply_linear_acceleration(Vector)` (per `query_data.rs:482-487`) — write; bypasses mass divisor.
  - `Forces::apply_angular_acceleration(AngularVector)` (per `query_data.rs:539-544`) — write; bypasses inertia tensor multiplication.
- **`bevy::prelude::Component` + `#[derive(Component)]`** — Bevy-0.18 attribute macro for component traits. Stable since Bevy 0.7. Standard usage; no surprises.
- **`bevy::prelude::Update` schedule** — Bevy-0.18 standard schedule for per-render-frame systems. Already exercised by `pause::toggle_pause_on_escape`.
- **No version bumps:** `bevy = "0.18"`, `avian3d = "0.6"`, `bevy_mod_outline = "0.12"`, `bevy_kira_audio = "0.25"`, `leafwing-input-manager = "0.20"` — all unchanged.

## Project context reference

- **Memory:** `MEMORY.md` (auto-loaded at session start) — Till's user memories include `feedback_full_build_output.md` (per-command-grep verification discipline), `feedback_compact_review_style.md` (compact responses), `feedback_staged_rollout.md` (staged-rollout preference, justifies the lean Story 3.8 scope: dampener toggle ONLY; HUD indicator deferred to 3.11/5.4).
- **Brainstorming canon:** `_bmad-output/brainstorming/brainstorming-session-2026-04-21-1114.md` — original concept doc; the inertial dampener is the toggle that enables FR5's "Newtonian-vs-arcade modulation" core mechanic (motivates the Caravan-supply-run combat flow that needs both drift-aiming and stop-on-target precision).
- **Architecture canon:** `_bmad-output/planning-artifacts/architecture.md` — single-file authoritative architecture.
- **Sprint plan:** `_bmad-output/implementation-artifacts/sprint-status.yaml` — Story 3.8 is the next backlog item after 3.7 done.
- **Deferred work:** `_bmad-output/implementation-artifacts/deferred-work.md` — Story 3.8 inherits open entries:
  - line 198 (GameState `Copy` — re-deferred; 3.8 doesn't legitimately touch state.rs)
  - line 222 (NaN/inf guard on tuning scalars — applies to dampener tuning fields too; consolidated entry covers all TuningConfig f32 fields)
  - line 228 (range/sign validation on tuning scalars — same; `dampener_*_strength = 0.0` would disable damping silently, `< 0.0` would amplify velocity exponentially)
  - line 232 (touchpad ergonomics for mouse pitch/yaw — orthogonal to 3.8's dampener; documents why touchpad-on-macOS users may especially want the dampener on for snap-aim recovery)
- **No new external research needed.** All API surfaces are documented in the source files referenced above.

### References

- [Source: [`_bmad-output/planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md:199-228`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md)] — Story 3.8 epic spec (User story + 5 BDD ACs + epic context).
- [Source: [`_bmad-output/planning-artifacts/prd.md:504`](../planning-artifacts/prd.md)] — FR5 inertial dampener — toggleable Newtonian-drift-vs-arcade-tightness modulation (the FR Story 3.8 closes).
- [Source: [`_bmad-output/planning-artifacts/architecture.md:240`](../planning-artifacts/architecture.md)] — Avian `FixedUpdate` at 60 Hz; Story 3.8's `apply_dampener` runs in this schedule.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:411-412`](../planning-artifacts/architecture.md)] — `enum FlightSystems { ReadInput, ApplyForces, IntegratePhysics }` example; Story 3.8 reuses the existing `ApplyForces` variant from 3.6/3.7.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:415`](../planning-artifacts/architecture.md)] — `.after(specific_function)` forbidden; SystemSet ordering only.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:454`](../planning-artifacts/architecture.md)] — Pattern Deviation Process; the `apply_*_acceleration` choice (vs. `apply_force` / `apply_torque` per AC literal text) is a documented deviation per AC #5.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:558-563`](../planning-artifacts/architecture.md)] — `src/flight/{mod,components,input,physics,camera}.rs` file structure prescription; Story 3.8 creates components.rs (per 3.7 dev-notes punt) and extends input.rs + physics.rs + mod.rs.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:646`](../planning-artifacts/architecture.md)] — `FlightPlugin` plugin-boundaries: owns "Thrusters, dampener, cockpit Camera3d"; Story 3.8 extends the dampener portion.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:677`](../planning-artifacts/architecture.md)] — FR5 → `src/flight/physics.rs` (toggleable damping coefficient) mapping.
- [Source: [`Cargo.toml:8-12`](../../Cargo.toml)] — pinned versions: bevy 0.18, avian3d 0.6, bevy_mod_outline 0.12, bevy_kira_audio 0.25, leafwing-input-manager 0.20.
- [Source: [`src/main.rs:36-47`](../../src/main.rs)] — current plugin-registration block; FlightPlugin already wired.
- [Source: [`src/state.rs:11-20`](../../src/state.rs)] — `GameState::Arena` variant; `run_if(in_state(GameState::Arena))` gate.
- [Source: [`src/flight/mod.rs:1-142`](../../src/flight/mod.rs)] — Story 3.7 baseline; 3.8 extends.
- [Source: [`src/flight/mod.rs:24-28`](../../src/flight/mod.rs)] — current `FlightSystems` enum (variants `Setup`, `ApplyForces`); 3.8 adds NO new variants.
- [Source: [`src/flight/mod.rs:45-66`](../../src/flight/mod.rs)] — current `FlightPlugin::build` body; 3.8 appends apply_dampener to the ApplyForces tuple + adds an Update block for toggle_dampener.
- [Source: [`src/flight/mod.rs:97-118`](../../src/flight/mod.rs)] — current `spawn_player_ship` component tuple; 3.8 appends `DampenerState::default()`.
- [Source: [`src/flight/input.rs:1-35`](../../src/flight/input.rs)] — Stories 3.6/3.7 FlightAction enum + default_input_map; 3.8 extends with ToggleDampener.
- [Source: [`src/flight/physics.rs:1-248`](../../src/flight/physics.rs)] — Stories 3.6/3.7 thrust + rotation helpers + systems + tests; 3.8 extends with dampener helper + 2 systems + 5 tests.
- [Source: [`src/tuning/config.rs:11-27`](../../src/tuning/config.rs)] — `TuningConfig` struct with `mouse_sensitivity` and `ship_torque_nm` (added 3.7); 3.8 adds `dampener_linear_strength` and `dampener_angular_strength` fields.
- [Source: [`src/tuning/config.rs:41-47`](../../src/tuning/config.rs)] — `default_mouse_sensitivity` / `default_ship_torque_nm` helpers (added 3.7); 3.8 adds 2 sister helpers.
- [Source: [`src/tuning/config.rs:49-62`](../../src/tuning/config.rs)] — `Default for TuningConfig` impl; 3.8 extends struct-literal.
- [Source: [`src/tuning/config.rs:96-141`](../../src/tuning/config.rs)] — 3 existing test functions; 3.8 extends in-place.
- [Source: [`src/pause/mod.rs:34-42`](../../src/pause/mod.rs)] — 3-system Update-schedule tuple precedent (`pause_on_focus_loss`, `resume_on_focus_gain`, `toggle_pause_on_escape`); Story 3.8's 3-system FixedUpdate ApplyForces tuple mirrors this style.
- [Source: [`src/pause/mod.rs:38-41`](../../src/pause/mod.rs)] — `toggle_pause_on_escape` Update-schedule loose-system precedent; Story 3.8's `toggle_dampener` mirrors this.
- [Source: [`src/pause/mod.rs:122-133`](../../src/pause/mod.rs)] — `pause_simulation_clocks` halts `Time<Virtual>` + `Time<Physics>`; relevant to AC #11 dampener-decay-halts-during-pause invariant.
- [Source: [`assets/config/tuning.ron:1-10`](../../assets/config/tuning.ron)] — current 10-line config; 3.8 appends 2 lines.
- [Source: [`_bmad-output/implementation-artifacts/3-7-flight-input-3-axis-rotation-pitch-yaw-roll.md`](./3-7-flight-input-3-axis-rotation-pitch-yaw-roll.md)] — predecessor story; Dev Agent Record + 11 ACs + Task 5 verification harness format. Story 3.8 mirrors directly.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:198`](./deferred-work.md)] — `GameState` lacks `Copy`; re-deferred (Story 3.8 doesn't legitimately touch state.rs).
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:222`](./deferred-work.md)] — NaN/inf guard on mouse axis or tuning scalars; the dampener `_strength` fields share the gap; consolidated entry covers all TuningConfig f32 fields.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:228`](./deferred-work.md)] — range/sign validation on tuning scalars; `dampener_*_strength = 0.0` (silent disable) and `< 0.0` (exponential amplification) are confusing failure modes; same gap as `mouse_sensitivity` / `ship_torque_nm`.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:232`](./deferred-work.md)] — touchpad-ergonomics-for-mouse-pitch/yaw context; orthogonal to dampener but motivates the dampener-on default (snap-aim recovery on touchpad).
- [Source: avian3d 0.6 — `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/forces/query_data.rs:204-214`] — `Forces::linear_velocity` / `angular_velocity` reads (world-space).
- [Source: avian3d 0.6 — `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/forces/query_data.rs:482-487, 539-544`] — `Forces::apply_linear_acceleration` / `apply_angular_acceleration` writes (bypass mass / inertia divisors).
- [Source: leafwing-input-manager 0.20 — `~/.cargo/registry/src/.../leafwing-input-manager-0.20.0/src/action_state/mod.rs:589-599`] — `ActionState::just_pressed` edge-triggered button-state contract.
- [Source: bevy 0.18 — `bevy::prelude::Update`] — Update schedule (per-render-frame).

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Opus 4.7, 1M context)

### Debug Log References

Local verification sweep (all logs in `/tmp/`):

| Command | Log file | `grep -cE 'warning:\|error:'` | Notes |
|---|---|---|---|
| `cargo check` | `/tmp/story-3-8-check.log` | 0 | 0.14s incremental cache hit |
| `cargo build` (debug) | `/tmp/story-3-8-build.log` | 0 | 3.12s; full debug rebuild after physics.rs touches |
| `cargo test` | `/tmp/story-3-8-test.log` | 0 | `test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Test count: **36** (= 30 pre-3.8 + 5 dampener tests in `flight/physics.rs` + 1 default-active test in `flight/components.rs`). |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-3-8-clippy.log` | 0 | 0.69s; clean |
| `cargo fmt --all -- --check` | `/tmp/story-3-8-fmt.log` | 0 | exit code 0; no fmt drift |
| `cargo build --release` | `/tmp/story-3-8-release.log` | 0 | 4m 18s (LTO=fat + codegen-units=1); within Story 3.7's 4m 21s benchmark — no regression |
| `cargo run` runtime smoke | `/tmp/story-3-8-run.log` | n/a | **Pending Till's runtime smoke** — 13 sub-bullets (a)–(l) per AC #11 + Task 7 |

**Cargo.lock delta check:** `git diff --stat Cargo.lock` shows **no changes** — no new external surface in 3.8 (DampenerState uses `bevy::prelude::Component`; Avian's `Forces`/acceleration APIs already in scope from 3.6/3.7). Confirms AC #11 expectation.

**File-size deltas (post-3.8 implementation, pre-smoke):**

| File | Lines | Delta vs Story 3.7 |
|---|---|---|
| `src/flight/components.rs` (NEW) | 27 | +27 from 0 (DampenerState + Default + 1 unit test); within the ~25-line target |
| `src/flight/input.rs` (modified) | 37 | +2 vs 35 baseline (1 enum variant + 1 binding row) |
| `src/flight/physics.rs` (modified) | 339 | +91 vs 248 baseline (use line + helper + 2 systems + 5 tests + module-doc edit); within the ~330-line target |
| `src/flight/mod.rs` (modified) | 153 | +11 vs 142 baseline (1 mod decl + 1 use line + 1 system in tuple + 1 Update block + 1 spawn-tuple line + module-doc edit); within the ~155-line target |
| `src/tuning/config.rs` (modified) | 159 | +18 vs 141 baseline (2 fields + 2 helpers + 2 Default literals + 6 in-place assertions + 1 ron-bytes literal extension); within the ~157-line target |
| `assets/config/tuning.ron` (modified) | 12 | +2 vs 10 baseline |

### Completion Notes List

- **AC #1** ✓ — `src/flight/input.rs` extended (35 → 37 lines). `FlightAction` gains 1 new variant `ToggleDampener` (default-buttonlike, no `#[actionlike(...)]` attribute). Final variant count: 11 (= 6 thrust + 2 axis pitch/yaw + 2 roll + 1 dampener-toggle). `default_input_map()` extended with `(FlightAction::ToggleDampener, KeyCode::KeyX)` row inside the `InputMap::new([...])` slice; the `.with_axis(...).with_axis(...)` chain remains intact after the closing `])`. Compiled clean on first try.

- **AC #2** ✓ — NEW FILE `src/flight/components.rs` (27 lines): module doc-comment "Marker / state components for FlightPlugin entities" (NO story-id reference per anti-pattern #1); `DampenerState { pub active: bool }` component with `#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]`; `impl Default` returns `Self { active: true }`; co-located `#[cfg(test)] mod tests` with `dampener_state_default_is_active` test passing. `pub mod components;` declared in `src/flight/mod.rs:4` (alphabetical: components → input → physics). `DampenerState::default()` appended to the `spawn_player_ship` component-tuple in `src/flight/mod.rs:122` AFTER `ActionState::<FlightAction>::default()` (13-component spawn tuple total, well within Bevy's 15-tuple bundle limit).

- **AC #3** ✓ — `TuningConfig` extended at `src/tuning/config.rs` with `pub dampener_linear_strength: f32` (`#[serde(default = "default_dampener_linear_strength")]`) and `pub dampener_angular_strength: f32` (`#[serde(default = "default_dampener_angular_strength")]`) AFTER the existing `ship_torque_nm` field. Two new helper functions added (`default_dampener_linear_strength() -> 2.0`, `default_dampener_angular_strength() -> 3.0`). `Default` impl extended with both new fields in struct-literal order. `assets/config/tuning.ron` extended with `dampener_linear_strength: 2.0,` and `dampener_angular_strength: 3.0,` after `ship_torque_nm: 80.0,`. All 3 existing `tuning::config::tests` tests extended in-place per spec; total of 6 new assertions across 3 tests. **Net new test functions in tuning: 0.**

- **AC #4** ✓ — `apply_dampener` system at `src/flight/physics.rs:163-186`. Signature: `(tuning_assets: Res<Assets<TuningConfig>>, tuning_handle: Res<TuningHandle>, mut ships: Query<(Forces, &DampenerState), With<PlayerShip>>)`. Body: cold-start `unwrap_or_default()` fallback for tuning, iterates the query (one match expected — same pattern as `apply_thrust` / `apply_torque`), reads `forces.linear_velocity()` and `forces.angular_velocity()` (both world-space), computes `(linear_accel, angular_accel) = dampener_acceleration(*state, lin_v, ang_v, tuning.dampener_linear_strength, tuning.dampener_angular_strength)`, applies via `forces.apply_linear_acceleration(linear_accel)` and `forces.apply_angular_acceleration(angular_accel)`. NO `info!`/`warn!` per-tick logs. `apply_*_acceleration` no-ops on Vec3::ZERO per Avian 0.6's internal early-return.

- **AC #5** ✓ — `dampener_acceleration` pure helper at `src/flight/physics.rs:147-161`. Signature: `(state: DampenerState, linear_velocity: Vec3, angular_velocity: Vec3, linear_strength: f32, angular_strength: f32) -> (Vec3, Vec3)`. Implementation:
  - Early-return `(Vec3::ZERO, Vec3::ZERO)` if `!state.active` — covers dampener-off case before any arithmetic.
  - When active: returns `(-linear_velocity * linear_strength, -angular_velocity * angular_strength)` — both contributions independent (linear vs. angular axes do not couple).
  - NO clamping or NaN guarding (consistent with unclamped `ship_local_thrust_vector` / `ship_local_torque_vector` precedent).
  - **PATTERN DEVIATION** documented inline in `apply_dampener` (`src/flight/physics.rs:168-170`): `// PATTERN DEVIATION: Avian's apply_*_acceleration bypasses the mass/inertia divisor; mathematically equivalent to applying force = -velocity * strength * mass per the AC, but skips a redundant query of ComputedMass / ComputedAngularInertia.`

- **AC #6** ✓ — `toggle_dampener` system at `src/flight/physics.rs:189-204`. Signature: `(mut ships: Query<(&ActionState<FlightAction>, &mut DampenerState), With<PlayerShip>>)`. Body iterates the query and, when `action_state.just_pressed(&FlightAction::ToggleDampener)`, flips `dampener.active = !dampener.active` AND emits exactly one `info!("dampener {}", if dampener.active { "engaged" } else { "disengaged" });` log per toggle. Registered in `Update` (`src/flight/mod.rs:67-70`) with `.run_if(in_state(GameState::Arena))` — input-handling cadence pattern matches `pause::toggle_pause_on_escape`. NO SystemSet placement. `apply_dampener` registered in `FixedUpdate::FlightSystems::ApplyForces` bundled with `apply_thrust` + `apply_torque` (3-system tuple at `src/flight/mod.rs:54-65`). NO new `FlightSystems` variants added.

- **AC #7** ✓ — Dampener convergence math validated: at `dampener_linear_strength = 2.0`: `exp(-6) ≈ 0.0025` → 0.25% of initial after 3s (well within "5%" bound); at `dampener_angular_strength = 3.0`: `exp(-9) ≈ 0.00012` → 0.012% → well within tolerance. Smoke verification (Till's task) confirms perceived convergence speed.

- **AC #8** ✓ — 5 new co-located unit tests added to `flight::physics::tests` at `src/flight/physics.rs:265-339`. Plus 1 new test in `flight::components::tests` (`dampener_state_default_is_active`). Tests:
  1. `dampener_inactive_returns_zero_acceleration` ✓ (early-return gate; non-zero velocities prove gate isn't bypassed)
  2. `dampener_active_zero_velocity_returns_zero_acceleration` ✓ (active-but-quiet case)
  3. `dampener_active_linear_velocity_returns_negative_proportional_acceleration` ✓ (linear-only contribution + sign + scalar product)
  4. `dampener_active_angular_velocity_returns_negative_proportional_acceleration` ✓ (angular-only contribution + sign + scalar product)
  5. `dampener_combines_linear_and_angular_independently` ✓ (vector-sum + independent strengths)
  6. `dampener_state_default_is_active` ✓ (in `flight/components.rs`; guards default-active invariant)
  All pass on first run. The 9 existing thrust + rotation tests in `flight/physics.rs` unchanged. Net new test functions: **6**. Total test count post-Task-4: 30 → 36.

- **AC #9, #10, #11** ✓ — Till ran the runtime smoke 2026-05-05 and confirmed "alles grün". All 13 sub-bullets (a)–(l) verified: X-press toggle + `info!` log; `just_pressed` edge semantic (no auto-repeat spam); convergence-on-release ≈ 0 within 3 s (linear + angular); tug-of-war terminal velocity bounded (~7.5 m/s linear, ~28°/s angular); dampener-OFF regression matches 3.6/3.7 baseline (unbounded drift/rotation); Esc-pause mid-decay preserves velocity → resume continues decay; X-press-during-Paused is a no-op (run_if gate); Cmd-Tab focus-loss preserves DampenerState; clean window-close (no panic). Post-runtime grep matched all expected counts (`entered Loading`=1, `entered MainMenu`=1, `entered Arena`≥1 = `spawned PlayerShip` count, `dampener (engaged|disengaged)`≥2, `panic|backtrace|FATAL`=0, `ambiguous.*camera.*order`=0, `ERROR.*avian|WARN.*Avian`=0). Pre-existing 3 documented WARNs (splash-cleanup, wgpu fragment-output, winit Skipped Destroyed) reappear unchanged — no fourth WARN.

- **AC #12** ✓ — All cargo subtasks pass per `feedback_full_build_output.md` discipline (6/6 complete: check, build, test, clippy, fmt, release — all 0 grep matches; release 4m 18s within 3.7 benchmark). Cargo.lock unchanged. Git status final delta matches AC #11 spec exactly:
  - **Modified:** `src/flight/input.rs`, `src/flight/mod.rs`, `src/flight/physics.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`, `_bmad-output/implementation-artifacts/sprint-status.yaml`
  - **Added (untracked):** `src/flight/components.rs`, `_bmad-output/implementation-artifacts/3-8-inertial-dampener-toggle.md`
  - **Pre-existing untracked (not 3.8-introduced):** `.claude/scheduled_tasks.lock` (unchanged from session start)
  - **NOT modified (per AC #11):** `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `src/state.rs`, `src/arena/**`, `src/pause/**`, `src/visual/**`, `src/ui/**`, `src/splash.rs`, `src/logging.rs`, `assets/strings/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`. ✓

**Deviations:**

1. **`apply_dampener` implementation uses `apply_linear_acceleration` / `apply_angular_acceleration` instead of `apply_force` + `apply_torque`** (PATTERN DEVIATION per architecture.md:454, pre-declared in story spec AC #5). Mathematically identical (`accel = force / mass = (-v * strength * mass) / mass = -v * strength`) but avoids redundant `ComputedMass` / `ComputedAngularInertia` queries. Documented inline in `apply_dampener`'s body. NO impact on AC compliance.

### File List

**Modified:**

- `src/flight/input.rs` (+2 net lines: 1 new enum variant `ToggleDampener` + 1 new button binding `KeyX → ToggleDampener`)
- `src/flight/physics.rs` (+91 net lines: `dampener_acceleration` helper; `apply_dampener` system with PATTERN DEVIATION inline doc; `toggle_dampener` system with single `info!` log; 5 new unit tests; 1 new use line for `DampenerState`; module doc-comment generalized from "Flight force/torque application" to "Flight force/torque/acceleration application" with FR5 added)
- `src/flight/mod.rs` (+11 net lines: 1 module decl `pub mod components;`; 1 use line `use crate::flight::components::DampenerState;`; 1 system `physics::apply_dampener` added to existing `add_systems(FixedUpdate, ...)` tuple; 1 new `add_systems(Update, ...)` block for `physics::toggle_dampener`; 1 new spawn-tuple line `DampenerState::default()`; module doc-comment generalized to mention "inertial dampener toggle")
- `src/tuning/config.rs` (+18 net lines: 2 new fields with serde-default annotations; 2 new helpers; 2 new lines in `Default` impl; 6 new assertions across 3 existing tests; 1 ron-bytes literal extension in deserialize test)
- `assets/config/tuning.ron` (+2 lines: `dampener_linear_strength: 2.0,` and `dampener_angular_strength: 3.0,`)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (3-8 status flip backlog → ready-for-dev → in-progress; final `→ review` flip pending Till's runtime smoke; `last_updated:` bumped to 2026-05-04)

**Added (untracked at commit time):**

- `src/flight/components.rs` (NEW FILE, 27 lines: DampenerState component + Default + 1 unit test)
- `_bmad-output/implementation-artifacts/3-8-inertial-dampener-toggle.md` (this file: tasks 1-6 fully [x]; task 7 cargo subtasks 5/6 [x] + release build + runtime smoke pending; tasks 8-9 partially executed; Dev Agent Record populated through cargo verification)

**NOT modified (validated via `git status --short`):**

- `Cargo.toml` (no dep added)
- `Cargo.lock` (no transitive-dep churn — no new external surface in 3.8)
- `src/main.rs` (FlightPlugin already registered; dampener wiring is per-plugin)
- `src/state.rs` (`Copy` derive on GameState remains deferred per deferred-work.md:198 — `run_if(in_state(...))` handles cloning internally; OnEnter/OnExit unaffected)
- `src/arena/**`, `src/pause/**`, `src/visual/**`, `src/ui/**`, `src/splash.rs`, `src/logging.rs`, `assets/strings/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` — all unchanged

### Review Findings

- [x] [Review][Patch] **World-space intent not stated in `apply_dampener` PATTERN DEVIATION comment** [`src/flight/physics.rs`] — fixed: added world-space clarification line to PATTERN DEVIATION comment — The existing comment explains the mass/inertia bypass but not the world-space vs. local-frame choice. A reader comparing `apply_thrust` (`apply_local_force`, ship-local) with `apply_dampener` (`apply_linear_acceleration`, world-space) has no inline cue for why the frames differ. Add one line: `// apply_*_acceleration operate in world-space, matching forces.linear/angular_velocity() — no local-frame transform needed (contrast: apply_thrust uses apply_local_force for ship-relative thrust).`

- [x] [Review][Defer] **AC 4(c) structural early-return missing from `apply_dampener` loop body** [`src/flight/physics.rs`] — deferred, functionally equivalent. Spec says "early-return if `!state.active`" should live in the loop body; implementation delegates to `dampener_acceleration()` helper instead. `apply_*_acceleration(Vec3::ZERO)` is a no-op per Avian (AC 4(e) acknowledges this), so behavior is correct. Spec itself contains internal tension between AC 4(c) and AC 4(e).

- [x] [Review][Defer] **Dampener strength values are implicitly timestep-coupled** [`src/flight/physics.rs`, `assets/config/tuning.ron`] — deferred, pre-existing pattern. `dampener_acceleration` returns `-v * strength` applied per FixedUpdate tick; effective decay rate is `strength / fixed_hz`. If `Time<Fixed>` is ever reconfigured, tuned values silently change feel. Same implicit coupling exists in `apply_thrust` / `apply_torque` — project-wide concern, not 3.8-specific.
