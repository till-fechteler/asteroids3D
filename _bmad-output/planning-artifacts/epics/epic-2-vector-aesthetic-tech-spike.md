# Epic 2: Vector Aesthetic Tech Spike

Custom WGSL Toon `Material` + `bevy_mod_outline` render identically on Metal (macOS), Vulkan (Linux), and DX12 (Windows). M1 go/fallback decision documented. Portfolio-quality shader artifact authored by Till. M-alignment: M1. FRs covered: FR49, FR50.

## Story 2.1: VisualPlugin Skeleton + Reference Scene

As a developer,
I want a `VisualPlugin` module and a committed dev-only reference scene (asteroid + ship-cockpit + projectile placeholders with 3-point lighting),
So that Stories 2.2–2.5 have a stable, reproducible stage to validate shaders and outlines against across all three GPU backends.

**Acceptance Criteria:**

**Given** the project from Epic 1
**When** `src/visual/mod.rs` is authored with a `VisualPlugin: Plugin` struct and added via `App::add_plugins(VisualPlugin)` in `main.rs`
**Then** the plugin declares a `VisualSystems` SystemSet enum per the architecture naming convention
**And** the plugin builds on all three platforms

**Given** a reference-scene module behind `cfg(debug_assertions)`
**When** it runs on `OnEnter(GameState::Loading)`
**Then** the scene contains exactly three placeholder meshes: an icosphere asteroid, a cuboid ship-cockpit placeholder, a small sphere projectile
**And** three `PointLight` entities form a 3-point lighting setup (key, fill, back)
**And** every spawned entity carries a `ReferenceSceneEntity` marker component

**Given** the reference scene is in the scene graph
**When** `cargo run` is invoked in a debug build
**Then** all three placeholders render using Bevy's default `StandardMaterial` (toon comes in Story 2.3)
**And** all three placeholders are inside the camera frustum

**Given** a release build (`cargo build --release`)
**When** the binary is inspected for the symbol `ReferenceSceneEntity`
**Then** the symbol is absent (reference scene compiled out of release)

## Story 2.2: SemanticAccent Palette Primitives

As a developer,
I want a `SemanticAccent` enum with a color-lookup function and a committed visual distinguishability reference under three color-blindness simulations,
So that FR50 semantic accent colors rest on a tested NFR-A1 foundation before any shader consumes them.

**Acceptance Criteria:**

**Given** `src/visual/palette.rs` is authored
**When** it defines `SemanticAccent` as an enum with variants `Enemy`, `Salvage`, `Hazard`, `PlayerOwned`, `Neutral`
**Then** each variant has a specified `Color` with its hex value documented as a comment
**And** `pub fn color_for(accent: SemanticAccent) -> Color` returns the mapped color

