# Story 3.3: Hand-Designed Arena Zone with Static Asteroid Field

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player entering Arena from MainMenu,
I want a hand-designed Arena zone with a visible asteroid field rendered in the toon-shaded vector aesthetic from Epic 2,
So that the first second of gameplay drops me into a perceivable 3D space with navigational reference points — not the blank void Story 3.2 currently leaves on Arena entry.

## Acceptance Criteria

1. **Given** Story 3.2's `ArenaPlugin` skeleton exists at `src/arena/mod.rs` with `ArenaEntity` marker, `ArenaSystems::Setup` SystemSet (configured on `OnEnter(GameState::Arena)`), and the generic `cleanup_on_exit::<ArenaEntity>` system on `OnExit(GameState::Arena)`
   **When** Story 3.3 lands
   **Then** a new file `src/arena/zone.rs` is authored with a `spawn_arena_zone` system that runs **once** on `OnEnter(GameState::Arena)`
   **And** the system is registered in `ArenaPlugin::build` via `app.add_systems(OnEnter(GameState::Arena), spawn_arena_zone.in_set(ArenaSystems::Setup))` (uses 3.2's existing set declaration; no new SystemSet needed)
   **And** `pub mod zone;` is added to `src/arena/mod.rs` so the new file is visible to the plugin

2. **Given** `spawn_arena_zone` runs on Arena entry
   **When** the system completes
   **Then** **between 15 and 25 asteroid entities** (inclusive) are spawned at hand-picked `Transform` positions (literal `Vec3` array in source code — NOT random/seeded)
   **And** the spawned positions span a roughly **200 m × 200 m × 200 m** volume centered on the world origin (asteroid centers within `±100 m` on each axis)
   **And** each asteroid uses a **placeholder icosphere `Mesh3d`** (`Sphere::new(radius).mesh().ico(2).unwrap()`) with **radius between 3.0 m and 12.0 m** (per-asteroid radius hand-picked alongside its position)
   **And** each asteroid carries a `MeshMaterial3d<ToonMaterial>` (the Epic-2 `crate::visual::toon_material::ToonMaterial`) with `tint = color_for(SemanticAccent::Neutral).into()` and remaining fields at `ToonMaterial::default()`
   **And** each asteroid carries a `SemanticAccent::Neutral` component
   **And** each asteroid is an Avian `RigidBody::Static` with a `Collider::sphere(radius)` whose radius matches the icosphere's radius (so visual radius == physics radius — projectiles in 3.10 hit what the player sees)
   **And** each asteroid carries the `ArenaEntity` marker (so 3.2's `cleanup_on_exit::<ArenaEntity>` despawns it on Arena exit)

3. **Given** toon shading needs a directional light to produce readable posterized bands (cell shading degenerates without directional shading information)
   **When** `spawn_arena_zone` runs
   **Then** **exactly one** `DirectionalLight` entity is spawned (Bevy 0.18 component) with the `ArenaEntity` marker
   **And** the light's `Transform` rotates the default `-Z` direction such that the light comes from above and slightly to the side (e.g., `Transform::default().looking_to(Vec3::new(-0.3, -1.0, -0.4).normalize(), Vec3::Y)` or equivalent quaternion construction — exact rotation tunable, but the light direction must be non-axis-aligned for legible posterization across multi-faceted asteroid surfaces)
   **And** **no** ambient-light overrides are inserted (Bevy default `AmbientLight` stays — dark-space aesthetic preserved per architecture's "scientific-instrument-panel over military-HUD" framing)
   **And** **no** secondary lights (`PointLight`, additional `DirectionalLight`, `SpotLight`) are spawned — single key light only

4. **Given** also exactly one `Camera3d` is required to render the asteroid field (Story 3.5 will replace this with a cockpit-attached camera; for 3.3, a stand-in camera lets the player verify the scene is visible at all)
   **When** `spawn_arena_zone` runs
   **Then** exactly one `Camera3d` entity is spawned with the `ArenaEntity` marker
   **And** its `Transform` is positioned so the player sees a cluster of asteroids on Arena entry — recommended `Transform::from_xyz(0.0, 5.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y)` (placing the camera 80 m back along +Z, 5 m above the XY plane, framing the origin where most asteroids will cluster)
   **And** the camera carries a placeholder marker `Camera3d`-only (no `CockpitCamera` marker — Story 3.5 introduces that)
   **And** **no** `Camera2d` is spawned in this story (Story 3.1's MainMenu Camera2d was already despawned via `cleanup_main_menu` on `OnExit(MainMenu)`)

5. **Given** Story 3.2's `cleanup_on_exit::<ArenaEntity>` runs on `OnExit(GameState::Arena)`
   **When** the Arena state is exited (entry path arrives in Epic 4 — `Arena → MainMenu` via post-run flow, or `Arena → Paused` in Story 3.4; not exercised in 3.3 itself unless manually verified)
   **Then** every entity carrying `ArenaEntity` is despawned — specifically: 15–25 asteroids + 1 DirectionalLight + 1 Camera3d
   **And** no orphaned arena entities remain (verifiable by manual `Arena → MainMenu` round-trip if test wiring exists; otherwise satisfied by inspection — `cleanup_on_exit::<ArenaEntity>` already proven correct in 3.2)

6. **Given** `VisualSystems::Setup` is currently configured on `OnEnter(GameState::Loading)` with **zero consumers** (Story 3.1 deleted `reference_scene.rs`; deferred-work.md:139 flagged this for cleanup in Story 3.3)
   **When** Story 3.3 lands
   **Then** `spawn_arena_zone` is placed in **`ArenaSystems::Setup`** (the Story-3.2 set on `OnEnter(Arena)`) — **NOT** `VisualSystems::Setup`
   **And** the existing `VisualSystems::Setup` declaration in `src/visual/mod.rs:13-16` (the `enum VisualSystems { Setup }` definition) is **left in place** — the empty no-op `configure_sets(OnEnter(Loading), VisualSystems::Setup)` at `src/visual/mod.rs:23-26` is also left in place (deletion deferred to a dedicated cleanup chore — Story 3.3 stays scope-focused; see Dev Notes "VisualSystems::Setup decision")
   **And** a follow-up entry is appended to `deferred-work.md` documenting the deviation from the 3.1 deferral guidance and the chosen path forward (delete the dormant set in a future cleanup chore, OR re-purpose it for OnEnter(Arena) visual systems if a non-arena-state-bound visual system arrives later)

7. **Given** `SemanticAccent` and `color_for` carry `#[allow(dead_code, reason = "no current consumer; gameplay consumer arrives in Story 4.5 ...")]` annotations at `src/visual/palette.rs:7-10,20-23` (form verified against current source 2026-04-30; the deferred-work.md:102 historical text mentions a `cfg_attr(not(debug_assertions), ...)` form, but the source landed as the simpler `#[allow]` — both forms accomplish the same suppression and are equally targets for deletion)
   **When** Story 3.3 attaches `SemanticAccent::Neutral` to asteroid entities AND uses `color_for(SemanticAccent::Neutral)` to derive the toon tint — both in non-cfg-gated paths
   **Then** **both** `#[allow(dead_code, ...)]` annotation blocks (above `pub enum SemanticAccent` AND above `pub fn color_for`) are **deleted** (the release-path consumer the deferral was waiting for has now arrived ahead of schedule in Story 3.3)
   **And** the post-edit release build is clean: `cargo build --release 2>&1 | grep -cE 'warning:|error:'` returns **0**
   **And** `nm target/release/asteroids3D | grep -c color_for` returns **≥ 1** (the symbol survives release DCE — i.e., the gameplay-path attachment is doing its job)
   **And** the `Story 4.5`-pointed deferred-work entry at `deferred-work.md:100-102` is **resolved** with a pointer to Story 3.3 (`> ✅ RESOLVED 2026-04-30 by Story 3.3 — ...`), preserving the historical body intact (resolution-prefix pattern matches Stories 2.5 and 3.1 cleanup-resolution precedents)

8. **Given** the post-3.2 source baseline (test count = 14; `cargo build --release` clean; 0 Avian-emitted runtime WARNs; `src/main.rs` ~56 lines; `src/arena/mod.rs` ~30 lines)
   **When** Story 3.3 verification runs locally (per `feedback_full_build_output.md` — exit-0 + tail is NOT proof; grep explicitly)
   **Then** **all six** of `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --release` produce **0** lines matching `grep -cE 'warning:|error:'` per their respective full output logs
   **And** `cargo test` summary line reads exactly `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N = 14** if no new tests are added, or **N ≥ 14** if minimal asteroid-count / radius-bounds invariant tests are added (see Dev Notes "Test policy")
   **And** `cargo run` (with `RUST_LOG=info,wgpu=warn,naga=warn` or similar) opens a window, transitions Loading → MainMenu → Arena, AND on Arena entry the player sees a starfield-free dark scene with **15–25 toon-shaded grey icospheres** illuminated by a single directional light (visible posterization: distinct light-side / dark-side bands; outline silhouettes if `bevy_mod_outline` is wired — see AC #9)
   **And** `/tmp/story-3-3-run.log` contains exactly **1** occurrence of `entered Arena` and **0** occurrences of `panic`, `backtrace`, `FATAL`, or any `ERROR`-level log from Bevy / Avian / wgpu beyond known noise (splash-cleanup race per deferred-work.md:137; winit `Skipped event Destroyed` per 1.6 deferred-work LOW-1)
   **And** `git status --short` final set is **exactly**: `src/arena/mod.rs` (M — `pub mod zone;` added + `add_systems` line for `spawn_arena_zone`), `src/arena/zone.rs` (?? — new file), `src/visual/palette.rs` (M — two `cfg_attr` blocks deleted per AC #7), `_bmad-output/implementation-artifacts/sprint-status.yaml` (M — bookkeeping), `_bmad-output/implementation-artifacts/3-3-...-md` (M — this file's Status flip + Dev Agent Record), `_bmad-output/implementation-artifacts/deferred-work.md` (M — entries per AC #6 + #7); **NO** entries under `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `src/state.rs`, `src/splash.rs`, `src/logging.rs`, `src/ui/**`, `src/tuning/**`, `src/visual/mod.rs`, `src/visual/outline.rs`, `src/visual/toon_material.rs`, `assets/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `_bmad-output/planning-artifacts/**`

9. **Given** `bevy_mod_outline::OutlinePlugin` is registered in `VisualPlugin::build` (Story 2.4) and the project's vector aesthetic depends on per-mesh `OutlineVolume` for silhouette legibility (FR49)
   **When** asteroids spawn in `spawn_arena_zone`
   **Then** each asteroid bundle includes an `OutlineVolume` component constructed from the **current** `TuningConfig` outline values — pattern: read `Res<TuningHandle>` + `Res<Assets<TuningConfig>>` at system entry, fall back to `TuningConfig::default()` if the asset isn't loaded yet (cold-start safety — `OnEnter(Arena)` arrives well after `Startup` `load_tuning`, so the asset is normally present, but the fallback prevents a `None`-unwrap panic if the file was deleted or failed to parse)
   **And** the existing `apply_tuning_to_outlines` system at `src/visual/outline.rs:14` (registered in `VisualPlugin` on `Update.in_set(TuningSystems::Reload)`) automatically propagates hot-reloaded outline width/colour to the new asteroid `OutlineVolume`s **without** modification (the system queries `Query<&mut OutlineVolume>` over all entities — no per-entity registration required)
   **And** **no** `generate_outline_normals(...)` call is needed on the icosphere mesh (icospheres have smooth interpolated normals; only hard-edged meshes like Cuboid require outline-normal smoothing — see deleted `reference_scene.rs:69` precedent which skipped the call for spheres)
   **And** the `OutlineVolume`'s `visible: true`, `width` from tuning (default 3.0), `colour` from tuning (default `Color::srgba(0.05, 0.05, 0.05, 1.0)` near-black)

## Tasks / Subtasks

- [x] **Task 1: Author `src/arena/zone.rs`** (AC: #1, #2, #3, #4, #9)
  - [x] Create `src/arena/zone.rs`. Target file size: **120–180 lines** including module doc, system, helpers, optional unit tests. Comment density per `karpathy-guidelines.md` — only WHY-comments where invariants are non-obvious; no narrative code-walkthrough.
  - [x] Module doc 2 lines max, no story-id references (per Story 1.5 review patch BH8 + Story 3.2 patch precedent — see commit `5134b3c`).
  - [x] **Imports:** `bevy::prelude::*`, `avian3d::prelude::{Collider, RigidBody}`, `bevy_mod_outline::OutlineVolume`, `super::ArenaEntity`, `crate::tuning::{TuningHandle, config::TuningConfig}`, `crate::visual::palette::{SemanticAccent, color_for}`, `crate::visual::toon_material::ToonMaterial`. Avoid wildcard imports beyond `bevy::prelude::*`.
  - [x] **Hand-picked asteroid layout — literal `Vec3` array (15–25 entries):**
    ```rust
    /// (position, radius_m) tuples — hand-picked layout spanning ~200×200×200 m
    /// centered on origin. Radii vary 3.0–12.0 m. Origin (0,0,0) and the +Z corridor
    /// (camera spawn → asteroid cluster) are kept clear so the camera frames the field
    /// without an asteroid eclipsing the view; Story 3.5 PlayerShip spawn at origin
    /// will inherit this clearance and have line-of-sight to ≥3 asteroids within 50 m
    /// for AC #3.5.
    const ASTEROIDS: &[(Vec3, f32)] = &[
        (Vec3::new(  20.0,   8.0, -30.0),  6.5),
        (Vec3::new( -25.0,  -5.0, -45.0),  4.5),
        // ... 13–23 more entries
    ];
    ```
    - **Layout discipline:** include 3+ asteroids within a 50 m radius of `Vec3::ZERO` (Story 3.5 line-of-sight precondition); spread the rest across the volume for navigational variety; avoid colocating two asteroids closer than (sum of their radii × 1.5) so colliders don't overlap (causes Avian to emit `WARN: penetration` on first frame).
    - **Suggested distribution:** 5 close (within 50 m radial of origin), 6 mid (~50–100 m radial), 6 far (~100–135 m radial). **All asteroid centers strictly within ±100 m on each individual axis** (X, Y, Z each clamped to ±100 — the radial-distance metric exceeds 100 m only when multiple axes contribute simultaneously, e.g., a corner-of-the-volume asteroid at (90, 50, 80) has radial distance ≈ 130 m but every axis is ≤ 90).
    - **Determinism:** literal positions in source — review-friendly, git-diff-friendly, hand-tunable. Random/seeded layouts rejected for 3.3 (`StdRng::seed_from_u64(...)` is overkill for 17 placeholder asteroids and obscures intent).
  - [x] **System signature:**
    ```rust
    pub fn spawn_arena_zone(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ToonMaterial>>,
        tuning_assets: Res<Assets<TuningConfig>>,
        tuning_handle: Res<TuningHandle>,
    ) {
        let tuning = tuning_assets
            .get(tuning_handle.0.id())
            .cloned()
            .unwrap_or_default();
        let outline_volume = || {
            let [r, g, b, a] = tuning.outline_color;
            OutlineVolume { visible: true, width: tuning.outline_width, colour: Color::srgba(r, g, b, a) }
        };

        // Camera (3.5 will replace with cockpit camera).
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 5.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y),
            ArenaEntity,
        ));

        // Directional light — non-axis-aligned for legible posterization on multi-facet meshes.
        commands.spawn((
            DirectionalLight {
                illuminance: 5_000.0,    // moderate; toon material posterizes regardless
                shadows_enabled: false,  // shadows are post-MVP; FR49 toon look does not require them
                ..default()
            },
            Transform::default().looking_to(Vec3::new(-0.3, -1.0, -0.4).normalize(), Vec3::Y),
            ArenaEntity,
        ));

        // Asteroid field.
        let neutral_tint = color_for(SemanticAccent::Neutral).into();
        for &(position, radius) in ASTEROIDS {
            // ico(2) is well below Bevy's MAX_SUBDIVISIONS=80; unwrap is safe.
            let mesh = meshes.add(Sphere::new(radius).mesh().ico(2).unwrap());
            let material = materials.add(ToonMaterial { tint: neutral_tint, ..default() });
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(position),
                SemanticAccent::Neutral,
                RigidBody::Static,
                Collider::sphere(radius),
                outline_volume(),
                ArenaEntity,
            ));
        }
    }
    ```
  - [x] **Why `tuning.outline_color` is `[f32; 4]` not `Color`:** `TuningConfig::outline_color` is `[f32; 4]` (Story 2.4 schema) — destructure into RGBA components and reassemble via `Color::srgba(r, g, b, a)`. Pattern matches `apply_tuning_to_outlines` at `src/visual/outline.rs:18-20`.
  - [x] **Why a closure for `outline_volume`:** `OutlineVolume` is `!Copy`; spawning the same volume on N+2 entities (N asteroids + 1 light + 1 camera... actually only N asteroids carry it) requires either `.clone()` per entity or a closure that constructs fresh values. Closure is cheap (3 f32 + Color srgba) and avoids `Clone`-on-each-spawn. The DirectionalLight and Camera3d entities do **NOT** carry `OutlineVolume` — outlines apply to mesh geometry only.
  - [x] **Optional unit tests** (in-file `#[cfg(test)] mod tests`):
    - `asteroid_count_in_range`: `assert!((15..=25).contains(&ASTEROIDS.len()))` — guards against accidental array-edit drift.
    - `asteroid_radii_in_range`: `for &(_, r) in ASTEROIDS { assert!((3.0..=12.0).contains(&r)); }` — guards against radius drift.
    - `asteroid_positions_within_volume`: `for &(p, _) in ASTEROIDS { assert!(p.x.abs() <= 100.0 && p.y.abs() <= 100.0 && p.z.abs() <= 100.0); }` — guards against AC #2 volume-bound drift.
    - `colliders_do_not_overlap`: pairwise `(a,b)` check `(a.0 - b.0).length() >= (a.1 + b.1) * 1.0` — N=20 → 190 pairs, runs in microseconds.
    - **Test budget:** 0–4 new tests. If included, expect post-3.3 test count of **14–18**. If skipped (compile-time `const { ... }` assertion alternative is awkward for variable-length data + iteration), document in Completion Notes.

- [x] **Task 2: Wire `zone` module into `ArenaPlugin::build`** (AC: #1)
  - [x] In `src/arena/mod.rs`, add `pub mod zone;` after the module doc-comment, before the `use bevy::prelude::*;` line — OR after the `use` block (rustfmt may reorder; accept its order). The most idiomatic placement is at the top after doc-comment, mirroring `src/visual/mod.rs:7-9` where `pub mod outline; pub mod palette; pub mod toon_material;` sits between the module doc and the `use` block.
  - [x] In `ArenaPlugin::build`, add the system registration line. Final `build` block:
    ```rust
    impl Plugin for ArenaPlugin {
        fn build(&self, app: &mut App) {
            app.configure_sets(OnEnter(crate::state::GameState::Arena), ArenaSystems::Setup);
            app.add_systems(
                OnEnter(crate::state::GameState::Arena),
                zone::spawn_arena_zone.in_set(ArenaSystems::Setup),
            );
            app.add_systems(
                OnExit(crate::state::GameState::Arena),
                cleanup_on_exit::<ArenaEntity>,
            );
        }
    }
    ```
  - [x] **Net delta to `src/arena/mod.rs`:** +1 line (`pub mod zone;`) + 4 lines (the new `add_systems` call). Total ~35 lines (was ~30). Still under any line-budget the architecture might prescribe.
  - [x] **Why `.in_set(ArenaSystems::Setup)`:** Story 3.2's design intent — `ArenaSystems::Setup` is the SystemSet for OnEnter(Arena) work. Future arena systems (3.5 PlayerShip spawn, 3.11 HUD root) will join the same set. Ordering between siblings inside `ArenaSystems::Setup` is currently irrelevant (only `spawn_arena_zone` lives there); when 3.5 lands, **3.5's author** decides whether PlayerShip-spawn `.after(zone::spawn_arena_zone)` is required (it isn't — they're independent spawns operating on disjoint entities).
  - [x] **Why NOT `VisualSystems::Setup`:** see AC #6 + Dev Notes "VisualSystems::Setup decision".

- [x] **Task 3: Remove cfg_attr gating on `SemanticAccent` + `color_for`** (AC: #7)
  - [x] Edit `src/visual/palette.rs`. Delete the two annotation blocks:
    - Lines 7–10 (`#[allow(dead_code, reason = "no current consumer; gameplay consumer arrives in Story 4.5 ...")]`) above `pub enum SemanticAccent`. **NOTE the file currently uses `#[allow(...)]` not `#[cfg_attr(not(debug_assertions), allow(...))]`** — verify the actual annotation form before deleting (Story 2.3 Task 9 may have left only `#[allow(...)]` after the cfg_attr was already removed once; deferred-work.md:102 is the authoritative trail). If the form is `#[allow(...)]`, delete the four-line block. If `#[cfg_attr(not(debug_assertions), allow(...))]`, delete the corresponding block.
    - Lines 20–23 (the analogous block above `pub fn color_for`).
  - [x] After deletion, the resulting `src/visual/palette.rs:1-32` should look like:
    ```rust
    //! Semantic accent palette — FR50 colors with NFR-A1 colorblind distinguishability.
    //! Wong (2011) "Points of view: Color blindness", Nature Methods 8(6), p.441.

    use bevy::prelude::*;

    #[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub enum SemanticAccent {
        Enemy,
        Salvage,
        Hazard,
        PlayerOwned,
        #[default]
        Neutral,
    }

    pub fn color_for(accent: SemanticAccent) -> Color {
        match accent {
            SemanticAccent::Enemy => Color::srgb_u8(0xD5, 0x5E, 0x00),
            // ... unchanged ...
        }
    }
    ```
  - [x] **Verification (Task 4 sweep):** `cargo build --release` produces 0 warnings (the annotations were suppressing release-build dead-code warnings; their removal is safe **only because** Story 3.3's asteroid spawn is a non-cfg-gated consumer).
  - [x] **If clippy/rustc complains about dead-code on release** (i.e., somehow the asteroid spawn isn't reaching the release build — e.g., the `arena` module is accidentally cfg-gated, or the dev forgot Task 2's plugin registration): **revert the deletion**, add the `cfg_attr` blocks back with reason `"reference_scene-style consumer; release consumer still pending — Story 3.3 wiring did not fulfill the requirement"`, and update the deferred-work re-deferral entry. This is a **safety-net only** path; the expected outcome is clean release build after Tasks 1–3 land cleanly.
  - [x] **No structural changes** to `palette.rs` beyond the two annotation deletions. Tests, color values, function signatures — all unchanged. The release-binary `nm` symbol-survival check happens in Task 4.

- [x] **Task 4: Local verification sweep — full build + runtime smoke** (AC: #8, #9)
  - [x] **`cargo check`:**
    ```bash
    cargo check 2>&1 | tee /tmp/story-3-3-check.log
    grep -cE 'warning:|error:' /tmp/story-3-3-check.log
    ```
    Expected: `0`. Likely failure modes: (a) `Sphere::new(radius).mesh().ico(2)` API drift (Bevy 0.18 should accept this — verify via `cargo doc --open` if needed; the deleted reference_scene.rs:69 used the same pattern); (b) `Collider::sphere` import path — should be `avian3d::prelude::Collider` and `Collider::sphere(f32)`; (c) `DirectionalLight` field names — Bevy 0.18 may have renamed fields between point releases; default-spread (`..default()`) covers most of this; (d) `OutlineVolume` color field is `colour` (British spelling) per the source — common typo trap.
  - [x] **`cargo build` (debug):**
    ```bash
    cargo build 2>&1 | tee /tmp/story-3-3-build.log
    grep -cE 'warning:|error:' /tmp/story-3-3-build.log
    ```
    Expected: `0`. Build time should be **incremental** — avian3d, bevy_mod_outline, bevy core all in cache from 3.2.
  - [x] **`cargo test`:**
    ```bash
    cargo test 2>&1 | tee /tmp/story-3-3-test.log
    grep -cE 'warning:|error:|FAILED' /tmp/story-3-3-test.log
    ```
    Expected: `0`. Summary line MUST read `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where `N ∈ {14, 15, 16, 17, 18}` per Task 1's optional-tests budget. Document the chosen N in Completion Notes.
  - [x] **`cargo clippy --all-targets -- -D warnings`:**
    ```bash
    cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-3-3-clippy.log
    grep -cE 'warning:|error:' /tmp/story-3-3-clippy.log
    ```
    Expected: `0`. Particular vigilance for: (a) `unused_imports` if Task 3 deletes a `use` that nothing else needs; (b) `clippy::needless_pass_by_value` if `spawn_arena_zone` accepts `Res<TuningHandle>` by value (acceptable — the `Res<T>` smart pointer is the Bevy convention; if clippy disagrees, use `Res<'_, TuningHandle>` explicitly or `&Res<TuningHandle>` — but Bevy idioms strongly prefer the `Res<T>` form, and this lint should not fire here); (c) `clippy::cast_precision_loss` on the literal `3.0..=12.0` range tests if added as `f32` literals (no cast involved → should not fire).
  - [x] **`cargo fmt --all -- --check`:**
    ```bash
    cargo fmt --all -- --check
    echo $?
    ```
    Expected exit: `0`. If non-zero, run `cargo fmt --all` once and re-check. Rustfmt may reflow the `ASTEROIDS` constant array depending on line length.
  - [x] **`cargo build --release`:**
    ```bash
    cargo build --release 2>&1 | tee /tmp/story-3-3-release.log
    grep -cE 'warning:|error:' /tmp/story-3-3-release.log
    ```
    Expected: `0`. This is the AC #7 release-build assertion (dead-code warnings on `SemanticAccent` / `color_for` would surface here if Task 3's annotation removal is unsafe).
  - [x] **Release-binary symbol survival (AC #7):**
    ```bash
    nm target/release/asteroids3D | grep -c color_for
    ```
    Expected: **≥ 1**. The non-debug consumer (asteroid spawn loop) is the only thing keeping `color_for` alive in release; if this returns 0, AC #7 is violated and Task 3 must be reverted. **macOS-only verification:** `nm` is part of Xcode CLT; same check on Linux uses `nm` (binutils); Windows path is `dumpbin /symbols target/release/asteroids3D.exe | findstr color_for` — but this verification step targets the local-dev macOS build (Till's working environment).
  - [x] **`cargo run` runtime smoke:**
    ```bash
    RUST_LOG=info,wgpu=warn,naga=warn,avian3d=info cargo run 2>&1 | tee /tmp/story-3-3-run.log &
    PID=$!
    sleep 4    # splash (~2 s) + MainMenu paint (1 s) + buffer
    # Manual: focus the window, press Enter to transition to Arena, observe asteroid field
    # Total interactive smoke: ~10 s. SIGINT or close window to terminate.
    ```
  - [x] **Log-grep evidence for runtime smoke:**
    ```bash
    grep -c 'entered Loading' /tmp/story-3-3-run.log              # expected: 1
    grep -c 'splash timer elapsed' /tmp/story-3-3-run.log          # expected: 1
    grep -c 'entered MainMenu' /tmp/story-3-3-run.log              # expected: 1
    grep -c 'MainMenu: Enter pressed' /tmp/story-3-3-run.log       # expected: 1
    grep -c 'entered Arena' /tmp/story-3-3-run.log                 # expected: 1
    grep -cE 'panic|backtrace|FATAL' /tmp/story-3-3-run.log        # expected: 0
    grep -cE 'WARN.*Avian|ERROR.*avian' /tmp/story-3-3-run.log     # expected: 0
    grep -c 'penetration' /tmp/story-3-3-run.log                   # expected: 0 (sentinel for overlapping colliders)
    ```
    All five lifecycle counts MUST be 1 (after a single Enter press). The penetration grep MUST be 0 — if it fires, two ASTEROIDS entries are too close; widen separation.
  - [x] **Visual verification (manual — interactive):**
    - On Arena entry, the screen shows a dark scene populated with **15–25 grey toon-shaded icospheres**.
    - Each asteroid has visible **posterized banding** (light-side bright grey, dark-side dim grey, with discrete steps — no smooth gradient; that's the FR49 toon shader doing its job).
    - Each asteroid has a **dark silhouette outline** (`bevy_mod_outline`'s `OutlineVolume`).
    - The DirectionalLight is **non-axis-aligned**, so multi-facet asteroids (icospheres have 80 faces at subdivisions=2) show readable highlight/shadow transitions on different sides — no "uniformly lit half" effect.
    - **Asteroid distribution:** at least 3 visible within the camera's frustum at the spawn position (camera at `(0, 5, 80)` looking at origin); other asteroids may be off-frame at left/right/top/bottom (camera FOV ≈ 45–60° default).
    - **No** Bevy console errors about ambiguous camera order, missing-asset handles, or shader compile failures.
  - [x] **Tuning hot-reload integration (informal — verifies AC #9 propagation):** while the game is running in Arena, edit `assets/config/tuning.ron` to change `outline_width: 3.0` → `outline_width: 8.0` (or change `outline_color` from near-black to bright-red for high-contrast visibility), save, observe the asteroid outlines update in-place within ~100 ms (Bevy's `file_watcher` debounce). Confirms `apply_tuning_to_outlines` reaches the new asteroid `OutlineVolume` instances. Revert tuning.ron after testing — DO NOT commit a tuning.ron edit.
  - [x] **Cross-state cleanup smoke (informal — verifies AC #5):** there's no `Arena → MainMenu` UI path in 3.3 (Story 3.4 introduces Esc to pause; Story 4.7 introduces post-run flow). Manual cleanup verification is therefore deferred to Stories 3.4+ when an exit path arrives. AC #5 is satisfied **structurally** by `cleanup_on_exit::<ArenaEntity>` (proven correct in 3.2) catching all `ArenaEntity`-marked spawns from this story. Document this in Completion Notes.
  - [x] **`ArenaEntity` marker convention check** (per deferred-work.md:148 resolution path from Story 3.2 review): `grep -c 'ArenaEntity' src/arena/zone.rs` → expected `≥ 4` (one occurrence per spawned entity in the new file: 1 camera + 1 light + 1 in the asteroid loop = 3 spawn-site uses; plus 1 `use super::ArenaEntity;` at top = 4 minimum). `grep -rn 'commands.spawn(' src/arena/zone.rs` enumerates spawn sites (3 sites: camera, light, asteroid loop) — confirm each spawn tuple includes `ArenaEntity`. Without this convention, the Arena exit cleanup silently misses untagged entities.

- [x] **Task 5: Update `_bmad-output/implementation-artifacts/deferred-work.md`** (AC: #6, #7)
  - [x] **Resolve the Story 3.1 entry at deferred-work.md:139** (`VisualSystems::Setup is now empty`):
    - Append a `> ✅ RESOLVED 2026-04-30 by Story 3.3` block (or `> 🔁 RE-DEFERRED 2026-04-30 by Story 3.3`) explaining: Story 3.3 chose `ArenaSystems::Setup` (over the deferred-work-suggested `VisualSystems::Setup`) for `spawn_arena_zone`, in deference to Story 3.2's design intent (architecture.md:347 — `<Feature>Systems` enum per plugin); `VisualSystems::Setup` remains an empty no-op configured on `OnEnter(Loading)` and is now flagged for deletion in a dedicated cleanup chore (or re-purposing if Epic 4+ surfaces a non-arena-state-bound visual system at Arena entry — color-correction, post-process passes, etc.).
    - Append a NEW deferred entry under "Deferred from: 3-3-... (2026-04-30)" header section: **"`VisualSystems::Setup` empty no-op cleanup chore"** — body: "Configured on `OnEnter(Loading)` in `src/visual/mod.rs:23-26` with zero consumers post-3.3. Story 3.3 declined to delete it (out of scope; cleanup deserves its own focused commit). Resolution path: dedicated 1-task chore story OR bundle with the next story that touches `src/visual/mod.rs`. Source: Story 3.3 deviation from deferred-work.md:139 prescription."
  - [x] **Resolve the Story 2.3 → 4.5 re-deferral at deferred-work.md:102** (the `cfg_attr` annotations on `SemanticAccent` / `color_for`):
    - Append `> ✅ RESOLVED 2026-04-30 by Story 3.3` after the existing 2026-04-28 block (preserving the historical body intact). Body: "`SemanticAccent::Neutral` attached to asteroid entities + `color_for(SemanticAccent::Neutral)` derives toon tint in `src/arena/zone.rs::spawn_arena_zone` — both non-cfg-gated paths. The two `cfg_attr(not(debug_assertions), allow(dead_code))` annotations were deleted from `src/visual/palette.rs:7-10,20-23`. Verification: `cargo build --release` clean (0 warnings); `nm target/release/asteroids3D | grep -c color_for ≥ 1`. Story 4.5 (full SemanticAccent wiring on enemies/salvage/playership/projectiles) inherits the un-annotated palette and proceeds with the broader gameplay-entity wiring without needing to touch palette.rs."
  - [x] **Append a NEW deferred-work entry section** under header `## Deferred from: 3-3-hand-designed-arena-zone-with-static-asteroid-field (2026-04-30)`:
    - **`VisualSystems::Setup` empty no-op cleanup chore** (per above bullet) — copy the body verbatim.
    - **(Optional, if Task 1 added unit tests):** any test-coverage gap or invariant that should be re-evaluated when Stories 3.4–3.10 attach more arena entities.
    - **Replacement camera contract (Story 3.5):** Story 3.3's stand-in `Camera3d` at `(0, 5, 80)` looking at origin must be **replaced** by Story 3.5's `CockpitCamera` child of PlayerShip. Story 3.5's `OnEnter(Arena)` order ambiguity: if `spawn_arena_zone` (3.3) and `spawn_player_ship` (3.5) both spawn a Camera3d on OnEnter(Arena), Bevy emits an "ambiguous camera order" runtime warning per frame. **Resolution:** Story 3.5 must either (a) NOT spawn a top-level Camera3d if 3.3's already exists (cockpit camera is a child of PlayerShip and so its world-space camera is the only render-order-relevant Camera3d), OR (b) despawn 3.3's stand-in Camera3d at the top of 3.5's spawn system (filter `Query<Entity, (With<Camera3d>, With<ArenaEntity>, Without<CockpitCamera>)>` and despawn). Flag this here so 3.5's author makes the decision intentionally.

- [x] **Task 6: Bookkeeping — story status flip + commit + push** (AC: all)
  - [x] Populate this story file's **Dev Agent Record**: Agent Model Used, Debug Log References (per-command grep counts + log paths), Completion Notes (per-AC evidence + any deviations), File List (added / modified). Section structure follows Story 3.2's precedent.
  - [x] Set this story's `Status:` header → `review`.
  - [x] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - Flip `3-3-hand-designed-arena-zone-with-static-asteroid-field: ready-for-dev` → `3-3-hand-designed-arena-zone-with-static-asteroid-field: review`.
    - Bump `last_updated:` (both top-comment line and YAML body key) to: `last_updated: 2026-04-30 (Story 3.3 ready-for-dev → review — hand-designed asteroid field + DirectionalLight + cfg_attr removal)`.
    - epic-3 status stays `in-progress` — third story; no transition.
    - YAML parse verification: `python3 -c "import yaml; yaml.safe_load(open('_bmad-output/implementation-artifacts/sprint-status.yaml')); print('OK')"` → expected `OK`.
  - [ ] **Commit 1 (source — triggers CI):** stage `src/arena/mod.rs`, `src/arena/zone.rs`, `src/visual/palette.rs`. NO `_bmad-output/**` files in this commit. *(Awaits Till's authorization per project convention; not auto-executed by the dev agent.)*
    - Commit message subject (HEREDOC, ≤ 70 chars): `feat: hand-designed Arena asteroid field + light (Story 3.3)`. Literal length: 56 chars.
    - Push to `origin/master`. Triggers full 4-job `ci.yml` matrix (paths-ignore excludes `_bmad-output/**` only).
    - **Expected CI outcome:** all 4 jobs ✓. Wall time: **~5–10 m on warm avian3d cache** (3.2 already paid the cold-build cost). msrv-check (Rust 1.89) MUST pass — no new dep additions, just usage of existing crates.
    - `gh run list --workflow=ci.yml -L 1` → capture run ID. Wait for completion. `gh run view <ID> --log | grep -cE 'warning:|error:'` → expected `0` (modulo `Free disk space` action ambient noise per 3.1/3.2 precedent).
  - [ ] **Commit 2 (bookkeeping — does NOT trigger CI):** stage `_bmad-output/implementation-artifacts/sprint-status.yaml`, `_bmad-output/implementation-artifacts/3-3-hand-designed-arena-zone-with-static-asteroid-field.md`, `_bmad-output/implementation-artifacts/deferred-work.md`. *(Awaits Till's authorization.)*
    - Commit message subject (HEREDOC): `bmad: story 3.3 ready-for-dev → review (asteroid field + DirectionalLight)`. Matches Story 3.1 / 3.2 bookkeeping commit shape.
    - Push to `origin/master`. Does NOT trigger CI — `_bmad-output/**` is in `paths-ignore`.
  - [x] **Why two commits, not one:** matches Stories 3.1 / 3.2 / 2.4 / 2.5 / 2.6 precedent. Clean diff focus + CI cost focus + roll-back granularity.
  - [x] Story awaits code review. **Code review recommended via `bmad-code-review` skill, ideally with a different LLM than the implementer.** Diff surface is medium (~150 lines new in `src/arena/zone.rs`; ~5 lines modified in `src/arena/mod.rs`; ~8 lines deleted in `src/visual/palette.rs`); a 3-agent review fits this scope. Specific review attention areas:
    - **(a) Asteroid layout sanity:** are 15–25 entries actually present? Are positions within the 200×200×200 m volume? Are radii in `[3.0, 12.0]`? Do any colliders overlap (penetration log on first physics tick)?
    - **(b) Camera replacement contract:** does Task 5's deferred-work entry adequately warn Story 3.5 about the stand-in Camera3d? Or should Story 3.3 NOT spawn a Camera3d at all (deferring all Arena rendering to Story 3.5's PlayerShip Camera3d)?
    - **(c) `cfg_attr` removal safety:** verify the `nm color_for ≥ 1` claim in CI by adding a `nm` symbol-survival check to the verification harness (or accept Till's local-only verification).
    - **(d) `VisualSystems::Setup` deviation:** is `ArenaSystems::Setup` the right home, or should `VisualSystems::Setup` be re-targeted? If the latter is the architectural intent, Story 3.3 should be amended.
    - **(e) Outline normals on icospheres:** the spec asserts no `generate_outline_normals` is needed for icospheres; verify that the visual outline is actually visible at runtime (smoke screenshot if needed).

## Dev Notes

### Why this story exists

Story 3.3 is the **first story in Epic 3 to produce visible gameplay content**. Stories 3.1 (title screen stub) and 3.2 (Avian + ArenaPlugin skeleton) prepared the runway; 3.3 lays the asphalt. Three concrete things land:

1. **A perceivable Arena scene.** Pre-3.3, pressing Enter on the title screen transitions to `GameState::Arena` and the screen goes **blank** (Story 3.2's verification log explicitly notes this — "the screen goes blank (Arena state has no rendering yet — that's Story 3.3)"). Story 3.3 fills that blank with a hand-designed asteroid field — the first frame of *gameplay context* the player ever sees.

2. **The arena-spawn pattern Stories 3.5–3.11 follow.** Story 3.5 (PlayerShip), 3.9 (projectiles on fire), 3.11 (HUD root) all spawn entities tagged `ArenaEntity` on `OnEnter(Arena)` (or in response to Arena-scoped input events). Story 3.3 establishes the precedent: **bundle = visual + physics + accent + lifecycle marker**. Subsequent stories copy this template with their own components.

3. **The first non-cfg-gated consumer of `SemanticAccent` + `color_for`.** Stories 2.2 / 2.3 added `palette.rs` with a `cfg_attr(not(debug_assertions), allow(dead_code))` placeholder (deferred-work.md:102). Story 3.3 — by attaching `SemanticAccent::Neutral` to asteroid entities — makes the palette **release-binary-relevant**, fulfilling the long-deferred consumer requirement six stories ahead of the original Story 4.5 schedule.

[Source: [`epics/epic-3-arena-flight-first-combat-first-playable.md:57-81`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md) (Story 3.3 epic spec); [`architecture.md:240`](../planning-artifacts/architecture.md) (FixedUpdate physics decision); [`architecture.md:415-420`](../planning-artifacts/architecture.md) (`cleanup_on_exit::<T>` pattern via `ArenaEntity`); [`architecture.md:343-350`](../planning-artifacts/architecture.md) (Plugin-per-feature module pattern); [`3-2-...-md` lines 478-492](./3-2-avian-physics-foundation-arena-state-skeleton.md) (Story 3.3 forward-compat hand-off)]

### Inherited context from Stories 1.1, 2.1, 2.2, 2.3, 2.4, 3.1, 3.2

| Fact | Value | Source |
|---|---|---|
| Bevy version | `0.18` (resolved `0.18.1`) | `Cargo.toml:8` |
| Avian version | `avian3d = "0.6"` (resolved `0.6.1`) — first usage in 3.2; reused in 3.3 | `Cargo.toml:9`, `Cargo.lock` |
| `bevy_mod_outline` version | `0.12` (resolved `0.12.0`); `OutlinePlugin` registered in `VisualPlugin::build` since 2.4 | `Cargo.toml:10`, `src/visual/mod.rs:21` |
| `Collider::sphere` signature | `pub fn sphere(radius: f32) -> Collider` (Avian 0.6 `parry/mod.rs:725`; `Scalar = f32` per `math/single.rs:6`) | empirical |
| `RigidBody::Static` | enum variant in `avian3d::prelude::RigidBody` — `pub enum RigidBody { Dynamic, Static, Kinematic }` (`dynamics/rigid_body/mod.rs:284-305`) | empirical |
| `Sphere::new(r).mesh().ico(n)` | Bevy 0.18 mesh builder; `n=2` → 80 faces, smooth interpolated normals → no `generate_outline_normals` needed | deleted `reference_scene.rs:69`, Story 2.1 |
| `OutlineVolume` field names | `visible: bool, width: f32, colour: Color` (British spelling) | `bevy_mod_outline-0.12.0/src/lib.rs:217-225` |
| `MeshMaterial3d<ToonMaterial>` | Bevy 0.18 generic component (`Mesh3d` + `MeshMaterial3d<M>` replaces `MaterialMeshBundle` from 0.16-) | `src/visual/toon_material.rs:34`, deleted `reference_scene.rs:73` |
| `ToonMaterial` defaults | `tint: WHITE, steps: 4, rim_power: 2.0, rim_intensity: 0.3` | `src/visual/toon_material.rs:23-32` |
| `TuningHandle` | `Resource(pub Handle<TuningConfig>)` populated in `Startup` `load_tuning` | `src/tuning/mod.rs:17, 38-40` |
| `TuningConfig::default()` | `outline_width: 3.0, outline_color: [0.05, 0.05, 0.05, 1.0]` (near-black) | `src/tuning/config.rs:23-29` |
| `assets/config/tuning.ron` | live, hot-reload via `file_watcher` Bevy feature | `Cargo.toml:8` (`"file_watcher"` feature) |
| `apply_tuning_to_outlines` | `Update.in_set(TuningSystems::Reload)` queries `Query<&mut OutlineVolume>` — applies to ALL outline-bearing entities, no per-entity registration | `src/visual/outline.rs:14-27` |
| `cleanup_on_exit::<T>` | generic `pub fn` in `src/arena/mod.rs` since 3.2; despawns roots, children cascade via Bevy 0.18 `ChildOf` linked-despawn | `src/arena/mod.rs:26-30` |
| `ArenaEntity` marker | unit struct `pub`; tags any entity that should be despawned on Arena exit | `src/arena/mod.rs:13-14` |
| `ArenaSystems::Setup` | SystemSet configured on `OnEnter(Arena)`; consumer-less in 3.2; 3.3 is the first consumer | `src/arena/mod.rs:8-11, 18` |
| `VisualSystems::Setup` | configured on `OnEnter(Loading)` since 2.1; consumer-less since 3.1 deleted reference_scene | `src/visual/mod.rs:13-16, 23-26` |
| `SemanticAccent::Neutral` | enum variant returning `#9A9A9A` (neutral grey) via `color_for` | `src/visual/palette.rs:11-18, 30` |
| `cfg_attr` on palette items | currently `#[allow(dead_code, reason = "...Story 4.5...")]` (verify exact form before Task 3 deletion) | `src/visual/palette.rs:7-10, 20-23` (per current read) |
| Test count post-3.2 | **14 passing** | `_bmad-output/implementation-artifacts/3-2-...-md` Dev Agent Record |
| Test count post-3.3 (expected) | **14–18** depending on Task 1's optional-test-budget choice | this story |
| `tracing` + panic-hook | live since 1.8 | `src/logging.rs` |
| Splash race re-deferred | non-deterministic; not a 3.3 regression | deferred-work.md:137 |
| Splash file location debt | `src/splash.rs` flat at `src/`; 3.3 does NOT touch | deferred-work.md:138 |
| Commit style precedent | `feat:` for source, `bmad:` for bookkeeping; HEREDOC for multi-line; no `Co-Authored-By` trailer | `git log --oneline -n 15` |
| Two-commit pattern | source + bookkeeping; used by Stories 1.7/2.4/2.5/2.6/3.1/3.2 | `git log` |
| `paths-ignore` in CI | `.github/workflows/ci.yml` excludes `_bmad/**` and `_bmad-output/**` from triggers | deferred-work.md:5 |

### Six-key constraint summary (memorize these)

1. **`spawn_arena_zone` runs ONCE per Arena entry — `OnEnter(GameState::Arena)`.** Not `Update`, not `OnEnter(Loading)`, not `Startup`. Story 3.2's `ArenaSystems::Setup` is the canonical landing; use `.in_set(ArenaSystems::Setup)`.
2. **15–25 asteroids — count is bounded.** AC #2 requires literal count in `[15, 25]` inclusive. A const-array `ASTEROIDS: &[(Vec3, f32)] = &[...]` makes this self-documenting and unit-testable.
3. **Hand-picked positions, NOT random.** No `StdRng`, no `rand` crate, no `seed_from_u64`. Literal `Vec3::new(x, y, z)` per asteroid. Layout is review-friendly and git-diff-friendly. **Volume bound: ±100 m on each axis.**
4. **Visual radius == physics radius.** `Mesh3d(Sphere::new(r))` and `Collider::sphere(r)` MUST use the same `r` per asteroid. If they diverge, projectiles in 3.10 will visually miss while colliding (or vice versa) — a frustrating bug that's much cheaper to prevent than diagnose. The `for &(position, radius) in ASTEROIDS` loop binds them via the same local.
5. **DirectionalLight, NOT PointLight.** AC #3 requires *exactly one* DirectionalLight. The deleted `reference_scene.rs` used three PointLights for 3-point lighting; that pattern is **not** what 3.3 wants. Toon shading needs a single dominant directional source for legible posterization; multiple point lights produce muddled banding.
6. **Tuning hot-reload "just works" — no special wiring.** The existing `apply_tuning_to_outlines` system at `src/visual/outline.rs:14` queries `Query<&mut OutlineVolume>` over ALL outline-bearing entities. Story 3.3's new asteroid `OutlineVolume` instances are picked up automatically. Do **NOT** add a per-asteroid tuning-watcher system; that would be redundant.

### Architecture compliance

- **Arena-state-scoped spawning + cleanup** matches `architecture.md:415-420` ("entities spawned for a state tag themselves with a marker component (e.g., `ArenaEntity`) and are despawned by a `cleanup_on_exit::<ArenaEntity>` system in `OnExit(GameState::Arena)`"). Story 3.3's spawned asteroids + light + camera all carry `ArenaEntity` and are caught by 3.2's already-registered cleanup system.
- **Plugin-per-feature module + SystemSet ordering** matches `architecture.md:343-350` and Story 3.2's `ArenaPlugin` precedent. `spawn_arena_zone` is a system within `ArenaPlugin`'s ownership; placement in `ArenaSystems::Setup` SystemSet is the architectural idiom.
- **Component-composition-first** matches `architecture.md:73, 211-213` ("ECS Data Modeling: Component-composition-first. Small reusable components shared across entity archetypes"). Asteroid bundle = `Mesh3d` + `MeshMaterial3d<ToonMaterial>` + `Transform` + `SemanticAccent` + `RigidBody` + `Collider` + `OutlineVolume` + `ArenaEntity`. Eight small components, each single-purpose, no god-struct. **Forward-compat:** Story 3.10 will add `AsteroidHp` to this bundle; that's a one-component addition, not a refactor.
- **Asset-load-at-state-entry pattern** matches `architecture.md:426-429` ("Forbidden: `AssetServer::load(&str)` scattered inside gameplay systems"). Story 3.3 uses `meshes.add(...)` and `materials.add(...)` (in-system asset *creation*, not asset-server *loading*) — these are runtime-generated assets (the icosphere mesh + the toon material are constructed at spawn-time, not loaded from disk). This is **explicitly permitted** by the architecture; the prohibition targets `AssetServer::load("path/to/file.gltf")` calls inside `Update` systems, not `Assets<T>::add(asset_value)` calls inside `OnEnter` systems.
- **Cross-cutting Resources read-only** matches `architecture.md:660-664`. `spawn_arena_zone` reads `Res<TuningHandle>` and `Res<Assets<TuningConfig>>` — both read-only. The `ResMut<Assets<Mesh>>` and `ResMut<Assets<ToonMaterial>>` are mutable because asset creation requires it; this is Bevy-idiomatic and not a "writes into another plugin's internal Resource" violation (the asset Storage is a Bevy-engine-owned resource shared by all asset-creating systems).
- **No god-plugin** — `ArenaPlugin` owns ONLY arena-state lifecycle (now including the zone spawn). Visual-pipeline setup stays in `VisualPlugin` (`OutlinePlugin` registration, `MaterialPlugin<ToonMaterial>` registration). Tuning lives in `TuningPlugin`. Plugin boundaries match `architecture.md:643-657`.
- **Past-tense events** — none yet. Story 3.10 will introduce `AsteroidDestroyed`, `ProjectileHitAsteroid` (PascalCase past-tense per `architecture.md:324`). Story 3.3 emits no events.
- **Naming** — `spawn_arena_zone` is a snake_case verb-phrase system per `architecture.md:323`. `ArenaEntity` is PascalCase noun marker per `architecture.md:322`. `ASTEROIDS` is SCREAMING_SNAKE_CASE constant per Rust convention.

### Library / framework requirements

| Crate | Version | Change in Story 3.3 |
|---|---|---|
| `bevy` | `0.18` (resolved `0.18.1`) | unchanged — uses `Sphere`, `Mesh3d`, `MeshMaterial3d`, `DirectionalLight`, `Camera3d`, `Transform`, `Color`, `Vec3` (all `bevy::prelude::*`) |
| `avian3d` | `0.6` (resolved `0.6.1`) | unchanged — first usage of `RigidBody::Static` and `Collider::sphere` (preludes already in scope thanks to 3.2's foundation) |
| `bevy_mod_outline` | `0.12` (resolved `0.12.0`) | unchanged — first usage of `OutlineVolume` in gameplay code (was used in deleted reference_scene.rs) |
| All other pinned deps | unchanged | unchanged |
| `Cargo.toml` | unchanged | no feature additions, no version bumps, no new deps |
| `Cargo.lock` | unchanged (expected) | no dep tree change; should be byte-identical post-3.3 |

**Avian 0.6 imports needed in 3.3:** `RigidBody`, `Collider` from `avian3d::prelude::{...}`. Both are re-exported via the `avian3d::prelude` module (verified in `Cargo.lock`'s avian3d-0.6.1 source at `src/lib.rs:550` — `pub use crate::collision::prelude::*; ... pub use crate::dynamics::{prelude::*};`).

**No imports needed yet (deferred to Stories 3.5+):** `LinearVelocity`, `AngularVelocity`, `ExternalForce`, `ExternalTorque`, `CollisionLayers`, `CollisionStarted`/`CollisionEnded` events, `Sensor`. Story 3.3 uses static physics only.

### File structure changes

| Path | Action | Purpose |
|---|---|---|
| `src/arena/zone.rs` | **Add** | `pub fn spawn_arena_zone(...)` system + `ASTEROIDS: &[(Vec3, f32)]` const + optional unit tests; ~120-180 lines. |
| `src/arena/mod.rs` | **Modify** | +1 line `pub mod zone;`, +4 lines `add_systems(OnEnter(Arena), zone::spawn_arena_zone.in_set(ArenaSystems::Setup))`. Net +5 lines. |
| `src/visual/palette.rs` | **Modify** | Delete two `#[allow(dead_code, reason = "...")]` blocks (above `pub enum SemanticAccent` and above `pub fn color_for`). Net −8 lines. |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | **Modify** | 3-3 → review, last_updated bump |
| `_bmad-output/implementation-artifacts/3-3-...-md` (this file) | **Modify** | Tasks checked, Dev Agent Record populated, Status → review |
| `_bmad-output/implementation-artifacts/deferred-work.md` | **Modify** | Resolve 3.1 entry at :139, append RESOLVED block at :100-102, add new "Deferred from: 3-3-..." section |
| `Cargo.toml`, `Cargo.lock` | **Do NOT touch** | No version bumps, no feature additions |
| `src/main.rs` | **Do NOT touch** | All wiring stays inside `ArenaPlugin::build` (added in 3.2). main.rs is unchanged. |
| `src/state.rs` | **Do NOT touch** | `log_arena_entered` already exists since 3.1 |
| `src/splash.rs` | **Do NOT touch** | Splash race + location debt re-deferred |
| `src/logging.rs` | **Do NOT touch** | Out of scope |
| `src/ui/**` | **Do NOT touch** | Title screen stays as-is |
| `src/visual/mod.rs` | **Do NOT touch** | The `VisualSystems::Setup` empty-no-op cleanup is deferred to a separate chore (see AC #6 / Task 5) |
| `src/visual/outline.rs` | **Do NOT touch** | `apply_tuning_to_outlines` already handles new asteroid OutlineVolumes via `Query<&mut OutlineVolume>` |
| `src/visual/toon_material.rs` | **Do NOT touch** | `ToonMaterial` is consumed via `materials.add(ToonMaterial { ... })`; no API change needed |
| `src/tuning/**` | **Do NOT touch** | TuningHandle + TuningConfig consumed read-only via `Res<>` |
| `assets/**` | **Do NOT touch** | No new assets; the icosphere is procedurally generated |
| `assets/config/tuning.ron` | **Do NOT touch (commit)** | Manual hot-reload smoke per Task 4 may temporarily edit; revert before commit |
| `docs/**` | **Do NOT touch** | Out of scope |
| `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.gitignore` | **Do NOT touch** | Out of scope |

### `src/arena/zone.rs` skeleton (rustfmt-tolerant, near-verbatim)

```rust
//! Hand-designed Arena zone — static asteroid field + key light + stand-in camera.
//! Spawns on OnEnter(Arena); cleaned up on OnExit(Arena) via 3.2's cleanup_on_exit::<ArenaEntity>.

use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use bevy_mod_outline::OutlineVolume;

use super::ArenaEntity;
use crate::tuning::TuningHandle;
use crate::tuning::config::TuningConfig;
use crate::visual::palette::{SemanticAccent, color_for};
use crate::visual::toon_material::ToonMaterial;

/// Hand-picked (position, radius_m) layout — 17 asteroids covering ~200×200×200 m.
/// Origin and the +Z corridor (toward the spawn camera) kept clear so Story 3.5's
/// PlayerShip at origin has line-of-sight to ≥3 asteroids within 50 m.
const ASTEROIDS: &[(Vec3, f32)] = &[
    // Close cluster (within 50 m of origin) — 5 asteroids
    (Vec3::new(  18.0,   3.0, -25.0),  6.5),
    (Vec3::new( -22.0,  -4.0, -38.0),  4.5),
    (Vec3::new(  -8.0,  10.0, -42.0),  5.0),
    (Vec3::new(  30.0,  -8.0, -18.0),  3.5),
    (Vec3::new(  -5.0,  -3.0,  35.0),  4.0),

    // Mid-range (50–100 m radial) — 6 asteroids
    (Vec3::new(  60.0,  20.0, -50.0),  9.0),
    (Vec3::new( -55.0,  -15.0, -75.0), 7.5),
    (Vec3::new(  45.0, -25.0, -90.0),  6.0),
    (Vec3::new( -70.0,  10.0, -25.0),  8.0),
    (Vec3::new(  85.0,   5.0,  40.0),  5.5),
    (Vec3::new( -50.0,  35.0,  55.0),  7.0),

    // Far field (90–135 m radial — strictly within ±100 m on every axis) — 6 asteroids
    (Vec3::new(  95.0, -45.0, -75.0),  11.0),
    (Vec3::new( -85.0,  40.0, -95.0),  10.5),
    (Vec3::new(  20.0,  55.0,  -90.0), 12.0),
    (Vec3::new( -30.0, -55.0,  -95.0),  9.5),
    (Vec3::new(  75.0,  25.0,  90.0),   8.5),
    (Vec3::new( -90.0, -10.0,  85.0),  10.0),
];

pub fn spawn_arena_zone(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ToonMaterial>>,
    tuning_assets: Res<Assets<TuningConfig>>,
    tuning_handle: Res<TuningHandle>,
) {
    // Cold-start safety: tuning.ron may not be loaded if a future re-entry path
    // races OnEnter(Arena) ahead of TuningPlugin's Startup load. Default fallback
    // matches assets/config/tuning.ron initial values.
    let tuning = tuning_assets
        .get(tuning_handle.0.id())
        .cloned()
        .unwrap_or_default();
    let outline_volume = || {
        let [r, g, b, a] = tuning.outline_color;
        OutlineVolume {
            visible: true,
            width: tuning.outline_width,
            colour: Color::srgba(r, g, b, a),
        }
    };

    // Stand-in camera — Story 3.5 replaces with cockpit camera (child of PlayerShip).
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y),
        ArenaEntity,
    ));

    // Key light — non-axis-aligned for readable posterization on multi-facet asteroids.
    commands.spawn((
        DirectionalLight {
            illuminance: 5_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(-0.3, -1.0, -0.4).normalize(), Vec3::Y),
        ArenaEntity,
    ));

    // Asteroid field.
    let neutral_tint = color_for(SemanticAccent::Neutral).into();
    for &(position, radius) in ASTEROIDS {
        // ico(2) is well below Bevy's MAX_SUBDIVISIONS=80; unwrap is safe.
        let mesh = meshes.add(Sphere::new(radius).mesh().ico(2).unwrap());
        let material = materials.add(ToonMaterial { tint: neutral_tint, ..default() });
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(position),
            SemanticAccent::Neutral,
            RigidBody::Static,
            Collider::sphere(radius),
            outline_volume(),
            ArenaEntity,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asteroid_count_in_acceptance_range() {
        assert!(
            (15..=25).contains(&ASTEROIDS.len()),
            "AC #2: expected 15–25 asteroids; got {}",
            ASTEROIDS.len()
        );
    }

    #[test]
    fn asteroid_radii_within_3_to_12() {
        for &(pos, r) in ASTEROIDS {
            assert!(
                (3.0..=12.0).contains(&r),
                "AC #2: radius {} at {:?} outside [3.0, 12.0]",
                r,
                pos
            );
        }
    }

    #[test]
    fn asteroid_positions_within_volume() {
        for &(pos, _) in ASTEROIDS {
            assert!(
                pos.x.abs() <= 100.0 && pos.y.abs() <= 100.0 && pos.z.abs() <= 100.0,
                "AC #2: position {:?} outside ±100 m volume",
                pos
            );
        }
    }

    #[test]
    fn asteroid_colliders_do_not_overlap() {
        // (a,b) pairs: collider radii × 1.0 minimum centerline separation. This is
        // looser than the Task 1 spec's 1.5 multiplier — but tests should match the
        // physical-overlap threshold (penetration = sum of radii); the 1.5x in Task 1
        // is a safety margin for layout authors, not a hard physics requirement.
        for (i, &(p1, r1)) in ASTEROIDS.iter().enumerate() {
            for &(p2, r2) in ASTEROIDS.iter().skip(i + 1) {
                let distance = (p1 - p2).length();
                let min_separation = r1 + r2;
                assert!(
                    distance >= min_separation,
                    "asteroids at {:?} (r={}) and {:?} (r={}) overlap (distance={}, min={})",
                    p1, r1, p2, r2, distance, min_separation
                );
            }
        }
    }
}
```

### `src/arena/mod.rs` post-edit (rustfmt-tolerant — diff against current)

```rust
//! ArenaPlugin — owns GameState::Arena entity lifecycle (spawn / cleanup).
//! Later stories attach asteroid spawning; following stories add player ship, projectiles, and HUD.

pub mod zone;

use bevy::prelude::*;

pub struct ArenaPlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ArenaSystems {
    Setup,
}

#[derive(Component)]
pub struct ArenaEntity;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(OnEnter(crate::state::GameState::Arena), ArenaSystems::Setup);
        app.add_systems(
            OnEnter(crate::state::GameState::Arena),
            zone::spawn_arena_zone.in_set(ArenaSystems::Setup),
        );
        app.add_systems(
            OnExit(crate::state::GameState::Arena),
            cleanup_on_exit::<ArenaEntity>,
        );
    }
}

pub fn cleanup_on_exit<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
```

### `src/visual/palette.rs` post-edit (rustfmt-tolerant — diff against current)

```rust
//! Semantic accent palette — FR50 colors with NFR-A1 colorblind distinguishability.
//! Wong (2011) "Points of view: Color blindness", Nature Methods 8(6), p.441.

use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SemanticAccent {
    Enemy,
    Salvage,
    Hazard,
    PlayerOwned,
    #[default]
    Neutral,
}

pub fn color_for(accent: SemanticAccent) -> Color {
    match accent {
        SemanticAccent::Enemy => Color::srgb_u8(0xD5, 0x5E, 0x00),
        SemanticAccent::Salvage => Color::srgb_u8(0x00, 0x9E, 0x73),
        SemanticAccent::Hazard => Color::srgb_u8(0xF0, 0xE4, 0x42),
        SemanticAccent::PlayerOwned => Color::srgb_u8(0x56, 0xB4, 0xE9),
        SemanticAccent::Neutral => Color::srgb_u8(0x9A, 0x9A, 0x9A),
    }
}

#[cfg(test)]
mod tests {
    // ... unchanged ...
}
```

### VisualSystems::Setup decision — why ArenaSystems::Setup wins

Two SystemSets currently target `OnEnter(...)` schedules: `VisualSystems::Setup` (configured on `OnEnter(Loading)`, **0 consumers** post-3.1) and `ArenaSystems::Setup` (configured on `OnEnter(Arena)`, **0 consumers** post-3.2). Both are syntactically eligible homes for `spawn_arena_zone`. Story 3.3 picks **`ArenaSystems::Setup`** — here's why:

| Option | Pros | Cons |
|---|---|---|
| **`ArenaSystems::Setup` (selected)** | Architectural intent is per-feature SystemSets (architecture.md:347, 643-657). Story 3.2 explicitly created this set "so 3.3 can `.in_set(ArenaSystems::Setup)` without a follow-up patch" (3-2-...-md:93). Future arena systems (3.5 PlayerShip, 3.11 HUD) will join the same set. | Leaves `VisualSystems::Setup` orphaned — 0 consumers, configured on a schedule (`OnEnter(Loading)`) that no longer hosts any visual setup. Cleanup deferred to a later chore. |
| `VisualSystems::Setup` (rejected — was deferred-work.md:139's prescription) | Reuses an existing SystemSet, avoids leaving an orphan. Architecturally "visual setup" is a defensible characterization of asteroid-mesh spawning. | Conflicts with 3.2's explicit ArenaSystems intent. Reconfiguring `VisualSystems::Setup` from `OnEnter(Loading)` to `OnEnter(Arena)` mid-stream changes the set's semantic meaning — a refactor, not a feature addition. Architecturally muddier (where does PlayerShip-spawn (3.5) go? It's not "visual setup" — it's player-state-bound). |
| Inline (no SystemSet) | Minimal diff. | Defeats Story 3.2's `ArenaSystems::Setup` rationale. Future ordering work between 3.5+ systems becomes ad-hoc `.after(spawn_arena_zone)` — explicitly forbidden by architecture.md:415. |

**Decision:** `ArenaSystems::Setup` honors 3.2's design and architecture.md's plugin-bound SystemSet idiom. The `VisualSystems::Setup` orphan becomes a separate cleanup task (Task 5 deferred-work entry). The deferred-work.md:139 guidance was written before Story 3.2's `ArenaSystems::Setup` was finalized; Story 3.3 supersedes it with a cleaner architectural alignment.

### Camera placement — why a stand-in vs. wait for 3.5

AC #4 requires a `Camera3d` in the scene. Without it, the asteroid field renders to no surface — the player sees a black screen and AC #8's visual-verification step fails. Two paths:

| Option | Pros | Cons |
|---|---|---|
| **Stand-in Camera3d in 3.3 (selected)** | Player sees the scene immediately on Arena entry — gives AC #8 visual evidence; tightens the 3.3 → 3.5 hand-off (Story 3.5's PlayerShip-with-cockpit-camera replaces the stand-in, validated against a known-rendering baseline). | Story 3.5 must despawn / replace this camera; risk of dual-camera "ambiguous order" warning if 3.5's author misses the contract (mitigated by Task 5 deferred-work entry). |
| Defer all cameras to 3.5 | Cleaner separation of concerns (3.3 = scene, 3.5 = player + camera). | 3.3's runtime smoke (AC #8) cannot visually verify the asteroid field — only logs prove the spawn ran. Regressions invisible until 3.5. Bad for momentum. |

**Decision:** stand-in Camera3d in 3.3. The hand-off contract is documented in Task 5 deferred-work entry; Story 3.5's author reads it before adding cockpit camera.

### Test policy — why 4 tests are reasonable here

Story 3.2's "zero new tests" discipline was correct for a plugin-skeleton story (nothing to test besides Bevy's own derive). Story 3.3 introduces **literal data invariants** (count, radii, positions, non-overlap) that are uniquely testable as pure-data unit tests:

- **No App harness required:** the tests inspect `ASTEROIDS` const, no ECS / spawn / state-transition machinery.
- **Deterministic:** literal const data → deterministic test outcomes; no flakiness risk.
- **Cheap:** N=17 → 17 + 17 + 17 + 136 (pairs) = 187 inspections, runs in <1 ms.
- **Catches real regressions:** future story authors editing `ASTEROIDS` (e.g., adding an asteroid for 3.5 PlayerShip-clearance tweaks) get immediate feedback if they bust an invariant.

This is precisely the "pure-logic module first-class test target" pattern at architecture.md:351-353. Skipping the tests is a defensible YAGNI call (Story 3.2 precedent), but adding them costs ~30 lines and pays for itself the first time a maintainer asks "why is this asteroid here?". Recommended: include the 4 tests.

### Latest technical information

- **Avian 0.6.1 `RigidBody::Static`** — non-default enum variant (`Dynamic` is `#[default]`); attaches via component spawn — `commands.spawn((RigidBody::Static, Collider::sphere(r), ...))`. Static bodies have infinite mass and are not affected by forces; collisions with dynamic bodies (Story 3.10 projectiles) trigger contact events. [Source: avian3d-0.6.1 `dynamics/rigid_body/mod.rs:284-305`]
- **Avian 0.6.1 `Collider::sphere(radius: f32)`** — 3D constructor (cfg-gated to `feature = "3d"`, which is the project's pinned feature set). Wraps `parry3d::SharedShape::ball(radius)`. [Source: avian3d-0.6.1 `collision/collider/parry/mod.rs:725`]
- **Bevy 0.18 `Sphere::new(r).mesh().ico(n)`** — icosphere mesh builder. `n` is subdivision count; `n=2` yields 80 triangular faces. Returns `Result<Mesh, IcoSphereError>` to signal the (not-applicable-here) `n > MAX_SUBDIVISIONS=80` overflow case. **`unwrap()` is safe at `n=2`.** Smooth interpolated normals automatically — no `generate_outline_normals` call needed for outlines. [Source: deleted `reference_scene.rs:69` precedent + Bevy 0.18 docs]
- **Bevy 0.18 `Mesh3d` + `MeshMaterial3d<M>` components** — replace the legacy `MaterialMeshBundle` from Bevy ≤0.16. The new pattern: spawn `(Mesh3d(mesh_handle), MeshMaterial3d(material_handle), Transform, ...)` as a flat tuple. **Do NOT use `MaterialMeshBundle`, `PbrBundle`, etc.** — those are removed in 0.18. [Source: `src/visual/toon_material.rs` precedent, deleted `reference_scene.rs:74` + Bevy 0.18 release notes]
- **Bevy 0.18 `DirectionalLight`** — Component (not bundle); fields include `illuminance: f32` (lux), `color: Color` (default `WHITE`), `shadows_enabled: bool` (default `false`), `shadow_depth_bias: f32`, `shadow_normal_bias: f32`. The light direction is determined by the entity's `Transform`'s rotation: by default light shines along the entity's local `-Z` axis (use `Transform::default().looking_to(target_dir, up)` to point it at `target_dir`). [Source: Bevy 0.18 `bevy_pbr` docs]
- **`bevy_mod_outline` 0.12 `OutlineVolume`** — Component with `visible: bool`, `width: f32` (logical pixels), `colour: Color` (British spelling). Combines with `OutlinePlugin` (registered in `VisualPlugin::build`). Smooth-normal meshes (icospheres, spheres) render correctly without `generate_outline_normals`; hard-edged meshes (cuboids) require pre-spawn `mesh.generate_outline_normals(...)`. [Source: `bevy_mod_outline-0.12.0/src/lib.rs:217-225`, Story 2.4]
- **Bevy 0.18 `Camera3d::default()`** — Component (replaces `Camera3dBundle`). For multi-camera setups, `Camera { order: i32, ..default() }` controls render order; lower-order cameras render first (used as "background"), higher render on top (used for HUD overlays). Story 3.3 has only one Camera3d, so default `order: 0` is fine. **Story 3.5 must coordinate camera order with 3.3's stand-in or despawn the stand-in.** [Source: deleted `reference_scene.rs:54-63` precedent]

### Previous-story intelligence — what to learn from 2.1 / 2.3 / 2.4 / 3.1 / 3.2

**From Story 2.1 (VisualPlugin + reference_scene):**
- The original `reference_scene.rs` (deleted in 3.1) used `(Mesh3d, MeshMaterial3d<ToonMaterial>, Transform, SemanticAccent, OutlineVolume, ReferenceSceneEntity)` as the asteroid bundle. Story 3.3's bundle is a **superset** of that pattern (same components plus `RigidBody::Static`, `Collider::sphere(radius)`, `ArenaEntity`).
- The `outline_volume` closure pattern in deleted `reference_scene.rs:43-49` is the canonical idiom for cold-start tuning fallback. Story 3.3 uses the identical pattern.

**From Story 2.3 (ToonMaterial implementation):**
- `ToonMaterial::default()` provides reasonable defaults (`tint: WHITE, steps: 4, rim_power: 2.0, rim_intensity: 0.3`). Story 3.3 overrides only `tint`; the rest spread via `..default()`.
- Tuning hot-reload via `apply_tuning_to_toon_materials` mutates `Assets<ToonMaterial>` — Story 3.3's per-asteroid material handles automatically pick up the updates without any per-system wiring.

**From Story 2.4 (bevy_mod_outline integration):**
- `OutlinePlugin` registered in `VisualPlugin::build` since 2.4 — Story 3.3 inherits.
- `apply_tuning_to_outlines` queries `Query<&mut OutlineVolume>` and applies tuning to **every** outline-bearing entity. Story 3.3's new outlines are picked up automatically.
- The `[f32; 4]` → `Color::srgba(r, g, b, a)` destructure pattern at `src/visual/outline.rs:18-20` is what Story 3.3's `outline_volume` closure mimics.

**From Story 3.1 (UiPlugin + cleanup_main_menu + capture/reference_scene teardown):**
- Story 3.1 deleted `src/visual/reference_scene.rs` — Story 3.3 cannot copy code from it directly (file gone), but the patterns are well-documented in 3.1's commit `5146caa` and in the deleted file's git history.
- The marker-only-on-roots cleanup pattern (children cascade via Bevy 0.18 `ChildOf` linked-despawn) is the canonical idiom — Story 3.3's asteroid bundles have **no children** (no `with_children` calls), so the question is moot for asteroids; the DirectionalLight and Camera3d are also leaf entities.

**From Story 3.2 (ArenaPlugin + Avian foundation):**
- `ArenaSystems::Setup` is configured but consumer-less in 3.2; Story 3.3 is the first consumer. The set's purpose materializes here.
- `cleanup_on_exit::<ArenaEntity>` is registered on `OnExit(Arena)` — Story 3.3's spawned entities are caught automatically as long as they carry `ArenaEntity`.
- The **review patch** in 3.2 (commit `5134b3c`) removed a story-id reference from the module doc-comment. Story 3.3's module doc must avoid story-id references too (Bevy 1.5 review patch BH8 + 3.2 patch precedent).
- Two-commit pattern (source + bookkeeping) is now the project default.

### Forward compatibility — Story 3.4 (pause overlay) hand-off

Story 3.4 introduces `GameState::Paused` and the Esc-pause / focus-loss-pause flow. **Story 3.3 has no direct hand-off to 3.4** — pausing freezes the simulation but does NOT despawn arena entities (Bevy's `Time::<Virtual>::pause()` keeps the world intact). The `Arena → Paused` transition uses `OnExit(Arena)` only if 3.4 chooses to push state; if 3.4 uses Bevy's standard pause-via-time-control, asteroids remain spawned and the cleanup_on_exit is NOT triggered. Story 3.4's author should consult `architecture.md:218-220` and the epic-3 spec for the canonical pause approach.

### Forward compatibility — Story 3.5 (PlayerShip + cockpit camera) hand-off

Story 3.5 will:
- Spawn a `PlayerShip` entity at `Vec3::ZERO` (or near origin) with `RigidBody::Dynamic`, a placeholder mesh, and a `Collider`.
- Spawn a child `Camera3d` with `CockpitCamera` marker, positioned at the pilot-seat wingtip-framing position.
- **Despawn or coexist-with** Story 3.3's stand-in `Camera3d`. **Recommended: despawn at top of 3.5's spawn system** via `Query<Entity, (With<Camera3d>, With<ArenaEntity>, Without<CockpitCamera>)>`; alternative: spawn the cockpit camera with `order: 0` and the stand-in stays at default `order: 0` → ambiguous; not recommended.
- **Verify line-of-sight contract:** Story 3.3's `ASTEROIDS` const places **5 asteroids within 50 m of origin**, satisfying Story 3.5 AC #3.5 ("≥ 3 asteroids within 50 m"). Story 3.5's author should not need to modify `ASTEROIDS`.

The deferred-work entry from Task 5 carries this contract forward; 3.5's author reads it before authoring.

### Forward compatibility — Stories 3.6–3.8 (flight input, dampener) hand-off

3.6 (translation), 3.7 (rotation), 3.8 (dampener) all attach `ExternalForce` / `ExternalTorque` to the `PlayerShip` from 3.5. **Story 3.3's static asteroids are passive obstacles** for these stories — they participate in collision detection (Story 3.10 wires the projectile-asteroid contact event handler) but apply no forces themselves. No 3.3 work is needed for 3.6–3.8.

### Forward compatibility — Story 3.9 (projectiles) + Story 3.10 (collision) hand-off

Story 3.9 spawns `Projectile` entities with `RigidBody::Dynamic` + `Collider`. Story 3.10 wires the projectile-asteroid contact event:
- Story 3.10 will extend `ASTEROIDS` spawn loop to also attach `AsteroidHp { current: 1 }` to each asteroid bundle. **Story 3.3 does NOT pre-add `AsteroidHp`** — that's 3.10's component addition (single-line edit to the spawn loop).
- Story 3.10's `ProjectileHitAsteroid` event needs a way to read which asteroid was hit; the asteroid `Entity` (already a Bevy fundamental) suffices — no extra component needed in 3.3.
- Avian's `CollisionLayers` setup (which projectiles can hit which asteroids) is 3.10's concern; 3.3 spawns asteroids without `CollisionLayers`, accepting Avian's default "all-layers-collide" behavior. Story 3.10 will add explicit layers.

### Forward compatibility — Story 3.11 (HUD baseline) hand-off

Story 3.11 spawns `HudEntity`-tagged screen-space UI nodes on `OnEnter(Arena)`. **Story 3.3 has no direct hand-off** — HUD setup is independent of the asteroid field. Both spawn systems run on `OnEnter(Arena)` in `ArenaSystems::Setup` (3.3) and likely a parallel `HudSystems::Setup` set introduced by 3.11 (or 3.11 also reuses `ArenaSystems::Setup` — that's 3.11's call).

### Project structure notes

- **Path alignment:**
  - `src/arena/zone.rs` is **NEW**. Architecture.md:582-588 doesn't list `src/arena/zone.rs` (it lists `src/run/arena.rs` for the Arena tutorial — different file path). Story 3.2's Dev Notes already reconciled `src/arena/` as a legitimate leaf for Arena-state lifecycle code. Story 3.3 follows the same precedent: `src/arena/zone.rs` is a child of the established `src/arena/` subtree.
  - `src/arena/mod.rs` is **MODIFIED** in-place; net +5 lines.
  - `src/visual/palette.rs` is **MODIFIED** in-place; net −8 lines.
- **No path conflicts** introduced by Story 3.3.
- **`src/main.rs` is UNCHANGED.** All wiring stays inside `ArenaPlugin::build` (which is registered in main.rs since 3.2).
- **Splash file location debt re-deferred** — `src/splash.rs` stays flat at `src/`. 3.3 does NOT touch splash.
- **Architecture path discrepancy** (`src/arena/` not in tree) is now Story-3.2-precedent-resolved; Story 3.3 inherits without re-litigating.
- **`Cargo.toml`, `Cargo.lock`, `.gitignore`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `assets/**`, `docs/**` — UNTOUCHED.**

### LLM dev-agent guardrails — most-likely-to-go-wrong patterns

1. **Forgetting `.in_set(ArenaSystems::Setup)` on `add_systems`.** Without it, `spawn_arena_zone` still runs (because `OnEnter(Arena)` fires once on state entry), but it's not in the SystemSet — future stories ordering against the set will silently miss it. Always include `.in_set(ArenaSystems::Setup)`.
2. **Using `RigidBody::Dynamic` instead of `RigidBody::Static` for asteroids.** Dynamic asteroids would drift due to (zero-g) integration round-off and emit `WARN: penetration` on first physics tick from any pre-existing micro-overlap. Static is correct.
3. **Missing `Collider::sphere(radius)` to match the visual radius.** The for-loop binds `position` and `radius` from the same tuple — using a different value for the collider (e.g., a hardcoded `Collider::sphere(5.0)`) breaks AC #2 "visual radius == physics radius" + Story 3.10 trustworthy collision.
4. **Spawning `OutlineVolume` on the DirectionalLight or Camera3d.** `OutlineVolume` is a mesh-geometry component; non-mesh entities don't render through the outline pipeline, so `bevy_mod_outline` may emit warnings. Outline only on the asteroid loop.
5. **Calling `mesh.generate_outline_normals(...)` on icospheres.** Icospheres have smooth interpolated normals; the call is unnecessary and adds 80-vertex CPU work × 17 asteroids = ~1360 vertex normals re-computed for nothing. Skip the call.
6. **Picking `Sphere::new(r).mesh().build()` (UV sphere) instead of `.ico(2)` (icosphere).** UV spheres have visible "polar streaking" under toon shading because their tessellation density varies with latitude. Icospheres distribute vertices uniformly → uniform posterization band thickness. Use `.ico(2)`.
7. **Hardcoding `Color::WHITE` or `Color::srgb(0.5, 0.5, 0.5)` for the toon tint instead of `color_for(SemanticAccent::Neutral)`.** The whole point of using `SemanticAccent::Neutral` (AC #2) is that colorblind redundancy is encoded centrally in palette.rs. Hardcoding bypasses the architectural commitment AND keeps the cfg_attr-removal AC #7 unfulfilled.
8. **Using `RigidBody` without `RigidBody::Static`.** `RigidBody` is an enum; `commands.spawn((RigidBody, ...))` is a type error. Always specify the variant: `RigidBody::Static`.
9. **Importing `avian3d::prelude::*` (wildcard).** Project import discipline (per `src/main.rs:5`) prefers selective imports: `use avian3d::prelude::{Collider, RigidBody};`. Wildcard pollutes the symbol surface.
10. **Adding `AmbientLight::default()` or any `AmbientLight` resource override.** AC #3 explicitly requires NO ambient light beyond Bevy defaults. Adding one washes out the toon shading. Skip.
11. **Putting `spawn_arena_zone` in `Update` or `Startup`.** Must be `OnEnter(GameState::Arena)`. Other schedules either re-fire every frame (Update) or fire once at app start before any state machine engages (Startup, before `Loading`).
12. **Using `Camera2d` instead of `Camera3d`.** Camera2d renders 2D quads; the asteroid mesh is 3D, will render to nothing. Always Camera3d for 3D scenes.
13. **Touching `src/main.rs`.** Story 3.3 makes ZERO edits to main.rs. All wiring is inside `ArenaPlugin::build` via `pub mod zone;` + `add_systems` line in `src/arena/mod.rs`. If you find yourself editing main.rs, stop and reconsider.
14. **Touching `Cargo.toml`.** No version bumps, no feature additions. All deps already pinned.
15. **Editing `assets/config/tuning.ron` and committing it.** Manual hot-reload smoke per Task 4 may temporarily edit the file to validate `apply_tuning_to_outlines` propagation; **revert before commit**. Tuning.ron is a runtime asset, not a Story 3.3 deliverable.
16. **Skipping the `cargo build --release` AC #7 verification.** Without the release build, the `cfg_attr` removal in Task 3 isn't validated. The release-build step is fast (3.2 paid the cold-build cost; expect 4-min on warm cache) and load-bearing for AC #7's symbol-survival check.
17. **Skipping the runtime smoke (Task 4 `cargo run`).** Without it, you don't catch (a) Bevy ambiguous-camera-order warnings, (b) Avian penetration warnings if asteroids overlap, (c) shader compile failures, (d) outline render failures. The smoke is fast (~10 s of human time after the build).
18. **Adding more than 25 asteroids "for variety".** AC #2 caps at 25. Adding more triggers the `asteroid_count_in_acceptance_range` test to fail (if added) and violates the AC explicitly. Stay in [15, 25].
19. **Spawning asteroids inside the camera's near plane.** Default Camera3d near plane is 0.1 m; asteroids at `Vec3::new(0, 5, 79)` (just in front of camera at `(0, 5, 80)`) would pop in/out as camera moves. Story 3.5's PlayerShip motion will move the camera; defensively keeping asteroids at `|z|` ≥ 15 m from origin avoids near-plane clipping.
20. **Touching `_bmad-output/planning-artifacts/**`.** Read-only from story-execution perspective.

### `cfg_attr` removal — the long-deferred consumer arrives

The story-2.2-introduced `#[cfg_attr(not(debug_assertions), allow(dead_code, reason = "no current consumer; gameplay consumer arrives in Story 4.5"))]` annotations on `SemanticAccent` (palette.rs:7-13) and `color_for` (palette.rs:23-29) — re-deferred from 2.3 to "Story 4.5" per deferred-work.md:102 — find their **first non-cfg-gated consumer** in Story 3.3.

The deferral chain was:
- **Story 2.2 (palette introduced)** — no consumer at all → `#[cfg(test)]` only? No, the swatches in `reference_scene.rs` were the dev-time consumer; they were `#[cfg(debug_assertions)]`-gated, hence the cfg_attr.
- **Story 2.3 (toon material)** — toon material consumed `color_for` only inside `reference_scene.rs` (still cfg-gated), so `color_for` remained release-DCE-eligible. Annotations stayed.
- **Story 4.5 (planned)** — the original target for "SemanticAccent on gameplay entities (asteroids, salvage, enemies, projectiles, playership)" was Epic-4-spec-named Story 4.5.
- **Story 3.3 (this story)** — Epic 3 spec line 67 ("each asteroid uses ToonMaterial from Epic 2 with SemanticAccent::Neutral") makes Story 3.3 the **earliest** non-cfg-gated consumer. The cfg_attr annotations should drop here, not wait for 4.5.

This is an example of the deferral chain compressing organically as the implementation proceeds. AC #7 + Task 3 + Task 5's deferred-work resolution all land together.

### Why bundle camera + light + asteroids in one story

Three alternatives were considered:

**Alternative A (rejected): asteroids-only — defer camera + light to 3.5.**
- Pro: very narrow diff (~70 lines for asteroid spawn loop + const).
- Con: AC #8's runtime smoke can't visually verify the field. Player sees a black screen on Arena entry → hard to distinguish "spawn ran but rendered nothing" from "spawn didn't run." Bad signal.
- Con: bundling DirectionalLight with the field is conceptually clean — the light is *for* the field; separating them creates an awkward "lighting-only intermission story."

**Alternative B (rejected): asteroids + light, NO camera — defer camera entirely to 3.5.**
- Pro: cleaner separation of concerns.
- Con: same visual-verification gap as Alternative A. Renders to nothing without a camera.

**Alternative C (selected): asteroids + light + stand-in camera in one story.**
- Pro: matches the epic-3 AC verbatim (the AC text doesn't mention camera, but visual-verification ACs imply it's needed).
- Pro: produces a coherent post-commit state — Arena entry shows a populated, lit scene with a default camera framing it. Story 3.5 can replace the camera incrementally without preamble.
- Pro: single ~150-line file is easier to review than three smaller files.
- Con: the stand-in camera creates a hand-off contract for Story 3.5 (mitigated by Task 5 deferred-work entry).

### References

- [Source: [`_bmad-output/planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md:57-81`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md)] — Story 3.3 epic spec.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:240`](../planning-artifacts/architecture.md)] — Avian FixedUpdate at 60 Hz decision.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:415-420`](../planning-artifacts/architecture.md)] — `cleanup_on_exit::<ArenaEntity>` pattern.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:343-350`](../planning-artifacts/architecture.md)] — Plugin-per-feature module pattern + `<Feature>Systems` SystemSet.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:660-664`](../planning-artifacts/architecture.md)] — Cross-cutting Resources read in main.rs (here: `TuningHandle` + `Assets<TuningConfig>` are read-only in `spawn_arena_zone`).
- [Source: [`_bmad-output/planning-artifacts/architecture.md:355-359`](../planning-artifacts/architecture.md)] — TuningConfig hot-reload pattern; runtime-tunable gameplay values.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:582-588`](../planning-artifacts/architecture.md)] — Source-tree layout (note: `src/arena/zone.rs` not listed; reconciled by Story 3.2 precedent).
- [Source: [`_bmad-output/planning-artifacts/prd.md`](../planning-artifacts/prd.md)] — FR2 6-DOF translation (3.6 consumer), FR8 cockpit-only (3.5 consumer), FR12 projectile damage (3.10 consumer), FR49 toon shader (this story exercises it on ≥15 entities), FR50 SemanticAccent (this story exercises Neutral).
- [Source: [`Cargo.toml:8-12`](../../Cargo.toml)] — bevy 0.18 + avian3d 0.6 + bevy_mod_outline 0.12 + bevy_kira_audio 0.25 + leafwing-input-manager 0.20 pinned versions.
- [Source: [`src/main.rs`](../../src/main.rs)] — current plugin-registration block (post-3.2; unchanged by 3.3).
- [Source: [`src/arena/mod.rs`](../../src/arena/mod.rs)] — `ArenaPlugin` + `ArenaEntity` + `ArenaSystems::Setup` + `cleanup_on_exit::<T>` from Story 3.2.
- [Source: [`src/visual/mod.rs:18-37`](../../src/visual/mod.rs)] — `MaterialPlugin<ToonMaterial>` + `bevy_mod_outline::OutlinePlugin` registration; `apply_tuning_to_toon_materials` + `apply_tuning_to_outlines` propagation.
- [Source: [`src/visual/palette.rs:6-32`](../../src/visual/palette.rs)] — `SemanticAccent` enum + `color_for` function; AC #7 deletion target.
- [Source: [`src/visual/toon_material.rs:11-32`](../../src/visual/toon_material.rs)] — `ToonMaterial` struct + `Default` impl.
- [Source: [`src/visual/outline.rs:14-27`](../../src/visual/outline.rs)] — `apply_tuning_to_outlines` system; the Tuning→OutlineVolume propagation that Story 3.3 inherits without modification.
- [Source: [`src/tuning/mod.rs:17,38-40`](../../src/tuning/mod.rs)] — `TuningHandle` Resource + `load_tuning` Startup system.
- [Source: [`src/tuning/config.rs:11-29`](../../src/tuning/config.rs)] — `TuningConfig` struct (outline_width: f32, outline_color: [f32; 4]) + `Default` impl.
- [Source: [`_bmad-output/implementation-artifacts/3-2-avian-physics-foundation-arena-state-skeleton.md:478-492`](./3-2-avian-physics-foundation-arena-state-skeleton.md)] — Story 3.3 forward-compat hand-off describing what 3.3 will do.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:139`](./deferred-work.md)] — `VisualSystems::Setup` empty-no-op cleanup (resolved by Story 3.3 with deviation).
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:100-102`](./deferred-work.md)] — Story 4.5 cfg_attr re-deferral (resolved by Story 3.3 ahead of schedule).
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:137-138`](./deferred-work.md)] — Splash race + location debt (re-deferred for 3.3).
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:148`](./deferred-work.md)] — Story 3.2 review finding "no enforcement that arena-spawned entities carry the ArenaEntity marker" (Story 3.3 honors the convention; verification check `grep -rn 'ArenaEntity' src/` per the deferral's resolution path).
- [Source: avian3d-0.6.1 source — `dynamics/rigid_body/mod.rs:284-305`] — `RigidBody` enum.
- [Source: avian3d-0.6.1 source — `collision/collider/parry/mod.rs:725`] — `Collider::sphere(radius: f32)`.
- [Source: bevy_mod_outline-0.12.0 source — `lib.rs:217-225`] — `OutlineVolume` struct.
- [Source: deleted `src/visual/reference_scene.rs` (commit `5146caa^:src/visual/reference_scene.rs`)] — historical asteroid + outline + toon-material spawn pattern from Story 2.1; Story 3.3 inherits the pattern's ECS shape minus the M1-tech-spike scaffolding.
- [Source: [`MEMORY.md` → `feedback_full_build_output.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_full_build_output.md)] — verification discipline: exit-0 + tail is NOT proof; grep explicitly.
- [Source: [`MEMORY.md` → `feedback_compact_review_style.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_compact_review_style.md)] — Till's compact-review style (single-line responses; no required elaboration).
- [Source: [`MEMORY.md` → `feedback_staged_rollout.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_staged_rollout.md)] — staged-rollout preference; informs scope-bundling rationale.
- [Source: [`MEMORY.md` → `project_principle3_deferred.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/project_principle3_deferred.md)] — Asteroid motion (Kepler/spline) is post-MVP item #11; **MVP ships static asteroids per PRD amendment 2026-04-22**. Story 3.3's `RigidBody::Static` choice aligns with this principle.

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Opus 4.7, 1M context)

### Debug Log References

Local verification sweep (all logs in `/tmp/`):

| Command | Log file | `grep -cE 'warning:\|error:'` | Notes |
|---|---|---|---|
| `cargo check` | `/tmp/story-3-3-check.log` | 0 | 0.14s; rust-analyzer pre-warmed cache |
| `cargo build` (debug) | `/tmp/story-3-3-build.log` | 0 | 2.16s; avian3d + bevy_mod_outline already in cache from 3.2 |
| `cargo test` | `/tmp/story-3-3-test.log` | 0 (also 0 `FAILED`) | `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Test count: 14 baseline + 5 new (asteroid_count_in_acceptance_range, asteroid_radii_within_3_to_12, asteroid_positions_within_volume, asteroid_colliders_do_not_overlap, at_least_three_asteroids_within_50m_of_origin) |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-3-3-clippy.log` | 0 | 0.47s; 0 dead_code complaints on `color_for` (consumed by `spawn_arena_zone`); `#[allow]` preserved on the SemanticAccent enum suppresses the per-variant warnings on Enemy/Salvage/Hazard/PlayerOwned |
| `cargo fmt --all -- --check` | exit code | 0 | After one `cargo fmt --all`: rustfmt collapsed `ASTEROIDS.iter().filter(...).count()` from a 4-line builder to a single line (Task 1 unit-test reformatting) |
| `cargo build --release` | `/tmp/story-3-3-release.log` | 0 | 4m 06s (LTO=fat + codegen-units=1, full re-link); critical AC #7 evidence — zero dead_code warnings on `color_for` confirms reachability |
| `cargo run` runtime smoke | `/tmp/story-3-3-run.log` | n/a | Launched via `cargo run` (NOT direct binary execution — release binary's CWD-based asset lookup misses `target/release/assets/`; `cargo run` sets CWD to project root so `assets/` resolves correctly). Cycle Loading→MainMenu→Arena confirmed; window terminated via SIGINT after Arena entry. |

**Runtime-smoke lifecycle counts** (per AC #8 grep harness):

| Marker | Count | Expected |
|---|---|---|
| `entered Loading` | 1 | 1 |
| `splash timer elapsed` | 1 | 1 |
| `entered MainMenu` | 1 | 1 |
| `entered Arena` | 1 | 1 |
| `panic\|backtrace\|FATAL` | 0 | 0 |
| `penetration` (Avian collider overlap) | 0 | 0 |
| Avian WARN/ERROR | 0 | 0 |
| `Path not found` (asset resolution) | 0 | 0 |
| `ambiguous` (camera order) | 0 | 0 |
| `TuningReloaded` (palette tuning loaded) | 1 | 1 |

**Documented (non-3.3-regression) WARNs in run log** — consistent with deferred-work.md:75-76, :137, and Story 1.6 LOW-1:

1. `bevy_ecs::error::handler: Encountered an error in command ... Entity despawned: ID 87v0 invalid; generation 1` — splash-cleanup race observed during 2.1 dev-verification, re-deferred to next splash-touching story.
2. `bevy_winit::state: Skipped event Destroyed for unknown winit Window Id ...` — close-time noise from Bevy 0.18 winit-event race; not Story-1.6/3.3-introduced.
3. `wgpu_core::device::resource: The fragment stage "fragment" output @location(0) values are ignored` — pre-existing wgpu warning from Story 2.3's ToonMaterial fragment shader output binding; not 3.3-introduced.

**ArenaEntity convention check** (per deferred-work.md:148 resolution path):

```
$ grep -c 'ArenaEntity' src/arena/zone.rs
5
$ grep -c 'commands.spawn(' src/arena/zone.rs
3
$ grep -n 'ArenaEntity' src/arena/zone.rs
2://! ... cleanup_on_exit::<ArenaEntity>.
8:use super::ArenaEntity;
67:        ArenaEntity,    # Camera3d spawn
78:        ArenaEntity,    # DirectionalLight spawn
98:            ArenaEntity, # asteroid loop spawn
```

All 3 spawn sites carry `ArenaEntity`. Convention upheld.

**Release-binary symbol survival (AC #7) — partial fulfillment:**

```
$ nm target/release/asteroids3D | grep -c color_for
0
$ nm target/debug/asteroids3D | grep -c color_for
1
```

Release returns **0** because Cargo.toml's `lto = "fat"` + `codegen-units = 1` inlines `color_for` (single-match function) into `spawn_arena_zone` and DCEs the standalone symbol. The dev binary returns **1**, confirming reachability before LTO. The authoritative reachability oracle is rustc's dead_code lint, which emits **0** complaints on `color_for` in `cargo build --release` (would have fired if the function were genuinely unreachable). AC #7's `nm ≥ 1` literal threshold is therefore unfulfillable under the project's release LTO config; the AC's architectural intent (palette is reachable in non-cfg-gated paths) IS fulfilled. Documented as a follow-up correction to deferred-work.md (new entry "`nm color_for ≥ 1` proxy unreliable under release LTO").

### Completion Notes List

- **AC #1** ✓ — `src/arena/zone.rs` authored (168 lines including module doc + 5 unit tests). `pub mod zone;` added at line 4 of `src/arena/mod.rs`. `ArenaPlugin::build` registers `zone::spawn_arena_zone.in_set(ArenaSystems::Setup)` on `OnEnter(GameState::Arena)` (`src/arena/mod.rs:21-24`).

- **AC #2** ✓ — `ASTEROIDS` const-array contains 17 entries (within [15, 25]). Each tuple is `(Vec3, f32)` with positions strictly within ±100 m on each axis (verified `asteroid_positions_within_volume` test) and radii in [3.0, 12.0] (verified `asteroid_radii_within_3_to_12` test). Each asteroid spawns with: `Mesh3d(Sphere::new(radius).mesh().ico(2).unwrap())`, `MeshMaterial3d<ToonMaterial>` with `tint = color_for(SemanticAccent::Neutral).into()` and `..default()`, `Transform::from_translation(position)`, `SemanticAccent::Neutral`, `RigidBody::Static`, `Collider::sphere(radius)` (visual radius == physics radius), `OutlineVolume` from current `TuningConfig`, and `ArenaEntity` marker. Pairwise non-overlap proven by `asteroid_colliders_do_not_overlap` test (136 pair-distance assertions all ≥ sum-of-radii).

- **AC #3** ✓ — Exactly one `DirectionalLight` spawned at `src/arena/zone.rs:71-79` with `illuminance: 5_000.0`, `shadows_enabled: false`, and `Transform::default().looking_to(Vec3::new(-0.3, -1.0, -0.4).normalize(), Vec3::Y)` for non-axis-aligned light direction. No `AmbientLight` resource override; no PointLight/SpotLight/secondary DirectionalLight added. The `ArenaEntity` marker is attached.

- **AC #4** ✓ — Exactly one `Camera3d` spawned at `src/arena/zone.rs:64-68` at `Transform::from_xyz(0.0, 5.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y)`. Tagged `ArenaEntity`. No `CockpitCamera` marker (deferred to Story 3.5). No Camera2d spawned in this story.

- **AC #5** ✓ — Story 3.2's `cleanup_on_exit::<ArenaEntity>` (registered at `src/arena/mod.rs:25-28`) catches all `ArenaEntity`-marked spawns from this story (17 asteroids + 1 light + 1 camera = 19 entities). Cross-state cleanup smoke is **structural only** — no Arena-exit UI path exists in 3.3 scope (Story 3.4 introduces Esc-pause; Story 4.7 introduces post-run flow). Dynamic verification deferred to whichever future story first exercises an Arena-exit path.

- **AC #6** ✓ — `spawn_arena_zone` registered in `ArenaSystems::Setup` (NOT `VisualSystems::Setup`). Rationale: Story 3.2's design intent (set declared specifically for OnEnter(Arena) work; arena-zone spawning is arena-state-bound, not visual-pipeline setup). `VisualSystems::Setup` declaration in `src/visual/mod.rs:13-16, 23-26` left intact; deletion deferred to a follow-up cleanup chore. Deferred-work.md:139 entry updated with a "🔁 RE-DEFERRED 2026-04-30 by Story 3.3" annotation explaining the deviation.

- **AC #7** ✓ (partial; surgical outcome) — `#[allow(dead_code, ...)]` block on `pub fn color_for` (palette.rs:23-29) **deleted**. `#[allow]` block on `pub enum SemanticAccent` (palette.rs:7-10) **preserved with updated reason** (`"Neutral consumed by spawn_arena_zone; Enemy/Salvage/Hazard/PlayerOwned variants pending consumer in Story 4.5"`) because the four non-Neutral variants still have no non-cfg-gated consumer (rustc's dead_code lint fires per-variant when an enum has any unused variants — Story 3.3 only consumes `Neutral`). `cargo build --release` clean (0 warnings). `nm color_for ≥ 1` **unfulfilled** under release LTO (LTO=fat inlines the function); reachability evidence shifted to dev-binary `nm = 1` + zero release-build dead_code warnings. Deferred-work.md:100-102 entry updated with "✅ PARTIALLY RESOLVED 2026-04-30 by Story 3.3"; new deferred entry filed re. the `nm`-proxy unreliability.

- **AC #8** ✓ — All 6 cargo commands report 0 warnings/errors per the per-command grep. Test count: 19 (= 14 baseline + 5 new asteroid-invariant tests, within the AC #8 N ∈ {14, 15, 16, 17, 18} range — actually one over, justified by including the at_least_three_asteroids_within_50m_of_origin test for Story 3.5 hand-off contract). `cargo fmt --all -- --check` exit 0. Runtime smoke shows 4×1 lifecycle markers + 0 panics + 0 penetration + 0 Avian errors + 0 path-not-found + 0 camera ambiguity. Git status final delta exactly matches AC #8 expectations.

- **AC #9** ✓ — Each asteroid spawn includes `OutlineVolume` constructed from the current `TuningConfig` outline values via the cold-start-safe closure pattern (read `tuning_assets.get(...).cloned().unwrap_or_default()`). The existing `apply_tuning_to_outlines` system at `src/visual/outline.rs:14` automatically propagates hot-reloaded outline width/colour to the new asteroid `OutlineVolume`s via its `Query<&mut OutlineVolume>` query — no per-asteroid wiring added. No `generate_outline_normals(...)` call (icospheres have smooth interpolated normals).

**Deviations:**

- **5 unit tests added (vs spec's 0–4 budget).** Story 3.3 spec Task 1 listed 4 optional tests; I added a 5th (`at_least_three_asteroids_within_50m_of_origin`) because the Story 3.5 forward-compat hand-off ("≥ 3 asteroids within 50 m for line-of-sight") is a load-bearing layout invariant, and an explicit test prevents future `ASTEROIDS` edits from silently violating it. Net post-3.3 test count: **19** (= 14 baseline + 5 new).
- **AC #7 nm-proxy unreliable.** AC text expected `nm color_for ≥ 1`; LTO inlining returns 0. Substituted: dev-binary `nm = 1` + zero release dead_code warnings as the reachability evidence. Documented in Completion Notes + new deferred-work entry.
- **AC #7 surgical (not full) `#[allow]` removal.** Spec assumed deleting both annotation blocks would clear all dead_code warnings. Empirically, only the function annotation could be deleted; the enum annotation must survive until Story 4.5 wires the remaining four variants. Adjusted in-flight per the AC #7 contingency clause.
- **Two-commit push — NOT YET EXECUTED.** Per project rules (Story 3.2 precedent, Story 1.7/2.4/2.5/2.6/3.1 precedent), commits and pushes await Till's explicit authorization. The Task 6 subtasks "Commit 1" + "Commit 2" remain unchecked deliberately; Dev Agent Record + Status flip + sprint-status update are saved without staging or pushing.
- **Runtime smoke via `cargo run`, NOT direct binary execution.** First attempt ran the release binary directly (`./target/release/asteroids3D`); Bevy's asset search resolved relative to the binary location and missed `target/release/assets/` (path doesn't exist). Switched to `cargo run` which sets CWD to project root, allowing `assets/config/tuning.ron` and `assets/shaders/toon.wgsl` to resolve correctly. The runtime evidence in `/tmp/story-3-3-run.log` is from the cargo-run invocation; the direct-binary run was a verification-tooling false negative on asset paths (not a 3.3 regression).

### File List

**Added:**

- `src/arena/zone.rs` (new file; 168 lines after rustfmt — system + ASTEROIDS const + 5 unit tests)

**Modified:**

- `src/arena/mod.rs` (+5 net lines: `pub mod zone;` declaration + `add_systems(OnEnter(Arena), zone::spawn_arena_zone.in_set(ArenaSystems::Setup))` block)
- `src/visual/palette.rs` (−4 net lines: `#[allow]` block on `pub fn color_for` deleted; `#[allow]` block on `pub enum SemanticAccent` preserved with updated `reason` text)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (3-3 status flip ready-for-dev → in-progress → review; `last_updated` bump)
- `_bmad-output/implementation-artifacts/3-3-hand-designed-arena-zone-with-static-asteroid-field.md` (this file: tasks/subtasks checked except Commit 1/Commit 2 awaiting authorization, Dev Agent Record populated, Status → review)
- `_bmad-output/implementation-artifacts/deferred-work.md` (resolved Story 2.3 cfg_attr re-deferral with PARTIALLY-RESOLVED note; resolved Story 3.1 VisualSystems::Setup entry with RE-DEFERRED note; appended HONORED-FOR-3.3 note under 3.2 review entry; appended new "Deferred from: 3-3-..." section with 5 entries — VisualSystems cleanup chore, Camera3d replacement contract for 3.5, partial SemanticAccent un-annotation, nm-proxy unreliability, asteroid layout drift hazard, splash race re-deferred)

**Untouched (verified):** `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `src/state.rs`, `src/splash.rs`, `src/logging.rs`, `src/ui/**`, `src/visual/mod.rs`, `src/visual/outline.rs`, `src/visual/toon_material.rs`, `src/tuning/**`, `assets/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.gitignore`.

### Review Findings

**Review Date:** 2026-04-30 | **Reviewers:** Blind Hunter (adversarial), Edge Case Hunter, Acceptance Auditor | **Dismissed:** 18

- [ ] [Review][Decision] AC #7 partial — `#[allow]` on `SemanticAccent` updated instead of deleted — AC #7 requires "delete BOTH blocks," but deleting the SemanticAccent block causes 4 dead_code warnings (Enemy/Salvage/Hazard/PlayerOwned variants still unused). Current state: updated reason string + documented in deferred-work.md. Accept as-is, or try per-variant suppression (`#[allow(dead_code)]` on individual variants)? [`src/visual/palette.rs:6-10`]
- [ ] [Review][Decision] AC #7/Task 5 — deferred-work.md uses "PARTIALLY RESOLVED" not spec-prescribed "RESOLVED" — Factually more accurate (one annotation block survives for Story 4.5), but literal AC deviation. Accept "PARTIALLY RESOLVED" for audit clarity, or change to "RESOLVED"? [`_bmad-output/implementation-artifacts/deferred-work.md`]
- [x] [Review][Patch] Missing `warn!` log when tuning.ron not loaded at Arena entry — fixed: `warn!("tuning.ron not loaded at Arena entry; using TuningConfig defaults")` added before `.unwrap_or_default()`. [`src/arena/zone.rs:50-55`]
- [x] [Review][Patch] `.unwrap()` on `ico(2)` should use `.expect()` — fixed: `.expect("ico(2): subdivision=2 is within MAX_SUBDIVISIONS=80")`. [`src/arena/zone.rs:85-89`]
- [x] [Review][Defer] `despawn()` not `despawn_recursive()` in `cleanup_on_exit` — pre-existing pattern from Story 3.2; currently safe (no ArenaEntity has children); latent risk if future stories attach child entities to arena roots. [`src/arena/mod.rs:34`] — deferred, pre-existing
- [x] [Review][Defer] Unique `Mesh` handle per asteroid — all 17 radii are currently distinct so no GPU-upload duplication occurs; future optimization if asteroid variants share a radius. [`src/arena/zone.rs:85`] — deferred, pre-existing
- [x] [Review][Defer] Unique `ToonMaterial` handle per asteroid (all 17 identical) — 17 separate material assets where 1 shared handle would suffice; minor draw-call overhead for 17 static objects. [`src/arena/zone.rs:86-89`] — deferred, pre-existing

### Change Log

| Date | Change | Story |
|---|---|---|
| 2026-04-30 | Hand-designed Arena zone shipped: 17 static asteroids (icosphere meshes + ToonMaterial Neutral tint + RigidBody::Static + Collider::sphere + OutlineVolume + ArenaEntity), single non-axis-aligned DirectionalLight, stand-in Camera3d at (0, 5, 80) framing origin. `spawn_arena_zone` registered in `ArenaSystems::Setup` on `OnEnter(Arena)`. `#[allow(dead_code)]` removed from `color_for` (palette.rs); preserved on SemanticAccent enum until Story 4.5 wires remaining variants. 5 invariant unit tests added (count, radii, volume, non-overlap, line-of-sight). Test count 14 → 19. Three deferred-work entries resolved/updated; one new deferred-work section opened with five entries. | 3.3 |
