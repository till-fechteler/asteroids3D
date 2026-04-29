# Story 2.4: bevy_mod_outline Integration + Wiring

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want `bevy_mod_outline` (v0.12, Bevy-0.18-compatible) wired into `VisualPlugin` with `OutlineVolume` components attached to every toon-shaded mesh in the reference scene, plus `outline_width` + `outline_color` fields hot-reloadable through the existing `TuningConfig`,
So that FR49's silhouette outlines render consistently without per-entity hardcoding and the dev can iterate on outline width/color in <1 s without recompile.

## Acceptance Criteria

1. **Given** `bevy_mod_outline::OutlinePlugin` is added to `VisualPlugin`'s `build()`
   **When** `cargo run` (debug) and `cargo build --release` are executed
   **Then** the app launches without panic on the dev machine
   **And** the plugin's render-graph nodes (`MsaaExtraWritebackPass`, `OutlinePass`, `EndOutlinePasses`) hook into `Core3d` after `Tonemapping` and before any later AA per the plugin's documented schedule

2. **Given** `TuningConfig` is extended with `outline_width: f32` and `outline_color: [f32; 4]` (sRGBA) loaded from `assets/config/tuning.ron`
   **When** the reference scene spawns its three placeholders on `OnEnter(GameState::Loading)`
   **Then** each placeholder is spawned with an `OutlineVolume { visible: true, width, colour }` whose `width` and `colour` derive from `TuningConfig` defaults at spawn-time

3. **Given** outlines are applied to the asteroid (icosphere), ship (cuboid), and projectile (UV-sphere)
   **When** the reference scene renders in a debug build
   **Then** each placeholder shows a continuous silhouette outline visible against the dark scene background
   **And** the cuboid ship's outline shows no hard-edge spike artefacts (because `OutlineMeshExt::generate_outline_normals` is invoked on the cuboid mesh before insertion into `Assets<Mesh>`)
   **And** outlines do not z-fight with mesh surfaces at the default camera distance (`Transform::from_xyz(0.0, 1.5, 6.0)`)

4. **Given** `assets/config/tuning.ron` is edited at runtime with the dev `cargo run` build still running
   **When** `outline_width: 3.0` is changed to `outline_width: 6.0` and saved
   **Then** within ~1 second the running reference scene updates to thicker outlines on all three placeholders without restart
   **And** when `outline_color` is edited (e.g. `[0.05, 0.05, 0.05, 1.0]` → `[1.0, 0.0, 0.0, 1.0]`), all outlines visibly turn red within ~1 second

5. **Given** `TuningConfig::default()` and `assets/config/tuning.ron` initial values
   **When** the unit tests `tuning_config_default_matches_ron_initial_values` and `tuning_config_deserializes_from_ron_bytes` run via `cargo test`
   **Then** both pass with the new `outline_width` + `outline_color` fields covered
   **And** a new test asserts that a RON byte slice **without** `outline_width` / `outline_color` (legacy 2.3 schema) still deserializes via `#[serde(default)]` — the forward-compat contract for Story 4.x's future field additions

## Tasks / Subtasks

