# Story 4.1: Enemy Entity Foundation + SemanticAccent::Enemy

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player flying the Arena cockpit ship,
I want a visible enemy ship present in the Arena when I spawn,
So that I have a new target type to recognize before AI and combat dynamics come online in Story 4.2 — opening Epic 4 / M3 Itch.io stop-and-ship per FR14.

## Acceptance Criteria

1. **(File scaffold)** A new submodule `src/combat/enemy.rs` is authored. `pub mod enemy;` is added to `src/combat/mod.rs` after the existing `pub mod damage;` line (alphabetical order: `components, damage, enemy, input, projectiles`).
   **And** **NO** new plugin (e.g. `EnemyPlugin`) is introduced — `CombatPlugin::build` is extended with the spawn registration, matching the established "one Plugin per top-level feature module, submodules contribute systems/types" pattern (precedent: `combat::damage` from Story 3.10, `combat::projectiles` from Story 3.9, `flight::physics` / `flight::input` from Stories 3.6–3.8). The epic-4 spec wording at `epics/epic-4-enemies-alive-stop-ship-itchio-prototype.md:7` ("integrated with `CombatPlugin`") is satisfied by the extension; the literal phrasing does not require a separate plugin struct.

2. **(Component vocabulary)** The following types are defined at the top of `src/combat/enemy.rs` (after `use` block, after constants):
   ```rust
   /// Empty marker tagging entities hostile to the player. Queried by Story 4.2's
   /// AI transitions and Story 4.3's damage-routing for `EnemyProjectile` collisions.
   #[derive(Component, Debug, Clone, Copy)]
   pub struct Enemy;

   /// Typed enemy archetype. Story 4.1 ships a single variant; Story 4.2 may extend
   /// when AI variants land, Story 4.4 weapon archetypes may further extend.
   #[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
   pub enum EnemyShip {
       Standard,
   }
   ```
   **And** **NO** `Default` derive on either type (callers always specify the variant explicitly at spawn — same footgun-prevention rationale as `AsteroidHp` from Story 3.10 and `HudField` / `HudPlaceholder` from Story 3.11).

3. **(Spawn registration — `OnTransition`, NOT `OnEnter`)** `pub fn spawn_enemy_ship(...)` is registered on `OnTransition { exited: GameState::MainMenu, entered: GameState::Arena }` and placed `.in_set(CombatSystems::Setup)`.
   **And** **NO** `OnEnter(GameState::Arena)` registration is used despite the epic-4 spec at `epics/epic-4-enemies-alive-stop-ship-itchio-prototype.md:18` literally reading "OnEnter(GameState::Arena) runs after Story 3.3's zone spawn". The OnTransition pattern is the project's pause-roundtrip-preservation contract per Story 3.9's source-of-truth fix (`deferred-work.md:220`); deviating from it would re-introduce the despawn-respawn-on-pause regression that Story 3.9 explicitly closed. Same deviation as every Epic-3 story from 3.5 onward (PlayerShip, asteroids, projectiles, HUD).
   **And** the existing transitive ordering chain `ArenaSystems::Setup → FlightSystems::Setup → CombatSystems::Setup` (configured by `ArenaPlugin`, `FlightPlugin`, `CombatPlugin` respectively at `src/arena/mod.rs:25-31`, `src/flight/mod.rs:45-51`, `src/combat/mod.rs:34-40`) ensures `spawn_enemy_ship` runs after `spawn_arena_zone` (Story 3.3) and after `spawn_player_ship` (Story 3.5), satisfying epic-4 AC line 13's "after Story 3.3's zone spawn" requirement.
   **And** `spawn_enemy_ship` co-exists in `CombatSystems::Setup` with the existing `projectiles::attach_combat_to_player_ship` — the two systems have no inter-dependency (one mutates the existing PlayerShip, the other spawns a new entity); register them as a tuple `(projectiles::attach_combat_to_player_ship, enemy::spawn_enemy_ship).in_set(CombatSystems::Setup)` for compactness, OR as separate `.add_systems(...)` calls — both are idiomatic.