**Given** a dev-only visualization scene (extension of Story 2.1's reference scene or a standalone example)
**When** the 5 accent colors are rendered as labeled swatches side-by-side
**Then** screenshots are captured under: (a) normal vision, (b) protanopia simulation, (c) deuteranopia simulation, (d) tritanopia simulation
**And** all 4 screenshots are committed to `docs/tech-spike/m1-palette/`

**Given** the simulated-vision screenshots
**When** visually inspected
**Then** every accent color remains distinguishable from every other accent color under all three simulations
**And** failing pairs (if any) are documented in `docs/tech-spike/m1-palette/review-notes.md` with a proposed color adjustment

**Given** the `SemanticAccent` enum
**When** later stories need per-entity accent tagging
**Then** they may attach `SemanticAccent` as a component so shaders and outlines can read it without entity-level hardcoding

## Story 2.3: WGSL Toon Material Implementation

As the primary shader author,
I want a hand-written WGSL `ToonMaterial` implementing N·L posterization with configurable step count, rim-light term, and `SemanticAccent` tinting,
So that FR49 toon-shading ships as a portfolio-quality self-authored artifact — the primary M1 learning target.

**Acceptance Criteria:**

**Given** `assets/shaders/toon.wgsl` is authored by hand
**When** its fragment shader is reviewed
**Then** shading is computed as `floor(max(dot(N,L), 0.0) * steps) / steps` posterization
**And** a rim-light term `pow(1.0 - dot(N,V), rim_power) * rim_intensity` is additive to the posterized base
**And** a `tint: vec4<f32>` uniform multiplies the final color
**And** uniforms `steps: u32`, `rim_power: f32`, `rim_intensity: f32`, `tint: vec4<f32>` are declared in a single uniform buffer

**Given** `src/visual/toon_material.rs` is authored
**When** it defines `ToonMaterial` implementing Bevy's `Material` trait
**Then** `fragment_shader()` returns a handle to `assets/shaders/toon.wgsl`
**And** `AsBindGroup` is derived and matches the WGSL uniform layout
**And** `MaterialPlugin::<ToonMaterial>::default()` is registered inside `VisualPlugin`

**Given** `src/tuning.rs` defines a `TuningConfig` resource loaded from `assets/config/tuning.ron` with fields `toon_steps: u32`, `toon_rim_power: f32`, `toon_rim_intensity: f32`
**When** `tuning.ron` is edited during `cargo run` (dev hot-reload enabled via `AssetPlugin::watch_for_changes_override`)
**Then** `ToonMaterial` uniforms update live in the reference scene without restart

**Given** the reference scene's three placeholders
**When** they are re-materialized with `ToonMaterial` instead of `StandardMaterial`
**Then** each placeholder shows visible posterized banding
**And** the rim-light term is visible at grazing angles on the asteroid silhouette
**And** entities carrying a `SemanticAccent` component render with the corresponding `tint`

**Given** `toon_steps` is set to 3, then 5, then 8 via hot-reload
**When** each value is observed on the asteroid
**Then** the number of visible shading bands matches the uniform value within ±1 band (anti-aliasing tolerance)

## Story 2.4: bevy_mod_outline Integration + Wiring

As a developer,
I want `bevy_mod_outline` integrated with `OutlineBundle` attached to every toon-shaded mesh in the reference scene, width and color tunable via `TuningConfig`,
So that FR49 silhouette outlines render consistently without per-entity hardcoding.

**Acceptance Criteria:**

**Given** `bevy_mod_outline`'s plugin is added to `VisualPlugin`
**When** the app starts
**Then** the plugin's systems are scheduled per its documented requirements
**And** the app still launches on all three platforms

**Given** `TuningConfig` is extended with `outline_width: f32` and `outline_color: Color` fields loaded from `assets/config/tuning.ron`
**When** the reference scene spawns its three placeholders
**Then** each placeholder is spawned with an `OutlineBundle` whose `width` and `color` read from `TuningConfig`

**Given** outlines are applied
**When** the reference scene renders in a debug build
**Then** the asteroid, ship, and projectile each show a continuous silhouette outline visible against any background
**And** outlines do not z-fight with mesh surfaces at the default camera distance

**Given** `assets/config/tuning.ron` is edited at runtime
**When** `outline_width` changes from 2.0 to 4.0
**Then** the running reference scene updates to thicker outlines without restart

## Story 2.5: Three-Backend Parity Validation Gate

As the project author,
I want the reference scene (toon + outlines) rendered on Metal, Vulkan, and DX12 with committed 1080p screenshots and a pairwise-diff report,
So that M1's completion criterion has objective, reviewable evidence and any backend divergence is documented before M2.

**Acceptance Criteria:**

**Given** the reference scene is complete from Stories 2.3 and 2.4
**When** `cargo run --release` is executed on macOS (Apple Silicon, Metal)
**Then** a screenshot is captured at a fixed deterministic camera transform (hardcoded `Transform` in a capture-mode code path)
**And** scene time is frozen at `t=0` (no animation) to ensure reproducibility
**And** the 1920×1080 PNG is committed to `docs/tech-spike/m1-backends/metal.png`

**Given** the same reference scene
**When** `cargo run --release` is executed on Linux with `WGPU_BACKEND=vulkan`
**Then** a 1920×1080 PNG at the same camera transform is committed to `docs/tech-spike/m1-backends/vulkan.png`

**Given** the same reference scene
**When** `cargo run --release` is executed on Windows with `WGPU_BACKEND=dx12`
**Then** a 1920×1080 PNG at the same camera transform is committed to `docs/tech-spike/m1-backends/dx12.png`

**Given** all three screenshots exist
**When** they are compared (ImageMagick `compare`, Beyond Compare, or manual overlay)
**Then** `docs/tech-spike/m1-backends/parity-report.md` documents each pairwise diff summary
**And** any >5% pixel divergence is annotated with a root-cause hypothesis
**And** the report closes with a go / no-go recommendation for Story 2.6

## Story 2.6: Go/Fallback Decision Document

As the project author,
I want a committed decision whether to proceed with the custom toon shader or fall back to flat + rim-light,
So that M1 closes with explicit scope resolution for M2 and the rationale is auditable later.

**Acceptance Criteria:**

**Given** the parity report from Story 2.5
**When** `docs/tech-spike/m1-decision.md` is authored
**Then** it contains sections: `Decision`, `Rationale`, `Risks Accepted`, `Fallback Trigger Criteria`, `M2 Impact`
**And** `Decision` is exactly one of: `GO toon`, `GO toon with scope reduction`, `FALLBACK flat+rim-light`

**Given** the decision is `GO toon` or `GO toon with scope reduction`
**When** M1 is declared complete
**Then** Story 2.7 is marked `Not Needed`
**And** `ToonMaterial` is confirmed as the M2 production shader

**Given** the decision is `FALLBACK flat+rim-light`
**When** M1 is declared complete
**Then** Story 2.7 is unblocked
**And** `ToonMaterial` is scheduled for removal or deprecation in Story 2.7

## Story 2.7: Fallback Material Scaffold (Conditional on Story 2.6)

As a developer,
I want a flat-shaded + rim-light fallback material scaffolded only if Story 2.6's decision is `FALLBACK flat+rim-light`,
So that M1 closes with a viable aesthetic path even when custom WGSL proves untenable across backends.

**Acceptance Criteria:**

**Given** Story 2.6's decision is `GO toon` or `GO toon with scope reduction`
**When** this story is reviewed at M1 closeout
**Then** this story is marked `Not Needed` and skipped
**And** no code changes are made

**Given** Story 2.6's decision is `FALLBACK flat+rim-light`
**When** this story is executed
**Then** `src/visual/flat_rim_material.rs` is authored with a Bevy `Material` impl using a `StandardMaterial`-compatible flat base plus a minimal rim-light fragment term
**And** `ToonMaterial` is either deleted or retained with a `#[deprecated]` attribute pointing at the fallback material
**And** the reference scene re-materializes its placeholders with the fallback material

**Given** the fallback material is applied
**When** the reference scene is re-rendered on all three backends
**Then** parity screenshots are captured to `docs/tech-spike/m1-backends-fallback/{metal,vulkan,dx12}.png`
**And** `docs/tech-spike/m1-backends/parity-report.md` is appended with a fallback-parity section

<!-- Epic 2 complete — 7 stories (incl. 1 conditional) cover M1 go/fallback gate. Next epic to decompose: Epic 3 (Arena Flight & First Combat / M2). -->
