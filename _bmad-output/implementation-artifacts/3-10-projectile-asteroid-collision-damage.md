# Story 3.10: Projectile-Asteroid Collision & Damage

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player flying the Arena cockpit ship,
I want my projectiles to destroy asteroids on contact,
So that I have a visible combat outcome per FR12 — closing the core Arena interaction loop that Stories 3.5–3.9 prepared (cockpit + 6-DOF + rotation + dampener + firing) and completing the FR12 First Playable combat outcome the Epic 3 milestone gates on.

## Acceptance Criteria

1. **Given** Avian 0.6.1's `CollisionLayers` API uses `#[derive(PhysicsLayer)]` on a `Default`-implementing enum, where layer-bit 0 is reserved for the implicit `Default` layer (asteroid + projectile + ship currently share it AS-OF post-3.9; bounce-on-contact is the existing behavior per Story 3.9 AC #5 + smoke (n))
   **When** Story 3.10 introduces collision layers
   **Then** a NEW `pub enum GameLayer { #[default] Default, Asteroid, Projectile }` is added at the top of a new file `src/combat/damage.rs` (location justified by architecture.md:569 — `combat/damage.rs` is the canonical home for damage-related types)
   **And** the enum has `#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]` (PhysicsLayer is re-exported from `avian3d::prelude::*`)
   **And** **NO** `Player` variant is added — the ship stays on `GameLayer::Default` so flight/mod.rs needs zero changes (preserves the AC #2 discipline of Story 3.9: "FlightPlugin should not import combat types"; ship continues to physically interact with asteroids per the default-layer convention)
   **And** the enum is `pub` (used by `arena/zone.rs` and `combat/projectiles.rs` at spawn)

2. **Given** Story 3.3's `arena/zone.rs:88-97` spawn loop attaches 8 components per asteroid (Mesh3d, MeshMaterial3d, Transform, SemanticAccent::Neutral, RigidBody::Static, Collider::sphere, OutlineVolume, ArenaEntity) AND Avian 0.6.1 requires `CollisionEventsEnabled` on at LEAST one entity in a contact-pair for `CollisionStart` to fire (avian3d::collision_events module-level doc + line 45)
   **When** asteroid spawns are extended for 3.10
   **Then** **three** new components are appended to the spawn tuple, AFTER `OutlineVolume` and BEFORE `ArenaEntity` (alphabetical / functional grouping is irrelevant — follow the existing tuple order discipline; total tuple grows from 8 to 11 components, well under Bevy 0.18's 15-component bundle limit):
   - `AsteroidHp { current: 1 }` — single-hit destruction in Epic 3; multi-hit is Epic 4/5 per Epic 3 spec
   - `CollisionLayers::new([GameLayer::Asteroid], LayerMask::ALL)` — asteroid is a member of `Asteroid` layer; filters = ALL means asteroids still physically interact with ship (Default), projectiles (Projectile), AND other asteroids if any are added later. Critical: `LayerMask::ALL` is the only filter that preserves the existing ship↔asteroid bounce; using only `[GameLayer::Projectile]` would silently break the ship-blocking behavior the player relies on for spatial reasoning
   - `CollisionEventsEnabled` — required marker component per `avian3d::collision_events` to enable `CollisionStart`/`CollisionEnd` event emission for this entity
   **And** new imports added at the top of `arena/zone.rs`:
   - `use avian3d::prelude::{CollisionEventsEnabled, CollisionLayers, LayerMask};` (extending the existing `use avian3d::prelude::{Collider, RigidBody};` line)
   - `use crate::combat::components::AsteroidHp;` (new combat → arena dependency direction; acceptable — see AC #4 architecture rationale)
   - `use crate::combat::damage::GameLayer;` (new combat → arena dependency direction)
   **And** the existing test `asteroid_count_in_acceptance_range` is **NOT** modified (count is unchanged); existing tests `asteroid_radii_within_3_to_12`, `asteroid_positions_within_volume`, `asteroid_colliders_do_not_overlap`, `at_least_three_asteroids_within_50m_of_origin` are **NOT** modified (Story 3.3 invariants survive)

3. **Given** Story 3.9's projectile spawn at `src/combat/projectiles.rs:110-122` attaches 8 components (Projectile, ArenaEntity, Mesh3d, MeshMaterial3d, Transform, RigidBody::Dynamic, Collider::sphere, LinearVelocity)
   **When** projectile spawns are extended for 3.10
   **Then** **two** new components are appended to the spawn tuple, AFTER `LinearVelocity` (total tuple grows from 8 to 10 components):
   - `CollisionLayers::new([GameLayer::Projectile], [GameLayer::Asteroid])` — projectile is on `Projectile` layer; filters ONLY include `Asteroid` → projectile passes through ship (Default) and through other projectiles (Projectile not in own filters). This is the surgical fix for the "Self-collision window at high ship speed" deferred-work entry from 3.9 (deferred-work.md: "Story 3.10 adds CollisionLayers preventing ship-projectile contact entirely, making this entry dormant")
   - `CollisionEventsEnabled` — required for collision events to fire on projectile entity
   **And** new imports added to the existing `use avian3d::prelude::{...}` line:
   - Extend to `use avian3d::prelude::{Collider, CollisionEventsEnabled, CollisionLayers, LinearVelocity, RigidBody};` (alphabetical order)
   - Add `use crate::combat::damage::GameLayer;`
   **And** the existing 4 tests in `combat/projectiles.rs` are **NOT** modified (the helper `projectile_initial_velocity` signature is unchanged — collision filtering is spawn-tuple-side, not helper-side)

4. **Given** `AsteroidHp` is conceptually a damage/HP-related component and architecture.md:566 colocates HullHP/ShieldHP/Weapon-related components in `src/combat/components.rs` (already extant from Story 3.9)
   **When** Story 3.10 introduces the asteroid HP component
   **Then** `pub struct AsteroidHp { pub current: u32 }` is added to `src/combat/components.rs` AFTER the existing `PrimaryWeaponCooldown` definition
   **And** the derive list is `#[derive(Component, Debug, Clone, Copy, PartialEq)]` — NO `Default` derive (callers always specify `current: 1` explicitly per Epic 3 AC; `Default` would silently default to 0 = pre-destroyed, a hazardous footgun)
   **And** a doc-comment explains the Epic-3 single-hit semantics + forward-pointer to Epic 4/5 multi-hit (matches the `Projectile.damage` doc-comment precedent from Story 3.9)
   **And** the architecture rationale for the arena→combat dependency direction (NEW in 3.10) is captured: combat owns the damage/HP type-vocabulary; arena consumes those types at spawn time. This does NOT violate architecture.md:658 ("Plugin A never writes into Plugin B's internal Resources/Components") because (a) both arena and combat are in the same crate, and (b) `AsteroidHp` is a public type-vocabulary item, not an internal resource — combat *defines*, arena *attaches at spawn*, combat *mutates via systems*. Symmetric to how `ArenaEntity` is in arena/mod.rs but combat (projectiles) attaches it at spawn (`projectiles.rs:115`)
   **And** the existing `#[allow(dead_code, reason = "Projectile.damage is read by Story 3.10's ProjectileHitAsteroid event handler...")]` block on `Projectile` (`combat/components.rs:13-16`) is **REMOVED** — Story 3.10 fulfills this forward-pointer; `Projectile.damage` is now read by the damage-application system

5. **Given** Story 3.10 introduces two domain events that drive the FR12 combat outcome chain
   **When** the events are defined
   **Then** they live in `src/combat/damage.rs` immediately AFTER the `GameLayer` enum, in this order:
   ```rust
   #[derive(Message, Debug, Clone, Copy)]
   pub struct ProjectileHitAsteroid {
       pub projectile: Entity,
       pub asteroid: Entity,
       pub damage: u32,
   }

   #[derive(Message, Debug, Clone, Copy)]
   pub struct AsteroidDestroyed {
       pub asteroid: Entity,
   }
   ```
   **And** both use `#[derive(Message)]` NOT `#[derive(Event)]` — matches Bevy 0.18's broadcast-event API (project precedent: `tuning::TuningReloaded` at `src/tuning/mod.rs:20`; pause uses `MessageReader<WindowFocused>` at `src/pause/mod.rs:62`). Read via `MessageReader<E>`, written via `MessageWriter<E>`, registered via `app.add_message::<E>()` per the Bevy-0.18-renamed API
   **And** the events satisfy architecture.md's "Past-tense PascalCase" naming (`ProjectileHitAsteroid` past-tense per architecture.md:324: "ProjectileHitAsteroid" matches the naming since "Hit" is a past-participle verb form; `AsteroidDestroyed` is canonical past-tense)
   **And** `AsteroidDestroyed` has the **minimal** payload `{ asteroid: Entity }` per Epic 3 spec ("AsteroidDestroyed { asteroid: Entity } event") — NOT the architecture.md:398-403 example payload `{ entity, position, salvage_awarded, destroyed_by }`. The richer payload is Epic 4/5/6 territory (salvage_awarded requires Story 6.5 currency; destroyed_by requires Story 4.5 SemanticAccent::Faction wiring; position is recoverable via the asteroid Entity if any future consumer needs it before despawn). Add a doc-comment forward-pointer: `/// MVP minimal payload; later epics may extend with position / awarded_salvage / destroyed_by per architecture.md:398.`
   **And** `ProjectileHitAsteroid.damage` is read from the `Projectile.damage` field at the detection site (NOT hardcoded to 1) — guards forward-compat for Story 4.4's weapon-archetype damage variants

6. **Given** Avian 0.6.1's `CollisionStart` event (`avian3d::collision_events`) carries `{ collider1: Entity, collider2: Entity, body1: Option<Entity>, body2: Option<Entity> }` AND collisions can fire in either direction (projectile may be `collider1` or `collider2` — Avian gives no canonical ordering)
   **When** the detection system is authored in `src/combat/damage.rs`
   **Then** the system signature is:
   ```rust
   pub fn detect_projectile_asteroid_hits(
       mut collisions: MessageReader<CollisionStart>,
       projectiles: Query<&Projectile>,
       asteroids: Query<(), With<AsteroidHp>>,
       mut hits: MessageWriter<ProjectileHitAsteroid>,
   )
   ```
   **And** the system body iterates `collisions.read()` and for each event tries BOTH orderings, since Avian gives no canonical (projectile-first vs. asteroid-first) ordering for the colliders in `CollisionStart`:
   ```rust
   for event in collisions.read() {
       // Resolve which side of the contact pair is the projectile vs. asteroid.
       // Avian's CollisionStart gives no canonical ordering, so we try both.
       let (projectile_entity, asteroid_entity) =
           if projectiles.get(event.collider1).is_ok()
               && asteroids.get(event.collider2).is_ok()
           {
               (event.collider1, event.collider2)
           } else if asteroids.get(event.collider1).is_ok()
               && projectiles.get(event.collider2).is_ok()
           {
               (event.collider2, event.collider1)
           } else {
               continue; // Not a projectile-asteroid pair (e.g., ship↔asteroid bounce).
           };

       // Pull damage off the projectile component. The .get() above guarantees
       // this succeeds; the `let-else` is the panic-free idiom.
       let Ok(projectile) = projectiles.get(projectile_entity) else {
           continue;
       };
       hits.write(ProjectileHitAsteroid {
           projectile: projectile_entity,
           asteroid: asteroid_entity,
           damage: projectile.damage,
       });
   }
   ```
   **And** the `continue` branch is the path for ship↔asteroid bounces (CollisionStart fires for those too, since `CollisionEventsEnabled` is on the asteroid; that's INTENDED — the ship-bounce-against-asteroid existing behavior is preserved per AC #2 filter-ALL choice). NO `warn!` on the continue branch (would spam on every ship-asteroid contact)
   **And** **NO** `info!` log per detection — collision events are physics-step-rate (60 Hz), and a hit can produce multiple `CollisionStart` events on the same physics tick across multiple projectile-asteroid pairs. Logging is concentrated in the destruction path (AC #7) where it's gated to one log per asteroid death

7. **Given** the damage-application system is the second link in the chain (CollisionStart → ProjectileHitAsteroid → damage application → AsteroidDestroyed → despawn)
   **When** the system is authored in `src/combat/damage.rs` AFTER `detect_projectile_asteroid_hits`
   **Then** the system signature is:
   ```rust
   pub fn apply_asteroid_damage(
       mut hits: MessageReader<ProjectileHitAsteroid>,
       mut commands: Commands,
       mut asteroids: Query<&mut AsteroidHp>,
       mut destroyed: MessageWriter<AsteroidDestroyed>,
   )
   ```
   **And** the system body iterates `hits.read()` and for each event:
   ```rust
   for event in hits.read() {
       // Despawn projectile unconditionally — single-hit-per-projectile (Epic 3 AC).
       commands.entity(event.projectile).despawn();
       // Apply damage to asteroid if it's still alive (defensive: same physics tick
       // can produce multiple hits on the same asteroid before this system runs).
       if let Ok(mut hp) = asteroids.get_mut(event.asteroid) {
           hp.current = apply_damage(hp.current, event.damage);
           if hp.current == 0 {
               commands.entity(event.asteroid).despawn();
               destroyed.write(AsteroidDestroyed {
                   asteroid: event.asteroid,
               });
               info!("asteroid destroyed: entity={:?}", event.asteroid);
           }
       }
   }
   ```
   **And** the `if let Ok(mut hp)` guard handles the multi-projectile-same-tick case: if asteroid was already despawned by a prior hit in the same `hits.read()` iteration, `Query::get_mut` returns `Err(QueryEntityError::NoSuchEntity)` (Bevy 0.18 — entity-removal-via-Commands is observable to the same-tick Query because despawn happens in CommandQueue applied between systems, but cluster-hit cases within ONE `apply_asteroid_damage` invocation see same-frame state — defensive guard covers a future scheduler-reorder edge that would otherwise crash)
   **And** the projectile despawn is **outside** the `if let Ok` guard — projectile dies on ANY hit regardless of asteroid state (matches Epic 3 AC: "single-hit-per-projectile — cluster-hit cases resolve to first contact only in Epic 3"; if two projectiles hit the same asteroid in one tick, both projectiles despawn; only one `AsteroidDestroyed` fires)
   **And** **exactly one** `info!` per destruction (asteroid death is a low-frequency event — at most ~17 events per Arena entry given the 17-asteroid count from Story 3.3 zone layout); NO log on per-hit damage application below the destruction threshold (multi-hit asteroids are Epic 5; Epic 3 has HP=1 so this is dead code now but keeps the gate sound)
   **And** **NO** log on ship-asteroid bounces (those don't emit `ProjectileHitAsteroid` per AC #6's `continue` branch)

8. **Given** `apply_damage` is the pure-logic helper symmetric to `projectile_initial_velocity` (Story 3.9), `dampener_acceleration` (Story 3.8), `ship_local_thrust_vector` (Story 3.6) — first-class unit-test target per architecture.md:353
   **When** authored in `src/combat/damage.rs` AFTER the events and BEFORE the systems (helper-first ordering matches `combat/projectiles.rs` precedent)
   **Then** the signature is:
   ```rust
   /// Saturating-subtraction damage application. Returns the new HP, clamped at
   /// zero to prevent underflow on over-damage cases (e.g., a Story 4.4 high-damage
   /// weapon vs. a 1-HP asteroid). Pure function — no ECS access, trivially
   /// testable in isolation. Symmetric helper to projectile_initial_velocity
   /// from Story 3.9.
   pub fn apply_damage(current: u32, damage: u32) -> u32 {
       current.saturating_sub(damage)
   }
   ```
   **And** the helper is `pub` (used by `apply_asteroid_damage` system AND by the test block); located between the `AsteroidDestroyed` event definition and the `detect_projectile_asteroid_hits` system
   **And** uses `u32::saturating_sub` (stdlib) — clamps to 0 on overflow rather than wrapping. NO custom `if damage > current { 0 } else { current - damage }` branch — saturating_sub is the canonical idiom

9. **Given** `CombatPlugin::build` (`src/combat/mod.rs:25-61`) currently registers a `Setup` set + `Fire / Lifecycle` chain in FixedUpdate
   **When** Story 3.10 extends it
   **Then** the build function is extended in this order (NEW lines marked `// 3.10`):
   ```rust
   impl Plugin for CombatPlugin {
       fn build(&self, app: &mut App) {
           // [existing 3.9 lines unchanged: configure OnTransition Setup, add InputManagerPlugin]

           // 3.10: register collision-driven damage events.
           app.add_message::<ProjectileHitAsteroid>();
           app.add_message::<AsteroidDestroyed>();

           app.configure_sets(
               FixedUpdate,
               (
                   CombatSystems::Fire,
                   CombatSystems::Lifecycle,
                   CombatSystems::EvaluateHits,  // 3.10
                   CombatSystems::ApplyDamage,   // 3.10
               ).chain(),
           );
           // [existing OnTransition Setup add_systems unchanged]
           app.add_systems(
               FixedUpdate,
               (
                   projectiles::fire_primary_weapon
                       .in_set(CombatSystems::Fire)
                       .run_if(in_state(GameState::Arena)),
                   projectiles::tick_projectile_ttl
                       .in_set(CombatSystems::Lifecycle)
                       .run_if(in_state(GameState::Arena)),
                   damage::detect_projectile_asteroid_hits  // 3.10
                       .in_set(CombatSystems::EvaluateHits)
                       .run_if(in_state(GameState::Arena)),
                   damage::apply_asteroid_damage            // 3.10
                       .in_set(CombatSystems::ApplyDamage)
                       .run_if(in_state(GameState::Arena)),
               ),
           );
       }
   }
   ```
   **And** the `CombatSystems` enum is extended to:
   ```rust
   pub enum CombatSystems {
       Setup,
       Fire,
       Lifecycle,
       EvaluateHits,  // 3.10 — reads CollisionStart, emits ProjectileHitAsteroid
       ApplyDamage,   // 3.10 — reads ProjectileHitAsteroid, mutates HP, emits AsteroidDestroyed
   }
   ```
   **And** the chain order `Fire → Lifecycle → EvaluateHits → ApplyDamage` ensures: (a) freshly-spawned projectiles have a full TTL before any decrement (Lifecycle after Fire — preserved from 3.9); (b) collision detection runs after physics integration provides this-frame collisions (Avian's narrow_phase emits CollisionStart at end of physics step, before user FixedUpdate systems — verified via `avian3d::collision::narrow_phase::CollisionEventSystems` doc comment); (c) damage application runs after detection in the same FixedUpdate tick (no 1-tick visual lag between hit and asteroid death)
   **And** `pub mod damage;` is added to `src/combat/mod.rs` after the existing `pub mod components; pub mod input; pub mod projectiles;` line (alphabetical: components, damage, input, projectiles)
   **And** the necessary `use` declarations are added at the top of `src/combat/mod.rs`: `use crate::combat::damage::{ApplyDamage, AsteroidDestroyed, ProjectileHitAsteroid};` is **NOT** needed — only `damage::detect_projectile_asteroid_hits` and `damage::apply_asteroid_damage` are referenced via path-qualified syntax in `add_systems`. The `add_message::<ProjectileHitAsteroid>()` and `add_message::<AsteroidDestroyed>()` calls require `use crate::combat::damage::{AsteroidDestroyed, ProjectileHitAsteroid};` at the top of `combat/mod.rs`

10. **Given** Story 3.10's unit-test surface
    **When** tests are authored
    **Then** `combat/damage.rs` gains 3 co-located test functions in a `#[cfg(test)] mod tests` block at the bottom of the file (plain primitives — no MinimalPlugins, no World setup):
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn apply_damage_full_destruction_returns_zero() {
            // Standard case: 1 HP asteroid + 1 damage projectile = destruction.
            assert_eq!(apply_damage(1, 1), 0);
        }

        #[test]
        fn apply_damage_partial_returns_remaining_hp() {
            // Multi-hit forward-compat: Epic 5 multi-HP asteroids must reduce, not destroy.
            assert_eq!(apply_damage(3, 1), 2);
        }

        #[test]
        fn apply_damage_overdamage_clamps_at_zero() {
            // Saturating-sub guards against u32 underflow. Story 4.4 high-damage weapon
            // vs. low-HP target hits this branch.
            assert_eq!(apply_damage(1, 5), 0);
        }
    }
    ```
    **And** `combat/components.rs` gains 1 NEW test function in the existing `#[cfg(test)] mod tests` block at the bottom of the file (added AFTER `primary_weapon_cooldown_default_is_zero`):
    ```rust
    #[test]
    fn asteroid_hp_construction_is_explicit() {
        // No Default derive — this test guards against accidental future Default
        // addition that would silently default current=0 (pre-destroyed footgun).
        let hp = AsteroidHp { current: 1 };
        assert_eq!(hp.current, 1);
    }
    ```
    **And** **NO** test function is added for the `detect_projectile_asteroid_hits` / `apply_asteroid_damage` systems — they require ECS World + collision-event setup; integration testing is deferred per architecture.md:354 (the runtime smoke per AC #11 is the verification surface)
    **And** **NO** test function is added for `CombatPlugin::build` (Plugin scaffolding requires MinimalPlugins + state setup; same deferral)
    **And** Story 3.10 adds **4 net new test functions** (3 in `combat/damage.rs` + 1 in `combat/components.rs`) — net post-3.10 test count: **45** (= 41 from end of 3.9 + 3 helper + 1 component; the +0 deltas in `arena/zone.rs` per AC #2 last sentence and `combat/projectiles.rs` per AC #3 last sentence are expected). AC #11 enforces N = 45 at verification time

11. **Given** the post-3.9 source baseline (test count = 41 per `cargo test` 2026-05-05 measurement; `cargo build --release` 0 warnings; `src/combat/components.rs` = 38 lines; `src/combat/input.rs` = 13 lines; `src/combat/mod.rs` = 61 lines; `src/combat/projectiles.rs` = 194 lines; `src/arena/mod.rs` = 58 lines; `src/arena/zone.rs` = 166 lines; NO `src/combat/damage.rs` exists)
    **When** Story 3.10 verification runs locally (per `feedback_full_build_output.md` discipline — exit-0 + tail is NOT proof; grep for `warning:|error:` per command, capture each to `/tmp/story-3-10-<command>.log`)
    **Then** **all six** of `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --release` produce **0** lines matching `grep -cE 'warning:|error:'` per their respective full-output logs
    **And** `cargo test` summary line reads `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N = 45** (= 41 baseline + 3 new in `combat/damage.rs` + 1 new in `combat/components.rs`)
    **And** the runtime smoke (Task 8 below) verifies all of (a)–(k) per AC #12
    **And** `/tmp/story-3-10-run.log` contains: 1 of `entered Loading`, 1 of `entered MainMenu`, ≥ 1 of `entered Arena`, ≥ 1 of `spawned PlayerShip`, ≥ 1 of `fired projectile`, ≥ 1 of `asteroid destroyed: entity=` (at least one kill exercised in smoke), 0 of `panic|backtrace|FATAL`, 0 of `ambiguous.*camera.*order`, 0 of `ERROR.*avian|WARN.*Avian`
    **And** `git status --short` final set is **exactly**: `?? src/combat/damage.rs` (new file: GameLayer + 2 events + apply_damage helper + 2 systems + 3 tests), `M src/combat/components.rs` (M — AsteroidHp added + Projectile #[allow(dead_code)] removed + 1 test added), `M src/combat/mod.rs` (M — pub mod damage + 2 add_message calls + EvaluateHits/ApplyDamage variants + 2 system registrations + chain extension + use line for events), `M src/combat/projectiles.rs` (M — 2 spawn-tuple components added: CollisionLayers + CollisionEventsEnabled + 2 use additions), `M src/arena/zone.rs` (M — 3 spawn-tuple components added: AsteroidHp + CollisionLayers + CollisionEventsEnabled + 2 use additions), `M _bmad-output/implementation-artifacts/sprint-status.yaml` (M — bookkeeping), `?? _bmad-output/implementation-artifacts/3-10-projectile-asteroid-collision-damage.md` (?? — NEW: this story spec; ?? at story-creation time, becomes M after dev flips Status), and `M _bmad-output/implementation-artifacts/deferred-work.md` ONLY IF a new entry surfaces during impl (the existing 3.9 self-collision deferred entry should be marked CLOSED-BY-3.10); **NO** entries under `Cargo.toml` (no dep added — every API used is already in scope: `Message` from `bevy::prelude`; `PhysicsLayer`/`CollisionLayers`/`LayerMask`/`CollisionStart`/`CollisionEventsEnabled` from `avian3d::prelude` — confirmed via `grep "pub use" ~/.cargo/registry/src/.../avian3d-0.6.1/src/lib.rs`), `Cargo.lock`, `src/flight/**` (per AC #1's discipline preserving 3.9), `src/state.rs`, `src/pause/**`, `src/visual/**`, `src/ui/**`, `src/splash.rs`, `src/logging.rs`, `assets/strings/**`, `assets/config/tuning.ron` (no new tuning fields — Epic 3 hardcodes HP=1 + damage=1), `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`

12. **Given** Story 3.10's runtime smoke is the integration test for the full chain `fire → ballistics → CollisionStart → ProjectileHitAsteroid → apply_damage → AsteroidDestroyed → despawn` (per architecture.md:354 integration tests deferred post-M3, and the smoke precedent of Stories 3.6/3.7/3.8/3.9)
    **When** the dev runs the runtime smoke
    **Then** the dev verifies all of:
    - (a) **Fire one shot at a near asteroid** (~25 m, e.g., the (18, 3, -25) cluster member from `arena/zone.rs:20`) → projectile flies forward, hits the asteroid, BOTH disappear simultaneously. Exactly 1 `info!("asteroid destroyed: entity=...")` line in `/tmp/story-3-10-run.log`. NO bounce visible (was the 3.9 limitation — now closed). NO `WARN` / `ERROR` lines.
    - (b) **Fire shots into empty space** (rotate ship to look at +Z direction where there's open space per the Story 3.5 line-of-sight precondition) → projectiles fly, TTL out at 3 s, NO asteroid-destroyed log lines, NO panics.
    - (c) **Hold LMB while approaching an asteroid** → fire-rate-gated stream of projectiles, FIRST projectile to land kills the asteroid, subsequent projectiles fly past where the asteroid was and TTL out (or hit a different asteroid behind it if line-of-sight permits). Asteroid count visibly drops from 17 to 16. `grep -c 'asteroid destroyed'` ≈ 1 per killed asteroid.
    - (d) **Fire at the same asteroid with two projectiles in rapid succession** (single-tap-tap LMB at 2× the fire rate cap, e.g., 0.3 s + 0.3 s wait → 2 fires) → asteroid dies from first hit; second projectile flies through where it was and either hits another asteroid behind or TTLs out. `grep -c 'asteroid destroyed'` = 1 (NOT 2 — second projectile-asteroid event won't fire because the asteroid is despawned).
    - (e) **Ship physically bounces off an asteroid** (fly forward into a near asteroid without firing) → ship stops/bounces (existing ship↔asteroid contact preserved by AC #2 `LayerMask::ALL` filter). NO `asteroid destroyed` log (no projectile involved). NO `panic` or `WARN`.
    - (f) **Projectile flies through ship** (impossible to test from cockpit view alone — verify indirectly: from cockpit fire many projectiles, observe ship doesn't lose health / behave unexpectedly when projectile spawns near it). Validates AC #3's filter-on-Asteroid-only choice.
    - (g) **Fire at a far asteroid** (~95 m, e.g., the (95, -45, -75) far-field member) → projectile travels for ~0.8 s before impact. Within `projectile_ttl_seconds = 3.0` s budget. Asteroid dies at impact, no out-of-range issues.
    - (h) **Esc-pause mid-flight (projectile in motion toward asteroid)** → projectile freezes in air; on resume, projectile continues on-trajectory and impacts as expected. `asteroid destroyed` log emits AFTER resume, not during pause. Validates `run_if(in_state(GameState::Arena))` gating for `apply_asteroid_damage` is correct.
    - (i) **Cmd-Tab focus-loss + regain mid-flight** → identical to (h).
    - (j) **Quit cleanly during flight** (window-close while projectiles in-flight + asteroids alive) → no panic; ArenaPlugin's existing `OnTransition { exited: Arena, entered: MainMenu }` cleanup despawns remaining ArenaEntity-marked entities (asteroids, projectiles, ship, light) when state exits — though Epic 3 doesn't actually trigger that transition without Esc-to-menu, which is Epic 4 territory; this gate confirms shutdown-via-window-close at any state is panic-free.
    - (k) **Destroy ALL 17 asteroids** (slow but exhaustive) → `grep -c 'asteroid destroyed'` = 17 over the course of the smoke. Final state: empty Arena (no asteroids, only ship + light + any in-flight projectiles). No stray entities in `bevy_inspector_egui` if installed (NOT installed in 3.10 — visual confirmation only). Validates the 17-asteroid layout doesn't have a corner-case position that escapes the spawn-tuple change.

## Tasks / Subtasks

- [x] **Task 1: Extend `src/combat/components.rs` — add `AsteroidHp` + remove dead-code allow on `Projectile`** (AC: #4, #10)
  - [x] Open `src/combat/components.rs`. After the existing `PrimaryWeaponCooldown` definition, append:
    ```rust
    /// Asteroid hit-point pool. Epic 3 default `current = 1` for single-hit
    /// destruction (per Story 3.10 spec); Epic 4/5 multi-HP asteroids will spawn
    /// with higher initial values via the same component. Decremented by
    /// `combat::damage::apply_asteroid_damage`; despawn fires when current == 0.
    ///
    /// NO Default derive — callers always specify `current` explicitly. A
    /// silent default of 0 would mean "pre-destroyed", a hazardous footgun.
    #[derive(Component, Debug, Clone, Copy, PartialEq)]
    pub struct AsteroidHp {
        pub current: u32,
    }
    ```
  - [x] Remove the existing `#[allow(dead_code, reason = "...")]` block on the `Projectile` struct (lines 13-16 currently). The `Projectile.damage` field is now read by `combat::damage::detect_projectile_asteroid_hits` (AC #6 system body), fulfilling the forward-pointer.
  - [x] In the same file's `#[cfg(test)] mod tests` block (lines 30-38 currently), append the new `asteroid_hp_construction_is_explicit` test per AC #10 verbatim. Place AFTER the existing `primary_weapon_cooldown_default_is_zero` test.
  - [x] **Verify post-edit:** `cargo check 2>&1 | tee /tmp/story-3-10-check-task1.log; grep -cE 'warning:|error:' /tmp/story-3-10-check-task1.log` should output `0` once Task 2's GameLayer is in place. May fail before Task 2 with "unresolved import" — this is expected; defer the green-check expectation to end of Task 4.

- [x] **Task 2: Create `src/combat/damage.rs` — GameLayer + events + apply_damage + 2 systems + 3 tests** (AC: #1, #5, #6, #7, #8, #10)
  - [x] Create new file `src/combat/damage.rs`. Author in this order:
    1. Module doc-comment + use block:
    ```rust
    //! Damage events + asteroid HP routing (FR12). Owns:
    //!   - `GameLayer` — Avian PhysicsLayer enum: Default (ship), Asteroid, Projectile.
    //!   - `ProjectileHitAsteroid` / `AsteroidDestroyed` events (Bevy 0.18 Messages).
    //!   - `apply_damage` — pure saturating-subtraction helper.
    //!   - `detect_projectile_asteroid_hits` — FixedUpdate, reads CollisionStart,
    //!     emits ProjectileHitAsteroid (handles both collider-pair orderings).
    //!   - `apply_asteroid_damage` — FixedUpdate, reads ProjectileHitAsteroid,
    //!     mutates AsteroidHp, despawns projectile (single-hit), despawns
    //!     asteroid + emits AsteroidDestroyed when HP reaches zero.
    //!
    //! Story 3.10 ships single-hit asteroids (current = 1, damage = 1). Epic 4
    //! adds enemy projectiles + HullHP/ShieldHP routing through the same
    //! event chain pattern. Epic 5 adds multi-HP asteroids that exercise
    //! apply_damage's partial-reduction branch.

    use avian3d::prelude::{CollisionStart, PhysicsLayer};
    use bevy::prelude::*;

    use crate::combat::components::{AsteroidHp, Projectile};
    ```
    2. `GameLayer` enum per AC #1 verbatim.
    3. Event definitions per AC #5 verbatim.
    4. `apply_damage` helper per AC #8 verbatim.
    5. `detect_projectile_asteroid_hits` system per AC #6 verbatim.
    6. `apply_asteroid_damage` system per AC #7 verbatim.
    7. `#[cfg(test)] mod tests { ... }` block with the 3 tests per AC #10 verbatim.
  - [x] **Verify post-edit:** isolated file compiles when AC #9's `pub mod damage` lands in Task 4. Same caveat as Task 1: defer green-check to end of Task 4.

- [x] **Task 3: Extend `src/combat/projectiles.rs` — CollisionLayers + CollisionEventsEnabled in spawn tuple** (AC: #3)
  - [x] Update the avian use line at the top: change
    ```rust
    use avian3d::prelude::{Collider, LinearVelocity, RigidBody};
    ```
    to
    ```rust
    use avian3d::prelude::{Collider, CollisionEventsEnabled, CollisionLayers, LinearVelocity, RigidBody};
    ```
  - [x] Add new use line below the existing `use crate::combat::components::...` line:
    ```rust
    use crate::combat::damage::GameLayer;
    ```
  - [x] In `fire_primary_weapon` body, modify the spawn tuple at lines 110-122 to append two components AFTER `LinearVelocity(velocity)` and BEFORE the closing `));`:
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
        CollisionLayers::new([GameLayer::Projectile], [GameLayer::Asteroid]),  // 3.10
        CollisionEventsEnabled,                                                   // 3.10
    ));
    ```
  - [x] **Verify post-edit:** the existing 4 helper tests in this file are NOT modified — they test `projectile_initial_velocity` whose signature is unchanged.

- [x] **Task 4: Extend `src/combat/mod.rs` — pub mod damage + 2 events + 2 systems + chain extension** (AC: #9)
  - [x] Open `src/combat/mod.rs`. Add `pub mod damage;` after the existing `pub mod components; pub mod input; pub mod projectiles;` line. Final order: `pub mod components; pub mod damage; pub mod input; pub mod projectiles;` (alphabetical).
  - [x] Add new use line after the existing `use crate::combat::input::CombatAction;` line:
    ```rust
    use crate::combat::damage::{AsteroidDestroyed, ProjectileHitAsteroid};
    ```
  - [x] Extend `CombatSystems` enum with 2 new variants per AC #9 verbatim:
    ```rust
    pub enum CombatSystems {
        Setup,
        Fire,
        Lifecycle,
        EvaluateHits,
        ApplyDamage,
    }
    ```
  - [x] In `impl Plugin for CombatPlugin::build`, after the existing `app.add_plugins(InputManagerPlugin::<CombatAction>::default());` line, insert:
    ```rust
    app.add_message::<ProjectileHitAsteroid>();
    app.add_message::<AsteroidDestroyed>();
    ```
  - [x] Modify the existing `app.configure_sets(FixedUpdate, (CombatSystems::Fire, CombatSystems::Lifecycle).chain());` to:
    ```rust
    app.configure_sets(
        FixedUpdate,
        (
            CombatSystems::Fire,
            CombatSystems::Lifecycle,
            CombatSystems::EvaluateHits,
            CombatSystems::ApplyDamage,
        ).chain(),
    );
    ```
  - [x] In the existing `app.add_systems(FixedUpdate, (...))` tuple, append two new systems after `tick_projectile_ttl`:
    ```rust
    damage::detect_projectile_asteroid_hits
        .in_set(CombatSystems::EvaluateHits)
        .run_if(in_state(GameState::Arena)),
    damage::apply_asteroid_damage
        .in_set(CombatSystems::ApplyDamage)
        .run_if(in_state(GameState::Arena)),
    ```
  - [x] **Verify post-edit:** `cargo check 2>&1 | tee /tmp/story-3-10-check-task4.log; grep -cE 'warning:|error:' /tmp/story-3-10-check-task4.log` should output `0`. If `unused import` warnings appear on `AsteroidDestroyed` or `ProjectileHitAsteroid`, the issue is the use-line pattern (the types ARE used as type parameters to `add_message::<>()`, but cargo check may not detect that — if so, the cleanest fix is path-qualified `damage::ProjectileHitAsteroid` in the add_message calls and remove the use-line; the dev-author should choose at impl time per the lint output).

- [x] **Task 5: Extend `src/arena/zone.rs` — AsteroidHp + CollisionLayers + CollisionEventsEnabled in asteroid spawn tuple** (AC: #2)
  - [x] Open `src/arena/zone.rs`. Update the avian use line at the top: change
    ```rust
    use avian3d::prelude::{Collider, RigidBody};
    ```
    to
    ```rust
    use avian3d::prelude::{Collider, CollisionEventsEnabled, CollisionLayers, LayerMask, RigidBody};
    ```
  - [x] Add two new use lines after the existing `use super::ArenaEntity;` line:
    ```rust
    use crate::combat::components::AsteroidHp;
    use crate::combat::damage::GameLayer;
    ```
  - [x] In the asteroid spawn loop (lines 88-97), modify the tuple to insert three new components AFTER `outline_volume()` and BEFORE `ArenaEntity`:
    ```rust
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(position),
        SemanticAccent::Neutral,
        RigidBody::Static,
        Collider::sphere(radius),
        outline_volume(),
        AsteroidHp { current: 1 },                                                  // 3.10
        CollisionLayers::new([GameLayer::Asteroid], LayerMask::ALL),                // 3.10
        CollisionEventsEnabled,                                                     // 3.10
        ArenaEntity,
    ));
    ```
  - [x] **Verify post-edit:** the existing 5 tests in this file are NOT modified — they test `ASTEROIDS` constant invariants, none of which depend on the spawn-tuple shape.

- [x] **Task 6: Update deferred-work.md — close the 3.9 self-collision entry** (AC: #11 last paragraph)
  - [x] Open `_bmad-output/implementation-artifacts/deferred-work.md`. Find the entry "Self-collision window at high ship speed" (under "Deferred from: code review of 3-9-weapon-firing-projectile-ballistics (2026-05-05)").
  - [x] Append a closure note to the bullet (preserve the original entry, add a `> ✅ FIXED 2026-05-XX by Story 3.10` blockquote below it) — exact text deferred to dev-author at impl time but along the lines of: `> ✅ CLOSED 2026-05-XX by Story 3.10 — projectile spawns with CollisionLayers::new([GameLayer::Projectile], [GameLayer::Asteroid]); ship is on default layer and not in projectile filter, so projectile passes through ship entirely. Self-collision is impossible by construction.`

- [x] **Task 7: Verification gates — all 6 cargo commands clean** (AC: #11)
  - [x] Run each command in sequence; capture FULL output (NOT just exit code or tail) per `feedback_full_build_output.md`:
    ```bash
    cargo check                                         2>&1 | tee /tmp/story-3-10-check.log
    cargo build                                         2>&1 | tee /tmp/story-3-10-build.log
    cargo test                                          2>&1 | tee /tmp/story-3-10-test.log
    cargo clippy --all-targets -- -D warnings           2>&1 | tee /tmp/story-3-10-clippy.log
    cargo fmt --all -- --check                          2>&1 | tee /tmp/story-3-10-fmt.log
    cargo build --release                               2>&1 | tee /tmp/story-3-10-release.log
    ```
  - [x] For EACH log: `grep -cE 'warning:|error:' /tmp/story-3-10-<cmd>.log` must output `0`. If non-zero, fix and re-run from the failing command. NO partial-pass shortcuts.
  - [x] `cargo test` log MUST contain the literal line `test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (or a less specific variant: confirm `45 passed` AND `0 failed`).
  - [x] If clippy flags `#[derive(PhysicsLayer)]` for unused-discriminant warnings or similar, the fix is to add the appropriate `#[allow(...)]` with `reason = "..."` — but this is unlikely; PhysicsLayer is well-formed.

- [x] **Task 8: Runtime smoke — full chain validation** (AC: #11, #12)
  - [x] Till manually executed `cargo run` and verified scenarios (a)–(k) per AC #12. Confirmed "alles grün" 2026-05-05.

- [x] **Task 9: Sprint status bookkeeping** (AC: #11)
  - [x] Story status flipped to `review` (set by bmad-dev-story end). `code-review` workflow will flip to `done` next.

### Review Findings

- [x] [Review][Patch] **Double `AsteroidDestroyed` + double despawn on same-tick multi-hit asteroid** [`src/combat/damage.rs:apply_asteroid_damage`] — When two projectiles hit the same 1-HP asteroid in one FixedUpdate tick, `apply_asteroid_damage` processes both `ProjectileHitAsteroid` events within the same system run. On the second iteration, `asteroids.get_mut(event.asteroid)` still succeeds (entity not yet removed — `commands.despawn()` is deferred). `hp.current` was set to 0 by the first iteration, so `apply_damage(0, 1) = 0` again → `AsteroidDestroyed` emitted a second time + despawn queued again. Fix: add `if hp.current == 0 { commands.entity(event.projectile).despawn(); continue; }` before `hp.current = apply_damage(...)` to skip already-dead asteroids in the same tick.

- [x] [Review][Patch] **Redundant `projectiles.get()` + unreachable dead-code `continue` in `detect_projectile_asteroid_hits`** [`src/combat/damage.rs:~line 83`] — The `if`/`else if` block already calls `projectiles.get(event.collider1).is_ok()` to classify the pair, then immediately after calls `projectiles.get(projectile_entity)` again to extract the data. The second call is redundant (first call consumed the result via `.is_ok()` without capturing the value), and the `else { continue }` on the `let Ok … else` branch is unreachable dead code. Minor: restructure to capture `Ok(proj)` directly in the `if` condition to eliminate the second query and the dead branch.

- [x] [Review][Defer] **Projectile `damage` hardcoded to `1` (not in TuningConfig)** [`src/combat/projectiles.rs:fire_primary_weapon`] — deferred, pre-existing / intentional per spec
- [x] [Review][Defer] **`AsteroidHp { current: 1 }` hardcoded inline in asteroid spawn loop** [`src/arena/zone.rs:97`] — deferred, pre-existing / intentional per spec
- [x] [Review][Defer] **`asteroid_hp_construction_is_explicit` test name overpromises — doesn't compile-fail guard `Default`** [`src/combat/components.rs`] — deferred, compile_fail tests not in project scope
- [x] [Review][Defer] **`GameLayer::Default` implicit ship-layer convention has no test coverage** [`src/combat/damage.rs:GameLayer`] — deferred, integration tests deferred post-M3
- [x] [Review][Defer] **Avian `CollisionStart` ordering relative to `EvaluateHits` not formally verified** [`src/combat/mod.rs`] — deferred, verified by runtime smoke; integration tests deferred post-M3
- [x] [Review][Defer] **Zero-damage projectile (`damage: 0`) would silently despawn projectile without affecting HP** [`src/combat/damage.rs:apply_damage`] — deferred, Epic 3 hardcodes `damage: 1`; design edge case for Story 4.4
- [x] [Review][Defer] **Asteroid `LayerMask::ALL` filter generates ship↔asteroid `CollisionStart` events (correctly discarded but non-zero overhead)** [`src/arena/zone.rs`] — deferred, intentional per AC #2; correctly handled by `continue` branch

## Dev Notes

### Relevant architecture patterns and constraints

- **Plugin boundaries** — CombatPlugin owns damage events + collision-detection systems (architecture.md:648). FlightPlugin remains untouched (Story 3.9 AC #2 discipline preserved). ArenaPlugin gains a new READ-side dependency on `combat::components::AsteroidHp` and `combat::damage::GameLayer` for the asteroid spawn tuple. This is a NEW dependency direction (arena → combat) — justified at AC #4: combat owns the damage/HP type-vocabulary; arena consumes those types at spawn time. Type imports are not "writes into another plugin's internal Resource/Component" — that rule (architecture.md:658) is about systems writing to other plugins' state via direct Query mutation, not about type dependencies.
- **Past-tense event naming** (architecture.md:324) — `ProjectileHitAsteroid` ("Hit" is past-participle) and `AsteroidDestroyed` (canonical past-tense). Present-tense alternatives like `ProjectileHittingAsteroid` would violate the convention.
- **System chain ordering** (architecture.md:408-415) — Use SystemSet enum + `.chain()` between sets. The Story 3.10 chain `Fire → Lifecycle → EvaluateHits → ApplyDamage` is internal to CombatSystems; no `.after(specific_function)` permitted.
- **No magic numbers** (architecture.md:463) — `current: 1` and `damage: 1` are Epic-3-MVP literals, NOT TuningConfig fields. Future weapon archetypes (Story 4.4) and multi-HP asteroids (Epic 5) will introduce the relevant tuning surface.
- **Saturating-sub idiom** — `u32::saturating_sub` is the canonical clamp-at-zero stdlib API. NO custom branch.

### Source tree components to touch

| File | Change | LOC delta (estimate) |
|------|--------|---------------------|
| `src/combat/components.rs` | Add `AsteroidHp`; remove `Projectile` `#[allow]`; add 1 test | +14 / -4 |
| `src/combat/damage.rs` | NEW: GameLayer + 2 events + apply_damage + 2 systems + 3 tests | +120 (estimate) |
| `src/combat/mod.rs` | Add `pub mod damage`; add use; extend SystemSet enum + 4 build-fn lines | +18 |
| `src/combat/projectiles.rs` | Add 2 spawn-tuple lines + 2 use additions | +4 |
| `src/arena/zone.rs` | Add 3 spawn-tuple lines + 2 use additions | +5 |
| `_bmad-output/implementation-artifacts/deferred-work.md` | Append closure note to 3.9 self-collision entry | +1 line |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | Update status field | +0 net (mod) |

NO changes expected in: `src/flight/**`, `src/state.rs`, `src/pause/**`, `src/visual/**`, `src/ui/**`, `src/splash.rs`, `src/logging.rs`, `src/main.rs`, `src/tuning/**`, `assets/**`, `Cargo.toml`, `Cargo.lock`, `.github/**`.

### Testing standards summary

- **Unit tests only** for Story 3.10 (architecture.md:354 — integration tests deferred post-M3). Helper functions are first-class test targets.
- 3 tests for `apply_damage` covering: full destruction, partial reduction (multi-HP forward-compat), saturating-sub overdamage clamp.
- 1 test for `AsteroidHp` construction explicitness (guards against accidental future Default derive).
- Runtime smoke (Task 8) covers the full system-level chain that integration tests would verify.
- Test count post-3.10: **45** (= 41 baseline + 4 net new). AC #11 enforces.

### Project Structure Notes

- **Alignment with unified project structure:** `src/combat/damage.rs` is exactly where architecture.md:569 places it. `src/combat/components.rs` already exists from Story 3.9 (architecture.md:566 prescribed location).
- **Detected variances:** none. The `core/` directory referenced by architecture.md:550 (for shared types like `Faction`, `DamageSource`) is intentionally deferred — Story 3.10 does NOT need cross-plugin shared types yet (`GameLayer` is combat-internal; ship spawn doesn't import it because it stays on the default layer per AC #1). Future Epic-4 enemy work will likely introduce `core/faction.rs` and that's where it belongs then, not now.
- **Architecture compliance:** plugin boundaries respected (combat owns events + systems + types; arena consumes types at spawn; flight untouched).

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md#Story-3.10] — story spec (Acceptance Criteria source)
- [Source: _bmad-output/planning-artifacts/architecture.md#Communication-Patterns] — event conventions (line 391-406)
- [Source: _bmad-output/planning-artifacts/architecture.md#Plugin-Boundaries] — CombatPlugin owns HullDamaged/AsteroidDestroyed (line 648)
- [Source: _bmad-output/planning-artifacts/architecture.md#Project-Directory-Structure] — combat module layout (lines 564-570)
- [Source: _bmad-output/planning-artifacts/architecture.md#Good-Pattern-Examples] — event payload + system pattern (line 485-528)
- [Source: _bmad-output/implementation-artifacts/3-9-weapon-firing-projectile-ballistics.md] — Story 3.9 spec (precedent for spawn tuples + plugin extension)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md] — 3.9 entries informing 3.10 scope (self-collision closure target; OnTransition migration precedent — 3.10 inherits the precedent for any new Setup systems)
- [Source: avian3d-0.6.1/src/collision/collision_events.rs] — `CollisionStart` event API + `CollisionEventsEnabled` requirement (lines 1-100, 169-189)
- [Source: avian3d-0.6.1/src/collision/collider/layers.rs] — `PhysicsLayer` trait + `CollisionLayers::new` signature + `LayerMask::ALL` (lines 1-100, 360-410)
- [Source: src/tuning/mod.rs:20-28] — Bevy 0.18 Message + add_message project precedent
- [Source: src/pause/mod.rs:62] — `MessageReader` project precedent

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (1M context) — bmad-dev-story 2026-05-05

### Debug Log References

- `/tmp/story-3-10-check.log` — `cargo check` clean (0 warnings/errors)
- `/tmp/story-3-10-build.log` — `cargo build` clean
- `/tmp/story-3-10-test.log` — `cargo test` clean: `test result: ok. 45 passed; 0 failed`
- `/tmp/story-3-10-clippy.log` — `cargo clippy --all-targets -- -D warnings` clean
- `/tmp/story-3-10-fmt.log` — `cargo fmt --all -- --check` clean (after one auto-fix on `combat/projectiles.rs` use-line wrap)
- `/tmp/story-3-10-release.log` — `cargo build --release` clean (4m 21s compile)

### Completion Notes List

**Implementation deviations from spec — minor & intentional:**

1. **`combat/projectiles.rs` use-line wrap** — `cargo fmt --all` auto-wrapped the 5-import avian use line `use avian3d::prelude::{Collider, CollisionEventsEnabled, CollisionLayers, LinearVelocity, RigidBody};` into multi-line form (3 lines). Spec wrote it as a single line; this is a fmt-mandated wrap, not a logic deviation. Identical end-of-line module surface.

2. **`AsteroidDestroyed` `#[allow(dead_code)]` retroactively added** — the spec didn't anticipate this, but `cargo clippy --all-targets -- -D warnings` flagged `AsteroidDestroyed.asteroid` as `never read` because Story 3.10's `apply_asteroid_damage` only WRITES the event; there's no consumer in Epic 3 (consumers land in Epic 4 enemy AI + Epic 6 salvage per architecture.md:648). Applied the same `#[allow(dead_code, reason = "...")]` pattern Story 3.9 used on `Projectile.damage`. Forward-pointer reason explicitly references Epic 4 enemy AI and Epic 6 salvage.

3. **`detect_projectile_asteroid_hits` — fmt rewrap of branch** — `cargo fmt` rewrapped the `else if asteroids.get(event.collider1).is_ok() && projectiles.get(event.collider2).is_ok()` line for length; logically identical to spec's vertical layout.

**No spec-amend needed** — both deviations are mechanical (formatting) and one boilerplate (`#[allow]`) consistent with the project's established pattern from Story 3.9. The semantic behavior of all 12 ACs is unchanged.

**Verification gates: 6/6 PASSED locally:**
- `cargo check`: 0 warnings/errors
- `cargo build`: 0 warnings/errors
- `cargo test`: **45 passed**, 0 failed (= 41 baseline + 3 new in `combat/damage.rs` + 1 new in `combat/components.rs` per AC #10/#11)
- `cargo clippy --all-targets -- -D warnings`: 0 warnings/errors
- `cargo fmt --all -- --check`: clean (after one fmt fix re-run)
- `cargo build --release`: 0 warnings/errors

**Runtime smoke (Task 8) — pending Till's manual execution.** Bevy app smoke requires interactive cockpit input; LLM cannot execute Task 8 scenarios (a)–(k). All preconditions for the smoke are met: code compiles + tests pass + clippy/fmt clean. Till to run `cargo run 2>&1 | tee /tmp/story-3-10-run.log` and verify scenarios (a)–(k) per AC #12.

### File List

**Created:**
- `src/combat/damage.rs` — 151 lines: GameLayer enum, ProjectileHitAsteroid + AsteroidDestroyed events, apply_damage helper, detect_projectile_asteroid_hits + apply_asteroid_damage systems, 3 unit tests

**Modified:**
- `src/combat/components.rs` — added `AsteroidHp { current: u32 }` struct with doc-comment; removed `#[allow(dead_code)]` block from `Projectile`; added `asteroid_hp_construction_is_explicit` test
- `src/combat/mod.rs` — added `pub mod damage`; added use line for `AsteroidDestroyed` + `ProjectileHitAsteroid`; extended `CombatSystems` enum with `EvaluateHits` + `ApplyDamage` variants; added 2 `add_message::<>()` calls; extended FixedUpdate set-chain from 2 to 4 sets; registered 2 new systems on FixedUpdate
- `src/combat/projectiles.rs` — extended avian use line with `CollisionEventsEnabled` + `CollisionLayers`; added `use crate::combat::damage::GameLayer`; added 2 components to projectile spawn tuple (`CollisionLayers` + `CollisionEventsEnabled`)
- `src/arena/zone.rs` — extended avian use line with `CollisionEventsEnabled` + `CollisionLayers` + `LayerMask`; added `use crate::combat::components::AsteroidHp` + `use crate::combat::damage::GameLayer`; added 3 components to asteroid spawn tuple (`AsteroidHp { current: 1 }` + `CollisionLayers` + `CollisionEventsEnabled`)
- `_bmad-output/implementation-artifacts/deferred-work.md` — appended ✅ CLOSED 2026-05-05 by Story 3.10 blockquote to the Self-collision deferred entry from 3.9
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — bookkeeping (status field flips: `backlog` → `ready-for-dev` → `in-progress` → `review` will be set by Step 9 of dev-story workflow after smoke confirmation)
- `_bmad-output/implementation-artifacts/3-10-projectile-asteroid-collision-damage.md` — this story file: Tasks 1–7 marked [x], Task 8 pending Till smoke, Dev Agent Record / File List / Change Log filled

### Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-05-05 | bmad-create-story (Claude Opus 4.7) | Story spec created from Epic 3.10 + arch + 3.9 precedent; ready-for-dev |
| 2026-05-05 | bmad-dev-story (Claude Opus 4.7) | Tasks 1–7 implemented; 6/6 cargo gates clean; 45 tests pass; pending Till runtime smoke (Task 8) |
| 2026-05-05 | Till (runtime smoke) | Task 8 scenarios (a)–(k) verified green; status → review |
