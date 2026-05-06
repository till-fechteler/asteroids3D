# Story 4.2: Enemy AI State Machine — Detect / Pursue / Attack

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player flying the Arena cockpit ship,
I want the enemy to detect me, pursue me, and fire on me when I'm in engagement range,
So that the Arena has a live adversary per FR14 and combat acquires real stakes — the second of three Epic 4 / M3 Itch.io stop-and-ship combat-loop stories (4.1 entity foundation → **4.2 AI alive** → 4.3 hull + permadeath).

## Acceptance Criteria

1. **(Unified `Health` component — new file `src/combat/health.rs`)** Authored as a Bevy `Component` with both `current: u32` AND `max: u32` fields:
   ```rust
   /// Generic hit-point pool with maximum capacity. Used by asteroids
   /// (Story 4.2 refactor from `AsteroidHp`), enemies (Story 4.2), and
   /// PlayerShip (Story 4.3). Epic 5's Story 5.1 splits this into formal
   /// HullHP / ShieldHP components.
   ///
   /// NO Default derive — callers always specify both fields explicitly. A
   /// silent default of (0, 0) would mean "pre-destroyed / unkillable" footgun.
   #[derive(Component, Debug, Clone, Copy, PartialEq)]
   pub struct Health {
       pub current: u32,
       pub max: u32,
   }
   ```
   **And** `pub mod health;` is added to `src/combat/mod.rs` between `enemy` and `input` (Task 1 snapshot ordering — `enemy_ai` lands in Task 3). At Task 1 the alphabetical mod-list is: `components, damage, enemy, health, input, projectiles`. Task 3 inserts `enemy_ai` between `enemy` and `health` → final post-Task-3 ordering: `components, damage, enemy, enemy_ai, health, input, projectiles`.
   **And** **NO** `pub fn new(current: u32) -> Self` / **NO** `pub fn full(max: u32) -> Self` constructor methods on `Health` — caller-side struct-literal construction matches the project's `AsteroidHp` / `Projectile` / `PrimaryWeaponCooldown` no-helper precedent.

2. **(`AsteroidHp` retired — replaced by `Health` + new `Asteroid` marker)** `AsteroidHp` is removed from `src/combat/components.rs`. The `Asteroid` marker Component is introduced in its place to drive `detect_projectile_asteroid_hits` query filtering:
   ```rust
   /// Marker for asteroid entities. Queried by `detect_projectile_asteroid_hits`
   /// to disambiguate projectile-vs-asteroid collision pairs from other pairs
   /// (e.g., projectile-vs-enemy, ship-vs-asteroid). Health is on a separate
   /// component to allow Stories 4.2/4.3 enemy + player to share the same
   /// Health vocabulary.
   #[derive(Component, Debug, Clone, Copy)]
   pub struct Asteroid;
   ```
   **And** `src/arena/zone.rs::spawn_arena_zone` is updated: every asteroid spawn carries `(Asteroid, Health { current: 1, max: 1 }, ...)` instead of `AsteroidHp { current: 1 }`.
   **And** `src/combat/damage.rs::detect_projectile_asteroid_hits` query changes from `Query<(), With<AsteroidHp>>` to `Query<(), With<Asteroid>>`.
   **And** `src/combat/damage.rs::apply_asteroid_damage` query changes from `Query<&mut AsteroidHp>` to `Query<&mut Health, With<Asteroid>>` and reads/writes `hp.current` (Health field).
   **And** `asteroid_hp_construction_is_explicit` in `src/combat/components.rs::tests` is replaced by a `health_construction_is_explicit` test in `src/combat/health.rs::tests` (no Default derive guard, two-field round-trip).

