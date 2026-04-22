# Epic 6: Caravan Run Framework

Player flies a Caravan run from start to target destination (5–8 min) with waypoint navigation, selectable difficulty (easy/medium/hard), trigger-volume combat pockets, tractor-beam intact-asteroid pickup, boost, pay-to-shoot economy with intact > destroyed yield math, salvage banking on success, death, and abort-forfeit. **⚠️ Danger Stretch (M5):** stories sliced into weekly sub-milestones per epic-summary guidance. M-alignment: M5. FRs covered: FR6, FR7, FR11, FR13, FR17, FR29, FR30, FR31, FR32, FR33, FR34, FR35.

## Story 6.1: Caravan State + RunPlugin Skeleton

As a developer,
I want a `Caravan` GameState and a `RunPlugin` with run-lifecycle events,
So that subsequent Epic 6 stories have a common state-and-lifecycle foundation.

**Acceptance Criteria:**

**Given** `GameState` enum from Story 1.6
**When** extended
**Then** `Caravan` and `DifficultyPicker` variants are added

**Given** `src/run/mod.rs` is authored with `RunPlugin`
**When** added to main.rs
**Then** it declares `RunSystems` SystemSet, defines `CaravanEntity` marker + `cleanup_on_exit::<CaravanEntity>` on OnExit(GameState::Caravan)

**Given** events are registered
**When** inspected
**Then** `RunStarted { difficulty, start_time_sec }` and `RunEnded { outcome, salvage_banked, duration_sec }` exist
**And** `Difficulty` enum = `Easy, Medium, Hard` (parameters in Story 6.13)
**And** `RunOutcome` enum = `TargetReached, HullDepleted, Aborted`

**Given** OnEnter(GameState::Caravan) fires
**When** run-start runs
**Then** `RunStarted` is emitted
**And** `CurrentRun { start_time_sec, difficulty, salvage_accumulated: 0 }` resource is inserted

**Given** OnExit(GameState::Caravan) fires
**When** run-end runs
**Then** `RunEnded` is emitted with computed outcome and duration
**And** `CaravanEntity`-marked entities are despawned
**And** `CurrentRun` resource is removed

## Story 6.2: Arena → Caravan Gate — Tutorial-Complete Sentinel + Difficulty-Picker Stub

As a player,
I want Arena on first run only, subsequent runs starting via a difficulty picker into Caravan,
So that FR29's tutorial-once-then-main-game flow works.

**Acceptance Criteria:**

**Given** SaveData from Story 4.6
**When** extended
**Then** a `tutorial_complete: bool` (default `false`) is added
**And** `SaveData.version` is bumped to 2 with a `v1_to_v2` migration injecting the default — the first real use of Story 5.6's migration scaffold

**Given** the player destroys the first asteroid ever (first `AsteroidDestroyed` in `GameState::Arena`)
**When** the tutorial-complete system observes
**Then** `SaveData.tutorial_complete = true` is set in-memory
**And** `save(&save_data)` persists the flag

**Given** the player clicks "Start Run" from MainMenu
**When** the router decides next state
**Then** `tutorial_complete == false` → `Arena`, else → `DifficultyPicker`

**Given** OnEnter(GameState::DifficultyPicker)
**When** the picker is built
**Then** three buttons spawn (Easy / Medium / Hard) + a Back button to MainMenu
**And** entities carry `DifficultyPickerEntity` marker

**Given** the player clicks a difficulty
**When** the action fires
**Then** `PickedDifficulty(Difficulty)` resource is inserted
**And** `NextState<GameState>` = `Caravan`
**And** OnEnter(Caravan) reads `PickedDifficulty` into `CurrentRun.difficulty`

**Given** OnExit(DifficultyPicker)
**When** cleanup runs
**Then** `DifficultyPickerEntity`-marked entities are despawned

## Story 6.3: Caravan Zone Layout — Start + Target + Landmarks

As a player,
I want a hand-designed Caravan zone with clear start, distant target, and a few navigational landmarks,
So that I have perceivable 3D space with orientation cues.

**Acceptance Criteria:**

**Given** `src/run/zone.rs` with `spawn_caravan_zone` on OnEnter(Caravan)
**When** the system runs
**Then** start-point = PlayerShip spawn; target-point ~800 m away (500–1000 m bracket)
**And** the target entity carries a `CaravanTarget` marker
**And** path is non-linear — 2–3 landmark asteroid clusters hand-placed to require gentle course correction

