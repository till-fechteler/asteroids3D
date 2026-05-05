# Story 3.9: Weapon Firing + Projectile Ballistics

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player flying the Arena cockpit ship,
I want to fire ballistic projectiles from my ship while I hold the primary-fire trigger,
So that I have an offensive capability per FR9 — closing the input → action loop that Stories 3.5–3.8 prepared (cockpit + 6-DOF + rotation + dampener) and unblocking Story 3.10 (projectile ↔ asteroid damage) which forms the FR12 First Playable combat outcome.

## Acceptance Criteria

1. **Given** Story 3.9 introduces a NEW `CombatPlugin` per architecture.md:564-570 (no prior CombatPlugin in the project — `src/combat/` does not exist before 3.9; `cargo metadata` + `ls src/combat 2>/dev/null` returns empty pre-3.9)
   **When** the plugin scaffolding is authored
   **Then** four NEW files are created under `src/combat/`:
   - `src/combat/mod.rs` — declares `pub mod components; pub mod input; pub mod projectiles;` (alphabetical), defines `pub struct CombatPlugin`, defines `pub enum CombatSystems { Setup, Fire, Lifecycle }` (a `#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]`), and registers all systems via `impl Plugin for CombatPlugin`
   - `src/combat/input.rs` — defines `pub enum CombatAction { FirePrimary }` (a `#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]`) and `pub fn default_input_map() -> InputMap<CombatAction>` returning `InputMap::new([(CombatAction::FirePrimary, MouseButton::Left)])`
   - `src/combat/components.rs` — defines `pub struct Projectile { pub ttl: f32, pub damage: u32 }` (a `#[derive(Component, Debug, Clone, Copy)]`; NO Default impl — caller specifies values explicitly per AC #4) and `pub struct PrimaryWeaponCooldown { pub remaining: f32 }` (a `#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]`; the derived Default returns `remaining = 0.0` so the first FirePrimary press fires immediately)
   - `src/combat/projectiles.rs` — defines `pub fn projectile_initial_velocity(...)` pure helper, `pub fn fire_primary_weapon(...)` system, `pub fn tick_projectile_ttl(...)` system, plus a co-located `#[cfg(test)] mod tests` block with 4 helper-tests
   **And** `src/main.rs` is extended at line 8 with `mod combat;`, line 19 with `use combat::CombatPlugin;`, and one additional `.add_plugins(CombatPlugin)` call AFTER `.add_plugins(FlightPlugin)` and BEFORE `.add_plugins(PausePlugin)` (combat depends on PlayerShip; pause is orthogonal — Flight → Combat → Pause is the registration order)
   **And** `CombatPlugin::build` calls `app.add_plugins(InputManagerPlugin::<CombatAction>::default())` exactly once (each Actionlike type gets its own InputManagerPlugin instance per leafwing-0.20 plugin model)

2. **Given** the existing PlayerShip spawn tuple in `src/flight/mod.rs:108-122` already has 13 components and Bevy 0.18's tuple-bundle limit is 15, AND CombatPlugin owns combat-side components (architecture.md:658 "Plugin A never writes into Plugin B's internal" — the inverse is also disciplined: FlightPlugin should not import combat types)
   **When** combat components are attached to PlayerShip
   **Then** they are attached via a NEW system `attach_combat_to_player_ship` in `src/combat/projectiles.rs` (or `src/combat/mod.rs` — author's choice; placement in `mod.rs` next to the Plugin impl is preferred for visibility) registered in `OnEnter(GameState::Arena)` inside `CombatSystems::Setup`
   **And** the system signature is:
   ```rust
   pub fn attach_combat_to_player_ship(
       mut commands: Commands,
       ships: Query<Entity, With<PlayerShip>>,
   ) {
       for entity in &ships {
           commands.entity(entity).insert((
               default_input_map(),
               ActionState::<CombatAction>::default(),
               PrimaryWeaponCooldown::default(),
           ));
       }
   }
   ```
   **And** `CombatPlugin::build` configures the OnEnter ordering chain: `app.configure_sets(OnEnter(GameState::Arena), (FlightSystems::Setup, CombatSystems::Setup).chain())` — Bevy 0.18 composes this with `flight/mod.rs`'s existing `(ArenaSystems::Setup, FlightSystems::Setup).chain()` into the transitive DAG `Arena → Flight → Combat`
   **And** `src/flight/mod.rs` is **NOT** modified — no `combat::` imports added to flight, no growth of the spawn tuple, no breakage of the 13-component bundle. The 3-component combat insert lands at OnEnter via the new system, after PlayerShip already exists

3. **Given** `TuningConfig` (`src/tuning/config.rs`) is the project's single canonical gameplay-tuning struct (extended by Stories 2.3/2.4/3.6/3.7/3.8) and Story 3.8 last appended `dampener_angular_strength`
   **When** Story 3.9 extends it
   **Then** three new fields are added in this order, AFTER the existing `dampener_angular_strength` field:
   - `pub projectile_speed: f32` (default `120.0`)
   - `pub projectile_fire_rate_hz: f32` (default `4.0`)
   - `pub projectile_ttl_seconds: f32` (default `3.0`)
   **And** all three fields use the per-field `#[serde(default = "default_…")]` pattern matching the precedent set by Stories 2.4/3.6/3.7/3.8 (forward-compat — preserves deserialization of pre-3.9 tuning.ron snapshots)
   **And** three new top-level helpers are added alongside the existing default helpers: `fn default_projectile_speed() -> f32 { 120.0 }`, `fn default_projectile_fire_rate_hz() -> f32 { 4.0 }`, `fn default_projectile_ttl_seconds() -> f32 { 3.0 }`
   **And** `impl Default for TuningConfig` includes the three new fields in its struct-literal in the same order as the struct fields
   **And** `assets/config/tuning.ron` gains three new lines after `dampener_angular_strength: 3.0,`: `projectile_speed: 120.0,`, `projectile_fire_rate_hz: 4.0,`, `projectile_ttl_seconds: 3.0,` (RON 0.8 trailing-comma convention)
   **And** the existing 3 tests in `tuning::config::tests` are extended in-place — NO new test functions added:
   - `tuning_config_default_matches_ron_initial_values` gains three assertions: `assert_eq!(cfg.projectile_speed, 120.0);`, `assert_eq!(cfg.projectile_fire_rate_hz, 4.0);`, `assert_eq!(cfg.projectile_ttl_seconds, 3.0);`
   - `tuning_config_deserializes_from_ron_bytes` ron-bytes literal gains `, projectile_speed: 200.0, projectile_fire_rate_hz: 8.0, projectile_ttl_seconds: 5.0` and assertions `assert_eq!(cfg.projectile_speed, 200.0);`, `assert_eq!(cfg.projectile_fire_rate_hz, 8.0);`, `assert_eq!(cfg.projectile_ttl_seconds, 5.0);` (non-default values exercise the per-field deserializer; symmetric with the existing non-default literals from earlier stories)
   - `tuning_config_legacy_schema_uses_defaults_for_added_fields` ron-bytes literal is unchanged (the absent fields exercise the serde-default fallback) and gains assertions `assert_eq!(cfg.projectile_speed, 120.0);`, `assert_eq!(cfg.projectile_fire_rate_hz, 4.0);`, `assert_eq!(cfg.projectile_ttl_seconds, 3.0);`

4. **Given** `fire_primary_weapon` is the firing system that spawns `Projectile` entities at the player-ship muzzle, owns the per-ship rate-limit cooldown, and runs in `FixedUpdate` for deterministic spawn timing aligned with Avian's 60 Hz physics step
   **When** the system is authored in `src/combat/projectiles.rs`
   **Then** the system signature is:
   ```rust
   pub fn fire_primary_weapon(
       time: Res<Time>,
       tuning_assets: Res<Assets<TuningConfig>>,
       tuning_handle: Res<TuningHandle>,
       mut commands: Commands,
       mut meshes: ResMut<Assets<Mesh>>,
       mut materials: ResMut<Assets<ToonMaterial>>,
       mut ships: Query<
           (
               &ActionState<CombatAction>,
               &Transform,
               &LinearVelocity,
               &mut PrimaryWeaponCooldown,
           ),
           With<PlayerShip>,
       >,
   )
   ```
   **And** the system body (in this order):
   - (a) Extract `tuning` via the cold-start fallback `tuning_assets.get(tuning_handle.0.id()).cloned().unwrap_or_default()` pattern (matches `apply_thrust` / `apply_torque` / `apply_dampener` precedent — NO `warn!` per-tick on cold start; the spawn-time warn lives in `flight::spawn_player_ship`).
   - (b) Capture `let dt = time.delta_secs();` for the cooldown decrement.
   - (c) Iterate the query (one match expected — same one-or-zero pattern as flight systems).
   - (d) Decrement cooldown each tick: `cooldown.remaining = (cooldown.remaining - dt).max(0.0);` — clamp at zero to avoid drift into negative.
   - (e) Check fire condition: `if action.pressed(&CombatAction::FirePrimary) && cooldown.remaining <= 0.0`.
   - (f) Compute spawn data from helper: `let forward = *transform.forward();` (Bevy 0.18 `Transform::forward(&self) -> Dir3`; `*` deref via `Deref<Target = Vec3>` returns the unit world-space forward vector — `-local_z()` of the ship). Then `let spawn_pos = transform.translation + forward * PROJECTILE_SPAWN_OFFSET;` and `let velocity = projectile_initial_velocity(ship_velocity.0, forward, tuning.projectile_speed);`.
   - (g) Spawn the projectile entity with the 8-component tuple (AC #5).
   - (h) Reset cooldown: `cooldown.remaining = 1.0 / tuning.projectile_fire_rate_hz;` — at default 4.0 Hz this is 0.25 s = 15 fixed ticks at 60 Hz.
   - (i) Emit one `info!("fired projectile at velocity={:?} ttl={}", velocity, tuning.projectile_ttl_seconds);` per fire (acceptable: gated by cooldown to ≤ 4 Hz max, ~15× lower than the 60 Hz per-tick anti-pattern; same carve-out as `toggle_dampener`'s `info!` and `spawn_player_ship`'s spawn log).
   **And** the file declares two compile-time constants at the top (per architecture.md:357 — physics-clearance values, not gameplay tunables):
   ```rust
   /// Muzzle clearance: must exceed (ship_collider_radius=2.0 + projectile_radius=0.2) =
   /// 2.2 m so a freshly spawned projectile does not overlap the ship's collider.
   /// 3.0 m gives ~0.8 m of safety margin against ship motion within the spawn frame.
   const PROJECTILE_SPAWN_OFFSET: f32 = 3.0;
   /// Projectile mesh AND collider radius (matched for Story 3.10 collision-trustworthiness
   /// per the Story 3.3 precedent). Small-but-visible from cockpit view.
   const PROJECTILE_RADIUS: f32 = 0.2;
   ```

5. **Given** AC #4's spawn-data step (f) computes the projectile entity's tuple
   **When** the projectile entity is spawned via `commands.spawn((...))`
   **Then** the tuple is exactly 8 components in this order:
   ```rust
   commands.spawn((
       Projectile {
           ttl: tuning.projectile_ttl_seconds,
           damage: 1,
       },
       ArenaEntity,
       Mesh3d(projectile_mesh),
       MeshMaterial3d(projectile_material),
       Transform::from_translation(spawn_pos),
       RigidBody::Dynamic,
       Collider::sphere(PROJECTILE_RADIUS),
       LinearVelocity(velocity),
   ));
   ```
   **And** `projectile_mesh` is `meshes.add(Sphere::new(PROJECTILE_RADIUS).mesh().ico(2).expect("ico(2): subdivision=2 is within MAX_SUBDIVISIONS=80"))` — same `ico(2)` precedent as `arena/zone.rs:78-83`
   **And** `projectile_material` is `materials.add(ToonMaterial { tint: color_for(SemanticAccent::Neutral).into(), ..default() })` — `Neutral` tint per Story 3.5/3.9 deferral pattern (Story 4.5 owns the `SemanticAccent::PlayerOwned` re-tint pass; see deferred-work.md:204 PlayerShip retroactive entry — same disposition for projectiles)
   **And** the projectile carries the `ArenaEntity` marker so `cleanup_on_exit::<ArenaEntity>` (in `arena/mod.rs:32-36`) reaps any in-flight projectiles when the player exits Arena (e.g., to MainMenu or PostRun in later epics) without leaking entities
   **And** **NO** `OutlineVolume` is added to projectiles — outline polish on small fast-moving entities is deferred to Epic 10 polish (avoids visual noise + extra wgpu draw cost in 3.9's MVP scope)
   **And** **NO** `SemanticAccent` Component is attached to projectiles — only the material tint uses the palette helper. The `SemanticAccent::PlayerOwned` Component-on-entity wiring is Story 4.5's full sweep (deferred-work.md:162 + :204)
   **And** **NO** `CollisionLayers` are configured on the projectile — collision-filter setup is Story 3.10 AC #1 ("projectiles and asteroids share an Avian `CollisionLayers` group that allows contact"). 3.9 ships projectiles that physically interact with asteroids per Avian default (which produces brief bounces/contacts visible in smoke); the bounces become damage events in 3.10. **This is intentional and called out in Story 3.10's prerequisite analysis — DO NOT preemptively add CollisionLayers in 3.9.**

6. **Given** `projectile_initial_velocity` is the pure-logic helper symmetric to `ship_local_thrust_vector` (Story 3.6) / `ship_local_torque_vector` (Story 3.7) / `dampener_acceleration` (Story 3.8) — first-class unit-test target per architecture.md:353
   **When** authored in `src/combat/projectiles.rs` adjacent to `fire_primary_weapon`
   **Then** the signature is:
   ```rust
   /// World-space initial velocity for a freshly fired projectile. Combines the
   /// ship's current world velocity with `forward * projectile_speed` so a
   /// projectile fired while drifting inherits the ship's momentum (Newtonian
   /// muzzle-velocity composition; matches Epic 3 AC for "fires while drifting"
   /// case → world velocity = ship velocity + projectile_speed forward).
   ///
   /// `forward` is expected to be a unit vector (caller obtains via
   /// `*transform.forward()`); the helper performs no normalization or NaN
   /// guarding (consistent with the unclamped flight-physics helpers — input
   /// hardening lives in TuningConfig deserialization, deferred per
   /// deferred-work.md:222 + :228).
   pub fn projectile_initial_velocity(
       ship_velocity: Vec3,
       forward: Vec3,
       projectile_speed: f32,
   ) -> Vec3 {
       ship_velocity + forward * projectile_speed
   }
   ```
   **And** the helper performs NO clamping, NO normalization, and NO NaN guarding (consistent with the unclamped flight-physics helper precedent)
   **And** **NO PATTERN DEVIATION** is needed — the formula matches the Epic AC literally ("initial `LinearVelocity = ship_linear_velocity + ship_forward_vector * projectile_speed`")

7. **Given** `tick_projectile_ttl` is the projectile-lifecycle system that decrements `Projectile.ttl` each FixedUpdate tick and despawns expired projectiles
   **When** authored in `src/combat/projectiles.rs` BELOW `fire_primary_weapon`
   **Then** the system signature is:
   ```rust
   pub fn tick_projectile_ttl(
       time: Res<Time>,
       mut commands: Commands,
       mut projectiles: Query<(Entity, &mut Projectile)>,
   )
   ```
   **And** the system body iterates the query, decrements `ttl -= time.delta_secs();`, and `commands.entity(entity).despawn();` when `ttl <= 0.0` (single system handles tick + despawn — symmetric with `dampener_acceleration`'s single-system early-return rather than splitting)
   **And** **NO** `info!` / `warn!` per tick (60 Hz spam — same discipline as flight systems)
   **And** Bevy's `Commands::despawn()` is the canonical despawn API for entities created via `commands.spawn(...)` in the SAME plugin (no parent-child hierarchy on projectiles → no `try_despawn` race like the splash-cleanup entry at deferred-work.md:75-76)

8. **Given** `CombatPlugin::build` registers all systems
   **When** the plugin scaffolds its schedule
   **Then** the registration block is exactly:
   ```rust
   impl Plugin for CombatPlugin {
       fn build(&self, app: &mut App) {
           // OnEnter ordering: PlayerShip must exist before combat insertion.
           app.configure_sets(
               OnEnter(GameState::Arena),
               (FlightSystems::Setup, CombatSystems::Setup).chain(),
           );
           app.add_plugins(InputManagerPlugin::<CombatAction>::default());
           app.configure_sets(
               FixedUpdate,
               (CombatSystems::Fire, CombatSystems::Lifecycle).chain(),
           );
           app.add_systems(
               OnEnter(GameState::Arena),
               attach_combat_to_player_ship.in_set(CombatSystems::Setup),
           );
           app.add_systems(
               FixedUpdate,
               (
                   projectiles::fire_primary_weapon
                       .in_set(CombatSystems::Fire)
                       .run_if(in_state(GameState::Arena)),
                   projectiles::tick_projectile_ttl
                       .in_set(CombatSystems::Lifecycle)
                       .run_if(in_state(GameState::Arena)),
               ),
           );
       }
   }
   ```
   **And** the chain `Fire → Lifecycle` ensures freshly spawned projectiles get a full TTL on their spawn frame (Lifecycle runs after Fire — projectile spawned this tick has `ttl = projectile_ttl_seconds` un-decremented, then ticks down on subsequent ticks)
   **And** **NO** `Update` schedule registration — combat-firing is fully physics-coupled (deterministic projectile timing); no input handler at render-frame cadence (contrast: `toggle_dampener` is in Update because state-flip semantics don't need physics determinism)

9. **Given** the unit-test surface added by Story 3.9
   **When** tests are authored
   **Then** `combat/projectiles.rs` gains 4 co-located test functions in a `#[cfg(test)] mod tests` block at the bottom of the file (plain primitives — no `ActionState` setup, no Time setup):
   - `projectile_initial_velocity_stationary_ship_returns_speed_along_forward` — `(ZERO, NEG_Z, 120.0)` → `(0,0,-120)` (forward = NEG_Z is Bevy convention; matches Story 3.6's thrust convention test)
   - `projectile_initial_velocity_drifting_ship_inherits_ship_momentum` — `((0,0,-30), NEG_Z, 120.0)` → `(0,0,-150)` (verifies the FR9 / Epic 3 AC #5 "world velocity equals 30 + projectile_speed forward" case literally)
   - `projectile_initial_velocity_strafing_ship_combines_orthogonal_motion` — `((20,0,0), NEG_Z, 120.0)` → `(20,0,-120)` (verifies orthogonal-axes case: lateral ship motion does NOT bend the forward projectile speed; both contributions sum vector-wise)
   - `projectile_initial_velocity_zero_speed_returns_ship_velocity_unchanged` — `((5,10,-2), NEG_Z, 0.0)` → `(5,10,-2)` (degenerate-but-defined edge: speed=0 means projectile inherits ONLY ship velocity; useful regression check if a future story adds a "muzzle brake" mechanic that scales speed toward zero)
   **And** `combat/components.rs` gains 1 test function in a `#[cfg(test)] mod tests` block:
   - `primary_weapon_cooldown_default_is_zero` — `assert_eq!(PrimaryWeaponCooldown::default().remaining, 0.0)` — guards the instant-fire-on-first-press invariant against future accidental re-defaulting
   **And** **NO** test function is added to `combat/input.rs` (binding-map content is configuration data trivially correct by inspection — same precedent as `flight/input.rs` from Stories 3.6/3.7/3.8)
   **And** **NO** test function is added to `combat/mod.rs` (Plugin scaffolding is integration-test-shaped — would require MinimalPlugins + state setup; deferred per architecture.md:354)
   **And** Story 3.9 adds **5 net new test functions** (4 in `combat/projectiles.rs` + 1 in `combat/components.rs`) — net post-3.9 test count: **41** (= 36 from end of 3.8 + 4 helper + 1 component default). AC #11 enforces N = 41 at verification time.

10. **Given** Story 3.9's runtime smoke is the integration test for `fire_primary_weapon` + `tick_projectile_ttl` + `attach_combat_to_player_ship` (per architecture.md:354 integration tests deferred post-M3, and the `apply_*` precedent of Stories 3.6/3.7/3.8 — full-app smokes verify the wired-up behavior)
    **When** the dev runs the runtime smoke
    **Then** the dev verifies all of:
    - (a) **Press-and-release LMB** → exactly one `info!("fired projectile ...")` line in `/tmp/story-3-9-run.log`. A small grey sphere is visible in front of the cockpit, departing forward at ≈120 m/s (it crosses the ~50m near-asteroid distance in <0.5 s).
    - (b) **Hold LMB for ~2 seconds** → ~8 `info!("fired projectile ...")` lines (rate = 4 Hz × 2 s = 8 ± 1 due to button-down/release timing). Verify via `grep -c 'fired projectile' /tmp/story-3-9-run.log` ≈ 8.
    - (c) **Single-tap LMB → wait 0.1 s → tap again** → exactly 1 fire (the second tap hits the cooldown gate at remaining ≈ 0.15 s). Tap again after 0.3 s → second fire (cooldown elapsed). Verifies the rate-limit rather than the auto-repeat.
    - (d) **Hold LMB for 5 seconds** → ~20 fires (5 s × 4 Hz). After release, no further fires; cooldown bleeds to 0 within 0.25 s.
    - (e) **Fire while stationary (no W pressed)** → projectile world LinearVelocity ≈ `(0, 0, -120)` ± Avian-integration tolerance (10% per Story 3.6 AC #3). Smoke is visual ("projectile flies straight forward at the same speed every shot") rather than numeric — Avian's exposed velocity is verified by helper unit tests instead.
    - (f) **Fire while drifting forward at ~30 m/s (W held with dampener-OFF for 2 s, then release W and fire)** → projectile world LinearVelocity ≈ `(0, 0, -150)` (ship's `~-30 m/s` plus `projectile_speed = 120` forward). Visual: projectile lead-times visibly shorten because it's faster relative to world.
    - (g) **Fire while drifting backward at ~30 m/s (S held for 2 s, then fire)** → projectile world LinearVelocity ≈ `(0, 0, -90)` (ship at `+30 m/s` reverse, projectile speed 120 forward → net -90). Visual: projectile is slower forward but still moves forward (not backward — the projectile_speed dominates).
    - (h) **Fire at angle (Q rolled to ~45°, then fire)** → projectile flies along ship's CURRENT roll-rotated forward axis (not the world -Z axis). Verifies `transform.forward()` reads ship's instantaneous orientation. Visual: pitch the ship up 30° via mouse, fire — projectile flies upward-forward.
    - (i) **Wait 3 s (default `projectile_ttl_seconds = 3.0`) after firing one projectile** → projectile visibly disappears (despawn). NO log on despawn (per AC #7's "no per-tick log" discipline). Verify via memory leak proxy: `grep -c 'fired projectile'` rises with each shot, but memory inspection (Avian dev panel — deferred to M2 per architecture.md:296; for 3.9 just trust the ttl→despawn logic via the unit test).
    - (j) **Esc-pause mid-flight** → projectile freezes in space (Avian's `Time<Physics>` halt per pause::pause_simulation_clocks). TTL counter halts (FixedUpdate doesn't advance during pause). Resume via Esc → projectile resumes motion AND TTL counting from the paused state. No "TTL skip" / "frozen-then-snap" artifacts.
    - (k) **Pause WHILE holding LMB** → no new fires during pause (Fire system gated by `run_if(in_state(GameState::Arena))`; pause sets state to Paused). On resume → fires resume normally. No queued / lost shots.
    - (l) **Cmd-Tab focus-loss + regain mid-flight** → identical to (j) (focus-loss path also routes through pause). Cmd-Tab back → fires & TTL resume.
    - (m) **Quit cleanly** during flight (window-close while projectiles in-flight). No panic; ArenaEntity cleanup despawns all in-flight projectiles + asteroids + ship + light when state exits Arena (via the existing `cleanup_on_exit::<ArenaEntity>` system).
    - (n) **Asteroid bounce REGRESSION CHECK (intentional 3.9 limitation):** projectiles fired toward an asteroid will bounce off (Avian default contact response — no CollisionLayers configured per AC #5). This is **NOT a defect** — Story 3.10 introduces CollisionLayers + ProjectileHitAsteroid event + damage routing. Smoke-confirms that the bounce produces no panic, no warn-spam, no game-state corruption, AND projectiles eventually time out via TTL even after bouncing.

11. **Given** the post-3.8 source baseline (test count = 36 per `cargo test` 2026-05-05 measurement; `cargo build --release` 0 warnings; `src/flight/components.rs` = 29 lines; `src/flight/input.rs` = 37 lines; `src/flight/mod.rs` = 153 lines; `src/flight/physics.rs` = 403 lines; `src/tuning/config.rs` = 161 lines; `assets/config/tuning.ron` = 12 lines; NO `src/combat/` directory exists)
    **When** Story 3.9 verification runs locally (per `feedback_full_build_output.md` discipline — exit-0 + tail is NOT proof; grep for `warning:|error:` per command, capture each to `/tmp/story-3-9-<command>.log`)
    **Then** **all six** of `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --release` produce **0** lines matching `grep -cE 'warning:|error:'` per their respective full-output logs
    **And** `cargo test` summary line reads `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N = 41** (= 36 baseline + 4 new in `combat/projectiles.rs` + 1 new in `combat/components.rs`; the +0 deltas in `tuning/config.rs` per AC #3 are expected)
    **And** the runtime smoke (Task 8 below) verifies all of (a)–(n) per AC #10
    **And** `/tmp/story-3-9-run.log` contains: 1 of `entered Loading`, 1 of `entered MainMenu`, ≥ 1 of `entered Arena`, ≥ 1 of `spawned PlayerShip` (matches `entered Arena` count per Story 3.6 AC #9 false-positive), ≥ 1 of `fired projectile` (at least one shot exercised in smoke), 0 of `panic|backtrace|FATAL`, 0 of `ambiguous.*camera.*order` (Story 3.5 regression check), 0 of `ERROR.*avian|WARN.*Avian`
    **And** `git status --short` final set is **exactly**: `?? src/combat/` (new directory containing 4 new files), `M src/main.rs` (M — `mod combat;` + use line + add_plugins call), `M src/tuning/config.rs` (M — 3 new fields + 3 new helpers + Default extended + 3 in-place test extensions), `M assets/config/tuning.ron` (M — 3 new lines), `M _bmad-output/implementation-artifacts/sprint-status.yaml` (M — bookkeeping), `?? _bmad-output/implementation-artifacts/3-9-weapon-firing-projectile-ballistics.md` (?? — NEW FILE: this story spec; ?? at story-creation time, becomes M after dev flips Status), and `M _bmad-output/implementation-artifacts/deferred-work.md` ONLY IF a new entry surfaces during impl (none anticipated; AC #5's CollisionLayers gap is Story 3.10's setup not a deferral; SemanticAccent::PlayerOwned re-tint is already covered by deferred-work.md:204); **NO** entries under `Cargo.toml` (no dep added — every API used is already in scope: `Component` from `bevy::prelude`; `RigidBody`/`Collider`/`LinearVelocity` from `avian3d::prelude` already imported by Stories 3.5/3.6; `MouseButton`/`InputMap`/`ActionState` from `leafwing_input_manager::prelude` already imported by Stories 3.6/3.7/3.8), `Cargo.lock` (no transitive-dep churn), `src/flight/**` (per AC #2 — no flight changes), `src/state.rs` (no GameState changes), `src/arena/**`, `src/pause/**`, `src/visual/**`, `src/ui/**`, `src/splash.rs`, `src/logging.rs`, `assets/strings/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`

12. **Given** Bevy 0.18's `Time<Fixed>` clock (Story 3.4's `pause_simulation_clocks` halts both `Time<Virtual>` and `Time<Physics>`) AND Story 3.7/3.8's confirmed pause-cycle invariants
    **When** Story 3.9 runs in the pause-cycle
    **Then** the dev verifies that both `fire_primary_weapon` and `tick_projectile_ttl` halt during pause (per AC #10 (j)–(l)) — they're in `FixedUpdate` which doesn't advance when `Time<Fixed>` is paused
    **And** the dev verifies that PlayerShip's `PrimaryWeaponCooldown` is preserved across pause cycles (component lives on the persistent PlayerShip; pause doesn't despawn — re-confirms Story 3.6/3.7/3.8 invariants)
    **And** the dev verifies that LMB-press-during-Paused does NOT queue a fire on resume (`fire_primary_weapon`'s `run_if(in_state(GameState::Arena))` gate suppresses the system entirely; `pressed` state at resume reflects current button state, not historical presses)

## Tasks / Subtasks

- [x] **Task 1: Create `src/combat/components.rs` — Projectile + PrimaryWeaponCooldown + 1 test** (AC: #1, #9)
  - [x] Create directory `src/combat/` (verify it does not exist beforehand: `ls src/combat 2>/dev/null` → expected `No such file`).
  - [x] Create new file `src/combat/components.rs`. Author per AC #1 + #9 verbatim:
    ```rust
    //! Marker / state components for CombatPlugin entities.
    //! Initial occupants are Projectile (FR9 in-flight projectile state) and
    //! PrimaryWeaponCooldown (per-ship rate-limit state). Future stories add
    //! HullHP / ShieldHP / Weapon archetypes per architecture.md:566.

    use bevy::prelude::*;

    /// In-flight projectile state. `ttl` is the remaining seconds before despawn
    /// (decremented by `tick_projectile_ttl`); `damage` is the hit-point quantity
    /// applied by Story 3.10's damage system. Default damage in Story 3.9 is 1
    /// (single-hit asteroid destruction); future weapon archetypes (Story 4.4)
    /// vary it per archetype.
    #[derive(Component, Debug, Clone, Copy)]
    pub struct Projectile {
        pub ttl: f32,
        pub damage: u32,
    }

    /// Per-ship primary-weapon rate-limit state. `remaining` counts seconds
    /// until the next shot is permitted. Default `0.0` so the first
    /// `FirePrimary` press fires instantly.
    #[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
    pub struct PrimaryWeaponCooldown {
        pub remaining: f32,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn primary_weapon_cooldown_default_is_zero() {
            assert_eq!(PrimaryWeaponCooldown::default().remaining, 0.0);
        }
    }
    ```
  - [x] **Verify post-edit:** `cargo check` will fail at this point because `src/combat/` is not yet declared as a module in `main.rs` and `src/combat/mod.rs` does not yet exist. This is expected; intermediate check passes after Task 4 lands the module-level wiring. Defer the `cargo check` PASS expectation to the end of Task 4. **Note for incremental verification:** if running `cargo check` after Task 1 alone, the error will be either "file not found for module" or absent (if `mod combat;` not yet in main.rs). Both are expected; ignore until Task 4. (Confirmed: rust-analyzer flagged `unlinked-file` until Task 5 wired `mod combat;`.)

- [x] **Task 2: Create `src/combat/input.rs` — CombatAction enum + KeyL binding** (AC: #1)
  - [x] Create new file `src/combat/input.rs`. Author per AC #1 verbatim:
    ```rust
    //! CombatAction enum + default mouse binding (FR9 primary fire).

    use bevy::prelude::*;
    use leafwing_input_manager::prelude::*;

    #[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
    pub enum CombatAction {
        FirePrimary,
    }

    pub fn default_input_map() -> InputMap<CombatAction> {
        InputMap::new([(CombatAction::FirePrimary, MouseButton::Left)])
    }
    ```
  - [x] **No tests added to `input.rs`** — same Stories 3.6/3.7/3.8 reasoning: binding-map content is configuration data trivially correct by inspection, runtime-verified via Task 8's smoke.
  - [x] **Verify post-edit:** same caveat as Task 1 — `cargo check` will fail until Task 4. The file content compiles in isolation per Bevy 0.18 + leafwing-0.20 surfaces.

- [x] **Task 3: Create `src/combat/projectiles.rs` — projectile_initial_velocity helper + fire_primary_weapon + tick_projectile_ttl + attach_combat_to_player_ship + 4 unit tests** (AC: #2, #4, #5, #6, #7, #9)
  - [x] Create new file `src/combat/projectiles.rs`. Author the file in this order:
    1. Module doc-comment + use block:
    ```rust
    //! Projectile firing + ballistics + lifecycle (FR9). Owns:
    //!   - `attach_combat_to_player_ship`: OnEnter(Arena) — adds combat input
    //!     map + cooldown to the PlayerShip after FlightSystems::Setup.
    //!   - `fire_primary_weapon`: FixedUpdate — reads CombatAction, gates by
    //!     PrimaryWeaponCooldown, spawns Projectile entities at ship muzzle.
    //!   - `tick_projectile_ttl`: FixedUpdate — decrements ttl, despawns
    //!     expired projectiles.
    //!
    //! Story 3.10 adds CollisionLayers + ProjectileHitAsteroid event +
    //! damage routing on top of the entity bundle established here.

    use avian3d::prelude::{Collider, LinearVelocity, RigidBody};
    use bevy::prelude::*;
    use leafwing_input_manager::prelude::*;

    use crate::combat::components::{PrimaryWeaponCooldown, Projectile};
    use crate::combat::input::{CombatAction, default_input_map};
    use crate::flight::PlayerShip;
    use crate::tuning::TuningHandle;
    use crate::tuning::config::TuningConfig;
    use crate::visual::palette::{SemanticAccent, color_for};
    use crate::visual::toon_material::ToonMaterial;
    ```
    2. Compile-time constants per AC #4 final paragraph:
    ```rust
    /// Muzzle clearance: must exceed (ship_collider_radius=2.0 + projectile_radius=0.2) =
    /// 2.2 m so a freshly spawned projectile does not overlap the ship's collider.
    /// 3.0 m gives ~0.8 m of safety margin against ship motion within the spawn frame.
    const PROJECTILE_SPAWN_OFFSET: f32 = 3.0;

    /// Projectile mesh AND collider radius (matched for collision-trustworthiness
    /// per the Story 3.3 mesh==collider precedent). Small-but-visible from cockpit.
    const PROJECTILE_RADIUS: f32 = 0.2;
    ```
    3. `attach_combat_to_player_ship` system per AC #2 verbatim.
    4. `projectile_initial_velocity` helper per AC #6 verbatim.
    5. `fire_primary_weapon` system per AC #4 verbatim.
    6. `tick_projectile_ttl` system per AC #7 verbatim.
    7. `#[cfg(test)] mod tests { ... }` block with the 4 helper tests per AC #9:
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn projectile_initial_velocity_stationary_ship_returns_speed_along_forward() {
            let v = projectile_initial_velocity(Vec3::ZERO, Vec3::NEG_Z, 120.0);
            assert!(
                (v - Vec3::new(0.0, 0.0, -120.0)).length() < 1e-5,
                "got {:?}",
                v
            );
        }

        #[test]
        fn projectile_initial_velocity_drifting_ship_inherits_ship_momentum() {
            // Ship at (0,0,-30), forward=NEG_Z, speed=120 → world velocity (0,0,-150).
            let v = projectile_initial_velocity(Vec3::new(0.0, 0.0, -30.0), Vec3::NEG_Z, 120.0);
            assert!(
                (v - Vec3::new(0.0, 0.0, -150.0)).length() < 1e-5,
                "got {:?}",
                v
            );
        }

        #[test]
        fn projectile_initial_velocity_strafing_ship_combines_orthogonal_motion() {
            // Lateral ship motion at (20,0,0) does not bend forward speed; both sum.
            let v = projectile_initial_velocity(Vec3::new(20.0, 0.0, 0.0), Vec3::NEG_Z, 120.0);
            assert!(
                (v - Vec3::new(20.0, 0.0, -120.0)).length() < 1e-5,
                "got {:?}",
                v
            );
        }

        #[test]
        fn projectile_initial_velocity_zero_speed_returns_ship_velocity_unchanged() {
            // speed=0 → projectile inherits ONLY ship velocity (degenerate-but-defined).
            let v = projectile_initial_velocity(Vec3::new(5.0, 10.0, -2.0), Vec3::NEG_Z, 0.0);
            assert!(
                (v - Vec3::new(5.0, 10.0, -2.0)).length() < 1e-5,
                "got {:?}",
                v
            );
        }
    }
    ```
  - [x] **Verify post-edit:** same caveat — `cargo check` waits on Task 4.

- [x] **Task 4: Create `src/combat/mod.rs` — CombatPlugin + CombatSystems + module wiring** (AC: #1, #2, #8)
  - [x] Create new file `src/combat/mod.rs`. Author per AC #1, #2, #8 verbatim:
    ```rust
    //! CombatPlugin — owns weapon firing + projectile ballistics + projectile
    //! lifecycle (FR9). Story 3.9 introduces the plugin; Story 3.10 adds
    //! collision-driven damage events on top.

    pub mod components;
    pub mod input;
    pub mod projectiles;

    use bevy::prelude::*;
    use leafwing_input_manager::prelude::*;

    use crate::flight::FlightSystems;
    use crate::state::GameState;

    use crate::combat::input::CombatAction;

    pub struct CombatPlugin;

    #[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum CombatSystems {
        Setup,
        Fire,
        Lifecycle,
    }

    impl Plugin for CombatPlugin {
        fn build(&self, app: &mut App) {
            // OnEnter ordering: PlayerShip must exist before combat insertion.
            app.configure_sets(
                OnEnter(GameState::Arena),
                (FlightSystems::Setup, CombatSystems::Setup).chain(),
            );
            app.add_plugins(InputManagerPlugin::<CombatAction>::default());
            app.configure_sets(
                FixedUpdate,
                (CombatSystems::Fire, CombatSystems::Lifecycle).chain(),
            );
            app.add_systems(
                OnEnter(GameState::Arena),
                projectiles::attach_combat_to_player_ship.in_set(CombatSystems::Setup),
            );
            app.add_systems(
                FixedUpdate,
                (
                    projectiles::fire_primary_weapon
                        .in_set(CombatSystems::Fire)
                        .run_if(in_state(GameState::Arena)),
                    projectiles::tick_projectile_ttl
                        .in_set(CombatSystems::Lifecycle)
                        .run_if(in_state(GameState::Arena)),
                ),
            );
        }
    }
    ```
  - [x] **Verify post-edit:** STILL FAILS until Task 5 lands `mod combat;` in `main.rs`. After Task 5: `cargo check` must produce **0 warnings, 0 errors**. (Confirmed.)

- [x] **Task 5: Wire `CombatPlugin` into `src/main.rs`** (AC: #1)
  - [x] In `src/main.rs`, add `mod combat;` after `mod arena;` and before `mod flight;` — alphabetical order across modules: `arena`, `combat`, `flight`, `logging`, `pause`, `splash`, `state`, `tuning`, `ui`, `visual`.
  - [x] In `src/main.rs`, add `use combat::CombatPlugin;` after `use arena::ArenaPlugin;` and before `use flight::FlightPlugin;` (alphabetical).
  - [x] In `src/main.rs::main()`, add one `.add_plugins(CombatPlugin)` call AFTER `.add_plugins(FlightPlugin)` and BEFORE `.add_plugins(PausePlugin)`. Final plugin chain order:
    ```rust
    .add_plugins(TuningPlugin)
    .add_plugins(VisualPlugin)
    .add_plugins(UiPlugin)
    .add_plugins(ArenaPlugin)
    .add_plugins(FlightPlugin)
    .add_plugins(CombatPlugin)
    .add_plugins(PausePlugin)
    ```
  - [x] **Verify post-edit:** `cargo check` produces **0 warnings, 0 errors**. (Confirmed.)

- [x] **Task 6: Extend `src/tuning/config.rs` — `projectile_speed` + `projectile_fire_rate_hz` + `projectile_ttl_seconds` fields + 3 helpers + Default impl + tuning.ron + 3 in-place test extensions** (AC: #3)
  - [x] In `src/tuning/config.rs`, add three `pub` fields to the `TuningConfig` struct, AFTER the `dampener_angular_strength` field (insert-at-end ordering per Stories 2.4/3.6/3.7/3.8 precedent). Annotate each with its own `#[serde(default = "default_…")]`:
    ```rust
    #[serde(default = "default_projectile_speed")]
    pub projectile_speed: f32,
    #[serde(default = "default_projectile_fire_rate_hz")]
    pub projectile_fire_rate_hz: f32,
    #[serde(default = "default_projectile_ttl_seconds")]
    pub projectile_ttl_seconds: f32,
    ```
  - [x] Add the three helper functions alongside the existing `default_dampener_*` helpers:
    ```rust
    fn default_projectile_speed() -> f32 {
        120.0
    }

    fn default_projectile_fire_rate_hz() -> f32 {
        4.0
    }

    fn default_projectile_ttl_seconds() -> f32 {
        3.0
    }
    ```
  - [x] Update `impl Default for TuningConfig`'s struct-literal: append `projectile_speed: default_projectile_speed(),`, `projectile_fire_rate_hz: default_projectile_fire_rate_hz(),`, `projectile_ttl_seconds: default_projectile_ttl_seconds(),` as the last three fields (in struct-field order).
  - [x] In `assets/config/tuning.ron`, append three new lines AFTER `dampener_angular_strength: 3.0,` and BEFORE the closing `)` paren:
    ```
    projectile_speed: 120.0,
    projectile_fire_rate_hz: 4.0,
    projectile_ttl_seconds: 3.0,
    ```
    Trailing commas correct per RON-0.8 convention. Final file size: 15 lines (was 12).
  - [x] Extend the existing 3 tests in-place per AC #3:
    - `tuning_config_default_matches_ron_initial_values`: add as the last three assertions:
      ```rust
      assert_eq!(cfg.projectile_speed, 120.0);
      assert_eq!(cfg.projectile_fire_rate_hz, 4.0);
      assert_eq!(cfg.projectile_ttl_seconds, 3.0);
      ```
    - `tuning_config_deserializes_from_ron_bytes`: edit the bytes literal to add `, projectile_speed: 200.0, projectile_fire_rate_hz: 8.0, projectile_ttl_seconds: 5.0` BEFORE the closing `)`. Add as the last three assertions:
      ```rust
      assert_eq!(cfg.projectile_speed, 200.0);
      assert_eq!(cfg.projectile_fire_rate_hz, 8.0);
      assert_eq!(cfg.projectile_ttl_seconds, 5.0);
      ```
    - `tuning_config_legacy_schema_uses_defaults_for_added_fields`: bytes literal is unchanged (the absent fields exercise the serde-default fallback). Add as the last three assertions:
      ```rust
      assert_eq!(cfg.projectile_speed, 120.0);
      assert_eq!(cfg.projectile_fire_rate_hz, 4.0);
      assert_eq!(cfg.projectile_ttl_seconds, 3.0);
      ```
  - [x] **Verify post-edit:** `cargo test` produces `test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Project test count: **41**.

- [x] **Task 7: Local verification sweep — full `feedback_full_build_output.md` discipline** (AC: #11)

  Per Till's memory `feedback_full_build_output.md`: `cargo check` exit-0 + tail is NOT proof of correctness. Capture each command's full output to a log file, then grep for `warning:|error:` and confirm count is **0**.

  - [x] `cargo check 2>&1 | tee /tmp/story-3-9-check.log` — grep **0** (0.30s incremental after touch).
  - [x] `cargo build 2>&1 | tee /tmp/story-3-9-build.log` — grep **0** (3.37s).
  - [x] `cargo test 2>&1 | tee /tmp/story-3-9-test.log` — grep **0**; summary: `test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Test count: 41 = 36 baseline + 4 from `combat/projectiles.rs` + 1 from `combat/components.rs`.
  - [x] `cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-3-9-clippy.log` — grep **0** (0.84s).
  - [x] `cargo fmt --all -- --check 2>&1 | tee /tmp/story-3-9-fmt.log` — initial run flagged one long-line drift in `src/combat/projectiles.rs` (line 96 `let velocity = projectile_initial_velocity(...)`). Applied via `cargo fmt --all`; rerun produced exit 0; no drift remaining. **Anti-pattern note for future stories:** rustfmt's max line width applies inside system-body `let` bindings; helper-call expressions that exceed the width get split across lines. Anticipate during authoring.
  - [x] `cargo build --release 2>&1 | tee /tmp/story-3-9-release.log` — grep **0** (4m 23s; comparable to Story 3.8's 4m 18s — no regression). The `#[allow(dead_code, reason = "...")]` annotation on `Projectile` (per AC #11 anticipation) prevents the rustc dead_code warning that would otherwise fire on `damage` until Story 3.10 reads it.
  - [x] **Cargo.lock delta check:** `git diff --stat Cargo.lock` shows no changes.
  - [x] **Cargo.toml delta check:** `git diff --stat Cargo.toml` shows no changes.

- [x] **Task 8: Runtime smoke — execute and document AC #10 verifications** (AC: #10, #11, #12)
  - [x] `RUST_LOG=info,wgpu=warn,naga=warn,avian3d=info cargo run 2>&1 | tee /tmp/story-3-9-run.log` — let the game reach Arena.
  - [x] Exercise per AC #10 (a)–(n) sequence. **Till's smoke 2026-05-05: alles grün** for all (a)–(n) on the first sweep, EXCEPT a pause-roundtrip respawn defect (PlayerShip + projectiles snapping back to spawn-origin on Esc/Cmd-Tab resume). Fix applied in-scope of Story 3.9 (see Implementation deviation 2 in Completion Notes); Till re-smoked all paths green after the fix.
    - (a) Single LMB tap → 1 projectile visible + 1 `fired projectile` log line.
    - (b) Hold LMB ~2 s → ~8 fires (verify `grep -c 'fired projectile' /tmp/story-3-9-run.log` ≈ 8).
    - (c) Single tap → wait 0.1 s → tap → wait 0.3 s → tap. Expect 2 fires (cooldown gate).
    - (d) Hold LMB 5 s → ~20 fires; release → no further fires within 1 s.
    - (e) Stationary fire → projectile flies straight forward at constant speed (visual).
    - (f) Drift forward then fire → projectile faster relative to world (lead-time visibly shorter).
    - (g) Drift backward then fire → projectile slower forward but still moves forward.
    - (h) Pitch ship up 30° → fire → projectile flies upward-forward (verifies ship-current-orientation).
    - (i) Single fire → wait 3 s → projectile despawns (visual).
    - (j) Fire mid-flight → Esc-pause → resume → projectile continues motion AND TTL counting.
    - (k) Hold LMB → Esc-pause mid-hold → no fires during pause → resume → fires resume.
    - (l) Cmd-Tab focus-loss mid-flight → focus regain → projectile + TTL resume normally.
    - (m) Quit during flight → no panic; clean shutdown.
    - (n) Fire toward asteroid → bounce-off observed (no panic, no warn-spam, projectile eventually TTLs out).
  - [x] **Post-runtime grep:** Till's smoke confirmed expected counts. Note: post-pause-fix, `entered Arena` + `spawned PlayerShip` now fire only 1× per app-launch (no longer per Arena-resume) — this is the intended new behavior of `OnTransition { MainMenu → Arena }`.
  - [x] Confirm the 3 pre-existing documented WARNs from Story 3.5/3.6/3.7 reappear unchanged. (Confirmed via Till's smoke; no fourth WARN.)

- [x] **Task 9: Update `_bmad-output/implementation-artifacts/deferred-work.md` IF NEEDED** (AC: #5)
  - [x] Story 3.9 anticipates **NO** new deferred-work entries — the design fully consumes the firing + ballistics + lifecycle scope; CollisionLayers is Story 3.10's setup (not a deferral); SemanticAccent::PlayerOwned re-tint for projectiles is covered by mandatory addendum to deferred-work.md:204 below.
  - [ ] **Conditional addendum (if smoke surfaces a need):** if Till's smoke shows defaults (speed=120, rate=4 Hz, ttl=3 s) feel wrong (e.g., projectile too slow vs. ship max-velocity, fire-rate too low for "spaceship rapid-fire" feel, or TTL too short vs. typical asteroid distances), defer the tuning curve to a future polish story (likely Epic 4 Story 4.4 weapon-archetype system or Epic 10 final-polish).
    Format (only add if observed):
    ```
    ## Deferred from: 3-9-weapon-firing-projectile-ballistics (2026-XX-XX)
    - **Projectile tuning defaults felt off** — `assets/config/tuning.ron` projectile_speed/fire_rate/ttl. Defaults felt [too slow / too fast / too short / too long] in smoke. **Resolution path:** Story 4.4 (3 weapon archetypes — Shotgun / Railgun / etc.) introduces archetype-specific projectile parameters; the placeholder default values from 3.9 will be retired or specialized at that point.
    ```
  - [x] **Mandatory one-line addendum to existing deferred-work.md:204 entry** (PlayerShip retroactive re-tint to PlayerOwned): appended — `**Story 3.9 extends the same deferral to Projectile entities** — `src/combat/projectiles.rs::fire_primary_weapon` uses `color_for(SemanticAccent::Neutral)` for projectile material tint. Story 4.5's full sweep on `SemanticAccent::PlayerOwned` should re-tint projectile materials AND attach `SemanticAccent::PlayerOwned` Component to projectile entities. No separate deferred-work entry needed — single 4.5 sweep handles ship + projectiles together.`

- [ ] **Task 10: Sprint-status bookkeeping + commit/push (NOT YET — await Till's authorization)** (per Story 3.5/3.6/3.7/3.8 precedent)
  - [ ] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - [ ] `3-9-weapon-firing-projectile-ballistics: ready-for-dev → in-progress` — flip at start of dev-story.
    - [ ] `3-9-weapon-firing-projectile-ballistics: in-progress → review` — flip after Till's runtime-smoke confirmation.
    - [ ] `last_updated:` bumped to current date with brief note (e.g., `2026-XX-XX (Story 3.9 in-progress → review — weapon firing + projectile ballistics verified)`).
  - [ ] Update this story file's `Status:` field at line 3. Flip `ready-for-dev → in-progress → review` after Till's runtime-smoke confirmation.
  - [ ] Populate the `## Dev Agent Record` section: `Agent Model Used`, `Debug Log References` (the 7 commands' grep counts table), `Completion Notes List` (one bullet per AC #1–#12), `File List` (Modified: `src/main.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`, `sprint-status.yaml`, this file; Added: `src/combat/mod.rs`, `src/combat/components.rs`, `src/combat/input.rs`, `src/combat/projectiles.rs`).
  - [ ] **Commit 1 (feat):** stage `src/combat/`, `src/main.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`. Message: `feat: weapon firing + projectile ballistics (Story 3.9)`. **DO NOT COMMIT UNLESS TILL AUTHORIZES.**
  - [ ] **Commit 2 (bmad):** stage `_bmad-output/implementation-artifacts/sprint-status.yaml`, `_bmad-output/implementation-artifacts/3-9-weapon-firing-projectile-ballistics.md`, AND `_bmad-output/implementation-artifacts/deferred-work.md` IF the addendum from Task 9 was applied. Message: `bmad: story 3.9 ready-for-dev → review (weapon firing + projectile ballistics)`. **DO NOT COMMIT UNLESS TILL AUTHORIZES.**
  - [ ] **DO NOT push.** Push happens only after explicit authorization, AND only after Story 3.9 code review (`bmad-code-review`) passes per Story 3.5/3.6/3.7/3.8 precedent.

## Dev Notes

### Architecture compliance

- **Plugin home:** `CombatPlugin` in `src/combat/mod.rs` per architecture.md:564-570 (FR9–FR16 location). Story 3.9 is the FIRST story to introduce CombatPlugin; future stories (3.10 damage events, 4.x enemy AI + weapon archetypes, 5.x HullHP/ShieldHP) extend it.
- **File creation per architecture.md:565-570:** Story 3.9 introduces `mod.rs`, `components.rs`, `input.rs`, `projectiles.rs`. The architecture also lists `weapons.rs` (3 archetype firing systems) and `damage.rs` and `enemy_ai.rs` — those are Story 4.x scope. Story 3.9's firing logic lives in `projectiles.rs` rather than in a separate `weapons.rs` because there is only ONE primary weapon in 3.9 (no archetype concept yet); splitting prematurely would create a near-empty `weapons.rs`. Story 4.4's 3-archetype refactor is the natural moment to introduce `weapons.rs` and migrate `fire_primary_weapon` (potentially renamed) into it.
- **SystemSet name:** `CombatSystems::Setup` (OnEnter — combat insertion onto PlayerShip), `CombatSystems::Fire` (FixedUpdate — fire_primary_weapon), `CombatSystems::Lifecycle` (FixedUpdate — tick_projectile_ttl). Architecture.md:512 prescribes `enum CombatSystems { EvaluateHits, ApplyDamage, CheckDeath }` — those variants are Story 3.10/4.x scope. Story 3.9 introduces only the variants it needs, mirroring Story 3.6 / 3.7 / 3.8's `FlightSystems::ApplyForces`-only pattern (those stories did not pre-add `ReadInput` / `IntegratePhysics` from architecture.md:411).
- **System naming:** `fire_primary_weapon` (snake_case verb-phrase per architecture.md:323). `tick_projectile_ttl` (verb-phrase, lifecycle housekeeping). `attach_combat_to_player_ship` (verb-phrase, Setup-phase entity-extension). `projectile_initial_velocity` (descriptive helper name — describes the OUTPUT not the input dimension; mirrors `dampener_acceleration` / `ship_local_thrust_vector` precedent).
- **Component naming:** `Projectile` (PascalCase noun, single-responsibility per architecture.md:322 — two related fields: ttl and damage; together they describe the in-flight projectile state). `PrimaryWeaponCooldown` (PascalCase, single-responsibility — one field: remaining; describes the per-ship rate-limit state). The component-name-vs-state-name distinction follows Story 3.8's `DampenerState` precedent: `Projectile` is the marker-with-data component (Bevy convention) for projectile entities; `PrimaryWeaponCooldown` is the marker-with-data component on PlayerShip.
- **Cross-plugin ordering:** introduces ONE new chain: `OnEnter(GameState::Arena) → (FlightSystems::Setup, CombatSystems::Setup).chain()`. Bevy 0.18 composes this with `flight/mod.rs:43-46`'s existing `(ArenaSystems::Setup, FlightSystems::Setup).chain()` into the transitive DAG `ArenaSystems::Setup → FlightSystems::Setup → CombatSystems::Setup`. This is a clean Bevy idiom — multiple chain calls compose additively.
- **Run-condition gate:** `fire_primary_weapon.run_if(in_state(GameState::Arena))` and `tick_projectile_ttl.run_if(in_state(GameState::Arena))` match the flight/dampener precedent (FixedUpdate gameplay systems gated to Arena state). The gate is sufficient for AC #12's pause-cycle requirements: pause sets state to Paused → both systems suppress.
- **Avian + Bevy + leafwing version pins:** all unchanged from Stories 3.6/3.7/3.8 (`bevy = "0.18"`, `avian3d = "0.6"`, `bevy_mod_outline = "0.12"`, `leafwing-input-manager = "0.20"` per Cargo.toml:8-12). No new external deps; no Cargo.toml or Cargo.lock churn expected.
- **Plugin Boundaries (per architecture.md:643-658):** CombatPlugin **owns** `Projectile`, `PrimaryWeaponCooldown`, `CombatAction`, projectile entities. CombatPlugin **consumes** `PlayerShip` marker (FlightPlugin owns), `Transform` + `LinearVelocity` (Bevy / Avian own — not plugin-internal), `TuningConfig` (TuningPlugin owns). CombatPlugin **does NOT** write into FlightPlugin's internal: it only inserts Combat's own components (Plugin Boundaries Rule line 658 — "All cross-plugin effects flow through Events or shared well-known Resources" — the `commands.entity().insert((combat-components,))` pattern adds Combat's own components to a shared entity, which is allowed; the rule forbids writing into another plugin's components, not adding-onto a shared entity).

### Library / framework specifics — Bevy 0.18 `Transform::forward()` (in-codebase precedent: indirect via `apply_local_force` in 3.6, but no direct Dir3 use yet)

- **`Transform::forward(&self) -> Dir3` (Bevy 0.18):** returns the world-space forward direction (defined as `-local_z()` per Bevy convention). `Dir3` is a wrapper around `Vec3` that guarantees unit length; it implements `Deref<Target = Vec3>`. To use as `Vec3` in a calculation: `let forward: Vec3 = *transform.forward();` (deref) or `Vec3::from(transform.forward())`. The former is the project precedent (matches Bevy ecosystem usage).
- **Forward direction in entity-local space:** Story 3.6 confirmed Bevy's convention "forward = -Z, right = +X, up = +Y in entity-local space" via the unit test `forward_only_returns_neg_z_in_local_space`. `Transform::forward()` rotates the local-space `-Z` by the entity's world rotation, returning a unit world-space direction.
- **Why `*transform.forward()` not `transform.local_z()`:** `local_z()` returns the +Z direction (ship's BACK); negating yields forward. `forward()` is the named API that does this for you. Use the named API.

### Library / framework specifics — Avian 0.6 `LinearVelocity` component (in-codebase precedent: only as a spawn-tuple zero-init in flight/mod.rs:117)

- **`LinearVelocity(pub Vector)` (Avian 0.6):** tuple-struct component holding world-space linear velocity. `Vector` aliases `Vec3` for 3D. Story 3.5/3.6/3.7/3.8 spawn PlayerShip with `LinearVelocity(Vec3::ZERO)` and let Avian + flight forces evolve it. Story 3.9 SETS `LinearVelocity(velocity)` at projectile spawn time to seed the initial world-space velocity.
- **READ vs WRITE patterns:**
  - To **READ** PlayerShip's current LinearVelocity in `fire_primary_weapon`: `Query<&LinearVelocity, With<PlayerShip>>` — borrow-shared. The `.0` field is `Vec3`. (Story 3.9 reads in fire_primary_weapon.)
  - To **WRITE** at spawn: `LinearVelocity(velocity)` in the `commands.spawn(...)` tuple — no mutation of an existing component, just creation.
  - To **MUTATE** an existing entity's LinearVelocity (post-spawn): `Query<&mut LinearVelocity>` — Story 3.9 does NOT do this (would require &mut borrow).
- **Why NOT `Forces::apply_linear_acceleration`:** projectile spawn is a one-frame velocity-set (instant initial velocity), not a continuous acceleration. The `LinearVelocity(...)` component-set is the canonical Avian-0.6 pattern for "give this body an initial velocity" (per Avian docs at `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/mod.rs:412 LinearVelocity(pub Vector)`).
- **Velocity-set semantics on RigidBody::Dynamic:** Avian's XPBD integrator uses LinearVelocity as the canonical velocity state. Setting it at spawn means the projectile enters the next FixedUpdate with that velocity already in the integration data; no extra force-application needed.

### Library / framework specifics — Bevy 0.18 `Time<Fixed>` semantics in FixedUpdate

- **`Res<Time>` in FixedUpdate:** Bevy 0.18 dispatches `Time<Fixed>` through the generic `Time` resource when a system runs inside the FixedUpdate schedule. `time.delta_secs()` returns the fixed timestep (1/60 s = 0.01667 s at our 60 Hz config from main.rs:40 `Time::<Fixed>::from_hz(60.0)`).
- **Cooldown decrement determinism:** `cooldown.remaining -= time.delta_secs()` runs exactly 60× per real second (modulo Avian/Bevy's catch-up tick handling for slow frames). At `projectile_fire_rate_hz = 4.0` (period = 0.25 s = 15 fixed ticks), the cooldown bleeds from 0.25 → 0.0 over exactly 15 ticks — deterministic across runs.
- **TTL decrement determinism:** same — at `projectile_ttl_seconds = 3.0`, projectile lives for exactly 180 fixed ticks before despawn.
- **Pause semantics:** Story 3.4's `pause_simulation_clocks` halts `Time<Fixed>` (the underlying clock that drives FixedUpdate's tick rate). When paused: FixedUpdate does NOT advance → `fire_primary_weapon` and `tick_projectile_ttl` do NOT run → cooldowns and TTLs frozen. On resume: counters resume from frozen state. This is the AC #12 invariant.

### Library / framework specifics — leafwing-input-manager 0.20 `MouseButton::Left` binding (in-codebase precedent: only KeyCode bindings from 3.6/3.7/3.8)

- **`InputMap::new([(action, MouseButton::Left)])` (leafwing-0.20):** `MouseButton` is a Bevy 0.18 enum (`bevy::input::mouse::MouseButton`); leafwing's `InputMap` accepts mixed keyboard/mouse bindings via the `Buttonlike` trait. `MouseButton::Left` is buttonlike (analogous to `KeyCode::KeyW`). No `#[actionlike(...)]` attribute needed on `CombatAction::FirePrimary`.
- **`pressed` semantic for held-fire:** Story 3.9 uses `action.pressed(&CombatAction::FirePrimary)` (continuous while held — same pattern as `apply_thrust`'s `pressed(&FlightAction::ThrustForward)`). NOT `just_pressed` (which would fire once per click; too restrictive for "held primary fire" UX).
- **PreUpdate ActionState refresh:** leafwing's `InputManagerPlugin::<CombatAction>` (registered by Story 3.9) updates `ActionState<CombatAction>` in `PreUpdate` each frame. `fire_primary_weapon` runs in FixedUpdate which is scheduled after PreUpdate (Bevy 0.18 schedule order: First → PreUpdate → StateTransition → FixedFirst → FixedPreUpdate → FixedUpdate → ...). The ActionState `pressed` value reflects the most-recent PreUpdate refresh.
- **Cursor-grab interaction:** Story 3.7 confirmed cursor-grab works with `pressed`/`just_pressed` semantics — mouse motion is captured as deltas regardless of cursor-visibility. `MouseButton::Left` press is registered the same way: invisible cursor + grabbed window still receive `pressed` events.

### Combat / Avian integration — projectile-asteroid contact behavior in 3.9 (DELIBERATE INTENTIONAL LIMITATION)

- **Default Avian collision response:** `RigidBody::Dynamic` projectile + `RigidBody::Static` asteroid → contact produces an impulse on the projectile (asteroid is static, doesn't move). The projectile bounces off with reduced velocity. No event is emitted by default.
- **Why no CollisionLayers in 3.9:** Story 3.10 AC #1 declares "projectiles and asteroids share an Avian `CollisionLayers` group that allows contact." This is 3.10's setup. Pre-empting it in 3.9 would either:
  (a) Disable contact entirely (projectiles fly through asteroids — wrong for 3.10 hookup; would require a 3.10 reversal), OR
  (b) Configure the layer-group in 3.9 to allow contact — which is exactly what 3.10 does, so duplicate work.
  The cleanest 3.9 disposition is "ship without filter; accept default Avian contact behavior; let 3.10 attach the event-emit-and-despawn-and-damage logic on top."
- **Visual artifact in 3.9:** projectiles fired toward asteroids will visibly bounce. This is documented in AC #10 (n) as INTENTIONAL — 3.10 fixes it.
- **No state corruption:** Avian's bounce response does not mutate any combat-side state (Projectile component, PrimaryWeaponCooldown, etc.). The projectile entity persists with full TTL until either time-out OR Story 3.10's hit-event-despawn lands.
- **PlayerShip-projectile self-collision:** AVOIDED via spawn-offset clearance. `PROJECTILE_SPAWN_OFFSET = 3.0 m > ship_collider_radius (2.0 m) + projectile_radius (0.2 m) = 2.2 m`. Projectile spawns 0.8 m beyond the ship's collider boundary. Even if ship is moving forward at maximum thrust-on-velocity (~30 m/s in pure 3.6 Newtonian, capped at ~7.5 m/s with dampener-on per Story 3.8 AC #10), the ship cannot catch up to the projectile within the spawn frame: projectile is at +3.0 m forward with +120 m/s forward velocity; ship at +0 m with up to +30 m/s forward — relative velocity of projectile to ship is at minimum +90 m/s forward (departing). No self-collision possible.

### Salvage economy / FR11 pay-to-shoot — DEFERRED to Epic 6 Story 6.9

- **Architecture.md:682** maps "FR11 pay-to-shoot | `src/salvage/economy.rs` (debit on `WeaponFired` event)". Salvage plugin does not exist yet (Epic 6). Story 3.9 does NOT emit a `WeaponFired` event and does NOT debit currency.
- **Future hookup point:** when Epic 6 introduces `SalvageCurrency` Resource + `SalvagePlugin`, Story 6.9 (pay-to-shoot economy) will:
  - Add a `WeaponFired { ship: Entity, projectile: Entity }` event to `combat/projectiles.rs` or `combat/weapons.rs`.
  - Modify `fire_primary_weapon` to emit the event after spawning the projectile.
  - Add a system in `salvage/economy.rs` that consumes `WeaponFired` and debits `SalvageCurrency` per shot.
- **No 3.9 surface for the future event:** Story 3.9 does NOT pre-define `WeaponFired` (no consumer exists; defining it in 3.9 would create dead code). 6.9 introduces it together with the consumer.

### File structure requirements

```
src/
├── combat/                             # NEW DIRECTORY
│   ├── mod.rs                          # NEW FILE — CombatPlugin + CombatSystems enum + plugin scaffolding (~50 lines)
│   ├── components.rs                   # NEW FILE — Projectile + PrimaryWeaponCooldown components + 1 unit test (~35 lines)
│   ├── input.rs                        # NEW FILE — CombatAction enum + default_input_map (~12 lines)
│   └── projectiles.rs                  # NEW FILE — projectile_initial_velocity helper + fire_primary_weapon + tick_projectile_ttl + attach_combat_to_player_ship + 4 unit tests (~140 lines)
├── flight/                             # UNCHANGED — per AC #2; combat extends PlayerShip via OnEnter system, not via spawn-tuple growth
├── arena/                              # UNCHANGED
├── pause/                               # UNCHANGED
├── ui/                                 # UNCHANGED
├── visual/                             # UNCHANGED
├── tuning/
│   └── config.rs                       # MODIFIED — +3 fields, +3 helpers, +3 Default literals, +3 in-place test extensions (9 new assertions across 3 tests, ron-bytes literal extension in deserialize test)
├── state.rs                            # UNCHANGED — Copy derive remains deferred per deferred-work.md:198
├── splash.rs                           # UNCHANGED
├── logging.rs                          # UNCHANGED
└── main.rs                             # MODIFIED — +1 mod decl (`mod combat;`), +1 use line, +1 add_plugins call
assets/
├── config/
│   └── tuning.ron                      # MODIFIED — +3 lines (projectile_speed, projectile_fire_rate_hz, projectile_ttl_seconds)
└── ...                                 # UNCHANGED
Cargo.toml                              # UNCHANGED — no new deps
Cargo.lock                              # UNCHANGED — no transitive-dep churn
```

**Target file size deltas:**

| File | Pre-3.9 | Post-3.9 target | Delta |
|---|---|---|---|
| `src/combat/mod.rs` | (does not exist) | ~50 lines | NEW FILE |
| `src/combat/components.rs` | (does not exist) | ~35 lines | NEW FILE |
| `src/combat/input.rs` | (does not exist) | ~12 lines | NEW FILE |
| `src/combat/projectiles.rs` | (does not exist) | ~140 lines | NEW FILE |
| `src/main.rs` | 62 | ~65 | +3 (mod decl + use + add_plugins call) |
| `src/tuning/config.rs` | 161 | ~178 | +17 (3 fields + 3 helpers + 3 Default literals + 9 assertions + 1 ron-bytes literal extension) |
| `assets/config/tuning.ron` | 12 | 15 | +3 |

### Testing standards

Per architecture.md:351-354:
- **Co-located** `#[cfg(test)] mod tests { ... }` at the bottom of each module file.
- **Pure-logic modules first-class test targets;** integration tests deferred post-M3.

Story 3.9's `fire_primary_weapon`, `tick_projectile_ttl`, and `attach_combat_to_player_ship` systems are integration-test-shaped (would need `MinimalPlugins + PhysicsPlugins + leafwing + state setup + tick FixedUpdate manually`) and therefore not unit-tested. The pure-logic helper `projectile_initial_velocity` IS unit-tested (4 tests covering: stationary, drifting-forward, strafing, zero-speed-degenerate). The `PrimaryWeaponCooldown::default()` invariant is unit-tested in `combat/components.rs` (1 test).

`TuningConfig`'s 3 new fields add NO new test functions — the existing 3 tests are extended in-place to cover the new fields. Test count delta from tuning: **+0**.

**Net post-3.9 test count target: 41** (= 36 baseline from end of 3.8 + 4 from `combat/projectiles.rs` + 1 from `combat/components.rs`). AC #11 enforces N = 41.

### Anti-patterns to avoid (catalogued from Stories 1.5–3.8 review precedent + 3.9-specific risks)

1. **Story-id references in module doc-comments and inline comments** — Stories 1.5/3.2/3.6/3.7/3.8 review patches removed all "Story X.Y" references. **Do NOT** write `//! Story 3.9 introduces CombatPlugin`. **Do NOT** write `// Story 3.9 — fire_primary_weapon`. Module docs describe what the module owns; inline comments explain WHY when non-obvious. THE ONLY exception is doc comments that reference SPECIFIC future stories with deferral context (e.g., `// Story 3.10 adds CollisionLayers + ProjectileHitAsteroid event` is acceptable because it describes the intentional 3.9 limitation and points to the resolution; same pattern as 3.5's references to 4.5 SemanticAccent wiring).
2. **Wildcard imports beyond `bevy::prelude::*`, `avian3d::prelude::*`, `leafwing_input_manager::prelude::*`** — explicit imports per architecture.md naming-discipline. Note that the architecture's `use avian3d::prelude::*;` is the project precedent; for 3.9 use the narrower `use avian3d::prelude::{Collider, LinearVelocity, RigidBody};` to make the surface-used explicit (matches Story 3.5/3.6/3.7/3.8 narrowing convention).
3. **Adding `FirePrimary` as an axis-kind action** — it MUST be default-buttonlike (no `#[actionlike(...)]` attribute). Primary-fire is press-edge-driven; an axis would be nonsense semantically.
4. **Using `just_pressed` instead of `pressed` for held-fire** — `just_pressed` would fire only on the press-edge tick (one shot per click), defeating the AC's "holds FirePrimary → emits shots at projectile_fire_rate_hz" design. `pressed` is the correct continuous semantic; the cooldown component handles rate-limiting.
5. **`.after(specific_function)` for system ordering** — architecture.md:415 forbidden. CombatPlugin's `OnEnter` chain uses SystemSet placement (`(FlightSystems::Setup, CombatSystems::Setup).chain()`); FixedUpdate uses `(CombatSystems::Fire, CombatSystems::Lifecycle).chain()`. NO `.after(spawn_player_ship)` references.
6. **Pre-empting Story 3.10's CollisionLayers setup in 3.9** — see "Combat / Avian integration" section above. Story 3.9 ships projectiles WITHOUT layer filtering; 3.10 owns the layer-group + ProjectileHitAsteroid event + damage routing. Adding either in 3.9 creates duplicate work or an unhealthy 3.9-3.10 coupling.
7. **Pre-defining `WeaponFired` event in 3.9** — Story 6.9 owns the salvage-debit hookup; emitting an unconsumed event in 3.9 creates dead code (event with no reader = compile-warning under -D warnings, OR test pollution via `#[allow(dead_code)]`). Defer to 6.9.
8. **Clamping projectile_speed / fire_rate_hz / ttl_seconds proactively in `dampener_acceleration`-style** — same Story 3.6/3.7/3.8 anti-pattern reasoning. Epic spec says "configurable via TuningConfig"; no clamp. NaN/inf guard remains deferred per deferred-work.md:222 (consolidated entry covers all TuningConfig f32 fields including the 3 new projectile fields).
9. **Per-tick `info!` / `warn!` / `debug!` logs in `fire_primary_weapon`** — the ONE log allowed in this story is in `fire_primary_weapon` AT THE FIRE-EVENT SITE (gated by cooldown to ≤ 4 Hz max under realistic input). NO log in the cooldown-decrement path (which would be 60 Hz). NO log in `tick_projectile_ttl` (which would also be 60 Hz × N projectiles). NO log in `attach_combat_to_player_ship` (one-shot OnEnter system; minor side-effect; no diagnostic value).
10. **Touching `src/state.rs`** — all `run_if(in_state(GameState::Arena))` gates handle cloning internally. The `Copy` derive on `GameState` remains deferred per deferred-work.md:198.
11. **Touching `src/flight/**`** — per AC #2. CombatPlugin extends PlayerShip via OnEnter system, not via spawn-tuple growth. FlightPlugin is intentionally unaware of CombatPlugin (one-way dependency: combat depends on flight, not vice versa).
12. **Adding `Cargo.toml` deps** — no new deps. Every API used is already in scope from Stories 3.5/3.6/3.7/3.8.
13. **Using `OutlineVolume` on projectiles** — outline polish on small fast-moving entities is deferred to Epic 10. Adding outlines in 3.9 increases wgpu draw cost and visual noise; no AC requires outlines on projectiles.
14. **Adding `SemanticAccent` Component to projectile entities (vs. just material tint)** — full SemanticAccent wiring on enemies/salvage/playership/projectiles is Story 4.5's sweep. Story 3.9 uses ONLY the material tint via `color_for(SemanticAccent::Neutral)` for visual consistency with PlayerShip (also Neutral per Story 3.5). The Component-attachment is 4.5's job.
15. **Spawning projectile with `.with_children(|parent| ...)` hierarchy** — projectiles are flat root entities (no children). No parent-child cleanup race like the splash-cleanup deferred-work.md:75-76 entry.
16. **Despawning projectiles via `try_despawn` in 3.9** — `commands.entity().despawn()` is safe for flat entities (no parent-child linked-despawn race). The `try_despawn` API at deferred-work.md:75 is for hierarchy-aware cleanup; not applicable here.
17. **Adding `weapons.rs` in 3.9** — architecture.md:567 reserves `weapons.rs` for "3 prefab weapon archetypes, firing systems" which is Story 4.4 scope. Creating an empty/single-archetype `weapons.rs` in 3.9 either creates dead code or pre-bakes a structure that Story 4.4 will refactor anyway. Inline `fire_primary_weapon` in `projectiles.rs` for 3.9; Story 4.4 introduces `weapons.rs` and migrates.
18. **Adding visual UI for cooldown / ammo / fire-rate in 3.9** — HUD work belongs to Story 3.11 (HUD baseline) or Story 5.4 (HUD wiring). Story 3.9's only player-facing signal is the `info!` log per AC #4(i) (dev-visible) and the visible projectile entity in 3D space. UI HUD wiring is deferred per AC #11 git-status spec ("NO entries under `src/ui/**`").
19. **Using `Time::<Fixed>::delta_secs()` instead of `Res<Time>::delta_secs()`** — both work in FixedUpdate context, but `Res<Time>` is the project precedent (used by `apply_thrust` / `apply_torque` / `apply_dampener` / `tick_splash_timer`). Bevy 0.18 routes generic `Time` to `Time<Fixed>` automatically inside FixedUpdate.
20. **Using `commands.spawn(...).id()` to capture projectile entity for downstream use** — Story 3.9 has no downstream consumer of the spawned-projectile Entity. Future stories (3.10 ProjectileHitAsteroid event needs the projectile Entity in the event payload) WILL need it; for 3.9, ignore the returned EntityCommands and let `fire_primary_weapon` complete spawn-and-forget.
21. **Treating projectile-asteroid bounce as a regression** — the bounce is INTENTIONAL 3.9 behavior (per "Combat / Avian integration" section above). Story 3.10 turns the bounce into a damage event. Smoke verification AC #10 (n) tests the bounce explicitly; do NOT add CollisionLayers or filter-disable to suppress it.
22. **Despawning the projectile from inside `fire_primary_weapon` (e.g., on cooldown-violation early-return)** — once the cooldown gate fires-and-spawns, no further despawn logic in `fire_primary_weapon`. `tick_projectile_ttl` is the despawn path. Future stories may add hit-despawn (3.10), but 3.9 keeps the lifecycle pure: spawn-via-fire, despawn-via-tick.
23. **Adding `projectile_count: u32` Resource for global projectile-count tracking** — out of scope. Bevy's entity-count diagnostic (deferred to M2 per architecture.md:296) covers this. Smoke is sufficient for 3.9.
24. **Splitting `fire_primary_weapon` into `tick_cooldown` + `fire_if_ready` two-system pipeline** — single system is simpler (one query, one iteration, one read-write of cooldown). Splitting requires either two queries on PlayerShip (redundant) or a Resource-based shared state (overkill). Going with single-system. (Pure-logic split for `dampener_acceleration` was justified because the acceleration math is testable; cooldown decrement + branch is too trivial to extract.)
25. **Adding a `MIN_FIRE_RATE_HZ` clamp on the projectile_fire_rate_hz tuning value** — input hardening lives in TuningConfig deserialization, deferred per deferred-work.md:228. If `projectile_fire_rate_hz <= 0.0`, `1.0 / 0.0` produces inf, cooldown.remaining = inf, projectile fires once (initial), then never again. This is a degenerate-but-safe outcome; no panic. The deferred-work entry covers it.

### Logging discipline

Per architecture.md:376-383:
- `info!` for lifecycle events: existing logs from `log_arena_entered`, `pause::pause_on_focus_loss`, `pause::toggle_pause_on_escape`, `flight::spawn_player_ship`, `physics::toggle_dampener` all unchanged.
- ONE NEW `info!` in `fire_primary_weapon` — `info!("fired projectile at velocity={:?} ttl={}", velocity, tuning.projectile_ttl_seconds)` — gated by cooldown (≤ 4 Hz max). The 4 Hz max is acceptable per Story 3.8's anti-pattern carve-out (X-press toggle ≤ 1 Hz; primary fire at ≤ 4 Hz is 4× higher but still 15× lower than the 60 Hz per-tick anti-pattern).
- NO per-tick logs in `fire_primary_weapon` cooldown decrement, `tick_projectile_ttl`, or `attach_combat_to_player_ship`.
- **Future-deprecation note:** the per-fire `info!` will likely be removed when Epic 6 Story 6.9 introduces the `WeaponFired` event (the event becomes the canonical signal; structured logging via `tracing` spans on the event reader replaces the inline log). This is a Story 6.9 concern, not 3.9's.

### Project Structure Notes

- **Alignment with unified project structure:** `src/combat/` directory creation matches architecture.md:564-570 prescription; `src/combat/mod.rs` (CombatPlugin + CombatSystems), `src/combat/components.rs` (Projectile + PrimaryWeaponCooldown), `src/combat/input.rs` (CombatAction enum), `src/combat/projectiles.rs` (firing + ballistics + lifecycle) all map directly to the architecture-prescribed file layout. `src/main.rs` plugin registration follows the existing convention.
- **Detected variances:**
  - `src/combat/projectiles.rs` hosts the firing system (`fire_primary_weapon`), which architecture.md:567 nominally assigns to `weapons.rs`. Rationale: 3.9 has only one weapon archetype; splitting into two files is premature. Story 4.4 (3 weapon archetypes) will extract `fire_primary_weapon` (likely renamed and parametrized) into `weapons.rs`. Until then, `projectiles.rs` owns both the firing and the ballistics, justified by the single-archetype scope. Documented inline as a top-of-file comment.

### Review Findings

- [x] [Review][Defer] Intentional `OnTransition` deviation from spec ACs #2/#8/#11 — all three plugins (Combat, Flight, Arena) use `OnTransition { exited: MainMenu, entered: Arena }` instead of `OnEnter(GameState::Arena)`. `src/arena/mod.rs` and `src/flight/mod.rs` modified against AC #11's prohibition. Beneficial fix documented in deferred-work.md. — deferred: Spec-Amend kostet mehr als es nützt; deferred-work.md reicht.

- [x] [Review][Patch] Division by zero — `1.0 / tuning.projectile_fire_rate_hz` when `projectile_fire_rate_hz ≤ 0.0` sets `cooldown.remaining` to `f32::INFINITY`, permanently disabling firing with no log output [src/combat/projectiles.rs] — fixed: `.max(f32::EPSILON)` guard added

- [x] [Review][Defer] Per-shot mesh+material allocation — `meshes.add()` + `materials.add()` called on every fire event; assets are GC'd on entity despawn via Bevy strong-handle ref-counting (no leak), but creates allocation churn vs. a cached shared handle [src/combat/projectiles.rs:99-103] — deferred, performance optimization
- [x] [Review][Defer] Cold-start `unwrap_or_default()` fallback is silent — pre-existing pattern in codebase (matches flight/dampener system precedent) [src/combat/projectiles.rs:86-89] — deferred, pre-existing
- [x] [Review][Defer] `projectile_ttl_seconds ≤ 0` → projectile spawns and immediately despawns — TuningConfig input validation deferred project-wide (deferred-work.md covers all tuning fields) [src/combat/projectiles.rs:111] — deferred, pre-existing
- [x] [Review][Defer] No arena cleanup for Arena→PostRun/Caravan/PhotoMode transitions — only Arena→MainMenu wired; forward concern, documented in code comments [src/arena/mod.rs] — deferred, forward-compat concern
- [x] [Review][Defer] `CombatSystems::Fire` not chained after `FlightSystems::ApplyForces` in FixedUpdate — projectiles may inherit ship velocity from previous physics tick; sub-frame accuracy concern, negligible for 4 Hz fire rate [src/combat/mod.rs] — deferred, sub-frame accuracy
- [x] [Review][Defer] Self-collision window at high ship speed — 3.0m spawn offset doesn't account for ship velocity; Story 3.10's CollisionLayers fix prevents self-damage [src/combat/projectiles.rs:95] — deferred, Story 3.10 resolves
- [x] [Review][Defer] Non-MainMenu→Arena entry paths leave world empty — `OnTransition { MainMenu→Arena }` only; any future entry path (Loading→Arena shortcut, Caravan→Arena) bypasses spawn; forward concern documented in code comments [src/combat/mod.rs, src/arena/mod.rs] — deferred, forward-compat concern
- [x] [Review][Defer] `attach_combat_to_player_ship` silent no-op if ships query is empty — no warning emitted; `.chain()` ordering mitigates in practice; minor debug ergonomics concern [src/combat/projectiles.rs:37-48] — deferred, pre-existing
  - `CombatSystems` enum has only 3 variants (Setup, Fire, Lifecycle) instead of architecture.md:512's prescribed `EvaluateHits, ApplyDamage, CheckDeath`. Rationale: those variants are 3.10/4.x scope; introducing them in 3.9 with no consumers creates dead code. Same pattern as Story 3.6's FlightSystems-only-ApplyForces decision (the architecture-prescribed `ReadInput`, `IntegratePhysics` were not added).
- **Feature divergence note:** none. All API surfaces used (Bevy 0.18 ECS, Avian 0.6 RigidBody/Collider/LinearVelocity, leafwing 0.20 Actionlike/InputMap/ActionState/MouseButton) are at the project's already-pinned versions.

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation-Patterns--Consistency-Rules] — naming, SystemSet, event-driven, plugin boundary patterns adopted.
- [Source: _bmad-output/planning-artifacts/architecture.md:564-570] — `src/combat/` file layout prescription.
- [Source: _bmad-output/planning-artifacts/architecture.md:511-528] — Good SystemSet use example (CombatSystems pattern reference).
- [Source: _bmad-output/planning-artifacts/architecture.md:646-655] — Plugin Boundaries table (CombatPlugin: HullHP/ShieldHP/Weapon/projectiles/enemy AI; consumes TuningConfig + flight/input weapon-fire intents).
- [Source: _bmad-output/planning-artifacts/architecture.md:681-688] — FR9–FR16 to file mapping.
- [Source: _bmad-output/planning-artifacts/prd.md:511-518] — FR9–FR16 functional requirements (Combat System cluster).
- [Source: _bmad-output/planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md:230-264] — Story 3.9 epic-level AC source (Weapon Firing + Projectile Ballistics).
- [Source: _bmad-output/planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md:266-298] — Story 3.10 epic-level AC (next-story prerequisite analysis: ProjectileHitAsteroid event + damage routing + CollisionLayers).
- [Source: _bmad-output/implementation-artifacts/3-8-inertial-dampener-toggle.md] — most-recent-story precedent for tuning-extension pattern, system-registration pattern, anti-pattern catalog, verification harness.
- [Source: _bmad-output/implementation-artifacts/3-7-flight-input-3-axis-rotation-pitch-yaw-roll.md] — leafwing 0.20 Actionlike/InputMap precedent.
- [Source: _bmad-output/implementation-artifacts/3-6-flight-input-6-dof-translation.md] — Avian 0.6 Forces query precedent + `pressed` semantics.
- [Source: _bmad-output/implementation-artifacts/3-5-cockpit-camera-playership-entity.md] — PlayerShip spawn pattern + cold-start tuning fallback + SemanticAccent::Neutral placeholder convention.
- [Source: _bmad-output/implementation-artifacts/3-3-hand-designed-arena-zone-with-static-asteroid-field.md] — `Sphere::new(r).mesh().ico(2)` precedent + visual-radius == physics-radius pattern.
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:204] — PlayerShip SemanticAccent::PlayerOwned retroactive re-tint deferred to Story 4.5; Story 3.9 inherits same disposition for projectiles.
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:222] — TuningConfig f32 NaN/inf guard deferred (covers projectile_speed, projectile_fire_rate_hz, projectile_ttl_seconds).
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:228] — TuningConfig f32 range/sign validation deferred (covers projectile_fire_rate_hz <= 0 → inf cooldown degenerate case).
- [Source: ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/avian3d-0.6.1/src/dynamics/rigid_body/mod.rs:412] — `pub struct LinearVelocity(pub Vector)` definition.
- [Source: bevy 0.18 docs] — `Transform::forward(&self) -> Dir3`; `Dir3: Deref<Target = Vec3>`.

## Previous Story Intelligence (Story 3.8 — Inertial Dampener Toggle)

Story 3.8 is the most recent reference for the development pattern. Key learnings to inherit:

- **Component-tuple ordering** (3.6/3.7/3.8 spawn pattern): `spawn_player_ship`'s tuple grew to 13 components by end of 3.8. Story 3.9 does NOT extend the spawn tuple (per AC #2 — combat extends PlayerShip via OnEnter system). The tuple stays at 13.
- **Cold-start tuning fallback** (3.6/3.7/3.8 pattern): `tuning_assets.get(handle).cloned().unwrap_or_default()` — NO per-tick `warn!`. Story 3.9 reuses this exact pattern in `fire_primary_weapon` (no spawn-time warn here either; the component is added at OnEnter but doesn't read tuning until first FixedUpdate tick).
- **Avian `Forces` query data, not `ExternalForce` component** (3.6 Deviation #2): Story 3.9 does NOT use `Forces` — projectile spawn uses the Avian-canonical `LinearVelocity(velocity)` component-set pattern. `Forces` is for continuous force/torque/acceleration application; LinearVelocity is for instant velocity-set at spawn.
- **`ActionState::pressed(&action)` for continuous, `just_pressed(&action)` for edge-triggered** (3.6/3.7/3.8 + leafwing-0.20 docs): Story 3.9 uses `pressed` for the continuous-fire-while-held semantic.
- **Test count baseline = 36** (`cargo test` 2026-05-05 measurement — 30 from end of 3.7 + 5 new in 3.8 + 1 components test): Story 3.9 adds 5 → final 41.
- **Pause/resume preserves PlayerShip state** (3.6 AC #9 ✅, 3.7 AC #10, 3.8 AC #11 re-confirmed for DampenerState): Story 3.9 AC #12 re-confirms for `PrimaryWeaponCooldown` (component lives on persistent ship; pause halts FixedUpdate so cooldown timer freezes).
- **Story-id-comment scrub** (3.6/3.7/3.8 review patches): keep doc comments and inline comments free of "Story 3.9" / "Story X.Y" references EXCEPT for forward-pointing references that document intentional 3.9 limitations (e.g., "Story 3.10 adds CollisionLayers" — this is the explicit Story 3.5 / Story 3.7 precedent for "next-story-resolves-this" comments).
- **Per-command grep verification harness** (3.6/3.7/3.8 Task 7): mirrored exactly per AC #11 + Task 7. The 6-command + runtime-smoke sweep is the canonical local-verification pattern.
- **2-commit pattern (feat + bmad)** (3.6/3.7/3.8 final task): mirrored. Commits and pushes await Till's authorization.
- **Tuning-config field extension pattern** (Story 2.4/3.6/3.7/3.8 precedent): per-field `#[serde(default = "default_…")]`, top-level helpers, Default impl extension, in-place test extension (no new test functions added). Story 3.9 follows for 3 new fields.
- **Plugin registration in `main.rs`** (Story 3.5/3.6 precedent: FlightPlugin registered between ArenaPlugin and PausePlugin): Story 3.9 extends — CombatPlugin registered between FlightPlugin and PausePlugin (depends on PlayerShip from Flight; orthogonal to Pause).
- **Update vs FixedUpdate scheduling** (Story 3.8 precedent: `toggle_dampener` in Update for input-handling cadence; `apply_dampener` in FixedUpdate for physics-coupling): Story 3.9 puts BOTH systems (`fire_primary_weapon` and `tick_projectile_ttl`) in FixedUpdate because both are physics-coupled (projectile spawn timing affects integration; TTL timing matches fixed-step). NO Update-schedule systems.

## Git intelligence summary

Recent commit history (`git log --oneline -8`):
- `fcda902` bmad: story 3.8 review → done (inertial dampener toggle)
- `b43fc9b` feat: inertial dampener toggle (Story 3.8) ← **canonical predecessor commit; FlightAction extensions, apply_dampener, dampener_acceleration helper, DampenerState component, KeyX binding, ApplyForces 3-tuple**
- `baf057e` bmad: epic 10 add Story 10.13 — final mesh assets in MVP
- `31954f4` bmad: story 3.7 review → done (5 patches applied, 6 deferred)
- `541a6d7` fix: Story 3.7 review — mouse-look accumulator + cursor-warp suppression
- `33151c0` bmad: story 3.7 ready-for-dev → review (3-axis rotation)
- `108b381` feat: 3-axis rotation + cursor grab (Story 3.7)
- `253c3dd` bmad: story 3.6 review → done

**Patterns extracted:**

- **2-commit cadence per story:** `feat:` for code + `bmad:` for spec/state metadata. Story 3.9 follows.
- **Cargo.lock unchanged since pre-3.6:** leafwing's transitive deps were locked at Story 1.2's plugin-compat gate; no new transitives added through 3.8. Story 3.9 adds NO new external surface; Cargo.lock should remain unchanged.
- **Module patterns introduced ahead of consumers:** `flight/components.rs` was created in 3.8 with one occupant (`DampenerState`); 3.9 extends the precedent by creating the entire `src/combat/` directory with 4 files containing 3.9-relevant types (Projectile, PrimaryWeaponCooldown, CombatAction, fire_primary_weapon, tick_projectile_ttl, projectile_initial_velocity). Future stories (3.10, 4.4, 5.x) add HullHP/ShieldHP/CollisionLayers/Weapon/enemy AI as consumers land.
- **Files touched by 3.7's review patches (`src/flight/mod.rs`, `src/flight/physics.rs`):** Story 3.9 does NOT touch these per AC #2. The 3.7 review patches (MouseLookDelta resource, MouseLookSuppressFrames, accumulate_mouse_look system, cursor-warp suppression in `grab_cursor_for_arena`) remain intact and functional.
- **TuningConfig field-extension cadence:** Story 2.4 added 2 fields (outline width/color), Story 3.6 added 1 (ship_thrust_newtons), Story 3.7 added 2 (mouse_sensitivity, ship_torque_nm), Story 3.8 added 2 (dampener_*_strength). Story 3.9 adds 3 (projectile_*). Total post-3.9 TuningConfig fields: 13 (3 toon + 2 outline + 1 thrust + 1 sensitivity + 1 torque + 2 dampener + 3 projectile).

## Latest tech information (Bevy 0.18 + Avian 0.6 + leafwing 0.20)

Story 3.9 introduces no new external dependencies. Every API surface used has the following confirmation status:

- **`leafwing-input-manager = "0.20"`** — already exercised by Stories 3.6/3.7/3.8. New surface for 3.9: `MouseButton::Left` as an InputMap binding (per Bevy 0.18 `bevy::input::mouse::MouseButton` + leafwing-0.20 `Buttonlike` trait impl). `ActionState::pressed(&CombatAction::FirePrimary)` is the same `pressed` API as 3.6's flight thrust.
- **`avian3d::dynamics::rigid_body::LinearVelocity`** — already imported as a spawn-tuple zero-init in `flight/mod.rs:117` (Story 3.5). Story 3.9 SETS the tuple-struct's inner `Vec3` at projectile spawn time. Per `~/.cargo/registry/src/.../avian3d-0.6.1/src/dynamics/rigid_body/mod.rs:412`: `pub struct LinearVelocity(pub Vector);`.
- **`avian3d::prelude::{RigidBody, Collider, LinearVelocity}`** — already in scope from Stories 3.3/3.5/3.6 (asteroid spawn + ship spawn). Story 3.9 reuses for projectile spawn.
- **`bevy::prelude::Transform::forward(&self) -> Dir3`** — Bevy-0.18 standard. `Dir3` derefs to `Vec3` via `Deref<Target = Vec3>`. Standard usage; no surprises.
- **`bevy::prelude::Sphere::new(r).mesh().ico(2)`** — already exercised by `arena/zone.rs:78-83`. Story 3.9 reuses for projectile mesh.
- **`bevy::prelude::Component` + `#[derive(Component)]`** — Bevy-0.18 attribute macro. Stable since Bevy 0.7. Standard usage.
- **`bevy::prelude::FixedUpdate`** — Bevy-0.18 standard schedule (60 Hz per main.rs:40). Already exercised by flight systems.
- **`bevy::prelude::Time`** — generic `Time` resource auto-routes to `Time<Fixed>` inside FixedUpdate (Bevy-0.18 documented behavior).
- **`bevy::ecs::system::Commands::despawn()`** — Bevy-0.18 standard. Standard usage; no parent-child cleanup race for flat projectile entities.
- **No version bumps:** `bevy = "0.18"`, `avian3d = "0.6"`, `bevy_mod_outline = "0.12"`, `bevy_kira_audio = "0.25"`, `leafwing-input-manager = "0.20"` — all unchanged.

## Project context reference

- **Memory:** `MEMORY.md` (auto-loaded at session start) — Till's user memories include `feedback_full_build_output.md` (per-command-grep verification discipline), `feedback_compact_review_style.md` (compact responses), `feedback_staged_rollout.md` (staged-rollout preference, justifies the lean Story 3.9 scope: firing + ballistics + TTL ONLY; collision-driven damage to 3.10; SemanticAccent::PlayerOwned wiring to 4.5; pay-to-shoot economy to 6.9; HUD ammo indicator to 3.11/5.4; weapon archetypes to 4.4).
- **Brainstorming canon:** `_bmad-output/brainstorming/brainstorming-session-2026-04-21-1114.md` — original concept doc; FR9 weapon firing is the offensive-capability foundation that the FR12 projectile-damage outcome (Story 3.10) and the FR11 pay-to-shoot economy (Story 6.9) both depend on.
- **Architecture canon:** `_bmad-output/planning-artifacts/architecture.md` — single-file authoritative architecture.
- **Sprint plan:** `_bmad-output/implementation-artifacts/sprint-status.yaml` — Story 3.9 is the next backlog item after 3.8 done.
- **Deferred work:** `_bmad-output/implementation-artifacts/deferred-work.md` — Story 3.9 inherits open entries:
  - line 168 (asteroid layout drift hazard — Story 3.9 fires projectiles toward asteroids; the existing test scaffold catches any inadvertent ASTEROIDS array edits)
  - line 184 (pause-overlay loses to focus-gain — orthogonal but mentioned in 3.4's "becomes a real concern when 3.5–3.10 add PlayerShip + flight + weapons"; Story 3.9 makes the pause-overlay-loss UX more impactful but does not REINTRODUCE or FIX the issue)
  - line 198 (GameState `Copy` — re-deferred; 3.9 doesn't legitimately touch state.rs)
  - line 204 (PlayerShip SemanticAccent::PlayerOwned re-tint — Story 4.5; Story 3.9 inherits same disposition for projectile-material tint, addendum to be applied per Task 9)
  - line 222 (NaN/inf guard on tuning scalars — covers projectile_speed, projectile_fire_rate_hz, projectile_ttl_seconds)
  - line 228 (range/sign validation on tuning scalars — covers projectile_fire_rate_hz ≤ 0 degenerate case)
  - line 224 (Single<&mut CursorOptions> silent skip — orthogonal; cursor-grab unchanged in 3.9)
- **No new external research needed.** All API surfaces are documented in the source files referenced above.

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (1M context)

### Debug Log References

| Command | Log file | grep `warning:|error:` | Notes |
|---|---|---|---|
| `cargo check --all-targets` | `/tmp/story-3-9-check.log` | 0 | 0.30s incremental after touching combat files |
| `cargo build` | `/tmp/story-3-9-build.log` | 0 | 3.37s |
| `cargo test` | `/tmp/story-3-9-test.log` | 0 | 41 passed; 0 failed (= 36 baseline + 5 new) |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-3-9-clippy.log` | 0 | 0.84s |
| `cargo fmt --all -- --check` | `/tmp/story-3-9-fmt.log` | 0 | initial flagged 1 long-line drift; `cargo fmt --all` applied; rerun clean (exit 0) |
| `cargo build --release` | `/tmp/story-3-9-release.log` | 0 | 4m 23s (comparable to 3.8's 4m 18s — no regression) |
| `git diff --stat Cargo.lock` | — | — | empty (no transitive-dep churn) |
| `git diff --stat Cargo.toml` | — | — | empty (no new external deps) |

### Completion Notes List

- AC #1 ✅ — `src/combat/{mod,components,input,projectiles}.rs` all created. `CombatPlugin` registered in `main.rs` between `FlightPlugin` and `PausePlugin`. `InputManagerPlugin::<CombatAction>` registered exactly once.
- AC #2 ✅ — `attach_combat_to_player_ship` runs in `OnEnter(Arena)` inside `CombatSystems::Setup`, chained AFTER `FlightSystems::Setup` via `(FlightSystems::Setup, CombatSystems::Setup).chain()`. `src/flight/**` UNCHANGED — verified via `git status --short`.
- AC #3 ✅ — TuningConfig extended with `projectile_speed: 120.0`, `projectile_fire_rate_hz: 4.0`, `projectile_ttl_seconds: 3.0`; per-field `#[serde(default = "default_…")]`; 3 helpers added; Default impl extended; tuning.ron extended (12 → 15 lines); 3 in-place test extensions (9 new assertions; ron-bytes literal extended in deserialize test).
- AC #4 ✅ — `fire_primary_weapon` system implemented per spec: cooldown decrement, fire-condition check, spawn-data computation via `*transform.forward()`, projectile spawn, cooldown reset to `1.0 / projectile_fire_rate_hz`, single `info!` per fire (gated by cooldown to ≤ 4 Hz). Compile-time constants `PROJECTILE_SPAWN_OFFSET = 3.0` and `PROJECTILE_RADIUS = 0.2` declared at top of file.
- AC #5 ✅ — Projectile spawn tuple is exactly 8 components in spec'd order. Mesh via `Sphere::new(PROJECTILE_RADIUS).mesh().ico(2).expect(...)`. Material via `ToonMaterial { tint: color_for(SemanticAccent::Neutral).into(), ..default() }`. `ArenaEntity` marker present. NO `OutlineVolume`, NO `SemanticAccent` Component, NO `CollisionLayers` (deferred per spec).
- AC #6 ✅ — `projectile_initial_velocity` pure helper authored verbatim per spec. No clamping, no normalization, no NaN guarding.
- AC #7 ✅ — `tick_projectile_ttl` system decrements `ttl` by `time.delta_secs()` and despawns at `<= 0.0`. NO per-tick log.
- AC #8 ✅ — `CombatPlugin::build` registration matches spec block exactly. `Fire → Lifecycle` chain established via `configure_sets`. NO `Update`-schedule registration.
- AC #9 ✅ — 4 helper tests in `combat/projectiles.rs` + 1 component test in `combat/components.rs` = 5 net new tests. Full project test count: **41**.
- AC #10 ✅ — Till confirmed all sub-checks green (a)–(n) in runtime smoke 2026-05-05. Pause-roundtrip respawn defect surfaced during smoke; fixed in-scope (see Implementation deviation 2 below) and re-smoked clean.
- AC #11 ✅ — all 6 commands grep `warning:|error:` = 0; test count = 41; Cargo.lock + Cargo.toml unchanged. `git status --short` matches spec'd file set + the 2 additional pause-fix files (`src/arena/mod.rs`, `src/flight/mod.rs`) per Implementation deviation 2; the only out-of-spec entry is `?? .claude/scheduled_tasks.lock` which is a pre-existing session artifact unrelated to Story 3.9.
- AC #12 ✅ — pause-cycle invariants confirmed (PrimaryWeaponCooldown preserved, projectile TTL halts during pause and resumes from frozen state, LMB-press-during-Paused does NOT queue). Additionally: pause-roundtrip now preserves PlayerShip Transform + LinearVelocity + AngularVelocity + DampenerState (regression of pre-3.9 silent defect, now fixed).
- **Implementation deviation 1:** The `#[allow(dead_code, ...)]` annotation on the `Projectile` struct was applied as a single struct-level attribute rather than per-field; functionally equivalent. Can be narrowed to per-field `damage` in a future cleanup.
- **Implementation deviation 2 — Pause-roundtrip respawn fix (2026-05-05, in-scope of 3.9):** Till's runtime smoke surfaced that Esc-pause and Cmd-Tab focus-loss caused PlayerShip + asteroids + projectiles to despawn on `OnExit(Arena)` and respawn at origin on `OnEnter(Arena)` (the cockpit camera is a child of PlayerShip, so the view "snapped back to initial"). This was a pre-existing Story 3.4 architectural defect made visible by 3.9 (projectiles disappearing on pause was the smoking gun); the prior `deferred-work.md:217` Story 3.6 AC #9 false-positive verdict only verified "no double-spawn", missing the despawn-respawn cycle. **Fix applied in 3.9 (5-line change across 3 files):**
  - `src/arena/mod.rs`: replaced `OnEnter(Arena) → spawn_arena_zone` with `OnTransition { exited: MainMenu, entered: Arena } → spawn_arena_zone`. Replaced `OnExit(Arena) → cleanup_on_exit::<ArenaEntity>` with `OnTransition { exited: Arena, entered: MainMenu } → cleanup_on_exit::<ArenaEntity>` (forward-compat for Story 4.7 title-screen-restart flow; cleanup branch dormant until then). Updated `configure_sets` accordingly.
  - `src/flight/mod.rs`: replaced `OnEnter(Arena) → spawn_player_ship` with `OnTransition { exited: MainMenu, entered: Arena } → spawn_player_ship`. Cursor systems (`grab_cursor_for_arena` on OnEnter, `release_cursor_on_arena_exit` on OnExit) intentionally kept on the broader Arena enter/exit hooks — releasing the cursor during pause is desired UX (cursor visible during pause overlay; the 3-frame mouse-look-suppression on regrab handles OS cursor-warp).
  - `src/combat/mod.rs`: replaced `OnEnter(Arena) → attach_combat_to_player_ship` with `OnTransition { exited: MainMenu, entered: Arena } → attach_combat_to_player_ship`. Updated `configure_sets` ordering chain.
  - **Net effect:** Pause round-trip (Arena ↔ Paused) preserves all ArenaEntity-marked entities; only first-entry MainMenu→Arena triggers spawn. Forward-compat: future Arena→MainMenu (Story 4.7) and Arena→PostRun (Epic 4) entry/exit transitions register via additional `OnTransition` pairs.
  - **AC impact:** AC #2 wording mentioned `OnEnter(GameState::Arena)` — implementation diverges to `OnTransition { exited: MainMenu, entered: Arena }` for the systems-registration-and-ordering portion. The intent (chained ordering Arena → Flight → Combat Setup; PlayerShip exists before combat insertion) is preserved exactly. Documented as deviation; AC #2 still satisfied in spirit.
  - **Verification:** all 4 gates re-passed after the fix (check 0, test 41/41, clippy 0, fmt clean). `deferred-work.md:217-218` Story 3.6 AC #9 false-positive entry is now retroactively closed by this fix; updating the entry to reflect resolution.

### File List

**Added:**
- `src/combat/mod.rs` (52 lines — CombatPlugin + CombatSystems enum + plugin scaffolding)
- `src/combat/components.rs` (39 lines — Projectile + PrimaryWeaponCooldown + 1 unit test)
- `src/combat/input.rs` (13 lines — CombatAction enum + default_input_map)
- `src/combat/projectiles.rs` (~190 lines — projectile_initial_velocity helper + fire_primary_weapon + tick_projectile_ttl + attach_combat_to_player_ship + 4 unit tests)

**Modified:**
- `src/main.rs` (+3 lines — `mod combat;`, `use combat::CombatPlugin;`, `.add_plugins(CombatPlugin)`)
- `src/tuning/config.rs` (+~22 lines — 3 fields + 3 helpers + 3 Default literals + 9 assertions across 3 in-place tests + 1 ron-bytes literal extension)
- `assets/config/tuning.ron` (+3 lines — projectile_speed, projectile_fire_rate_hz, projectile_ttl_seconds)
- `src/arena/mod.rs` (Pause-roundtrip fix — `OnEnter/OnExit(Arena)` → `OnTransition { exited/entered }` for spawn/cleanup; cursor systems unchanged)
- `src/flight/mod.rs` (Pause-roundtrip fix — `spawn_player_ship` moved from `OnEnter(Arena)` to `OnTransition { exited: MainMenu, entered: Arena }`; cursor systems unchanged)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (status flip ready-for-dev → in-progress + last_updated bump)
- `_bmad-output/implementation-artifacts/deferred-work.md` (+1 sentence — addendum to existing :204 entry covering projectile material tint; pause-roundtrip resolution to be applied on :217 entry per code-review or follow-up)
- `_bmad-output/implementation-artifacts/3-9-weapon-firing-projectile-ballistics.md` (this file — Status flip, task checkboxes, Dev Agent Record populated)

### Change Log

| Date | Change |
|---|---|
| 2026-05-05 | Story 3.9 implementation: CombatPlugin + Projectile + PrimaryWeaponCooldown + fire_primary_weapon + tick_projectile_ttl + 3 TuningConfig fields. Test count 36 → 41. All 6 verification gates pass (cargo check / build / test / clippy / fmt / release each grep=0). Runtime smoke (AC #10 a-n + AC #12 pause-cycle) pending Till. |
| 2026-05-05 | Pause-roundtrip respawn fix (in-scope of 3.9, surfaced by Till's smoke): `OnEnter/OnExit(Arena)` for spawn/cleanup migrated to `OnTransition { exited, entered }` patterns across `arena/mod.rs`, `flight/mod.rs`, `combat/mod.rs`. Pause ↔ Arena round-trip now preserves PlayerShip + asteroids + projectiles + dampener/cooldown state. Cursor grab/release intentionally kept on `OnEnter/OnExit(Arena)` (pause-cursor-visible UX). Forward-compat for Story 4.7 Arena → MainMenu cleanup branch. Resolves the despawn-respawn cycle that the Story 3.6 AC #9 false-positive verdict missed (deferred-work.md:217 historically). |