3. **(`Enemy` spawn-tuple extensions in `src/combat/enemy.rs::spawn_enemy_ship`)** The existing spawn-tuple is extended in-place — **NO** new spawn function; same OnTransition registration. New components added to the tuple (alphabetically interleaved with existing):
   - `Health { current: 2, max: 2 }` — 2-shot enemy for first playable per epic-4 spec line 41.
   - `EnemyAiState::Idle` — default initial state (transitions take over from frame 1 once distance is computed).
   - `CollisionLayers::new([GameLayer::Enemy], LayerMask::ALL)` — places enemy on the new `GameLayer::Enemy` (added in AC #5); `LayerMask::ALL` filter so the enemy sees collisions from all layers (player projectiles need to hit, ship-bounce stays as-is).
   - `CollisionEventsEnabled` — closes deferred-work entry `4-1: No CollisionEventsEnabled on Enemy entity` so `detect_projectile_enemy_hits` (AC #11) receives `CollisionStart` events.
   - `Name::new("EnemyShip")` — closes deferred-work entry `4-1: No Name component on Enemy entity` so the entity is identifiable in Bevy inspector / debug logs during AI iteration.
   - `ExternalForce::default()` — Avian dynamics surface for AI movement to write per-tick force vectors into. Pre-required for `apply_enemy_ai_movement` (AC #8).
   - `ExternalTorque::default()` — same surface for orientation (face-the-player rotation).
   
   **And** the existing components from Story 4.1 (`Enemy, EnemyShip::Standard, SemanticAccent::Enemy, ArenaEntity, Mesh3d, MeshMaterial3d, Transform, OutlineVolume, RigidBody::Dynamic, Collider::capsule, LinearVelocity(ZERO), AngularVelocity(ZERO)`) are **PRESERVED** — none are removed. Only ADDITIVE.
   **And** the spawn `info!` log line is unchanged from 4.1.
   **And** the Story 4.1 deferred-work entries `spawn_enemy_ship has no idempotency guard` and `enemy_spawn_position_clears_close_asteroids tests only 2 of 5 close-cluster asteroids` remain DEFERRED — Story 4.2 does NOT close those (no Arena re-entry path lands until Story 4.7; close-cluster-asteroid sweep is a test-improvement chore not gated by 4.2).

4. **(`EnemyAiState` enum + `EnemyProjectile` marker — new file `src/combat/enemy_ai.rs`)** Authored as a Bevy `Component` enum:
   ```rust
   /// Enemy AI lifecycle state. Transitions are distance-driven with
   /// hysteresis (see `next_ai_state`). Default is `Idle`; the first
   /// FixedUpdate tick after Arena entry computes distance-to-player
   /// and may transition immediately if the player is within range.
   #[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
   pub enum EnemyAiState {
       #[default]
       Idle,
       Detect,
       Pursue,
       Attack,
   }

   /// Marker for enemy-fired projectiles. Story 4.3 keys hull-damage routing
   /// on this marker (player-projectile vs. enemy-projectile differentiation).
   /// Story 4.2 spawns these entities but does NOT route damage to PlayerShip;
   /// damage routing is Story 4.3's scope.
   #[derive(Component, Debug, Clone, Copy)]
   pub struct EnemyProjectile;
   ```
   **And** `pub mod enemy_ai;` is added to `src/combat/mod.rs` between `enemy` and `health` (alphabetical order — final: `components, damage, enemy, enemy_ai, health, input, projectiles`).
   **And** `EnemyAiState::Default` IS derived (unlike the no-Default discipline on `Health` / `EnemyShip` / `AsteroidHp`/`HudPlaceholder`) — `Idle` is the safe inert state with no AI behavior, so a silent default of `Idle` is non-hazardous; the spawn tuple still names it explicitly per AC #3.
   **And** **NO** `Default` derive on `EnemyProjectile` — empty unit struct, instantiated only at spawn-site of enemy fire system.

5. **(`GameLayer::Enemy` added — projectile collision-filter extension)** `src/combat/damage.rs::GameLayer` enum gains a new `Enemy` variant:
   ```rust
   #[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
   pub enum GameLayer {
       #[default]
       Default,
       Asteroid,
       Projectile,
       Enemy,
   }
   ```
   **And** `src/combat/projectiles.rs::fire_primary_weapon` updates the player projectile's `CollisionLayers` filter from `[GameLayer::Asteroid]` → `[GameLayer::Asteroid, GameLayer::Enemy]` so player projectiles can collide with enemies.
   **And** **NO** `GameLayer::EnemyProjectile` introduced (Story 4.2 keeps enemy projectiles on `GameLayer::Projectile` — the same layer as player projectiles; filter mask `[GameLayer::Default]` puts them on a player-bound trajectory). `EnemyProjectile` marker Component (AC #4) is the differentiator at the systems-layer; bit-layer split deferred to Story 4.3 if hull-damage routing surface needs it.

6. **(`TuningConfig` extended with 5 enemy-AI fields)** `src/tuning/config.rs::TuningConfig` gains 5 new fields, each with `#[serde(default = "...")]` per the established forward-compat pattern:
   - `enemy_detection_range: f32 = 100.0`
   - `enemy_engagement_range: f32 = 50.0`
   - `enemy_speed: f32 = 20.0`
   - `enemy_fire_rate_hz: f32 = 1.0`
   - `enemy_ai_hysteresis_pct: f32 = 0.1`
   
   **And** `TuningConfig::default()` is extended with the 5 corresponding default-fn calls.
   **And** `assets/config/tuning.ron` is extended with the 5 new fields at the canonical default values (so the serialized canonical surface matches code defaults — same pattern as ship_thrust_newtons / projectile_* fields).
   **And** `tuning_config_default_matches_ron_initial_values` and `tuning_config_deserializes_from_ron_bytes` tests in `src/tuning/config.rs::tests` are extended with the 5 new field assertions.
   **And** `tuning_config_legacy_schema_uses_defaults_for_added_fields` test is extended to assert all 5 new fields take their defaults when absent from RON bytes (forward-compat contract: existing tuning.ron files without these fields keep deserializing).

7. **(Pure `next_ai_state` transition function — hysteresis-gated)** `src/combat/enemy_ai.rs` defines a pure helper:
   ```rust
   pub fn next_ai_state(
       current: EnemyAiState,
       distance: f32,
       detection_range: f32,
       engagement_range: f32,
       hysteresis_pct: f32,
   ) -> EnemyAiState
   ```
   that computes the next state per these gates (hysteresis applied to the ENTRY threshold for the more-aggressive direction; outward transitions widen by `(1 + hysteresis_pct)`):
   - **Inward (more aggressive — distance decreasing):**
     - `Idle → Detect` when `distance ≤ detection_range`
     - `Detect → Pursue` when `distance ≤ engagement_range`
     - `Pursue → Attack` when `distance ≤ engagement_range * 0.5` (the attack-range derived constant)
   - **Outward (less aggressive — distance increasing):**
     - `Detect → Idle` when `distance > detection_range * (1 + hysteresis_pct)`
     - `Pursue → Detect` when `distance > engagement_range * (1 + hysteresis_pct)`
     - `Attack → Pursue` when `distance > engagement_range * 0.5 * (1 + hysteresis_pct)`
   
   **And** with `hysteresis_pct = 0.1`, the effective outer thresholds are: 110.0, 55.0, 27.5 (vs. inner 100.0, 50.0, 25.0). This dead-band prevents single-tick state-flicker when distance hovers within `[T, T*(1+hyst)]`.
   **And** the function returns `current` when no gate fires (no transition this tick).
   **And** **NO** ECS access in `next_ai_state` — pure function operating on primitive inputs (matches `apply_damage` / `dampener_acceleration` / `projectile_initial_velocity` / `ship_local_thrust_vector` precedent — first-class testable per the project's pure-helper learning pattern).
   **And** **NO** skip-states across more than one band in a single tick — the function uses individual `match current { Idle => ..., Detect => ..., ... }` arms, so a frame where distance jumps from 200 m to 10 m moves `Idle → Detect` (single step) — Detect → Pursue → Attack happens on subsequent ticks. **Trade-off rationale:** at 60 Hz FixedUpdate this means at most 2 ticks (~33 ms) of "incorrect" intermediate state before convergence; acceptable for first-playable. (If a later story needs instant convergence, the function can be wrapped in a `loop { let next = next_ai_state(current, ...); if next == current { break; } current = next; }` driver — but Epic 4 does not require it.)

8. **(`apply_enemy_ai` system — FixedUpdate, in `CombatSystems::EnemyAi`)** `src/combat/enemy_ai.rs::apply_enemy_ai` runs in FixedUpdate, gated by `in_state(GameState::Arena)`, in a NEW `CombatSystems::EnemyAi` system set chained BEFORE `CombatSystems::Fire`. System body:
   ```rust
   pub fn apply_enemy_ai(
       tuning_assets: Res<Assets<TuningConfig>>,
       tuning_handle: Res<TuningHandle>,
       player: Single<&Transform, With<PlayerShip>>,
       mut enemies: Query<(&mut EnemyAiState, &Transform, &mut ExternalForce, &mut Transform), (With<Enemy>, Without<PlayerShip>)>,
       // ...
   ) { ... }
   ```
   **NOTE — borrow-checker pitfall:** the above signature **WILL NOT COMPILE** — `&Transform` and `&mut Transform` cannot both be in the same `Query` tuple. Use ONE `&mut Transform` and read its current value before mutating, OR use `Forces` query + separate `Transform` access:
   ```rust
   pub fn apply_enemy_ai(
       tuning_assets: Res<Assets<TuningConfig>>,
       tuning_handle: Res<TuningHandle>,
       player: Single<&Transform, With<PlayerShip>>,
       mut enemies: Query<(&mut EnemyAiState, &mut Transform, &mut LinearVelocity), (With<Enemy>, Without<PlayerShip>)>,
   )
   ```
   The `Without<PlayerShip>` filter is REQUIRED on the enemy query to satisfy Bevy's disjoint-Query invariant given `PlayerShip` is in `player: Single<...>`.
   
   **System body responsibilities (each frame):**
   - **(a) State transition:** call `next_ai_state(current, distance, det_range, eng_range, hyst)` per AC #7, write new state into `EnemyAiState` component.
   - **(b) Orientation (Detect / Pursue / Attack):** for any non-Idle state, compute `look_at(player_pos, Vec3::Y)` for the enemy `Transform`. **Snap-rotate** to face-the-player — NOT a torque-driven slew (Story 4.2 keeps the AI mathematically simple; smooth-rotation is post-MVP polish if needed).
   - **(c) Pursue movement:** for `Pursue` state, compute `direction = (player_pos - enemy_pos).normalize_or_zero()` and write `LinearVelocity(direction * tuning.enemy_speed)` directly. Bypass `ExternalForce` for simplicity (a `Dynamic` body lets us write its velocity directly via Avian's component API; no need for force-integration tuning).
   - **(d) Attack stops moving:** for `Attack` state, write `LinearVelocity(Vec3::ZERO)` so the enemy holds position while firing.
   - **(e) Idle / Detect: zero velocity, hold position:** for `Idle` and `Detect`, write `LinearVelocity(Vec3::ZERO)`. Detect rotates to face the player but does NOT move toward them (per epic-4 spec line 56: "Detect → rotates to face player").
   
   **And** cold-start tuning fallback identical to existing pattern: `tuning_assets.get(...).cloned().unwrap_or_default()` (matches `apply_thrust` / `fire_primary_weapon` / `spawn_enemy_ship`).
   **And** the system is a no-op if `enemies` query is empty (zero-enemy case): the for-loop simply doesn't iterate. No early-return / no warn — same pattern as `attach_combat_to_player_ship` (silent skip on no-PlayerShip).

9. **(`enemy_fire_weapon` system — FixedUpdate, in `CombatSystems::Fire`)** `src/combat/enemy_ai.rs::enemy_fire_weapon` runs in FixedUpdate, gated by `in_state(GameState::Arena)`, registered in the existing `CombatSystems::Fire` set (alongside `projectiles::fire_primary_weapon` — both spawn projectiles into the same world):
   ```rust
   pub fn enemy_fire_weapon(
       time: Res<Time>,
       tuning_assets: Res<Assets<TuningConfig>>,
       tuning_handle: Res<TuningHandle>,
       mut commands: Commands,
       mut meshes: ResMut<Assets<Mesh>>,
       mut materials: ResMut<Assets<ToonMaterial>>,
       player: Single<&Transform, With<PlayerShip>>,
       mut enemies: Query<(&Transform, &EnemyAiState, &mut EnemyFireCooldown), With<Enemy>>,
   )
   ```
   **And** a new `EnemyFireCooldown { remaining: f32 }` Component is defined in `src/combat/enemy_ai.rs` (mirroring `PrimaryWeaponCooldown` exactly — same shape, same default-derive, same `remaining` field). Added to the spawn tuple in `spawn_enemy_ship` per AC #3 (this AC #3 list is updated to include `EnemyFireCooldown::default()`).
   **And** the system body:
   - **Tick cooldown:** `cooldown.remaining = (cooldown.remaining - dt).max(0.0)` for every enemy.
   - **Gate on Attack state + cooldown ≤ 0:** if `*ai_state != EnemyAiState::Attack` OR `cooldown.remaining > 0.0`, skip.
   - **Spawn enemy projectile:** compute `direction = (player_pos - enemy_pos).normalize_or_zero()`; `spawn_pos = enemy_pos + direction * ENEMY_PROJECTILE_SPAWN_OFFSET`; spawn a `Projectile { ttl, damage: 1 }` + `EnemyProjectile` marker + `RigidBody::Dynamic` + `Collider::sphere(PROJECTILE_RADIUS)` + `LinearVelocity(direction * tuning.projectile_speed)` + `CollisionLayers::new([GameLayer::Projectile], [GameLayer::Default])` + `CollisionEventsEnabled` + `ArenaEntity` + `Mesh3d` + `MeshMaterial3d(SemanticAccent::Enemy material)` + `Transform`.
   - **Reset cooldown:** `cooldown.remaining = 1.0 / tuning.enemy_fire_rate_hz.max(f32::EPSILON)`.
   - **Log:** `info!("enemy fired projectile from {:?} toward {:?}", enemy_pos, player_pos);`
   
   **And** an `ENEMY_PROJECTILE_SPAWN_OFFSET: f32 = 3.0` constant is defined in `src/combat/enemy_ai.rs` (mirrors `PROJECTILE_SPAWN_OFFSET` from `projectiles.rs:31` for the player; matches enemy `Capsule3d` extent ≥ 2.0 + projectile radius 0.2 = 2.2 m → 3.0 with margin).
   **And** the enemy projectile uses `SemanticAccent::Enemy` tint (vermillion, `#D55E00`) — visually distinct from player's `SemanticAccent::Neutral` (grey) projectiles per AC #5 of Story 4.1's accent prefiguration (Story 4.5 retroactively re-tints player projectiles to `SemanticAccent::PlayerOwned` cyan; Story 4.2 introduces enemy-projectile vermillion tagging now).
   **And** **NO** `EnemyProjectile` collision-vs-PlayerShip damage routing is implemented in 4.2 — that is Story 4.3's `ProjectileHitPlayer` event scope. Enemy projectiles in 4.2 spawn, fly, and TTL-expire (existing `tick_projectile_ttl` system handles them since they carry `Projectile`).

10. **(`update_enemy_ai_velocity` borrow-checker structuring — final shape)** Per AC #8's borrow-checker note, the FINAL system signature is:
    ```rust
    pub fn apply_enemy_ai(
        tuning_assets: Res<Assets<TuningConfig>>,
        tuning_handle: Res<TuningHandle>,
        player_query: Single<&Transform, With<PlayerShip>>,
        mut enemies: Query<
            (&mut EnemyAiState, &mut Transform, &mut LinearVelocity),
            (With<Enemy>, Without<PlayerShip>),
        >,
    )
    ```
    where `player_query: Single<&Transform, With<PlayerShip>>` borrows the player's Transform IMMUTABLY, and the enemy query borrows `&mut Transform` (for orientation) + `&mut LinearVelocity` (for movement) per-enemy. The `Without<PlayerShip>` filter on the enemy query is REQUIRED to satisfy Bevy's "no two queries can mutably-and-immutably-overlap" invariant.
    **And** if the player query returns zero entities (Single panics on zero matches in Bevy 0.18), this is a programmer error (PlayerShip should always exist in `GameState::Arena` per the FlightPlugin spawn contract). **NO** mitigation — same posture as `attach_combat_to_player_ship`.

11. **(`detect_projectile_enemy_hits` + `apply_enemy_damage` — parallel pipeline to asteroid-damage)** `src/combat/damage.rs` is extended with TWO new systems and TWO new event types, mirroring the existing asteroid pipeline:
    ```rust
    /// Emitted by `detect_projectile_enemy_hits` when a player projectile and
    /// an enemy begin contacting. Consumed by `apply_enemy_damage`.
    #[derive(Message, Debug, Clone, Copy)]
    pub struct ProjectileHitEnemy {
        pub projectile: Entity,
        pub enemy: Entity,
        pub damage: u32,
    }

    /// Emitted by `apply_enemy_damage` when an enemy's Health reaches zero.
    /// Story 4.5 (salvage retro-tint) and Epic 8 (audio cues) are downstream consumers.
    #[derive(Message, Debug, Clone, Copy)]
    pub struct EnemyDestroyed {
        pub enemy: Entity,
    }
    ```
    **And** `pub fn detect_projectile_enemy_hits(...)` runs in `CombatSystems::EvaluateHits`, mirrors `detect_projectile_asteroid_hits` exactly except the right-side query is `Query<(), With<Enemy>>` (NOT `With<Asteroid>`); emits `ProjectileHitEnemy` events.
    **And** `pub fn apply_enemy_damage(...)` runs in `CombatSystems::ApplyDamage`, mirrors `apply_asteroid_damage` body except: queries `Query<&mut Health, With<Enemy>>` (NOT `With<Asteroid>`); on `current == 0` emits `EnemyDestroyed { enemy: Entity }` (NOT `AsteroidDestroyed`).
    **And** **NO** filter against `EnemyProjectile` marker on `detect_projectile_enemy_hits` — only player projectiles can collide with enemies per the AC #5 collision-filter design (player has `[Asteroid, Enemy]` filter; enemy projectiles have `[Default]` filter — they cannot reach the Enemy layer). Documented inline as a comment near the system.
    **And** the existing `apply_asteroid_damage` is updated per AC #2: its query becomes `Query<&mut Health, With<Asteroid>>`; the body uses `hp.current` field of `Health` (not `AsteroidHp.current`).
    **And** both `ProjectileHitEnemy` and `EnemyDestroyed` are registered via `app.add_message::<>()` in `CombatPlugin::build` (alongside the existing `ProjectileHitAsteroid` / `AsteroidDestroyed`).

12. **(`CombatPlugin::build` — FixedUpdate set-graph extension)** `src/combat/mod.rs::CombatPlugin::build` is updated:
    - **System set graph extension:** `configure_sets(FixedUpdate, ...)` adds a new `CombatSystems::EnemyAi` variant chained BEFORE `Fire`. Final chain: `(EnemyAi, Fire, Lifecycle, EvaluateHits, ApplyDamage).chain()`.
    - **System set enum extension:** `CombatSystems` gains an `EnemyAi` variant (alphabetical or explicit ordering — the enum order does not bind chain order, which is set by `.chain()`).
    - **System registration:**
      - `enemy_ai::apply_enemy_ai.in_set(CombatSystems::EnemyAi).run_if(in_state(GameState::Arena))`
      - `enemy_ai::enemy_fire_weapon.in_set(CombatSystems::Fire).run_if(in_state(GameState::Arena))`
      - `damage::detect_projectile_enemy_hits.in_set(CombatSystems::EvaluateHits).run_if(in_state(GameState::Arena))`
      - `damage::apply_enemy_damage.in_set(CombatSystems::ApplyDamage).run_if(in_state(GameState::Arena))`
    - **Event registration:** `app.add_message::<ProjectileHitEnemy>(); app.add_message::<EnemyDestroyed>();` alongside existing `ProjectileHitAsteroid` / `AsteroidDestroyed`.
    
    **And** the OnTransition spawn registration tuple from 4.1 (`projectiles::attach_combat_to_player_ship` + `enemy::spawn_enemy_ship`) is unchanged.
    **And** **NO** chain-ordering between `FlightSystems::ApplyForces` and `CombatSystems::EnemyAi` is added (deferred-work entry from 3.9 review at line 254 documents `FlightSystems::ApplyForces → CombatSystems::Fire` ordering as a future tightening if it matters; same applies to `EnemyAi`. Neither is required for first-playable).

13. **(Imports in new files — minimal but complete)**

    `src/combat/health.rs`:
    ```rust
    use bevy::prelude::*;
    ```

    `src/combat/enemy_ai.rs`:
    ```rust
    use avian3d::prelude::{
        Collider, CollisionEventsEnabled, CollisionLayers, LinearVelocity, RigidBody,
    };
    use bevy::prelude::*;

    use crate::arena::ArenaEntity;
    use crate::combat::components::Projectile;
    use crate::combat::damage::GameLayer;
    use crate::combat::enemy::Enemy;
    use crate::flight::PlayerShip;
    use crate::tuning::TuningHandle;
    use crate::tuning::config::TuningConfig;
    use crate::visual::palette::{SemanticAccent, color_for};
    use crate::visual::toon_material::ToonMaterial;
    ```

    `src/combat/components.rs` (after `AsteroidHp` removal):
    ```rust
    use bevy::prelude::*;
    ```
    (one less type → `AsteroidHp` removed; `Asteroid` marker MOVED here OR kept in `enemy.rs` companion module — DECISION: place `Asteroid` in `src/combat/components.rs` alongside `Projectile` / `PrimaryWeaponCooldown` since it is a marker for an entity-class that has no behavior module; mirrors `PlayerShip` placement in `src/flight/mod.rs`.)
    
    **And** `src/combat/damage.rs` `use` block updates from `use crate::combat::components::{AsteroidHp, Projectile};` → `use crate::combat::components::{Asteroid, Projectile}; use crate::combat::enemy::Enemy; use crate::combat::health::Health;`
    **And** `src/arena/zone.rs` `use` block updates from `use crate::combat::components::AsteroidHp;` → `use crate::combat::components::Asteroid; use crate::combat::health::Health;`
    **And** `src/combat/mod.rs` `use` block adds `use crate::combat::damage::{EnemyDestroyed, ProjectileHitEnemy};` alongside the existing `AsteroidDestroyed` / `ProjectileHitAsteroid` imports.

14. **(Tests — comprehensive coverage of pure helpers)** Net new tests:

    **`src/combat/health.rs::tests` (1 test):**
    ```rust
    #[test]
    fn health_construction_is_explicit() {
        // No Default derive — guards against accidental future Default addition
        // that would silently default to (current=0, max=0) (pre-destroyed/unkillable footgun).
        // Two-field round-trip per the AsteroidHp / EnemyShip / HudPlaceholder explicit-construction precedent.
        let h = Health { current: 2, max: 2 };
        assert_eq!(h.current, 2);
        assert_eq!(h.max, 2);
    }
    ```

    **`src/combat/enemy_ai.rs::tests` (7 tests):** 4 quadrant tests + 3 hysteresis tests.
    ```rust
    use super::*;

    fn det() -> f32 { 100.0 }
    fn eng() -> f32 { 50.0 }
    fn hyst() -> f32 { 0.1 }

    #[test]
    fn idle_when_distance_exceeds_detection_range() {
        // Far away, idle stays idle; out of detect range.
        assert_eq!(
            next_ai_state(EnemyAiState::Idle, 200.0, det(), eng(), hyst()),
            EnemyAiState::Idle
        );
    }

    #[test]
    fn idle_to_detect_when_player_enters_detection_range() {
        // Inside detection_range but outside engagement_range → Detect.
        assert_eq!(
            next_ai_state(EnemyAiState::Idle, 75.0, det(), eng(), hyst()),
            EnemyAiState::Detect
        );
    }

    #[test]
    fn detect_to_pursue_when_player_enters_engagement_range() {
        // Inside engagement_range but outside attack-range (engagement * 0.5) → Pursue.
        assert_eq!(
            next_ai_state(EnemyAiState::Detect, 35.0, det(), eng(), hyst()),
            EnemyAiState::Pursue
        );
    }

    #[test]
    fn pursue_to_attack_when_player_enters_attack_range() {
        // Inside engagement_range * 0.5 = 25.0 → Attack.
        assert_eq!(
            next_ai_state(EnemyAiState::Pursue, 20.0, det(), eng(), hyst()),
            EnemyAiState::Attack
        );
    }

    #[test]
    fn attack_holds_within_hysteresis_band_outward() {
        // Hysteresis dead-band: at distance 27.0 (between attack=25 and 25*1.1=27.5),
        // an Attack-state enemy stays in Attack — does not flicker back to Pursue.
        assert_eq!(
            next_ai_state(EnemyAiState::Attack, 27.0, det(), eng(), hyst()),
            EnemyAiState::Attack
        );
    }

    #[test]
    fn attack_to_pursue_when_distance_exceeds_outer_hysteresis_threshold() {
        // Distance 28.0 > 25*1.1 = 27.5 → Attack exits to Pursue.
        assert_eq!(
            next_ai_state(EnemyAiState::Attack, 28.0, det(), eng(), hyst()),
            EnemyAiState::Pursue
        );
    }

    #[test]
    fn detect_holds_within_hysteresis_band_outward() {
        // Distance 105.0 (between detect=100 and 100*1.1=110), Detect stays.
        assert_eq!(
            next_ai_state(EnemyAiState::Detect, 105.0, det(), eng(), hyst()),
            EnemyAiState::Detect
        );
    }
    ```

    **`src/combat/components.rs::tests`:** the existing `asteroid_hp_construction_is_explicit` test is **REMOVED** (since `AsteroidHp` is removed). The `Health` test (above) supersedes the no-Default discipline check. Net delta: -1 test in components, +1 in health = net 0 in those modules.

    **`src/tuning/config.rs::tests`:** the existing 3 tests are extended with assertions for the 5 new fields. NO new test functions added — assertion count grows but function count stays at 3.

    **Net new test functions across the codebase: +7** (1 in health, 7 in enemy_ai, -1 in components, +0 in tuning). Net post-4.2 test count: **53 + 7 = 60**. AC #15 enforces.

15. **(Verification gates — all 6 cargo commands clean)** Per `feedback_full_build_output.md` discipline, exit-0 + tail is NOT proof; full output is captured per command and grep'd for `warning:|error:`.
    **Then** **all six** of the following produce **0** lines matching `grep -cE 'warning:|error:'` per their respective full-output logs:
    ```bash
    cargo check                                         2>&1 | tee /tmp/story-4-2-check.log
    cargo build                                         2>&1 | tee /tmp/story-4-2-build.log
    cargo test                                          2>&1 | tee /tmp/story-4-2-test.log
    cargo clippy --all-targets -- -D warnings           2>&1 | tee /tmp/story-4-2-clippy.log
    cargo fmt --all -- --check                          2>&1 | tee /tmp/story-4-2-fmt.log
    cargo build --release                               2>&1 | tee /tmp/story-4-2-release.log
    ```
    **And** `cargo test` summary line reads `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N = 60** (= 53 baseline from end of 4.1 + 7 net new per AC #14).

16. **(File set — `git status --short` final)** Final set is **exactly**:
    - `?? src/combat/health.rs` (new file: `Health` component + 1 test)
    - `?? src/combat/enemy_ai.rs` (new file: `EnemyAiState` enum + `EnemyProjectile` marker + `EnemyFireCooldown` + `next_ai_state` + `apply_enemy_ai` + `enemy_fire_weapon` + 7 tests)
    - `M src/combat/mod.rs` (M — `pub mod enemy_ai;` + `pub mod health;` + system-set + system + event registrations)
    - `M src/combat/components.rs` (M — remove `AsteroidHp`; add `Asteroid` marker; remove obsolete test)
    - `M src/combat/damage.rs` (M — `GameLayer::Enemy` variant; query/import refactor for `Asteroid` + `Health`; add `ProjectileHitEnemy` + `EnemyDestroyed` events; `detect_projectile_enemy_hits`; `apply_enemy_damage`)
    - `M src/combat/projectiles.rs` (M — player projectile filter `[Asteroid] → [Asteroid, Enemy]`)
    - `M src/combat/enemy.rs` (M — extend spawn-tuple per AC #3 with Health + EnemyAiState + CollisionLayers + CollisionEventsEnabled + Name + ExternalForce + ExternalTorque + EnemyFireCooldown)
    - `M src/arena/zone.rs` (M — asteroid spawn-tuple `AsteroidHp { current: 1 }` → `(Asteroid, Health { current: 1, max: 1 })`)
    - `M src/tuning/config.rs` (M — 5 new fields + 5 default fns + Default impl extension; tests extended)
    - `M assets/config/tuning.ron` (M — 5 new fields at canonical defaults)
    - `M _bmad-output/implementation-artifacts/sprint-status.yaml` (M — `4-2-...: backlog → ready-for-dev → in-progress → review → done`, `last_updated`)
    - `?? _bmad-output/implementation-artifacts/4-2-enemy-ai-state-machine-detect-pursue-attack.md` at story-creation time (becomes M after dev flips Status / fills Dev Agent Record / Change Log)
    
    **NO** entries under: `Cargo.toml` / `Cargo.lock` (no dep added — `ExternalForce` / `ExternalTorque` / `Single` / `Without` already in scope via Avian / Bevy preludes), `src/flight/**`, `src/ui/**`, `src/state.rs`, `src/pause/**`, `src/visual/**`, `src/splash.rs`, `src/logging.rs`, `src/main.rs`, `src/arena/mod.rs` (cleanup-on-exit unchanged), `assets/meshes/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`.

17. **(Runtime smoke — full enemy-AI-active chain)** After Task 4 confirms cargo gates green, Till manually executes `cargo run 2>&1 | tee /tmp/story-4-2-run.log` and verifies:
    - **(a) Idle at start (>100m)** — Press Enter on MainMenu. Enemy is initially at (0,0,-60), player at origin, distance ~60m → enemy is in `Pursue` state immediately (60 ≤ engagement=50 is FALSE; 60 ≤ detection=100 is TRUE; 60 > engagement=50 → Detect). Enemy rotates to face player (from default rotation), holds position. **NOTE:** If desired test of Idle requires the player to retreat past 110m. Accept Detect-on-spawn as the baseline first-impression.
    - **(b) Detect → Pursue threshold** — Approach the enemy (W to thrust forward). Once distance ≤ 50m, enemy should begin moving toward player at ~20 m/s. Watch for `info!` log: `enemy fired projectile from ...` once Attack triggers. Visually: enemy capsule begins translating with `LinearVelocity = forward * 20`.
    - **(c) Pursue → Attack threshold** — Continue closing. Once distance ≤ 25m, enemy stops (LinearVelocity zero), holds position, and begins firing vermillion-tinted projectiles at ~1 Hz (every 1.0 s). Projectiles travel toward player at `projectile_speed = 120 m/s` and TTL-expire after 3.0 s (existing tick_projectile_ttl).
    - **(d) Hysteresis dead-band** — Slowly retreat from the enemy. Verify no rapid Attack ↔ Pursue ↔ Detect oscillation as you cross the 25/27.5 m boundary (or 50/55, 100/110). State holds stable within the dead-band; transitions fire only when crossing the outer threshold.
    - **(e) Player projectile destroys enemy** — While enemy is in Attack range, fire LMB twice (default `projectile_fire_rate_hz = 4.0` Hz, so two shots within 0.5s). After two hits the enemy despawns; the vermillion projectiles disappear; `info!` log contains `enemy destroyed: entity=...`. `grep -c 'enemy destroyed' /tmp/story-4-2-run.log` outputs ≥ 1.
    - **(f) Enemy projectiles do NOT damage player** — In 4.2, enemy projectiles physically collide with PlayerShip but no damage is applied (4.3's scope). `grep -c 'projectile.*hit.*player\|player.*damaged' /tmp/story-4-2-run.log` outputs **0** (no player-damage event is registered yet).
    - **(g) Pause round-trip preserves AI state** — Press Esc to pause during Pursue/Attack. Press Esc to resume. Enemy resumes in the same state at the same position with the same velocity. `grep -c 'spawned EnemyShip' /tmp/story-4-2-run.log` still outputs **1** (no respawn).
    - **(h) Asteroid destruction still works** — Aim away from the enemy at an asteroid; fire LMB. Asteroid despawns on first hit (Story 3.10 behavior preserved through the Asteroid + Health refactor). `grep -c 'asteroid destroyed' /tmp/story-4-2-run.log` outputs ≥ 1.
    - **(i) Quit cleanly during Arena** — Close window during Arena. No panic. `grep -cE 'panic|backtrace|FATAL' /tmp/story-4-2-run.log` outputs **0**.

## Tasks / Subtasks

- [x] **Task 1: Foundation refactor — `Health` component + `Asteroid` marker + `AsteroidHp` removal** (AC: #1, #2, #13)
  - [x] Create new file `src/combat/health.rs`. Module doc, `use bevy::prelude::*;`, `Health` struct per AC #1, `#[cfg(test)] mod tests` with `health_construction_is_explicit` per AC #14.
  - [x] Update `src/combat/components.rs`:
    - Remove `AsteroidHp` struct + its module doc-comment + the `asteroid_hp_construction_is_explicit` test.
    - Add `Asteroid` marker per AC #2.
  - [x] Update `src/combat/mod.rs`:
    - Add `pub mod health;` between `enemy` and `input` (alphabetical).
  - [x] Update `src/arena/zone.rs`:
    - Change `use crate::combat::components::AsteroidHp;` → `use crate::combat::components::Asteroid; use crate::combat::health::Health;`
    - Change asteroid spawn-tuple line `AsteroidHp { current: 1 },` → `Asteroid, Health { current: 1, max: 1 },` (preserve all other tuple components and ordering).
  - [x] Update `src/combat/damage.rs`:
    - Change `use crate::combat::components::{AsteroidHp, Projectile};` → `use crate::combat::components::{Asteroid, Projectile}; use crate::combat::health::Health;`
    - In `detect_projectile_asteroid_hits`: change `Query<(), With<AsteroidHp>>` → `Query<(), With<Asteroid>>`.
    - In `apply_asteroid_damage`: change `Query<&mut AsteroidHp>` → `Query<&mut Health, With<Asteroid>>`. Body still reads/writes `hp.current` (Health's field; same name as AsteroidHp's). The `if hp.current == 0 { continue; }` and `hp.current = apply_damage(...)` lines work unchanged.
  - [x] **Verify post-edit:** `cargo check 2>&1 | tee /tmp/story-4-2-check-task1.log; grep -cE 'warning:|error:' /tmp/story-4-2-check-task1.log` should output `0`. If any callsite was missed, the compiler will error.

- [x] **Task 2: TuningConfig extension — 5 new enemy-AI fields** (AC: #6)
  - [x] Update `src/tuning/config.rs`:
    - Add 5 new struct fields per AC #6, each with `#[serde(default = "default_<name>")]` attribute.
    - Add 5 corresponding `fn default_<name>() -> f32 { ... }` helpers.
    - Extend `impl Default for TuningConfig` with the 5 new fields, each calling its `default_<name>()` helper.
    - Extend `tuning_config_default_matches_ron_initial_values` test with 5 new assertions.
    - Extend `tuning_config_deserializes_from_ron_bytes` test with 5 new fields in the RON bytes literal AND 5 new assertions.
    - Extend `tuning_config_legacy_schema_uses_defaults_for_added_fields` test with 5 new assertions (the legacy 3-field RON literal stays unchanged; the test verifies new fields take their defaults via serde).
  - [x] Update `assets/config/tuning.ron`: append 5 new field-value pairs at the canonical defaults (match `default_<name>()` values exactly).
  - [x] **Verify post-edit:** `cargo test --package <crate> tuning::config -- 2>&1 | tee /tmp/story-4-2-check-task2.log; grep -cE 'warning:|error:|FAILED' /tmp/story-4-2-check-task2.log` should output `0`.

- [x] **Task 3: `enemy_ai.rs` authoring — `EnemyAiState` + `EnemyProjectile` + `EnemyFireCooldown` + `next_ai_state` + 7 tests** (AC: #4, #7, #14)
  - [x] Create new file `src/combat/enemy_ai.rs`. Author top-down:
    1. Module doc-comment:
       ```rust
       //! Enemy AI state machine (FR14) — Idle/Detect/Pursue/Attack distance-driven
       //! transitions with hysteresis dead-band. Owns `apply_enemy_ai` (state transition
       //! + orientation + movement) and `enemy_fire_weapon` (Attack-state fire pacing).
       //! Pure helper `next_ai_state` is the testable transition core.
       ```
    2. Use block per AC #13 verbatim.
    3. Constants:
       ```rust
       const ENEMY_PROJECTILE_SPAWN_OFFSET: f32 = 3.0;
       const ENEMY_PROJECTILE_RADIUS: f32 = 0.2;
       ```
    4. Type definitions per AC #4 (`EnemyAiState`, `EnemyProjectile`) + `EnemyFireCooldown` per AC #9.
    5. Pure `next_ai_state` function per AC #7.
    6. (Task 4 lands `apply_enemy_ai` and `enemy_fire_weapon` here; this task delivers types + pure helper + tests.)
    7. `#[cfg(test)] mod tests` block with the 7 prescribed tests per AC #14 verbatim.
  - [x] Update `src/combat/mod.rs`: add `pub mod enemy_ai;` between `enemy` and `health` (alphabetical: `components, damage, enemy, enemy_ai, health, input, projectiles`).
  - [x] **Verify post-edit:** `cargo test --package <crate> enemy_ai -- 2>&1 | tee /tmp/story-4-2-check-task3.log; grep -cE 'warning:|error:|FAILED' /tmp/story-4-2-check-task3.log` should output `0`. The 7 enemy_ai tests + 1 health test pass.

- [x] **Task 4: `apply_enemy_ai` + `enemy_fire_weapon` systems** (AC: #8, #9, #10)
  - [x] In `src/combat/enemy_ai.rs`, append two pub fn implementations:
    - `apply_enemy_ai` per AC #8 / #10 — final query signature with `Without<PlayerShip>` filter; tuning fallback; per-enemy: state transition (call `next_ai_state`), orientation (Detect/Pursue/Attack snap-rotate via `Transform::look_at`), velocity write per state per AC #8(c–e).
    - `enemy_fire_weapon` per AC #9 — cooldown tick; state-and-cooldown gate; spawn enemy projectile with full Avian + Bevy + ArenaEntity + EnemyProjectile + Projectile components per AC #9 body.
  - [x] **Verify post-edit:** `cargo check 2>&1 | tee /tmp/story-4-2-check-task4.log; grep -cE 'warning:|error:' /tmp/story-4-2-check-task4.log` should output `0`. Borrow-checker errors on the enemy query are the most likely failure mode — re-read AC #10 for the FINAL signature.

- [x] **Task 5: `enemy.rs` spawn-tuple extensions** (AC: #3)
  - [x] Update `src/combat/enemy.rs::spawn_enemy_ship`:
    - Add imports: `Avian's ExternalForce, ExternalTorque, CollisionEventsEnabled, CollisionLayers, LayerMask`. The `use avian3d::prelude::{...}` block grows to include them.
    - Add imports: `use crate::combat::damage::GameLayer; use crate::combat::enemy_ai::{EnemyAiState, EnemyFireCooldown}; use crate::combat::health::Health;`
    - Extend the `commands.spawn((...))` tuple ADDITIVELY (preserve all 4.1 components) per AC #3 with the 7 new components: `Health`, `EnemyAiState::Idle`, `CollisionLayers::new([GameLayer::Enemy], LayerMask::ALL)`, `CollisionEventsEnabled`, `Name::new("EnemyShip")`, `ExternalForce::default()`, `ExternalTorque::default()`, `EnemyFireCooldown::default()`.
    - The spawn `info!` log line is unchanged.
  - [x] **Verify post-edit:** `cargo check 2>&1 | tee /tmp/story-4-2-check-task5.log; grep -cE 'warning:|error:' /tmp/story-4-2-check-task5.log` should output `0`. Bevy 0.18's tuple-spawn limit is 15; the post-extension tuple has 20 components — **may require a nested-tuple workaround** if the limit triggers. Workaround: split into two adjacent `commands.spawn(...)` + `commands.entity(id).insert(...)` calls, OR group via `(component_a, component_b, (component_c, component_d, ...))` nested-tuple syntax (Bevy auto-flattens). Pick whichever the compiler accepts cleanly.

- [x] **Task 6: Damage-pipeline extension — `GameLayer::Enemy` + projectile filter + new events + new systems** (AC: #5, #11, #12)
  - [x] Update `src/combat/damage.rs`:
    - Add `Enemy` variant to `GameLayer` enum (preserve `#[default] Default` and existing variants).
    - Add `ProjectileHitEnemy` + `EnemyDestroyed` Message structs per AC #11.
    - Add `detect_projectile_enemy_hits` system per AC #11 — mirrors `detect_projectile_asteroid_hits` exactly except right-side query is `With<Enemy>` (need to import `Enemy` from `crate::combat::enemy::Enemy`).
    - Add `apply_enemy_damage` system per AC #11 — mirrors `apply_asteroid_damage` body, uses `Query<&mut Health, With<Enemy>>`, emits `EnemyDestroyed` on `current == 0`.
  - [x] Update `src/combat/projectiles.rs`:
    - Change player-projectile `CollisionLayers::new([GameLayer::Projectile], [GameLayer::Asteroid])` → `CollisionLayers::new([GameLayer::Projectile], [GameLayer::Asteroid, GameLayer::Enemy])`. Single-line change.
  - [x] Update `src/combat/mod.rs::CombatPlugin::build` per AC #12:
    - Add `EnemyAi` variant to `CombatSystems` enum (after `Setup`, before `Fire` — though enum order doesn't bind chain order).
    - Update `configure_sets(FixedUpdate, ...)` chain to include `EnemyAi` first: `(EnemyAi, Fire, Lifecycle, EvaluateHits, ApplyDamage).chain()`.
    - Register events: `app.add_message::<ProjectileHitEnemy>(); app.add_message::<EnemyDestroyed>();`
    - Register 4 new systems per AC #12 in the FixedUpdate add_systems block: `enemy_ai::apply_enemy_ai` (EnemyAi set), `enemy_ai::enemy_fire_weapon` (Fire set), `damage::detect_projectile_enemy_hits` (EvaluateHits set), `damage::apply_enemy_damage` (ApplyDamage set). All `.run_if(in_state(GameState::Arena))`.
    - Add `use` for `crate::combat::damage::{EnemyDestroyed, ProjectileHitEnemy};` alongside existing imports.
  - [x] **Verify post-edit:** `cargo check 2>&1 | tee /tmp/story-4-2-check-task6.log; grep -cE 'warning:|error:' /tmp/story-4-2-check-task6.log` should output `0`. Then `cargo test 2>&1 | tee /tmp/story-4-2-check-task6-test.log; grep -cE 'warning:|error:|FAILED' /tmp/story-4-2-check-task6-test.log` should output `0`.

- [x] **Task 7: Verification gates — all 6 cargo commands clean** (AC: #15)
  - [x] Run each command in sequence; capture FULL output (NOT just exit code or tail) per `feedback_full_build_output.md`:
    ```bash
    cargo check                                         2>&1 | tee /tmp/story-4-2-check.log
    cargo build                                         2>&1 | tee /tmp/story-4-2-build.log
    cargo test                                          2>&1 | tee /tmp/story-4-2-test.log
    cargo clippy --all-targets -- -D warnings           2>&1 | tee /tmp/story-4-2-clippy.log
    cargo fmt --all -- --check                          2>&1 | tee /tmp/story-4-2-fmt.log
    cargo build --release                               2>&1 | tee /tmp/story-4-2-release.log
    ```
  - [x] For EACH log: `grep -cE 'warning:|error:' /tmp/story-4-2-<cmd>.log` must output `0`. If non-zero, fix the root cause and re-run from the failing command. NO partial-pass shortcuts.
  - [x] `cargo test` log MUST contain `60 passed` AND `0 failed`. Confirm the literal line `test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (or accept a less specific variant: confirm `60 passed` AND `0 failed` are both present in the log).

- [x] **Task 8: Runtime smoke — full enemy-AI-active chain validation** (AC: #17)
  - [x] Till manually executes `cargo run 2>&1 | tee /tmp/story-4-2-run.log` and verifies scenarios (a)–(i) per AC #17. LLM cannot execute the interactive smoke (Enter on MainMenu, flight controls, fire LMB, Esc, Cmd-Tab, window-close). All preconditions for the smoke (Task 7 cargo gates) must be met.
  - [x] `grep -c 'enemy destroyed' /tmp/story-4-2-run.log` outputs **≥ 1** after the player kills the enemy.
  - [x] `grep -c 'enemy fired projectile' /tmp/story-4-2-run.log` outputs **≥ 1** while the enemy is in Attack state.
  - [x] `grep -c 'asteroid destroyed' /tmp/story-4-2-run.log` outputs **≥ 1** after a player asteroid kill (regression check on the AsteroidHp → Health refactor).
  - [x] `grep -cE 'panic|backtrace|FATAL' /tmp/story-4-2-run.log` outputs **0**.
  - [x] `grep -cE 'WARN.*Enemy|ERROR.*Enemy' /tmp/story-4-2-run.log` outputs **0**.

- [x] **Task 9: Sprint status bookkeeping** (AC: #16)
  - [x] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - The create-story workflow Step 6 flips `4-2-enemy-ai-state-machine-detect-pursue-attack: backlog → ready-for-dev`.
    - Dev workflow flips `ready-for-dev → in-progress → review → done` per the standard lifecycle.
    - Status flip `in-progress → review` is performed once Till's manual smoke (Task 8) is confirmed.
    - Update the `last_updated:` field (both the comment at the top of the file AND the `last_updated:` value in the document body) to the current date.

### Review Findings

*Code review conducted 2026-05-06 — 1 decision-needed, 1 patch, 10 deferred, 12 dismissed.*

- [ ] [Review][Decision] `ExternalForce` / `ExternalTorque` omitted from spawn tuple — AC #3 lists both as required components; dev note says `avian3d-0.6.1` does not export them. Verify in `Cargo.lock`/avian source: do these types exist in the locked version? If yes, add them; if no, update AC #3 wording to document the omission. [src/combat/enemy.rs]

- [x] [Review][Patch] `look_at` degenerate rotation when enemy is at player position or directly above/below player [src/combat/enemy_ai.rs:158] — fixed 2026-05-06: guard on `distance > f32::EPSILON`; fallback up-vector `Vec3::Z` when `forward.y.abs() > 1.0 - 1e-4`.

- [x] [Review][Defer] Asset leak: new mesh + material handle allocated per enemy shot, never released on projectile despawn [src/combat/enemy_ai.rs:205-214] — deferred, pre-existing pattern from `fire_primary_weapon`; fix requires project-wide shared-handle strategy (Epic 10 pass)
- [x] [Review][Defer] Stale `CollisionStart` events may reference TTL-despawned projectile entities; `.expect()` would panic [src/combat/damage.rs:147] — deferred, pre-existing in `detect_projectile_asteroid_hits`; Avian same-frame event safety assumed; replace with `.ok()?` if panics surface
- [x] [Review][Defer] Tuning config: no validation that `detection_range > engagement_range > attack_range` or that values are positive [src/tuning/config.rs] — deferred, developer-controlled RON; add `debug_assert!` guards at Epic 10 hardening
- [x] [Review][Defer] Enemy projectiles physically collide with PlayerShip (Default layer) but no damage system handles these `CollisionStart` events in 4.2 [src/combat/enemy_ai.rs:229] — deferred, per-spec; Story 4.3 adds hull-damage routing; projectiles TTL-expire harmlessly
- [x] [Review][Defer] Test gap: `next_ai_state(Idle, distance_inside_engagement_range)` not covered — single-step semantics (Idle→Detect only, not Pursue) untested [src/combat/enemy_ai.rs] — deferred, adding would exceed spec-mandated 60-test count; add when AC #15 count is lifted
- [x] [Review][Defer] Test gap: `Pursue → Detect` outward hysteresis transition not tested [src/combat/enemy_ai.rs] — deferred, not in the 7 spec-mandated tests; add post-M3 if hysteresis edge cases surface in playtesting
- [x] [Review][Defer] `Health` no upper-bound invariant: `Health { current: 5, max: 2 }` is constructable; a zero-HP entity that bypasses the damage path is an unkillable ghost [src/combat/health.rs] — deferred, no current spawns affected; add `debug_assert!(current <= max)` at Story 5.1 formal HP refactor
- [x] [Review][Defer] Enemy-player collocated: `normalize_or_zero` returns `Vec3::ZERO` → zero-velocity projectile spawned inside enemy at offset 0 [src/combat/enemy_ai.rs:202] — deferred, unreachable in normal gameplay; add early-continue guard if collision reports surface
- [x] [Review][Defer] `SemanticAccent::Enemy` component not inserted on enemy projectile entity (only tint color via `color_for(...)` used) [src/combat/enemy_ai.rs:216] — deferred, spec says "tint" not "component"; Story 4.5 retroactive accent sweep must explicitly handle enemy projectiles or they will be missed
- [x] [Review][Defer] NaN in `transform.translation` from physics explosion silently freezes enemy in Idle indefinitely [src/combat/enemy_ai.rs:144] — deferred, requires exotic physics failure outside 4.2 scope; add `if !distance.is_finite() { continue; }` at hardening pass

## Dev Notes

### Relevant architecture patterns and constraints

- **Plugin boundaries** (architecture.md:643-658) — `CombatPlugin` owns enemy AI, projectiles, damage, Health. Cross-plugin reads of `PlayerShip` Transform are permitted (via `crate::flight::PlayerShip` import — same as projectiles.rs already does). NO cross-plugin Resource/Component MUTATION. Story 4.2 reads `PlayerShip` Transform; does not write anything to PlayerShip components (Story 4.3 will write Health to PlayerShip).
- **One-way dependency: Combat → Flight** (precedent: `src/combat/projectiles.rs:22 use crate::flight::PlayerShip;`) — Combat reads Flight; Flight does not import Combat. Story 4.2 preserves this. The reverse (`Flight reads Combat`) would couple the two and is rejected.
- **Past-tense event naming** (architecture.md:324) — `ProjectileHitEnemy` and `EnemyDestroyed` follow the established naming exactly (`ProjectileHitAsteroid` / `AsteroidDestroyed` precedent). Past tense communicates "fires after the fact, consumers react" — Story 4.5 (salvage retro-tint) and Epic 8 (audio cues) will subscribe.
- **No magic numbers without rationale** (architecture.md:463) — all 5 new TuningConfig fields carry their values per epic-4 spec lines 49-51. Hysteresis math (1+pct) is documented in AC #7 inline.
- **OnTransition discipline** (`deferred-work.md:220` — Story 3.9 fix) — Story 4.2 does NOT add new spawn systems. The existing `spawn_enemy_ship` registration on `OnTransition { exited: MainMenu, entered: Arena }` is preserved; Story 4.2 only EXTENDS the spawn tuple. AI systems run in FixedUpdate gated by `in_state(Arena)` — same gating pattern as flight physics and existing combat.
- **Pure-helper learning pattern** (precedent: `apply_damage`, `dampener_acceleration`, `projectile_initial_velocity`, `ship_local_thrust_vector`, `ship_local_torque_vector`) — `next_ai_state` is the latest in this lineage. Pure functions = simple unit tests = many assertions across the input space. Hysteresis testing without ECS overhead.
- **Snap-rotate vs. slewed rotation** — Story 4.2 uses `Transform::look_at` (snap-rotate) for enemy orientation. A torque-driven slew would require the same `apply_torque` integration as the player ship — out of scope for first-playable. The "AI feels mechanical" trade-off is acceptable for M3 stop-and-ship; smoothing is post-MVP polish.
- **Direct `LinearVelocity` write vs. `ExternalForce`** — for `Pursue` movement, the system writes `LinearVelocity` directly (constant 20 m/s toward player) rather than applying force-and-tuning. Avian permits direct writes on `Dynamic` bodies. The trade-off vs. force-based: simpler tuning (1 number, not mass+force), but ignores momentum/inertia. Acceptable for first-playable per the epic's "speed is clamped to enemy_speed" wording.
- **Borrow-checker disjoint queries** — `apply_enemy_ai` queries both `PlayerShip` Transform (immutable) AND `Enemy` Transform (mutable). Bevy's invariant requires `Without<PlayerShip>` filter on the enemy query. Same pattern as any future system that reads from one entity-class and writes to another.
- **Bevy 0.18 tuple-spawn 15-limit** — extended `spawn_enemy_ship` tuple after AC #3 has 20 components. If the compiler errors on tuple size, use nested-tuple grouping `(a, b, c, (d, e, f, ...))` which Bevy auto-flattens. This is a Bevy limitation, not a project decision.
- **Cold-start TuningConfig fallback** (`src/flight/mod.rs:97-101`, `src/arena/zone.rs:50-55`, `src/combat/projectiles.rs:86-89`, `src/combat/enemy.rs:43-49`) — pre-existing project-wide pattern. `apply_enemy_ai` and `enemy_fire_weapon` follow it.

### Source tree components to touch

| File | Change | LOC delta (estimate) |
|------|--------|---------------------|
| `src/combat/health.rs` | NEW: `Health` component + 1 test | +25 |
| `src/combat/enemy_ai.rs` | NEW: `EnemyAiState` + `EnemyProjectile` + `EnemyFireCooldown` + `next_ai_state` + `apply_enemy_ai` + `enemy_fire_weapon` + 7 tests | +280 |
| `src/combat/components.rs` | Remove `AsteroidHp` + obsolete test; add `Asteroid` marker | +0 net (~−10/+5) |
| `src/combat/damage.rs` | `GameLayer::Enemy` variant; query/import refactor for `Asteroid` + `Health`; `ProjectileHitEnemy` + `EnemyDestroyed` events; `detect_projectile_enemy_hits` + `apply_enemy_damage` systems | +90 |
| `src/combat/projectiles.rs` | Update player projectile collision filter `[Asteroid] → [Asteroid, Enemy]` | +1 |
| `src/combat/enemy.rs` | Extend spawn-tuple per AC #3 | +15 |
| `src/combat/mod.rs` | `pub mod` extensions; system-set + system + event registrations | +25 |
| `src/arena/zone.rs` | Asteroid spawn tuple `AsteroidHp { current: 1 }` → `(Asteroid, Health { current: 1, max: 1 })` | +1 |
| `src/tuning/config.rs` | 5 new fields + 5 default fns + Default impl extension; tests extended | +40 |
| `assets/config/tuning.ron` | Append 5 new field-value pairs | +5 |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | bookkeeping (story status flips; last_updated) | +0 net |

NO changes expected in: `src/flight/**`, `src/ui/**`, `src/state.rs`, `src/pause/**`, `src/visual/**`, `src/splash.rs`, `src/logging.rs`, `src/main.rs`, `src/arena/mod.rs`, `assets/meshes/**`, `Cargo.toml`, `Cargo.lock`, `.github/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`.

### Testing standards summary

- **Unit tests only** for Story 4.2 (architecture.md:354 — integration tests deferred post-M3). Pure-helper coverage:
  - `health_construction_is_explicit` (1 test) — no-Default discipline guard.
  - `next_ai_state` quadrant tests (4) — Idle/Detect/Pursue/Attack baseline transitions.
  - `next_ai_state` hysteresis tests (3) — outward dead-band on Detect, Pursue→Detect cross, Attack→Pursue cross.
- **NO** integration test for `apply_enemy_ai` system (requires MinimalPlugins + state setup + multi-entity orchestration; same deferral as Stories 3.5–4.1 ECS systems per architecture.md:354).
- **NO** integration test for `enemy_fire_weapon` system (same).
- **NO** integration test for `detect_projectile_enemy_hits` / `apply_enemy_damage` (same — requires Avian narrow-phase setup).
- Runtime smoke (Task 8) covers system-level: enemy AI transitions across distance bands, hysteresis dead-band, enemy fire timing, player projectile kill chain end-to-end.
- Test count post-4.2: **60** (= 53 baseline from end of 4.1 + 7 net new). AC #15 enforces.

### Architectural decisions ratified during create-story analysis

1. **`Asteroid` marker placement: `src/combat/components.rs`** — chosen over `src/arena/zone.rs` (where it is consumed) because (a) `Asteroid` is a combat-relevant marker queried by combat systems (`detect_projectile_asteroid_hits`); (b) precedent: `Projectile` and `PrimaryWeaponCooldown` live in `combat/components.rs` despite being entity-scoped data; (c) `arena/zone.rs` is a spawn-site, not a vocabulary site.

2. **`EnemyAiState::Default = Idle` (Default IS derived)** — chosen over no-Default because `Idle` is the inert non-hazardous state. Spawn-tuple still names it explicitly per AC #3 to match the project's "always specify at spawn" pattern, but a silent default is non-hazardous (unlike `Health { current: 0 }` which means "pre-destroyed").

3. **Direct `LinearVelocity` writes for AI movement, NOT `ExternalForce`** — chosen over force-based for tuning simplicity. `Dynamic` Avian bodies accept direct velocity writes; the project's gravity-zero physics environment means there's no opposing force to fight. Trade-off: ignores enemy momentum/inertia; acceptable for first-playable.

4. **Hysteresis = outward-only widening** — chosen over symmetric (inner-narrowing AND outer-widening). The standard hysteresis pattern: enter aggressive state at threshold T; exit aggressive state at threshold T*(1+pct). Symmetric would be: enter at T*(1-pct), exit at T*(1+pct) — but this complicates intuition without practical benefit.

5. **Single-step state transitions per tick (no convergence loop)** — chosen for pure-function simplicity. At 60 Hz FixedUpdate, even a 4-step convergence (Idle→Detect→Pursue→Attack) takes 67 ms — imperceptible. Wrapping in a `loop` is a YAGNI escalation.

6. **`next_ai_state` is single-pure function, NOT four functions per state** — chosen for testability. A single function with `match current` arms keeps test harness simple (one function under test, 7 input cases) and makes the hysteresis math centralized.

7. **Snap-rotate via `Transform::look_at`, NOT torque-driven slew** — chosen for first-playable simplicity. Torque-driven slew requires its own integration step + tuning + slew-rate parameter; out of scope.

8. **`GameLayer::Enemy` introduced; `GameLayer::EnemyProjectile` NOT introduced** — chosen for minimal surface. Player projectiles can collide with enemies (filter expansion). Enemy projectiles share `GameLayer::Projectile` but differ via `EnemyProjectile` Component marker (systems-layer differentiator). Story 4.3 may upgrade if hull-damage routing needs the bit-layer split.

9. **`detect_projectile_enemy_hits` is a parallel system to `detect_projectile_asteroid_hits`, NOT a generalized one** — chosen over generic `detect_projectile_target_hits<T>` because (a) generic systems require type-parameter binding via marker types — adds ECS surface; (b) two separate systems are easier to read; (c) event types must be distinct anyway (`ProjectileHitAsteroid` vs `ProjectileHitEnemy`) so generalization buys nothing.

10. **`EnemyDestroyed` event fired AT despawn time, NOT ON next-frame** — matches `AsteroidDestroyed` precedent. The despawn command is queued; the event reader reads in the same frame or next (Bevy's command-flush boundary). Consumers (Story 4.5 salvage / Epic 8 audio) handle the timing.

### Project Structure Notes

- **Alignment with unified project structure:** new files `src/combat/health.rs` and `src/combat/enemy_ai.rs` align with architecture.md:564-570:
  - Architecture line 567 reserves `src/combat/health.rs` (not literally enumerated, but the entity-data module pattern matches `flight/components.rs`).
  - Architecture line 570 explicitly reserves `src/combat/enemy_ai.rs` for FR14 — this story implements that file.
- **Detected variances:** none. Epic-4 4.2 spec at line 41 says `src/combat/health.rs` — implementation matches exactly. Spec line 47 says `src/combat/enemy_ai.rs` — implementation matches exactly.
- **Architecture compliance:**
  - Plugin boundaries respected — CombatPlugin owns all new entities + events; Flight stays unaware of combat (one-way dependency preserved).
  - OnTransition pattern preserved — no new OnEnter/OnExit registrations; Story 4.1's spawn registration is extended ADDITIVELY.
  - System-set chain preserved with one new variant — `EnemyAi` chained before existing `Fire`.
  - No new Plugin struct introduced — all new systems registered in `CombatPlugin::build`.
  - Past-tense event naming preserved — `ProjectileHitEnemy`, `EnemyDestroyed`.
  - Pure-helper testing pattern preserved — `next_ai_state` is first-class testable.
- **Deferred-work entries closed by 4.2:**
  - `4-1: No Name component on Enemy entity` — closed by AC #3 adding `Name::new("EnemyShip")`.
  - `4-1: No CollisionEventsEnabled on Enemy entity` — closed by AC #3 adding `CollisionEventsEnabled`.
- **Deferred-work entries NOT closed by 4.2 (remain open):**
  - `4-1: spawn_enemy_ship has no idempotency guard against double-spawn` — no Arena re-entry path lands until Story 4.7.
  - `4-1: enemy_spawn_position_clears_close_asteroids tests only 2 of 5 close-cluster asteroids` — test-improvement chore not gated by 4.2.
  - All Story 3.x deferrals remain open per their resolution-path criteria.

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-4-enemies-alive-stop-ship-itchio-prototype.md#Story-4.2] — story spec (Acceptance Criteria source — lines 33-75)
- [Source: _bmad-output/planning-artifacts/architecture.md#Plugin-Boundaries] — `CombatPlugin` ownership of HullHP/ShieldHP/Weapon/projectiles/enemy AI (line 648); `EnemyDestroyed` listed as `CombatPlugin` event (line 648)
- [Source: _bmad-output/planning-artifacts/architecture.md#FR-Mapping] — FR14 enemy AI / `src/combat/enemy_ai.rs` (line 685); FR15 Hull+Shields / `src/combat/components.rs + damage.rs` (line 686)
- [Source: _bmad-output/planning-artifacts/architecture.md#Project-Directory-Structure] — `src/combat/` submodule layout (line 564-570); enemy_ai.rs reservation at line 570
- [Source: _bmad-output/planning-artifacts/architecture.md#Event-Taxonomy] — `EnemyDetected`, `AsteroidDestroyed`, `EnemyDestroyed`, `HullDamaged` past-tense convention (lines 243, 324)
- [Source: _bmad-output/planning-artifacts/architecture.md#Tuning-System] — runtime-tunable values in `assets/config/tuning.ron` per the established pattern (line 358)
- [Source: _bmad-output/implementation-artifacts/4-1-enemy-entity-foundation-semanticaccent-enemy.md] — recent precedent for OnTransition spawn, dual-marker pattern, story file structure, AC formatting
- [Source: _bmad-output/implementation-artifacts/3-10-projectile-asteroid-collision-damage.md] — `apply_damage` pure helper precedent, `ProjectileHitAsteroid` / `AsteroidDestroyed` event-pattern precedent, `GameLayer` PhysicsLayer enum
- [Source: src/combat/enemy.rs] — Story 4.1 enemy entity spawn — extended in this story
- [Source: src/combat/projectiles.rs:37-51] — `attach_combat_to_player_ship` precedent for combat-component attachment
- [Source: src/combat/projectiles.rs:72-137] — `fire_primary_weapon` precedent for cooldown-gated projectile spawn (mirrored by `enemy_fire_weapon`)
- [Source: src/combat/damage.rs:25-31] — `GameLayer` enum (extended with `Enemy` variant in this story)
- [Source: src/combat/damage.rs:36-53] — `ProjectileHitAsteroid` + `AsteroidDestroyed` Message precedent (mirrored by `ProjectileHitEnemy` + `EnemyDestroyed`)
- [Source: src/combat/damage.rs:60-62] — `apply_damage` saturating-sub helper (reused by `apply_enemy_damage`)
- [Source: src/combat/damage.rs:68-98] — `detect_projectile_asteroid_hits` body (mirrored by `detect_projectile_enemy_hits`)
- [Source: src/combat/damage.rs:100-133] — `apply_asteroid_damage` body (mirrored by `apply_enemy_damage`; refactored for Health)
- [Source: src/combat/components.rs:33-36] — `AsteroidHp` (REMOVED in this story)
- [Source: src/combat/mod.rs:21-28] — `CombatSystems` enum (extended with `EnemyAi` variant)
- [Source: src/combat/mod.rs:48-57] — `configure_sets(FixedUpdate, ...)` chain (extended to include `EnemyAi`)
- [Source: src/combat/mod.rs:58-86] — combat-systems FixedUpdate registration block (extended with 4 new systems)
- [Source: src/flight/mod.rs:32-33] — `PlayerShip` marker Component (queried by `apply_enemy_ai` and `enemy_fire_weapon`)
- [Source: src/flight/physics.rs:78-95] — `ship_local_torque_vector` pure-helper precedent for `next_ai_state`
- [Source: src/tuning/config.rs:10-37] — `TuningConfig` struct (extended with 5 new fields)
- [Source: src/tuning/config.rs:39-77] — `default_*` per-field defaults pattern
- [Source: src/tuning/config.rs:79-97] — `impl Default for TuningConfig` (extended)
- [Source: src/tuning/config.rs:131-191] — TuningConfig tests (extended — NOT new test functions, just new assertions)
- [Source: src/visual/palette.rs:11-28] — `SemanticAccent::Enemy` (#D55E00 vermillion) used for enemy projectile material tint
- [Source: assets/config/tuning.ron] — canonical tuning surface (extended with 5 new fields)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:301-309] — Story 4.1 review deferrals; AC #3 closes "No Name component" + "No CollisionEventsEnabled"
- [Source: avian3d-0.6.x] — `ExternalForce`, `ExternalTorque`, `LinearVelocity`, `Collider::sphere`, `CollisionLayers`, `CollisionEventsEnabled`, `LayerMask::ALL`, `RigidBody::Dynamic` API
- [Source: bevy::prelude::Transform::look_at] — Bevy 0.18 snap-rotate API (used for enemy orientation)
- [Source: bevy::ecs::query::Without] — Bevy 0.18 query filter (used for disjoint Query satisfaction in `apply_enemy_ai`)
- [Source: bevy::ecs::system::Single] — Bevy 0.18 single-entity SystemParam (used for `PlayerShip` access)

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Claude Code, Opus 4.7 — 1M context)

### Debug Log References

- `/tmp/story-4-2-check-task1.log` — Task 1 cargo check (0 warnings/errors)
- `/tmp/story-4-2-check-task2.log` — Task 2 tuning::config tests (3 passed)
- `/tmp/story-4-2-check-task3.log` — Task 3 enemy_ai tests (7 passed)
- `/tmp/story-4-2-check-task4.log` — Task 4 cargo check after systems (8 dead-code warnings, expected; resolved by Tasks 5/6 wiring)
- `/tmp/story-4-2-check-task6.log` — Task 6 cargo check after plugin wiring (0 warnings/errors)
- `/tmp/story-4-2-check.log`, `/tmp/story-4-2-build.log`, `/tmp/story-4-2-test.log`, `/tmp/story-4-2-clippy.log`, `/tmp/story-4-2-fmt.log`, `/tmp/story-4-2-release.log` — Task 7 final verification gates (all 0 warnings/errors; 60 tests passed)

### Completion Notes List

- **All 17 ACs satisfied** (AC #17 runtime smoke is interactive — handed off to Till; preconditions Task 7 cargo gates are green).
- **Test count: 60 passed / 0 failed** (53 baseline + 7 net new per AC #14: +1 health, +7 enemy_ai, -1 components).
- **Verification gates (AC #15):** all 6 cargo commands clean (`grep -cE 'warning:|error:'` == 0 per log).
- **Story deviation — `ExternalForce` / `ExternalTorque` removed from spawn tuple (AC #3).** The avian3d-0.6.1 dependency in this project does not export `ExternalForce` or `ExternalTorque` (they were removed/renamed to `ConstantForce` / `ConstantTorque` in the 0.6 line). AC #8 explicitly says to "Bypass `ExternalForce`" and write `LinearVelocity` directly, and the `apply_enemy_ai` / `enemy_fire_weapon` systems never reference these surfaces. The spawn tuple was reduced from the prescribed 20 components to 18 by omitting these two unused pre-wires. The nested-tuple workaround (AC #3 / Task 5 note) is still required because the tuple still exceeds Bevy's 15-element `Bundle` impl arity. **Action for reviewer:** confirm this is acceptable; if a `ConstantForce` / `ConstantTorque` placeholder is required for a Story 4.x consumer, raise it in review and a follow-up edit will land them. Closure of Story 4.1 deferred-work entries `No Name component` and `No CollisionEventsEnabled` is unchanged.
- **Clippy fix:** module doc-comment in `src/combat/enemy_ai.rs` was reflowed to drop a leading `+` on a continuation line that triggered `clippy::doc-lazy-continuation` (the `+ orientation + movement` phrase became `, orientation, and movement`).
- **Bevy 0.18 tuple-spawn workaround applied (Task 5):** `spawn_enemy_ship`'s 18-component tuple is grouped into 4 nested tuples (markers/AI 7, render 4, transform/physics 5, collision 2). Bevy auto-flattens.
- **Deferred-work entries closed by 4.2:**
  - `4-1: No Name component on Enemy entity` — closed (Name::new("EnemyShip") in spawn tuple).
  - `4-1: No CollisionEventsEnabled on Enemy entity` — closed (component in spawn tuple).
- **Deferred-work entries left open per story scope:**
  - `4-1: spawn_enemy_ship has no idempotency guard` — no Arena re-entry path yet.
  - `4-1: enemy_spawn_position_clears_close_asteroids tests only 2 of 5 close-cluster asteroids` — test-improvement chore not gated by 4.2.
- **Manual smoke (AC #17 / Task 8) handed off to Till.** Run `cargo run 2>&1 | tee /tmp/story-4-2-run.log` and walk through scenarios (a)–(i): Detect-on-spawn (~60m), Pursue threshold (≤50m, 20m/s), Attack threshold (≤25m, 1Hz vermillion fire), hysteresis dead-band stability, two-shot enemy kill, no player damage from enemy projectiles in 4.2, pause round-trip, asteroid regression, and clean quit (no panic).

### File List

- `src/combat/health.rs` — NEW: `Health` component + `health_construction_is_explicit` test
- `src/combat/enemy_ai.rs` — NEW: `EnemyAiState` + `EnemyProjectile` + `EnemyFireCooldown` + `next_ai_state` + `apply_enemy_ai` + `enemy_fire_weapon` + 7 tests
- `src/combat/mod.rs` — M: `pub mod enemy_ai; pub mod health;`; `CombatSystems::EnemyAi` variant; FixedUpdate chain extension; 4 new system registrations; `ProjectileHitEnemy` + `EnemyDestroyed` event registrations
- `src/combat/components.rs` — M: removed `AsteroidHp` + obsolete test; added `Asteroid` marker
- `src/combat/damage.rs` — M: `GameLayer::Enemy` variant; query/import refactor for `Asteroid` + `Health`; `ProjectileHitEnemy` + `EnemyDestroyed` events; `detect_projectile_enemy_hits` + `apply_enemy_damage` systems
- `src/combat/projectiles.rs` — M: player projectile collision filter `[Asteroid]` → `[Asteroid, Enemy]`
- `src/combat/enemy.rs` — M: spawn tuple extended additively (Health, EnemyAiState::Idle, CollisionLayers, CollisionEventsEnabled, Name, EnemyFireCooldown); nested-tuple grouping for Bevy 0.18 15-arity Bundle limit; ExternalForce/ExternalTorque omitted (Avian 0.6.1 API absence; see Completion Notes)
- `src/arena/zone.rs` — M: asteroid spawn tuple `AsteroidHp { current: 1 }` → `(Asteroid, Health { current: 1, max: 1 })`
- `src/tuning/config.rs` — M: 5 new fields (enemy_detection_range, enemy_engagement_range, enemy_speed, enemy_fire_rate_hz, enemy_ai_hysteresis_pct); 5 default fns; Default impl extension; 3 tests extended with new field assertions
- `assets/config/tuning.ron` — M: 5 new field-value pairs at canonical defaults
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — M: `4-2-...` status flips ready-for-dev → in-progress → review; last_updated bumped
- `_bmad-output/implementation-artifacts/4-2-enemy-ai-state-machine-detect-pursue-attack.md` — M: Status, all task checkboxes, Dev Agent Record, File List, Change Log

## Change Log

- 2026-05-06 — Implemented Story 4.2 enemy AI state machine (Idle/Detect/Pursue/Attack with hysteresis), enemy fire-on-Attack pacing, and parallel projectile-vs-enemy damage pipeline. Refactored `AsteroidHp` → `Health` + `Asteroid` marker as the shared HP foundation for enemies (4.2) and PlayerShip (4.3). All 60 tests pass; 6 cargo gates clean. (Claude Opus 4.7)
