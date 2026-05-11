# Story 4.3: Hull Component + Permadeath → PostRun State

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player flying the Arena cockpit ship,
I want my Hull to take damage from enemy projectiles and the run to end on Hull-zero,
So that permadeath per FR16 is real and combat has stakes — the third of three Epic 4 / M3 Itch.io stop-and-ship combat-loop stories (4.1 entity foundation → 4.2 AI alive → **4.3 hull + permadeath**).

## Acceptance Criteria

1. **(`TuningConfig.player_hull_max` — new field on `src/tuning/config.rs::TuningConfig`)** A new `pub player_hull_max: u32` field is added, with `#[serde(default = "default_player_hull_max")]` per the established forward-compat pattern. The default fn returns `3` (matches epic-4 spec line 87: "PlayerShip carries `Health { current: 3, max: 3 }` sourced from `TuningConfig.player_hull_max: u32 = 3`").
   ```rust
   #[serde(default = "default_player_hull_max")]
   pub player_hull_max: u32,

   fn default_player_hull_max() -> u32 { 3 }
   ```
   **And** `TuningConfig::default()` is extended with `player_hull_max: default_player_hull_max()`.
   **And** `assets/config/tuning.ron` is extended with `player_hull_max: 3` at the canonical default value (so the serialized canonical surface matches code defaults — same pattern as `enemy_*` fields in 4.2).
   **And** the existing 3 `tuning::config::tests` round-trip / RON-bytes / legacy-schema tests are extended with assertions for `player_hull_max`. NO new test functions; assertion count grows from 18 to 21 across the 3 test fns.

