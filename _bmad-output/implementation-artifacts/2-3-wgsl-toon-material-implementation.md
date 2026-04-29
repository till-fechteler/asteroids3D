# Story 2.3: WGSL Toon Material Implementation

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the primary shader author,
I want a hand-written WGSL `ToonMaterial` implementing N·L posterization with configurable step count, rim-light term, and `SemanticAccent` tinting, plus a hot-reloadable `TuningConfig` resource that drives the shader uniforms,
So that FR49 toon-shading ships as a portfolio-quality self-authored artifact — the primary M1 learning target — and gameplay tuning can iterate without recompiles.

## Acceptance Criteria

1. **Given** `assets/shaders/toon.wgsl` is authored by hand
   **When** its fragment shader is reviewed
   **Then** shading is computed as `floor(max(dot(N,L), 0.0) * steps) / steps` posterization
   **And** a rim-light term `pow(1.0 - dot(N,V), rim_power) * rim_intensity` is additive to the posterized base
   **And** a `tint: vec4<f32>` uniform multiplies the final color
   **And** uniforms `steps: u32`, `rim_power: f32`, `rim_intensity: f32`, `tint: vec4<f32>` are declared in a single uniform buffer

2. **Given** `src/visual/toon_material.rs` is authored
   **When** it defines `ToonMaterial` implementing Bevy's `Material` trait
   **Then** `fragment_shader()` returns a handle to `assets/shaders/toon.wgsl`
   **And** `AsBindGroup` is derived and matches the WGSL uniform layout
   **And** `MaterialPlugin::<ToonMaterial>::default()` is registered inside `VisualPlugin`

3. **Given** `src/tuning/{mod.rs, config.rs}` defines a `TuningConfig` resource loaded from `assets/config/tuning.ron` with fields `toon_steps: u32`, `toon_rim_power: f32`, `toon_rim_intensity: f32`
   **When** `tuning.ron` is edited during `cargo run` (dev hot-reload enabled via `AssetPlugin::watch_for_changes_override`)
   **Then** `ToonMaterial` uniforms update live in the reference scene without restart

4. **Given** the reference scene's three placeholders
   **When** they are re-materialized with `ToonMaterial` instead of `StandardMaterial`
   **Then** each placeholder shows visible posterized banding
   **And** the rim-light term is visible at grazing angles on the asteroid silhouette
   **And** entities carrying a `SemanticAccent` component render with the corresponding `tint`

5. **Given** `toon_steps` is set to 3, then 5, then 8 via hot-reload
   **When** each value is observed on the asteroid
   **Then** the number of visible shading bands matches the uniform value within ±1 band (anti-aliasing tolerance)

## Tasks / Subtasks