**Given** asteroid clusters spawn
**When** placed
**Then** each cluster has 5–15 asteroids (3.0–12.0 m radius)
**And** asteroids use `ToonMaterial` + `SemanticAccent::Salvage` + `OutlineBundle`
**And** each is `RigidBody::Static` with sphere `Collider`
**And** each has `Health { current: 1, max: 1 }` and `CaravanEntity` marker

**Given** combat-pocket hooks are needed
**When** the zone spawner places hidden spawn data
**Then** 2–4 `CombatPocketSpawn { pocket_id: u32, spawn_positions: Vec<Vec3> }` entities are spawned at hand-picked positions along the path
**(Trigger logic itself is Story 6.8)**

**Given** toon shading readability
**When** OnEnter(Caravan) runs
**Then** one `DirectionalLight` with `CaravanEntity` marker is spawned

**Given** the PlayerShip spawn
**When** placed
**Then** it spawns at start-point with zero velocity, oriented toward target (so Story 6.4's pointer is initially on-screen)

## Story 6.4: Waypoint-Pointer World-Space HUD

As a player,
I want a cockpit indicator showing direction + distance to target,
So that I can navigate per FR33.

**Acceptance Criteria:**

**Given** `src/ui/waypoint.rs` integrated into `HudPlugin`
**When** OnEnter(Caravan) runs
**Then** a `WaypointPointer` entity spawns as a child of the cockpit camera
**And** the entity uses a simple 3D arrow/chevron placeholder mesh (final mesh is Epic 10 polish)
**And** it is world-space but pinned at a fixed cockpit-frame position (e.g., top-center just under windshield rim)

**Given** the waypoint pointer exists
**When** the update system runs each frame
**Then** the pointer rotates to aim from its cockpit position toward `CaravanTarget` world position
**And** rotation uses shortest-arc quaternion interpolation (no flip)

**Given** target is at distance D
**When** the distance-display system runs
**Then** a small text Node below the pointer shows "<D> m" rounded to nearest 10 m
**And** it updates each frame

**Given** the player approaches target
**When** D drops below 50 m
**Then** the pointer + distance pulse subtly (color shift or scale) as a "near" cue — arrival detection is Story 6.6

**Given** target is reached (Story 6.6)
**When** OnExit(Caravan) runs
**Then** the pointer disappears with Caravan cleanup

## Story 6.5: Salvage Currency Resource + HUD Wiring

As a player,
I want my live salvage count visible on HUD during Caravan,
So that I can track economy per FR17 and judge pay-to-shoot affordability.

**Acceptance Criteria:**

**Given** Story 3.11's "SALVAGE 0" placeholder
**When** Story 6.5 wires real data
**Then** the field reads `CurrentRun.salvage_accumulated` live each frame

**Given** `src/run/salvage.rs` is authored
**When** salvage APIs are defined
**Then**:
- `fn add_salvage(current_run: &mut CurrentRun, amount: u32, source: SalvageSource)` exposed; emits `SalvageAdded` event
- `fn spend_salvage(current_run: &mut CurrentRun, amount: u32) -> Result<(), InsufficientFunds>` exposed
- `SalvageSource` enum = `IntactCapture, DestroyedAsteroid, Other`
- `InsufficientFunds` is a simple marker error type

**Given** `spend_salvage` is called with amount > `salvage_accumulated`
**When** returned
**Then** `Err(InsufficientFunds)` is returned and nothing is modified

**Given** HUD reflects the resource
**When** accumulation or spending occurs
**Then** the HUD updates with at most one-frame lag (reactive, not cached stale)

## Story 6.6: Caravan Duration + RunCompleted on Target Reached

As a player,
I want the run to complete when I reach the target within 5–8 min,
So that FR30's duration commitment has a clear success condition.

**Acceptance Criteria:**

**Given** `CaravanTarget` from Story 6.3
**When** Story 6.6 is implemented
**Then** an Avian sensor `Collider` (sphere, radius 20 m, `TuningConfig.target_trigger_radius`) is attached
**And** the sensor emits Avian collision events on PlayerShip entry

**Given** PlayerShip enters the target sensor
**When** target-reached system observes
**Then** salvage is banked via Story 6.7's banking system
**And** `RunEnded { outcome: TargetReached, salvage_banked, duration_sec }` is emitted
**And** `NextState<GameState>` = `PostRun`

**Given** arrival in <5 min or >8 min
**When** run-ended system logs
**Then** an `info!` log records actual duration (FR30 is design target, not hard constraint — tuning via zone layout Story 6.3)

**Given** the player presses Esc during Caravan
**When** the abort path is triggered
**Then** `NextState<GameState>` = `MainMenu`
**And** `RunEnded { outcome: Aborted, .. }` is emitted
**And** abort processing proceeds to Story 6.7 (forfeit, no banking)

## Story 6.7: Salvage Banking on Run End — Success + Death + Abort

As a player,
I want salvage banked into meta-currency on run end, with different policies per outcome,
So that FR34/FR35 progression commitments and abort-anti-abuse both hold.

**Acceptance Criteria:**

**Given** Epic 7's `meta_currency: u32` placeholder in SaveData (Story 4.6)
**When** Story 6.7 banks
**Then** `SaveData.meta_currency += CurrentRun.salvage_accumulated * banking_rate`
**And** `banking_rate: f32` from `TuningConfig.salvage_banking_rate = 1.0` default (Epic 7 may retune)

**Given** `RunEnded { outcome: TargetReached }`
**When** banking runs
**Then** full salvage banks, `save(&save_data)` persists

**Given** `RunEnded { outcome: HullDepleted }`
**When** banking runs
**Then** salvage STILL banks at full rate (FR35 no-death-penalty)
**And** `save(&save_data)` persists

**Given** `RunEnded { outcome: Aborted }` (Esc → Menu)
**When** banking runs
**Then** salvage is NOT banked (forfeit — prevents Esc-abuse to dodge death)
**And** an `info!` log records "run aborted, salvage forfeited"

**Given** FR35's "meta-unlocks persist" clause
**When** banking runs
**Then** `SaveData.unlocked_upgrades` is untouched
**And** a unit test verifies: start with 3 unlocks → run ends (any outcome) → reload → still 3 unlocks

## Story 6.8: Combat-Pocket Trigger System

As a player,
I want enemies to ambush me at predetermined points when I get close,
So that the route feels populated per FR32.

**Acceptance Criteria:**

**Given** `CombatPocketSpawn` entities from Story 6.3
**When** Story 6.8 is implemented
**Then** each has an Avian sensor `Collider` (sphere, radius `TuningConfig.pocket_trigger_radius: f32 = 150.0`)
**And** the sensor detects PlayerShip entry

**Given** PlayerShip enters a pocket trigger
**When** trigger-fire system observes
**Then** the pocket's `triggered: bool` flag is set (prevents retrigger)
**And** for each spawn position in `spawn_positions`, one `EnemyShip` entity spawns at that position (clamped to [`pocket_enemy_count_min = 1`, `pocket_enemy_count_max = 3`, `spawn_positions.len()`])
**And** spawned enemies carry `CaravanEntity` marker

**Given** enemies spawn
**When** Story 4.2's `EnemyAiState` system runs
**Then** newly-spawned enemies start in `Idle` and transition normally

**Given** a pocket was triggered
**When** the player leaves and re-enters
**Then** no new enemies spawn (`triggered == true` persists)
**And** existing enemies remain until destroyed

## Story 6.9: Pay-to-Shoot Economy

As a player,
I want each shot to cost salvage,
So that I have economic tension per FR11.

**Acceptance Criteria:**

**Given** Story 3.9 / 4.4's primary-fire flow
**When** Story 6.9 updates pre-fire
**Then** `spend_salvage(&mut current_run, shot_cost)` is called from Story 6.5 API
**And** `shot_cost: u32` is archetype-specific from `TuningConfig.shot_cost_per_weapon` (Pulse=1, Shotgun=3, Railgun=5)

**Given** `spend_salvage` returns `Ok(())`
**When** fire proceeds
**Then** the projectile spawns normally (all existing fire mechanics preserved)
**And** a `SalvageSpent { amount, reason: SpendReason::Shot }` event is emitted (for future analytics / Epic 10 tuning)

**Given** `spend_salvage` returns `Err(InsufficientFunds)`
**When** fire aborts
**Then** no projectile spawns
**And** the SALVAGE HUD field flashes red for 0.3 s
**And** no weapon audio / fire animation plays (no phantom feedback)

**Given** 0-salvage bankruptcy per Till's decision 2026-04-22
**When** the player is at 0 salvage with enemies pursuing
**Then** no shots are possible until salvage is earned (Story 6.11 yields)
**And** this is a design feature (tension, tactical choice)

**Given** `GameState::Arena` (tutorial)
**When** the player fires in Arena
**Then** pay-to-shoot is DISABLED — shots fire without salvage check
**And** gating is via a `ShotEconomyActive: bool` resource, set true on OnEnter(Caravan), false on OnExit

## Story 6.10: Tractor-Beam Intact Asteroid Capture

As a player,
I want a hold-to-tractor beam that pulls an asteroid until it collides with me for intact capture,
So that FR7's intact-capture mechanic gives me a higher-yield alternative to shooting.

**Acceptance Criteria:**

**Given** `CombatAction` from Story 3.9
**When** extended
**Then** `HoldTractorBeam` is added bound to E key (or F — implementation choice)

**Given** `src/run/tractor.rs` is authored
**When** systems are registered
**Then** a target-acquisition system runs each frame:
- While held, find the asteroid whose direction-from-player is closest to `aim_direction` (or `ship_forward_vector` if not in decoupled aim), distance ≤ `TuningConfig.tractor_range: f32 = 60.0`
- Best candidate is stored in a `TractorTarget(Option<Entity>)` Resource
- No candidate → `TractorTarget = None`

**Given** `TractorTarget = Some(asteroid)`
**When** the pull system runs in FixedUpdate
**Then** an `ExternalForce` magnitude `TuningConfig.tractor_force: f32 = 500.0` is applied toward PlayerShip
**And** if the asteroid is `RigidBody::Static`, it is converted to `RigidBody::Dynamic` one-time on first engagement (so it can actually move)

**Given** a tractored asteroid is being pulled
**When** VFX system runs
**Then** a world-space line (bevy_gizmos or simple cylinder mesh) connects PlayerShip → asteroid
**And** the line pulses or flows as a "beam active" placeholder cue (final VFX Epic 10)

**Given** a tractored asteroid contacts the PlayerShip
**When** the capture system observes the Avian collision
**Then** `AsteroidCaptured { asteroid, size_category }` is emitted (past-tense PascalCase)
**And** `SizeCategory` = `Small / Medium / Large` computed from mesh radius with thresholds in `TuningConfig`
**And** the asteroid is despawned
**And** yield calc (Story 6.11) fires

**Given** the player releases `HoldTractorBeam`
**When** the pull system observes no-hold
**Then** ExternalForce on the formerly-tractored asteroid is removed
**And** the asteroid keeps its current velocity (no snap-stop)
**And** the visual beam disappears
**And** the asteroid stays `RigidBody::Dynamic` (once-dynamic-always-dynamic policy — simpler)

## Story 6.11: Intact Capture vs Destroyed Yield Math

As a player,
I want intact-captured asteroids to yield strictly more than destroyed ones,
So that FR13's tactical incentive to tractor-capture is real.

**Acceptance Criteria:**

**Given** `TuningConfig` is extended
**When** yield constants are defined
**Then** six yields loaded (captured > destroyed invariant per size):
- `yield_captured_small = 5`, `yield_captured_medium = 15`, `yield_captured_large = 40`
- `yield_destroyed_small = 2`, `yield_destroyed_medium = 6`, `yield_destroyed_large = 15`

**And** a `#[cfg(test)]` unit test asserts `yield_captured_<size> > yield_destroyed_<size>` for every size

**Given** `AsteroidDestroyed` from Story 3.10 / 4.2
**When** Story 6.11 extends handling
**Then** the asteroid's SizeCategory is read from its mesh radius
**And** `add_salvage(current_run, yield_destroyed_<size>, SalvageSource::DestroyedAsteroid)` is called

**Given** `AsteroidCaptured` from Story 6.10
**When** handling runs
**Then** `add_salvage(current_run, yield_captured_<size>, SalvageSource::IntactCapture)` is called

**Given** the yield-delta indicator (FR25 / Epic 8)
**When** Story 6.11 exposes data
**Then** a `PotentialYield { destroy: u32, capture: u32 }` component is attached to each asteroid on spawn (Epic 8 renders it as world-space cockpit HUD)

**Given** `GameState::Arena` (tutorial)
**When** an asteroid is destroyed in Arena
**Then** NO salvage is added — `ShotEconomyActive` resource gates the salvage-add path (same gate as Story 6.9's shot-cost)

## Story 6.12: Boost + Rechargeable Resource

As a player,
I want a boost mechanic granting a short thrust burst off a rechargeable resource,
So that I have a mobility option per FR6.

**Acceptance Criteria:**

**Given** `FlightAction` from Story 3.6
**When** extended
**Then** `ActivateBoost` is added bound to Left Shift

**Given** PlayerShip is extended
**When** the boost component is added
**Then** `BoostResource { current: f32 = 1.0, max: f32 = 1.0, recharge_rate_per_sec: f32 = 0.2, active_duration_sec: f32 = 2.0, active_remaining_sec: f32 = 0.0 }` is on the PlayerShip
**And** defaults come from `TuningConfig.boost_*`

**Given** the player presses `ActivateBoost` while `current >= 1.0`
**When** the boost system observes in FixedUpdate
**Then** `active_remaining_sec = active_duration_sec`, `current = 0.0`
**And** a `BoostActivated { entity }` event is emitted

**Given** `active_remaining_sec > 0`
**When** Story 3.6's thrust system reads boost state
**Then** `ship_thrust_newtons` is multiplied by `TuningConfig.boost_thrust_multiplier: f32 = 3.0`
**And** `active_remaining_sec -= delta` each frame
**And** when `active_remaining_sec ≤ 0`, multiplier returns to 1.0

**Given** `active_remaining_sec == 0` AND `current < max`
**When** the recharge system runs
**Then** `current += recharge_rate_per_sec * delta`, clamped to `max`

**Given** the player presses `ActivateBoost` while `current < 1.0`
**When** activation is checked
**Then** the action is rejected (no event, no effect)

**Given** HUD needs a boost indicator per Till's decision 2026-04-22
**When** Story 6.12 renders UI
**Then** a small bar is rendered as a world-space element on the cockpit frame (not screen-space corner)
**And** fill reflects `current / max`, colored green when ready, amber dim while recharging or active

## Story 6.13: Difficulty Variant System — Easy / Medium / Hard

As a player,
I want three difficulty variants that meaningfully change the run experience,
So that I can pick a challenge level per FR31.

**Acceptance Criteria:**

**Given** `Difficulty` enum from Story 6.1
**When** Story 6.13 finalizes
**Then** `Easy / Medium / Hard` variants are the finalized three
**And** `PickedDifficulty` is read on OnEnter(Caravan) into `CurrentRun.difficulty`

**Given** `TuningConfig` is extended with per-difficulty multipliers
**When** Caravan systems read them
**Then**:
- **Easy**: `enemy_health_mult = 0.75`, `enemy_damage_mult = 0.75`, `enemy_fire_rate_mult = 0.75`, `pocket_enemy_count_mult = 0.75`, `yield_mult = 1.25`
- **Medium**: all multipliers = 1.0 (baseline = existing TuningConfig values)
- **Hard**: `enemy_health_mult = 1.5`, `enemy_damage_mult = 1.5`, `enemy_fire_rate_mult = 1.5`, `pocket_enemy_count_mult = 1.5`, `yield_mult = 0.75`

**Given** enemies spawn in Caravan
**When** Story 6.8's pocket-spawn runs
**Then** count is `(base_count * pocket_enemy_count_mult).floor()`, clamped to [1, spawn_positions.len()]

**Given** an enemy is spawned
**When** stats are applied
**Then** `Health.max` and `Health.current` are scaled by `enemy_health_mult`
**And** projectile damage dealt is scaled by `enemy_damage_mult`
**And** fire rate is scaled by `enemy_fire_rate_mult`

**Given** salvage yields
**When** yield is added via Story 6.11
**Then** final yield is `(base_yield * yield_mult).floor()`

**Given** the three variants
**When** a full Caravan is played at each difficulty
**Then** runs feel tangibly different; defaults above are starting-point tuning to be refined via playtest at M5 gate

<!-- Epic 6 complete — 13 stories deliver M5 Caravan Run Framework (Danger Stretch). Next epic to decompose: Epic 7 (Roguelite Loop / M6 EA-Viable). -->
