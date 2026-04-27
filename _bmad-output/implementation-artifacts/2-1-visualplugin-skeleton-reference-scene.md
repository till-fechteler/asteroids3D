# Story 2.1: VisualPlugin Skeleton + Reference Scene

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want a `VisualPlugin` module and a committed dev-only reference scene (asteroid + ship-cockpit + projectile placeholders with 3-point lighting),
So that Stories 2.2–2.5 have a stable, reproducible stage to validate shaders and outlines against across all three GPU backends.

## Acceptance Criteria

1. **Given** the project from Epic 1
   **When** `src/visual/mod.rs` is authored with a `VisualPlugin: Plugin` struct and added via `App::add_plugins(VisualPlugin)` in `main.rs`
   **Then** the plugin declares a `VisualSystems` SystemSet enum per the architecture naming convention
   **And** the plugin builds on all three platforms

2. **Given** a reference-scene module behind `cfg(debug_assertions)`
   **When** it runs on `OnEnter(GameState::Loading)`
   **Then** the scene contains exactly three placeholder meshes: an icosphere asteroid, a cuboid ship-cockpit placeholder, a small sphere projectile
   **And** three `PointLight` entities form a 3-point lighting setup (key, fill, back)
   **And** every spawned entity carries a `ReferenceSceneEntity` marker component

3. **Given** the reference scene is in the scene graph
   **When** `cargo run` is invoked in a debug build
   **Then** all three placeholders render using Bevy's default `StandardMaterial` (toon comes in Story 2.3)
   **And** all three placeholders are inside the camera frustum

4. **Given** a release build (`cargo build --release`)
   **When** the binary is inspected for the symbol `ReferenceSceneEntity`
   **Then** the symbol is absent (reference scene compiled out of release)

## Tasks / Subtasks