2. **(PlayerShip spawn-tuple extensions in `src/flight/mod.rs::spawn_player_ship`)** The existing spawn-tuple is extended in-place — **NO** new spawn function; same OnTransition registration. New components added (in canonical position alphabetically grouped with existing flight/avian components):
   - `Health { current: tuning.player_hull_max, max: tuning.player_hull_max }` — sourced from `TuningConfig.player_hull_max` (NOT hardcoded `3`); cold-start tuning fallback path returns the default `3` per `tuning_opt.cloned().unwrap_or_default()` precedent at `flight/mod.rs:97-101`.
   - `CollisionEventsEnabled` — closes deferred-work entry `4-2: Enemy projectiles physically collide with PlayerShip (Default layer) but no damage system handles these CollisionStart events` (line 333). Required so `detect_projectile_player_hits` (AC #5) receives `CollisionStart` events when enemy projectiles strike the ship. Mirrors enemy / asteroid `CollisionEventsEnabled` precedent.

   **And** the existing components (`PlayerShip, ArenaEntity, Mesh3d, MeshMaterial3d, Transform, OutlineVolume, RigidBody::Dynamic, Collider::sphere, LinearVelocity(ZERO), AngularVelocity(ZERO), default_input_map, ActionState<FlightAction>, DampenerState`) and the cockpit-camera child are **PRESERVED** — none are removed. Only ADDITIVE.
   **And** the spawn `info!` log line is unchanged.
   **And** **NO** explicit `CollisionLayers` is added to PlayerShip — the ship continues to inherit `CollisionLayers::DEFAULT` (membership=`[GameLayer::Default]`, filter=`LayerMask::ALL`). Enemy projectiles target `[GameLayer::Default]` (`enemy_ai.rs:233`), so the existing physics-layer wiring already routes the collision; only the event-emission opt-in is new.

3. **(`DeathCause` enum + `RunResult` resource — new types in `src/combat/damage.rs`)** Two new types are added alongside the existing `ProjectileHitAsteroid` / `EnemyDestroyed` / `apply_damage` triad:
   ```rust
   /// Cause-of-death taxonomy for `HullDepleted`. Story 4.3 only routes
   /// `EnemyFire`; `AsteroidCollision` and `Unknown` are pre-wired enum
   /// variants for forward-compat (asteroid-collision damage is a polish-pass
   /// item; `Unknown` covers future unattributable damage sources).
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum DeathCause {
       EnemyFire,
       AsteroidCollision,
       Unknown,
   }

   /// Run-end summary handed to PostRun (Story 4.9 consumer). Inserted as a
   /// `Resource` by `apply_player_damage` on Hull=0, read by Story 4.9's
   /// post-run summary screen, removed by Story 4.9 on PostRun → MainMenu /
   /// PostRun → Arena exit. Story 4.3 hardcodes `salvage_banked: 0` (Epic 6
   /// economy wires real value).
   #[derive(Resource, Debug, Clone, Copy)]
   pub struct RunResult {
       pub cause: DeathCause,
       pub salvage_banked: u32,
       pub run_duration_seconds: f32,
   }
   ```
   **And** **NO** `Default` derive on either type — `DeathCause::Default` would silently default to one variant (likely `EnemyFire`), which would mask attribution bugs; `RunResult::Default` would imply zero-cause-zero-duration which is misleading. Caller-side struct-literal construction matches the project's `Health` / `Projectile` / `EnemyShip` no-Default precedent.
   **And** the placement decision is `src/combat/damage.rs` (NOT a new `src/run/` module): architecture.md:687 anchors FR16 / `HullDepleted` / `RunEnded` to `src/run/director.rs`, but Epic 6 introduces `RunPlugin`. For Epic 4 first-playable, keeping these types in `combat/damage.rs` (alongside the damage-event family they belong to) is the lowest-friction placement; Epic 6 Story 6.1 may relocate `DeathCause` + `RunResult` into `src/run/director.rs` when `RunPlugin` lands. Inline doc-comment on both types records this forward-compat note.

4. **(`ProjectileHitPlayer` + `HullDepleted` events — Bevy 0.18 Messages in `src/combat/damage.rs`)** Two new Message-derived event types:
   ```rust
   /// Emitted by `detect_projectile_player_hits` when an enemy projectile and
   /// the PlayerShip begin contacting. Consumed by `apply_player_damage`.
   #[derive(Message, Debug, Clone, Copy)]
   pub struct ProjectileHitPlayer {
       pub projectile: Entity,
       pub player: Entity,
       pub damage: u32,
   }

   /// Emitted by `apply_player_damage` when PlayerShip Health reaches zero.
   /// Triggers `NextState<GameState>::PostRun` and the `RunResult` insertion.
   /// Story 4.9's PostRun summary screen is the downstream consumer.
   #[derive(Message, Debug, Clone, Copy)]
   pub struct HullDepleted {
       pub player: Entity,
       pub cause: DeathCause,
   }
   ```
   **And** both types are registered via `app.add_message::<>()` in `CombatPlugin::build` (alongside the existing `ProjectileHitAsteroid` / `AsteroidDestroyed` / `ProjectileHitEnemy` / `EnemyDestroyed`).
   **And** **NO** `#[allow(dead_code)]` on `HullDepleted.cause` — Story 4.3 itself reads this field when constructing `RunResult`. (Compare the 4.2 pattern where `EnemyDestroyed.enemy` is `#[allow(dead_code)]` because no in-story consumer reads it.)
   **And** `ProjectileHitPlayer` field naming mirrors `ProjectileHitAsteroid` exactly (`projectile`, `<target>`, `damage`) for cross-event symmetry; `HullDepleted` uses `player` + `cause` because no projectile is meaningful at the death-event boundary.

5. **(`detect_projectile_player_hits` system — `CombatSystems::EvaluateHits`, FixedUpdate)** New system in `src/combat/damage.rs`, gated by `in_state(GameState::Arena)`, mirroring `detect_projectile_enemy_hits` exactly except the right-side is the `PlayerShip` entity:
   ```rust
   pub fn detect_projectile_player_hits(
       mut collisions: MessageReader<CollisionStart>,
       projectiles: Query<&Projectile, With<EnemyProjectile>>,
       players: Query<(), With<PlayerShip>>,
       mut hits: MessageWriter<ProjectileHitPlayer>,
   )
   ```
   **And** the projectiles query is **filtered by `With<EnemyProjectile>`** — only enemy-fired projectiles are damage-eligible against the player. Without this filter, a player-fired projectile that somehow ended up colliding with the ship (e.g., a future tractor-beam-arc edge case) would self-damage. The collision-layer design already prevents this physically (player projectiles filter `[Asteroid, Enemy]`, NOT `[Default]`), but the marker-filter is defense-in-depth.
   **And** both collider1/collider2 orderings are tried (Avian gives no canonical ordering — same pattern as `detect_projectile_asteroid_hits` and `detect_projectile_enemy_hits`).
   **And** the `Projectile` lookup uses `.get(...).expect("projectile_entity verified above")` per the existing pattern at `damage.rs:113`. **NOTE:** `deferred-work.md:Story 4.2 review entry "Stale CollisionStart events may reference TTL-despawned projectile entities"` flags this `.expect()` as a latent panic risk; it remains DEFERRED per that entry's resolution path (Epic 10 hardening).
   **And** `crate::combat::enemy_ai::EnemyProjectile` is added to the `use` block at the top of `damage.rs`.

6. **(`apply_player_damage` system — `CombatSystems::ApplyDamage`, FixedUpdate)** New system in `src/combat/damage.rs`, gated by `in_state(GameState::Arena)`, mirroring `apply_enemy_damage` body except: the player is NOT despawned on HP=0; instead `HullDepleted` is emitted and `RunResult` is inserted as a `Resource`:
   ```rust
   pub fn apply_player_damage(
       mut hits: MessageReader<ProjectileHitPlayer>,
       mut commands: Commands,
       mut players: Query<&mut Health, With<PlayerShip>>,
       mut depleted: MessageWriter<HullDepleted>,
       run_start: Res<RunStartedAt>,
       virtual_time: Res<Time<Virtual>>,
   )
   ```
   System body responsibilities:
   - **Despawn projectile unconditionally** — single-hit-per-projectile (Epic 3 AC, mirrored). `commands.entity(event.projectile).despawn();`
   - **Apply damage if player still alive** — same two-guard pattern as `apply_asteroid_damage`: (1) `players.get_mut(event.player)` Err on flushed-despawn (programmer-error case for player; player despawn does NOT happen in 4.3 but the guard is symmetry-preserving free), (2) `hp.current == 0` continue (already-dead-this-tick guard for multi-hit-same-tick races).
   - **Saturating-sub** via `apply_damage(hp.current, event.damage)` — pure helper precedent.
   - **On `hp.current == 0` after the sub**: emit `HullDepleted { player: event.player, cause: DeathCause::EnemyFire }`; insert `RunResult { cause: DeathCause::EnemyFire, salvage_banked: 0, run_duration_seconds: virtual_time.elapsed_secs() - run_start.0 };`; emit `info!("hull depleted: cause={:?} run_duration={:.2}s", cause, run_duration_seconds);`. **DO NOT despawn** the player — Arena → PostRun cleanup is owned by AC #11's `OnTransition` cleanup hook.

   **And** `RunResult` is inserted via `commands.insert_resource(RunResult { ... })`, NOT `commands.insert_resource_if_new(...)` — re-insertion is the explicit overwrite case for back-to-back deaths.
   **And** the cause-attribution is hardcoded `DeathCause::EnemyFire` because `ProjectileHitPlayer` events only fire from the `EnemyProjectile`-marker filter (AC #5). The other `DeathCause` variants are forward-compat slots, not active cases in Story 4.3.

7. **(`check_player_death` system — `CombatSystems::CheckDeath`, FixedUpdate)** A new `CombatSystems::CheckDeath` set is added to the chain (positioned AFTER `ApplyDamage`), per architecture.md:512 explicit guidance ("`CombatSystems::EvaluateHits, ApplyDamage, CheckDeath`"). System body:
   ```rust
   pub fn check_player_death(
       mut depleted: MessageReader<HullDepleted>,
       mut next_state: ResMut<NextState<GameState>>,
   ) {
       for event in depleted.read() {
           info!("transitioning to PostRun (cause={:?} player={:?})", event.cause, event.player);
           next_state.set(GameState::PostRun);
           return; // Single death-per-tick semantics; multi-event drains harmlessly.
       }
   }
   ```
   **And** the `return` after `next_state.set(...)` is intentional — multiple `HullDepleted` events within a single tick (a multi-projectile-hit case) should result in a single state transition. Subsequent events in the iterator are drained but ignored.
   **And** **NO** `if depleted.is_empty() { return; }` early-return guard — the for-loop is itself the no-op when empty.
   **And** the system is gated by `in_state(GameState::Arena)` — same `run_if` posture as the rest of `CombatSystems`. PostRun-state `HullDepleted` events (impossible in Epic 4 — there is no enemy in PostRun — but defensive against Epic 5+ regression) would otherwise re-trigger the transition.

8. **(`RunStartedAt` resource — wall-clock anchor for `run_duration_seconds`)** A new `Resource` in `src/combat/damage.rs`:
   ```rust
   /// Anchor for `RunResult.run_duration_seconds`. Records the value of
   /// `Time<Virtual>::elapsed_secs()` at Arena entry; subtracted at HullDepleted
   /// to yield wall-clock-of-unpaused-gameplay duration. Idempotent overwrite
   /// at every Arena entry — no cleanup needed.
   #[derive(Resource, Debug, Clone, Copy)]
   pub struct RunStartedAt(pub f32);
   ```
   **And** a new system `record_run_started_at` is registered on `OnTransition { MainMenu → Arena }`, in a new position chained AFTER existing `CombatSystems::Setup`:
   ```rust
   pub fn record_run_started_at(
       virtual_time: Res<Time<Virtual>>,
       mut commands: Commands,
   ) {
       commands.insert_resource(RunStartedAt(virtual_time.elapsed_secs()));
       info!("run started at virtual elapsed = {:.2}s", virtual_time.elapsed_secs());
   }
   ```
   **And** `Time<Virtual>` is the right clock: it pauses with `pause_simulation_clocks` (`pause/mod.rs:122`), so paused intervals do NOT inflate `run_duration_seconds`. `Time<Real>` would inflate; `Time<Fixed>` is FixedUpdate-only.
   **And** `RunStartedAt` is registered to `CombatSystems::Setup` (same set as `attach_combat_to_player_ship` + `spawn_enemy_ship`) — set-ordering between siblings within a set is unconstrained, which is correct here since none read each other.

9. **(`CombatPlugin::build` — FixedUpdate set-graph extension)** `src/combat/mod.rs::CombatPlugin::build` is updated:
   - **System set graph extension:** `configure_sets(FixedUpdate, ...)` adds `CombatSystems::CheckDeath` chained AFTER `ApplyDamage`. Final chain: `(EnemyAi, Fire, Lifecycle, EvaluateHits, ApplyDamage, CheckDeath).chain()`.
   - **System set enum extension:** `CombatSystems` gains a `CheckDeath` variant (positioned after `ApplyDamage` for declaration-order alignment with chain order).
   - **System registration (FixedUpdate):**
     - `damage::detect_projectile_player_hits.in_set(CombatSystems::EvaluateHits).run_if(in_state(GameState::Arena))`
     - `damage::apply_player_damage.in_set(CombatSystems::ApplyDamage).run_if(in_state(GameState::Arena))`
     - `damage::check_player_death.in_set(CombatSystems::CheckDeath).run_if(in_state(GameState::Arena))`
   - **System registration (OnTransition MainMenu → Arena):** `damage::record_run_started_at.in_set(CombatSystems::Setup)`
   - **Event registration:** `app.add_message::<ProjectileHitPlayer>(); app.add_message::<HullDepleted>();` alongside existing 4 messages.

   **And** the existing 4.2 OnTransition Setup tuple (`projectiles::attach_combat_to_player_ship`, `enemy::spawn_enemy_ship`) gains a third entry: `damage::record_run_started_at` — ALL three live in `CombatSystems::Setup`. Final tuple: `(attach_combat_to_player_ship, spawn_enemy_ship, record_run_started_at)`.

10. **(HUD live-wiring for Hull — new system in `src/ui/hud.rs`)** A new system `update_hud_hull` reads PlayerShip `Health.current` and writes the Hull Text node live:
    ```rust
    pub fn update_hud_hull(
        players: Query<&Health, (With<PlayerShip>, Changed<Health>)>,
        mut hud_texts: Query<(&mut Text, &HudPlaceholder)>,
    ) {
        let Ok(health) = players.single() else { return; };
        for (mut text, placeholder) in &mut hud_texts {
            if placeholder.field == HudField::Hull {
                **text = format!("HULL {}", health.current);
            }
        }
    }
    ```
    **And** the `Changed<Health>` filter ensures the for-loop only iterates when PlayerShip Health was mutated this tick — first-playable performance posture (one PlayerShip, one Health per ship; the Changed filter prevents redundant Text writes every frame).
    **And** the Text mutation uses `**text = format!(...)` per Bevy 0.18 `Text(String)` deref pattern (NOT `text.0 = ...`).
    **And** the system is registered in `UiPlugin::build` (`src/ui/mod.rs`) on `Update` schedule, gated by `run_if(in_state(GameState::Arena))`. Pause-state HUD updates are skipped (correct: paused gameplay should not visually mutate state).
    **And** the placement decision is `src/ui/hud.rs` (NOT `src/combat/`) — HUD mutation is a UI concern reading combat state. Mirrors the architecture.md:592 mapping (`src/ui/hud.rs` for screen-space tactical-state).
    **And** `crate::combat::health::Health` and `crate::flight::PlayerShip` are added to the `use` block at `src/ui/hud.rs` top.
    **And** the existing `HUD_TEXT_COLOR` / `HUD_FONT_SIZE` / corner-margin constants are unchanged — only the Text content is mutated, not the styling.

11. **(`OnTransition Arena → PostRun` cleanup — closes deferred-work line 252 for the PostRun branch)** `src/arena/mod.rs::ArenaPlugin::build` is extended with a parallel cleanup registration:
    ```rust
    app.add_systems(
        OnTransition {
            exited: GameState::Arena,
            entered: GameState::PostRun,
        },
        cleanup_on_exit::<ArenaEntity>,
    );
    ```
    **And** placement is alongside the existing `OnTransition { Arena → MainMenu }` cleanup at `arena/mod.rs:43-50`.
    **And** the inline comment block at `arena/mod.rs:39-43` is updated: the line "Forward-compat: Arena → MainMenu wiring lands in Story 4.7 title-screen restart flow; Arena → PostRun in Epic 4 death/run-end flow." gets the `Arena → PostRun` clause struck out, and a "(closed Story 4.3)" note is appended. Caravan / PhotoMode branches remain dormant.
    **And** the existing `cleanup_on_exit::<T>` generic helper at `arena/mod.rs:54-58` is unchanged — only a new `add_systems` registration is added.
    **And** the cleanup is ALL-`ArenaEntity`-marked entities: PlayerShip, asteroids, enemy ship, all live projectiles (player + enemy), HUD nodes (dual-marked `HudEntity` + `ArenaEntity`), and the directional-light. Story 4.9 spawns its PostRun UI fresh in `OnEnter(PostRun)` after the cleanup completes.

12. **(Imports — minimal but complete)**

    `src/combat/damage.rs` — `use` block additions:
    ```rust
    use crate::combat::enemy_ai::EnemyProjectile;
    use crate::flight::PlayerShip;
    use crate::state::GameState;
    ```

    `src/combat/mod.rs` — `use` block additions:
    ```rust
    use crate::combat::damage::{HullDepleted, ProjectileHitPlayer};
    ```

    `src/flight/mod.rs` — `use` block addition:
    ```rust
    use avian3d::prelude::CollisionEventsEnabled;
    use crate::combat::health::Health;
    ```
    (`AngularVelocity, Collider, LinearVelocity, RigidBody` already imported at line 8; only `CollisionEventsEnabled` is new.)

    `src/ui/hud.rs` — `use` block additions:
    ```rust
    use crate::combat::health::Health;
    use crate::flight::PlayerShip;
    use crate::state::GameState;
    ```

    `src/ui/mod.rs` — `use` block addition:
    ```rust
    use bevy::prelude::*;  // already present via other registrations
    ```
    (no new types — `hud::update_hud_hull` is referenced via `hud::` path.)

13. **(Tests — pure-helper + invariant coverage)** Net new tests:

    **`src/combat/damage.rs::tests` (3 new tests):**
    ```rust
    #[test]
    fn death_cause_variants_are_distinct() {
        // No Default derive — guards future variant additions; round-trip explicit
        // construction. Mirrors HudField::variants_distinct precedent at hud.rs:154.
        assert_ne!(DeathCause::EnemyFire, DeathCause::AsteroidCollision);
        assert_ne!(DeathCause::AsteroidCollision, DeathCause::Unknown);
        assert_ne!(DeathCause::EnemyFire, DeathCause::Unknown);
    }

    #[test]
    fn run_result_construction_is_explicit() {
        // No Default derive — silent default would imply zero-cause-zero-duration
        // which is misleading. Round-trip explicit construction.
        let r = RunResult {
            cause: DeathCause::EnemyFire,
            salvage_banked: 0,
            run_duration_seconds: 42.5,
        };
        assert_eq!(r.cause, DeathCause::EnemyFire);
        assert_eq!(r.salvage_banked, 0);
        assert!((r.run_duration_seconds - 42.5).abs() < 1e-5);
    }

    #[test]
    fn run_started_at_construction_is_explicit() {
        // No Default derive — captures intent at the call site.
        let r = RunStartedAt(0.0);
        assert_eq!(r.0, 0.0);
    }
    ```

    **`src/tuning/config.rs::tests`:** the existing 3 tests are extended with `assert_eq!(cfg.player_hull_max, 3)` (default test), `assert_eq!(cfg.player_hull_max, 5)` for the round-trip RON-bytes test (RON bytes string is extended with `, player_hull_max: 5`), and `assert_eq!(cfg.player_hull_max, 3)` (legacy-schema default test). NO new test fns — assertion count grows by 3 across the 3 fns.

    **`src/combat/health.rs::tests`:** UNCHANGED — the existing `health_construction_is_explicit` already covers the `Health` no-Default discipline that PlayerShip relies on.

    **NO new tests in `src/flight/mod.rs::tests`:** the spawn-tuple extensions are integration-level concerns; the existing flight tests are pure-helper tests (`apply_dampener`, `mouse-look`, etc.). Adding a "spawn tuple contains Health" test would require Bevy world setup and is over-engineering for first-playable. The runtime smoke (AC #16) is the verification path.

    **NO new tests in `src/ui/hud.rs::tests`:** `update_hud_hull` is a Query+system function; the format-string `"HULL {n}"` is a one-liner that the runtime smoke verifies. The existing `hud_corner_labels_contain_expected_field_names` test already covers the static-label contract.

    **Net new test functions across the codebase: +3** (3 in damage; 0 in tuning fns although assertions added; 0 in health/flight/hud). Net post-4.3 test count: **60 + 3 = 63**. AC #15 enforces.

14. **(Verification gates — all 6 cargo commands clean)** Per `feedback_full_build_output.md` discipline, exit-0 + tail is NOT proof; full output is captured per command and grep'd for `warning:|error:`.
    **Then** **all six** of the following produce **0** lines matching `grep -cE 'warning:|error:'` per their respective full-output logs:
    ```bash
    cargo check                                         2>&1 | tee /tmp/story-4-3-check.log
    cargo build                                         2>&1 | tee /tmp/story-4-3-build.log
    cargo test                                          2>&1 | tee /tmp/story-4-3-test.log
    cargo clippy --all-targets -- -D warnings           2>&1 | tee /tmp/story-4-3-clippy.log
    cargo fmt --all -- --check                          2>&1 | tee /tmp/story-4-3-fmt.log
    cargo build --release                               2>&1 | tee /tmp/story-4-3-release.log
    ```
    **And** `cargo test` summary line reads `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N = 63** (= 60 baseline from end of 4.2 + 3 net new per AC #13).

15. **(File set — `git status --short` final)** Final set is **exactly**:
    - `M src/flight/mod.rs` (M — extend PlayerShip spawn-tuple with `Health` + `CollisionEventsEnabled`; add `Health` + `CollisionEventsEnabled` imports)
    - `M src/combat/damage.rs` (M — add `DeathCause`, `RunResult`, `RunStartedAt`, `ProjectileHitPlayer`, `HullDepleted` types; add `detect_projectile_player_hits`, `apply_player_damage`, `check_player_death`, `record_run_started_at` systems; +3 tests; new imports)
    - `M src/combat/mod.rs` (M — `CombatSystems::CheckDeath` variant; system-set chain extension; system registrations; event registrations; new imports)
    - `M src/arena/mod.rs` (M — `OnTransition Arena → PostRun` cleanup registration; comment block update)
    - `M src/ui/hud.rs` (M — `update_hud_hull` system; new imports)
    - `M src/ui/mod.rs` (M — `update_hud_hull` registration on Update gated by Arena)
    - `M src/tuning/config.rs` (M — `player_hull_max` field + default fn + Default impl extension; tests extended)
    - `M assets/config/tuning.ron` (M — `player_hull_max: 3` field added)
    - `M _bmad-output/implementation-artifacts/sprint-status.yaml` (M — `4-3-...: backlog → ready-for-dev → in-progress → review → done`, `last_updated`)
    - `M _bmad-output/implementation-artifacts/deferred-work.md` (M — close 2 entries: line 252 PostRun-cleanup branch, line 333 enemy-projectile-vs-player damage)
    - `?? _bmad-output/implementation-artifacts/4-3-hull-component-permadeath-postrun-state.md` at story-creation time (becomes M after dev flips Status / fills Dev Agent Record / Change Log)

    **NO** entries under: `Cargo.toml` / `Cargo.lock` (no dep added — `Time<Virtual>`, `NextState`, `Message` already in scope via Bevy / Avian preludes), `src/combat/components.rs`, `src/combat/health.rs`, `src/combat/enemy.rs`, `src/combat/enemy_ai.rs`, `src/combat/projectiles.rs`, `src/combat/input.rs`, `src/arena/zone.rs`, `src/state.rs` (GameState::PostRun already exists), `src/pause/**`, `src/visual/**`, `src/splash.rs`, `src/logging.rs`, `src/main.rs`, `assets/meshes/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`.

16. **(Runtime smoke — full hull-damage-to-PostRun chain)** After AC #14 cargo gates green, Till manually executes `cargo run 2>&1 | tee /tmp/story-4-3-run.log` and verifies:
    - **(a) HUD shows `HULL 3` on Arena entry** — Press Enter on MainMenu. Top-right HUD reads `HULL 3` (NOT `HULL 100` from the Story 3.11 placeholder). `grep -c 'HULL 3' /tmp/story-4-3-run.log` is not directly meaningful (the value is rendered, not logged); visual confirmation only.
    - **(b) Enemy projectile hit decrements Hull** — Approach the enemy (W to thrust forward). Stay still while enemy enters Attack range and fires. On the first projectile contact, HUD updates from `HULL 3` → `HULL 2`. `grep -c 'projectile fired' /tmp/story-4-3-run.log` ≥ 1; subsequent hits decrement to `HULL 1` then `HULL 0`.
    - **(c) Hull-zero triggers PostRun transition** — On the third enemy hit, `HUD` briefly shows `HULL 0` before Arena cleanup; `grep -c 'hull depleted: cause=EnemyFire' /tmp/story-4-3-run.log` ≥ 1; `grep -c 'transitioning to PostRun' /tmp/story-4-3-run.log` ≥ 1; the screen goes black or shows whatever PostRun's stub is (no PostRun UI lands until Story 4.9 — for 4.3 this manifests as: cockpit despawns, cursor releases, screen goes black or shows the splash-camera fallback).
    - **(d) `RunResult` resource is queryable in PostRun** — manually verify by adding a temporary `dbg!` line in PostRun-state-entry hook OR via `bevy-inspector-egui` if installed. `RunResult.cause == DeathCause::EnemyFire`; `RunResult.salvage_banked == 0`; `RunResult.run_duration_seconds` is a positive float matching wall-clock-of-unpaused-Arena-time. (Optional verification — `dbg!` line is removed before commit.)
    - **(e) Arena cleanup is complete** — `grep -cE 'PlayerShip|EnemyShip|Asteroid|spawned' /tmp/story-4-3-run.log | head` shows spawn lines but NO post-PostRun-entry spawn lines; world is empty after the transition.
    - **(f) Pause does not advance run_duration_seconds** — Esc-pause, wait 5 seconds, Esc-resume. Inspect `RunResult.run_duration_seconds` after the next death — paused interval should NOT contribute to the total. (This validates AC #8 `Time<Virtual>` choice.)
    - **(g) Player projectiles do NOT damage player** — Fire LMB while moving. No HUD decrement. `grep -c 'projectile.*hit.*player' /tmp/story-4-3-run.log` = 1 line per enemy-projectile-hit only; player-projectiles do NOT match the `EnemyProjectile`-marker filter (AC #5 defense-in-depth).

17. **(Pre-flight: NO out-of-scope work.)** Story 4.3 explicitly does NOT:
    - Add `Shields` component or shield regen — Epic 5 (5.1, 5.2, 5.3).
    - Add `HullHP` / `ShieldHP` formal-split components — Epic 5 (5.1).
    - Add asteroid-collision-induced hull damage — `DeathCause::AsteroidCollision` is a forward-compat enum slot only.
    - Add post-run summary screen UI — Story 4.9 (consumes `RunResult` resource).
    - Add Retry / Main Menu buttons — Story 4.9.
    - Add `RunStarted` / `RunEnded` events from architecture.md:651 — Epic 6 (6.1) when `RunPlugin` lands.
    - Wire `salvage_banked` to a real currency — Epic 6 (6.5, 6.7); hardcoded `0` in 4.3.
    - Address `PauseLatch` deferred-work entry (line 184) — explicitly out of scope; the focus-loss-during-Esc-pause auto-resume race is not relevant to 4.3 unless playtesting hits it.
    - Add `bevy-inspector-egui` as a dependency — runtime smoke AC #16(d) is optional manual verification only.

## Tasks / Subtasks

- [x] **Task 1 — Tuning extension** (AC: #1)
  - [x] Add `player_hull_max: u32` field + `default_player_hull_max() -> u32 { 3 }` fn to `src/tuning/config.rs`
  - [x] Extend `TuningConfig::default()` with the new field
  - [x] Extend the 3 existing `tuning::config::tests` with `player_hull_max` assertions; add `player_hull_max: 5` to the round-trip RON bytes string
  - [x] Add `player_hull_max: 3` to `assets/config/tuning.ron`
  - [x] Run `cargo test tuning::config::tests` — all 3 tests pass with the +3 assertions

- [x] **Task 2 — PlayerShip spawn-tuple extension** (AC: #2)
  - [x] Add `use avian3d::prelude::CollisionEventsEnabled;` and `use crate::combat::health::Health;` to `src/flight/mod.rs`
  - [x] In `spawn_player_ship`, extend the spawn tuple with `Health { current: tuning.player_hull_max, max: tuning.player_hull_max }` and `CollisionEventsEnabled` (additive — preserves all 13 existing components; final arity = 15, at the Bundle-derive cap)
  - [x] `cargo build` — verifies clean (no nested-tuple grouping needed at arity 15)

- [x] **Task 3 — Damage event types + DeathCause + RunResult** (AC: #3, #4, #8)
  - [x] Add `DeathCause` enum to `src/combat/damage.rs` (3 variants, no Default; per-variant `#[allow(dead_code)]` on the two forward-compat slots)
  - [x] Add `RunResult` Resource (3 fields, no Default; struct-level `#[allow(dead_code)]` until Story 4.9 reads the fields)
  - [x] Add `RunStartedAt(pub f32)` Resource (no Default)
  - [x] Add `ProjectileHitPlayer` Message (3 fields)
  - [x] Add `HullDepleted` Message (2 fields)
  - [x] All 5 types carry inline doc-comments per the existing `ProjectileHitAsteroid` / `EnemyDestroyed` precedent

- [x] **Task 4 — Damage detection + application + death systems** (AC: #5, #6, #7, #8)
  - [x] Add `crate::combat::enemy_ai::EnemyProjectile`, `crate::flight::PlayerShip`, `crate::state::GameState` imports to `src/combat/damage.rs`
  - [x] Implement `detect_projectile_player_hits` with `With<EnemyProjectile>` filter + dual-ordering pattern
  - [x] Implement `apply_player_damage` with two-guard pattern; emit `HullDepleted`; insert `RunResult`
  - [x] Implement `check_player_death`; transitions to `GameState::PostRun`. AC's `for + return` pattern triggered clippy `never_loop`; refactored to functionally-equivalent `if let Some(event) = depleted.read().next()` (single-event semantics preserved; remaining events stay queued harmlessly).
  - [x] Implement `record_run_started_at`; inserts `RunStartedAt(virtual_time.elapsed_secs())`

- [x] **Task 5 — CombatPlugin wiring** (AC: #9)
  - [x] Add `CombatSystems::CheckDeath` variant to the `CombatSystems` enum
  - [x] Extend `configure_sets(FixedUpdate, ...)` chain to include `CheckDeath` after `ApplyDamage`
  - [x] Register the 3 new FixedUpdate systems with their respective sets + Arena gate
  - [x] Register `record_run_started_at` in `CombatSystems::Setup` on the OnTransition tuple (alongside `attach_combat_to_player_ship` and `spawn_enemy_ship`)
  - [x] Register `ProjectileHitPlayer` + `HullDepleted` via `app.add_message::<>()`
  - [x] Add `crate::combat::damage::{HullDepleted, ProjectileHitPlayer}` to imports

- [x] **Task 6 — HUD live-wiring** (AC: #10)
  - [x] Add `crate::combat::health::Health`, `crate::flight::PlayerShip` imports to `src/ui/hud.rs` (no `GameState` import needed in hud.rs — Arena gate lives on the registration in `ui/mod.rs`)
  - [x] Implement `update_hud_hull` with `Changed<Health>` filter
  - [x] Register `update_hud_hull` in `UiPlugin::build` on `Update` gated by `in_state(GameState::Arena)`
  - [x] Removed obsolete `#[allow(dead_code)]` on `HudPlaceholder.field` — Story 4.3 now reads it.

- [x] **Task 7 — Arena → PostRun cleanup** (AC: #11)
  - [x] Add `OnTransition { Arena → PostRun }` registration to `ArenaPlugin::build` calling `cleanup_on_exit::<ArenaEntity>`
  - [x] Update the inline comment block at `arena/mod.rs:39-43` to mark "Arena → PostRun" as closed by Story 4.3

- [x] **Task 8 — Tests** (AC: #13)
  - [x] Add 3 new tests in `src/combat/damage.rs::tests`: `death_cause_variants_are_distinct`, `run_result_construction_is_explicit`, `run_started_at_construction_is_explicit`
  - [x] Run `cargo test` — total = 63 (60 baseline + 3 net new), all pass.

- [x] **Task 9 — Verification gates** (AC: #14)
  - [x] Run all 6 cargo commands (check / build / test / clippy / fmt / release) with `2>&1 | tee /tmp/story-4-3-{name}.log`
  - [x] `grep -cE 'warning:|error:' /tmp/story-4-3-*.log` returns 0 for all 6 logs
  - [x] Fixed clippy `never_loop` at root (refactored `for + return` → `if let Some(...).next()`); fixed pre-existing fmt drift in `src/combat/components.rs:39` (trailing blank line) and `src/combat/enemy_ai.rs:158` (4.2 look_at-patch multi-line `if/else`) via `cargo fmt --all` — see Completion Notes for AC #15 deviation.

- [ ] **Task 10 — Runtime smoke** (AC: #16) — *Deferred to Till's manual verification*
  - [ ] `cargo run 2>&1 | tee /tmp/story-4-3-run.log` and walk through smoke scenarios (a)–(g) from AC #16
  - [ ] Confirm `HULL 3 → 2 → 1 → 0` decrement on enemy hits, PostRun transition, no player-self-damage from own projectiles
  - [ ] Confirm pause does NOT advance `run_duration_seconds`

- [x] **Task 11 — Close deferred-work entries** (closes existing entries 252 + 333)
  - [x] In `_bmad-output/implementation-artifacts/deferred-work.md`, appended `> **✅ CLOSED 2026-05-08 by Story 4.3** ...` to entry at line 252 (Arena → PostRun cleanup branch — PostRun branch only; Caravan + PhotoMode remain dormant) and line 333 (enemy-projectile-vs-player damage routing).

### Review Findings

- [x] [Review][Defer] `RunStartedAt` stale anchor if future PostRun → Arena bypass skips MainMenu [src/combat/damage.rs] — deferred, forward-compat risk: `record_run_started_at` is registered on `OnTransition { MainMenu → Arena }` only; if a future story adds PostRun → Arena directly (e.g., Retry flow in Story 4.9), `RunStartedAt` retains the previous run's value and `run_duration_seconds` would be inflated. Resolution: also register `record_run_started_at` on `OnTransition { PostRun → Arena }` when that transition is introduced.
- [x] [Review][Defer] `player_hull_max: 0` in tuning.ron produces permanently-undead PlayerShip [src/tuning/config.rs, assets/config/tuning.ron] — deferred, tuning validation gap: `Health { current: 0, max: 0 }` causes `hp.current == 0` guard in `apply_player_damage` to always `continue`; `HullDepleted` never fires; player cannot die. Add `player_hull_max >= 1` validation in `TuningConfig` loading (Epic 10 polish pass or first time a misconfigured `tuning.ron` surfaces in playtesting).
- [x] [Review][Defer] HULL 0 never displayed on HUD before Arena→PostRun transition [src/ui/hud.rs, src/ui/mod.rs] — deferred, cosmetic: `update_hud_hull` runs on `Update` with `run_if(Arena)`; the `Changed<Health>` mutation from FixedUpdate (HP hits 0) is gated out by the state-transition on the next `Update` tick; HUD counter stays at 1 (last pre-death value). Minor UX gap acceptable for first-playable. Revisit in Epic 10 HUD polish pass.
- [x] [Review][Defer] `cleanup_on_exit::<ArenaEntity>` orphan risk for future untagged `with_children` descendants [src/arena/mod.rs] — deferred, pre-existing pattern risk: helper calls `despawn()` (Bevy 0.18 recursive) per query result; any future child added via `with_children` that is NOT tagged `ArenaEntity` would receive a double-despawn warning (or be missed if not tagged). All current descendants are dual-tagged; no immediate issue. Mitigation path: Epic 10 audit of `with_children` usage to ensure all descendants are tagged or switch root nodes to explicit `despawn_recursive`.

## Dev Notes

### Architectural Anchors

**Damage routing pattern (architecture.md:485-507, :512-527).** Story 4.3 follows the canonical CombatPlugin shape: `EvaluateHits` (collision-pair → event), `ApplyDamage` (event → HP mutation), `CheckDeath` (HP=0 → state transition / despawn). Architecture.md:512 explicitly names `CombatSystems::CheckDeath` as a peer to `EvaluateHits` and `ApplyDamage`. Story 4.3 introduces `CheckDeath` for the first time (4.2 fused death-detection into `apply_enemy_damage` because the enemy's death is a despawn-and-emit-event, not a state transition; the player's death is a state transition that must run AFTER the HP mutation flushes, so the separate set is justified).

**Event past-tense convention (architecture.md:324).** `HullDepleted` is past-tense ("Hull has been depleted"). `ProjectileHitPlayer` is past-tense ("a projectile has hit the player"). Both fit the "fires after the fact, consumers react" idiom.

**No god-structs (architecture.md:74, :460).** `Health { current, max }` is a single-responsibility component shared across asteroids, enemies, and the player ship. Adding `regen_rate` / `cooldown_remaining` would violate single-responsibility — those land on a separate `ShieldHP` component in Epic 5 (architecture.md:476-482 shows the precedent).

**Resource convention (architecture.md:325, :244).** `RunResult` is a Resource (continuous-state holder), not an event. It is consumed by Story 4.9's PostRun screen at a specific frame; events would have lifetime issues (Bevy events drop after 2 frames; PostRun screen entry might race the event consumption).

### Code-Reuse Discipline (LLM Wheel-Reinvention Prevention)

**REUSE — DO NOT DUPLICATE:**
- `apply_damage(current, damage)` from `src/combat/damage.rs:83` — saturating-sub helper, already proves out 1-HP and over-damage cases.
- `Health` from `src/combat/health.rs:15` — already authored in Story 4.2 with no-Default discipline.
- `cleanup_on_exit::<T>` from `src/arena/mod.rs:54` — generic cleanup already drops `ArenaEntity` entities, just needs a new OnTransition registration.
- `HudPlaceholder` + `HudField::Hull` from `src/ui/hud.rs:32, :44` — Story 3.11 already wired the placeholder slot; Story 4.3 only needs to add the update system.
- `MessageReader<CollisionStart>` + dual-ordering collider1/collider2 pattern from `damage.rs:91-120` — pattern is repeated for asteroid + enemy + now player.

**DO NOT REINVENT:**
- A new "PlayerHealth" or "Hull" component — `Health` is the shared vocabulary per architecture.md:74.
- A new GameLayer for PlayerShip — the ship inherits `CollisionLayers::DEFAULT`, which is `[GameLayer::Default]` (membership) + `LayerMask::ALL` (filter). Enemy projectiles already filter `[GameLayer::Default]`. The wiring works as-is.
- A new despawn pattern for the player — Arena cleanup is `cleanup_on_exit::<ArenaEntity>`, transitively despawns PlayerShip via the `ArenaEntity` marker.
- A `DeathCause::Shotgun` or other weapon-archetype variants — Story 4.4's scope.
- A retry-flow / scene-reload — Story 4.9's scope (Retry button calls `NextState<GameState>::Arena`).

### Previous Story Intelligence (Story 4.2 Learnings)

**Pattern that worked:** Pure helpers (e.g., `next_ai_state`) extracted to module level, tested in isolation. **Apply to 4.3:** The death-attribution mapping (`ProjectileHitPlayer → DeathCause::EnemyFire`) is a one-liner inside `apply_player_damage`; no pure-helper extraction needed. If Stories 5.x add asteroid-collision damage with attribution, that future story can extract a `cause_for_damage_source` helper.

**Borrow-checker pitfall:** `Single<&Transform>` + `Query<&mut Transform>` overlap caused 4.2 a compile error; the fix was `Without<PlayerShip>` filter on the enemy query. **Apply to 4.3:** `Single<&mut Health, With<PlayerShip>>` (in `apply_player_damage`) is a single mutable borrow — no overlap with any other query in the system. No `Without<>` filters needed because PlayerShip is the only target.

**Bundle-arity 15 cap (Bevy 0.18):** 4.2's enemy spawn tuple needed nested grouping after exceeding 15 components. **Apply to 4.3:** PlayerShip currently has 13 components in the spawn tuple (line 117-130 of `flight/mod.rs`). Adding 2 (`Health` + `CollisionEventsEnabled`) brings it to 15 — at the edge but still under the cap. Verify at compile time; if Bevy 0.18 starts complaining, group into nested tuples per the 4.2 enemy-spawn precedent.

**Cold-start tuning fallback:** 4.2 confirmed `tuning_assets.get(...).cloned().unwrap_or_default()` is the correct pattern — `default_player_hull_max() -> 3` makes the fallback path return `Health { current: 3, max: 3 }`. **Apply to 4.3:** spawn_player_ship already uses this pattern at lines 97-101; the new `Health { current: tuning.player_hull_max, max: tuning.player_hull_max }` slots in directly after `tuning` is resolved.

**4.2 review patches:** the `look_at` patch (degenerate `forward.y` collinear-with-`Vec3::Y` case) is now in `enemy_ai.rs:159-163`. **Apply to 4.3:** the player ship's `Transform::look_at` is NOT used in 4.3 — orientation lives on flight input + physics. The patch is unrelated to 4.3.

### Cross-Story Dependencies

**Depends on (must be done before 4.3):**
- 3.5 (PlayerShip spawn) — extended in AC #2.
- 3.10 (asteroid damage routing) — pattern mirrored for player.
- 3.11 (HUD baseline) — `HudPlaceholder { field: HudField::Hull }` slot consumed by AC #10.
- 4.1 (Enemy entity) — `Enemy` component used by 4.2's projectile chain.
- 4.2 (Enemy AI) — `EnemyProjectile` marker, `Health` component, `GameLayer::Enemy`, `apply_enemy_damage` pattern all already present.

**Blocks (cannot proceed without 4.3):**
- 4.9 (PostRun summary screen) — consumes `RunResult` resource directly.
- 4.7 (Title screen full FR36) — Retry path from PostRun → Arena requires PostRun state to exist; this story drives the first Arena → PostRun transition.

**Independent (can proceed in parallel post-4.3):**
- 4.4 (Weapon archetypes) — orthogonal to hull damage.
- 4.5 (SemanticAccent retro-tint) — visual-layer only, orthogonal.
- 4.6 (PersistencePlugin) — settings persistence, orthogonal.
- 4.8 (Settings menu), 4.10 (Release workflow) — orthogonal.

### Source Tree Components Touched (per architecture.md:547-606)

```
src/
├── arena/
│   └── mod.rs               # M: OnTransition Arena → PostRun cleanup
├── combat/
│   ├── damage.rs            # M: DeathCause/RunResult/RunStartedAt/ProjectileHitPlayer/HullDepleted + 4 systems + 3 tests
│   └── mod.rs               # M: CheckDeath set; system + event registrations
├── flight/
│   └── mod.rs               # M: PlayerShip spawn-tuple extension
├── tuning/
│   └── config.rs            # M: player_hull_max field + default fn + Default impl + tests extension
└── ui/
    ├── hud.rs               # M: update_hud_hull system
    └── mod.rs               # M: update_hud_hull registration

assets/
└── config/
    └── tuning.ron           # M: player_hull_max field
```

### Testing Standards Summary

**From CLAUDE-equivalent project conventions and observed pattern:**
- Pure-helper-first: extract testable functions before ECS-bound systems. AC #13 reflects this — the 3 new tests are construction-discipline + variant-distinctness checks, all callable without a Bevy world setup.
- No-Default-derive guard tests: every new component / resource / enum without a Default derive gets a `..._construction_is_explicit` test that round-trips a literal. Mirrors `health_construction_is_explicit`, `enemy_ship_construction_is_explicit`, `hud_placeholder_carries_specified_field`.
- Tuning round-trip extension: every new `TuningConfig` field grows assertions in 3 existing tests (default-matches-RON, RON-bytes-deserializes, legacy-schema-falls-back-to-default). Pattern from Stories 2.4, 3.6–3.10, 4.2.
- Tests live in `mod tests { ... }` at the bottom of each module — never in a separate `tests/` directory (project does not use Bevy integration tests yet).
- `cargo test` is run with full output captured, NOT just exit-code-tail-checked, per `feedback_full_build_output.md`.

### Known Risks / Watch-Outs

**Bevy 0.18 `Time<Virtual>` semantics.** `Time<Virtual>::elapsed_secs()` is monotonically-increasing and pause-aware (paused intervals do not advance it) per Bevy docs. **Verify at runtime** (AC #16(f)): pause for ~5 seconds, resume, die — `RunResult.run_duration_seconds` must NOT include the paused interval. If Bevy 0.18's behavior differs from expectation, `Time<Real>` would inflate the duration; a `Time<Fixed>` accumulator would also work but requires per-tick accumulation.

**Avian `CollisionEventsEnabled` symmetry.** Avian 0.6 fires `CollisionStart` if AT LEAST ONE participant has `CollisionEventsEnabled`. Enemy projectiles already have it (`enemy_ai.rs:234`); adding it to PlayerShip is defense-in-depth (and consistency with asteroid + enemy precedent). **Verify at runtime** by removing `CollisionEventsEnabled` from PlayerShip mid-debug; the player should still take damage from enemy projectiles (because the projectile-side opt-in is sufficient). If it does NOT take damage, AC #2's `CollisionEventsEnabled` is load-bearing — keep it.

**Bundle arity at 15.** Bevy 0.18's `Bundle` derive is implemented up to arity 15. PlayerShip post-4.3 has 15 components; the cockpit-camera `with_children` block stays a separate spawn. If a future story adds a 16th component to PlayerShip, the spawn tuple needs nested grouping (per the 4.2 enemy-spawn precedent). 4.3 itself is at the limit, not over.

**Single-tick multi-hit edge case.** If two enemy projectiles hit the player in the same FixedUpdate tick (rare but plausible at high projectile-density Story 4.4 future), `apply_player_damage` processes both: the second hit hits the `hp.current == 0` early-continue. `HullDepleted` fires once. **Behavior is correct** — no double-emission.

**`PauseLatch` deferred-work entry (line 184).** A user who Esc-pauses, Alt-Tabs away, and Alt-Tabs back is silently auto-resumed. **Out-of-scope for 4.3** but worth flagging: a player who pauses to think mid-combat and Alt-Tabs to a browser will return to find their ship took damage during the focus-resume gap. The deferred-work entry resolution path (introduce `PauseLatch { user_initiated: bool }`) is independent of 4.3 and should be picked up separately at first playtest feedback if it surfaces.

### Project Structure Notes

**Alignment with unified project structure:**
- `DeathCause` and `RunResult` live in `src/combat/damage.rs` for Story 4.3, against architecture.md:687's "FR16 permadeath → src/run/director.rs" guidance. Decision: defer the `src/run/` module until Epic 6 Story 6.1 introduces `RunPlugin`. Story 6.1 may relocate these types.
- `RunStartedAt` similarly lives in `src/combat/damage.rs` (paired with `RunResult`); same future-relocation note.
- HUD update lives in `src/ui/hud.rs` per architecture.md:592's `src/ui/hud.rs` mapping for screen-space HUD. NOT placed in `src/combat/` or `src/flight/` — UI mutations are UI concerns reading domain state.
- Cleanup registration lives in `src/arena/mod.rs` (existing Plugin) — NO new module added.
- No new file additions in 4.3 — the story is entirely additive to existing modules. (Story 4.2 added 2 new files: `enemy_ai.rs`, `health.rs`. Story 4.3 adds 0 new files.)

**Detected variances (intentional, with rationale):**
- `DeathCause` placement (combat vs run/) — see above. Marked with inline doc-comment for relocation forward-compat.
- `record_run_started_at` placed in `src/combat/damage.rs` rather than a hypothetical `src/run/` module. Same rationale.

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-4-enemies-alive-stop-ship-itchio-prototype.md#Story 4.3] — Acceptance criteria (epic 4 spec lines 77-110)
- [Source: _bmad-output/planning-artifacts/architecture.md#Combat-Damage-Pattern] — `CombatSystems::EvaluateHits/ApplyDamage/CheckDeath` chain (lines 512-527)
- [Source: _bmad-output/planning-artifacts/architecture.md#Naming-Conventions] — Past-tense Event names; PascalCase Resources (lines 322-326)
- [Source: _bmad-output/planning-artifacts/architecture.md#Plugin-Boundaries] — `CombatPlugin` publishes `HullDepleted` (line 648); `RunPlugin` consumes (line 651)
- [Source: _bmad-output/planning-artifacts/architecture.md#FR-Mapping] — FR15 / FR16 → `src/combat/components.rs` + `src/combat/damage.rs` (lines 686-687)
- [Source: _bmad-output/planning-artifacts/prd.md#FR16] — "When player Hull reaches zero, the run ends (permadeath)" (line 518)
- [Source: _bmad-output/planning-artifacts/prd.md#Design-Principle-4] — Death is feedback, not punishment — informs the `RunResult` shape and 4.9 PostRun framing (line 488)
- [Source: _bmad-output/implementation-artifacts/4-2-enemy-ai-state-machine-detect-pursue-attack.md] — Health component, EnemyProjectile marker, GameLayer::Enemy, dual-ordering CollisionStart pattern, cold-start tuning fallback
- [Source: _bmad-output/implementation-artifacts/4-1-enemy-entity-foundation-semanticaccent-enemy.md] — Enemy spawn-tuple discipline, ArenaEntity dual-marker
- [Source: _bmad-output/implementation-artifacts/3-11-hud-baseline-screen-space-placeholders.md] — HudPlaceholder slot, HudField::Hull, HUD_TEXT_COLOR styling
- [Source: _bmad-output/implementation-artifacts/3-10-projectile-asteroid-collision-damage.md] — `apply_damage` saturating-sub semantics, `detect_projectile_asteroid_hits` dual-ordering pattern
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:252] — Arena → PostRun cleanup branch (closed by AC #11)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:333] — Enemy projectile vs PlayerShip damage routing (closed by AC #5/#6)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:184] — PauseLatch deferred-work (out-of-scope, flagged in Risks)
- [Source: src/combat/health.rs:14-18] — `Health` component shape (no-Default discipline)
- [Source: src/combat/damage.rs:83-85] — `apply_damage` saturating-sub helper
- [Source: src/combat/damage.rs:91-120] — `detect_projectile_asteroid_hits` dual-ordering pattern (template for `detect_projectile_player_hits`)
- [Source: src/combat/damage.rs:160-182] — `apply_enemy_damage` body (template for `apply_player_damage`)
- [Source: src/combat/enemy_ai.rs:48-53] — `EnemyProjectile` marker (consumed by AC #5 filter)
- [Source: src/flight/mod.rs:88-140] — `spawn_player_ship` body (extended by AC #2)
- [Source: src/arena/mod.rs:23-58] — `cleanup_on_exit::<T>` helper + existing OnTransition registration pattern
- [Source: src/ui/hud.rs:31-50] — `HudField::Hull` + `HudPlaceholder` (consumed by AC #10)
- [Source: src/pause/mod.rs:122-131] — `pause_simulation_clocks` confirms `Time<Virtual>::pause()` is the paused-clock contract
- [Source: src/tuning/config.rs:37-46, :89-107] — Per-field `#[serde(default = "...")]` forward-compat pattern (template for `player_hull_max`)
- [Memory: feedback_full_build_output.md] — `cargo check` exit-0 + tail is NOT proof; grep for `warning:|error:` (governs AC #14)
- [Memory: feedback_compact_review_style.md] — Compact single-line review answers (governs review interaction style)

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Opus 4.7, 1M context) — bmad-dev-story workflow

### Debug Log References

- `cargo check` → `/tmp/story-4-3-check.log` (0 warning/error lines)
- `cargo build` → `/tmp/story-4-3-build.log` (0 warning/error lines)
- `cargo test` → `/tmp/story-4-3-test.log` (0 warning/error lines; `test result: ok. 63 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`)
- `cargo clippy --all-targets -- -D warnings` → `/tmp/story-4-3-clippy.log` (0 warning/error lines)
- `cargo fmt --all -- --check` → `/tmp/story-4-3-fmt.log` (0 warning/error lines, exit 0)
- `cargo build --release` → `/tmp/story-4-3-release.log` (0 warning/error lines)

### Completion Notes List

- ✅ **All 11 ACs implemented per spec** with two intentional deviations documented below.
- ✅ **63 tests passing** (60 baseline + 3 net new in `combat::damage::tests`: `death_cause_variants_are_distinct`, `run_result_construction_is_explicit`, `run_started_at_construction_is_explicit`).
- ✅ **All 6 cargo gates green** per `feedback_full_build_output.md` discipline (full-output capture + `grep -cE 'warning:|error:'` = 0 for all 6 logs).
- ✅ **Two deferred-work entries closed** (lines 252 PostRun-cleanup branch, 333 enemy-projectile-vs-player damage routing).
- ⚠️ **Deviation from AC #7 code-shape (clippy-driven):** AC #7 specifies `for event in depleted.read() { ...; return; }` for `check_player_death`. Clippy's `never_loop` lint (denied by AC #14's `-D warnings`) flags this. Per AC #14's "fix at root, do NOT `#[allow]` without reasoning", refactored to functionally-equivalent `if let Some(event) = depleted.read().next() { ...; }`. Single-event semantics preserved; multi-event drainage behavior is identical (both forms read only the first event; remaining events stay queued and the next-tick run-if guard suppresses re-entry post-PostRun). The AC's `is_empty()` early-return guard prohibition is also still respected.
- ⚠️ **Deviation from AC #15 file set (fmt-driven):** AC #15 specifies the file set should NOT touch `src/combat/components.rs` or `src/combat/enemy_ai.rs`. However `cargo fmt --all -- --check` (AC #14) found pre-existing fmt drift in those two files inherited from earlier stories: `components.rs:39` (trailing blank line in tests module — pre-Story-4.3) and `enemy_ai.rs:158` (the 4.2 review patch's single-line `let up = if forward.y.abs() > 1.0 - 1e-4 { Vec3::Z } else { Vec3::Y };` which rustfmt now wants multi-line). To satisfy AC #14, ran `cargo fmt --all` and accepted these fmt-only edits as a side-effect. These are non-semantic whitespace/layout changes only — no logic touched. **Both files appear in the final `git status` set.** Suggest the upcoming code-review verify these are pure-fmt diffs and either (a) accept the AC #15 deviation as a fmt-debt sweep, or (b) split the components.rs / enemy_ai.rs fmt-only diffs into a separate "chore: fmt sweep" commit.

- 🔍 **Bevy ECS "entity despawned" warnings on Arena → PostRun cleanup — diagnosed as PRE-EXISTING project-wide bug, NOT 4.3-induced.** Runtime smoke (Till, 2026-05-08) surfaced 4 `WARN bevy_ecs::error::handler: ... Entity despawned: ID NvX is invalid; generation now Y` lines per death. **Initial hypothesis (incorrect):** treated as orphaned-cockpit-camera issue from Story 3.5; attempted dual-mark fix on the child. **Re-investigation (correct):** the same warning class also appears on **every `Splash → MainMenu` transition** (one warning, ID `87v0`, ~2 seconds into every game launch — Story 1.7 / `splash.rs::cleanup_loading_entities`) — this is **project code that 4.3 never touches**, conclusively proving the bug is pre-existing. Root cause: Bevy 0.18's `EntityCommands::despawn()` is recursive (contrary to the 3.11 deferred-work entry's assumption); dual-marked descendants under dual-marked roots in `splash.rs:49`, `hud.rs:74-78/93-97/113-117/133-137` cause double-despawn warnings — the parent's recursive despawn invalidates children, then the cleanup-query loop reaches them and queues a now-stale despawn. The cockpit-camera dual-mark fix attempt was reverted (it would have ADDED the same warning class to PlayerShip→Camera, not removed an existing one). **No code change in 4.3 for this issue.** New deferred-work entry authored at end of `_bmad-output/implementation-artifacts/deferred-work.md` ("Observed during: 4-3-hull-component-permadeath-postrun-state dev-smoke (2026-05-08)") with full diagnosis + two resolution paths (a) project-wide remove-descendant-markers, (b) defensive-iterate with `queue_silenced`. Scope: dedicated hardening story (4.3.1 or Epic 10 Story 10.11). **Cosmetic only** — no crashes, no functional regression; user can safely live with log noise until the systemic fix lands.
- 🔄 **Task 10 (Runtime smoke) deferred to Till's manual verification** per AC #16 (interactive `cargo run` walkthrough — checks (a) HUD shows `HULL 3` on Arena entry, (b) decrement on hit, (c) PostRun transition on Hull=0, (d) RunResult queryable, (e) Arena cleanup complete, (f) pause does not advance run_duration_seconds, (g) player-projectiles do NOT self-damage).
- 📌 **PlayerShip spawn-tuple arity is now exactly 15** (was 13 + Health + CollisionEventsEnabled), at Bevy 0.18's `Bundle` derive cap. Future stories adding a 16th component will need nested-tuple grouping per the 4.2 enemy-spawn precedent.

### File List

**M (modified) — implementation:**
- `src/flight/mod.rs` — extend PlayerShip spawn-tuple with `Health` + `CollisionEventsEnabled`; add `Health` + `CollisionEventsEnabled` imports
- `src/combat/damage.rs` — add `DeathCause`, `RunResult`, `RunStartedAt`, `ProjectileHitPlayer`, `HullDepleted` types; add `detect_projectile_player_hits`, `apply_player_damage`, `check_player_death`, `record_run_started_at` systems; +3 tests; new imports (`EnemyProjectile`, `PlayerShip`, `GameState`)
- `src/combat/mod.rs` — add `CombatSystems::CheckDeath` variant; extend FixedUpdate set-graph chain; register 3 new FixedUpdate systems + 1 Setup-set system; register 2 new messages; extend imports
- `src/arena/mod.rs` — add `OnTransition { Arena → PostRun }` cleanup registration; update inline comment block to mark PostRun-branch as closed
- `src/ui/hud.rs` — add `update_hud_hull` system with `Changed<Health>` filter; add `Health` + `PlayerShip` imports; remove obsolete `#[allow(dead_code)]` on `HudPlaceholder.field`
- `src/ui/mod.rs` — register `update_hud_hull` on `Update` gated by Arena
- `src/tuning/config.rs` — add `player_hull_max: u32` field + `default_player_hull_max() -> u32 { 3 }` fn; extend `TuningConfig::default()`; extend 3 existing tests with +3 assertions; round-trip RON bytes extended with `player_hull_max: 5`
- `assets/config/tuning.ron` — add `player_hull_max: 3` field

**M (modified) — fmt-only sweep (AC #15 deviation, see Completion Notes):**
- `src/combat/components.rs` — fmt: remove pre-existing trailing blank line in tests module
- `src/combat/enemy_ai.rs` — fmt: pre-existing 4.2 look_at-patch's single-line `let up = if ... { ... } else { ... };` reformatted to multi-line per current rustfmt

**M (modified) — bookkeeping:**
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `4-3-...: ready-for-dev → in-progress → review`; `last_updated` bumped
- `_bmad-output/implementation-artifacts/deferred-work.md` — close entries 252 (PostRun branch) + 333 (enemy-projectile-vs-player damage)
- `_bmad-output/implementation-artifacts/4-3-hull-component-permadeath-postrun-state.md` — flip `Status: ready-for-dev → in-progress → review`; check off Tasks 1–9, 11 (Task 10 deferred to user); fill Dev Agent Record sections; add Change Log entry

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-05-06 | bmad-create-story (auto) | Initial story creation; status: ready-for-dev |
| 2026-05-08 | bmad-dev-story (Opus 4.7) | Implemented all 11 ACs; 63 tests passing; all 6 cargo gates green; 2 deferred-work entries closed; status → review. Deviations documented in Completion Notes (clippy-driven `if let` form, fmt-sweep AC #15 file-set deviation). |
| 2026-05-08 | bmad-dev-story (Opus 4.7) | Post-smoke fix attempt: dual-marked cockpit-camera child with `ArenaEntity` to silence Bevy "entity despawned" warning on Arena → PostRun cleanup. |
| 2026-05-08 | bmad-dev-story (Opus 4.7) | Reverted cockpit-camera dual-mark after re-investigation: warning is PRE-EXISTING (also appears on Splash → MainMenu, code 4.3 never touches) and rooted in project-wide cleanup pattern (Bevy 0.18's `despawn()` is recursive; dual-marked descendants cause double-despawn warnings). New deferred-work entry authored ("Observed during: 4-3 dev-smoke 2026-05-08") with full diagnosis + resolution paths. 4.3 leaves this systemic issue out of scope. Gates remain clean. |
| 2026-05-11 | bmad-code-review (Sonnet 4.6) | Code review complete: 0 patches, 0 decisions, 4 deferred (RunStartedAt stale anchor, player_hull_max=0, HULL 0 display gap, cleanup orphan risk). 15 findings dismissed as false positives or handled. Status → done. |