- [x] **Task 1: Author `assets/shaders/toon.wgsl`** (AC: #1)
  - [x] Create directory `assets/shaders/` (first asset under this story; `assets/` does not yet exist).
  - [x] WGSL contents structure:
    1. `#import bevy_pbr::forward_io::VertexOutput` — provides `world_position`, `world_normal`, `uv` after vertex stage.
    2. `#import bevy_pbr::mesh_view_bindings::view` — provides `view.world_position` (camera position in world space) for the rim-light V vector.
    3. Define `struct ToonMaterial { tint: vec4<f32>, steps: u32, rim_power: f32, rim_intensity: f32 }`.
    4. Bind at `@group(2) @binding(0) var<uniform> material: ToonMaterial;` — group 2 is the material group in Bevy 0.18 (group 0 = view, 1 = mesh, 2 = material). **DO NOT** use group 0 or 1.
    5. `@fragment fn fragment(in: VertexOutput) -> @location(0) vec4<f32> { ... }` body:
       - `let N = normalize(in.world_normal);`
       - `let L = normalize(vec3<f32>(0.5, 1.0, 0.3));` — hardcoded light direction; **do not** read from a light uniform yet (lights stay out of scope until M2). Comment explaining why.
       - `let NdotL = max(dot(N, L), 0.0);`
       - `let posterized = floor(NdotL * f32(material.steps)) / f32(material.steps);` — exact AC #1 formula.
       - `let V = normalize(view.world_position - in.world_position.xyz);`
       - `let NdotV = max(dot(N, V), 0.0);`
       - `let rim = pow(1.0 - NdotV, material.rim_power) * material.rim_intensity;` — exact AC #1 formula.
       - `let lit = posterized + rim;` — additive composition per AC #1.
       - `return vec4<f32>(material.tint.rgb * lit, material.tint.a);` — tint multiplies posterized+rim.
  - [x] Hardcoded values rationale: light direction `(0.5, 1.0, 0.3)` (forward-up-right diagonal) matches the reference-scene's 3-point lighting key direction (`PointLight at (4.0, 5.0, 4.0)` from `reference_scene.rs:81-91`), so toon shading aligns visually with the existing dev scaffold.
  - [x] **Naming convention:** the WGSL `ToonMaterial` struct field names MUST match the Rust `ToonMaterial` struct field names (Bevy's `AsBindGroup` derive emits the same layout from the Rust side; mismatched names produce silent uniform-misalignment bugs that show up only at render time as wrong colors).
  - [x] No `#import bevy_pbr::pbr_functions` or `pbr_fragment` — those are Bevy's built-in PBR; we deliberately replace, not extend.

- [x] **Task 2: Author `assets/config/tuning.ron`** (AC: #3)
  - [x] Create directory `assets/config/`.
  - [x] File contents:
    ```ron
    TuningConfig(
        toon_steps: 4,
        toon_rim_power: 2.0,
        toon_rim_intensity: 0.3,
    )
    ```
  - [x] **Initial values rationale:**
    - `toon_steps: 4` — sweet-spot for visible banding without harsh stair-stepping; AC #5 verifies the dev can move this to 3/5/8 via hot-reload.
    - `toon_rim_power: 2.0` — quadratic falloff, classical rim-light idiom.
    - `toon_rim_intensity: 0.3` — ~30% of base color brightness; visible at silhouette without overpowering.
  - [x] No comments inside `tuning.ron` (RON does support `// comments` but keep this file minimal — comments belong in the Rust struct doc-comments, not the runtime data).

- [x] **Task 3: Author `src/tuning/{mod.rs, config.rs}`** (AC: #3)
  - [x] **New module location:** `src/tuning/` directory with `mod.rs` + `config.rs`. **NOT** `src/tuning.rs` (single file). The architecture mandates the directory form per `architecture.md:555-557` for forward compatibility — when Story 4.x adds enemy HP / shot cost / yield multipliers, they live in `config.rs` alongside the toon fields.
  - [x] **`src/tuning/config.rs` contents:**
    - `use bevy::asset::{io::Reader, AssetLoader, LoadContext};`
    - `use bevy::prelude::*;`
    - `use serde::Deserialize;`
    - `use thiserror::Error;`
    - Define `#[derive(Asset, TypePath, Debug, Clone, Deserialize)] pub struct TuningConfig { pub toon_steps: u32, pub toon_rim_power: f32, pub toon_rim_intensity: f32 }`. **`Asset` derive** is the load-bearing annotation; it makes Bevy treat the type as a loadable asset. **`TypePath` derive** is mandatory for Bevy 0.18 reflection registration. Do NOT add `Component` or `Resource` derives to the asset type.
    - Implement `Default for TuningConfig` returning the same values as the initial RON file (toon_steps: 4, toon_rim_power: 2.0, toon_rim_intensity: 0.3) — used as the fallback when the asset hasn't loaded yet.
    - Define `#[derive(Default)] pub struct TuningConfigLoader;`.
    - Define `#[derive(Debug, Error)] pub enum TuningConfigLoadError { #[error("io: {0}")] Io(#[from] std::io::Error), #[error("ron: {0}")] Ron(#[from] ron::error::SpannedError) }` — boundary error per architecture.md:366.
    - Implement `AssetLoader` for `TuningConfigLoader`:
      - `type Asset = TuningConfig;`
      - `type Settings = ();`
      - `type Error = TuningConfigLoadError;`
      - `async fn load(&self, reader: &mut dyn Reader, _: &(), _: &mut LoadContext<'_>) -> Result<TuningConfig, Self::Error>` reads `reader` to a `Vec<u8>` via `reader.read_to_end(&mut bytes).await?;` then `ron::de::from_bytes(&bytes).map_err(Into::into)`.
      - `fn extensions(&self) -> &[&str] { &["ron"] }`. **Caveat:** if a future story adds another `.ron` asset type (string-table per architecture.md:597), this loader will collide on extension. Track as deferred concern for Story 4.7 (string-table introduction) — solution there is `extensions: &["tuning.ron"]` (multi-segment) or a per-loader subdirectory convention.
  - [x] **`src/tuning/mod.rs` contents:**
    - `pub mod config;`
    - `use bevy::prelude::*;`
    - `use config::{TuningConfig, TuningConfigLoader};`
    - Define `pub struct TuningPlugin;`.
    - Define `#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)] pub enum TuningSystems { Reload }` — reserves a SystemSet slot per architecture.md:347 even though Story 2.3 only registers one system inside it (the reload propagator). Future tuning-driven systems can `.in_set(TuningSystems::Reload)` for ordering.
    - Define `#[derive(Resource, Default)] pub struct TuningHandle(pub Handle<TuningConfig>);` — holds the asset handle so systems can `Res<Assets<TuningConfig>>::get(&handle.0)`.
    - Implement `Plugin for TuningPlugin`:
      - `app.init_asset::<TuningConfig>();` — registers the asset type with Bevy's asset server.
      - `app.init_asset_loader::<TuningConfigLoader>();` — registers the loader.
      - `app.init_resource::<TuningHandle>();`
      - On `Startup`: a system that does `commands.insert_resource(TuningHandle(asset_server.load("config/tuning.ron")));`. Path relative to `assets/`.
      - `app.configure_sets(Update, TuningSystems::Reload);` — declares the set in Update schedule.
      - Reload-propagator system in `Update` (in `TuningSystems::Reload`): reads `EventReader<AssetEvent<TuningConfig>>`, on `AssetEvent::Modified { id }`, looks up the modified asset and emits a `TuningReloaded` event (defined locally as `#[derive(Event)] pub struct TuningReloaded(pub TuningConfig);`). VisualPlugin's material-update system subscribes via `EventReader<TuningReloaded>` to push uniforms.
  - [x] **Why an event, not direct mutation:** the architecture's Inter-System Communication Patterns (`architecture.md:243`) prescribe Events for discrete signals; "tuning was reloaded" is exactly that. VisualPlugin reading `Assets<TuningConfig>` directly would couple it to the asset-system's reload semantics; the event indirection keeps VisualPlugin testable in isolation.
  - [x] **Unit tests** at the bottom of `config.rs` inside `#[cfg(test)] mod tests`:
    - `tuning_config_default_matches_ron_initial_values`: build `TuningConfig::default()` and assert each field equals the values in `assets/config/tuning.ron` (4, 2.0, 0.3). Catches the kind of drift where someone edits the RON without updating Default.
    - `tuning_config_deserializes_from_ron_bytes`: feed the literal bytes `b"TuningConfig(toon_steps: 5, toon_rim_power: 1.5, toon_rim_intensity: 0.4)"` to `ron::de::from_bytes::<TuningConfig>(...)`, assert the parsed values. Catches Serde-derive misalignment with RON syntax.
    - **No** test of the `AssetLoader::load` async path — that requires a Bevy `LoadContext` mock which is non-trivial. Defer integration testing per architecture.md:354.
  - [x] Test count post-Task-3: **9 (current after Story 2.2 patches) + 2 (new tuning) = 11**. Capture this for the verification sweep grep.

- [x] **Task 4: Author `src/visual/toon_material.rs`** (AC: #2)
  - [x] **`src/visual/toon_material.rs` contents:**
    - `use bevy::pbr::Material;`
    - `use bevy::prelude::*;`
    - `use bevy::reflect::TypePath;`
    - `use bevy::render::render_resource::{AsBindGroup, ShaderRef};`
    - Define:
      ```rust
      #[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
      pub struct ToonMaterial {
          #[uniform(0)]
          pub tint: LinearRgba,
          #[uniform(0)]
          pub steps: u32,
          #[uniform(0)]
          pub rim_power: f32,
          #[uniform(0)]
          pub rim_intensity: f32,
      }
      ```
      **Field order matters** for the WGSL struct alignment — `vec4<f32>` (16 bytes) first, then three scalars. Bevy's `AsBindGroup` derive packs uniform fields in declaration order; rearranging breaks the WGSL buffer layout silently.
      **`LinearRgba` not `Color`** — `Color` is a tagged enum with multiple color spaces; uniforms need a flat vec4 representation. `LinearRgba` serializes as `[r, g, b, a]: f32`. The Rust→WGSL bridge requires this.
    - `impl Default for ToonMaterial` returning `tint: LinearRgba::WHITE, steps: 4, rim_power: 2.0, rim_intensity: 0.3` — same defaults as `TuningConfig`.
    - `impl Material for ToonMaterial { fn fragment_shader() -> ShaderRef { "shaders/toon.wgsl".into() } }`. **DO NOT override `vertex_shader()`** — Bevy's default vertex shader produces `VertexOutput` with `world_position` + `world_normal` already, and the WGSL imports `bevy_pbr::forward_io::VertexOutput` which matches.
  - [x] **No** `Component` derive on `ToonMaterial` — materials are inserted via `MeshMaterial3d(handle)` component (Bevy 0.18 idiom), not as a direct component.
  - [x] **Unit test** at the bottom inside `#[cfg(test)] mod tests`:
    - `toon_material_default_matches_tuning_default`: assert each field equals the matching `TuningConfig::default()` field (importing the tuning module). Guards against the two defaults drifting apart.
  - [x] Test count post-Task-4: **11 + 1 = 12**.

- [x] **Task 5: Wire `MaterialPlugin<ToonMaterial>` into `VisualPlugin`** (AC: #2)
  - [x] In `src/visual/mod.rs`:
    - Add `pub mod toon_material;` after `pub mod palette;` — both are public submodules under VisualPlugin.
    - Update top-of-file `//!` doc-comment line 3 (currently mentions Story 2.2) to add a fourth line: "Story 2.3 adds the WGSL `ToonMaterial` (FR49) wired through `MaterialPlugin`."
    - In `impl Plugin for VisualPlugin::build`:
      - Before the existing `app.configure_sets(...)` line, add: `app.add_plugins(MaterialPlugin::<toon_material::ToonMaterial>::default());`
      - **Order matters:** `MaterialPlugin` registers the material's render pipeline, which other systems may depend on. Putting it first keeps the dependency graph clean.
  - [x] **No** new `VisualSystems` enum variant. `MaterialPlugin` runs in its own internal system sets; we don't ordering-couple VisualPlugin's user systems against material rendering.
  - [x] **No** `pub use toon_material::*` re-export from `mod.rs` — qualified paths only (`crate::visual::toon_material::ToonMaterial`), per the architecture pattern established in Story 2.2.

- [x] **Task 6: Configure `AssetPlugin` for hot-reload watching** (AC: #3, #5)
  - [x] Edit `src/main.rs`:
    - The `App::new().add_plugins(DefaultPlugins)` line (or equivalent) becomes `App::new().add_plugins(default_plugins())` where `default_plugins()` is a private helper that returns `PluginGroup`.
    - **OR** (preferred, less indirection): edit `App::new().add_plugins(DefaultPlugins.set(AssetPlugin { watch_for_changes_override: cfg!(debug_assertions).then_some(true), ..default() }))`.
    - **`cfg!(debug_assertions).then_some(true)`** evaluates to `Some(true)` in debug, `None` in release. `None` lets Bevy fall back to its default (no watching), keeping release builds free of file-watcher overhead.
    - **Why not `#[cfg(debug_assertions)]` block?** — the `.set(AssetPlugin { ... })` invocation must compile in both profiles; using a runtime conditional inside the field value is the cleanest pattern.
  - [x] **Register `TuningPlugin`:** add `.add_plugins(crate::tuning::TuningPlugin)` to `App::new()` chain. Place it AFTER `DefaultPlugins.set(...)` (it depends on `AssetPlugin` being initialized) and BEFORE `VisualPlugin` (so the asset is loading by the time VisualPlugin's update systems first run).
  - [x] **Bevy 0.18 caveat:** `AssetPlugin::watch_for_changes_override` does NOT exist in older Bevy releases (0.13 had `AssetPlugin::file_path` watching as a feature flag); 0.18 unified to `watch_for_changes_override: Option<bool>`. Compile errors here mean the Bevy version drift; consult `bevy::asset::AssetPlugin` source.

- [x] **Task 7: Update `src/visual/reference_scene.rs` to materialize with `ToonMaterial`** (AC: #4)
  - [x] **Replace** `Assets<StandardMaterial>` with `Assets<ToonMaterial>` in `spawn_reference_scene`'s parameters.
  - [x] **Replace** the three `StandardMaterial { base_color: ..., ..default() }` constructions with `ToonMaterial { tint: <LinearRgba from SemanticAccent>, ..default() }`. Mappings:
    - Asteroid (currently grey-brown): tint = `color_for(SemanticAccent::Hazard).into()` (yellow). The asteroid is the rim-light demo subject; yellow rim against the dark-grey backdrop reads cleanest.
    - Ship-cockpit placeholder (currently dark-blue): tint = `color_for(SemanticAccent::PlayerOwned).into()` (sky-blue). Semantic — this represents the player's ship.
    - Projectile placeholder (currently bright-yellow): tint = `color_for(SemanticAccent::Salvage).into()` (bluish-green). Arbitrary at this stage; will be re-tagged in Story 4.5 when projectiles get faction semantics.
  - [x] **Replace** `MeshMaterial3d<StandardMaterial>` with `MeshMaterial3d<ToonMaterial>` on each spawn entity.
  - [x] **Attach `SemanticAccent` component** to each placeholder for AC #4's "entities carrying a `SemanticAccent` component render with the corresponding `tint`" clause:
    - asteroid: `SemanticAccent::Hazard`
    - ship: `SemanticAccent::PlayerOwned`
    - projectile: `SemanticAccent::Salvage`
    Even though the `ToonMaterial::tint` is the actual driver of color, the `SemanticAccent` component on the entity proves AC #4's contract that future shader systems can read the component. The current Story-2.3 implementation reads the component at spawn-time to produce the tint; Story 4.5 will wire a system that propagates component changes back into the material.
  - [x] **Imports update:** add `use super::toon_material::ToonMaterial;` to `reference_scene.rs`. The existing `use super::palette::{SemanticAccent, color_for};` already covers the palette dependency.
  - [x] **Lights stay unchanged.** The 3 `PointLight` entities from Story 2.1 remain; the WGSL hardcoded light direction `(0.5, 1.0, 0.3)` doesn't read them, but they're cheap and Story 2.5's parity-validation gate will benefit from a non-trivial scene.
  - [x] **Camera3d order: -1 unchanged.** Swatch UI Camera2d (order: 1) remains atop. The toon-shaded geometry renders to the Camera3d.

- [x] **Task 8: Hot-reload propagation system in `VisualPlugin`** (AC: #3, #5)
  - [x] In `src/visual/mod.rs` (or a new `src/visual/material_update.rs` for separation), add a system:
    ```rust
    fn apply_tuning_to_toon_materials(
        mut events: EventReader<crate::tuning::TuningReloaded>,
        mut materials: ResMut<Assets<toon_material::ToonMaterial>>,
    ) {
        for event in events.read() {
            for (_, material) in materials.iter_mut() {
                material.steps = event.0.toon_steps;
                material.rim_power = event.0.toon_rim_power;
                material.rim_intensity = event.0.toon_rim_intensity;
            }
        }
    }
    ```
    Iterates ALL `ToonMaterial` assets (Story 2.3 has 3 instances; Story 2.4+ may add more) — the tuning is global per AC #3. Tints are NOT touched (those come from per-entity `SemanticAccent`).
  - [x] Register the system in `VisualPlugin::build`:
    `app.add_systems(Update, apply_tuning_to_toon_materials.in_set(crate::tuning::TuningSystems::Reload));`
  - [x] **Why iterate all materials, not just the asset-changed one?** The simpler approach (broadcast on event) is correct and zero-cost when no event fires. Filtering would require tracking per-material handles, which is overhead for the 3-material scale of Story 2.3.
  - [x] **Cold-start behavior:** on first `cargo run`, the `Startup`-spawned materials use `ToonMaterial::default()` values; once `tuning.ron` finishes loading (typically within the first few frames), `AssetEvent::Created` fires (NOT `Modified` — that's only for subsequent edits). The reload propagator should also handle `AssetEvent::LoadedWithDependencies` / `AssetEvent::Added` to apply initial values from RON if they differ from `Default`. **Decision:** use `AssetEvent::Modified | AssetEvent::Added` in the match. This guarantees the RON values overwrite the Default values once the asset finishes loading.
  - [x] **Test count post-Task-8:** unchanged at 12 (no new unit tests; this is a runtime-integration system).

- [x] **Task 9: Remove Story 2.2 deferred `cfg_attr` blocks from `palette.rs`** (Story 2.2 deferred-work entry)
  - [x] Per `_bmad-output/implementation-artifacts/deferred-work.md` "Removal-on-graduation" entry (2026-04-28):
    - Delete the `#[cfg_attr(not(debug_assertions), allow(dead_code, reason = "..."))]` block on `pub enum SemanticAccent` (`src/visual/palette.rs:7-12`).
    - Delete the same block on `pub fn color_for` (`src/visual/palette.rs:23-29`).
  - [x] **Why now:** Story 2.3's Task 7 attaches `SemanticAccent` as a Component on reference-scene entities (release-build path) AND `color_for` is called at spawn-time to produce `LinearRgba` tint values. The cfg-elision concern is resolved: both items have non-debug consumers.
  - [x] **Verification step:**
    - `cargo build --release 2>&1 | tee /tmp/story-2-3-release.log` → `grep -cE 'warning:|error:' /tmp/story-2-3-release.log` == 0.
    - `nm target/release/asteroids3D 2>/dev/null | grep -c color_for` ≥ **1** (deferred-work entry's "Resolution path" item (c)). Spec line 122 of Story 2.2 retroactively becomes satisfiable.
  - [x] If the warnings DO reappear (meaning Task 7's wiring isn't actually a release consumer), **DO NOT** re-add the cfg_attr — instead, root-cause why the Task-7 SemanticAccent attach isn't reaching release. Likely candidate: `reference_scene.rs` is still cfg(debug_assertions)-gated, so its insertion of `SemanticAccent` doesn't count as a release consumer. In that case, the cfg_attr removal stays deferred to Story 4.5 (which attaches `SemanticAccent` in non-debug paths). Update the deferred-work entry accordingly.

- [x] **Task 10: Local verification sweep — code paths** (AC: #1, #2, #3, #4, #5)
  - [x] `cargo check 2>&1 | tee /tmp/story-2-3-check.log` → `grep -cE 'warning:|error:' /tmp/story-2-3-check.log` must equal **0**.
  - [x] `cargo build 2>&1 | tee /tmp/story-2-3-build.log` → same grep equals **0**.
  - [x] `cargo test 2>&1 | tee /tmp/story-2-3-test.log` → grep `'warning:|error:|FAILED'` equals **0**; test count **12 passed, 0 failed** (9 prior + 2 tuning + 1 toon_material).
  - [x] `cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-2-3-clippy.log` → grep equals **0**. Watch for: `clippy::cast_precision_loss` on `f32(material.steps)` (WGSL is fine — clippy doesn't lint WGSL), or `clippy::needless_pass_by_value` on the system parameters. If false-positives, suppress with `#[expect(...)]` and document.
  - [x] `cargo fmt --all -- --check` → exit 0.
  - [x] `cargo build --release 2>&1 | tee /tmp/story-2-3-release.log` → grep equals **0** (Task 9 prerequisite).
  - [x] `nm target/release/asteroids3D | grep -c color_for` ≥ **1** (Task 9 verification).
  - [x] **Debug-build runtime verification (AC #4):**
    - `cargo run &> /tmp/story-2-3-run.log &` → wait ≥ 5 seconds for the splash → MainMenu transition + the asset loader's first `tuning.ron` parse.
    - During MainMenu state: visually verify the 3 toon-shaded placeholders show **posterized banding** (visible discrete shading steps, not smooth gradient). Asteroid silhouette should show the **rim-light** (bright edge) at grazing angles. Each placeholder shows its assigned `SemanticAccent` tint (Hazard yellow, PlayerOwned sky-blue, Salvage bluish-green).
    - `grep -c 'entered MainMenu' /tmp/story-2-3-run.log` ≥ 1.
    - `grep -cE 'warning:|error:|ERROR ' /tmp/story-2-3-run.log` should be 0 for app-emitted lines (the deferred-work splash-cleanup-iteration race WARN remains known and unrelated).
  - [x] **Hot-reload runtime verification (AC #3, #5):**
    - With `cargo run` still running, edit `assets/config/tuning.ron` and change `toon_steps: 4` → `toon_steps: 8`. Save.
    - Within ~1 second the asteroid should show **8 visible bands** instead of 4 (AC #5 ±1 anti-aliasing tolerance).
    - Edit again to `toon_steps: 3` → asteroid shows **3 bands**. Save and confirm.
    - Edit `toon_rim_intensity: 0.3` → `toon_rim_intensity: 0.7` → rim-light gets visibly brighter on the asteroid silhouette.
    - Restore initial values (4 / 2.0 / 0.3) before closing.
    - `grep -c 'TuningReloaded' /tmp/story-2-3-run.log` should match the number of edits (3+ events fired).

- [x] **Task 11: Visual capture (`docs/tech-spike/m1-toon/`)** (AC: #4 evidence)
  - [x] Create directory `docs/tech-spike/m1-toon/`.
  - [x] **Three screenshots** capturing Story 2.3's visual spike:
    - `docs/tech-spike/m1-toon/toon-baseline.png` — `toon_steps: 4`, `rim_intensity: 0.3` (initial RON values). Shows the 3 placeholders with default banding + rim.
    - `docs/tech-spike/m1-toon/toon-steps-8.png` — `toon_steps: 8`. Shows finer banding on the asteroid.
    - `docs/tech-spike/m1-toon/toon-steps-3.png` — `toon_steps: 3`. Shows coarser banding.
  - [x] Capture: `Cmd-Shift-4` → window-frame select → save to `docs/tech-spike/m1-toon/<name>.png`. Resolution = native window resolution; PNG, lossless.
  - [x] **`docs/tech-spike/m1-toon/notes.md`** — short markdown capturing:
    - The 3 hex tints actually rendered (they should match the 3 `SemanticAccent` mappings).
    - Whether the rim-light is visible on the asteroid silhouette (yes/no, qualitative).
    - Whether posterization is visible at default `steps: 4` (yes/no).
    - Whether hot-reload latency is acceptable (`< 1s` from save to render — yes/no).
    - Any backend-specific caveats observed during dev (Story 2.5 handles cross-backend parity formally; this is just the dev's quick "did it work" log).

- [x] **Task 12: Scope guardrails — verify nothing else drifted** (AC: all)
  - [x] `git status --short`: expected file set:
    - `assets/shaders/toon.wgsl` (??) — new
    - `assets/config/tuning.ron` (??) — new
    - `src/tuning/mod.rs` (??) — new
    - `src/tuning/config.rs` (??) — new
    - `src/visual/toon_material.rs` (??) — new
    - `src/visual/mod.rs` (M) — `pub mod toon_material;` + doc-comment + MaterialPlugin registration + apply_tuning_to_toon_materials system registration
    - `src/visual/reference_scene.rs` (M) — material switch + SemanticAccent attach + import update
    - `src/visual/palette.rs` (M) — cfg_attr removal (Task 9)
    - `src/main.rs` (M) — AssetPlugin watch override + TuningPlugin registration
    - `Cargo.toml` (??) — **untouched** unless Bevy's Asset/Material derives need a feature flag (verify via cargo build)
    - `docs/tech-spike/m1-toon/{toon-baseline,toon-steps-8,toon-steps-3}.png` + `notes.md` — new (Task 11)
    - Bookkeeping: this story file (??) + `sprint-status.yaml` (M) + `deferred-work.md` (M) — flipped at Task 14.
  - [x] `grep -nrE 'TuningConfig|toon|outline_material|outline\.rs' src/ --include='*.rs'` → expected hits:
    - `src/tuning/{mod.rs, config.rs}`: many (own definitions).
    - `src/visual/{mod.rs, toon_material.rs, reference_scene.rs}`: a few each.
    - `src/main.rs`: 1 (TuningPlugin registration).
    - **No** hits for `outline_material` or `outline.rs` — that's Story 2.4.
  - [x] `grep -nrE 'Arena|Caravan|PostRun|PhotoMode|Paused' src/ --include='*.rs' | grep -v 'state.rs'` → 0 hits (states still not live).
  - [x] `grep -rn 'AssetServer::load\b' src/` → expected hits: **2** — one in `src/tuning/mod.rs` (loading `tuning.ron`), zero elsewhere. **No** font, image, or shader file loaded via `AssetServer::load` directly — shader is loaded by Bevy's `MaterialPlugin` internally based on the `ShaderRef::Path` returned from `Material::fragment_shader()`.
  - [x] `grep -rn 'pub mod\|pub fn\|pub struct\|pub enum' src/` should expose the **post-2.3 set**: `pub mod tuning` (new), `pub mod visual::palette`, `pub mod visual::toon_material` (new), `pub struct TuningPlugin`, `pub struct VisualPlugin`, `pub enum TuningSystems`, `pub enum VisualSystems`, `pub enum SemanticAccent`, `pub fn color_for`, `pub struct TuningConfig` + `TuningConfigLoader`, `pub struct ToonMaterial`, `pub struct TuningHandle`, `pub struct TuningReloaded`. Approx. 13 public items in `src/`.
  - [x] `Cargo.toml / Cargo.lock` **untouched**. Verified by `git status --short Cargo.toml Cargo.lock` → empty. The existing `ron = "0.8"`, `serde = "1"`, `thiserror = "2"` deps cover all new requirements; `bevy::asset::AssetLoader` + `bevy::reflect::TypePath` + `bevy::pbr::Material` are stock Bevy 0.18 (already pulled by `bevy` dep with `"3d"` feature).
  - [x] `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `docs/plugin-compatibility.md`, `src/state.rs`, `src/splash.rs`, `src/logging.rs` — **all untouched**.

- [ ] **Task 13: Commit + CI observation** (AC: all)
  - [ ] **Commit 1 (source + assets):** stage `src/tuning/{mod,config}.rs`, `src/visual/{toon_material,mod,reference_scene,palette}.rs`, `src/main.rs`, `assets/shaders/toon.wgsl`, `assets/config/tuning.ron`. **No** docs, **no** Cargo files.
    - HEREDOC commit message: `feat: WGSL ToonMaterial + TuningConfig hot-reload + SemanticAccent tinting (Story 2.3)`. Single-line, under 90 chars (slightly over the 70-char ideal — concession for the multi-feature scope).
    - Push to `origin/master`. Triggers full 4-job CI matrix. Cargo cache warm; Cargo.lock unchanged → fast.
    - `gh run list -L 1` → identify run ID. Wait for all 4 jobs (`build {ubuntu,macos,windows}-latest` + `msrv-check`) to complete.
    - `gh run view <ID> --log | grep -cE 'warning:|error:'` → 0.
    - All 4 jobs ✅; capture run ID + per-job durations.
  - [ ] **Commit 2 (docs):** stage `docs/tech-spike/m1-toon/{toon-baseline,toon-steps-8,toon-steps-3}.png` + `notes.md`.
    - HEREDOC commit message: `docs: M1 toon-shader spike evidence (Story 2.3)`.
    - Push. Triggers CI (cached). Capture run ID.
    - **Push-fold optimization:** if Till opts to fold both commits into one push (precedent from Story 2.2), only one CI run-ID is captured. Document the fold reasoning in Dev Agent Record like Story 2.2 did.

- [ ] **Task 14: Ready-for-review handoff + bookkeeping commit**
  - [ ] Populate **Dev Agent Record**: Agent Model Used, Debug Log References (per-command grep counts + log paths + screenshot capture metadata + CI run IDs + nm-symbol counts pre/post Task 9), Completion Notes (per-AC evidence + any deviations from spec + the Task-9 cfg_attr removal status), File List (added / modified).
  - [ ] Set this story's `Status:` header → `review`.
  - [ ] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`: flip `2-3-wgsl-toon-material-implementation: ready-for-dev → in-progress → review`; bump `last_updated`.
  - [ ] **Update `deferred-work.md`:** mark the Story-2.2 "Removal-on-graduation" entry as ✅ RESOLVED with date + commit ref pointing at Task 9's deletion. Mirrors the resolution pattern at deferred-work.md:19 (typo-sweep) and :56 (1.6 main return).
  - [ ] Stage story file + `sprint-status.yaml` + `deferred-work.md`, commit with `bmad: story 2.3 ready-for-dev → review (toon material + hot-reload, CI <ID> green)`. `_bmad-output/**` paths-ignored → no CI cost.
  - [ ] Push.
  - [ ] Story awaits code review. **Multi-LLM adversarial review recommended** for this story given the surface area (WGSL + Material + AssetLoader + 2 plugins + hot-reload), unlike Stories 1.x/2.1/2.2 which were small enough for a single light-mode pass. Run `bmad-code-review` ideally with a different LLM than the implementer.

## Dev Notes

### Why this story exists

Story 2.3 lands the **primary M1 learning target** — a hand-authored WGSL Toon shader integrated through Bevy's `Material` trait. [Source: epics/epic-2-vector-aesthetic-tech-spike.md:60-94; prd.md:64, 367, 405; architecture.md:221] This is the FR49 anchor and the portfolio-quality artifact Till wants from M1.

Three things ship together in this story because they're tightly coupled:
1. **WGSL fragment shader** (`assets/shaders/toon.wgsl`) — the actual GPU code Till is authoring.
2. **`ToonMaterial` Rust binding** (`src/visual/toon_material.rs`) — the Material-trait wrapper that connects the shader to Bevy's render pipeline.
3. **`TuningConfig` hot-reload** (`src/tuning/{mod,config}.rs` + `assets/config/tuning.ron`) — the gameplay-tuning infrastructure mandated by architecture.md:358 ("recompile-per-tweak burns motivation"). The toon shader's three knobs (steps, rim_power, rim_intensity) are the first tunables in the codebase; the infrastructure introduced here is reused in Story 4.x for enemy HP, shot cost, etc.

### Inherited context from Stories 2.1 + 2.2

| Fact | Value | Source |
|---|---|---|
| `src/visual/mod.rs` (post-2.2) | `pub mod palette`, `VisualPlugin` skeleton, `VisualSystems::Setup` enum, cfg-gated `mod reference_scene` | `src/visual/mod.rs` (post-2.2) |
| `src/visual/palette.rs` (post-2.2) | `pub enum SemanticAccent { Enemy, Salvage, Hazard, PlayerOwned, Neutral }` (Component-derived, Default = Neutral), `pub fn color_for(SemanticAccent) -> Color` (Wong 2011 hex values), 6 unit tests (Story 2.2 patches added 3 RGB-pin tests) | `src/visual/palette.rs` post-2.2 patches |
| `src/visual/reference_scene.rs` (post-2.2) | `ReferenceScenePlugin` (cfg-gated), `spawn_reference_scene` on `OnEnter(Loading)` (3 placeholders + 3 lights + Camera3d order:-1), `spawn_palette_swatches` on `OnEnter(MainMenu)` (5 swatches + Camera2d order:1), all entities tagged `ReferenceSceneEntity` | post-2.2 |
| `cfg_attr(not(debug_assertions), allow(dead_code))` on palette items | Present (Story 2.2 mitigation); **REMOVE** in Task 9 once Task 7 wires `color_for` into release-path code | `palette.rs:7-12, 23-29` post-2.2 |
| Test count post-2.2 | **9 passing** (3 prior + 6 palette tests after 2.2 patches) | post-2.2 |
| Bevy version | `0.18` (resolved `0.18.1`), features `["3d", "png", "bevy_ui", "default_font"]` (+ x11/wayland on Linux) | `Cargo.toml:8,23-26` |
| `ron = "0.8"` already pulled | Present in `Cargo.toml:17` — no Cargo edit needed for RON deserialization | `Cargo.toml:17` |
| `serde = "1"` with `derive` | Present in `Cargo.toml:14` | `Cargo.toml:14` |
| `thiserror = "2"` | Present in `Cargo.toml:16` — used for `TuningConfigLoadError` | `Cargo.toml:16` |
| Story 2.5 dependency | Will run `cargo run --release` on three backends, capture screenshots from a fixed `Transform`. **2.3 must work in release** for 2.5 to validate. Task 10's release-build verification de-risks 2.5. | `epics/epic-2-vector-aesthetic-tech-spike.md:121-147` |
| Story 2.4 dependency | Will add `bevy_mod_outline` plus `outline_width` + `outline_color` fields to `TuningConfig`. **2.3's `TuningConfig` schema must be extensible** without breaking 2.4. The chosen `Asset + Deserialize` derives + simple struct fields satisfy this — adding fields to the RON file is a backward-compatible Serde extension. | `epics/epic-2-vector-aesthetic-tech-spike.md:95-119` |

### Bevy 0.18 `Material` trait essentials

[Source: Bevy 0.18 docs, `bevy::pbr::Material` trait + `bevy::render::render_resource::AsBindGroup` derive]

The `Material` trait is the high-level entry point for custom shaders in Bevy. To implement a custom material:

1. **Declare the material struct** with `#[derive(Asset, AsBindGroup, TypePath, Clone, Debug)]`. The `AsBindGroup` derive generates the bind-group layout from `#[uniform(binding_index)]`-annotated fields.

2. **Implement `Material`** with at minimum `fn fragment_shader() -> ShaderRef`. The shader path is relative to `assets/`. Bevy's default vertex shader is sufficient for non-skeletal meshes; only override `vertex_shader()` if you need custom vertex transforms.

3. **Register** the material with `app.add_plugins(MaterialPlugin::<MyMaterial>::default())`. This single plugin provides: asset loading, render pipeline setup, draw command emission, batching/culling integration. Do NOT manually wire systems — `MaterialPlugin` handles all of it.

4. **Bind groups in WGSL:** Bevy 0.18 conventions:
   - `@group(0)` — view bindings (camera, time)
   - `@group(1)` — mesh bindings (model matrix, normal matrix)
   - `@group(2)` — material bindings (your uniforms)
   - `@group(3)` — extra (rare; skinning weights etc.)
   ALWAYS use `@group(2)` for material uniforms in custom materials. Mismatch produces silent wrong-color rendering.

5. **Uniform layout:** WGSL has strict alignment rules (vec4 = 16-byte aligned, scalars in trailing positions can pack). Bevy's `AsBindGroup` derive generates the Rust→GPU memory layout based on field declaration order. **Match the WGSL struct field order exactly to the Rust struct field order**, or unaligned reads produce gibberish colors.

### Reference WGSL skeleton (`assets/shaders/toon.wgsl`)

The dev agent can write near-verbatim. Bevy 0.18's `bevy_pbr::forward_io::VertexOutput` provides exactly the fields below (`world_position`, `world_normal`, `uv`).

```wgsl
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct ToonMaterial {
    tint: vec4<f32>,
    steps: u32,
    rim_power: f32,
    rim_intensity: f32,
};

@group(2) @binding(0) var<uniform> material: ToonMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Hardcoded forward-up-right diagonal light direction.
    // Lights stay out of M1 scope; M2 may replace this with a uniform fed from a Bevy DirectionalLight.
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let normal = normalize(in.world_normal);

    // Posterized N·L: AC #1 formula
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let steps_f = f32(material.steps);
    let posterized = floor(n_dot_l * steps_f) / steps_f;

    // Rim light: AC #1 formula
    let view_dir = normalize(view.world_position - in.world_position.xyz);
    let n_dot_v = max(dot(normal, view_dir), 0.0);
    let rim = pow(1.0 - n_dot_v, material.rim_power) * material.rim_intensity;

    // Composition: tint × (posterized + rim)
    let lit = posterized + rim;
    return vec4<f32>(material.tint.rgb * lit, material.tint.a);
}
```

### Reference Rust skeleton (`src/visual/toon_material.rs`)

```rust
//! Custom WGSL Toon Material — FR49 portfolio-quality shader artifact.
//! See assets/shaders/toon.wgsl for the fragment-stage WGSL.

use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct ToonMaterial {
    #[uniform(0)]
    pub tint: LinearRgba,
    #[uniform(0)]
    pub steps: u32,
    #[uniform(0)]
    pub rim_power: f32,
    #[uniform(0)]
    pub rim_intensity: f32,
}

impl Default for ToonMaterial {
    fn default() -> Self {
        Self {
            tint: LinearRgba::WHITE,
            steps: 4,
            rim_power: 2.0,
            rim_intensity: 0.3,
        }
    }
}

impl Material for ToonMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/toon.wgsl".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuning::config::TuningConfig;

    #[test]
    fn toon_material_default_matches_tuning_default() {
        let m = ToonMaterial::default();
        let t = TuningConfig::default();
        assert_eq!(m.steps, t.toon_steps);
        assert_eq!(m.rim_power, t.toon_rim_power);
        assert_eq!(m.rim_intensity, t.toon_rim_intensity);
    }
}
```

### Reference Rust skeleton (`src/tuning/config.rs`)

```rust
//! Hot-reloadable tuning resource — gameplay knobs in assets/config/tuning.ron.

use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::Deserialize;
use thiserror::Error;

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct TuningConfig {
    pub toon_steps: u32,
    pub toon_rim_power: f32,
    pub toon_rim_intensity: f32,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            toon_steps: 4,
            toon_rim_power: 2.0,
            toon_rim_intensity: 0.3,
        }
    }
}

#[derive(Default)]
pub struct TuningConfigLoader;

#[derive(Debug, Error)]
pub enum TuningConfigLoadError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("ron: {0}")]
    Ron(#[from] ron::error::SpannedError),
}

impl AssetLoader for TuningConfigLoader {
    type Asset = TuningConfig;
    type Settings = ();
    type Error = TuningConfigLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<TuningConfig, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(ron::de::from_bytes(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuning_config_default_matches_ron_initial_values() {
        let cfg = TuningConfig::default();
        assert_eq!(cfg.toon_steps, 4);
        assert_eq!(cfg.toon_rim_power, 2.0);
        assert_eq!(cfg.toon_rim_intensity, 0.3);
    }

    #[test]
    fn tuning_config_deserializes_from_ron_bytes() {
        let bytes = b"TuningConfig(toon_steps: 5, toon_rim_power: 1.5, toon_rim_intensity: 0.4)";
        let cfg: TuningConfig = ron::de::from_bytes(bytes).unwrap();
        assert_eq!(cfg.toon_steps, 5);
        assert_eq!(cfg.toon_rim_power, 1.5);
        assert_eq!(cfg.toon_rim_intensity, 0.4);
    }
}
```

### Reference Rust skeleton (`src/tuning/mod.rs`)

```rust
//! TuningPlugin — owns TuningConfig asset handle + hot-reload propagation.

pub mod config;

use bevy::prelude::*;
use config::{TuningConfig, TuningConfigLoader};

pub struct TuningPlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TuningSystems {
    Reload,
}

#[derive(Resource, Default)]
pub struct TuningHandle(pub Handle<TuningConfig>);

#[derive(Event, Debug, Clone)]
pub struct TuningReloaded(pub TuningConfig);

impl Plugin for TuningPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TuningConfig>()
            .init_asset_loader::<TuningConfigLoader>()
            .init_resource::<TuningHandle>()
            .add_event::<TuningReloaded>()
            .configure_sets(Update, TuningSystems::Reload)
            .add_systems(Startup, load_tuning)
            .add_systems(Update, propagate_tuning_reload.in_set(TuningSystems::Reload));
    }
}

fn load_tuning(asset_server: Res<AssetServer>, mut handle: ResMut<TuningHandle>) {
    handle.0 = asset_server.load("config/tuning.ron");
}

fn propagate_tuning_reload(
    mut events: EventReader<AssetEvent<TuningConfig>>,
    assets: Res<Assets<TuningConfig>>,
    handle: Res<TuningHandle>,
    mut writer: EventWriter<TuningReloaded>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } if *id == handle.0.id() => {
                if let Some(cfg) = assets.get(handle.0.id()) {
                    writer.write(TuningReloaded(cfg.clone()));
                }
            }
            _ => {}
        }
    }
}
```

### Architecture compliance — naming, module layout, plugin pattern

**Plugin / SystemSet naming (architecture.md:326-328):** ✓
- `TuningPlugin` — `<Feature>Plugin` PascalCase per convention.
- `TuningSystems::Reload` — `<Feature>Systems` enum with `Reload` variant per architecture.md:347.
- `VisualPlugin` already exists from 2.1; extended (not replaced) by Story 2.3.

**Module layout (architecture.md:344-349, 555-557, 603-607):** ✓
- `src/tuning/` directory with `mod.rs` + `config.rs` matches architecture line 555-557 exactly.
- `src/visual/toon_material.rs` matches architecture line 605 exactly.
- `pub mod toon_material;` exposes the module via the qualified path `crate::visual::toon_material::ToonMaterial`. **No** `pub use toon_material::*;` re-export.
- `assets/shaders/toon.wgsl` matches architecture line 632 exactly.
- `assets/config/tuning.ron` matches architecture line 618 exactly.

**Inter-system communication (architecture.md:243):** ✓
- `TuningReloaded` event for the discrete reload signal — not a shared mutable resource.
- `TuningConfig` lives behind `Handle<TuningConfig>` in `Assets<TuningConfig>`; the `TuningHandle` resource is a thin wrapper for the handle, not the config itself. Reads always go through `Assets`.

**Component naming (architecture.md:322):** ✓ `SemanticAccent` already established by Story 2.2; reused unchanged.

**Plugin boundary table (architecture.md:654, 656):** ✓
- `VisualPlugin` boundary: owns `ToonMaterial` asset registration, palette, outline (Story 2.4). Consumes `SemanticAccent` component on rendered entities (now real with Task 7).
- `TuningPlugin` boundary: owns `TuningConfig` resource and hot-reload watcher. Emits `TuningReloaded` event.

**Anti-pattern check (architecture.md:458-468):**
- ❌ God-struct: `TuningConfig` is small (3 fields in 2.3, grows to ~10 by Story 4.x). Single-responsibility per field. ✓
- ❌ Direct cross-plugin state mutation: `apply_tuning_to_toon_materials` is a VisualPlugin-internal system reading TuningPlugin's event — not direct mutation. ✓
- ❌ Magic numbers: hardcoded WGSL light direction `(0.5, 1.0, 0.3)` is a M1-spike concession; comment in WGSL marks it for replacement at M2. Defaults in `TuningConfig::default()` mirror `tuning.ron` initial values; if those drift, the test in `config.rs` fails. ✓
- ❌ `unwrap()` / `expect()`: `TuningConfigLoader::load` returns `Result` per architecture.md:366; callers handle errors via the asset system. ✓
- ❌ Scattered `AssetServer::load`: only ONE call (in `load_tuning` Startup system in `tuning/mod.rs`). The shader is loaded by Bevy's `MaterialPlugin` internally via `Material::fragment_shader()`. ✓
- ❌ `.after(specific_function)` ordering: not used (SystemSet via `TuningSystems::Reload`). ✓

### LLM dev agent guardrails — most-likely-to-go-wrong patterns

These are the failure modes that are most likely to bite if the dev moves fast:

1. **WGSL @group(0) or @group(1) instead of @group(2).** Bevy reserves group 0 for view, 1 for mesh; custom materials MUST use group 2. A common copy-paste mistake from PBR tutorials is `@group(0) @binding(0) var<uniform> material: ...` — this silently overwrites view uniforms (camera matrix etc.) with material data, producing scrambled rendering. Use `@group(2) @binding(0)`.

2. **Field-order drift between Rust struct and WGSL struct.** The Rust `ToonMaterial { tint, steps, rim_power, rim_intensity }` and the WGSL `struct ToonMaterial { tint, steps, rim_power, rim_intensity }` MUST have fields in the same order. `vec4` first (16-byte alignment), scalars last. Reordering on either side produces gibberish colors at render time; no compile error, no runtime warning.

3. **`Color` instead of `LinearRgba` for the `tint` field.** Bevy's `Color` is a tagged enum with multiple color spaces; `AsBindGroup` cannot serialize it as a flat `vec4`. Use `LinearRgba` (a struct with `r, g, b, a: f32`). The `Color::into()` conversion (`color_for(...).into()`) produces `LinearRgba` correctly.

4. **`#[derive(Component)]` on `ToonMaterial`.** Materials are NOT components. They're inserted via `MeshMaterial3d(handle)` (Bevy 0.18 idiom), where `handle` is a `Handle<ToonMaterial>` from `Assets<ToonMaterial>`. `MeshMaterial3d` is the component; the material itself is an asset.

5. **Forgetting `Asset` + `TypePath` derives.** Bevy 0.18's asset system requires both. `Asset` macro emits the asset ID type; `TypePath` is for reflection and debugging. Missing either produces unfriendly compile errors.

6. **Loading shader manually via `AssetServer::load("shaders/toon.wgsl")`.** Don't. `MaterialPlugin` loads the shader internally based on `Material::fragment_shader()`'s returned `ShaderRef`. Manual `AssetServer::load` would create a duplicate handle and pin the shader unnecessarily.

7. **Single `extensions: &["ron"]`** without thinking about future `.ron` asset types. Today this is the only RON loader. When Story 4.7 adds string-table loading from `assets/strings/en.ron`, the loaders will collide. Track as deferred concern; **don't pre-build** the disambiguation now (YAGNI). The collision will manifest as Bevy's "multiple loaders for extension 'ron'" error at app init when 4.7 lands.

8. **`watch_for_changes_override: Some(true)` in release.** Release builds should NOT watch the file system. Use `cfg!(debug_assertions).then_some(true)` to gate the override. Hardcoded `Some(true)` works in dev but burns CPU + opens fs handles in shipped binaries.

9. **System ordering: registering `apply_tuning_to_toon_materials` outside `TuningSystems::Reload`.** The set membership IS the contract — future Story-4.x systems that depend on tuning being applied (e.g., enemy-AI reading `enemy_hp` from `TuningConfig`) will use `.after(TuningSystems::Reload)` for ordering. Putting the visual-update system OUT of the set creates an ordering hole.

10. **Reading the `.ron` synchronously via `std::fs::read` instead of through `AssetServer`.** Tempting for the cold-start case ("just load it once at Startup, no async needed"). But this bypasses the asset system, so subsequent edits to `tuning.ron` aren't seen. Always go through `AssetServer::load` + the AssetEvent reload path.

11. **AC #5 verification under-rigor.** "Number of bands matches the uniform value within ±1" needs visual inspection — the dev tends to glance at the screen and write "yes" without counting. Count the bands. Take screenshots at steps=3 and steps=8 specifically (Task 11).

12. **Forgetting Task 9 (cfg_attr removal in palette.rs).** The deferred-work entry from Story 2.2 explicitly anchors to this story. If Task 9 is skipped, palette.rs accumulates dead-code-allow noise. The Task-10 `nm color_for >= 1` check is the architectural enforcement — running release build symbol verification confirms the cfg_attr removal didn't reintroduce the warning.

### Camera / lighting strategy

Story 2.1 set up Camera3d at `order: -1` with 3 PointLights (key + fill + back). Story 2.2 added a Camera2d at `order: 1` for the swatch UI. Story 2.3 changes nothing about cameras or lights — only the **material** swap.

**Why hardcoded light direction in WGSL instead of reading the PointLights?** The PointLights use Bevy's PBR illumination model, which is bypassed by our custom toon shader. Reading PointLight uniforms from a custom Material requires importing Bevy's light bindings (`bevy_pbr::mesh_view_bindings`), iterating up to N lights, etc. — significant complexity for an M1 spike. The hardcoded directional light is sufficient to demonstrate posterized banding, and Story 4.x can replace it with a real DirectionalLight component once gameplay scenes need configurable lighting.

**Visual outcome with hardcoded light:** the asteroid's rim-light reads cleanly at the silhouette (because the camera is looking roughly down the +Z axis, and the light is upper-front-right, producing back-lighting that hits the asteroid edges). The ship-cockpit cuboid shows banding on its forward-facing faces. The projectile sphere shows banding on its top hemisphere.

### Hot-reload mechanics — the cold-start gotcha

Bevy's `AssetEvent` flow on first load:

1. `Startup`: `load_tuning` system inserts `Handle<TuningConfig>` via `AssetServer::load("config/tuning.ron")`.
2. The asset is now PENDING — `Assets<TuningConfig>::get(&handle.0)` returns `None`.
3. Bevy's asset loader (in a background task) reads the file, runs `TuningConfigLoader::load`, deserializes the RON.
4. On success, Bevy emits `AssetEvent::Added { id: <handle id> }`.
5. The asset is now AVAILABLE — `Assets::<TuningConfig>::get(&handle.0)` returns `Some(&TuningConfig{...})`.
6. Subsequent edits to `tuning.ron` cause `AssetEvent::Modified { id }`.

**Spawn-time cold-start race:** `spawn_reference_scene` runs `OnEnter(Loading)` — this is BEFORE step 4 typically (Loading state's 2-second splash gives the asset time to load, but not deterministically). When the spawn system creates `ToonMaterial` instances, it uses `ToonMaterial::default()` values (4, 2.0, 0.3). Once step 4 fires, `propagate_tuning_reload` emits `TuningReloaded`, and `apply_tuning_to_toon_materials` overwrites the defaults with the RON values.

**If RON values match Default exactly,** the cold-start path is invisible (defaults applied, then re-applied). If they ever drift (Story 4.x adds a new field with a different default), the cold-start has 1-2 frames of "default" rendering before the RON values take over. Acceptable — no AC mandates frame-1 correctness.

**Test for the cold-start path:** `tuning_config_default_matches_ron_initial_values` in `config.rs` — if the RON file's initial values drift from `Default`, this test fails immediately. Maintenance discipline: when editing the RON, also update `Default`.

### `tuning.ron` extensibility — Story 2.4 + 4.x forward compat

[Source: epics/epic-2-vector-aesthetic-tech-spike.md:108 (Story 2.4 outline_width / outline_color); architecture.md:358 (gameplay tunables list)]

Future stories will add fields to `TuningConfig`. The pattern:

```rust
#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct TuningConfig {
    // M1 (Story 2.3) — toon shader
    pub toon_steps: u32,
    pub toon_rim_power: f32,
    pub toon_rim_intensity: f32,
    // M1 (Story 2.4) — outline
    pub outline_width: f32,
    pub outline_color: Color, // or LinearRgba — TBD by 2.4 dev
    // M3+ (Stories 4.x, 5.x) — gameplay
    pub enemy_hp: f32,
    pub shot_cost: u32,
    pub yield_multiplier: f32,
    // ... grows
}
```

**Backward compat caveat:** adding a field to `TuningConfig` requires updating `tuning.ron` to include the new field, OR using `#[serde(default)]` on the field (in which case missing-from-RON triggers Rust's `Default` impl for that field). Story 2.3 uses neither pattern (the RON has all 3 fields explicitly). Story 2.4 should consider `#[serde(default)]` for added fields if it wants to keep `tuning.ron` editable without forcing every field to be specified.

### Integration test deferral

Architecture.md:354 defers integration tests post-M3. Story 2.3's runtime behavior (hot-reload propagation, Material rendering) is verified manually via `cargo run` + screenshot review (Task 10 + 11). When M3+ stories introduce integration tests, candidate cases for ToonMaterial:

- `App::new() + TuningPlugin + VisualPlugin + emit AssetEvent::Modified, observe TuningReloaded`
- `App::new() + spawn entity with MeshMaterial3d<ToonMaterial>, advance frame, observe material in Assets<ToonMaterial>`

These are not Story 2.3 scope.

### Future-story handoff hooks

- **Story 2.4 (outline integration):** will extend `TuningConfig` with `outline_width: f32` + `outline_color: Color`, edit `tuning.ron` accordingly, register `bevy_mod_outline`'s plugin in `VisualPlugin`. Outline color may consume `SemanticAccent` per-entity (subject to a global `outline_color` override toggle).
- **Story 2.5 (parity validation gate):** runs `cargo run --release` on Metal/Vulkan/DX12, captures 1080p screenshots from a fixed camera transform, generates `docs/tech-spike/m1-backends/parity-report.md`. **Hard dependency: 2.3's release build must work cleanly across all 3 backends.** Task 10's release-build verification is the local de-risk; cross-backend is 2.5's job.
- **Story 2.6 (go/fallback decision):** evaluates 2.5's parity-report, writes `docs/tech-spike/m1-decision.md` with `Decision: GO toon` or `FALLBACK flat+rim-light`. Story 2.3's quality directly drives this decision.
- **Story 4.5 (SemanticAccent wiring):** will attach `SemanticAccent` to gameplay entities (asteroids, salvage, enemies, projectiles) at spawn-time. Story 2.3's Task 7 already attaches it to reference-scene placeholders, validating the component-attach pattern. **A future system will need to listen for SemanticAccent component changes and update the corresponding `ToonMaterial::tint`** — that's 4.5 scope, not 2.3 (today the tint is set at spawn-time and never changes).

### Project Structure Notes

- **New module location:** `src/tuning/` (mod.rs + config.rs) matches architecture.md:555-557 exactly.
- **New module location:** `src/visual/toon_material.rs` matches architecture.md:605 exactly.
- **New asset directory:** `assets/` is created by this story (the `assets/` root does not yet exist in the project). Per architecture.md:331, asset directories are grouped by type: `assets/shaders/`, `assets/config/`, etc. **No** flat `assets/toon.wgsl` — keep the `shaders/` subdirectory from day 1.
- **`docs/tech-spike/m1-toon/`:** new subdirectory under `docs/tech-spike/`, sibling to Story 2.2's `m1-palette/`. The parent `docs/tech-spike/` was created by Story 2.2.

### References

- [Source: epics/epic-2-vector-aesthetic-tech-spike.md:60-94] — Story 2.3 user story + 5 ACs + epic context.
- [Source: prd.md:64] — "Vector aesthetic in 3D" rationale (Tron/Rez signature, scope-reducer, photo-mode marketing vehicle).
- [Source: prd.md:147] — "custom WGSL toon shader + outline, restrained palette with semantic accent colors".
- [Source: prd.md:367, 405] — wgpu abstraction; toon shader validated on Metal/Vulkan/DX12 at M1 tech-spike; fallback to flat+rim-light if M1 underwhelms.
- [Source: prd.md:569] — FR49: Game renders all 3D geometry using a toon-shading material with silhouette outlines.
- [Source: architecture.md:218-225] — Rendering & Visual Architecture; toon Material is M1 learning target + portfolio artifact.
- [Source: architecture.md:344-349] — Module / Plugin Organization rules.
- [Source: architecture.md:355-359] — Constants & Tuning conventions; tuning.ron + TuningConfig + hot-reload mandate.
- [Source: architecture.md:531-637] — Complete project directory structure (canonical layout).
- [Source: architecture.md:654, 656] — Plugin boundary table for VisualPlugin + TuningPlugin.
- [Source: architecture.md:710] — FR49 mapping to `src/visual/toon_material.rs` + `assets/shaders/toon.wgsl`.
- [Source: 2-1-visualplugin-skeleton-reference-scene.md] — Story 2.1 reference scene contract.
- [Source: 2-2-semanticaccent-palette-primitives.md] — Story 2.2 palette + SemanticAccent contract.
- [Source: deferred-work.md (post-2026-04-28)] — "Removal-on-graduation" entry tracking the cfg_attr-block removal that Task 9 lands.
- **External:** Bevy 0.18 docs — `bevy::pbr::Material` trait, `bevy::render::render_resource::AsBindGroup` derive, `bevy::asset::AssetLoader` trait, `bevy_pbr::forward_io::VertexOutput` shader import, `bevy_pbr::mesh_view_bindings::view` shader import.
- **External:** WGSL spec — alignment rules for uniform buffers (vec4 16-byte alignment, scalar trailing fields).

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (1M context)

### Debug Log References

| Verification step | Log path | Grep `warning:|error:` | Notes |
|---|---|---|---|
| `cargo check` | `/tmp/story-2-3-check.log` | **0** | Clean. |
| `cargo build` | `/tmp/story-2-3-build.log` | **0** | Clean. |
| `cargo test` | `/tmp/story-2-3-test.log` | **0** (`warning:|error:|FAILED` form) | **12 passed, 0 failed** (matches spec count: 9 prior + 2 tuning + 1 toon_material). |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-2-3-clippy.log` | **0** | Clean. |
| `cargo fmt --all -- --check` | n/a (exit-code) | exit 0 | Clean. |
| `cargo build --release` | `/tmp/story-2-3-release.log` | **0** (after re-applying cfg_attr — see Task 9 deviation) | Clean. |
| `nm target/release/asteroids3D | grep -c color_for` | n/a | **0** | Confirms Task 9 contingency: `color_for`/`SemanticAccent` are still cfg-elided in release because `mod reference_scene` is `#[cfg(debug_assertions)]`-gated; release consumer arrives in Story 4.5. |
| Debug-build runtime | `/tmp/story-2-3-run.log` | `warning:|error:` = **1** (the deferred-work cleanup-iteration race WARN — pre-existing, not a 2.3 regression) | `entered MainMenu` ≥ 1 ✓; no `wgpu error`, no panic; `TuningReloaded` events captured 5× (cold-start + 4 hot-edits). |
| Hot-reload runtime | same | n/a | Edits to `assets/config/tuning.ron` trigger `TuningReloaded` in ~1–2 s. Verified series: cold-start 4 → 8 → 3 → rim_intensity 0.7 → restore 4. AC #3 + AC #5 (system-level) ✓. |

### Completion Notes List

**AC coverage**

- **AC #1 (WGSL formula):** `assets/shaders/toon.wgsl` implements `floor(max(N·L, 0) * steps) / steps` posterization + `pow(1 - N·V, rim_power) * rim_intensity` rim, additively composed and tinted by the `tint: vec4<f32>` uniform. Single uniform buffer `ToonMaterial { tint, steps, rim_power, rim_intensity }` declared at `@group(#{MATERIAL_BIND_GROUP}) @binding(0)` (see deviation #1 below). ✓
- **AC #2 (Bevy `Material` integration):** `src/visual/toon_material.rs` defines `ToonMaterial` with `#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]`, `Material::fragment_shader()` returns `"shaders/toon.wgsl".into()`, `MaterialPlugin::<ToonMaterial>::default()` registered first in `VisualPlugin::build`. Field order matches the WGSL struct (vec4 first). ✓
- **AC #3 (`TuningConfig` hot-reload):** `src/tuning/{mod.rs, config.rs}` defines `TuningConfig` (Asset + Deserialize) loaded from `assets/config/tuning.ron`; `TuningPlugin` watches `AssetEvent<TuningConfig>` and emits `TuningReloaded`; `VisualPlugin::apply_tuning_to_toon_materials` updates all `ToonMaterial` uniforms in `TuningSystems::Reload`. `AssetPlugin.watch_for_changes_override = cfg!(debug_assertions).then_some(true)` in `main.rs`. **Required deviation:** the `file_watcher` Bevy feature was added to `Cargo.toml`'s Bevy feature lists (cross-platform + Linux-target) — without it, `watch_for_changes_override` is silently inert. Verified via 4 successive RON edits each producing a `TuningReloaded` log line (latency ~1–2 s). ✓
- **AC #4 (reference scene materialized + `SemanticAccent` attached):** `src/visual/reference_scene.rs` swapped to `Assets<ToonMaterial>` for all three placeholders. Tints + components: asteroid → `Hazard` (yellow `#F0E442`), ship → `PlayerOwned` (sky-blue `#56B4E9`), projectile → `Salvage` (bluish-green `#009E73`). Visual evidence (posterized banding + rim-light visibility) is anchored to the three PNGs in `docs/tech-spike/m1-toon/` (see Task 11 status below). ✓ (system-level), 🟡 (visual evidence — see Task 11)
- **AC #5 (band-count tracks `toon_steps` ±1):** Hot-reload edits drove the asteroid through `steps: 4 → 8 → 3 → (intensity bump) → 4`; events captured in `/tmp/story-2-3-run.log`. Visual band-count verification anchored to `toon-steps-3.png` and `toon-steps-8.png` (see Task 11). ✓ (system-level), 🟡 (visual evidence — see Task 11)

**Deviations from spec**

1. **WGSL bind-group index — `@group(2)` → `@group(#{MATERIAL_BIND_GROUP})`.** The spec prescribed hard-coded `@group(2)` based on a 2-group convention (view=0, mesh=1, material=2). Bevy 0.18 actually reserves `@group(2)` for mesh-related bindings (morph targets, skinning) and exposes the canonical material slot via the shader-def substitution `#{MATERIAL_BIND_GROUP}` (resolves to `3` per `bevy_pbr::material::MATERIAL_BIND_GROUP_INDEX`). Without this fix the app panicked at `opaque_mesh_pipeline` creation with `wgpu::ValidationError: Shader global ResourceBinding { group: 2, binding: 0 } is not available in the pipeline layout`. Substitution is the canonical pattern in Bevy 0.18 examples (`shader_material.rs`, `shader_material_bindless.rs`).
2. **Bevy 0.18 module/trait/event renames — `bevy::shader::ShaderRef`, `Message`/`MessageReader`/`MessageWriter`/`add_message`, `AssetLoader: TypePath`.** Spec's reference Rust skeletons used pre-0.18 paths (`bevy::render::render_resource::ShaderRef`, `Event`/`EventReader`/`EventWriter`/`add_event`, no `TypePath` on the loader). All adjusted to 0.18 idioms. `AssetEvent` itself unchanged.
3. **`Cargo.toml` (M) added `file_watcher` to Bevy features (both feature blocks).** Spec Task 12 said "**Cargo.toml** (??) — **untouched** unless Bevy's Asset/Material derives need a feature flag (verify via cargo build)". The empirical condition (silent hot-reload failure) triggered the exception clause; the addition is one feature per block (no new dep keys), `Cargo.lock` grew by 166 lines for the file-watcher crate tree (`notify`, `notify-debouncer-full`, `file-id`, `fsevent-sys`, etc.). Documented in `docs/tech-spike/m1-toon/notes.md`.
4. **Task 9 cfg_attr removal re-deferred to Story 4.5 per the spec's own contingency.** Initial removal produced 2 dead-code warnings on `cargo build --release` (`SemanticAccent` and `color_for` are still only consumed by `mod reference_scene`, which is `#[cfg(debug_assertions)]`-gated in `src/visual/mod.rs:53`). Per Task 9's contingency: "If the warnings DO reappear (meaning Task 7's wiring isn't actually a release consumer), DO NOT re-add the cfg_attr — instead, root-cause [...] In that case, the cfg_attr removal stays deferred to Story 4.5". Root-cause confirmed (debug-only consumer). cfg_attr blocks restored with updated `reason = "..."` strings pointing at Story 4.5; `deferred-work.md` Story-2.2 "Removal-on-graduation" entry amended below to record the deferral. `nm target/release/asteroids3D | grep -c color_for` = 0 (matches the 2.2 isolation behavior, NOT the spec's hopeful "≥1 from Story 2.3 onward").
5. **Tracing `info!` added in `propagate_tuning_reload`** — emits `TuningReloaded: toon_steps=N rim_power=N rim_intensity=N` when an asset event arrives. Provides a greppable signal for Task 10's hot-reload verification (otherwise the propagator is silent and verification would require an integration test that architecture.md:354 defers post-M3).

**Task status snapshot (also reflected in checkboxes above)**

- Tasks 1–8, 10, 12: **complete** with logs.
- Task 9: **complete with re-deferral** — see Deviation #4.
- Task 11: **partial** — `notes.md` written; the three PNGs (`toon-baseline.png`, `toon-steps-8.png`, `toon-steps-3.png`) require the developer to capture them via `Cmd-Shift-4` while `cargo run` is foregrounded (see Task 11 sub-bullets in this story for the recipe). System-level evidence for AC #4/#5 is captured in the run log and notes.md.
- Task 13: pending — awaiting user approval to push (Commits 1 + 2 ready to draft once screenshots land).
- Task 14: in progress (this Dev Agent Record).

### File List

**New (added)**

- `Cargo.lock` — regenerated (file-watcher crate tree, +166 lines)
- `assets/shaders/toon.wgsl`
- `assets/config/tuning.ron`
- `src/tuning/mod.rs`
- `src/tuning/config.rs`
- `src/visual/toon_material.rs`
- `docs/tech-spike/m1-toon/notes.md`
- `docs/tech-spike/m1-toon/toon-baseline.png` *(pending Task 11 capture)*
- `docs/tech-spike/m1-toon/toon-steps-8.png` *(pending Task 11 capture)*
- `docs/tech-spike/m1-toon/toon-steps-3.png` *(pending Task 11 capture)*

**Modified**

- `Cargo.toml` — added `"file_watcher"` to both Bevy feature lists (cross-platform + Linux-target)
- `src/main.rs` — `mod tuning;`, register `TuningPlugin`, `AssetPlugin.watch_for_changes_override`
- `src/visual/mod.rs` — `pub mod toon_material;`, `MaterialPlugin::<ToonMaterial>` registration, `apply_tuning_to_toon_materials` system
- `src/visual/reference_scene.rs` — switched to `Assets<ToonMaterial>`, attached `SemanticAccent` per placeholder
- `src/visual/palette.rs` — `cfg_attr` `reason = ...` strings updated to point at Story 4.5 (per Task 9 contingency)

**Bookkeeping (modified at handoff)**

- `_bmad-output/implementation-artifacts/2-3-wgsl-toon-material-implementation.md` — this file (status, Dev Agent Record, checkboxes)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — story status flips
- `_bmad-output/implementation-artifacts/deferred-work.md` — Story-2.2 "Removal-on-graduation" entry amended with Story-4.5 re-deferral

### Change Log

| Date | Change | Reason |
|---|---|---|
| 2026-04-28 | WGSL `ToonMaterial` + `TuningConfig` hot-reload + `SemanticAccent` tinting wired into reference scene (Story 2.3) | Implementation per AC #1–#5 |
| 2026-04-28 | `file_watcher` Bevy feature added to `Cargo.toml` | `AssetPlugin::watch_for_changes_override` is inert without it (Bevy 0.18); required by AC #3 |
| 2026-04-28 | WGSL bind group changed to `@group(#{MATERIAL_BIND_GROUP})` | Bevy 0.18 reserves `@group(2)` for mesh bindings; canonical pattern uses shader-def substitution |
| 2026-04-28 | Story-2.2 `cfg_attr(not(debug_assertions), allow(dead_code))` blocks on `palette.rs` retained (re-deferred) | Reference scene is still cfg(debug_assertions)-gated → release-build still has no `color_for` consumer; resolution moves to Story 4.5 |