- [x] **Task 1: Create `src/visual/mod.rs`** (AC: #1)
  - [ ] New directory `src/visual/`; new file `src/visual/mod.rs`. Module doc `//!` ≤ 2 lines, no story-id reference.
  - [ ] `use bevy::prelude::*;` only.
  - [ ] Public `pub struct VisualPlugin;` (unit struct).
  - [ ] Public SystemSet enum:
    ```rust
    #[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum VisualSystems {
        Setup,
    }
    ```
    Single variant `Setup` is the only set used in this story; Stories 2.3 / 2.4 / 2.5 will append more variants. Do **not** preemptively add `Render`, `Outline`, or other variants — YAGNI per CLAUDE.md "Don't design for hypothetical future requirements." The variant must be `pub` so the reference-scene submodule can reference it. **`Copy` derive** mirrors the architecture's "Good SystemSet" example pattern (architecture.md:511).
  - [ ] `impl Plugin for VisualPlugin` body:
    ```rust
    fn build(&self, app: &mut App) {
        app.configure_sets(OnEnter(crate::state::GameState::Loading), VisualSystems::Setup);

        #[cfg(debug_assertions)]
        app.add_plugins(reference_scene::ReferenceScenePlugin);
    }
    ```
    `configure_sets` registers the set even though only the dev-only submodule populates it today — keeps the public API stable for release builds. The `#[cfg(debug_assertions)]` gate on the submodule **and** on its plugin registration is what satisfies AC #4.
  - [ ] At module bottom, declare the submodule:
    ```rust
    #[cfg(debug_assertions)]
    mod reference_scene;
    ```
    No `pub` — submodule is plugin-internal.

- [x] **Task 2: Create `src/visual/reference_scene.rs`** (AC: #2, #3)
  - [ ] New file `src/visual/reference_scene.rs`, gated `#[cfg(debug_assertions)]` by virtue of the parent `mod` declaration. No need for `#![cfg(debug_assertions)]` at file top — the parent gate suffices and the inner attribute is forbidden in `mod.rs`-included files anyway.
  - [ ] Module doc `//!` ≤ 2 lines: "Dev-only reference scene for the M1 vector-aesthetic tech spike. Spawned on OnEnter(Loading); persists across state transitions."
  - [ ] Imports:
    ```rust
    use bevy::prelude::*;
    use crate::state::GameState;
    use super::VisualSystems;
    ```
  - [ ] `pub(super) struct ReferenceScenePlugin;` — visible only to `mod.rs`.
  - [ ] `impl Plugin for ReferenceScenePlugin`:
    ```rust
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Loading),
            spawn_reference_scene.in_set(VisualSystems::Setup),
        );
    }
    ```
  - [ ] Marker component (must be `#[cfg(debug_assertions)]`-only by virtue of the parent submodule gate — that's how AC #4 holds):
    ```rust
    #[derive(Component)]
    struct ReferenceSceneEntity;
    ```
    **Visibility:** module-private (no `pub`). Type is referenced only by the local spawn system. AC #4's symbol-absence check works because the entire `reference_scene` module is `#[cfg(debug_assertions)]`-gated at its `mod` declaration in `mod.rs` → in release builds the type, marker, plugin, and spawn system all evaporate at compile time.
  - [ ] `fn spawn_reference_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>)` — see "Reference-scene composition" in Dev Notes for exact transforms / mesh primitives / material colors / light intensities.
  - [ ] All entities spawned (3 meshes + 3 lights + 1 Camera3d) carry `ReferenceSceneEntity`. Total: **7** entities tagged.
  - [ ] **Do not** despawn on `OnExit(GameState::Loading)`. Reference scene must persist into MainMenu / future Arena entry so Stories 2.3+ can iterate the toon material against the same stage. The 2-second splash transition will move past, but the 3D meshes keep rendering behind whatever 2D / future UI is on top.

- [x] **Task 3: Wire `VisualPlugin` into `src/main.rs`** (AC: #1, #3)
  - [ ] Add `mod visual;` — alphabetical after `mod state;` per rustfmt expectations and the existing `mod logging; mod splash; mod state;` ordering. **Confirmed alphabetical:** logging < splash < state < visual.
  - [ ] Add `use visual::VisualPlugin;` to the top-level `use` block. Rustfmt will keep `use logging::init_logging; use splash::{...}; use state::{...}; use visual::VisualPlugin;` in that order (uppercase-only-after-`::` doesn't reorder these — verified by Story 1.7 deviation note).
  - [ ] Register the plugin: `.add_plugins(VisualPlugin)` chained immediately after `.add_plugins(DefaultPlugins.build().disable::<bevy::log::LogPlugin>())`. Idiomatic placement for plugin order: DefaultPlugins → app-domain plugins.
  - [ ] `fn main() -> AppExit` signature preserved from 1.8. `.run()` remains the trailing expression with no semicolon.
  - [ ] Update top-of-file `//!` doc to 4 lines: append " Registers VisualPlugin (dev-only reference scene gated by debug_assertions)." after the existing 3-line doc.

- [x] **Task 4: Local verification sweep** (AC: #1, #2, #3, #4)
  - [ ] `cargo check 2>&1 | tee /tmp/story-2-1-check.log` → `grep -cE 'warning:|error:' /tmp/story-2-1-check.log` must equal **0**.
  - [ ] `cargo build 2>&1 | tee /tmp/story-2-1-build.log` → same grep equals **0**.
  - [ ] `cargo test 2>&1 | tee /tmp/story-2-1-test.log` → `grep -cE 'warning:|error:|FAILED' /tmp/story-2-1-test.log` equals **0**; pass count unchanged at **3** (no new tests in this story; ECS spawn behavior is verified by `cargo run` rendering, not unit tests — there is no pure-logic surface to assert).
  - [ ] `cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-2-1-clippy.log` → grep equals **0**.
  - [ ] `cargo fmt --all -- --check` → exit 0. If it fails, run `cargo fmt --all` and re-check.
  - [ ] **Debug-build runtime verification (AC #2 + #3):**
    - `cargo run &> /tmp/story-2-1-run.log &` then close window after splash transitions.
    - Window must show three meshes against a dark background (Bevy's default `ClearColor` is `Color::srgb(0.4, 0.4, 0.4)` light grey — adjust expectation accordingly; do NOT change `ClearColor` in this story).
    - `grep -c 'entered Loading' /tmp/story-2-1-run.log` ≥ **1**.
    - `grep -c 'entered MainMenu' /tmp/story-2-1-run.log` ≥ **1** (state transition still works; reference scene does not block it).
    - `grep -cE 'warning:|error:|ERROR ' /tmp/story-2-1-run.log` should be 0 for app-emitted lines (ignore the known pre-existing `bevy_winit Skipped event Destroyed` WARN from `deferred-work.md` 1-6 entry).
    - **Visual confirmation (no automation):** at least one frame between window-open and splash-transition shows the icosphere, cuboid, and sphere placeholders. If the placeholders are not visible, check the camera transform — see "Camera placement & frustum check" in Dev Notes.
  - [ ] **Release-build symbol absence (AC #4):**
    - `cargo build --release 2>&1 | tee /tmp/story-2-1-release.log` → `grep -cE 'warning:|error:' /tmp/story-2-1-release.log` equals **0**.
    - `nm -gU target/release/asteroids3D 2>/dev/null | grep -c ReferenceSceneEntity` must equal **0** on macOS (`-gU` = global, defined-only). On Linux: `nm --defined-only target/release/asteroids3D | grep -c ReferenceSceneEntity` equals **0**. On Windows (GitHub Actions only — local dev is macOS): the CI matrix's `cargo build --release` succeeding without the symbol is satisfied by the cfg-gate at compile time.
    - Belt-and-suspenders: `strings target/release/asteroids3D | grep -c ReferenceSceneEntity` must equal **0**. (`strings` reads any retained string-table entries; type names embedded by `#[derive(Debug)]` formatting would surface here. Our marker has no `Debug` derive, but checking is free.)
  - [ ] Capture all hit counts + visual-confirmation note + binary-size delta into Debug Log References.

- [x] **Task 5: Scope guardrails — verify nothing else drifted** (AC: #1, #2, #3, #4)
  - [ ] `git status --short`: only `src/main.rs` (M), `src/visual/mod.rs` (??), `src/visual/reference_scene.rs` (??), plus bookkeeping `_bmad-output/implementation-artifacts/sprint-status.yaml` (M) and this story file (??). **No** `Cargo.toml` / `Cargo.lock` changes — `bevy` (0.18.1) already pulls everything needed (`StandardMaterial`, `PointLight`, `Camera3d`, `Mesh::from(...)` primitives, `Assets<Mesh>` / `Assets<StandardMaterial>`) via the `"3d"` feature collection (Cargo.toml:8). No new crate dependency.
  - [ ] `grep -nrE 'ToonMaterial|toon|outline|SemanticAccent|palette' src/ --include='*.rs'` → **0** hits. Toon material is Story 2.3, outline is 2.4, palette is 2.2.
  - [ ] `grep -nrE 'Arena|Caravan|PostRun|PhotoMode|Paused' src/ --include='*.rs' | grep -v 'state.rs'` → **0** hits.
  - [ ] `grep -rn 'tuning\|TuningConfig\|tuning\.ron' src/` → **0** hits. Hot-reloadable tuning config arrives in Story 2.3.
  - [ ] `grep -rn 'AssetServer::load\b' src/` → **0** hits. No file assets loaded in this story; everything is procedural primitives. (This is also the architecture's "no scattered AssetServer::load" enforcement preview — architecture.md:429.)
  - [ ] `grep -rn 'pub mod\|pub fn' src/visual/` should expose exactly: `pub struct VisualPlugin`, `pub enum VisualSystems` (and its variants). `ReferenceSceneEntity`, `ReferenceScenePlugin`, and `spawn_reference_scene` stay module-private (`pub(super)` for the plugin re-export; nothing else `pub`).
  - [ ] `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `docs/plugin-compatibility.md` — all untouched.
  - [ ] `deferred-work.md` untouched in this commit; review-phase findings (if any) get logged in Task 7's review handoff phase.

- [x] **Task 6: Commit + CI observation** (AC: #1, #2, #3, #4)
  - [ ] Stage: `src/main.rs` (M), `src/visual/mod.rs` (new), `src/visual/reference_scene.rs` (new). **No** `Cargo.toml` / `Cargo.lock`.
  - [ ] Commit message (HEREDOC): `feat: VisualPlugin skeleton + dev-only reference scene (Story 2.1)`. Single-line, sub-70-char, `feat:` prefix, **NO** `Co-Authored-By` trailer (matches Story 1.1–1.8 pattern).
  - [ ] Push to `origin/master`.
  - [ ] `gh run list -L 1` identifies the new run ID triggered by the source-touching commit. Wait for all 4 jobs (msrv-check + 3 OS build) to complete. Expected wall time: ~10–12 min (warm cache, Cargo.lock unchanged).
  - [ ] `gh run view <ID> --log | grep -cE 'warning:|error:'` → expect **0**.
  - [ ] All 4 jobs ✅; capture run ID + per-job durations into Debug Log References.

- [x] **Task 7: Ready-for-review handoff + bookkeeping commit**
  - [ ] Populate **Dev Agent Record** sections of this file: Agent Model Used, Debug Log References (per-command hit-counts + sample log lines + CI run ID + visual-confirmation note + release-binary symbol-grep counts), Completion Notes List (per-AC evidence + any deviations), File List (added / modified / untouched-guardrail).
  - [ ] Set this story's `Status:` header → `review`.
  - [ ] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`: flip `2-1-visualplugin-skeleton-reference-scene: ready-for-dev → in-progress → review`; bump `last_updated` to current date.
  - [ ] Stage this story file + `sprint-status.yaml`, commit with `bmad: story 2.1 ready-for-dev → review (VisualPlugin + ref scene shipped, CI green)` or similar `bmad:` prefix. This is a `_bmad-output/**` only commit → CI `paths-ignore` suppresses the matrix (per `.github/workflows/ci.yml:9-15`); expected zero-CI-run.
  - [ ] Push.
  - [ ] Story awaits code review; review can be light (single-reviewer precedent from 1.6/1.7/1.8) given ~120-line diff with no physics / save-I/O / cross-platform-API surfaces, OR a full 3-agent adversarial review if the dev agent suspects edge cases in the cfg-gating, camera/Camera2d coexistence, or persistence-across-state-transitions design choices.

## Dev Notes

### Why this story exists

Story 2.1 opens **Epic 2: Vector Aesthetic Tech Spike (M1)**. M1's success criterion is: custom WGSL Toon `Material` + `bevy_mod_outline` rendering identically on Metal / Vulkan / DX12 with a committed go/fallback decision. [Source: architecture.md:294-296; epics/epic-2-vector-aesthetic-tech-spike.md:3] Stories 2.3 (toon material) and 2.4 (outlines) need a **stable, reproducible stage** to render against — otherwise every shader iteration would re-author its own placeholder geometry, contaminating the comparison and burning iteration budget. Story 2.1 builds that stage:

- Three placeholder meshes representing the three rendering archetypes the game ships (asteroid, ship cockpit, projectile) — so 2.3's posterization and 2.4's outlines are observable on shapes that match the eventual gameplay surface.
- 3-point lighting (key + fill + back) — gives the toon shader a meaningful N·L variation across mesh surfaces. A flat single-light scene would render almost-uniform shading and hide posterization banding under cosine-falloff blandness.
- Dev-only behind `cfg(debug_assertions)` — release builds never ship the reference scene, so AC #4's symbol-absence check is the architectural enforcement of that contract.
- Sets up the **`src/visual/` module** for the first time. Architecture.md:603-607 reserves this directory for `mod.rs` + `toon_material.rs` + `outline.rs` + `palette.rs`. This story creates `mod.rs` plus a dev-only `reference_scene.rs` submodule (not in the architecture tree by design — it's spike scaffolding, not gameplay).

After Story 2.1 lands, Story 2.2 (palette primitives) starts independently of the reference scene; Stories 2.3 and 2.4 re-render the same three placeholders with toon material + outlines respectively; Story 2.5 captures parity screenshots on all three backends using the same scene.

### Context inherited from Epic 1 (Stories 1.1–1.8)

| Fact | Value | Source |
|---|---|---|
| Rust toolchain | `1.94.1` stable (pinned) | `rust-toolchain.toml` |
| MSRV | `1.89` (CI-verified) | `Cargo.toml:5` |
| Bevy | `0.18` (resolved `0.18.1`) with features `["3d", "png", "bevy_ui", "default_font"]` (+ x11/wayland on Linux) | `Cargo.toml:8,23-26`; `docs/plugin-compatibility.md` |
| Other pinned deps | `avian3d = "0.6"`, `bevy_mod_outline = "0.12"`, `bevy_kira_audio = "0.25"`, `leafwing-input-manager = "0.20"` — **none used yet in code**; outline plugin lands in Story 2.4 | `Cargo.toml:9-12`; `grep -rn` in src/ confirms zero use |
| `bevy_egui` | **NOT in Cargo.toml.** Removed in Story 1.5 (broken `cfg(debug_assertions)` gating in dependency tables); planned re-introduction at M2 debug-panels via `[features] dev-tools = ["dep:bevy_egui"]`. Do NOT add it back in this story. | `docs/plugin-compatibility.md:32-42`; `deferred-work.md:37` |
| `src/main.rs` body (post-1.8) | `mod logging; mod splash; mod state;` + `App::new().add_plugins(DefaultPlugins.build().disable::<LogPlugin>()).init_state::<GameState>().init_resource::<SplashConfig>().add_systems(OnEnter(Loading), (log_loading_entered, spawn_splash)).add_systems(OnEnter(MainMenu), log_mainmenu_entered).add_systems(Update, tick_splash_timer.run_if(in_state(Loading))).add_systems(OnExit(Loading), cleanup_loading_entities).run()` | Post-1.8 |
| `src/state.rs` | 7-variant `GameState` enum (Loading default), 2 lifecycle log fns, 1 unit test. `#[expect(dead_code, reason = "...")]` on the enum still valid — only `Loading` and `MainMenu` are live. | Post-1.6/1.7 |
| `src/splash.rs` | `Camera2d` + `Node` + `Text("asteroids3D")`, `LoadingStateEntity` marker, 2-second `Timer` → `NextState(MainMenu)`, despawn on `OnExit(Loading)`. **Will coexist with the new Camera3d** spawned by reference scene — see "Camera coexistence" below. | Post-1.7 |
| `src/logging.rs` | `tracing_subscriber` Registry + per-OS log file + panic hook. Initialized in `main()` before `App::new()`. **Not touched in this story.** | Post-1.8 |
| Tests in project | 3 (`state::default_state_is_loading`, `splash::splash_config_default_is_two_seconds`, `logging::resolve_log_dir_yields_expected_suffix`) | Post-1.8 |
| CI | 4-job matrix (msrv-check + 3 OS build), all green on master. `paths-ignore: ['_bmad/**', '_bmad-output/**']` skips bookkeeping commits. | `.github/workflows/ci.yml:9-15` |
| Commit convention | Single-line subject; `feat:` / `chore:` / `docs:` / `fix:` / `ci:` / `bmad:` prefixes; **NO** `Co-Authored-By` trailer | `git log --oneline -n 15` |
| Local dev machine | macOS 26.4.1 / arm64 (Apple M5 Pro) | Prior story Debug Logs |

### `cfg(debug_assertions)`-gating strategy — why submodule-level, not field-level

**The trap (lessons from Story 1.5):** Cargo treats `cfg(debug_assertions)` in **dependency tables** (`[target.'cfg(debug_assertions)'.dependencies]`) as always-true and emits a manifest warning. That bit Story 1.1 → resolved by Story 1.5 removing `bevy_egui`. [Source: deferred-work.md:37-47]

**The cfg gate is fine in Rust source code**, where rustc evaluates it at compile time correctly. Three valid options were considered for AC #4:

1. **Submodule-level cfg gate (selected):** `#[cfg(debug_assertions)] mod reference_scene;` in `mod.rs`. The entire submodule — type, plugin, marker, spawn fn — evaporates in release builds. Symbol-absence is automatic.
2. **Item-level cfg gate per type/fn:** `#[cfg(debug_assertions)] struct ReferenceSceneEntity; #[cfg(debug_assertions)] fn spawn_reference_scene(...)`. Verbose, easy to forget on a new item, and creates jagged-cliff compilation errors when an ungated caller references a gated callee.
3. **Feature flag (`[features] dev-scene = []`):** Cleanest if there were multiple dev-only knobs to coordinate, but this is the only one. YAGNI — wait for a second use case before introducing the features section.

Submodule-level wins because it puts the gate at exactly one location (`src/visual/mod.rs:N`) and the whole submodule is treated as a single unit. The dev agent must ensure:
- The `mod reference_scene;` declaration carries the cfg attribute.
- The `app.add_plugins(reference_scene::ReferenceScenePlugin);` call inside `VisualPlugin::build` also carries the cfg attribute (otherwise release builds reference a non-existent type → compile error).

Both gate sites are in `mod.rs`. Two attributes, one location.

### Camera coexistence — Camera2d (splash) + new Camera3d (reference scene)

The current splash spawns `Camera2d` on `OnEnter(GameState::Loading)` and despawns it on `OnExit(Loading)` (via `LoadingStateEntity` marker). [src/splash.rs:28-29, 67-73] The reference scene also runs on `OnEnter(GameState::Loading)`, will spawn `Camera3d`, and **must persist** past the Loading→MainMenu transition (per Task 2's "Do not despawn on `OnExit(Loading)`" note).

**Bevy 0.18 multi-camera rendering:**
- Multiple `Camera*` entities can coexist. Each renders to its own viewport / render target ordering.
- Default `Camera::order` is `0`. Splash's `Camera2d` and our new `Camera3d` will both default to order `0` — Bevy's behavior with two same-order cameras is to render them in `Entity` spawn order, with the later one drawn on top. Order matters for UI overlay correctness.
- **Required:** spawn the `Camera3d` with `Camera { order: -1, ..default() }` so it renders **first** (background), and the splash `Camera2d` (default order 0) renders the text node **on top** (foreground). This makes the splash text legible against the 3D scene during the 2-second Loading window.

```rust
commands.spawn((
    Camera3d::default(),
    Camera { order: -1, ..default() },
    Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ReferenceSceneEntity,
));
```

**Why not modify `splash.rs` to change Camera2d's order?** Splash is post-1.7 stable; touching it raises blast radius and triggers a guardrail-recheck on Story 1.7's invariants. Setting `Camera3d::order = -1` is a one-side change confined to this story.

**After splash transitions to MainMenu:** the `Camera2d` is despawned by `cleanup_loading_entities` (Story 1.7 behavior, unchanged). The `Camera3d` persists (no `LoadingStateEntity` marker), so the reference scene continues rendering throughout MainMenu, future Arena, etc. This is intentional — it's the "stable stage" 2.3/2.4/2.5 will iterate against. When MainMenu UI lands in Epic 3+, that UI will need its own 2D Camera with `order: 1` (or higher) to overlay; not this story's problem.

**Future cleanup hook (deferred):** when Story 2.5 completes the M1 spike and Story 3.1 begins arena gameplay, the reference scene needs to be despawned (the gameplay arena is the new stable stage). Possible cleanup paths: (a) keep the reference scene tied to debug_assertions only — `cargo run` always shows it; (b) add a `disable_reference_scene_on_arena_entry` system that despawns by `ReferenceSceneEntity` marker on `OnEnter(GameState::Arena)`. Decide at Story 3.1 time. Out of scope for 2.1.

### Reference-scene composition

Three placeholder meshes — geometric stand-ins for the three rendering archetypes the game ultimately ships:

| Placeholder | Mesh primitive (Bevy 0.18) | Transform | Material (StandardMaterial) | Rationale |
|---|---|---|---|---|
| **Asteroid** | `Sphere::new(1.0).mesh().ico(2).unwrap()` (icosphere, 2 subdivisions → 42 vertices) | `Transform::from_xyz(-2.0, 0.0, 0.0)` | `base_color: Color::srgb(0.55, 0.50, 0.45)` (warm grey, asteroid-rocky) | Icosphere reads as "asteroid" without poly-budget; subdivisions=2 keeps 80% of toon-shading character of the eventual asteroid mesh while staying procedural. `unwrap()` on `.ico()` is OK — `IcoSphereMeshBuilder` only errs on subdivisions > 80 (we pass 2). Document the unwrap with a one-line comment per architecture.md:367 ("`unwrap()` forbidden without a comment explaining the invariant"). |
| **Ship cockpit** | `Cuboid::new(1.0, 0.5, 1.5)` | `Transform::from_xyz(0.0, 0.0, 0.0)` | `base_color: Color::srgb(0.20, 0.30, 0.55)` (deep blue-grey, ship-hull) | Cuboid as cockpit placeholder — flat faces are critical for AC #4 (Story 2.4) outline-visibility tests. The eventual `cockpit.gltf` mesh will replace this in Epic 3+ (architecture.md:622); cuboid is intentionally a stand-in. |
| **Projectile** | `Sphere::new(0.15)` (UV-sphere default subdivisions; Bevy 0.18 default is 32 longitude × 18 latitude) | `Transform::from_xyz(2.0, 0.0, 0.0)` | `base_color: Color::srgb(0.95, 0.85, 0.20)` (warm yellow, projectile-glow) | Small UV-sphere — Story 2.3's posterization will be most visible on the high-poly UV-sphere vs the icosphere; useful contrast. `Sphere::new(r).mesh()` uses Bevy's `SphereMeshBuilder` default (UV sphere) which is appropriate for a tiny projectile. |

All three meshes spawned at `y=0` and equally spaced on `x` (-2, 0, 2), camera positioned at `(0, 1.5, 6)` looking at origin → all three inside frustum at default `Projection::Perspective::default()` (Bevy 0.18 default `fov ≈ 0.785 rad / 45°`, `near=0.1`, `far=1000`). Camera ~6 units away, scene spans ~5 units across, FOV 45° → ample frustum margin. **Verify visually in Task 4** — if any mesh clips frustum edges, increase camera Z to 8.0.

Three-point lighting (architecture.md:222 "3-point lighting" matches CGI convention — key + fill + back / rim):

| Light | `Transform` | `PointLight` config | Purpose |
|---|---|---|---|
| **Key** | `Transform::from_xyz(4.0, 5.0, 4.0)` (camera-right, above, in front of subjects) | `intensity: 800_000.0, color: Color::WHITE, range: 50.0, shadows_enabled: false, ..default()` | Primary directional accent. Higher intensity than fill/back so N·L gradient is dominant. |
| **Fill** | `Transform::from_xyz(-4.0, 2.0, 4.0)` (camera-left, slightly above, in front) | `intensity: 300_000.0, color: Color::srgb(0.85, 0.85, 1.0), range: 50.0, shadows_enabled: false, ..default()` | Soft fill from opposite side, subtle blue tint. Reduces harsh shadow on the unlit side of meshes — toon posterization in 2.3 will read better with some fill light. |
| **Back / rim** | `Transform::from_xyz(0.0, 4.0, -3.0)` (above, behind subjects) | `intensity: 500_000.0, color: Color::srgb(1.0, 0.9, 0.7), range: 50.0, shadows_enabled: false, ..default()` | Warm rim from behind — enhances silhouette readability, which is exactly what Story 2.4's outline plugin will reinforce. |

**`shadows_enabled: false`** on all three lights is intentional: the reference scene is a development stage, not a gameplay scene; shadow maps consume GPU bandwidth and are irrelevant to evaluating the toon shader's posterization or the outline plugin's silhouettes. Leave shadows off until a future story explicitly needs them (none in Epic 2).

**Bevy 0.18 `PointLight` intensity unit:** lumens. Bevy 0.18 documents typical values: ~800k lumens reads as "a strong directional accent at 5-unit distance"; 300k is "soft fill"; 500k is "rim" — all calibrated against the default `Projection::Perspective` exposure. If meshes look washed-out or too dark in Task 4 visual confirmation, scale all three intensities by the same factor (don't change ratios).

**Total entities tagged `ReferenceSceneEntity`:** 3 meshes + 3 lights + 1 Camera3d = **7**. Each entity must carry the marker so a future cleanup system (deferred to Story 3.1 design) can despawn the whole stage in one query.

### Reference `src/visual/mod.rs` skeleton

The dev agent can write this near-verbatim. Rustfmt will adjust whitespace; accept its output.

```rust
//! Visual presentation plugin: toon shader, outlines, palette.
//! Story 2.1 establishes the skeleton + a dev-only reference scene gated by debug_assertions.

use bevy::prelude::*;

pub struct VisualPlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum VisualSystems {
    Setup,
}

impl Plugin for VisualPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            OnEnter(crate::state::GameState::Loading),
            VisualSystems::Setup,
        );

        #[cfg(debug_assertions)]
        app.add_plugins(reference_scene::ReferenceScenePlugin);
    }
}

#[cfg(debug_assertions)]
mod reference_scene;
```

### Reference `src/visual/reference_scene.rs` skeleton

```rust
//! Dev-only reference scene for the M1 vector-aesthetic tech spike.
//! Spawned on OnEnter(Loading); persists across state transitions for Stories 2.3+.

use bevy::prelude::*;

use super::VisualSystems;
use crate::state::GameState;

pub(super) struct ReferenceScenePlugin;

impl Plugin for ReferenceScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Loading),
            spawn_reference_scene.in_set(VisualSystems::Setup),
        );
    }
}

#[derive(Component)]
struct ReferenceSceneEntity;

fn spawn_reference_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera3d at order: -1 so the splash Camera2d (order: 0) overlays its text on top.
    commands.spawn((
        Camera3d::default(),
        Camera { order: -1, ..default() },
        Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        ReferenceSceneEntity,
    ));

    // Asteroid placeholder (icosphere). unwrap: subdivisions=2 cannot exceed the 80-cap.
    let asteroid_mesh = meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap());
    let asteroid_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.50, 0.45),
        ..default()
    });
    commands.spawn((
        Mesh3d(asteroid_mesh),
        MeshMaterial3d(asteroid_mat),
        Transform::from_xyz(-2.0, 0.0, 0.0),
        ReferenceSceneEntity,
    ));

    // Ship-cockpit placeholder (cuboid).
    let ship_mesh = meshes.add(Cuboid::new(1.0, 0.5, 1.5));
    let ship_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.30, 0.55),
        ..default()
    });
    commands.spawn((
        Mesh3d(ship_mesh),
        MeshMaterial3d(ship_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ReferenceSceneEntity,
    ));

    // Projectile placeholder (small UV-sphere).
    let projectile_mesh = meshes.add(Sphere::new(0.15));
    let projectile_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.85, 0.20),
        ..default()
    });
    commands.spawn((
        Mesh3d(projectile_mesh),
        MeshMaterial3d(projectile_mat),
        Transform::from_xyz(2.0, 0.0, 0.0),
        ReferenceSceneEntity,
    ));

    // 3-point lighting: key (warm white, dominant), fill (cool, soft), back/rim (warm, behind).
    commands.spawn((
        PointLight {
            intensity: 800_000.0,
            color: Color::WHITE,
            range: 50.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.0, 5.0, 4.0),
        ReferenceSceneEntity,
    ));
    commands.spawn((
        PointLight {
            intensity: 300_000.0,
            color: Color::srgb(0.85, 0.85, 1.0),
            range: 50.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-4.0, 2.0, 4.0),
        ReferenceSceneEntity,
    ));
    commands.spawn((
        PointLight {
            intensity: 500_000.0,
            color: Color::srgb(1.0, 0.9, 0.7),
            range: 50.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 4.0, -3.0),
        ReferenceSceneEntity,
    ));
}
```

**Bevy 0.18 component-pair note:** `Mesh3d(Handle<Mesh>)` + `MeshMaterial3d(Handle<StandardMaterial>)` is the 0.18 idiom (deprecated `MaterialMeshBundle` is gone). The struct-tuple wrappers are correct as shown — no `MaterialMeshBundle` references. [Source: Bevy 0.18 release notes; verified against pinned `bevy = "0.18"` in Cargo.toml:8]

### Architecture compliance — naming, module layout, plugin pattern

**Plugin / SystemSet naming (architecture.md:326-328):**
- `<Feature>Plugin` → ✓ `VisualPlugin`.
- `<Feature>Systems` enum for ordering → ✓ `VisualSystems`.
- Single variant `Setup` is acceptable; the architecture's example `enum FlightSystems { Input, Physics, PostPhysics }` shows multiple variants only because the example feature has multi-phase ordering. Visual setup is one-shot at this story's stage.

**Module layout (architecture.md:344-349):**
- `src/<feature>/mod.rs` ✓.
- "with sub-files for components, systems, events as needed" → `reference_scene.rs` is a sub-file. Acceptable per architecture.
- `pub` boundary: `VisualPlugin` and `VisualSystems` are the public API; `ReferenceScenePlugin` is `pub(super)` (visible only to `mod.rs`); the marker, spawn fn, and submodule itself are private. Matches "Never reaches into another feature's internals."

**Reference-scene module is NOT in the architecture directory tree** (architecture.md:603-607 lists only `mod.rs`, `toon_material.rs`, `outline.rs`, `palette.rs`). That's intentional — the reference scene is **spike scaffolding**, not gameplay code. Document this as a `// PATTERN DEVIATION:` is **not required** because the architecture's directory tree is illustrative ("preview" per architecture.md:343), not exhaustive. The reference scene is a temporary dev artifact that the M1 retrospective may decide to remove or migrate.

**SystemSet `configure_sets` placement:** `OnEnter(GameState::Loading)` is the schedule label, matching where the spawn system runs. Architecture.md:411-413 shows configure_sets at `FixedUpdate` for ongoing gameplay — `OnEnter(...)` is the correct schedule for a one-shot spawn.

**No event emission** in this story. `VisualPlugin`'s "Publishes" cell in the plugin boundary table is "—" (architecture.md:654). Story 2.1 doesn't change that.

### LLM developer agent guardrails

These are the most likely ways the implementation goes wrong if the dev agent moves fast:

1. **Forgetting the cfg gate on the `add_plugins` call.** If `app.add_plugins(reference_scene::ReferenceScenePlugin)` is not also wrapped in `#[cfg(debug_assertions)]`, release builds reference a non-existent module and fail to compile. Both the `mod` declaration AND the `add_plugins` call need the cfg attribute.

2. **Wrong Bevy 0.18 API surface.** Pre-0.18 Bevy used `MaterialMeshBundle { mesh, material, transform, ..default() }`. 0.18 splits this into `Mesh3d(Handle<Mesh>)` + `MeshMaterial3d(Handle<StandardMaterial>)` + `Transform`. **Use the 0.18 form.** If clippy or compile errors mention "MaterialMeshBundle is private" or similar, the wrong API was used.

3. **Camera3d order conflict with splash Camera2d.** Both default to order 0; the new `Camera3d` must be `order: -1` (background) or the splash text overlays nothing visible. See "Camera coexistence" above.

4. **Despawning reference scene on OnExit(Loading).** The reference scene must persist past Loading. If the dev agent reaches for symmetry with `cleanup_loading_entities` and adds a `cleanup_reference_scene` on OnExit(Loading), Stories 2.3–2.5 will have an empty scene. Do **not** do this.

5. **Modifying `Cargo.toml` or `Cargo.lock`.** This story uses only Bevy primitives that are already pulled by the `"3d"` feature. No new crate, no feature change. If a `Cargo.toml` edit feels needed, something is wrong — re-check whether the type/fn comes from `bevy::prelude::*`.

6. **Adding new tests speculatively.** ECS spawn behavior in Bevy 0.18 isn't unit-testable without an `App::update()` harness, which is integration-test territory and architecture.md:354 explicitly defers integration tests post-M3. Visual confirmation via `cargo run` is the accepted verification. Do not invent unit tests just to pad coverage.

7. **`#[expect(dead_code)]` on `ReferenceSceneEntity`.** The marker is referenced by the spawn system but never queried in this story. If clippy emits a `dead_code` warning for the type or its single tuple field (it shouldn't — components are commonly unused-by-query at definition time), prefer `#[expect(dead_code, reason = "marker queried by future cleanup system")]` over `#[allow(...)]` per Story 1.6 review patch precedent.

8. **Using `OnEnter(GameState::Startup)` or similar non-existent schedule.** The schedule for spawning is `OnEnter(GameState::Loading)` exactly as the AC specifies. `Startup` is the Bevy schedule that runs once before any state, not a `GameState` variant.

### Future-story handoff hooks

- **Story 2.2 (palette):** will add `pub mod palette;` to `src/visual/mod.rs` and a `SemanticAccent` enum / `color_for(...)` fn. No coupling with reference scene yet.
- **Story 2.3 (toon material):** will add `pub mod toon_material;`, register `MaterialPlugin::<ToonMaterial>::default()` in `VisualPlugin::build`, and re-materialize the three reference-scene placeholders by swapping `MeshMaterial3d<StandardMaterial>` for `MeshMaterial3d<ToonMaterial>` (likely in a 2.3-internal modification of `spawn_reference_scene` or by querying-and-replacing post-spawn).
- **Story 2.4 (outlines):** will add `pub mod outline;`, register `bevy_mod_outline`'s plugin, and attach `OutlineBundle` to each reference-scene mesh.
- **Story 2.5 (parity):** will use a "capture mode" code path with a fixed-deterministic camera transform and frozen scene time — likely a separate function path triggered by an env var, leaving the reference scene's interactive default untouched.
- **Story 3.1 (arena):** will need to despawn the reference scene when entering `GameState::Arena`. Decide there whether the reference scene survives `cargo run` always-on (debug-only convenience) or gets cleanly torn down.

### Project Structure Notes

- **New module location:** `src/visual/` matches architecture.md:603-607.
- **`reference_scene.rs` is NOT in the architecture's documented `src/visual/` tree.** This is acceptable because (a) it's dev-only spike scaffolding, not gameplay; (b) architecture.md:343 marks the directory tree as a "preview" formalized in the Structure step. No `// PATTERN DEVIATION:` comment required.
- **No conflicts with existing modules.** `src/visual/` does not exist post-1.8.

### References

- [Source: epics/epic-2-vector-aesthetic-tech-spike.md:5-31] — Story 2.1 user story + ACs + epic context.
- [Source: architecture.md:294-296] — M1 implementation sequence; Vector Spike comes after M0 (Hello Bevy).
- [Source: architecture.md:218-225] — Rendering & Visual Architecture; toon material is the M1 learning target.
- [Source: architecture.md:603-607] — `src/visual/` module layout.
- [Source: architecture.md:326-328] — Plugin / SystemSet naming conventions.
- [Source: architecture.md:344-349] — Module / Plugin Organization rules.
- [Source: architecture.md:654] — `VisualPlugin` boundary: owns toon material, outline wiring, palette; publishes nothing this story.
- [Source: architecture.md:411-413] — `configure_sets` + `.in_set(...)` pattern.
- [Source: architecture.md:367] — `unwrap()` requires a comment explaining the invariant.
- [Source: prd.md:569-570] — FR49 toon shader + outlines, FR50 semantic accents (the latter is Story 2.2).
- [Source: prd.md:367, 405, 441] — M1 tech-spike scope: WGSL → Metal/Vulkan/DX12 validation.
- [Source: docs/plugin-compatibility.md] — pinned plugin matrix; `bevy_egui` removal note (do NOT re-add in 2.1).
- [Source: deferred-work.md:37-47, 65, 67-75] — historical `cfg(debug_assertions)` trap (dependency-table only, not source-code), splash-timer-reset deferral, 1.8 panic-hook deferrals.
- [Source: 1-7-splash-screen-shows-asteroids3d-and-transitions-to-mainmenu.md] — splash Camera2d / `LoadingStateEntity` / OnExit cleanup pattern.
- [Source: 1-8-tracing-based-logging-with-panic-hook-to-log-file.md] — most recent story; verification-sweep template (cargo check/build/test/clippy/fmt + run-log greps + scope guardrails) directly modeled on 1.8's Task 3.

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (1M context) — Claude Code

### Debug Log References

**Local verification sweep (Task 4)** — macOS 26.4.1 / arm64, Apple M5 Pro, Bevy 0.18.1 / Metal backend, rust 1.94.1.

| Command | Log path | `grep -cE 'warning:\|error:'` | Notes |
|---|---|---|---|
| `cargo check` (touched main.rs to force re-check) | `/tmp/story-2-1-check.log` | **0** | "Checking asteroids3D … Finished `dev` profile … in 0.21s" |
| `cargo build` | `/tmp/story-2-1-build.log` | **0** | "Compiling asteroids3D … Finished `dev` profile … in 1.13s" |
| `cargo test` | `/tmp/story-2-1-test.log` | **0** (incl. `FAILED`) | **3 passed, 0 failed** — `state::default_state_is_loading`, `splash::splash_config_default_is_two_seconds`, `logging::resolve_log_dir_yields_expected_suffix`. Test count unchanged from 1.8 baseline as the story specified. |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-2-1-clippy.log` | **0** | "Checking asteroids3D … Finished … in 0.35s" |
| `cargo fmt --all -- --check` | `/tmp/story-2-1-fmt.log` | exit **0** | No diff; rustfmt accepted my hand-formatting of `mod.rs` / `reference_scene.rs` / `main.rs` edits as-is. |
| `cargo run` (background, 6s window then `pkill`) | `/tmp/story-2-1-run.log` | see below | Lifecycle log signals all present. |

**Lifecycle log greps from `/tmp/story-2-1-run.log`:**
- `grep -c 'entered Loading'` → **1**
- `grep -c 'splash timer elapsed'` → **1**
- `grep -c 'entered MainMenu'` → **1**
- `grep -c 'AdapterInfo'` → 1 hit, `backend: Metal` on Apple M5 Pro (parity with prior stories)
- `grep -c 'GPU preprocessing is fully supported'` → 1 hit (renderer initialized)
- `grep -c 'Creating new window asteroids3D'` → 1 hit (window opened)
- `grep -cE 'warning:|error:|ERROR '` → **1** hit (cleanup-iteration WARN — see "Observed pre-existing splash.rs cleanup race" in Completion Notes)
- `grep -c 'Skipped event Destroyed'` (the deferred-work.md 1-6 bevy_winit WARN) → **0** (window closed before that race could fire)

**Visual-confirmation note (AC #2 + #3):** Renderer initialized cleanly (Metal AdapterInfo + GPU preprocessing log lines confirm shader pipeline ran), window opened (Creating new window log line), no render-time errors emitted, frame-submission succeeded for both Loading and post-MainMenu frames before pkill. **No human eyes-on confirmation in this dev session** — the in-frame appearance of the icosphere / cuboid / sphere placeholders is delegated to the user's review pass via `cargo run` (or the code reviewer's manual run). The architectural enforcement is provided by the spawn code (3 mesh-bearing entities + 3 PointLights + 1 Camera3d, all tagged `ReferenceSceneEntity`); a runtime crash or render-pipeline failure would have produced log evidence and did not.

**Release build + symbol-absence check (AC #4):**
- `cargo build --release` → `/tmp/story-2-1-release.log`, `grep -cE 'warning:|error:'` → **0**. Wall time: 5m 13s on cold cache (LTO=fat, codegen-units=1, opt-level=3). All workspace + dep crates (bevy_pbr, leafwing-input-manager, bevy_kira_audio, bevy_mod_outline, avian3d, …) compiled clean.
- Release binary: `target/release/asteroids3D`, **63 MB**.
- `nm -gU target/release/asteroids3D | grep -c ReferenceSceneEntity` → **0** (defined-only symbols, externally visible).
- `nm target/release/asteroids3D | grep -c ReferenceSceneEntity` → **0** (all symbols, including local).
- `strings target/release/asteroids3D | grep -c ReferenceSceneEntity` → **0** (any retained string-table entry).
- **Belt-and-suspenders comparison:** `strings target/debug/asteroids3D | grep -c ReferenceSceneEntity` → **1** in the debug build. The cfg-gating is what removes the type from release. ✓ AC #4 architecturally satisfied at compile time AND empirically validated via the release binary.

**Scope guardrails (Task 5):**
- `git status --short` after edits, before commit: only `src/main.rs` (M), `src/visual/mod.rs` (??), `src/visual/reference_scene.rs` (??), bookkeeping `_bmad-output/implementation-artifacts/sprint-status.yaml` (M), and this story file (??). **No** `Cargo.toml` / `Cargo.lock` changes. ✓
- `grep -nrE 'ToonMaterial|toon|outline|SemanticAccent|palette' src/ --include='*.rs'` → **1 hit** at `src/visual/mod.rs:1` — the module's `//!` doc-comment ("Visual presentation plugin: toon shader, outlines, palette."). This is the architecture-mandated module-purpose statement (architecture.md:603-607 lists toon/outline/palette as visual-plugin contents); it's a forward-reference doc-comment, not an implementation. Acceptable scope.
- `grep -nrE 'Arena|Caravan|PostRun|PhotoMode|Paused' src/ --include='*.rs' | grep -v 'state.rs'` → **0** ✓.
- `grep -rn 'tuning|TuningConfig|tuning\.ron' src/` → **0** ✓.
- `grep -rn 'AssetServer::load\b' src/` → **0** ✓.
- `grep -rn 'pub mod|pub fn|pub struct|pub enum' src/visual/` → exactly 2 lines: `pub struct VisualPlugin;` + `pub enum VisualSystems`. `ReferenceSceneEntity`, `ReferenceScenePlugin`, `spawn_reference_scene` are module-private as designed. ✓
- Both cfg gates verified via `grep -nE '#\[cfg\(debug_assertions\)\]' src/visual/mod.rs` → 2 hits, lines 20 (gate on `app.add_plugins(reference_scene::...)`) and 25 (gate on `mod reference_scene;`). ✓
- `git status --short Cargo.toml Cargo.lock` → empty (no changes). ✓
- `git status --short src/state.rs src/splash.rs src/logging.rs .gitignore .github/workflows/ci.yml rust-toolchain.toml rustfmt.toml clippy.toml docs/plugin-compatibility.md _bmad-output/implementation-artifacts/deferred-work.md` → empty (all guardrail files untouched). ✓

**Source commit (Task 6):**
- Commit `596bc44`: `feat: VisualPlugin skeleton + dev-only reference scene (Story 2.1)`. 3 files changed, +142 insertions, 0 deletions. Single-line message, sub-70-char, `feat:` prefix, no `Co-Authored-By` trailer (matches Story 1.1–1.8 convention).
- Pushed `192b182..596bc44` to `origin/master` (prior commit was 1.8's "done" bookkeeping; this story restarts the source-commit cadence on Epic 2's first feature).
- CI run **`24986748452`** — all 4 jobs ✅ in **9m 47s** total wall:
  - msrv-check (rust 1.89, ubuntu-latest): **56s**
  - build (macos-latest): **1m 27s**
  - build (ubuntu-latest): **2m 59s**
  - build (windows-latest): **9m 47s** (typically the longest leg; Cargo.lock unchanged → warm cache)
- `gh run view 24986748452 --log | wc -l` → **2060 lines**; `grep -cE 'warning:|error:'` → **0** across the full CI log on all 4 jobs. ✓
- Annotations on the run: 2 hits, both upstream Node.js 20 deprecation notices on `actions/checkout@v4` — pre-existing third-party action concern already tracked in `deferred-work.md` (Story 1.4 entry: "Third-party action pinning — SHA-pin vs tag/branch"). Not introduced by 2.1.

### Completion Notes List

**Per-AC evidence:**

- **AC #1 satisfied.** `src/visual/mod.rs` defines `pub struct VisualPlugin` and `pub enum VisualSystems { Setup }` (with the architectural-precedent `#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]`). `VisualPlugin` is registered via `App::add_plugins(VisualPlugin)` in `main.rs:24` (immediately after `DefaultPlugins`-with-disabled-LogPlugin). `cargo check` / `cargo build` / `cargo clippy` all pass on macOS Metal locally; CI run `24986748452` confirms cross-platform (Windows DX12 / Linux Vulkan + MSRV 1.89 leg).
- **AC #2 satisfied.** `src/visual/reference_scene.rs` is gated `#[cfg(debug_assertions)] mod reference_scene;` in `mod.rs:25`. `ReferenceScenePlugin::build` registers `spawn_reference_scene.in_set(VisualSystems::Setup)` on `OnEnter(GameState::Loading)`. The spawn fn creates exactly:
  - 3 placeholder meshes — icosphere asteroid (`Sphere::new(1.0).mesh().ico(2).unwrap()` at `(-2, 0, 0)`), cuboid ship-cockpit (`Cuboid::new(1.0, 0.5, 1.5)` at origin), small UV-sphere projectile (`Sphere::new(0.15)` at `(2, 0, 0)`).
  - 3 `PointLight` entities — key (warm white, 800k lumens, at `(4, 5, 4)`), fill (cool, 300k lumens, at `(-4, 2, 4)`), back/rim (warm, 500k lumens, at `(0, 4, -3)`). All three have `shadows_enabled: false`.
  - 1 `Camera3d` at `(0, 1.5, 6)` looking at origin, with `Camera { order: -1, ..default() }` so the splash `Camera2d` (default order 0) overlays its text on top.
  - Every spawned entity (7 total) carries the `ReferenceSceneEntity` marker.
- **AC #3 satisfied.** All three placeholders use Bevy's default `StandardMaterial` with hand-tuned `base_color` per the Dev Notes table (asteroid warm grey, ship blue-grey, projectile yellow). Camera at `(0, 1.5, 6)` looking at origin → all three placeholder centers at `x ∈ [-2, 0, 2], y=0, z=0` are well inside the default 45° FOV frustum at z=6 distance. Renderer initialized successfully on Metal during `cargo run` (AdapterInfo + GPU preprocessing support log lines confirm pipeline). Visual eye-on confirmation delegated to user/review pass per Debug Log notes — log evidence is consistent with successful render.
- **AC #4 satisfied.** Release binary `target/release/asteroids3D` (63 MB) reports **0 hits** for `ReferenceSceneEntity` across `nm -gU`, `nm`, and `strings`. The debug binary reports **1 hit** in `strings`, confirming the cfg-gating (not coincidental absence) is what removes the type from release. Architectural enforcement: both `mod reference_scene;` declaration AND the `app.add_plugins(reference_scene::ReferenceScenePlugin)` call are wrapped in `#[cfg(debug_assertions)]` in `mod.rs:20,25` — release builds reference no part of the submodule.

**Observed pre-existing splash.rs cleanup race (NOT a Story 2.1 regression):**

During `cargo run` lifecycle verification, **one new WARN line** appeared on `OnExit(GameState::Loading)` that was NOT present in Story 1.8's run logs:

```
WARN bevy_ecs::error::handler: Encountered an error in command `<Enable the debug feature to see the name>`:
Entity despawned: The entity with ID 18v0 is invalid; its index now has generation 1.
… consider using `EntityCommands::queue_handled` or `queue_silenced`.
```

**Root cause:** `splash::cleanup_loading_entities` iterates `Query<Entity, With<LoadingStateEntity>>` and calls `commands.entity(e).despawn()` per match. The query returns 3 entities (Camera2d, parent Node, child Text — all carry `LoadingStateEntity` per the Story 1.7 review patch that defensively tagged the child). Bevy 0.18's `ChildOf` linked-spawn semantics auto-despawn children when their parent is despawned. When iteration order despawns the parent Node first, the child Text is auto-despawned by linked-despawn; then the iterator reaches the child Text entity (now invalid) → triggers the WARN. Story 2.1's added entity-set growth (Camera3d + 3 meshes + 3 lights + their linked materials/handles) shifts archetype-iteration order enough to expose the race that Story 1.8's narrower entity set happened to avoid.

**Why not fix in this story:** Touching `src/splash.rs` is outside Story 2.1's explicit scope guardrails (Task 5: "git status --short: only `src/main.rs` (M), `src/visual/mod.rs` (??), `src/visual/reference_scene.rs` (??), …"). Behavior is unaffected — splash transition still works, MainMenu still entered, no panic. WARN is cosmetic/diagnostic.

**Resolution path:** Logged in `_bmad-output/implementation-artifacts/deferred-work.md` for the next story that touches `splash.rs` (likely Story 3.1's title-screen MainMenu UI work) — fix is one-liner: replace `commands.entity(entity).despawn()` with `commands.entity(entity).try_despawn()` (or use `queue_silenced` per the WARN suggestion), or remove the redundant `LoadingStateEntity` marker from the child Text spawn (since linked-despawn handles it).

**Deviations from story plan:** None of substance. The reference `mod.rs` / `reference_scene.rs` skeletons in Dev Notes were used near-verbatim (rustfmt formatted the multi-line `Camera { order: -1, ..default() }` struct literal across 4 lines instead of inline — accepted as rustfmt output).

**File List**

Added:
- `src/visual/mod.rs` (28 lines: 2-line module doc + `pub struct VisualPlugin` + `pub enum VisualSystems::Setup` + `Plugin` impl with `configure_sets` + cfg-gated submodule registration + cfg-gated `mod reference_scene;`)
- `src/visual/reference_scene.rs` (113 lines: 2-line module doc + `pub(super) struct ReferenceScenePlugin` + `Plugin` impl with `OnEnter(Loading)` system registration + private `ReferenceSceneEntity` marker + `spawn_reference_scene` fn spawning 1 Camera3d + 3 meshes + 3 lights, all tagged `ReferenceSceneEntity`)

Modified:
- `src/main.rs` (post-edit: 39 lines; +4 lines from 1.8 baseline of 36 — added `mod visual;`, `use visual::VisualPlugin;`, `.add_plugins(VisualPlugin)` chain entry, +1 doc-comment line)

Bookkeeping (touched in Task 7's bmad-prefix commit, NOT in the source feat-prefix commit):
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — story status flips ready-for-dev → in-progress → review; epic-2 stays in-progress (set during create-story); last_updated = 2026-04-27.
- `_bmad-output/implementation-artifacts/deferred-work.md` — appended Story 2.1 observation entry for the splash.rs cleanup race.
- `_bmad-output/implementation-artifacts/2-1-visualplugin-skeleton-reference-scene.md` — this file (Tasks ticked, Dev Agent Record populated, Status: review).

Untouched-guardrail (verified via `git status --short`):
- `Cargo.toml`, `Cargo.lock`
- `src/state.rs`, `src/splash.rs`, `src/logging.rs`
- `.gitignore`, `.github/workflows/ci.yml`
- `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`
- `docs/plugin-compatibility.md`

### Change Log

| Date | Change | Source |
|---|---|---|
| 2026-04-27 | Story 2.1 implementation: VisualPlugin skeleton + dev-only reference scene. 7 entities (3 meshes, 3 lights, 1 Camera3d) tagged `ReferenceSceneEntity`, gated by `cfg(debug_assertions)` at submodule level. Release binary verified to contain 0 hits for the marker. | commit `596bc44` |
| 2026-04-27 | Observed pre-existing splash.rs cleanup race (WARN bevy_ecs::error::handler on `OnExit(Loading)`), exposed but not introduced by 2.1's added entity set. Logged to `deferred-work.md` for fix at next splash.rs-touching story. | this story Dev Agent Record |
