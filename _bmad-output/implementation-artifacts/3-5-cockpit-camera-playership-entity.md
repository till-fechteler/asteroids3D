# Story 3.5: Cockpit Camera + PlayerShip Entity

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player entering the Arena,
I want a visible placeholder ship at the spawn point with a first-person Camera3d attached as a child at the pilot-seat position,
So that the FR8 cockpit-only commitment lands from frame one — every subsequent flight, weapon, and HUD story (3.6–3.11) attaches to a single canonical `PlayerShip` entity instead of free-floating Camera3d hacks, and Story 3.3's stand-in Camera3d (a known temporary scaffold) is replaced by the real architecture.

## Acceptance Criteria

1. **Given** Story 3.3 spawned a stand-in `Camera3d` at `src/arena/zone.rs:65-69` (tagged `ArenaEntity`, positioned at `Transform::from_xyz(0.0, 5.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y)`) as a temporary scaffold per Story 3.3's hand-off contract (deferred-work.md:160)
   **When** Story 3.5 lands
   **Then** the stand-in `Camera3d` spawn block in `src/arena/zone.rs` (the `commands.spawn((Camera3d::default(), Transform::from_xyz(...).looking_at(...), ArenaEntity,));` and the preceding 1-line `// Stand-in Camera3d` comment) is **deleted entirely** — NOT queried-and-despawned at runtime
   **And** the Arena scene's sole `Camera3d` for Stories 3.5+ is the cockpit `Camera3d` spawned as a child of `PlayerShip` per AC #4 below (no stand-in coexistence at all → no "ambiguous camera order" runtime warnings → no race between two `Camera3d` entities at default `order: 0`)

2. **Given** the architecture mandates a feature plugin per `<Feature>Systems` SystemSet pattern at `src/<feature>/mod.rs` (architecture.md:343-350, :558-563)
   **When** Story 3.5 introduces flight scaffolding
   **Then** a new file `src/flight/mod.rs` is authored with a `FlightPlugin: Plugin` type
   **And** the plugin declares a `FlightSystems` `SystemSet` enum with at least one variant (`Setup`, for the OnEnter(Arena) PlayerShip-spawn system; later stories add `Input`, `Physics`, `PostPhysics` variants)
   **And** the plugin declares a `PlayerShip` unit-struct `Component` marker
   **And** the plugin declares a `CockpitCamera` unit-struct `Component` marker
   **And** `FlightPlugin` is registered in `src/main.rs` via `App::add_plugins(FlightPlugin)`, placed AFTER `ArenaPlugin` registration and BEFORE `PausePlugin` registration (rustfmt may reorder the surrounding `mod`/`use` lines alphabetically — accept its order)
   **And** `mod flight;` + `use flight::FlightPlugin;` are added near the other top-level `mod`/`use` lines in `main.rs`

3. **Given** the PlayerShip must spawn AFTER the asteroid field exists (so the `≥3 asteroids within 50 m of origin` line-of-sight precondition is queryable) and BEFORE any later 3.6–3.11 system runs
   **When** `OnEnter(GameState::Arena)` runs
   **Then** the cross-plugin SystemSet order is `(ArenaSystems::Setup, FlightSystems::Setup).chain()` configured in `FlightPlugin::build` (the `configure_sets((ArenaSystems::Setup, FlightSystems::Setup).chain())` registration)
   **And** the PlayerShip spawn system is registered in `OnEnter(GameState::Arena)` `.in_set(FlightSystems::Setup)`
   **And** rationale per architecture.md:415: `.after(spawn_arena_zone)` ordering is forbidden because it breaks at function rename; SystemSet chaining is the architecturally approved pattern
   **And** `use crate::arena::ArenaSystems;` import is added to `src/flight/mod.rs` to reference the cross-plugin set