4. **(Spawn-position constant — within ~80 m of PlayerShip)** A `const ENEMY_SPAWN_POSITION: Vec3 = Vec3::new(0.0, 0.0, -60.0);` is defined at module top.
   **And** distance from origin (PlayerShip spawn point per `src/flight/mod.rs:121`) is `60.0` m — comfortably within the epic-4 AC "~80 m" budget.
   **And** the position lies along the `-Z` corridor (cockpit Camera3d's default forward direction per Bevy convention), giving the player **immediate visual contact on first Arena entry** with no rotation required — the foundational "have a new target type to recognize" requirement of the user story.
   **And** clearance against the existing 17-asteroid hand-picked layout at `src/arena/zone.rs:20-41` is verified: closest asteroids are at `(-8, 10, -42)` (radius 5.0, distance ≈ 22.1 m, clearance ≈ 15.1 m) and `(18, 3, -25)` (radius 6.5, distance ≈ 39.5 m, clearance ≈ 26.5 m) — both clearances exceed the enemy's `Capsule3d` total bounding extent (8 m × 4 m × 4 m), so no overlap.

5. **(Mesh + collider — distinct from PlayerShip)** The enemy uses a `Capsule3d::new(2.0, 4.0)` mesh (radius 2 m, capped length 4 m → total bounding extent 8 m × 4 m × 4 m), which is **visually distinct** from the PlayerShip's `Cuboid::new(4.0, 2.0, 6.0)` (boxy) at `src/flight/mod.rs:109`. The capsule's elongated rounded silhouette is unambiguously "not the player's ship".
   **And** the Avian collider is `Collider::capsule(2.0, 4.0)` — exact mesh-radius / mesh-length match per the Story 3.3 mesh==collider precedent for collision-trustworthiness.
   **And** **NO** `CollisionLayers` component is added (enemy stays on the implicit `GameLayer::Default` layer per `src/combat/damage.rs:25-31`). Player projectile filters from Story 3.10 explicitly target `[GameLayer::Asteroid]` ONLY (`src/combat/projectiles.rs:125`), so player projectiles will pass through the enemy in Story 4.1 — this is **CORRECT**: damage routing for projectile-vs-enemy is Story 4.2's scope (`Health` component + `EnemyProjectile` marker land in 4.2). Introducing `GameLayer::Enemy` now would scope-creep into 4.2.
   **And** ship-vs-enemy bounce collisions WILL be emitted (both on `Default` layer); the existing `detect_projectile_asteroid_hits` system at `src/combat/damage.rs:67-98` correctly filters out non-projectile/non-asteroid pairs via the `continue` branch (line 85), so no spurious damage events fire.

6. **(Material + accent — vermillion via `SemanticAccent::Enemy`)** The enemy's `ToonMaterial` is built with `tint: color_for(SemanticAccent::Enemy).into()`, where `color_for(SemanticAccent::Enemy) = Color::srgb_u8(0xD5, 0x5E, 0x00)` (vermillion red-orange) per `src/visual/palette.rs:21`.
   **And** the enemy entity ALSO carries a `SemanticAccent::Enemy` Component (in addition to the material tint). This **establishes the per-entity accent-tagging convention** for new entities introduced after Story 3.5; Story 4.5 will retroactively apply the same convention to existing entities (asteroids → `SemanticAccent::Salvage`, PlayerShip + projectiles → `SemanticAccent::PlayerOwned`) per `deferred-work.md:206`.
   **NOTE — color-vocabulary discrepancy in the epic spec:** epic-4 spec at `epics/epic-4-enemies-alive-stop-ship-itchio-prototype.md:17` references "salvage (yellow) asteroids", but `palette.rs:23-24` defines `Salvage = #009E73 bluish-green` and `Hazard = #F0E442 yellow`. The two warm colors were apparently confused during planning. For Story 4.1's runtime this is moot: asteroids are still `SemanticAccent::Neutral` (grey, per `src/arena/zone.rs:94`); Story 4.5 retunes asteroids to `Salvage` (bluish-green, **NOT** yellow). The enemy's vermillion is trivially distinct from grey for 4.1 and from bluish-green for 4.5+. Spec amendment deferred per the Story 3.9 precedent (`deferred-work.md:262` — "Spec-Amend kostet mehr als es nützt; deferred-work.md reicht").

7. **(OutlineVolume — tuning-driven)** The enemy spawn includes an `OutlineVolume { visible: true, width: tuning.outline_width, colour: Color::srgba(...) }` constructed from the `TuningConfig` resource, mirroring the per-entity outline pattern at `src/flight/mod.rs:103-107` (PlayerShip) and `src/arena/zone.rs:57-64` (asteroid closure).
   **And** cold-start safety: `tuning_assets.get(tuning_handle.0.id())` MAY return `None` on a hypothetical first-frame race; the system falls back to `TuningConfig::default()` with a `warn!("tuning.ron not loaded at Enemy spawn; using TuningConfig defaults")` (mirrors `spawn_player_ship` at `src/flight/mod.rs:97-101` and `spawn_arena_zone` at `src/arena/zone.rs:50-55`).

8. **(RigidBody — Dynamic + zero velocities)** The enemy carries `RigidBody::Dynamic` per epic-4 AC line 15, with `LinearVelocity(Vec3::ZERO)` and `AngularVelocity(Vec3::ZERO)` for stationary first-impression behavior.
   **And** project-wide gravity is zero (`src/main.rs:43`), so a `Dynamic` body remains stationary unless an `ExternalForce` is applied. Story 4.2's AI applies forces to give the enemy deliberate motion; Story 4.1 establishes the static silhouette.
   **And** ship-vs-enemy collision (low probability, but possible if the player rams the enemy at the 60 m position) will displace the `Dynamic` enemy via Avian's collision response — **acceptable** for Story 4.1 because no system in 4.1 reads the enemy's coordinate; Story 4.2's AI re-orients toward the player on every transition tick anyway.

9. **(Markers — dual `Enemy + ArenaEntity`, NOT a 3rd cleanup-generic consumer)** The enemy carries dual markers: `Enemy` (component-vocabulary tag for combat queries from Story 4.2 / 4.3) AND `ArenaEntity` (state-scoped cleanup marker per `src/arena/mod.rs:21`).
   **And** cleanup on `Arena → MainMenu` is **transitive** via the existing `cleanup_on_exit::<ArenaEntity>` registered by `ArenaPlugin::build` at `src/arena/mod.rs:44-50` — Story 4.1 does **NOT** introduce a 3rd direct consumer of `cleanup_on_exit::<T>` (matches PlayerShip + ArenaEntity at `src/flight/mod.rs:117-118` and HUD nodes per Story 3.11 AC #2).
   **And** the `deferred-work.md:186-188` entry "Generic-cleanup home re-evaluation now triggered (3rd consumer pending)" is updated by Task 3 below with a `📝 UPDATED 2026-05-XX by Story 4.1` blockquote noting that Story 4.1, like Story 3.11, took the dual-marker path; the entry stays open for whichever future story (likely Story 4.9 post-run-summary or 9.2 photo-mode) introduces a genuine 3rd direct cleanup consumer.

10. **(Spawn-tuple body)** The exact spawn structure inside `spawn_enemy_ship`:
    ```rust
    commands.spawn((
        Enemy,
        EnemyShip::Standard,
        SemanticAccent::Enemy,
        ArenaEntity,
        Mesh3d(enemy_mesh),
        MeshMaterial3d(enemy_material),
        Transform::from_translation(ENEMY_SPAWN_POSITION),
        outline,
        RigidBody::Dynamic,
        Collider::capsule(ENEMY_CAPSULE_RADIUS, ENEMY_CAPSULE_LENGTH),
        LinearVelocity(Vec3::ZERO),
        AngularVelocity(Vec3::ZERO),
    ));
    info!(
        "spawned EnemyShip at {:?} ({}m from origin)",
        ENEMY_SPAWN_POSITION,
        ENEMY_SPAWN_POSITION.length()
    );
    ```
    **And** the system signature is exactly:
    ```rust
    pub fn spawn_enemy_ship(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ToonMaterial>>,
        tuning_assets: Res<Assets<TuningConfig>>,
        tuning_handle: Res<TuningHandle>,
    )
    ```
    Mirrors `spawn_player_ship` at `src/flight/mod.rs:88-94`.

11. **(Imports — minimal but complete)** The `use` block at the top of `src/combat/enemy.rs` (immediately after the `//!` module doc-comment) is exactly:
    ```rust
    use avian3d::prelude::{AngularVelocity, Collider, LinearVelocity, RigidBody};
    use bevy::prelude::*;
    use bevy_mod_outline::OutlineVolume;

    use crate::arena::ArenaEntity;
    use crate::tuning::TuningHandle;
    use crate::tuning::config::TuningConfig;
    use crate::visual::palette::{SemanticAccent, color_for};
    use crate::visual::toon_material::ToonMaterial;
    ```
    Mirrors the `src/flight/mod.rs:8-22` import pattern verbatim (sans the leafwing/cursor-grab imports irrelevant for the enemy).
    **And** **NO** `crate::combat::components::*` import (Story 4.1 does NOT reference `Projectile`, `AsteroidHp`, or `PrimaryWeaponCooldown` — the enemy entity has no combat components yet; Story 4.2 introduces `Health`).
    **And** **NO** `crate::combat::damage::GameLayer` import (Story 4.1 puts the enemy on the implicit `Default` layer per AC #5 — no `CollisionLayers` component).

12. **(Tests — 4 unit tests)** A `#[cfg(test)] mod tests` block at the bottom of `src/combat/enemy.rs` contains exactly these 4 tests:
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn enemy_ship_construction_is_explicit() {
            // No Default derive — guards against accidental future Default addition
            // that would silently default to a particular variant. Round-trip explicit
            // construction. Mirrors AsteroidHp / HudPlaceholder explicit-construction tests.
            let archetype = EnemyShip::Standard;
            assert_eq!(archetype, EnemyShip::Standard);
        }

        #[test]
        fn enemy_spawn_position_within_80m_of_origin() {
            // Epic-4 AC budget — keeps the enemy in immediate sensory range of the
            // PlayerShip spawn point. Tightening the budget below 80 m is fine; loosening
            // requires a story amendment.
            assert!(
                ENEMY_SPAWN_POSITION.length() <= 80.0,
                "ENEMY_SPAWN_POSITION distance from origin = {} exceeds AC budget of 80 m",
                ENEMY_SPAWN_POSITION.length()
            );
        }

        #[test]
        fn enemy_spawn_position_in_front_of_cockpit() {
            // Cockpit Camera3d default forward is -Z (Bevy convention). For first-impression
            // visual contact, the enemy must be on the negative-Z side. A future redesign
            // that picks a behind-or-beside position must amend the user-story rationale.
            assert!(
                ENEMY_SPAWN_POSITION.z < 0.0,
                "ENEMY_SPAWN_POSITION.z = {} is not in cockpit-forward direction (-Z)",
                ENEMY_SPAWN_POSITION.z
            );
        }

        #[test]
        fn enemy_spawn_position_clears_close_asteroids() {
            // Hardcoded coordinates of the two closest asteroids per src/arena/zone.rs:23,25
            // (kept inline rather than imported to preserve module separation; this test
            // validates the chosen enemy position against the known layout, NOT the layout
            // itself — that is src/arena/zone.rs::tests::asteroid_colliders_do_not_overlap's job).
            // If the arena layout changes, both tests need to be re-evaluated.
            let close_asteroids: &[(Vec3, f32)] = &[
                (Vec3::new(-8.0, 10.0, -42.0), 5.0),
                (Vec3::new(18.0, 3.0, -25.0), 6.5),
            ];
            let enemy_max_extent = ENEMY_CAPSULE_RADIUS + ENEMY_CAPSULE_LENGTH * 0.5;
            for &(asteroid_pos, asteroid_radius) in close_asteroids {
                let distance = (ENEMY_SPAWN_POSITION - asteroid_pos).length();
                let min_separation = asteroid_radius + enemy_max_extent;
                assert!(
                    distance > min_separation,
                    "ENEMY_SPAWN_POSITION at {:?} overlaps asteroid at {:?} (r={}); distance={}, min_separation={}",
                    ENEMY_SPAWN_POSITION,
                    asteroid_pos,
                    asteroid_radius,
                    distance,
                    min_separation
                );
            }
        }
    }
    ```
    **And** **NO** test for `spawn_enemy_ship` system (requires MinimalPlugins + state setup; deferred per architecture.md:354 — same deferral as Stories 3.5–3.11).
    **And** **NO** test for plugin registration in `CombatPlugin::build` (same deferral).
    **And** Story 4.1 adds **4 net new test functions** in `src/combat/enemy.rs`. Net post-4.1 test count: **53** (= 49 from end of 3.11 + 4 new). AC #13 enforces.

13. **(Verification gates — all 6 cargo commands clean)** Per `feedback_full_build_output.md` discipline, exit-0 + tail is NOT proof; full output is captured per command and grep'd for `warning:|error:`.
    **Then** **all six** of the following produce **0** lines matching `grep -cE 'warning:|error:'` per their respective full-output logs:
    ```bash
    cargo check                                         2>&1 | tee /tmp/story-4-1-check.log
    cargo build                                         2>&1 | tee /tmp/story-4-1-build.log
    cargo test                                          2>&1 | tee /tmp/story-4-1-test.log
    cargo clippy --all-targets -- -D warnings           2>&1 | tee /tmp/story-4-1-clippy.log
    cargo fmt --all -- --check                          2>&1 | tee /tmp/story-4-1-fmt.log
    cargo build --release                               2>&1 | tee /tmp/story-4-1-release.log
    ```
    **And** `cargo test` summary line reads `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N = 53** (= 49 baseline + 4 new in `src/combat/enemy.rs`).

14. **(File set — `git status --short` final)** Final set is **exactly**:
    - `?? src/combat/enemy.rs` (new file: types + constants + `spawn_enemy_ship` + 4 tests)
    - `M src/combat/mod.rs` (M — `pub mod enemy;` + spawn registration in `CombatPlugin::build`)
    - `M _bmad-output/implementation-artifacts/sprint-status.yaml` (M — bookkeeping: `epic-4: backlog → in-progress`, `4-1-...: backlog → ready-for-dev → in-progress → review → done`, `last_updated`)
    - `M _bmad-output/implementation-artifacts/deferred-work.md` (M — appended UPDATED note to "Generic-cleanup home" entry per AC #9 / Task 3)
    - `?? _bmad-output/implementation-artifacts/4-1-enemy-entity-foundation-semanticaccent-enemy.md` at story-creation time (becomes M after dev flips Status / fills Dev Agent Record / Change Log)
    
    **NO** entries under: `Cargo.toml` / `Cargo.lock` (no dep added — `Capsule3d`, `RigidBody`, `Collider::capsule`, `LinearVelocity`, `AngularVelocity` all already in scope via Bevy 0.18 prelude + Avian 0.6 prelude per existing imports), `src/arena/**`, `src/flight/**`, `src/ui/**`, `src/state.rs`, `src/pause/**`, `src/visual/**`, `src/splash.rs`, `src/logging.rs`, `src/main.rs`, `src/tuning/**`, `assets/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`.

15. **(Runtime smoke — full enemy-presence chain)** After Task 4 confirms cargo gates green, Till manually executes `cargo run 2>&1 | tee /tmp/story-4-1-run.log` and verifies:
    - **(a) Enemy visible at Arena entry** — within ~2 s of pressing Enter on MainMenu (after splash), the enemy capsule is visible directly ahead of the cockpit at distance ~60 m, with vermillion (red-orange) tint clearly distinct from the grey asteroids and the grey PlayerShip. The capsule silhouette is unmistakably distinct from the boxy PlayerShip.
    - **(b) Spawn-log line presence** — `grep -c 'spawned EnemyShip' /tmp/story-4-1-run.log` outputs **1** after a single Arena entry.
    - **(c) Enemy stationary** — observed for ~10 s without player input, the enemy does not drift, rotate, or accelerate. Position holds at `(0, 0, -60)`.
    - **(d) Player projectiles pass through enemy** — fire LMB while pointed at the enemy. Projectiles visibly pass through the capsule with no impact effect, no log line, no enemy despawn (correct: damage routing is Story 4.2). Ship-vs-enemy bounce collision (if the player rams the enemy by flying forward through it) emits a Bevy collision contact but no panic.
    - **(e) Pause round-trip preserves enemy** — press Esc to pause, press Esc to resume. Enemy capsule remains in place with no respawn. `grep -c 'spawned EnemyShip' /tmp/story-4-1-run.log` still outputs **1**.
    - **(f) Focus-loss round-trip preserves enemy** — Cmd-Tab away (macOS) / Alt-Tab (Win/Linux) and back. Enemy intact. Spawn-log count unchanged.
    - **(g) Quit cleanly during Arena** — close window while in Arena. No panic; no `ERROR` / `panic` / `backtrace` / `FATAL` lines in the smoke log. `grep -cE 'panic|backtrace|FATAL' /tmp/story-4-1-run.log` outputs **0**.

## Tasks / Subtasks

- [x] **Task 1: Author `src/combat/enemy.rs` — types, constants, spawn_enemy_ship, 4 unit tests** (AC: #1, #2, #4, #5, #6, #7, #8, #10, #11, #12)
  - [x] Create new file `src/combat/enemy.rs`. Author top-down in this order:
    1. Module doc-comment:
       ```rust
       //! Enemy entity foundation (FR14) — defines `Enemy` marker + `EnemyShip` archetype
       //! and spawns one stationary placeholder enemy in the Arena. AI / health / damage
       //! routing land in Stories 4.2 / 4.3. SemanticAccent::Enemy tagging here also
       //! prefigures Story 4.5's full retroactive accent sweep on existing entities.
       ```
    2. Use block per AC #11 verbatim.
    3. Constants:
       ```rust
       const ENEMY_SPAWN_POSITION: Vec3 = Vec3::new(0.0, 0.0, -60.0);
       const ENEMY_CAPSULE_RADIUS: f32 = 2.0;
       const ENEMY_CAPSULE_LENGTH: f32 = 4.0;
       ```
    4. Type definitions per AC #2 verbatim (`Enemy`, `EnemyShip`).
    5. `spawn_enemy_ship` system per AC #10 / signature in AC #10 (with cold-start TuningConfig fallback per AC #7).
    6. `#[cfg(test)] mod tests { ... }` block with the 4 tests per AC #12 verbatim.
  - [x] **Verify post-edit:** `cargo check 2>&1 | tee /tmp/story-4-1-check-task1.log; grep -cE 'warning:|error:' /tmp/story-4-1-check-task1.log` should output `0` only AFTER Task 2's `pub mod enemy;` lands. Defer the green-check expectation to end of Task 2.

- [x] **Task 2: Extend `src/combat/mod.rs` — `pub mod enemy;` + spawn_enemy_ship registration** (AC: #1, #3)
  - [x] Open `src/combat/mod.rs`. Add `pub mod enemy;` BETWEEN the existing `pub mod damage;` line and the `pub mod input;` line (alphabetical: `components, damage, enemy, input, projectiles`). Final ordering of the `pub mod` block:
    ```rust
    pub mod components;
    pub mod damage;
    pub mod enemy;
    pub mod input;
    pub mod projectiles;
    ```
  - [x] In `impl Plugin for CombatPlugin::build` (currently `src/combat/mod.rs:29-82`), locate the existing OnTransition registration of `projectiles::attach_combat_to_player_ship.in_set(CombatSystems::Setup)` (lines 57-63). Extend the system in-set tuple to register `enemy::spawn_enemy_ship` alongside it. Two equivalent forms are acceptable:
    
    **Option A — single tuple (compact, recommended):**
    ```rust
    app.add_systems(
        OnTransition {
            exited: GameState::MainMenu,
            entered: GameState::Arena,
        },
        (
            projectiles::attach_combat_to_player_ship,
            enemy::spawn_enemy_ship,
        )
            .in_set(CombatSystems::Setup),
    );
    ```
    
    **Option B — two separate `add_systems` calls (more verbose):**
    ```rust
    app.add_systems(
        OnTransition {
            exited: GameState::MainMenu,
            entered: GameState::Arena,
        },
        projectiles::attach_combat_to_player_ship.in_set(CombatSystems::Setup),
    );
    app.add_systems(
        OnTransition {
            exited: GameState::MainMenu,
            entered: GameState::Arena,
        },
        enemy::spawn_enemy_ship.in_set(CombatSystems::Setup),
    );
    ```
    
    Pick whichever reads cleaner with `cargo fmt`. Both are idiomatic per Bevy 0.18 system-tuple semantics.
  - [x] **Verify post-edit:** `cargo check 2>&1 | tee /tmp/story-4-1-check-task2.log; grep -cE 'warning:|error:' /tmp/story-4-1-check-task2.log` should output `0`. If `dead_code` warnings fire on `EnemyShip::Standard` or `Enemy` (because Story 4.1 only WRITES the components — no Query consumer yet; readers land in Stories 4.2 / 4.3), apply the project-precedent `#[allow(dead_code, reason = "...")]` pattern from `AsteroidDestroyed` (`src/combat/damage.rs:47-50`). Reason text: `"EnemyShip::Standard variant is read by Stories 4.2 (AI variants) and 4.4 (weapon archetypes); Story 4.1 establishes the typed slot. Enemy marker is queried by Story 4.2 AI transitions and Story 4.3 damage routing."`. Apply to `EnemyShip` enum if needed; `Enemy` may not need the allow if `cargo clippy` accepts a `Component`-derive struct without consumers.

- [x] **Task 3: Update `_bmad-output/implementation-artifacts/deferred-work.md` — annotate cleanup-3rd-consumer entry per AC #9** (AC: #9)
  - [x] Open `_bmad-output/implementation-artifacts/deferred-work.md`. Find the entry "Generic-cleanup home re-evaluation now triggered (3rd consumer pending)" at line 186, and the existing 3.11 update note at line 188.
  - [x] Append a new `> **📝 UPDATED 2026-05-XX by Story 4.1**` blockquote BELOW the 3.11 update note (preserve all prior content). Wording, matching AC #9: enemy entity introduced in Story 4.1 ALSO uses the dual-marker pattern (`Enemy + ArenaEntity`) rather than a 3rd direct consumer of `cleanup_on_exit::<T>` — same architectural decision as Story 3.11. The decision-trigger to relocate `cleanup_on_exit` to `src/core/cleanup.rs` remains the next genuine 3rd direct consumer (still likely Story 4.9 post-run-summary or 9.2 photo-mode camera).
  - [x] **Verify:** the entry continues to compile / read cleanly as a chronological 3.2 → 3.4 → 3.11 → 4.1 thread of decisions; no prior content modified.

- [x] **Task 4: Verification gates — all 6 cargo commands clean** (AC: #13)
  - [x] Run each command in sequence; capture FULL output (NOT just exit code or tail) per `feedback_full_build_output.md`:
    ```bash
    cargo check                                         2>&1 | tee /tmp/story-4-1-check.log
    cargo build                                         2>&1 | tee /tmp/story-4-1-build.log
    cargo test                                          2>&1 | tee /tmp/story-4-1-test.log
    cargo clippy --all-targets -- -D warnings           2>&1 | tee /tmp/story-4-1-clippy.log
    cargo fmt --all -- --check                          2>&1 | tee /tmp/story-4-1-fmt.log
    cargo build --release                               2>&1 | tee /tmp/story-4-1-release.log
    ```
  - [x] For EACH log: `grep -cE 'warning:|error:' /tmp/story-4-1-<cmd>.log` must output `0`. If non-zero, fix the root cause and re-run from the failing command. NO partial-pass shortcuts.
  - [x] `cargo test` log MUST contain `53 passed` AND `0 failed`. Confirm the literal line `test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (or accept a less specific variant: confirm `53 passed` AND `0 failed` are both present in the log).

- [x] **Task 5: Runtime smoke — full enemy-presence chain validation** (AC: #15)
  - [x] Till manually executes `cargo run 2>&1 | tee /tmp/story-4-1-run.log` and verifies scenarios (a)–(g) per AC #15. LLM cannot execute the interactive smoke (Enter on MainMenu, flight controls, fire LMB, Esc, Cmd-Tab, window-close). All preconditions for the smoke (Task 4 cargo gates) must be met.
  - [x] `grep -c 'spawned EnemyShip' /tmp/story-4-1-run.log` outputs **1** after a single Arena entry; **1** after a Pause round-trip (enemy does not respawn); **1** after a focus-loss round-trip (same).
  - [x] `grep -cE 'panic|backtrace|FATAL' /tmp/story-4-1-run.log` outputs **0**.
  - [x] `grep -cE 'WARN.*Enemy|ERROR.*Enemy' /tmp/story-4-1-run.log` outputs **0** (no Bevy or Avian errors related to the enemy entity setup).

- [x] **Task 6: Sprint status bookkeeping** (AC: #14)
  - [x] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - The create-story workflow Step 1 flips `epic-4: backlog → in-progress` (this is Story 4.1, the first story in epic 4).
    - The create-story workflow Step 6 flips `4-1-enemy-entity-foundation-semanticaccent-enemy: backlog → ready-for-dev`.
    - Dev workflow flips `ready-for-dev → in-progress → review → done` per the standard lifecycle.
    - Status flip `in-progress → review` is performed once Till's manual smoke (Task 5) is confirmed.
    - Update the `last_updated:` field (both the comment at the top of the file AND the `last_updated:` value in the document body) to the current date.

### Review Findings

- [x] [Review][Defer] `spawn_enemy_ship` has no idempotency guard against double-spawn [src/combat/enemy.rs:36-83] — deferred, pre-existing
- [x] [Review][Defer] `enemy_spawn_position_clears_close_asteroids` tests only 2 of 5 close-cluster asteroids [src/combat/enemy.rs:119-142] — deferred, pre-existing
- [x] [Review][Defer] No `Name` component on Enemy entity [src/combat/enemy.rs:58-76] — deferred, pre-existing
- [x] [Review][Defer] No `CollisionEventsEnabled` on Enemy entity — Story 4.3 will need it [src/combat/enemy.rs:58-76] — deferred, pre-existing

## Dev Notes

### Relevant architecture patterns and constraints

- **Plugin boundaries** (architecture.md:643-658) — `CombatPlugin` owns the enemy entity. Cross-plugin consumption: only public types from `arena` (`ArenaEntity`), `tuning` (`TuningHandle` / `TuningConfig`), and `visual` (`SemanticAccent`, `color_for`, `ToonMaterial`). NO cross-plugin Resource / Component mutation.
- **OnTransition discipline** (`deferred-work.md:220` — Story 3.9 fix) — pause-roundtrip preservation contract. Spawn on `OnTransition { exited: MainMenu, entered: Arena }`; cleanup runs on `OnTransition { exited: Arena, entered: MainMenu }` via existing `cleanup_on_exit::<ArenaEntity>` registered by `ArenaPlugin`. Epic-4 spec wording at line 18 ("OnEnter(GameState::Arena)") is implementation-detail-leakage from the planning phase, predates the OnTransition convention. Same deviation as every Epic-3 story from 3.5 onward.
- **Dual-marker cleanup** (precedent: PlayerShip at `src/flight/mod.rs:117-118`, HUD at Story 3.11 AC #2) — entities with state-scoped lifetimes carry their own component-vocabulary marker (`Enemy`) AND a state-scoped marker (`ArenaEntity`). The state-scoped marker drives cleanup via `cleanup_on_exit::<ArenaEntity>`; the component-vocabulary marker drives queries. This avoids introducing a 3rd direct consumer of the generic `cleanup_on_exit::<T>` (per `deferred-work.md:186-188`).
- **Past-tense event naming** (architecture.md:324) — N/A for Story 4.1 (no events introduced); Story 4.2 will introduce `EnemyDestroyed` matching the `AsteroidDestroyed` pre-declared shape at `src/combat/damage.rs:46-53`.
- **No magic numbers without rationale** (architecture.md:463) — `(0.0, 0.0, -60.0)` is the spawn-position constant with documented clearance rationale (AC #4); `(2.0, 4.0)` is the capsule dimensions matching collider exactly (AC #5). Neither is in `TuningConfig` for Story 4.1 because (a) only ONE enemy ships in this story; (b) Story 4.2 / 4.4 may externalize these to `TuningConfig` if multiple enemies / variants need them.
- **GameLayer enum convention** (`src/combat/damage.rs:25-31`) — current variants: `Default, Asteroid, Projectile`. Enemy stays on `Default` layer for 4.1; `GameLayer::Enemy` is Story 4.2's scope when enemy projectiles need their own collision filter (separate from player projectiles' `[GameLayer::Asteroid]` filter).
- **Cold-start TuningConfig fallback** (`src/flight/mod.rs:97-101`, `src/arena/zone.rs:50-55`, `src/combat/projectiles.rs:86-89`) — pre-existing project-wide pattern. Per `deferred-work.md:246`, this is silent (no error log) and consolidation is deferred to Epic 10 Story 10.2 asset-load audit. Story 4.1 follows the existing pattern.

### Source tree components to touch

| File | Change | LOC delta (estimate) |
|------|--------|---------------------|
| `src/combat/enemy.rs` | NEW: types + constants + `spawn_enemy_ship` + 4 tests | +130 |
| `src/combat/mod.rs` | Add `pub mod enemy;`; extend `CombatPlugin::build` with `enemy::spawn_enemy_ship` registration | +5 |
| `_bmad-output/implementation-artifacts/deferred-work.md` | Append UPDATED note to cleanup-3rd-consumer entry | +2 lines |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | bookkeeping (epic-4 → in-progress; story status flips; last_updated) | +0 net |

NO changes expected in: `src/arena/**`, `src/flight/**`, `src/ui/**`, `src/state.rs`, `src/pause/**`, `src/visual/**`, `src/splash.rs`, `src/logging.rs`, `src/main.rs`, `src/tuning/**`, `assets/**`, `Cargo.toml`, `Cargo.lock`, `.github/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`.

### Testing standards summary

- **Unit tests only** for Story 4.1 (architecture.md:354 — integration tests deferred post-M3). Pure-logic / type-vocabulary checks are first-class targets.
- 4 tests in `src/combat/enemy.rs`: `EnemyShip` explicit-construction guard; spawn-position 80 m budget; spawn-position cockpit-forward (-Z); spawn-position clears the two closest hand-picked asteroids.
- NO test for `spawn_enemy_ship` system (requires MinimalPlugins + state setup; same deferral as Stories 3.5–3.11).
- Runtime smoke (Task 5) covers system-level: MainMenu→Arena→enemy-visible→stationary→projectiles-pass-through→pause-roundtrip→focus-loss-roundtrip→quit-clean.
- Test count post-4.1: **53** (= 49 baseline from end of 3.11 + 4 new). AC #13 enforces.

### Project Structure Notes

- **Alignment with unified project structure:** `src/combat/enemy.rs` is a new submodule under `src/combat/`. Architecture.md:570 reserves `src/combat/enemy_ai.rs` for FR14 AI (Story 4.2's scope). For Story 4.1 we go with `enemy.rs` as the entity-foundation module name; Story 4.2 will introduce `enemy_ai.rs` for the state machine. The two separate modules cleanly separate **what the enemy IS** (entity definition — Story 4.1) from **what it DOES** (AI behavior — Story 4.2). This matches the project's pattern of `src/flight/components.rs` (data) vs. `src/flight/physics.rs` (behavior).
- **Detected variances:**
  - Epic-4 AC at line 18 says "OnEnter(GameState::Arena) runs after Story 3.3's zone spawn" — implementation uses `OnTransition { exited: MainMenu, entered: Arena }` per project pattern. Same deviation as every story from 3.5 onward; documented inline in AC #3 and dev notes. Spec amendment deferred per Story 3.9 deferred-work.md:262 precedent.
  - Epic-4 AC at line 17 references "salvage (yellow) asteroids" — but `palette.rs:23-24` defines `Salvage = #009E73 bluish-green` and `Hazard = #F0E442 yellow`. Color-vocabulary mismatch in the epic spec. For Story 4.1 in the runtime, asteroids are still `SemanticAccent::Neutral` (grey, per `src/arena/zone.rs:94`); the enemy's vermillion is trivially distinct. Story 4.5 will re-tint asteroids to `SemanticAccent::Salvage` (bluish-green, NOT yellow as epic spec implies). Recommend epic-4 AC text be amended at Story 4.5 review pass; Story 4.1 does not block on the spec correction.
- **Architecture compliance:** plugin boundaries respected (CombatPlugin owns Enemy; arena/tuning/visual modules consumed only via public type imports); OnTransition pattern preserves pause round-trip; no new `SystemSet` variant added (re-uses existing `CombatSystems::Setup`); no new direct consumer of `cleanup_on_exit::<T>` (dual-marker pattern).

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-4-enemies-alive-stop-ship-itchio-prototype.md#Story-4.1] — story spec (Acceptance Criteria source)
- [Source: _bmad-output/planning-artifacts/architecture.md#Plugin-Boundaries] — `CombatPlugin` ownership (line 648)
- [Source: _bmad-output/planning-artifacts/architecture.md#FR-Mapping] — FR14 enemy AI / `src/combat/enemy_ai.rs` (line 685, reserved for Story 4.2)
- [Source: _bmad-output/planning-artifacts/architecture.md#Project-Directory-Structure] — `src/combat/` submodule layout (line 564-570)
- [Source: _bmad-output/implementation-artifacts/3-11-hud-baseline-screen-space-placeholders.md] — recent precedent for dual-marker spawn, story file structure, deviation-from-epic-spec documentation
- [Source: src/flight/mod.rs:88-140] — `spawn_player_ship` — OnTransition spawn + dual-marker (`PlayerShip + ArenaEntity`) + `OutlineVolume` + `RigidBody::Dynamic` precedent
- [Source: src/arena/zone.rs:43-104] — `spawn_arena_zone` — TuningConfig cold-start fallback + `ToonMaterial` + `OutlineVolume` closure + `ArenaEntity` precedent
- [Source: src/combat/projectiles.rs:37-51] — `attach_combat_to_player_ship` — `CombatSystems::Setup` registration on OnTransition precedent
- [Source: src/combat/mod.rs:29-82] — `CombatPlugin::build` — current `OnTransition` registration site for `CombatSystems::Setup` systems
- [Source: src/combat/components.rs:33-36] — `AsteroidHp` — explicit-construction `#[derive(Component, Debug, Clone, Copy, PartialEq)]` no-Default precedent
- [Source: src/combat/damage.rs:25-31] — `GameLayer` enum — Default/Asteroid/Projectile (Enemy stays on Default for 4.1)
- [Source: src/combat/damage.rs:46-53] — `AsteroidDestroyed` — pre-declared `#[derive(Message)]` shape that 4.2's `EnemyDestroyed` will mirror
- [Source: src/visual/palette.rs:11-28] — `SemanticAccent` enum + `color_for` (Enemy = `#D55E00` vermillion)
- [Source: src/main.rs:43] — `Gravity(Vec3::ZERO)` — Dynamic body stays put without external force
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:186-188] — Generic-cleanup home re-evaluation entry (closed-by-non-introduction in this story, like Story 3.11)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:206] — Story 4.5 SemanticAccent retroactive sweep (sets the convention; Story 4.1 prefigures it for new entities)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:220] — Story 3.9 OnTransition fix (the source-of-truth for pause-roundtrip preservation)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:262] — "Spec-Amend kostet mehr als es nützt" precedent for declining spec text amendments in favor of deferred-work documentation
- [Source: avian3d-0.6.x] — `Collider::capsule(radius, length)` + `RigidBody::Dynamic` + `LinearVelocity` / `AngularVelocity` API
- [Source: bevy::math::primitives::Capsule3d] — Bevy 0.18 capsule primitive (re-exported via prelude)

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (1M context)

### Debug Log References

- `/tmp/story-4-1-check.log` — `cargo check` (0 warning|error)
- `/tmp/story-4-1-build.log` — `cargo build` (0 warning|error)
- `/tmp/story-4-1-test.log` — `cargo test` (`test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`)
- `/tmp/story-4-1-clippy.log` — `cargo clippy --all-targets -- -D warnings` (0 warning|error after const-block fix)
- `/tmp/story-4-1-fmt.log` — `cargo fmt --all -- --check` (0 warning|error, no diff)
- `/tmp/story-4-1-release.log` — `cargo build --release` (0 warning|error, finished in 4m 24s)

### Completion Notes List

- **Task 1 (`src/combat/enemy.rs`)** — authored verbatim per AC #2, #5–#8, #10–#12: module doc, `use bevy::prelude::*` + Avian + outline + `crate::arena::ArenaEntity` + tuning + visual imports, three constants (`ENEMY_SPAWN_POSITION = Vec3::new(0.0, 0.0, -60.0)`, `ENEMY_CAPSULE_RADIUS = 2.0`, `ENEMY_CAPSULE_LENGTH = 4.0`), `Enemy` empty marker, `EnemyShip::Standard` enum, `spawn_enemy_ship` system with cold-start tuning fallback + `OutlineVolume` construction + capsule mesh+material allocation + spawn-tuple matching AC #10 verbatim + `info!` log line, and `#[cfg(test)] mod tests` block with the 4 prescribed tests.
- **Dead-code allow on `EnemyShip` enum** — applied preemptively per Task 2 verify-step guidance: `#[allow(dead_code, reason = "EnemyShip::Standard variant is read by Stories 4.2 (AI variants) and 4.4 (weapon archetypes); Story 4.1 establishes the typed slot.")]`. The `Enemy` marker did NOT need the allow — its construction in the spawn tuple plus the `Component` derive trait-impl is sufficient to satisfy `cargo clippy --all-targets -- -D warnings`.
- **Task 2 (`src/combat/mod.rs`)** — added `pub mod enemy;` between `pub mod damage;` and `pub mod input;` (alphabetical: `components, damage, enemy, input, projectiles`). Extended the existing `OnTransition { exited: MainMenu, entered: Arena }` registration to a tuple `(projectiles::attach_combat_to_player_ship, enemy::spawn_enemy_ship).in_set(CombatSystems::Setup)` (Option A — single tuple, recommended in story spec for compactness). NO new SystemSet variant introduced. NO new plugin (EnemyPlugin) introduced.
- **Task 3 (`deferred-work.md`)** — appended `> **📝 UPDATED 2026-05-06 by Story 4.1**` blockquote BELOW the existing 3.11 update note under "Generic-cleanup home re-evaluation now triggered (3rd consumer pending)". Wording per AC #9: enemy entity uses dual-marker (`Enemy + ArenaEntity`), matching the architectural decision Story 3.11 made for HUD; trigger to relocate `cleanup_on_exit` remains the next genuine 3rd direct consumer.
- **Task 4 (cargo gates)** — all 6 commands tee'd to `/tmp/story-4-1-*.log`; `grep -cE 'warning:|error:'` returns `0` for each; `cargo test` reports `53 passed; 0 failed` (= 49 baseline + 4 new) precisely matching AC #13 expected count. `cargo build --release` succeeded in 4m 24s.
- **Clippy const-assertion fix** — initial `cargo clippy --all-targets -- -D warnings` failed at `enemy_spawn_position_in_front_of_cockpit` test with `clippy::assertions_on_constants` (the comparison `ENEMY_SPAWN_POSITION.z < 0.0` is a const-evaluable expression). Replaced `assert!(...)` with `const { assert!(ENEMY_SPAWN_POSITION.z < 0.0) }` — compile-time check, matches HUD font-size precedent at `src/ui/hud.rs::tests::hud_font_size_smaller_than_pause_overlay`. Re-ran clippy: 0 warning|error. Re-ran cargo test: 53 passed, 0 failed.
- **Task 5 (runtime smoke)** — Till confirmed scenarios (a)–(g) per AC #15 on 2026-05-06: enemy visible vermillion-tinted ~60m forward, capsule silhouette distinct from PlayerShip cuboid, stationary, projectiles pass through (damage routing is 4.2 scope), pause + focus-loss round-trips preserve enemy with single `spawned EnemyShip` log occurrence, clean quit with no panic / backtrace / FATAL lines.
- **Task 6 (sprint-status)** — `epic-4: backlog → in-progress` flipped at story-creation time (Story 4.1 is the first story in epic 4). `4-1-enemy-entity-foundation-semanticaccent-enemy: ready-for-dev → in-progress → review` per the standard lifecycle; final `review` flip applied after Till's Task 5 smoke confirmation. `last_updated:` field updated to `2026-05-06`. Story file `Status: review` set per workflow Step 9.
- **Architecture-compliance notes** — (a) plugin boundary respected: `CombatPlugin` owns the enemy entity; arena/tuning/visual modules consumed only via public type imports (`ArenaEntity`, `TuningHandle`, `TuningConfig`, `SemanticAccent`, `color_for`, `ToonMaterial`); no cross-plugin Resource/Component mutation. (b) `OnTransition` discipline preserved per the post-3.9 pause-roundtrip preservation contract. (c) Dual-marker pattern (`Enemy + ArenaEntity`) avoids introducing a 3rd direct consumer of `cleanup_on_exit::<T>`. (d) NO `GameLayer::Enemy` introduced — Story 4.2's scope; enemy stays on default layer in 4.1, ship-vs-enemy bounce contacts handled correctly by existing `detect_projectile_asteroid_hits` filter. (e) Per-entity `SemanticAccent::Enemy` Component prefigures Story 4.5's full retroactive sweep on existing entities. (f) NO `tuning.ron` mutation, no Cargo dep additions — consistent with AC #14's expected file-set.

### File List

- **NEW** `src/combat/enemy.rs` — module doc, use block, 3 constants, `Enemy` marker, `EnemyShip::Standard` enum (with `#[allow(dead_code, reason = ...)]`), `spawn_enemy_ship` system with cold-start tuning fallback, `#[cfg(test)] mod tests` (4 tests). 130 lines.
- **MODIFIED** `src/combat/mod.rs` — added `pub mod enemy;`; extended OnTransition registration tuple to include `enemy::spawn_enemy_ship` alongside `projectiles::attach_combat_to_player_ship` in `CombatSystems::Setup`. +5 lines net.
- **MODIFIED** `_bmad-output/implementation-artifacts/deferred-work.md` — appended `📝 UPDATED 2026-05-06 by Story 4.1` blockquote under the "Generic-cleanup home re-evaluation now triggered (3rd consumer pending)" entry. +2 lines.
- **MODIFIED** `_bmad-output/implementation-artifacts/sprint-status.yaml` — flipped `epic-4: backlog → in-progress`; flipped `4-1-enemy-entity-foundation-semanticaccent-enemy: backlog → ready-for-dev → in-progress`; updated `last_updated:` field. Final flip `in-progress → review` upon Till's Task 5 confirmation.
- **MODIFIED** `_bmad-output/implementation-artifacts/4-1-enemy-entity-foundation-semanticaccent-enemy.md` — Status, Tasks/Subtasks checkboxes, Dev Agent Record, File List, Change Log updated by this dev workflow run.

## Change Log

| Date       | Author       | Change                                                                                                                                                                          |
|------------|--------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 2026-05-06 | Amelia (Dev) | Authored `src/combat/enemy.rs` (Enemy marker + EnemyShip::Standard enum + `spawn_enemy_ship` + 4 tests); extended `CombatPlugin` OnTransition tuple with enemy spawn; updated deferred-work.md cleanup-3rd-consumer entry; clippy const-assertion fix (`const { assert!(...) }`); all 6 cargo gates green (53 tests passing); Till confirmed runtime smoke (a)–(g) per AC #15 — Status flipped to `review`. |