- [x] **Task 1: Extend `TuningConfig` schema with outline fields** (AC: #2, #5)
  - [ ] In `src/tuning/config.rs`, add to the `TuningConfig` struct:
    ```rust
    #[derive(Asset, TypePath, Debug, Clone, Deserialize)]
    pub struct TuningConfig {
        pub toon_steps: u32,
        pub toon_rim_power: f32,
        pub toon_rim_intensity: f32,
        // M1 Story 2.4 — outline (FR49)
        #[serde(default = "default_outline_width")]
        pub outline_width: f32,
        #[serde(default = "default_outline_color")]
        pub outline_color: [f32; 4],
    }
    ```
    **Why `#[serde(default = "fn")]` per field, not `#[serde(default)]` on the struct:** the struct-level `#[serde(default)]` requires the entire struct to have a `Default` impl AND falls back the *whole struct* on missing fields, which masks accidental schema drift. Per-field `#[serde(default = "fn")]` only fills missing fields, so a typo'd `outline_widht` in RON still surfaces as a deserialization error. [Source: serde docs `field-attrs` — `default = "..."`]
  - [ ] Define the per-field default fns at module scope:
    ```rust
    fn default_outline_width() -> f32 { 3.0 }
    fn default_outline_color() -> [f32; 4] { [0.05, 0.05, 0.05, 1.0] }
    ```
    **Initial values rationale:**
    - `outline_width: 3.0` — visually present but not heavy at the reference scene's camera distance (~6 units). Story 2.5 backend-parity gate captures screenshots at this default; 3.0 is the dev's seed value subject to taste-iteration via hot-reload.
    - `outline_color: [0.05, 0.05, 0.05, 1.0]` — near-black against the default Bevy clear color. Vector-aesthetic idiom (`linework on background`). Hot-reload lets Till test alternatives (white-on-dark, color-coded by `SemanticAccent`, etc.) without recompile.
  - [ ] Update `Default for TuningConfig` to mirror the new defaults:
    ```rust
    impl Default for TuningConfig {
        fn default() -> Self {
            Self {
                toon_steps: 4,
                toon_rim_power: 2.0,
                toon_rim_intensity: 0.3,
                outline_width: default_outline_width(),
                outline_color: default_outline_color(),
            }
        }
    }
    ```
    **Discipline:** the `Default` impl AND the `default_outline_*` fns AND `tuning.ron` initial values MUST stay in sync. The unit test `tuning_config_default_matches_ron_initial_values` (extended in Task 4) is the architectural enforcement.
  - [ ] **Why `[f32; 4]` not `LinearRgba`:** Bevy's color types do not have `Deserialize` impls without the `bevy/serialize` feature (not enabled in our `Cargo.toml:8`). Adding the feature to pull in one tiny derive is overkill. `[f32; 4]` deserializes from RON as a flat array (e.g. `[0.05, 0.05, 0.05, 1.0]`), and is converted to `Color::srgba(..)` at apply-time (Task 5). Same idiom as the toon shader's `tint` field — the WGSL/Bevy boundary takes flat numeric primitives and the conversion lives in the Rust glue code.
  - [ ] **Why sRGBA, not linear:** Till is hand-tuning colors via the RON file. sRGBA is what color pickers and design intuition speak; linear RGB requires gamma math the dev should not have to perform mentally. The conversion to linear happens automatically when Bevy renders the outline (it consumes `Color`, which carries colorspace tags).

- [x] **Task 2: Update `assets/config/tuning.ron` with outline values** (AC: #2)
  - [ ] Edit `assets/config/tuning.ron` (currently 5 lines) to:
    ```ron
    TuningConfig(
        toon_steps: 4,
        toon_rim_power: 2.0,
        toon_rim_intensity: 0.3,
        outline_width: 3.0,
        outline_color: [0.05, 0.05, 0.05, 1.0],
    )
    ```
  - [ ] No comments inside the RON (consistent with Story 2.3 convention — runtime data file kept minimal; rationale lives in Rust doc-comments).
  - [ ] **Verify after edit:** run `cargo run` and confirm the dev log shows `TuningReloaded: toon_steps=4 ...` (the existing log line in `src/tuning/mod.rs:52-55` should fire on cold-start once the RON is parsed). If the new fields cause a parse error, the log will show an `error!` from Bevy's asset loader before `TuningReloaded` ever fires.

- [x] **Task 3: Author `src/visual/outline.rs`** (AC: #1, #4)
  - [ ] Create new file `src/visual/outline.rs`. Architecture mandates this exact path per `architecture.md:606`.
  - [ ] File contents:
    ```rust
    //! bevy_mod_outline integration — silhouette outlines for FR49.
    //! Story 2.4 wires OutlinePlugin and the TuningConfig→OutlineVolume hot-reload propagator.
    //! The fallback switch (Story 2.7, conditional on Story 2.6's go/fallback decision) is NOT
    //! pre-scaffolded here — YAGNI per architecture.md:887.

    use bevy::prelude::*;
    use bevy_mod_outline::OutlineVolume;

    use crate::tuning::TuningReloaded;

    /// Listens for TuningReloaded messages and propagates outline_width / outline_color into
    /// every entity that carries an OutlineVolume. Subscribes via the existing TuningSystems::Reload
    /// SystemSet so future tuning-driven systems can chain on it via .after(TuningSystems::Reload).
    pub(super) fn apply_tuning_to_outlines(
        mut events: MessageReader<TuningReloaded>,
        mut outlines: Query<&mut OutlineVolume>,
    ) {
        for event in events.read() {
            let [r, g, b, a] = event.0.outline_color;
            let new_color = Color::srgba(r, g, b, a);
            let new_width = event.0.outline_width;
            for mut volume in &mut outlines {
                volume.width = new_width;
                volume.colour = new_color;
            }
        }
    }
    ```
  - [ ] **Why a Query, not iterating Assets**: `OutlineVolume` is a Component (per `bevy_mod_outline::OutlineVolume` derive at `src/lib.rs:214`), not an Asset. The toon-material reload system iterates `Assets<ToonMaterial>` because materials are assets; outlines are per-entity components, so a `Query<&mut OutlineVolume>` is the right primitive.
  - [ ] **Why `pub(super)`**: the system is consumed by `VisualPlugin` (sibling module) but not exposed outside `crate::visual`. Stays consistent with Story 2.3's `apply_tuning_to_toon_materials` visibility (currently a private fn in `mod.rs`).
  - [ ] **British spelling note:** the `OutlineVolume` field is `colour: Color` (not `color`). Don't fight the upstream API; just remember the spelling. Common typo. [Source: `bevy_mod_outline-0.12.0/src/lib.rs:223`]
  - [ ] **No** `OutlineMode` set explicitly — the default `ExtrudeFlat` (defined in `bevy_mod_outline/src/lib.rs:266`) is correct for our purposes. Don't add `OutlineMode::FloodFlat` (jump-flood mode) — it's experimental per `lib.rs:30`, requires the `flood` feature, and adds compile/render cost we don't need at M1.
  - [ ] **No** `OutlineStencil` explicitly attached — `OutlinePlugin` registers `OutlineStencil` as a required component for `OutlineVolume` (see `lib.rs:462-464`), and the inherited default is `OutlineStencilEnabled::IfVolume` (per `lib.rs:159-162`). This matches the 0.11 changelog entry "Changed default stencil enable to be conditional upon volume enable."

- [x] **Task 4: Wire `OutlinePlugin` and the propagator into `VisualPlugin`** (AC: #1, #4)
  - [ ] Edit `src/visual/mod.rs`:
    - Add `pub mod outline;` after `pub mod toon_material;` (line 9).
    - Update top-of-file `//!` doc-comment: append a fifth line: "Story 2.4 adds `bevy_mod_outline::OutlinePlugin` wiring + outline hot-reload propagation (FR49)."
    - In `impl Plugin for VisualPlugin::build`:
      - Immediately after the `MaterialPlugin::<ToonMaterial>::default()` line (currently `mod.rs:20`), add:
        ```rust
        app.add_plugins(bevy_mod_outline::OutlinePlugin);
        ```
        **Order matters:** registering before `apply_tuning_to_toon_materials` keeps the render-graph nodes added in a deterministic order. `OutlinePlugin` queues many sub-app-render systems; placing it next to `MaterialPlugin` documents the "render pipeline registrations live at the top of `build`" convention.
      - Locate the existing `app.add_systems(Update, apply_tuning_to_toon_materials.in_set(crate::tuning::TuningSystems::Reload))` line (currently `mod.rs:27-30`). Convert it to a tuple of two systems:
        ```rust
        app.add_systems(
            Update,
            (
                apply_tuning_to_toon_materials,
                outline::apply_tuning_to_outlines,
            )
                .in_set(crate::tuning::TuningSystems::Reload),
        );
        ```
        **Why both systems in `TuningSystems::Reload`**: AC #4 hot-reload contract spans BOTH toon material uniforms AND outline volume components. The set membership documents the "all consumers of TuningReloaded run here" intent; future tuning-driven systems (Story 4.x: enemy_hp, shot_cost) join the same set.
        **No `.chain()` needed:** the two systems read disjoint state (one mutates `Assets<ToonMaterial>`, one mutates `Query<&mut OutlineVolume>`) — Bevy can run them in parallel. Adding `.chain()` would force serialization for no benefit.
  - [ ] **Cargo.toml**: NO edit needed. `bevy_mod_outline = "0.12"` is already pinned at `Cargo.toml:10` (verified via `cargo tree --depth 1` → `bevy_mod_outline v0.12.0`). The default features (`flood`, `interpolation`, `reflect`, `scene` per `bevy_mod_outline-0.12.0/Cargo.toml:36-41`) are fine for our usage; we don't need any of them at the surface but they don't hurt compile time meaningfully and disabling default features would risk breaking the plugin's internal assumptions.

- [x] **Task 5: Update `src/visual/reference_scene.rs` to spawn `OutlineVolume` on each placeholder** (AC: #2, #3)
  - [ ] Add imports at the top of `reference_scene.rs`:
    ```rust
    use bevy_mod_outline::{
        GenerateOutlineNormalsSettings, OutlineMeshExt, OutlineVolume,
    };
    use crate::tuning::TuningConfig;
    ```
  - [ ] Update the `spawn_reference_scene` system signature to take `Res<Assets<TuningConfig>>` and `Res<crate::tuning::TuningHandle>` so spawn-time outlines pick up the loaded RON values when available:
    ```rust
    fn spawn_reference_scene(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ToonMaterial>>,
        tuning_assets: Res<Assets<TuningConfig>>,
        tuning_handle: Res<crate::tuning::TuningHandle>,
    ) {
    ```
  - [ ] Compute the spawn-time outline values from `TuningConfig` with a fallback to `Default`:
    ```rust
    let tuning = tuning_assets.get(tuning_handle.0.id())
        .cloned()
        .unwrap_or_default();
    let outline_volume = || OutlineVolume {
        visible: true,
        width: tuning.outline_width,
        colour: {
            let [r, g, b, a] = tuning.outline_color;
            Color::srgba(r, g, b, a)
        },
    };
    ```
    **Cold-start race acknowledgment:** `OnEnter(GameState::Loading)` fires before the asset loader finishes parsing `tuning.ron` (per Story 2.3 Dev Notes "Hot-reload mechanics — the cold-start gotcha", `2-3-wgsl-toon-material-implementation.md:649-664`). The `unwrap_or_default()` fallback uses the same defaults as `default_outline_*` from Task 1. Once the asset finishes loading, `apply_tuning_to_outlines` (Task 3) overwrites the spawn-time values with the RON-supplied values. Worst case: 1-2 frames of "default" outline thickness/color before RON kicks in. Acceptable — no AC mandates frame-1 correctness, and the defaults are deliberately set to match the RON.
  - [ ] **Asteroid (icosphere) — append `OutlineVolume`:**
    Currently spawns `(Mesh3d, MeshMaterial3d, Transform, SemanticAccent::Hazard, ReferenceSceneEntity)` at `reference_scene.rs:50-56`. Add `outline_volume()` to the tuple:
    ```rust
    commands.spawn((
        Mesh3d(asteroid_mesh),
        MeshMaterial3d(asteroid_mat),
        Transform::from_xyz(-2.0, 0.0, 0.0),
        SemanticAccent::Hazard,
        outline_volume(),
        ReferenceSceneEntity,
    ));
    ```
    **No `generate_outline_normals` for the icosphere:** ico-spheres have smooth interpolated normals; vertex extrusion produces a clean silhouette with no hard-edge artefacts. (Per `bevy_mod_outline/src/generate.rs:88-94` doc-comment "Vertex extrusion only works for meshes with smooth surface normals.")
  - [ ] **Ship-cockpit (cuboid) — generate outline normals BEFORE inserting into `Assets<Mesh>`:**
    Currently: `let ship_mesh = meshes.add(Cuboid::new(1.0, 0.5, 1.5));` at `reference_scene.rs:59`. Replace with:
    ```rust
    let ship_mesh = {
        let mut mesh = Cuboid::new(1.0, 0.5, 1.5).mesh().build();
        mesh.generate_outline_normals(&GenerateOutlineNormalsSettings::default())
            .expect("cuboid has TriangleList topology and Float32x3 positions");
        meshes.add(mesh)
    };
    ```
    Then append `outline_volume()` to its spawn tuple:
    ```rust
    commands.spawn((
        Mesh3d(ship_mesh),
        MeshMaterial3d(ship_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
        SemanticAccent::PlayerOwned,
        outline_volume(),
        ReferenceSceneEntity,
    ));
    ```
    **Why `.expect(...)`:** `generate_outline_normals` returns `Result<(), GenerateOutlineNormalsError>` with three failure modes (`UnsupportedPrimitiveTopology`, `MissingVertexAttribute`, `InvalidVertexAttributeFormat` per `bevy_mod_outline/src/generate.rs:73-82`). Bevy's `Cuboid::mesh().build()` always produces TriangleList topology with `Float32x3` positions (see Bevy 0.18 `bevy::math::primitives::Cuboid` mesh builder); none of the failure modes can fire here. `.expect("...")` documents the invariant per architecture.md:367 ("internal invariants: `expect("message explaining why this cannot fail")`").
    **Why this matters visually:** without `generate_outline_normals`, the cuboid's hard edges (each face has its own normal) cause vertex extrusion to spike outward at corners — visible as cross-shaped artefacts at the cube's corners. With the function applied, vertices at shared positions get angle-weighted averaged normals, producing a smooth silhouette. (Algorithm at `bevy_mod_outline/src/generate.rs:130-176`.)
  - [ ] **Projectile (UV-sphere) — append `OutlineVolume`:**
    Currently spawns `(Mesh3d, MeshMaterial3d, Transform, SemanticAccent::Salvage, ReferenceSceneEntity)` at `reference_scene.rs:78-84`. Add `outline_volume()` to the tuple. NO `generate_outline_normals` (UV-sphere has smooth normals).
  - [ ] **Lights and camera unchanged.** The 3 PointLights and the Camera3d (order: -1) remain untouched.
  - [ ] **Swatch UI camera + node tree (`spawn_palette_swatches`) unchanged.** UI elements don't get outlines (UI rendering uses Camera2d at order: 1, which is outside the Core3d render graph that `OutlinePlugin` extends). No risk of accidentally outlining the swatch panels.

- [x] **Task 6: Update unit tests for the new `TuningConfig` schema** (AC: #5)
  - [ ] In `src/tuning/config.rs`, update `tuning_config_default_matches_ron_initial_values`:
    ```rust
    #[test]
    fn tuning_config_default_matches_ron_initial_values() {
        let cfg = TuningConfig::default();
        assert_eq!(cfg.toon_steps, 4);
        assert_eq!(cfg.toon_rim_power, 2.0);
        assert_eq!(cfg.toon_rim_intensity, 0.3);
        assert_eq!(cfg.outline_width, 3.0);
        assert_eq!(cfg.outline_color, [0.05, 0.05, 0.05, 1.0]);
    }
    ```
  - [ ] Update `tuning_config_deserializes_from_ron_bytes` to include the new fields:
    ```rust
    #[test]
    fn tuning_config_deserializes_from_ron_bytes() {
        let bytes = b"TuningConfig(toon_steps: 5, toon_rim_power: 1.5, toon_rim_intensity: 0.4, outline_width: 5.0, outline_color: [1.0, 0.0, 0.0, 1.0])";
        let cfg: TuningConfig = ron::de::from_bytes(bytes).unwrap();
        assert_eq!(cfg.toon_steps, 5);
        assert_eq!(cfg.toon_rim_power, 1.5);
        assert_eq!(cfg.toon_rim_intensity, 0.4);
        assert_eq!(cfg.outline_width, 5.0);
        assert_eq!(cfg.outline_color, [1.0, 0.0, 0.0, 1.0]);
    }
    ```
  - [ ] **Add a NEW test** verifying `#[serde(default)]` forward-compat (AC #5 third clause):
    ```rust
    #[test]
    fn tuning_config_legacy_2_3_schema_uses_defaults_for_outline_fields() {
        // Story 2.3 schema lacked outline fields; #[serde(default = "...")] fallback must fill them.
        let bytes = b"TuningConfig(toon_steps: 4, toon_rim_power: 2.0, toon_rim_intensity: 0.3)";
        let cfg: TuningConfig = ron::de::from_bytes(bytes).unwrap();
        assert_eq!(cfg.outline_width, 3.0);
        assert_eq!(cfg.outline_color, [0.05, 0.05, 0.05, 1.0]);
    }
    ```
    **Why this test exists:** if a future Story 4.x dev removes `#[serde(default = "...")]` thinking it's redundant ("the RON always has all fields"), this test fails immediately with a clear "MissingField outline_width" error. The test enforces the forward-compat contract documented in Story 2.3 Dev Notes "tuning.ron extensibility" (`2-3-wgsl-toon-material-implementation.md:666-690`).
  - [ ] **NO** test for `apply_tuning_to_outlines` runtime behavior — it requires `App::new()` with the render plugins, which crosses into integration-test territory. Architecture.md:354 defers integration tests post-M3. The `#[cfg(test)] mod tests` in `src/visual/outline.rs` is **not added** for this story.
  - [ ] **Test count post-Task-6:** 12 (post-2.3) + 1 new (`tuning_config_legacy_2_3_schema_uses_defaults_for_outline_fields`) = **13**. The two existing tuning tests are updated in place, not added. Capture this count for the verification sweep.

- [x] **Task 7: Local verification sweep — code paths** (AC: #1, #2, #3, #4, #5)
  - [ ] `cargo check 2>&1 | tee /tmp/story-2-4-check.log` → `grep -cE 'warning:|error:' /tmp/story-2-4-check.log` must equal **0**.
  - [ ] `cargo build 2>&1 | tee /tmp/story-2-4-build.log` → same grep equals **0**.
  - [ ] `cargo test 2>&1 | tee /tmp/story-2-4-test.log` → grep `'warning:|error:|FAILED'` equals **0**; test count must read **13 passed, 0 failed** (12 from 2.3 + 1 new). If the count is 12, Task 6's new test was forgotten; if >13, an unintended test was added.
  - [ ] `cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-2-4-clippy.log` → grep equals **0**. Watch for:
    - `clippy::needless_pass_by_value` on the `Res<Assets<TuningConfig>>` parameter — false positive if it fires; suppress with `#[expect(clippy::needless_pass_by_value, reason = "Bevy ECS system param convention")]` and document.
    - `clippy::needless_pass_by_ref_mut` on the system parameters — same situation.
    - `clippy::missing_const_for_fn` on `default_outline_width` / `default_outline_color` — if it fires, leave them non-const (serde requires fn pointers, not `const fn` necessarily, but `const fn` works too).
  - [ ] `cargo fmt --all -- --check` → exit 0.
  - [ ] `cargo build --release 2>&1 | tee /tmp/story-2-4-release.log` → grep equals **0**. (Story 2.5 will run `cargo run --release` on three backends; release-build cleanliness here de-risks 2.5.)
  - [ ] **Debug-build runtime verification (AC #1, #3):**
    - `cargo run &> /tmp/story-2-4-run.log &` → wait ≥ 5 seconds for splash → MainMenu transition + first `tuning.ron` parse.
    - During MainMenu state: visually verify all THREE placeholders show **continuous silhouette outlines** (no breaks at corners on the cuboid ship — that's the `generate_outline_normals` payoff).
    - Verify outlines do NOT z-fight (no shimmering at mesh-outline boundary). If z-fighting appears, root-cause likely camera distance vs `outline_width: 3.0` — bumping `outline_width` to a smaller value usually resolves it; alternatively, attach `OutlineStencil { offset: 1.0, ..default() }` per `bevy_mod_outline/src/lib.rs:151-156` to push the stencil out from the mesh surface. Defer to a follow-up task only if visible at default 3.0; expected: no z-fight at this distance.
    - `grep -c 'TuningReloaded' /tmp/story-2-4-run.log` ≥ 1 (cold-start RON load fires the reload event once the asset finishes parsing).
    - `grep -cE 'warning:|error:|ERROR ' /tmp/story-2-4-run.log` should be 0 for app-emitted lines (the deferred-work splash-cleanup-iteration race WARN remains known and unrelated).
  - [ ] **Hot-reload runtime verification (AC #4):**
    - With `cargo run` still running, edit `assets/config/tuning.ron` and change `outline_width: 3.0` → `outline_width: 8.0`. Save.
    - Within ~1 second all three placeholders should show **noticeably thicker outlines** (8 logical pixels of extrusion vs 3).
    - Edit `outline_color: [0.05, 0.05, 0.05, 1.0]` → `outline_color: [1.0, 0.0, 0.0, 1.0]` (red). Save. All outlines turn **red** within ~1 second.
    - Edit back to defaults (`3.0` and `[0.05, 0.05, 0.05, 1.0]`) before closing.
    - `grep -c 'TuningReloaded' /tmp/story-2-4-run.log` should now match 1 (cold-start) + 3 (edits) = **4**.

- [x] **Task 8: Visual capture (`docs/tech-spike/m1-outline/`)** (AC: #3, #4 evidence)
  - [ ] Create directory `docs/tech-spike/m1-outline/`.
  - [ ] **Three screenshots** capturing Story 2.4's outline spike:
    - `docs/tech-spike/m1-outline/outline-baseline.png` — `outline_width: 3.0`, `outline_color: [0.05, 0.05, 0.05, 1.0]` (initial RON values). Shows all three placeholders with the default near-black outline.
    - `docs/tech-spike/m1-outline/outline-thick.png` — `outline_width: 8.0`. Shows visibly thicker outlines (AC #4 width hot-reload evidence).
    - `docs/tech-spike/m1-outline/outline-red.png` — `outline_color: [1.0, 0.0, 0.0, 1.0]`. Shows red outlines (AC #4 color hot-reload evidence).
  - [ ] Capture: `Cmd-Shift-4` → window-frame select → save to `docs/tech-spike/m1-outline/<name>.png`. Resolution = native window resolution; PNG, lossless. Same idiom as Story 2.3's `docs/tech-spike/m1-toon/` captures.
  - [ ] **`docs/tech-spike/m1-outline/notes.md`** — short markdown capturing:
    - Whether all three placeholders show continuous silhouettes (AC #3).
    - Whether the cuboid ship's corners are smooth (AC #3 hard-edge artefact check — `generate_outline_normals` payoff).
    - Whether z-fighting was observed at the default camera distance (yes/no; AC #3).
    - Whether hot-reload latency is acceptable (`< 1s` from save to render — yes/no; AC #4).
    - Any backend-specific caveats observed during dev (Story 2.5 handles cross-backend parity formally; this is just the dev's quick "did it work" log).

- [x] **Task 9: Scope guardrails — verify nothing else drifted** (AC: all)
  - [ ] `git status --short`: expected file set:
    - `src/visual/outline.rs` (??) — new
    - `src/visual/mod.rs` (M) — `pub mod outline;` + doc-comment + `OutlinePlugin` registration + outline propagator added to existing `Update` system tuple
    - `src/visual/reference_scene.rs` (M) — imports + spawn-tuple updates + cuboid `generate_outline_normals` + tuning param plumbing
    - `src/tuning/config.rs` (M) — two new fields + per-field `#[serde(default)]` + `default_outline_*` fns + Default impl extension + 1 new test + 2 updated tests
    - `assets/config/tuning.ron` (M) — two new lines (`outline_width`, `outline_color`)
    - `Cargo.toml` (??) — **untouched** (`bevy_mod_outline = "0.12"` already pinned at `Cargo.toml:10`)
    - `Cargo.lock` (??) — **untouched** (no new transitive deps; `bevy_mod_outline` was already resolved by 1.2's plugin compatibility gate)
    - `docs/tech-spike/m1-outline/{outline-baseline,outline-thick,outline-red}.png` + `notes.md` — new (Task 8)
    - Bookkeeping: this story file (??) + `sprint-status.yaml` (M) — flipped at Task 11.
  - [ ] `grep -nrE 'OutlineVolume|OutlinePlugin|outline_width|outline_color|outline\.rs|generate_outline_normals' src/ --include='*.rs'` → expected hits:
    - `src/visual/outline.rs`: own definition (3-5 hits).
    - `src/visual/mod.rs`: `pub mod outline;` + `OutlinePlugin` registration + system add (3 hits).
    - `src/visual/reference_scene.rs`: imports + 3 spawn sites + 1 generate_outline_normals call (5-7 hits).
    - `src/tuning/config.rs`: own field definitions + tests (4-8 hits).
    - **No** hits in `src/main.rs`, `src/state.rs`, `src/splash.rs`, `src/logging.rs`, `src/visual/palette.rs`, `src/visual/toon_material.rs`, `src/tuning/mod.rs`. (TuningPlugin internals don't need to know about outline fields — they just deserialize whatever's in the struct.)
  - [ ] `grep -nrE 'Arena|Caravan|PostRun|PhotoMode|Paused' src/ --include='*.rs' | grep -v 'state.rs'` → 0 hits (states still not live).
  - [ ] `grep -rn 'AssetServer::load\b' src/` → expected hits: **2** — same as post-2.3 (one in `src/tuning/mod.rs` for `tuning.ron`, zero elsewhere). Outline plugin loads its internal shaders via `load_internal_asset!` (per `bevy_mod_outline-0.12.0/src/lib.rs:434-446`), not `AssetServer::load`.
  - [ ] `cargo tree --depth 1 -p asteroids3D | grep -E 'bevy_mod_outline|bevy '` → expected: `├── bevy v0.18.1`, `├── bevy_mod_outline v0.12.0` (unchanged from post-1.2).
  - [ ] `Cargo.toml`, `Cargo.lock`, `.gitignore`, `.gitattributes`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `docs/plugin-compatibility.md`, `src/state.rs`, `src/splash.rs`, `src/logging.rs`, `src/main.rs`, `src/visual/palette.rs`, `src/visual/toon_material.rs`, `src/tuning/mod.rs`, `assets/shaders/toon.wgsl` — **all untouched**.

- [x] **Task 10: Commit + CI observation** (AC: all)
  - [ ] **Commit 1 (source + assets):** stage `src/visual/{mod,outline,reference_scene}.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`. **No** docs, **no** Cargo files (none changed).
    - HEREDOC commit message subject: `feat: bevy_mod_outline integration + outline hot-reload (Story 2.4)`. Single-line, under 80 chars. Match Till's commit-style precedent (`feat: WGSL ToonMaterial + TuningConfig hot-reload + SemanticAccent tinting (Story 2.3)` from `da0eb1f`).
    - Push to `origin/master`. Triggers full 4-job CI matrix (`.github/workflows/ci.yml` paths-ignore = `_bmad/**` + `_bmad-output/**` only; `src/`, `assets/` are NOT ignored). Cargo cache warm; Cargo.lock unchanged → fast.
    - `gh run list -L 1` → identify run ID. Wait for all 4 jobs (`build {ubuntu,macos,windows}-latest` + `msrv-check`) to complete.
    - `gh run view <ID> --log | grep -cE 'warning:|error:'` → 0.
    - All 4 jobs ✅; capture run ID + per-job durations.
  - [ ] **Commit 2 (docs):** stage `docs/tech-spike/m1-outline/{outline-baseline,outline-thick,outline-red}.png` + `notes.md`.
    - HEREDOC commit message subject: `docs: M1 outline-integration spike evidence (Story 2.4)`.
    - Push. Triggers CI (cached). Capture run ID.
    - **Push-fold optimization:** if Till opts to fold both commits into one push (precedent from Stories 2.2 + 2.3), only one CI run-ID is captured. Document the fold reasoning in Dev Agent Record like Story 2.3 did.

- [x] **Task 11: Ready-for-review handoff + bookkeeping commit**
  - [ ] Populate **Dev Agent Record**: Agent Model Used, Debug Log References (per-command grep counts + log paths + screenshot capture metadata + CI run IDs), Completion Notes (per-AC evidence + any deviations from spec — e.g. if z-fighting forced an `OutlineStencil` offset, capture the rationale; if a clippy false-positive needed `#[expect(...)]`, note the reason), File List (added / modified).
  - [ ] Set this story's `Status:` header → `review`.
  - [ ] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`: flip `2-4-bevy-mod-outline-integration-wiring: ready-for-dev → in-progress → review`; bump `last_updated`.
  - [ ] **Update `deferred-work.md` ONLY if new findings emerged.** Story 2.4 does NOT inherit any active deferred entries that resolve here:
    - The Story 2.2 cfg_attr removal stays re-deferred to **Story 4.5** (still cfg-gated reference scene; Story 2.4 doesn't change that).
    - The Story 2.3 `extensions: &["ron"]` collision concern stays open until **Story 4.7**.
    Both unchanged by this story.
  - [ ] Stage story file + `sprint-status.yaml` (and `deferred-work.md` only if edited), commit with `bmad: story 2.4 ready-for-dev → review (outline integration, CI <ID> green)`. `_bmad-output/**` paths-ignored → no CI cost.
  - [ ] Push.
  - [ ] Story awaits code review. **Multi-LLM adversarial review recommended** for this story given the third-party-plugin integration surface (render-graph nodes, sub-app systems, cross-feature hot-reload). Run `bmad-code-review` ideally with a different LLM than the implementer.

## Dev Notes

### Why this story exists

Story 2.4 closes the **second half of FR49** — silhouette outlines. The first half (the toon-shaded base color via custom WGSL Material) shipped in Story 2.3. Together, toon + outline define the asteroids3D vector aesthetic, and Story 2.5 will validate the combination across Metal/Vulkan/DX12 with committed parity screenshots. [Source: prd.md:569 (FR49); epics/epic-2-vector-aesthetic-tech-spike.md:95-119; architecture.md:222 ("`bevy_mod_outline` plugin for silhouette outlines (pinned, fork-ready, not a learning priority)")]

The architectural decision to use `bevy_mod_outline` (vs hand-authoring an outline shader) is deliberate per architecture.md:222: **outlines are a stable, well-trodden technique; the learning priority is the toon shader (Story 2.3), not the outline plumbing.** A maintained third-party plugin with fork-readiness as a contingency is the right tradeoff for M1's time budget. [Source: architecture.md:885-887; prd.md:401-403]

Three things ship together in this story because they're tightly coupled:
1. **Plugin wiring** (`bevy_mod_outline::OutlinePlugin` in `VisualPlugin::build`) — the render-graph extension that adds outline passes to `Core3d`.
2. **`OutlineVolume` attachment** (`reference_scene.rs` updates) — the per-entity contract that says "this mesh participates in outline rendering."
3. **Hot-reload propagation** (`outline_width` + `outline_color` in `TuningConfig`, plus `apply_tuning_to_outlines` in `outline.rs`) — extends the Story 2.3 hot-reload infrastructure to a second consumer, validating that the `TuningSystems::Reload` SystemSet pattern scales.

### Inherited context from Stories 2.1 + 2.2 + 2.3

| Fact | Value | Source |
|---|---|---|
| `src/visual/mod.rs` (post-2.3) | `pub mod palette`, `pub mod toon_material`, `VisualPlugin`, `VisualSystems::Setup`, MaterialPlugin<ToonMaterial> registered, `apply_tuning_to_toon_materials` system in `TuningSystems::Reload`, cfg-gated `mod reference_scene` | `src/visual/mod.rs` post-2.3 |
| `src/visual/reference_scene.rs` (post-2.3) | `ReferenceScenePlugin` (cfg-gated), `spawn_reference_scene` on `OnEnter(Loading)` (3 placeholders w/ `ToonMaterial` + `SemanticAccent` + 3 PointLights + Camera3d order:-1), `spawn_palette_swatches` on `OnEnter(MainMenu)` (5 swatches + Camera2d order:1), all entities tagged `ReferenceSceneEntity` | post-2.3 |
| `src/visual/toon_material.rs` (post-2.3) | `pub struct ToonMaterial` (Asset + AsBindGroup + TypePath + Clone + Debug) with `tint: LinearRgba`, `steps: u32`, `rim_power: f32`, `rim_intensity: f32`; `Default` + `Material` impls; 1 unit test | post-2.3 |
| `src/tuning/mod.rs` (post-2.3) | `TuningPlugin`, `TuningSystems::Reload` SystemSet, `TuningHandle(Handle<TuningConfig>)` Resource, `TuningReloaded(TuningConfig)` Message, `load_tuning` Startup system, `propagate_tuning_reload` Update system | post-2.3 |
| `src/tuning/config.rs` (post-2.3) | `pub struct TuningConfig` (Asset + TypePath + Clone + Debug + Deserialize) with `toon_steps`, `toon_rim_power`, `toon_rim_intensity`; `Default` impl; `TuningConfigLoader` (AssetLoader for `.ron`); 2 unit tests | post-2.3 |
| `assets/config/tuning.ron` (post-2.3) | `TuningConfig(toon_steps: 4, toon_rim_power: 2.0, toon_rim_intensity: 0.3)` | post-2.3 |
| Test count post-2.3 | **12 passing** | `_bmad-output/implementation-artifacts/2-3-wgsl-toon-material-implementation.md:117,146,219` |
| Bevy version | `0.18` (resolved `0.18.1`), features `["3d", "png", "bevy_ui", "default_font", "file_watcher"]` (+ x11/wayland on Linux) | `Cargo.toml:8,23-26` |
| `bevy_mod_outline` version | `0.12` (resolved `0.12.0`) — already pinned and validated by Story 1.2 | `Cargo.toml:10`, `docs/plugin-compatibility.md:28` |
| `serde = "1"` with `derive`, `ron = "0.8"`, `thiserror = "2"` | All present, no Cargo edit needed for 2.4 | `Cargo.toml:14,17,16` |
| Story 2.5 dependency | Will run `cargo run --release` on three backends, capture screenshots from a fixed `Transform`. **2.4 must work in release** for 2.5 to validate. Task 7's release-build verification de-risks 2.5. | `epics/epic-2-vector-aesthetic-tech-spike.md:121-147` |
| Story 2.6 dependency | Will read the visual results from 2.3+2.4 and decide GO toon vs FALLBACK. **2.4 must produce outlines visible enough to evaluate** — if outlines aren't visible at default `width: 3.0`, 2.6's evaluation is artificially biased toward fallback. Task 8's screenshot capture is the evidence 2.6 reviews. | `epics/epic-2-vector-aesthetic-tech-spike.md:149-170` |

### `bevy_mod_outline` 0.12 — what to know

[Source: `~/.cargo/registry/src/index.crates.io-*/bevy_mod_outline-0.12.0/src/lib.rs`, `examples/shapes.rs`, `examples/pieces.rs`, `CHANGELOG.md`]

**Public API surface used by Story 2.4 (only the parts we touch):**

- `OutlinePlugin` (struct, plugin) — registers all outline render-graph nodes, render systems, and required-component relationships. Single registration via `app.add_plugins(bevy_mod_outline::OutlinePlugin)`.
- `OutlineVolume` (Component) — `{ visible: bool, width: f32, colour: Color }`. Width is in logical pixels. Colour uses British spelling. This is the primary component you spawn on each mesh entity to get an outline.
- `OutlineMeshExt::generate_outline_normals(&mut Mesh, &GenerateOutlineNormalsSettings) -> Result<(), GenerateOutlineNormalsError>` — extension trait method on `Mesh`. Generates faux-smooth outline normals for hard-edge meshes (cuboids, dodecahedrons, anything with discontinuous normals at edges). Mutates the mesh in place by inserting an extra vertex attribute (`ATTRIBUTE_OUTLINE_NORMAL`).
- `GenerateOutlineNormalsSettings` (struct) — `Default::default()` is correct for our usage. Builder methods: `.with_ignore_vertex_normals(bool)` (force face normals over vertex normals, rarely needed) and `.with_stretch_edges(bool)` (extra outward angling for non-manifold meshes — definitely not needed).

**Public API we deliberately do NOT use (and why):**

- `AutoGenerateOutlineNormalsPlugin` — runs `generate_outline_normals` on EVERY mesh asset, including ones that don't need it (icosphere, UV-sphere). Wasteful, and makes the mesh modification non-deterministic from the call-site reader's perspective. We invoke `generate_outline_normals` per-mesh, only on the cuboid that needs it.
- `InheritOutline` (Component) — for parent-child hierarchies where a child should adopt the parent's outline config. We have no such hierarchy in 2.4; all three placeholders are sibling roots.
- `OutlineMode` enum — defaults to `ExtrudeFlat`. We don't override. `FloodFlat` (jump-flood) is experimental per `lib.rs:30` and requires the `flood` feature; not worth the risk at M1.
- `OutlineStencil` (Component) — auto-attached as a required component for `OutlineVolume` (`lib.rs:462-464`) with default `OutlineStencilEnabled::IfVolume`. We let the plugin's defaults handle this.
- `OutlinePlaneDepth` (Component) — for advanced depth sorting of flat outlines across overlapping objects. Our 3 placeholders don't overlap from the camera's perspective; not needed.
- `OutlineRenderLayers` (Component) — for restricting outlines to specific render layers. Default render layers cover our toon-shaded entities.
- `OutlineWarmUp` — pipeline pre-warming to avoid first-frame flicker on animated outlines. We don't animate outline properties (the hot-reload propagator updates discrete values, not continuous animations); not needed.
- `OutlineAlphaMask` — texture-driven outline masking. Not needed for our untextured placeholders.

**Render-graph integration (informational — we don't touch this directly):**

`OutlinePlugin::build` adds three nodes to `Core3d` (`lib.rs:526-545`):
1. `NodeOutline::MsaaExtraWritebackPass` (after `Tonemapping`)
2. `NodeOutline::OutlinePass`
3. `NodeOutline::EndOutlinePasses` (before `Fxaa` / `Smaa`)

This means outlines render **after tone-mapping but before AA**, so AA smooths the outline edges along with the base mesh. Right behavior — matches the upstream plugin's documented "after the main 3D pass" placement. [Source: `lib.rs:5-7`]

**0.12.0 breaking changes from 0.11 (informational):**

[Source: `bevy_mod_outline-0.12.0/CHANGELOG.md`]
- "Removed deprecated bundles" — the `OutlineBundle` struct that older 0.10/0.11 examples used is GONE in 0.12. Don't write `OutlineBundle { ... }` anywhere; spawn the components directly (`OutlineVolume`, optionally `OutlineStencil`/`OutlineMode`/etc).
- "Updated Bevy dependency to 0.18" — matches our project's Bevy 0.18 pin. No compat issue.
- The Story 2.4 epic spec at `epics/epic-2-vector-aesthetic-tech-spike.md:99-119` mentions "OutlineBundle" — this language predates the 0.12 plugin update; we're spawning components directly per the current API. Same outcome (mesh gets an outline), different syntax.

### Architecture compliance — naming, module layout, plugin pattern

**Plugin / SystemSet naming (architecture.md:326-328):** ✓
- `VisualPlugin` already exists; extended (not replaced) by Story 2.4.
- No new plugin or SystemSet introduced — Story 2.4 reuses `TuningSystems::Reload`.

**Module layout (architecture.md:603-607):** ✓
- `src/visual/outline.rs` matches architecture line 606 exactly.
- `pub mod outline;` exposes the module via the qualified path `crate::visual::outline::apply_tuning_to_outlines`. **No** `pub use outline::*;` re-export (consistent with Story 2.3's `pub mod toon_material` pattern).

**Inter-system communication (architecture.md:243):** ✓
- `apply_tuning_to_outlines` reads `MessageReader<TuningReloaded>` (the Bevy-0.18-renamed `EventReader`) — no direct cross-plugin state mutation.
- `OutlineVolume` is owned by `VisualPlugin` (it's a `bevy_mod_outline` component, but the entity carrying it is spawned by `VisualPlugin`'s reference scene). TuningPlugin only writes the `TuningReloaded` message; it doesn't reach into `Query<&mut OutlineVolume>`.

**Component naming (architecture.md:322):** ✓
- `OutlineVolume` is the plugin's component, named per its convention. We don't introduce new components in this story.

**Plugin boundary table (architecture.md:654, 656):** ✓
- `VisualPlugin` boundary: owns `ToonMaterial` asset registration, palette, **outline plugin wiring** (now real), hot-reload propagators for both materials and outlines.
- `TuningPlugin` boundary: owns `TuningConfig` resource and hot-reload watcher. Emits `TuningReloaded` event. No new responsibility added by 2.4 — the schema growth is internal to `TuningConfig`, transparent to `TuningPlugin`.

**Anti-pattern check (architecture.md:458-468):** ✓
- ❌ God-struct: `TuningConfig` grows by 2 fields (3→5), still small and single-responsibility per field. ✓
- ❌ Direct cross-plugin state mutation: `apply_tuning_to_outlines` is a VisualPlugin-internal system reading TuningPlugin's message — not direct mutation. ✓
- ❌ Magic numbers: `outline_width: 3.0` and `outline_color: [0.05, 0.05, 0.05, 1.0]` are dev-tweakable defaults; their rationale is documented in Task 1 inline. ✓
- ❌ `unwrap()` / `expect()`: `generate_outline_normals` returns `Result`; we use `.expect("cuboid has TriangleList topology and Float32x3 positions")` per architecture.md:367 internal-invariant convention. ✓
- ❌ Scattered `AssetServer::load`: still ONE call (in `load_tuning` Startup system). The outline plugin loads its shaders via `load_internal_asset!` macro (compile-time-baked into the binary), not `AssetServer::load`. ✓
- ❌ `.after(specific_function)` ordering: not used. SystemSet via `TuningSystems::Reload`. ✓

### LLM dev agent guardrails — most-likely-to-go-wrong patterns

These are the failure modes that are most likely to bite if the dev moves fast:

1. **Writing `OutlineBundle { ... }` from a stale tutorial.** `bevy_mod_outline 0.12.0` REMOVED deprecated bundles (CHANGELOG entry). Always spawn `OutlineVolume` as a standalone component in the `commands.spawn((...))` tuple, alongside `Mesh3d`/`MeshMaterial3d`/etc. The plugin auto-attaches required companions (`ComputedOutline`, `OutlineStencil`) via `register_required_components` — you don't need to spawn those.

2. **Forgetting `generate_outline_normals` on the cuboid.** Hard-edged meshes produce visible spike artefacts at vertex extrusion time. The icosphere and UV-sphere have smooth interpolated normals → no call needed. The cuboid (Bevy's `Cuboid::mesh()`) has per-face normals → MUST call `mesh.generate_outline_normals(&GenerateOutlineNormalsSettings::default())` before `meshes.add(mesh)`. Not calling it produces visually broken outlines that may be wrongly attributed to a Bevy bug or a backend-specific issue.

3. **`.color` instead of `.colour`.** The field is British-spelled (`OutlineVolume.colour`). Rust's compiler will produce a hard error if you write `.color`, but the typo is easy to make by reflex. [Source: `bevy_mod_outline-0.12.0/src/lib.rs:223`]

4. **Bevy 0.18 idiom drift: `EventReader<TuningReloaded>` vs `MessageReader<TuningReloaded>`.** Bevy 0.18 renamed the cross-system message primitive: `Event` → `Message`, `EventReader` → `MessageReader`, `EventWriter` → `MessageWriter`, `add_event` → `add_message`. Story 2.3's actually-shipped code uses the `Message` family (see `src/tuning/mod.rs:20,28,43,46`). DO NOT write `EventReader<TuningReloaded>` in `outline.rs` — it won't compile cleanly with the rest of the project. [Source: `src/tuning/mod.rs:20-46` post-2.3]

5. **Spawning OutlineVolume with `visible: false` from default.** `OutlineVolume::default()` produces `visible: false, width: 0.0, colour: <default>` — a no-op outline. The spawn-time `outline_volume()` closure in Task 5 sets `visible: true` explicitly. Forgetting `visible: true` produces "outlines silently don't render and the dev wonders why" — the most common bevy_mod_outline footgun.

6. **`#[serde(default)]` on the struct vs per-field.** Per-field `#[serde(default = "fn")]` ONLY fills missing fields with the named default fn. Struct-level `#[serde(default)]` falls back the WHOLE struct on parse errors, which masks accidental schema corruption. Use per-field. [Source: serde docs `field-attrs#default--`]

7. **Forgetting to update `Default for TuningConfig` after extending the struct.** Adding `outline_width: f32` to the struct without updating `Default` produces a compile error — `Default` derives won't generate for non-Default fields. The tests `tuning_config_default_matches_ron_initial_values` enforces drift detection at test-time, but the compiler catches the obvious case at build-time.

8. **Hot-reload race: editing `tuning.ron` while the file watcher hasn't seen the file's first AddedEvent.** If the dev edits the RON before the cold-start `AssetEvent::Added` fires (within the first ~200ms), the edit triggers `Modified` BEFORE `Added`, and the `propagate_tuning_reload` system at `src/tuning/mod.rs:42-62` matches BOTH `Added` and `Modified` so this is handled correctly. Not a bug; just noting why the match arm is `Added | Modified`.

9. **Z-fighting at large outline widths or close camera distances.** Symptom: shimmering pixels at the outline-mesh boundary. Cause: depth-buffer precision. Mitigation: smaller `outline_width`, OR attach `OutlineStencil { offset: <small_positive>, ..default() }` to push the stencil out from the mesh. Default stencil offset is `0.0`. We don't pre-set this; if z-fighting appears at default `width: 3.0`, document in Task 8 notes and add a Dev Agent Record entry — DO NOT silently change defaults.

10. **Adding a clippy `#[allow(...)]` to suppress a warning instead of root-causing.** Per architecture.md (CI policy) clippy is `-D warnings`. If clippy fires, prefer `#[expect(name, reason = "...")]` (the explicit form Bevy itself uses) and document the reason in Dev Agent Record. NEVER blanket-allow at module or crate level.

11. **Mutating `Cargo.toml` to add `bevy_mod_outline` features.** Default features (`flood`, `interpolation`, `reflect`, `scene`) are already enabled by Cargo's `bevy_mod_outline = "0.12"` line. Story 2.4 doesn't need any of them at the surface, but disabling default features risks breaking the plugin's internal assumptions (e.g. `flood` is needed by `OutlineMode::FloodFlat` even though we don't use it; `reflect` is needed by Bevy's reflection-driven debug tooling). Leave defaults on.

12. **AC #5 third clause skipped** ("legacy 2.3 schema deserializes via serde defaults"). The forward-compat test in Task 6 is the architectural enforcement of `#[serde(default = "...")]`. If the dev removes that test, future Story 4.x dev has no signal that removing `#[serde(default)]` breaks backward compat. Easy to forget because it's not on the critical path of "outlines work."

13. **Leaving runtime hot-reload latency unmeasured in the Dev Notes.** AC #4 says "within ~1 second." Bevy's file watcher polling interval is ~250 ms by default; the asset reload + system propagation adds another frame. Total observable latency is typically ~300-400 ms. If the dev observes >1 s, that's a signal that something's wrong (file watcher not running, asset loader stuck, system not registered). Capture the qualitative latency in `docs/tech-spike/m1-outline/notes.md`.

### Camera / lighting strategy — unchanged from 2.3

Story 2.3 set up `Camera3d at order: -1` with 3 PointLights and `Camera2d at order: 1` for the swatch UI. Story 2.4 changes nothing about cameras or lights — outlines render through the existing Camera3d's render graph. The Camera2d (UI) doesn't get outlines because UI runs through a separate render pipeline (bevy_ui's UiCameraBundle path) that doesn't intersect `Core3d`'s outline nodes.

### Forward compatibility — Story 2.6/2.7 fallback path

Per architecture.md:887, the M1 tech-spike's go/fallback decision (Story 2.6) may flip the project to a flat+rim-light fallback (Story 2.7), which would require disabling `OutlinePlugin` registration. The current Story 2.4 wiring at `src/visual/mod.rs` is a single line:

```rust
app.add_plugins(bevy_mod_outline::OutlinePlugin);
```

Disabling this in Story 2.7 (if triggered) is a 1-line `cfg`-gate or removal. We do NOT pre-build the cfg-gate now (YAGNI per architecture.md:887: "Fallback scope is not pre-built"). The fallback comment marker in `src/visual/outline.rs` (Task 3 doc-comment) flags the file for Story 2.7's attention if needed.

### Forward compatibility — Story 4.x tuning growth

[Source: architecture.md:355-359 ("Runtime-tunable gameplay values live in `assets/config/tuning.ron`")]

Story 4.x adds enemy_hp, shot_cost, yield_multiplier, and other gameplay tunables to `TuningConfig`. The pattern Story 2.4 establishes (per-field `#[serde(default = "fn")]` + `default_<field_name>` fn + Default impl + RON entry + test coverage) is the template Story 4.x repeats for each new field.

The forward-compat test added in Task 6 (`tuning_config_legacy_2_3_schema_uses_defaults_for_outline_fields`) is the safety net: if Story 4.x's dev forgets `#[serde(default)]`, the test fails with a clear "missing field" RON error, and the dev knows to add the attribute.

### Test count discipline

Post-2.3: 12 tests passing. Post-2.4 expected: **13** (12 + 1 new forward-compat test). The two existing tuning tests are extended in place, not added.

If `cargo test` reports anything other than `13 passed`, root-cause:
- **<13:** Task 6's new test was forgotten; check `src/tuning/config.rs#cfg(test)`.
- **>13:** an unintended test was added (likely an integration test snuck in); review the diff in Task 9's `git status --short`.

### Integration test deferral

Architecture.md:354 defers integration tests post-M3. Story 2.4's runtime behavior (outline rendering, outline hot-reload propagation) is verified manually via `cargo run` + screenshot review (Task 7 + 8). When M3+ stories introduce integration tests, candidate cases for outline:

- `App::new() + TuningPlugin + VisualPlugin + spawn entity with OutlineVolume + emit AssetEvent::Modified, observe OutlineVolume.width updated`
- `App::new() + spawn cuboid mesh, advance frame, observe ATTRIBUTE_OUTLINE_NORMAL inserted into mesh asset`

These are not Story 2.4 scope.

### Project Structure Notes

- **Path alignment with architecture.md:** all new files match the architecture-mandated paths exactly:
  - `src/visual/outline.rs` ↔ `architecture.md:606` ✓
  - `assets/config/tuning.ron` (extended, not relocated) ↔ `architecture.md:618` ✓
- **No path conflicts or variances.**
- **Module layout convention:** `src/visual/` continues to grow as a single feature module per architecture.md:344-349; sub-files are flat (`palette.rs`, `toon_material.rs`, `outline.rs`, `reference_scene.rs`), no nested directories. Consistent with the architecture's "feature module" pattern.

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-2-vector-aesthetic-tech-spike.md#Story-2.4 (lines 95-119)]
- [Source: _bmad-output/planning-artifacts/prd.md#FR49 (line 569)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Source-Tree (lines 603-607, 618)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Plugin-Boundaries (line 654)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Constants-and-Tuning (lines 355-359)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Tech-Risk-Resolution (lines 885-887)]
- [Source: _bmad-output/implementation-artifacts/2-3-wgsl-toon-material-implementation.md (Story 2.3 inherited context, all sections)]
- [Source: _bmad-output/implementation-artifacts/1-2-plugin-compatibility-verification-gate.md (bevy_mod_outline 0.12 pinning rationale)]
- [Source: docs/plugin-compatibility.md (line 28: bevy_mod_outline 0.12 / 0.12.0 / Bevy 0.18.1 verified)]
- [Source: ~/.cargo/registry/src/index.crates.io-*/bevy_mod_outline-0.12.0/src/lib.rs (OutlinePlugin, OutlineVolume, OutlineMeshExt API surface)]
- [Source: ~/.cargo/registry/src/index.crates.io-*/bevy_mod_outline-0.12.0/CHANGELOG.md (0.12.0 "Removed deprecated bundles")]
- [Source: ~/.cargo/registry/src/index.crates.io-*/bevy_mod_outline-0.12.0/examples/shapes.rs (canonical OutlineVolume + generate_outline_normals usage)]

## Dev Agent Record

### Agent Model Used

claude-opus-4-7[1m] (dev-story workflow, 2026-04-29)

### Debug Log References

| Step | Command | Log Path | Grep Result |
|---|---|---|---|
| Task 7 — check | `cargo check` | `/tmp/story-2-4-check.log` | `warning:|error:` → **0** |
| Task 7 — build | `cargo build` | `/tmp/story-2-4-build.log` | `warning:|error:` → **0** |
| Task 7 — test | `cargo test` | `/tmp/story-2-4-test.log` | `13 passed; 0 failed` (post-fix; see Completion Note 1) |
| Task 7 — clippy | `cargo clippy --all-targets -- -D warnings` | `/tmp/story-2-4-clippy.log` | `warning:|error:` → **0** |
| Task 7 — fmt | `cargo fmt --all -- --check` | (stdout) | exit **0** |
| Task 7 — release | `cargo build --release` | `/tmp/story-2-4-release.log` | `warning:|error:` → **0**; build time **3m 35s** |
| Task 7 — runtime smoke | 12s `cargo run` | `/tmp/story-2-4-run.log` | `TuningReloaded` × **1** (cold-start), `entered MainMenu` × **1**, `panic` × **0**, `wgpu_core` warns × **2** (see Completion Note 2) |
| Task 7 — hot-reload smoke | `cargo run` + 2 RON edits + restore | `/tmp/story-2-4-hotreload.log` | `TuningReloaded` × **3** (1 cold + 2 edits), latencies <4 s including 3 s sleep wait |
| Task 9 — git status | `git status --short` | (stdout) | 5 modified + 2 added (story file + outline.rs) — matches story spec file set |
| Task 9 — outline grep | `grep -nrE 'OutlineVolume|OutlinePlugin|outline_width|outline_color|outline\.rs|generate_outline_normals' src/` | (stdout) | hits in `outline.rs`, `mod.rs`, `reference_scene.rs`, `tuning/config.rs` only — no other src files |
| Task 9 — out-of-scope state grep | `grep -nrE 'Arena|Caravan|...' src/ ! state.rs` | (stdout) | **0** hits |
| Task 9 — AssetServer::load count | `grep -rnE 'asset_server\.load|AssetServer::load' src/` | (stdout) | **1** hit (only in `tuning/mod.rs:39` — same as post-2.3) |
| Task 9 — Cargo unchanged | `git diff Cargo.toml Cargo.lock` | (stdout) | empty diff |
| Task 9 — dep versions | `cargo tree --depth 1` | (stdout) | `bevy v0.18.1`, `bevy_mod_outline v0.12.0` (unchanged) |

### Completion Notes List

1. **AC #5 — RON tuple syntax for `[f32; 4]` (deviation from spec).** The story spec said `outline_color: [0.05, 0.05, 0.05, 1.0]` in RON (square brackets), but RON's serde integration parses `[T; N]` fixed-size arrays as serde tuples → tuple syntax `(...)`. Square-bracket syntax produced `SpannedError { code: ExpectedStructLike, position: ... col: 110 }` at test time. Fixed by switching the RON value and the `tuning_config_deserializes_from_ron_bytes` test to tuple form `(1.0, 0.0, 0.0, 1.0)`. The Rust struct field type `[f32; 4]` is unchanged — the deviation is purely RON syntax. The `tuning_config_legacy_2_3_schema_uses_defaults_for_outline_fields` test is unaffected (it omits the field entirely, exercising `#[serde(default)]`).

2. **AC #1 — upstream wgpu warns from bevy_mod_outline shaders.** `cargo run` emits 2 `wgpu_core::device::resource: The fragment stage "fragment" output @location(0) values are ignored` WARN lines at first frame. Root cause: `bevy_mod_outline-0.12.0`'s stencil/extrude pipelines bind a fragment shader that doesn't write to color attachments (the depth-only stencil pass and the depth-prepass form of the volume pass). This is upstream plugin behavior, not our code. Verified by stepping through `OutlinePlugin::build` at `bevy_mod_outline-0.12.0/src/lib.rs:432-560` and inspecting the bound shaders. The warns are informational; they do not block rendering. Track for Story 2.5 (three-backend parity gate) so the dev knows to expect them in Vulkan/DX12 logs as well — not a regression. **Action:** none required for Story 2.4; document as an upstream-plugin-emission characteristic.

3. **AC #1, #2, #3, #4 — runtime hot-reload propagation confirmed wiring-end-to-end.** 3 `TuningReloaded` events fired during the hot-reload smoke (1 cold-start + 2 edits within ~6 seconds). The propagation system `apply_tuning_to_outlines` runs in `TuningSystems::Reload` alongside `apply_tuning_to_toon_materials`; both share the same Update tuple per Task 4. Visual verification of outline width/color rendering deltas requires Task 8 screenshots (deferred — see below).

4. **AC #5 — test count = 13.** 12 pre-existing tests + 1 new `tuning_config_legacy_2_3_schema_uses_defaults_for_outline_fields`. The two pre-existing tuning tests (`tuning_config_default_matches_ron_initial_values`, `tuning_config_deserializes_from_ron_bytes`) were extended in place to cover the new fields; they did not become "new" tests for count purposes. Matches the story-spec target.

5. **`Res<TuningHandle>` + `Res<Assets<TuningConfig>>` in `spawn_reference_scene` — initialization-order safety verified.** `TuningPlugin` is registered BEFORE `VisualPlugin` in `src/main.rs:38-39`, so by the time `OnEnter(GameState::Loading)` fires (first frame's StateTransition schedule, after Startup), both `Assets<TuningConfig>` (via `init_asset`) and `TuningHandle` (via `init_resource`) are guaranteed to exist. The cold-start race acknowledged in Task 5 (`tuning_assets.get(...).cloned().unwrap_or_default()`) handles the case where the .ron file is still parsing on the asset thread.

6. **Task 8 — visual capture confirmed by user 2026-04-29.** All 3 PNGs (`outline-baseline.png` 221 KB, `outline-thick.png` 221 KB, `outline-red.png` 221 KB) and `notes.md` present in `docs/tech-spike/m1-outline/`. `notes.md` confirms all visual ACs: (a) continuous silhouettes on all 3 placeholders, (b) cuboid corners smooth (`generate_outline_normals` payoff verified), (c) no z-fighting at default camera distance, (d) hot-reload latency acceptable (<1 s save→render), (e) no backend-specific caveats observed during dev (Story 2.5 will validate cross-backend formally).

7. **Task 10 — Commits + CI verified 2026-04-29 ✓.** Two-commit fold pushed in one operation per Story 2.3 precedent. Commits: `9011739` (`feat: bevy_mod_outline integration + outline hot-reload (Story 2.4)`) + `a226645` (`docs: M1 outline-integration spike evidence (Story 2.4)`). CI run **25102919639** all 4 jobs ✓ green: ubuntu-latest 2m26s, macos-latest 1m44s, windows-latest 9m00s, msrv-check 0m40s. The 2 wgpu_core fragment-stage warnings (per Completion Note 2) are upstream `bevy_mod_outline` shader artifacts, NOT counted as warnings by `cargo` (they emit at runtime, not compile-time, and CI runs `cargo build/test/clippy/fmt`, not `cargo run` headless).

8. **Task 11 — Story status flipped to `review`.** Sprint-status.yaml `2-4-bevy-mod-outline-integration-wiring` → `review`; story file Status header → `review`. `deferred-work.md` unchanged (no new findings; pre-existing 2.2/2.3 cfg_attr removal stays re-deferred to Story 4.5; pre-existing 2.3 RON-extension-collision concern stays open until Story 4.7).

### File List

**New:**
- `src/visual/outline.rs` — bevy_mod_outline integration module (TuningConfig→OutlineVolume hot-reload propagator).
- `_bmad-output/implementation-artifacts/2-4-bevy-mod-outline-integration-wiring.md` — this story file.

**Modified:**
- `src/visual/mod.rs` — `pub mod outline;` declaration; doc-comment fifth line; `OutlinePlugin` registration; `apply_tuning_to_outlines` added to existing `TuningSystems::Reload` Update tuple.
- `src/visual/reference_scene.rs` — `bevy_mod_outline` imports; `TuningConfig`/`TuningHandle` system params; `outline_volume()` closure consumed by all 3 placeholder spawns; `Cuboid::mesh().build()` + `generate_outline_normals` path for the cuboid ship; module doc unchanged.
- `src/tuning/config.rs` — `outline_width` + `outline_color` fields with per-field `#[serde(default = "fn")]`; `default_outline_width` + `default_outline_color` fns; `Default` impl extended; 2 existing tests extended; 1 new forward-compat test.
- `assets/config/tuning.ron` — 2 new lines (`outline_width: 3.0,` + `outline_color: (0.05, 0.05, 0.05, 1.0),`).
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `2-4-bevy-mod-outline-integration-wiring` status flipped `backlog → ready-for-dev → in-progress`; `last_updated` bumped (will flip to `review` at Task 11 completion).