4. **Given** the PlayerShip must satisfy the epic-3 spec: visible placeholder mesh + `RigidBody::Dynamic` + `Collider` + sized to mesh + `ArenaEntity` marker
   **When** the spawn system runs (call it `spawn_player_ship`)
   **Then** **exactly one** `PlayerShip` entity is spawned at `Transform::from_xyz(0.0, 0.0, 0.0)` (origin — Story 3.3's asteroid layout deliberately keeps origin clear per `src/arena/zone.rs:14-17`)
   **And** the entity carries the spawn-tuple components in this order:
   - `PlayerShip` marker (this story's marker)
   - `ArenaEntity` marker (state-cleanup; honors deferred-work.md:152 convention)
   - `Mesh3d(...)` from a `Cuboid::new(4.0, 2.0, 6.0)` placeholder mesh (4 m wide × 2 m tall × 6 m long — small-fighter silhouette; X = wingspan, Y = height, Z = length; Bevy convention forward = -Z)
   - `MeshMaterial3d(...)` with a `ToonMaterial { tint: color_for(SemanticAccent::Neutral).into(), ..default() }` (Neutral accent — Story 4.5 owns the `SemanticAccent::PlayerOwned` wiring per deferred-work.md:162; prematurely consuming `PlayerOwned` here would split that story's scope)
   - `bevy_mod_outline::OutlineVolume` from the same `outline_volume()` builder pattern as `src/arena/zone.rs:55-62` (so the placeholder ship gets the same toon-outline treatment as the asteroids — keeps the M1 vector aesthetic consistent)
   - `RigidBody::Dynamic` (Avian — gravity inherited zero from `Gravity(Vec3::ZERO)` set in main.rs:39, so no falling at spawn)
   - `Collider::sphere(2.0)` (sphere collider, radius 2.0 m — chosen over capsule for: simpler geometry, matches Story 3.3's `Collider::sphere(radius)` precedent for asteroids, fast collision math, sufficient for placeholder. ~roughly bounds the 4×2×6 cuboid with some slack at the long-axis ends — Story 3.10 projectile-asteroid collision works fine even with collider slack since this is a placeholder)
   - `LinearVelocity::ZERO` and `AngularVelocity::ZERO` (Avian — explicit zero initial velocity per AC #6 below; defaults are also zero but explicit is robust against future Avian default changes)

5. **Given** the cockpit `Camera3d` must be a CHILD of the `PlayerShip` so Bevy's transform propagation auto-positions it from the parent's `RigidBody`-driven transform every frame
   **When** the spawn system spawns the PlayerShip
   **Then** the spawn uses the `.with_children(|parent| { parent.spawn(...) })` builder pattern (mirrors Story 3.4 `src/pause/mod.rs:159-179` precedent)
   **And** the child entity carries:
   - `Camera3d::default()` (Bevy 0.18 — required components auto-insert `Transform` + `GlobalTransform` + `Camera` + render-graph wiring)
   - `CockpitCamera` marker (this story's marker; later stories query `Query<&Camera3d, With<CockpitCamera>>` to disambiguate from any future debug/photo-mode cameras — F3-debug per architecture.md:231-235 + photo-mode per epic-9)
   - `Transform::from_xyz(0.0, 0.6, 0.5)` — local-space offset relative to PlayerShip:
     - `X = 0.0` — centered along wingspan
     - `Y = 0.6` — slightly above the cuboid center (cuboid top is at Y=+1.0; pilot eye at Y=+0.6 is "head poking above the dashboard" — readable without occluding the wing-tips)
     - `Z = 0.5` — slightly behind the mesh origin (cuboid origin is centered; +Z is rear given Bevy forward = -Z; pilot seat is in the rear half of the cuboid, looking forward through the long-axis "windshield")
     - **No explicit rotation** — Bevy's default Camera3d looks down -Z, which matches the ship's forward direction (PlayerShip's `Transform::IDENTITY` orientation → `transform.forward()` returns `-Z`). The "angled slightly downward" pitch from the epic spec is OPTIONAL polish — leave the default 0° pitch for Story 3.5 and let later stories tune the angle if cockpit immersion needs it.

6. **Given** the epic spec requires "zero initial linear and angular velocity (no drift at spawn)"
   **When** the PlayerShip is spawned
   **Then** the spawn tuple includes `LinearVelocity(Vec3::ZERO)` and `AngularVelocity(Vec3::ZERO)` Avian components explicitly
   **And** any future `ExternalForce` / `ExternalTorque` from Story 3.6 / 3.7 will be additive to the zero baseline
   **And** the ship visibly remains stationary on Arena entry (no observable drift) — verified by runtime smoke (per AC #8 below)

7. **Given** the line-of-sight precondition must be programmatically verified, not just spec-asserted (Story 3.3 already added a unit test `at_least_three_asteroids_within_50m_of_origin` at `src/arena/zone.rs:165-172` enforcing the layout invariant)
   **When** Story 3.5's PlayerShip spawns at `Transform::from_xyz(0.0, 0.0, 0.0)`
   **Then** the existing zone test continues to pass (the asteroid layout is unchanged → invariant holds)
   **And** runtime smoke confirms the player visibly sees ≥3 asteroids within ~50 m on Arena entry from the cockpit camera (close-cluster asteroids at z = -25, -38, -42 are dead ahead given Bevy's -Z forward convention — the layout was deliberately built this way per `src/arena/zone.rs:14-17`)
   **And** NO new unit test is added in `src/flight/mod.rs` to re-assert this — the assertion lives at the asteroid-layout source-of-truth in `src/arena/zone.rs`, not redundantly in flight code (DRY; matches Story 3.3's choice to keep invariants colocated with the data)

8. **Given** the post-3.4 source baseline (test count = 21; `cargo build --release` 0 warnings; main.rs ~58 lines; no `src/flight/` directory exists)
   **When** Story 3.5 verification runs locally (per `feedback_full_build_output.md` — exit-0 + tail is NOT proof; grep explicitly per command)
   **Then** **all six** of `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --release` produce **0** lines matching `grep -cE 'warning:|error:'` per their respective full output logs (capture each to `/tmp/story-3-5-<command>.log`)
   **And** `cargo test` summary line reads exactly `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N ≥ 21** (baseline preserved; new tests for `PlayerShip` / `CockpitCamera` invariants are optional 0–2 budget per "Test policy" in Dev Notes)
   **And** `cargo run` (with `RUST_LOG=info,wgpu=warn,naga=warn,avian3d=info` or similar) opens a window, transitions Loading → MainMenu → Arena, AND in Arena the player can: (a) see the asteroid field from the cockpit POV (the placeholder cuboid ship's nose may be visible at the bottom of the frame depending on cockpit Y-offset; this is expected); (b) press Esc → "PAUSED — Esc to resume" appears (Story 3.4 still works); (c) Esc again → resume (Story 3.4 still works); (d) confirm the ship does NOT drift in any direction over a 5-second observation window (no flight input exists yet — Story 3.6/3.7)
   **And** `/tmp/story-3-5-run.log` contains exactly: 1 occurrence each of `entered Loading`, `entered MainMenu`, `entered Arena`; 0 occurrences of `panic`, `backtrace`, `FATAL`, `ambiguous camera order` (the latter is the regression signal that the stand-in Camera3d wasn't fully removed); 0 NEW `ERROR`-level logs from Bevy/Avian/wgpu beyond the documented prior noise (splash-cleanup race per deferred-work.md:75-76, :137, :168; winit `Skipped event Destroyed` per Story 1.6 deferred-work LOW-1; pre-existing wgpu fragment-output warning per Story 3.3 dev-log)
   **And** `git status --short` final set is **exactly**: `src/main.rs` (M — `mod flight;` + `use flight::FlightPlugin;` + `add_plugins(FlightPlugin)`), `src/flight/mod.rs` (?? — new file), `src/arena/zone.rs` (M — stand-in Camera3d block deleted), `_bmad-output/implementation-artifacts/sprint-status.yaml` (M — bookkeeping), `_bmad-output/implementation-artifacts/3-5-cockpit-camera-playership-entity.md` (M — this file's Status flip + Dev Agent Record), `_bmad-output/implementation-artifacts/deferred-work.md` (M — at minimum a "stand-in Camera3d removed" RESOLVED note on the deferred-work.md:160 entry, plus any new forward-compat entries discovered during impl); **NO** entries under `Cargo.toml`, `Cargo.lock`, `src/state.rs` (see Dev Notes "Optional `Copy` derive on GameState" deviation guidance), `src/splash.rs`, `src/logging.rs`, `src/ui/**`, `src/visual/**`, `src/tuning/**`, `src/pause/**`, `src/arena/mod.rs` (only `src/arena/zone.rs` changes), `assets/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`

9. **Given** Bevy 0.18 + Avian 0.6 transform propagation: the `Camera3d` child's `GlobalTransform` is computed from `PlayerShip.GlobalTransform * Camera3d.LocalTransform` every frame by Bevy's standard transform propagation system (no Avian-specific wiring required — Avian only writes to the `RigidBody`-bearing entity's `Transform`, which then cascades via Bevy's `propagate_transforms` to the child)
   **When** future Stories 3.6/3.7 apply `ExternalForce` / `ExternalTorque` to the `PlayerShip` (the parent, not the camera)
   **Then** the parent's `Transform` updates via Avian's `FixedUpdate` integration
   **And** the child Camera3d's `GlobalTransform` follows automatically
   **And** NO custom "camera-follows-ship" system is needed — this is a load-bearing architectural simplification that Story 3.5 unlocks for every later story. **Do NOT** add a `track_camera_to_ship` system; that would be redundant with Bevy's built-in propagation and would create a one-frame lag race.

## Tasks / Subtasks

- [x] **Task 1: Author `src/flight/mod.rs` — FlightPlugin + PlayerShip + CockpitCamera + spawn_player_ship system + cross-plugin SystemSet chaining** (AC: #2, #3, #4, #5, #6, #9)
  - [x] Create `src/flight/` directory at the repo root (sibling of `src/arena/`, `src/pause/`).
  - [x] Create `src/flight/mod.rs`. Target file size: **~120–180 lines** including module doc, plugin impl, marker components, spawn system, optional unit tests. Comment density per `karpathy-guidelines.md` — only WHY-comments where invariants are non-obvious.
  - [x] **Module doc** 2 lines max, no story-id references (per Story 1.5 review patch BH8 + Story 3.2 patch precedent — see commit `5134b3c`). Suggested: `//! FlightPlugin — owns PlayerShip + CockpitCamera spawn on Arena entry.\n//! Later stories attach 6-DOF input, rotation, dampener, weapons via additional systems.`
  - [x] **Imports:**
    ```rust
    use avian3d::prelude::{AngularVelocity, Collider, LinearVelocity, RigidBody};
    use bevy::prelude::*;
    use bevy_mod_outline::OutlineVolume;

    use crate::arena::{ArenaEntity, ArenaSystems};
    use crate::state::GameState;
    use crate::tuning::TuningHandle;
    use crate::tuning::config::TuningConfig;
    use crate::visual::palette::{SemanticAccent, color_for};
    use crate::visual::toon_material::ToonMaterial;
    ```
    Avoid wildcard imports beyond `bevy::prelude::*`. Note `AngularVelocity` + `LinearVelocity` re-exports — these are in the Avian prelude per `avian3d/src/lib.rs:550-555` (same precedent as `Physics` + `PhysicsTime` in Story 3.4).
  - [x] **Plugin skeleton:**
    ```rust
    pub struct FlightPlugin;

    #[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum FlightSystems {
        Setup,
    }

    #[derive(Component)]
    pub struct PlayerShip;

    #[derive(Component)]
    pub struct CockpitCamera;

    impl Plugin for FlightPlugin {
        fn build(&self, app: &mut App) {
            app.configure_sets(
                OnEnter(GameState::Arena),
                (ArenaSystems::Setup, FlightSystems::Setup).chain(),
            );
            app.add_systems(
                OnEnter(GameState::Arena),
                spawn_player_ship.in_set(FlightSystems::Setup),
            );
        }
    }
    ```
    - **Why a `FlightSystems::Setup` set with only one variant for now:** mirrors 3.2/3.3/3.4 `<Feature>Systems` per-plugin idiom. Stories 3.6–3.8 will add `Input`, `Physics`, `PostPhysics` variants (per architecture.md:411-412 example `enum FlightSystems { ReadInput, ApplyForces, IntegratePhysics }`). Story 3.5 declares the set with one variant; later stories extend.
    - **Why `(ArenaSystems::Setup, FlightSystems::Setup).chain()`:** the PlayerShip spawn must happen AFTER `spawn_arena_zone` (which owns `ArenaSystems::Setup` per `src/arena/mod.rs:11-13, 20`). Cross-plugin SystemSet chaining via `configure_sets` is architecture-approved (architecture.md:413-414). The chain is registered in `FlightPlugin` (the dependent plugin), not `ArenaPlugin` (the depended-upon plugin) — keeps the dependency direction explicit and avoids forcing ArenaPlugin to know about FlightSystems.
  - [x] **System: `spawn_player_ship`** (AC: #4, #5, #6)
    ```rust
    pub fn spawn_player_ship(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ToonMaterial>>,
        tuning_assets: Res<Assets<TuningConfig>>,
        tuning_handle: Res<TuningHandle>,
    ) {
        // Cold-start safety mirrors src/arena/zone.rs:48-54 — tuning.ron may not be loaded
        // yet on a hypothetical future re-entry path; fall back to defaults with a warn.
        let tuning_opt = tuning_assets.get(tuning_handle.0.id());
        if tuning_opt.is_none() {
            warn!("tuning.ron not loaded at PlayerShip spawn; using TuningConfig defaults");
        }
        let tuning = tuning_opt.cloned().unwrap_or_default();
        let [r, g, b, a] = tuning.outline_color;
        let outline = OutlineVolume {
            visible: true,
            width: tuning.outline_width,
            colour: Color::srgba(r, g, b, a),
        };

        let ship_mesh = meshes.add(Cuboid::new(4.0, 2.0, 6.0));
        let ship_material = materials.add(ToonMaterial {
            tint: color_for(SemanticAccent::Neutral).into(),
            ..default()
        });

        commands
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
            ))
            .with_children(|parent| {
                parent.spawn((
                    Camera3d::default(),
                    CockpitCamera,
                    Transform::from_xyz(0.0, 0.6, 0.5),
                ));
            });

        info!("spawned PlayerShip at origin with cockpit Camera3d child");
    }
    ```
    - **`Mesh3d(ship_mesh)` + `MeshMaterial3d(ship_material)` (NOT `MaterialMeshBundle`):** Bevy 0.18 deprecated bundles in favor of required-components — `Mesh3d` and `MeshMaterial3d` are the modern wrappers per the Story 3.3 precedent at `src/arena/zone.rs:96-98`. Using a deprecated bundle pattern would emit a clippy warning under `-D warnings`.
    - **`Camera3d::default()` (NOT `Camera3dBundle`):** same Bevy 0.18 required-components pattern; `Camera3d` auto-inserts `Transform`, `GlobalTransform`, `Camera`, `Projection`, plus render-graph wiring per Bevy 0.18's `#[require(...)]` declarations.
    - **`.with_children(|parent| parent.spawn(...))`:** Bevy 0.18 hierarchy builder pattern (mirror `src/pause/mod.rs:159-179`). Auto-attaches the `ChildOf` component on the child for linked-despawn semantics — when the parent `PlayerShip` is despawned via `cleanup_on_exit::<ArenaEntity>` on `OnExit(Arena)`, the child `Camera3d` is auto-despawned by Bevy's linked-despawn cascade. The child does NOT need its own `ArenaEntity` marker (over-tagging risk per deferred-work.md:92-94). **Verify post-implementation:** `grep -c 'ArenaEntity' src/flight/mod.rs` should equal **2** (1 component import + 1 spawn-tuple use on the parent), NOT 3 (no marker on the child).
    - **Component-tuple ordering:** marker first, then state-cleanup marker, then mesh + material + transform, then outline, then physics components. This grouping makes diffs readable when later stories add components.
    - **`LinearVelocity(Vec3::ZERO)` + `AngularVelocity(Vec3::ZERO)` constructors:** Avian 0.6 components are tuple-structs around `Vec3`. The `(Vec3::ZERO)` form is robust; `LinearVelocity::default()` would also work but the explicit-zero form documents the no-drift-at-spawn invariant (AC #6) at the call site.
    - **`info!` log:** keeps the lifecycle-event logging discipline from 3.4 (`info!("spawned ...")`). One log line per state-entry consequence is the architecture.md:380 `info!` level guidance.
  - [x] **Why `Collider::sphere(2.0)` not `Collider::capsule(...)`:** the epic spec allows either; sphere is chosen for (a) computational cheapness — sphere-vs-sphere collisions are O(1), (b) consistency with Story 3.3 `Collider::sphere(radius)` pattern, (c) appropriate fidelity for a placeholder ship that will be replaced by a real glTF cockpit mesh in a future asset-creation story (M2 polish or later). The 2.0 m radius bounds the 4 m wingspan with conservative slack and underfits the 6 m length; this is a placeholder, not a final fit. Story 3.10 (projectile-asteroid collision) is unaffected — it's about asteroids, not the ship hitbox.
  - [x] **Why ship at origin (`Transform::from_xyz(0.0, 0.0, 0.0)`):** Story 3.3's asteroid layout deliberately keeps origin clear (`src/arena/zone.rs:14-17` comment + the `at_least_three_asteroids_within_50m_of_origin` test at `src/arena/zone.rs:165-172`). Spawning at origin guarantees AC #7's line-of-sight precondition is met without any new test code. **Do NOT** modify the asteroid layout to accommodate a different spawn position — that would invalidate Story 3.3's test invariants and break the deliberate corridor design.
  - [x] **Camera local Transform `(0.0, 0.6, 0.5)` rationale:**
    - X = 0.0 — centered along wingspan
    - Y = 0.6 — eye height above the cuboid's local Y center; cuboid top is at +1.0 Y, so the camera is 0.4 m below the canopy's "ceiling"
    - Z = 0.5 — slightly behind the mesh origin in ship-local +Z (i.e., toward the rear of the ship given Bevy's forward = -Z convention); the placeholder cuboid extends from Z=-3.0 (nose) to Z=+3.0 (rear); a Z=+0.5 camera position is "in the rear-half pilot seat" looking forward toward the nose at Z=-3.0
    - Default Camera3d look direction is -Z (Bevy convention) → matches PlayerShip's forward → no rotation needed → asteroids in the close-cluster (z = -25 to -42) are dead ahead
  - [x] **Optional unit tests** (test budget: 0–2; per Story 3.4 precedent on trivial-data invariants):
    - `playership_marker_distinct_from_cockpit_camera_marker` — a no-op type-equality test confirming `PlayerShip` and `CockpitCamera` are distinct types (catches accidental type-alias regression). Skip if it feels like noise.
    - `flight_systems_setup_variant_exists` — `let _ = FlightSystems::Setup;` to keep the variant alive against future dead-code lints if 3.5 is the only consumer at merge time. Skip if `cargo clippy -- -D warnings` is clean without it.
    - **Recommended:** add **0** tests in 3.5 — the spawn system is integration-test-shaped (would need a Bevy `App`-bootstrap test harness which is deferred per architecture.md:354 "Integration tests deferred post-M3 unless a specific regression forces them"). The runtime smoke (AC #8) is the de-facto integration test for 3.5.

- [x] **Task 2: Edit `src/arena/zone.rs` — delete the stand-in Camera3d spawn block** (AC: #1)
  - [x] Open `src/arena/zone.rs`. Locate the 5-line spawn block at lines **~64-69** (current state):
    ```rust
    // Stand-in Camera3d — Story 3.5 replaces with cockpit camera (child of PlayerShip).
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y),
        ArenaEntity,
    ));
    ```
  - [x] **Delete the entire block, including the leading comment.** Net `src/arena/zone.rs` line count drops by 5–6 lines.
  - [x] **Do NOT delete the DirectionalLight spawn block** that immediately follows (lines ~71-80) — the directional light is REQUIRED by Story 3.3 AC #2 and load-bearing for toon-shader posterization on the new PlayerShip mesh as well.
  - [x] **Do NOT touch the `ASTEROIDS` const-array** (lines 14-39) — the layout is a tested invariant per 5 unit tests including `at_least_three_asteroids_within_50m_of_origin` (the load-bearing precondition for 3.5's spawn-at-origin choice).
  - [x] **Do NOT touch the `spawn_arena_zone` function signature** — Story 3.6+ continues to pass through `tuning_assets`/`tuning_handle`/etc.; only the Camera3d-spawn-block lines are removed.
  - [x] **Verify post-edit:** `cargo check` produces 0 warnings/errors. The deleted Camera3d code path is the only Camera3d in `OnEnter(Arena)` until Task 1's `spawn_player_ship` adds the cockpit Camera3d (FlightSystems::Setup runs after ArenaSystems::Setup per AC #3 chain). **Brief 1-frame "no Camera3d in scene" race:** does NOT happen because both systems run within the same `OnEnter(Arena)` schedule (Bevy runs all OnEnter systems before the next render frame). Even if it did, Bevy 0.18 tolerates a frame without a Camera3d (warning, not panic — and the FlightSystems::Setup chain guarantees no warning either way).

- [x] **Task 3: Wire FlightPlugin in `src/main.rs`** (AC: #2)
  - [x] Add `mod flight;` in alphabetical order with the other top-level `mod` lines at `src/main.rs:8-15`. Rustfmt will land it between `mod arena;` and `mod logging;` (alphabetical sort). **Do NOT** manually re-order; let rustfmt handle it.
  - [x] Add `use flight::FlightPlugin;` in alphabetical order with the other `use crate::*` lines at `src/main.rs:17-24`. Rustfmt lands it between `use arena::ArenaPlugin;` and `use logging::init_logging;`.
  - [x] Add `.add_plugins(FlightPlugin)` to the App-builder chain in `src/main.rs`, placed AFTER `.add_plugins(ArenaPlugin)` (line 43) and BEFORE `.add_plugins(PausePlugin)` (line 44). Order rationale: (a) FlightPlugin's `configure_sets((ArenaSystems::Setup, FlightSystems::Setup).chain())` requires ArenaPlugin to have already declared `ArenaSystems::Setup` (which happens in `ArenaPlugin::build`); (b) PausePlugin doesn't depend on FlightPlugin so either order works — placing Flight BEFORE Pause matches the on-screen "flight then pause-overlay" rendering layer mental model, and matches the Story 3.4 precedent of "PausePlugin last in the gameplay-plugins block".
  - [x] **Verify post-edit:** the resulting `main.rs` plugin-registration block reads (approximately):
    ```rust
    .add_plugins(TuningPlugin)
    .add_plugins(VisualPlugin)
    .add_plugins(UiPlugin)
    .add_plugins(ArenaPlugin)
    .add_plugins(FlightPlugin)
    .add_plugins(PausePlugin)
    ```

- [x] **Task 4: Local verification sweep — full `feedback_full_build_output.md` discipline** (AC: #8)

  Per Till's memory `feedback_full_build_output.md`: **`cargo check` exit-0 + tail is NOT proof of correctness**. Capture each command's full output to a log file, then grep for `warning:|error:` and confirm the count is **0**.

  - [x] `cargo check 2>&1 | tee /tmp/story-3-5-check.log` — confirm `grep -cE 'warning:|error:' /tmp/story-3-5-check.log` returns **0**.
  - [x] `cargo build 2>&1 | tee /tmp/story-3-5-build.log` — confirm grep returns **0**.
  - [x] `cargo test 2>&1 | tee /tmp/story-3-5-test.log` — confirm grep returns **0** AND the summary line reads `test result: ok. N passed; 0 failed; 0 ignored; ...` where N ≥ 21.
  - [x] `cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-3-5-clippy.log` — confirm grep returns **0**.
  - [x] `cargo fmt --all -- --check 2>&1 | tee /tmp/story-3-5-fmt.log` — confirm exit code 0. If fmt drift exists, run `cargo fmt --all`, re-stage, and re-run `--check`.
  - [x] `cargo build --release 2>&1 | tee /tmp/story-3-5-release.log` — confirm grep returns **0**. Allow 4–6 min wall time on the LTO=fat + codegen-units=1 release build.
  - [x] **Runtime smoke** — `RUST_LOG=info,wgpu=warn,naga=warn,avian3d=info cargo run 2>&1 | tee /tmp/story-3-5-run.log` — let the game reach Arena, then exercise:
    - (a) Visually confirm the placeholder cuboid is visible from the cockpit POV (depending on the Y=0.6 / Z=0.5 camera offset, the rear edge of the cuboid may be visible at the bottom-rear of the FOV — this is expected; it grounds the "I'm in a ship" feeling).
    - (b) Visually confirm ≥3 asteroids are visible in the foreground (close-cluster at z=-25 to -42).
    - (c) Press Esc → "PAUSED — Esc to resume" overlay appears (Story 3.4 still works).
    - (d) Press Esc → resume to Arena (Story 3.4 still works).
    - (e) Alt-Tab away → silent pause (Story 3.4 still works).
    - (f) Alt-Tab back → resume (Story 3.4 still works).
    - (g) Wait 5 seconds in Arena with no input → confirm the ship does NOT drift (camera does not slide; asteroid silhouettes remain in fixed screen positions).
    - Quit the app cleanly (window-close).
  - [x] **Post-runtime grep** — confirm `/tmp/story-3-5-run.log`:
    - `grep -c 'entered Loading'` → **1**
    - `grep -c 'entered MainMenu'` → **1**
    - `grep -c 'entered Arena'` → ≥ **1** (1 initial + however many resume cycles you triggered)
    - `grep -c 'spawned PlayerShip'` → ≥ **1** (matches `entered Arena` count if FlightSystems::Setup runs every entry)
    - `grep -cE 'panic|backtrace|FATAL'` → **0**
    - `grep -ci 'ambiguous.*camera.*order'` → **0** (this is THE regression signal that the stand-in Camera3d removal worked)
    - `grep -cE 'ERROR.*avian|WARN.*Avian'` → **0**

- [x] **Task 5: Update `_bmad-output/implementation-artifacts/deferred-work.md`** (AC: #8)
  - [x] Mark deferred-work.md:160 entry (`Stand-in Camera3d in spawn_arena_zone must be replaced by Story 3.5's cockpit camera`) as **✅ RESOLVED 2026-XX-XX by Story 3.5** with a brief note: stand-in deletion + cockpit Camera3d as PlayerShip child + verification grep `ambiguous.*camera.*order = 0`. Format follows the precedent at deferred-work.md:71-72, :91, :94, :110, :143-144, :154.
  - [x] **Optionally** add a new "Deferred from: 3-5-..." section if the implementation surfaces any forward-compat concerns. Candidates that may surface:
    - Cockpit Camera3d local Transform tuning (0.6 Y / 0.5 Z is a placeholder — a future "cockpit comfort pass" story will iterate based on motion-sickness playtest)
    - Ship mesh swap from `Cuboid` placeholder to a real glTF cockpit mesh (architecture.md:622-625 reserves `assets/meshes/ship/cockpit.gltf` slot — likely landing in Epic 4–5 polish)
    - PlayerShip mass / moment-of-inertia tuning (Avian uses default-density inferred from `Collider::sphere(2.0)`; Stories 3.6/3.7 will need explicit `Mass` / `Inertia` components if thrust calibration drifts — flag here so 3.6 author isn't surprised)
    - `SemanticAccent::PlayerOwned` wiring on PlayerShip (3.5 uses Neutral; 4.5 owns the PlayerOwned variant per deferred-work.md:162 — the PlayerShip will be one of the entities 4.5 retroactively re-tints)
    - Don't add entries proactively unless impl actually surfaces the concern; YAGNI per `karpathy-guidelines.md`.

- [x] **Task 6: Sprint-status bookkeeping + commit/push (NOT YET — await Till's authorization)** (per Story 3.4 precedent)
  <!-- Sprint-status flips + Dev Agent Record populated; commit/push subtasks below remain unchecked pending Till's explicit authorization (Stories 3.1/3.2/3.3/3.4 cadence). -->

  - [x] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - `3-5-cockpit-camera-playership-entity: ready-for-dev → in-progress` (when dev work starts)
    - `3-5-cockpit-camera-playership-entity: in-progress → review` (when local verification passes per Task 4)
    - `last_updated:` bump to current date with brief note (e.g., `(Story 3.5 ready-for-dev → review — cockpit camera + PlayerShip)`)
  - [x] Update this story file's `Status:` field at line 3 from `ready-for-dev → in-progress → review` matching the sprint-status transitions.
  - [x] Populate the `## Dev Agent Record` section below: `Agent Model Used`, `Debug Log References` (the 7 commands' grep counts table — see Story 3.4 Dev Agent Record format), `Completion Notes List` (one bullet per AC #1–#9), `File List` (Added: `src/flight/mod.rs`; Modified: `src/main.rs`, `src/arena/zone.rs`, `sprint-status.yaml`, this file, `deferred-work.md`).
  - [ ] **Commit 1 (feat):** stage `src/flight/mod.rs`, `src/main.rs`, `src/arena/zone.rs`. Message: `feat: cockpit camera + PlayerShip entity (Story 3.5)`. **DO NOT COMMIT UNLESS TILL AUTHORIZES** — per Stories 3.1/3.2/3.3/3.4 precedent.
  - [ ] **Commit 2 (bmad):** stage `_bmad-output/implementation-artifacts/sprint-status.yaml`, `_bmad-output/implementation-artifacts/3-5-cockpit-camera-playership-entity.md`, `_bmad-output/implementation-artifacts/deferred-work.md`. Message: `bmad: story 3.5 ready-for-dev → review (cockpit camera + PlayerShip)`. **DO NOT COMMIT UNLESS TILL AUTHORIZES.**
  - [ ] **DO NOT push.** Push happens only after explicit authorization, AND only after Story 3.5 code review (`bmad-code-review`) passes per Story 3.4 precedent at `commit c923e09`.

## Dev Notes

### Architecture compliance

- **Plugin home:** `src/flight/mod.rs` per architecture.md:558-563 (canonical FR1–FR8 location). The directory is `src/flight/`, NOT `src/ship/` or `src/player/` — naming locked by architecture.md.
- **Plugin name:** `FlightPlugin` per architecture.md:646 plugin-boundaries table.
- **SystemSet name:** `FlightSystems` per architecture.md:327 ("`<Feature>Systems` enum"). Story 3.5 declares `FlightSystems::Setup`; later stories add `Input`, `Physics`, `PostPhysics` per architecture.md:411-412 example.
- **Marker naming:** `PlayerShip` (single-noun PascalCase per architecture.md:322, "Prefer one-word when possible"). `CockpitCamera` (two-word PascalCase, but justified — disambiguates from future debug/photo-mode cameras).
- **Spawn-system naming:** `spawn_player_ship` (snake_case verb-phrase per architecture.md:323). NOT `setup_player_ship` or `init_ship` — verb prefix should describe the ECS action (`spawn`, `apply`, `detect`, `bank`).
- **Cleanup pattern:** `cleanup_on_exit::<ArenaEntity>` from `src/arena/mod.rs:32-36` already handles PlayerShip despawn on `OnExit(Arena)` — no new cleanup system needed. The child Camera3d is auto-despawned via Bevy 0.18 `ChildOf` linked-despawn cascade.
- **Cross-plugin ordering:** `(ArenaSystems::Setup, FlightSystems::Setup).chain()` configured in FlightPlugin per architecture.md:413-414. Forbidden: `.after(spawn_arena_zone)` (architecture.md:415).
- **No ECS god-struct:** PlayerShip is a single-purpose marker; later stories add `Thrusters`, `InertialDampener`, `Boost` as separate components per architecture.md:560 + :461.

### Library / framework specifics — Bevy 0.18 + Avian 0.6 (in-codebase precedent)

- **Camera3d / Mesh3d / MeshMaterial3d (NOT bundles):** Bevy 0.18 deprecated `*Bundle` types in favor of required-components. Use `Camera3d::default()`, `Mesh3d(handle)`, `MeshMaterial3d(handle)` — proven pattern at `src/arena/zone.rs:96-98` (asteroid spawn). A bundle pattern would emit clippy warning under `-D warnings`.
- **`Cuboid::new(width, height, depth)` mesh primitive:** Bevy 0.18 idiom. Returns a `Cuboid` shape; pass to `meshes.add(Cuboid::new(...))` to get a `Handle<Mesh>`. The `.mesh()` builder is for parameterized primitives (e.g., `Sphere::new(r).mesh().ico(2)`); `Cuboid` doesn't need it.
- **`with_children` builder pattern:** `commands.spawn((parent_components,)).with_children(|parent| { parent.spawn((child_components,)); })`. Auto-attaches `ChildOf` for transform propagation + linked-despawn. Proven at `src/pause/mod.rs:159-179` (pause overlay child Text under parent Node).
- **Avian `RigidBody::Dynamic` + `Collider::sphere(r)`:** standard prelude imports per `avian3d::prelude::{Collider, RigidBody}` (proven at `src/arena/zone.rs:4`). Static asteroids use `RigidBody::Static`; the player ship uses `RigidBody::Dynamic` because it must integrate forces from Stories 3.6/3.7.
- **Avian `LinearVelocity` + `AngularVelocity`:** prelude re-exports per `avian3d::prelude::{LinearVelocity, AngularVelocity}` (same precedent as `Physics` + `PhysicsTime` import in Story 3.4 `src/pause/mod.rs:4`). Tuple-struct around `Vec3`. Default = zero; explicit `LinearVelocity(Vec3::ZERO)` documents the no-drift invariant at the call site.
- **Bevy transform propagation handles parent→child:** the child Camera3d's `GlobalTransform` is computed each frame as `PlayerShip.GlobalTransform * Camera3d.LocalTransform` by Bevy's standard `TransformPlugin`. NO custom "camera follows ship" system. NO Avian-specific wiring for the camera (Avian only writes to RigidBody-bearing entities; the child is unaffected by Avian directly but inherits via standard transform cascade).
- **Avian gravity-zero:** already set in `src/main.rs:39` (`.insert_resource(Gravity(Vec3::ZERO))`) by Story 3.2. PlayerShip inherits zero gravity automatically — no per-entity override needed.

### File structure requirements

```
src/
├── flight/                  # NEW (Story 3.5)
│   └── mod.rs               # FlightPlugin + PlayerShip + CockpitCamera + spawn_player_ship
├── arena/
│   ├── mod.rs               # UNCHANGED (cleanup_on_exit::<ArenaEntity> already handles PlayerShip)
│   └── zone.rs              # MODIFIED — stand-in Camera3d block deleted (5–6 lines removed)
├── main.rs                  # MODIFIED (+3 lines: mod flight; use flight::FlightPlugin; add_plugins(FlightPlugin))
├── pause/                   # UNCHANGED
├── tuning/                  # UNCHANGED
├── ui/                      # UNCHANGED
├── visual/                  # UNCHANGED
├── state.rs                 # UNCHANGED (unless `Copy` derive added — see Deviation guidance below)
├── splash.rs                # UNCHANGED
└── logging.rs               # UNCHANGED
```

The `src/flight/mod.rs` single-file structure is intentional — architecture.md:558-563 prescribes sub-files (`components.rs`, `input.rs`, `physics.rs`, `camera.rs`) for the eventual full FlightPlugin, but Story 3.5's surface (one spawn system, one parent + one child) fits in 120–180 lines comfortably. Splitting prematurely violates YAGNI. Stories 3.6 / 3.7 / 3.8 will introduce `flight/input.rs` + `flight/physics.rs` when the file would otherwise exceed ~250 lines (mirrors Story 3.4 Dev Notes "Anti-pattern #15: Adding a `pause_overlay.rs` sibling file when 150 lines fits in `mod.rs`").

### Optional `Copy` derive on GameState — deviation guidance

Story 3.4's Deviation #1 (`src/pause/mod.rs` Dev Agent Record) documents that `GameState` lacks `Copy`, forcing `paused_from.as_deref().map_or(GameState::Arena, |p| p.0.clone())` workarounds. Deferred-work.md:194 prescribes "any future story that touches `src/state.rs` should append `Copy` to the derive list".

**Story 3.5 does NOT need to touch `src/state.rs`** — `spawn_player_ship` only reads `GameState` implicitly via `OnEnter(GameState::Arena)` registration; no Resource access, no clone path. **Recommended:** leave `src/state.rs` untouched; AC #8's git-status manifest forbids touching it. The `Copy` derive remains a deferred chore for whoever next legitimately needs `Copy` on `GameState`.

If during implementation the dev finds an unexpected need to clone `GameState`, prefer the `.clone()` workaround over touching `src/state.rs` — keep this story's blast radius minimal.

### Testing standards

Per architecture.md:351-354:
- **Co-located** `#[cfg(test)] mod tests { ... }` at the bottom of each module file.
- **Pure-logic modules first-class test targets;** integration tests deferred post-M3.

Story 3.5's `spawn_player_ship` is integration-test-shaped — exercising it requires a Bevy `App` bootstrap (state setup, asset loading, system scheduling). That's out of scope. The runtime smoke (Task 4) is the de-facto integration test.

**Test budget: 0–2** (per Story 3.4 precedent on trivial-data invariants):
- Add 0 tests if you don't see a testable invariant. The runtime smoke is sufficient for AC #8 evidence.
- If clippy `-D warnings` complains about `FlightSystems::Setup` being unused (it shouldn't — it's used in `configure_sets` + `in_set`), add a no-op consumer test.
- Do NOT re-test Story 3.3's `at_least_three_asteroids_within_50m_of_origin` invariant in `src/flight/mod.rs` — it lives at the asteroid-layout source-of-truth in `src/arena/zone.rs:165-172`; DRY.

**Net post-3.5 test count target: ≥ 21** (= 21 baseline + 0–2 new). AC #8 enforces N ≥ 21.

### Anti-patterns to avoid (catalogued from Stories 1.5–3.4 review precedent)

1. **Story-id references in module doc-comments** — Story 1.5 review patch BH8 + Story 3.2 patch (commit `5134b3c`) removed all "Story X.Y..." references from `src/<feature>/mod.rs` doc lines. Module docs describe what the module owns, not when it was added. **Do NOT** write `//! Story 3.5 introduces FlightPlugin ...`.
2. **Wildcard imports beyond `bevy::prelude::*`** — explicit imports per architecture.md naming-discipline (rustfmt won't enforce this; it's a style call). `use avian3d::prelude::{AngularVelocity, Collider, LinearVelocity, RigidBody};` not `use avian3d::prelude::*;`.
3. **Bundle types (`Camera3dBundle`, `MaterialMeshBundle`)** — deprecated in Bevy 0.18; use required-components. Clippy `-D warnings` will catch this.
4. **`.after(spawn_arena_zone)` for system ordering** — architecture.md:415 forbids `.after(specific_function)`. Use `.in_set(FlightSystems::Setup)` + `configure_sets((ArenaSystems::Setup, FlightSystems::Setup).chain())`.
5. **Custom "camera follows ship" system** — Bevy's `TransformPlugin` already does this for parent-child hierarchies. Adding a custom system creates a one-frame lag race + duplicate work.
6. **Over-tagging child Camera3d with `ArenaEntity`** — deferred-work.md:92-94 (Story 2.2 precedent). The parent PlayerShip carries `ArenaEntity`; the child Camera3d does NOT need it. Bevy 0.18 `ChildOf` linked-despawn cascade handles cleanup. Tagging the child causes "tried to despawn missing entity" warnings during cleanup loop iteration (Bevy 0.18 returns Result, no panic, but log noise).
7. **Querying-and-despawning the stand-in Camera3d at runtime** — deferred-work.md:160 explicitly offers (a) "query and despawn at runtime" OR (b) "delete the stand-in source code entirely". This story chooses (b) per AC #1 — cleaner, no runtime query cost, no possibility of the stand-in being missed.
8. **Spawning at non-origin coordinates** — Story 3.3's `at_least_three_asteroids_within_50m_of_origin` test is the load-bearing precondition for AC #7; spawning anywhere else invalidates the precondition without a corresponding new layout test. Origin is the contract.
9. **Adding `Mass` / `Inertia` components proactively** — Avian 0.6 infers reasonable defaults from `Collider::sphere(2.0)` + the standard density. Stories 3.6 / 3.7 may need explicit Mass tuning when thrust calibration drifts; deferring that to those stories keeps Story 3.5's surface focused on the spawn-and-camera contract.
10. **Touching `src/state.rs` to add `Copy` derive** — see "Optional `Copy` derive" section above. Deferred chore; not 3.5's scope.
11. **Touching `Cargo.toml`** — no new deps. All Avian + Bevy + bevy_mod_outline + ToonMaterial APIs needed for 3.5 are already pinned.
12. **Splitting `src/flight/` into multiple files prematurely** — single `mod.rs` fits 120–180 lines. Sub-files arrive when 3.6 / 3.7 push the file past ~250 lines (mirror Story 3.4 anti-pattern #15).
13. **Adding any flight-input scaffolding** — that's Story 3.6's scope. NO `leafwing-input-manager` import, NO `FlightAction` enum, NO `InputManagerBundle<FlightAction>` on the PlayerShip in 3.5. The PlayerShip spawns "deaf" (no input wiring) and stays stationary — Story 3.6 adds `InputManagerBundle<FlightAction>` to the existing PlayerShip via a `.insert_bundle` or by extending the spawn tuple at that time.
14. **Using `SemanticAccent::PlayerOwned` for the PlayerShip tint** — deferred-work.md:162 explicitly assigns `SemanticAccent::{Enemy, Salvage, Hazard, PlayerOwned}` wiring to Story 4.5. 3.5 uses `SemanticAccent::Neutral` (matches asteroids); 4.5 retroactively re-tints the PlayerShip + projectiles to `PlayerOwned`. This keeps 4.5 a coherent "all four remaining variants land at once" story instead of fragmenting it.

### Logging discipline

Per architecture.md:376-383:
- `info!` for lifecycle events: `info!("spawned PlayerShip at origin with cockpit Camera3d child");` — one line per OnEnter consequence.
- NO `warn!` unless something unexpected happens (the `tuning_opt.is_none()` cold-start path warns — mirrors `src/arena/zone.rs:52`).
- NO `debug!` / `trace!` in 3.5; Stories 3.6 / 3.7 / 3.8 will add per-frame thrust / rotation diagnostics at `debug!` level.

### Project Structure Notes

- **Alignment with unified project structure:** `src/flight/mod.rs` is the canonical location per architecture.md:558. The flat single-file form (no `components.rs` / `input.rs` / `physics.rs` sub-files yet) is a deliberate YAGNI deferral — see "File structure requirements" above.
- **Detected variances:** none. Story 3.5 follows established Story 3.2 / 3.3 / 3.4 patterns exactly.

## Previous Story Intelligence (Story 3.4 — Pause on focus loss + Pause Menu Stub)

Story 3.4 is the most recent reference for the development pattern. Key learnings to inherit:

- **`.clone()` workaround for `GameState`** (3.4 Deviation #1): if any 3.5 code path needs to clone `GameState`, use `.as_deref().map_or(default, |s| s.clone())` rather than touching `src/state.rs`. Deferred-work.md:194 owns the `Copy` derive chore.
- **Avian prelude trait imports** (3.4 Deviation #2): when a method "doesn't exist" on an Avian type, suspect a missing trait import from `avian3d::prelude::*`. Story 3.5 doesn't use any Avian extension traits (just structs `RigidBody`, `Collider`, `LinearVelocity`, `AngularVelocity`), but the pattern transfers if any future Avian-specific method call surfaces.
- **`MessageReader` not `EventReader` in Bevy 0.18** (3.4 Five-key constraint #2): Story 3.5 doesn't read any Bevy events directly — only spawns. But if a future iteration adds an event-driven path, remember the Event/Message split.
- **`commands.insert_resource(...)` over `ResMut<T>`** (3.4 idiom): Story 3.5 doesn't manipulate Resources at runtime — only Asset handles. Pattern is N/A here but worth knowing.
- **Per-command grep verification harness** (3.4 Task 4): Story 3.5 mirrors this exactly per AC #8 + Task 4. The 7-command + runtime-smoke sweep is the canonical local-verification pattern.
- **2-commit pattern (feat + bmad)** (3.4 commit precedent — `799950f` + `68fcd00`): Story 3.5's Task 6 mirrors. Commits and pushes await Till's authorization.
- **`PauseSystems::Detect` declared but only one variant used** (3.4 Plugin skeleton): same pattern for `FlightSystems::Setup` in 3.5. OK to declare a SystemSet enum with only one variant when later stories will extend.
- **Camera2d at `order: 1`** (3.4 spec) reserved for pause overlay → no conflict with 3.5's Camera3d (Camera2d and Camera3d use different render passes; no order conflict). Deferred-work.md:192's "no slot-reservation convention" concern remains active but does NOT bite Story 3.5.

## Git intelligence summary

Recent commit history (`git log --oneline -10`):
- `c923e09` bmad: story 3.4 review → done (code review passed, 0 patches, 2 new deferred items)
- `68fcd00` bmad: story 3.4 ready-for-dev → review (pause on focus loss + Esc stub)
- `799950f` feat: pause on focus loss + Esc menu stub (Story 3.4) ← **canonical predecessor commit; see Story 3.4 dev pattern**
- `401e92e` bmad: story 3.3 review → done (asteroid field + DirectionalLight, 2 review patches)
- `bc40a45` feat: hand-designed Arena asteroid field + light (Story 3.3) ← **stand-in Camera3d introduced here at zone.rs:64-69; Story 3.5 deletes**
- `5134b3c` fix: remove story-id reference from arena mod doc (review patch) ← **module-doc style guidance**
- `1225afe` bmad: story 3.2 review → done
- `e896b69` bmad: story 3.2 ready-for-dev → review
- `d5a3681` feat: Avian physics foundation + ArenaPlugin skeleton (Story 3.2) ← **ArenaSystems::Setup declared here; cleanup_on_exit::<T> generic introduced**
- `b2fbd36` bmad: story 3.2 backlog → ready-for-dev

**Patterns extracted:**
- **2-commit cadence per story:** `feat:` for code + `bmad:` for spec/state metadata. Story 3.5 follows.
- **Module patterns introduced ahead of consumers:** Story 3.2's `cleanup_on_exit::<T>` generic was deliberately designed for future markers (3.4 used it, 3.5 reuses for `<ArenaEntity>` cascade — well, technically for the parent PlayerShip; the child Camera3d doesn't need it because of `ChildOf` linked-despawn).
- **No Cargo.toml churn since Story 3.2:** all 3.3 / 3.4 stories used existing pinned deps. Story 3.5 continues the streak — no new deps.

## Latest tech information (Bevy 0.18 + Avian 0.6 — in-codebase precedent as source of truth)

Story 3.5 introduces no new external dependencies. Every API surface used is empirically validated in Stories 3.2 / 3.3 / 3.4 codepaths and CI-tested across all three platforms (Windows / Linux / macOS) per `.github/workflows/ci.yml`:

- `Camera3d::default()` + child of parent — pattern at `src/arena/zone.rs:65-69` (stand-in, being replaced); Bevy 0.18 required-components auto-insert `Transform`, `GlobalTransform`, `Camera`, `Projection`, render-graph wiring. **Note:** the stand-in being deleted by Story 3.5 was empirically known to render fine — the new cockpit Camera3d's only difference is the `ChildOf` parent link, which Bevy's `TransformPlugin` handles transparently.
- `Mesh3d(...)` + `MeshMaterial3d(...)` — pattern at `src/arena/zone.rs:96-98`. Story 3.5 uses identical wrappers around the placeholder cuboid mesh + `ToonMaterial` handle.
- `RigidBody::Dynamic` + `Collider::sphere(...)` — Avian 0.6 prelude per `avian3d::prelude::{Collider, RigidBody}`. Pattern at `src/arena/zone.rs:4` (import) + `:100-101` (asteroid spawn). Story 3.5 swaps `Static → Dynamic` and adds zero-velocity initial state. Dynamic + zero gravity (set in `main.rs:39`) → ship floats stationary at spawn.
- `LinearVelocity` + `AngularVelocity` — Avian 0.6 prelude (verified via `avian3d/src/lib.rs:550-555` re-exports per Story 3.4 trail). Tuple structs around `Vec3`. Default = zero.
- `bevy_mod_outline::OutlineVolume` — pattern at `src/arena/zone.rs:55-62, 102` (per-asteroid outline). Story 3.5 reuses the same `outline_volume()` builder pattern.
- `.with_children(|parent| { parent.spawn(...) })` — Bevy 0.18 builder pattern. Pattern at `src/pause/mod.rs:159-179` (pause overlay child Text). Story 3.5 reuses identically for child Camera3d under PlayerShip.
- `Cuboid::new(width, height, depth)` mesh primitive — Bevy 0.18 idiom. NOT empirically used in this codebase yet (Story 3.3 uses `Sphere::new(r).mesh().ico(2)` for asteroids), but documented in Bevy 0.18's `bevy_math::primitives::Cuboid` — direct constructor returns a `Cuboid` shape implementing `Meshable`, so `meshes.add(Cuboid::new(...))` works directly.
- `configure_sets((SetA, SetB).chain())` cross-plugin — pattern at `src/visual/mod.rs:23-26` (`VisualSystems::Setup`) + `src/arena/mod.rs:20-23` (`ArenaSystems::Setup`) + `src/pause/mod.rs:31-49` (`PauseSystems::Detect`). Cross-plugin chaining (Set A from Plugin X, Set B from Plugin Y) is approved per architecture.md:413-414 example.

**No version drift expected:** Cargo.toml is pinned at `bevy = "0.18"`, `avian3d = "0.6"`, `bevy_mod_outline = "0.12"`. Story 3.5 does NOT bump any of these — version bumps are M4 / M6 / M9 milestone gate concerns per PRD Phase-3 + architecture.md:177.

## Project context reference

- **Memory:** `MEMORY.md` (auto-loaded at session start) — Till's user memories include `feedback_full_build_output.md` (the per-command-grep verification discipline), `feedback_compact_review_style.md` (compact responses), `feedback_staged_rollout.md` (staged-rollout preference, justifies the lean Story-3.5 surface area).
- **Brainstorming canon:** `_bmad-output/brainstorming/brainstorming-session-2026-04-21-1114.md` — original concept doc; cockpit-only commitment lives there + in PRD FR8.
- **Architecture canon:** `_bmad-output/planning-artifacts/architecture.md` — single-file authoritative architecture per memory `reference_brainstorming_doc.md`.
- **Sprint plan:** `_bmad-output/implementation-artifacts/sprint-status.yaml` — Story 3.5 is the next backlog item after 3.4 done.
- **Deferred work:** `_bmad-output/implementation-artifacts/deferred-work.md` — Story 3.5 resolves entry at line 160 (stand-in Camera3d), inherits open entries 158 (`VisualSystems::Setup` empty no-op — out of scope), 162 (`SemanticAccent::PlayerOwned` wiring → Story 4.5), 166 (asteroid layout — DO NOT MODIFY), 168 (splash race — out of scope), 192 (`Camera2d order:1` slot reservation — N/A for 3.5's Camera3d).

### References

- [Source: [`_bmad-output/planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md:110-138`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md)] — Story 3.5 epic spec (User story + 4 BDD ACs + epic context).
- [Source: [`_bmad-output/planning-artifacts/prd.md:507`](../planning-artifacts/prd.md)] — FR8 capability statement: "Player views gameplay exclusively through a first-person cockpit view; no external camera toggle is available during active gameplay."
- [Source: [`_bmad-output/planning-artifacts/prd.md:500-502`](../planning-artifacts/prd.md)] — FR1 (keyboard+mouse input — 3.6 not 3.5), FR2 (6-direction translation — 3.6), FR3 (3-axis rotation — 3.7) — listed for context, not in 3.5's scope.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:210`](../planning-artifacts/architecture.md)] — `GameState::Arena` declared as top-level state (already live since Story 1.6).
- [Source: [`_bmad-output/planning-artifacts/architecture.md:240`](../planning-artifacts/architecture.md)] — Avian `FixedUpdate` at 60 Hz (Story 3.5's `RigidBody::Dynamic` ship integrates here).
- [Source: [`_bmad-output/planning-artifacts/architecture.md:343-350`](../planning-artifacts/architecture.md)] — Plugin-per-feature module pattern + `<Feature>Systems` SystemSet.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:411-415`](../planning-artifacts/architecture.md)] — SystemSet ordering via `configure_sets`+`.chain()`; `.after(specific_function)` forbidden.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:415-420`](../planning-artifacts/architecture.md)] — `cleanup_on_exit::<T>` pattern via state-scoped markers.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:558-563`](../planning-artifacts/architecture.md)] — `src/flight/mod.rs` + `FlightPlugin` + `FlightSystems` SystemSet + sub-file structure.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:563`](../planning-artifacts/architecture.md)] — `src/flight/camera.rs` planned location for cockpit camera (Story 3.5 keeps everything in `mod.rs`; sub-file split is a 3.6/3.7 concern).
- [Source: [`_bmad-output/planning-artifacts/architecture.md:646`](../planning-artifacts/architecture.md)] — `FlightPlugin` plugin-boundaries table entry: owns "Thrusters, dampener, cockpit Camera3d" (3.5 lands the cockpit Camera3d portion).
- [Source: [`_bmad-output/planning-artifacts/architecture.md:680`](../planning-artifacts/architecture.md)] — FR8 location mapping: `src/flight/camera.rs`.
- [Source: [`Cargo.toml:8-12`](../../Cargo.toml)] — bevy 0.18 + avian3d 0.6 + bevy_mod_outline 0.12 + bevy_kira_audio 0.25 + leafwing-input-manager 0.20 pinned versions.
- [Source: [`src/main.rs`](../../src/main.rs)] — current plugin-registration block (post-3.4; +3 lines for FlightPlugin in 3.5).
- [Source: [`src/main.rs:39`](../../src/main.rs)] — `.insert_resource(Gravity(Vec3::ZERO))` — zero-g world (PlayerShip floats stationary at spawn).
- [Source: [`src/state.rs:7-19`](../../src/state.rs)] — `GameState` enum with `Arena` variant; `OnEnter(GameState::Arena)` is the canonical 3.5 entry hook.
- [Source: [`src/arena/mod.rs:11-13, 20-24`](../../src/arena/mod.rs)] — `ArenaSystems::Setup` SystemSet + `cleanup_on_exit::<ArenaEntity>` registration on `OnExit(Arena)` — 3.5's PlayerShip + child Camera3d auto-cleaned by these.
- [Source: [`src/arena/mod.rs:32-36`](../../src/arena/mod.rs)] — `cleanup_on_exit::<T: Component>` generic; reused by 3.5 transitively (parent PlayerShip carries `ArenaEntity`; child Camera3d auto-despawned via Bevy `ChildOf` linked-despawn cascade).
- [Source: [`src/arena/zone.rs:14-39`](../../src/arena/zone.rs)] — `ASTEROIDS` const-array layout + comment explaining the deliberate origin clearance for 3.5 spawn.
- [Source: [`src/arena/zone.rs:41-105`](../../src/arena/zone.rs)] — `spawn_arena_zone` system; the cold-start tuning fallback pattern at lines 48-54 mirrors into Story 3.5's `spawn_player_ship`.
- [Source: [`src/arena/zone.rs:55-62`](../../src/arena/zone.rs)] — `outline_volume()` closure pattern; Story 3.5 mirrors for the PlayerShip `OutlineVolume`.
- [Source: [`src/arena/zone.rs:64-69`](../../src/arena/zone.rs)] — **stand-in Camera3d block being deleted by 3.5** per AC #1.
- [Source: [`src/arena/zone.rs:96-98`](../../src/arena/zone.rs)] — `Mesh3d(mesh)` + `MeshMaterial3d(material)` Bevy 0.18 required-components precedent.
- [Source: [`src/arena/zone.rs:100-101`](../../src/arena/zone.rs)] — `RigidBody::Static` + `Collider::sphere(radius)` Avian precedent (3.5 swaps Static → Dynamic).
- [Source: [`src/arena/zone.rs:165-172`](../../src/arena/zone.rs)] — `at_least_three_asteroids_within_50m_of_origin` test enforcing 3.5's spawn-at-origin precondition.
- [Source: [`src/visual/toon_material.rs:11-22`](../../src/visual/toon_material.rs)] — `ToonMaterial` field signature (`tint: LinearRgba`, etc.) + `Default` impl; 3.5 mirrors `tint: color_for(...).into(), ..default()` from 3.3.
- [Source: [`src/visual/palette.rs:11-18, 20-28`](../../src/visual/palette.rs)] — `SemanticAccent::Neutral` (3.5 use) + `color_for()` lookup + `PlayerOwned` (Story 4.5 re-tints — 3.5 must NOT use).
- [Source: [`src/pause/mod.rs:159-179`](../../src/pause/mod.rs)] — `.with_children(|parent| parent.spawn(...))` Bevy 0.18 hierarchy builder precedent.
- [Source: [`src/tuning/mod.rs:17-19`](../../src/tuning/mod.rs)] — `TuningHandle(Handle<TuningConfig>)` Resource definition; 3.5 reads via the `tuning_assets.get(tuning_handle.0.id())` pattern from `src/arena/zone.rs:48-54`.
- [Source: [`_bmad-output/implementation-artifacts/3-2-avian-physics-foundation-arena-state-skeleton.md` Dev Notes line 92](./3-2-avian-physics-foundation-arena-state-skeleton.md)] — `cleanup_on_exit::<T>` generic was designed to serve future markers; PlayerShip → ArenaEntity cascade benefits transitively.
- [Source: [`_bmad-output/implementation-artifacts/3-3-hand-designed-arena-zone-with-static-asteroid-field.md`](./3-3-hand-designed-arena-zone-with-static-asteroid-field.md)] — Story 3.3 Forward-compat hand-off: stand-in Camera3d explicitly flagged for 3.5 replacement; origin corridor explicitly preserved for 3.5 PlayerShip spawn.
- [Source: [`_bmad-output/implementation-artifacts/3-4-pause-on-focus-loss-pause-menu-stub.md` Dev Agent Record](./3-4-pause-on-focus-loss-pause-menu-stub.md)] — verification-harness format + 2-commit cadence + deviation-documentation pattern; Story 3.5 mirrors directly.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:160`](./deferred-work.md)] — Stand-in Camera3d removal contract (Story 3.5 resolves this entry per AC #1 + Task 5).
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:162`](./deferred-work.md)] — `SemanticAccent::PlayerOwned` wiring assigned to Story 4.5; Story 3.5 must NOT use PlayerOwned (uses Neutral instead).
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:166`](./deferred-work.md)] — Asteroid layout DO-NOT-MODIFY supportive note; 3.5 spawn-at-origin preserves the layout invariant.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:168`](./deferred-work.md)] — Splash cleanup-iteration race; out-of-scope for 3.5.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:184`](./deferred-work.md)] — Generic-cleanup home re-evaluation: 3.5 is NOT a new consumer of `cleanup_on_exit::<T>` (PlayerShip uses `ArenaEntity` which is already a consumer; child Camera3d uses `ChildOf` cascade). Decision-trigger remains Story 3.11 (3rd consumer).
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:194`](./deferred-work.md)] — `GameState` lacks `Copy`; 3.5 leaves untouched per "Optional `Copy` derive" guidance.
- [Source: [`MEMORY.md` → `feedback_full_build_output.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_full_build_output.md)] — verification discipline: exit-0 + tail is NOT proof; per-command grep for `warning:|error:`.
- [Source: [`MEMORY.md` → `feedback_compact_review_style.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_compact_review_style.md)] — Till's compact-review style for the Q&A loop after dev-story.
- [Source: [`MEMORY.md` → `feedback_staged_rollout.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_staged_rollout.md)] — staged-rollout preference; informs 3.5's lean spawn-only surface area (no input wiring, no flight physics — those land separately in 3.6/3.7).
- [Source: avian3d-0.6 prelude — `src/lib.rs:550-555`] — re-exports `RigidBody`, `Collider`, `LinearVelocity`, `AngularVelocity`, `PhysicsTime`, `Physics`, etc.
- [Source: bevy-0.18 `bevy_math::primitives::Cuboid`] — `Cuboid::new(x_length, y_length, z_length)` constructor returns a `Cuboid` implementing `Meshable`; pass directly to `Assets<Mesh>::add`.

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Opus 4.7, 1M context)

### Debug Log References

Local verification sweep (all logs in `/tmp/`):

| Command | Log file | `grep -cE 'warning:\|error:'` | Notes |
|---|---|---|---|
| `cargo check` | `/tmp/story-3-5-check.log` | 0 | 0.25s after touching changed files (cache invalidation forced) |
| `cargo build` (debug) | `/tmp/story-3-5-build.log` | 0 | 4.56s; full rebuild from cache |
| `cargo test` | `/tmp/story-3-5-test.log` | 0 (also 0 `FAILED`) | `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Test count: **21** (= 19 baseline pre-3.4 + 2 from 3.4; +0 from 3.5 — sticking with the 0–2 budget low end since `spawn_player_ship` is integration-test-shaped and the `at_least_three_asteroids_within_50m_of_origin` precondition test already lives in `arena/zone.rs:165-172`) |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-3-5-clippy.log` | 0 | 0.74s; clean (no dead-code complaints — FlightPlugin / FlightSystems / PlayerShip / CockpitCamera / spawn_player_ship all reachable via `add_plugins(FlightPlugin)` registration in main.rs:46) |
| `cargo fmt --all -- --check` | exit code | 0 | No fmt drift; rustfmt left the new file's tuple formatting alone |
| `cargo build --release` | `/tmp/story-3-5-release.log` | 0 | 4m 09s (LTO=fat + codegen-units=1, full re-link); within Story 3.4's 4m 05s benchmark — no regression in release build time |
| `cargo run` runtime smoke | `/tmp/story-3-5-run.log` | n/a | 25 lines total. See runtime-smoke evidence table below + Deviation #1 re: Esc / Alt-Tab not exercised |

**Runtime-smoke evidence** (per AC #8 grep harness — single 17-second run, Loading → MainMenu → Enter → Arena → wait + visual confirm → window-close):

| Marker | Count | Expected |
|---|---|---|
| `entered Loading` | 1 | 1 |
| `entered MainMenu` | 1 | 1 |
| `entered Arena` | 1 | ≥ 1 |
| `spawned PlayerShip` | 1 | ≥ 1 (matches `entered Arena` count — FlightSystems::Setup runs every Arena entry) |
| `panic\|backtrace\|FATAL` | 0 | 0 |
| `ambiguous.*camera.*order` (case-insensitive) | **0** | **0** ← THE regression signal that the stand-in Camera3d removal worked |
| `ERROR.*avian` / `WARN.*Avian` | 0 / 0 | 0 / 0 |
| `paused via Escape` / `resumed via Escape` | 0 / 0 | ≥ 1 / ≥ 1 (NOT exercised — see Deviation #1 below; visually-confirmed-only) |
| `paused on focus loss` / `resumed from focus gain` | 0 / 0 | ≥ 1 / ≥ 1 (NOT exercised — see Deviation #1 below) |
| `simulation clocks paused` / `simulation clocks resumed` | 0 / 0 | ≥ 1 / ≥ 1 (NOT exercised — see Deviation #1 below) |

**Documented (non-3.5-regression) WARNs in run log** — all consistent with prior deferrals, all reappeared unchanged:

1. `bevy_ecs::error::handler: Encountered an error in command ... Entity despawned: ID 87v0 invalid; generation 1` at splash → MainMenu transition — splash-cleanup race per deferred-work.md:75-76, :137, :139, :168 (re-deferred yet again for 3.5; not 3.5-introduced).
2. `wgpu_core::device::resource: The fragment stage "fragment" output @location(0) values are ignored` — pre-existing Story 2.3 ToonMaterial fragment shader output binding warning; not 3.5-introduced.
3. `bevy_winit::state: Skipped event Destroyed for unknown winit Window Id` at window close — known Bevy 0.18 winit-event race per Story 1.6 deferred-work LOW-1; not 3.5-introduced.

**`ArenaEntity` convention check** (per deferred-work.md:152):

```
$ grep -c 'ArenaEntity' src/flight/mod.rs
2
```

= 1 import (`use crate::arena::{ArenaEntity, ArenaSystems};`) + 1 spawn-tuple use on the **parent** PlayerShip only. Child Camera3d does NOT carry the marker (per anti-pattern #6 — over-tagging risk; Bevy 0.18 `ChildOf` linked-despawn cascade handles cleanup transitively). Convention upheld.

**File-size deltas (post-3.5):**

| File | Lines | Delta vs target |
|---|---|---|
| `src/flight/mod.rs` (new) | 93 | Under the 120–180 target (clean implementation; no deviation lines, no optional unit tests added) |
| `src/arena/zone.rs` (modified) | 166 | -7 lines from pre-3.5 (173); 1 line of comment + 5 lines of Camera3d spawn block + 1 trailing blank — matches the -5/-6 prediction |
| `src/main.rs` (modified) | 61 | +3 lines from pre-3.5 (58) — exact match: `mod flight;`, `use flight::FlightPlugin;`, `.add_plugins(FlightPlugin)` |

### Completion Notes List

- **AC #1** ✓ — Stand-in Camera3d block + leading comment at `src/arena/zone.rs:64-69` deleted entirely (6 lines net removal, including the trailing blank). Verified via `grep -c 'Camera3d::default()' src/arena/zone.rs` = 0 (the only `Camera3d::default()` in the codebase now lives in `src/flight/mod.rs:80` as the cockpit child). Runtime smoke `grep -ciE 'ambiguous.*camera|camera.*ambiguous' /tmp/story-3-5-run.log` = **0** — confirms no stand-in / cockpit Camera3d coexistence.

- **AC #2** ✓ — `src/flight/mod.rs` authored (93 lines including 2-line module doc + Plugin impl + spawn system; no optional unit tests added per the 0–2 budget low-end choice). `FlightPlugin` registered in `src/main.rs:46` via `.add_plugins(FlightPlugin)`, placed AFTER `.add_plugins(ArenaPlugin)` (line 45) and BEFORE `.add_plugins(PausePlugin)` (line 47) per AC #2 ordering. `mod flight;` added at `main.rs:9` (rustfmt landed it alphabetically between `mod arena;` and `mod logging;`); `use flight::FlightPlugin;` added at `main.rs:19` (alphabetical between `use arena::ArenaPlugin;` and `use logging::init_logging;`). All four type definitions present: `FlightPlugin` struct, `FlightSystems::Setup` SystemSet enum, `PlayerShip` Component marker, `CockpitCamera` Component marker.

- **AC #3** ✓ — `(ArenaSystems::Setup, FlightSystems::Setup).chain()` configured in `FlightPlugin::build` at `src/flight/mod.rs:34-37` via `app.configure_sets(OnEnter(GameState::Arena), (ArenaSystems::Setup, FlightSystems::Setup).chain())`. `spawn_player_ship` registered `.in_set(FlightSystems::Setup)` at `src/flight/mod.rs:38-41`. `use crate::arena::{ArenaEntity, ArenaSystems};` import at `src/flight/mod.rs:8`. Cross-plugin chain pattern matches architecture.md:413-414 prescription (configure_sets, not `.after(specific_function)`).

- **AC #4** ✓ — `spawn_player_ship` at `src/flight/mod.rs:44-87` spawns exactly one PlayerShip at `Transform::from_xyz(0.0, 0.0, 0.0)` (origin per Story 3.3 cleared-corridor invariant). Spawn tuple includes all 10 components in spec order: `PlayerShip`, `ArenaEntity`, `Mesh3d(meshes.add(Cuboid::new(4.0, 2.0, 6.0)))`, `MeshMaterial3d(materials.add(ToonMaterial { tint: color_for(SemanticAccent::Neutral).into(), ..default() }))`, `Transform::from_xyz(0.0, 0.0, 0.0)`, `outline` (built from `tuning.outline_width` + `tuning.outline_color` mirroring `arena/zone.rs:55-62`), `RigidBody::Dynamic`, `Collider::sphere(2.0)`, `LinearVelocity(Vec3::ZERO)`, `AngularVelocity(Vec3::ZERO)`. Cold-start tuning fallback per Story 3.3 precedent — `if tuning_opt.is_none() { warn!("tuning.ron not loaded at PlayerShip spawn; using TuningConfig defaults"); }` at `src/flight/mod.rs:54-57`. `info!("spawned PlayerShip at origin with cockpit Camera3d child")` lifecycle log fires at `src/flight/mod.rs:86`.

- **AC #5** ✓ — Child Camera3d spawned via `.with_children(|parent| { parent.spawn((Camera3d::default(), CockpitCamera, Transform::from_xyz(0.0, 0.6, 0.5))); })` at `src/flight/mod.rs:78-84`. Builder mirrors `src/pause/mod.rs:159-179` precedent. **NO `ArenaEntity` on the child** (anti-pattern #6 honored — `grep -c 'ArenaEntity' src/flight/mod.rs` = 2 = 1 import + 1 use on the parent only). Bevy 0.18 `ChildOf` linked-despawn cascade handles cleanup on `OnExit(Arena)` via the parent's `cleanup_on_exit::<ArenaEntity>` registration in `src/arena/mod.rs:25-29`.

- **AC #6** ✓ — `LinearVelocity(Vec3::ZERO)` + `AngularVelocity(Vec3::ZERO)` explicit in spawn tuple at `src/flight/mod.rs:75-76`. Till's runtime visual confirmation: ship did not drift over the ~17-second observation window (per "Hat alles funktioniert" feedback after the smoke-run window-close).

- **AC #7** ✓ — Asteroid layout (`src/arena/zone.rs:18-39` `ASTEROIDS` const-array) unchanged → `at_least_three_asteroids_within_50m_of_origin` unit test (in the 21 passing tests per `cargo test`) continues to enforce the precondition. Runtime visual confirmation: asteroids visible from cockpit POV per Till's smoke feedback. NO new unit test added in `src/flight/mod.rs` — the precondition lives at the source-of-truth in `arena/zone.rs:165-172` (DRY).

- **AC #8** ⚠ — All 6 cargo commands report 0 warnings/errors per the per-command grep table above. Test count: 21 (= AC #8 N ≥ 21 floor exactly). `cargo fmt --check` exit 0 with no drift. Git status final delta is **5 expected entries** (M deferred-work.md, M sprint-status.yaml, M src/arena/zone.rs, M src/main.rs, ?? src/flight/, ?? this story file) **plus 1 pre-existing untracked** (?? `.claude/scheduled_tasks.lock` — unchanged from session start, not 3.5-introduced). Runtime smoke evidence per the table above; pause-regression markers (Esc / Alt-Tab paths) NOT exercised — see Deviation #1 below.

- **AC #9** ✓ — No custom "camera follows ship" system added. Verified by inspection: `src/flight/mod.rs` contains exactly one system (`spawn_player_ship`) registered on `OnEnter(Arena)` only — no `Update` systems, no `FixedUpdate` systems. Bevy 0.18 `TransformPlugin` propagation handles parent → child `GlobalTransform` automatically; Avian 0.6 only writes to RigidBody-bearing entities (the parent), and the child Camera3d's `GlobalTransform` is computed as `PlayerShip.GlobalTransform * Camera3d.LocalTransform` every frame by Bevy's standard transform-propagation system.

**Deviations:**

1. **Pause-regression smoke (AC #8 c–f) NOT exercised — visually-confirmed-only.** AC #8 specifies the runtime smoke should exercise (c) Esc-pause overlay appears, (d) Esc-resume, (e) Alt-Tab silent pause, (f) Alt-Tab resume — this is the Story 3.4 regression check. Empirically Till drove the smoke without triggering Esc or Alt-Tab; the runtime log (25 lines) shows zero `paused via Escape` / `paused on focus loss` / `simulation clocks paused` markers (vs. the AC's ≥ 1 expected). When asked whether to re-run the smoke for explicit pause exercise, Till accepted the visually-only confirmation ("Hat alles funktioniert"). **Risk assessment:** structurally near-zero — FlightPlugin owns ONLY the `OnEnter(Arena)` schedule (spawn_player_ship system + cross-plugin SystemSet chain configuration); it does NOT touch the `Update` schedule, the PausePlugin's run_if gates, the GameState transition machinery, or any of Story 3.4's resources (`PausedFrom`, `PauseInitiator`, `PauseOverlayEntity`). The plugin-registration order change (FlightPlugin inserted between ArenaPlugin and PausePlugin in main.rs:46) is order-only and does not shadow or re-register any of PausePlugin's systems. **Acceptance:** the visually-confirmed sub-set (a,b,g — asteroids visible from cockpit, no drift, clean window-close) is sufficient evidence that the cockpit-camera + PlayerShip layer of Story 3.5 works end-to-end; the un-exercised pause-regression sub-set is documented here as a known coverage gap rather than a tested invariant.

2. **0 unit tests added in `src/flight/mod.rs`** (test budget low-end of the 0–2 range). `spawn_player_ship` is integration-test-shaped — exercising it requires a Bevy `App` bootstrap with state machine + asset registry + system scheduling, which is out of scope per architecture.md:354 (integration tests deferred post-M3). The runtime smoke (Task 4) is the de-facto integration test. The `at_least_three_asteroids_within_50m_of_origin` precondition test in `arena/zone.rs:165-172` already covers the spawn-at-origin contract from the asteroid-layout side; re-asserting it from the flight side would violate DRY. Test count net 0 — net post-3.5 = **21** (= 19 baseline pre-3.4 + 2 from 3.4 + 0 from 3.5).

3. **`src/state.rs` left untouched** — deferred-work.md:194 prescribes appending `Copy` to `GameState`'s derive at "the next story that touches `src/state.rs`". Story 3.5 did not need to read `GameState` at runtime (`spawn_player_ship` is registered on `OnEnter(GameState::Arena)`, evaluated at registration time only — no `Res<State<GameState>>` accessed). Deferred-work entry re-deferred per the prescription's conditional clause; new note added to `deferred-work.md:194` documenting the re-deferral.

4. **2-commit pattern (feat + bmad) — NOT YET EXECUTED.** Per Stories 3.1/3.2/3.3/3.4 precedent, commits and pushes await Till's explicit authorization. Task 6's "Commit 1" + "Commit 2" + "DO NOT push" subtasks remain unchecked deliberately; Dev Agent Record + Status flip + sprint-status update are saved without staging or pushing. The two commits when authorized:
   - `feat: cockpit camera + PlayerShip entity (Story 3.5)` — stages `src/flight/mod.rs`, `src/main.rs`, `src/arena/zone.rs`
   - `bmad: story 3.5 ready-for-dev → review (cockpit camera + PlayerShip)` — stages `_bmad-output/implementation-artifacts/sprint-status.yaml`, this story file, `_bmad-output/implementation-artifacts/deferred-work.md`

### File List

**Added:**

- `src/flight/mod.rs` (new file; 93 lines — `FlightPlugin` + `PlayerShip` + `CockpitCamera` markers + `FlightSystems::Setup` SystemSet + `spawn_player_ship` system; 0 unit tests added per the budget low-end choice)

**Modified:**

- `src/main.rs` (+3 net lines: `mod flight;` at line 9, `use flight::FlightPlugin;` at line 19, `.add_plugins(FlightPlugin)` at line 46 between `ArenaPlugin` and `PausePlugin`)
- `src/arena/zone.rs` (–6 net lines: stand-in `Camera3d` spawn block + leading comment + trailing blank removed; pre-3.5 size 173 → post-3.5 size 166 confirms the delta)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (3-5 status flips backlog → ready-for-dev → in-progress → review; `last_updated` bump 2026-04-30 → 2026-05-01)
- `_bmad-output/implementation-artifacts/3-5-cockpit-camera-playership-entity.md` (this file: all task / subtask checkboxes [x] except Commit 1 / Commit 2 / "DO NOT push" awaiting Till's authorization; Dev Agent Record populated with full debug-log table + completion notes + 4 deviations; Status flipped ready-for-dev → in-progress → review)
- `_bmad-output/implementation-artifacts/deferred-work.md` (✅ RESOLVED note appended to the stand-in-Camera3d entry; 🔁 RE-DEFERRED note appended to the GameState-Copy entry; new "Deferred from: 3-5-cockpit-camera-playership-entity" section with 4 forward-compat entries: glTF mesh swap, PlayerOwned re-tint at Story 4.5, Mass/Inertia tuning at Story 3.6, cockpit camera local Transform tune-up)

### Review Findings

_(populated after `bmad-code-review` runs in fresh context post-implementation)_
