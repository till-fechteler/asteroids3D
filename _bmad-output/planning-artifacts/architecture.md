---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/prd-validation-report.md
  - _bmad-output/brainstorming/brainstorming-session-2026-04-21-1114.md
workflowType: 'architecture'
project_name: 'asteriods3D'
user_name: 'Till'
date: '2026-04-22'
lastStep: 8
status: 'complete'
completedAt: '2026-04-22'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:** 50 FRs across 8 capability clusters — Flight & Controls (8), Combat (8), Economy & Salvage (5), Perception & Sensors (5), Run Structure & Progression (9), UI & Feedback (8), Persistence & Platform (5), Visual Presentation (2). All validated as capability-focused (no implementation leakage) with traceability to user journeys and Success Criteria.

Architecturally significant FRs:
- FR4 decoupled aim → reticle overlay system independent of ship transform
- FR5 inertial dampener toggle → two physics response modes on player ship
- FR7 tractor beam on intact asteroids → Avian constraint/impulse subsystem
- FR8 cockpit-only in active gameplay + FR40 photo mode post-run only → two disjoint camera-controller contexts sharing the same world/shader pipeline
- FR22 + FR23 sensor radar + spatial stereo audio → dual perception channels with redundant encoding
- FR32 rendering-distance-triggered combat pockets → streaming/trigger-volume system in Caravan
- FR46 crash-safe saves → atomic write (temp-file + rename) pattern
- FR47 + FR48 three-platform first-class with macOS notarization → CI matrix from M0
- FR49 toon shader + FR50 semantic accent colors → custom WGSL material on wgpu with three-backend parity

**Non-Functional Requirements:** 18 NFRs. Architecturally binding:
- NFR-P1: 60 FPS @ 1080p on GTX 1060 / Apple M1 baseline — every system operates within ~16 ms frame budget
- NFR-P4: No steady-state hitches > 100 ms — streaming/async for assets, staged spawns
- NFR-P5: < 4 GB process memory
- NFR-R2: Atomic save writes (temp-file + rename)
- NFR-R4: No between-run meta-currency loss — persistence is load-bearing
- NFR-L3: Strings externalized from day 1 (JSON or RON; final format is an architecture decision)

**Scale & Complexity:**
- Primary domain: Desktop 3D game (Bevy + Avian + custom WGSL shader)
- Complexity level: medium-high
- Estimated architectural components: ~12–15 logical modules (Flight, Combat, Salvage/Economy, Ship-State, Enemy-AI, Perception/Sensors, HUD, Run-Director, Save/Load, Settings, Photo-Mode, Audio, Rendering/Shader, Platform-Shell)
- Binding constraint: motivation preservation over 10–14 calendar months at 4–8 h/week; every milestone must yield perceptibly improved playable state

### Technical Constraints & Dependencies

**Fixed stack (non-negotiable, carried from brainstorming Phase 3 + PRD):**
- Bevy (version-pinned at M0 start; upgrades only at M4/M6/M9 transitions with 4–6 h migration budget)
- Avian (Bevy-native XPBD physics, pinned; co-scheduled with Bevy upgrades)
- `bevy_mod_outline` (pinned, fork-ready if upstream stagnates)
- `bevy_kira_audio` (pinned, fork-ready)
- `directories` crate for OS save paths
- Serde for persistence

**Fixed pipelines:**
- Asset: Blender → glTF 2.0 only (no FBX, no Unity intermediates). Textures low-res to match vector aesthetic.
- Graphics: WGSL shaders via wgpu (Bevy-managed); three-backend validation (Metal / Vulkan / DX12) at M1 tech-spike gate.

**Platform matrix (all first-class, no deprioritization):**
- Windows 10+ (DX12 via wgpu)
- Linux major distros (Vulkan via wgpu)
- macOS Apple Silicon + Intel x86_64 (Metal via wgpu; code-signed + notarized from M3 onward)

**Third-party crate risk:** `bevy_mod_outline` and `bevy_kira_audio` carry upgrade-churn risk. Mitigated by version-pinning and fork-readiness acceptance.

### Cross-Cutting Concerns Identified

1. **ECS composition discipline** — every gameplay entity is a bundle of small components (per brainstorming A#6). Components such as `Damageable`, `Salvageable`, `Thrusters`, `GravityAffected`, `TractorTarget`, `Faction`, `HullHP`, `ShieldHP` should be shared across Ship / Asteroid / Projectile / Enemy. This is also an explicit learning goal — god-structs or OOP-in-ECS patterns are anti-requirements.
2. **Save/load atomicity** — any FR that mutates persistent state (meta-currency, unlocks, settings) funnels through a single save-service module with atomic write semantics (temp + rename).
3. **Cross-platform rendering parity** — WGSL shader + custom material + outline plugin must compile and render identically on Metal, Vulkan, DX12. Validated at M1 tech-spike and in CI matrix.
4. **Frame-budget ownership** — 60 FPS target is a milestone gate on every milestone. Profiling (flamegraph / tracy) integrated from M2 onward. No system is exempt from the budget.
5. **Version-pinning governance** — Bevy, Avian, bevy_mod_outline, bevy_kira_audio. Upgrades batched at M4/M6/M9 only with documented migration budget. No ad-hoc upgrades mid-milestone.
6. **Audio-sensor redundancy (R#6 two-stage)** — sensor UI is primary info source MVP; audio reinforces. Post-MVP hardcore mode reduces sensor UI, but MVP never depends on audio alone. Architecturally, perception is a pluggable pipeline with source-agnostic threat events.
7. **Camera context duality** — cockpit FPS camera during active play, free-cam over frozen sim in Photo Mode (M8). Shared rendering pipeline, different camera controllers. Photo Mode is also the debug camera (F3 toggle) per E#6 Phase-3 resolution.
8. **Localization readiness at near-zero MVP cost** — external string table from day 1 (NFR-L3). German localization is a post-MVP pull, not a refactor.
9. **Motivation-preservation ordering** — architecture decisions that lengthen invisible stretches are anti-requirements. M5 Caravan-framework stretch is the known danger zone — must be sliceable into weekly sub-milestones.

## Starter Template Evaluation

### Primary Technology Domain

Desktop 3D game in Rust + Bevy. Classical web-app starter taxonomies (Next.js, Vite, etc.) do not apply. The relevant starter space is Bevy-specific.

### Current Versions (verified April 2026)

- **Bevy:** 0.18 (released 2026-01-13) — introduces high-level cargo feature collections (`2d`, `3d`, `ui`) that significantly reduce compile time by selecting only the pieces needed.
- **Avian:** 0.6 (updated 2026-04-08) — Bevy 0.18 compatible. Successor to `bevy_xpbd`, ECS-native XPBD physics.
- **`bevy_mod_outline`:** verify Bevy 0.18 compatibility before pinning (last check during architecture — may need short-lag fork if not yet released for 0.18).
- **`bevy_kira_audio`:** same check.

### Starter Options Considered

1. **Option A — Minimal manual** (`cargo new --bin`, hand-authored): maximum learning, zero mystery, no CI/lint scaffolding.
2. **Option B — `bevy_game_template` (NiklasEi)**: widely used community template with CI for Windows/Linux/macOS/iOS/Android/Web. Learning cost: inherited patterns.
3. **Option C — `bevy_new_3d_rpg` (olekspickle)**: RPG-oriented 3D template with bells and whistles. Poor fit for cockpit shooter.
4. **Option D — Hybrid: Manual start + targeted borrowing** (selected).

### Selected Starter: Hybrid Manual

**Rationale for Selection:**

Till's explicit learning goal is "fluent in ECS-idiomatic Bevy by M3." Every line of gameplay, rendering, physics-integration, and save code should be authored by Till — pre-built templates that hide the "why" undercut this goal. However, infrastructure boilerplate (CI, lint config, gitignore) is not a learning target and can be adopted without loss. The hybrid path preserves learning ROI on the 90% that matters while absorbing infrastructure setup in the 10% that does not.

**Initialization Sequence:**

```bash
# Bootstrap
cargo new --bin asteriods3d
cd asteriods3d

# Cargo.toml and src/main.rs: author by hand (Till)
# See "Core Dependencies" decision below for exact pinned versions

# Infrastructure borrowing (study, adopt, don't re-derive):
# - .github/workflows/ci.yml (Windows + Linux + macOS matrix, strip iOS/Android/Web jobs)
# - .github/workflows/release.yml (binary artifact per OS, strip web)
# - .gitignore (Rust + Bevy asset-cache conventions)
# - rustfmt.toml, clippy allow/deny lists
```

### Architectural Decisions Provided by Starter Choice

**Language & Runtime:**
- Rust (edition 2024, latest stable toolchain). Single-crate workspace for M0–M3. Workspace split only if asset-build tooling crate becomes warranted later.

**Cargo Configuration:**
- Pin Bevy to `0.18` with explicit `default-features = false` + selective enabling. Use the 0.18 `"3d"` feature collection as a baseline; add `"png"`, platform-specific windowing (`"x11"` / `"wayland"` on Linux), and `"bevy_window"` as needed. Disable `"2d"` to keep compile time down.
- Pin Avian to `0.6` via `avian3d` crate.
- Pin `bevy_mod_outline` and `bevy_kira_audio` at Bevy-0.18-compatible versions (verify at start of M0; fork and maintain inline if upstream lags per PRD Implementation Considerations).
- Release profile: `lto = "fat"`, `codegen-units = 1`, `opt-level = 3` — per the PRD's cross-platform release-build baseline.
- Dev profile: `opt-level = 1` for dependencies (Bevy dev-speed convention), `opt-level = 0` for workspace code.

**Build Tooling:**
- `cargo` / `rustc` stable channel. Pin via `rust-toolchain.toml` to control CI reproducibility.
- No custom build scripts in M0–M3. `build.rs` only if forced by shader preprocessing or asset baking.

**Testing Framework:**
- Rust's built-in `#[test]` + `cargo test` for unit tests on pure-logic modules (economy math, trajectory prediction, save-schema serialization).
- Bevy-integration tests: deferred post-M3 unless specific regressions warrant.
- Playtest gates (per PRD) are the primary quality signal, not unit test count.

**Code Organization:**
- `src/main.rs`: `App::new()` assembly, plugin registration, top-level config.
- `src/<module>/mod.rs`: one module per logical domain (flight, combat, salvage, perception, etc.) — final module list is the Structure step's output.
- Plugin-per-module pattern: each feature module exposes a `Plugin` impl that registers its systems, components, resources, and events. This is the Bevy-idiomatic boundary.
- `assets/` directory next to the binary at runtime. Blender `.blend` source files kept out of repo; only glTF-exported meshes + textures live in `assets/`.

**Development Experience:**
- `cargo run` is the primary dev loop. Hot-reload via `bevy_asset` for textures/audio/shaders; Rust compile cycle for code changes (hot-code-reload via `subsecond` / `dexterous_developer` evaluated later, not MVP).
- `cargo clippy` + `cargo fmt` wired into CI. `#![deny(clippy::needless_pass_by_value)]` and similar quality lints on.
- `cargo flamegraph` / `tracy-client` integrated at M2 per PRD NFR-P4 requirement (no steady-state hitches > 100 ms).

**CI Matrix (adopted from `bevy_game_template`, stripped):**
- Windows (`windows-latest`), Linux (`ubuntu-latest`), macOS (`macos-latest` — Apple Silicon runner).
- Jobs: build + test + clippy + fmt-check per platform.
- Release build verification on each platform at milestone gates.
- **Removed:** iOS, Android, Web/WASM jobs (all explicitly out-of-scope per PRD NFR and project-type decisions).

**Note:** Project initialization using the sequence above should be the first implementation story of M0 ("Hello Bevy").

## Core Architectural Decisions

### Decision Priority Analysis

**Already Decided by PRD / Brainstorming / Starter:**
- Stack: Bevy 0.18 + Avian 0.6 + `bevy_mod_outline` + `bevy_kira_audio` (all version-pinned)
- Asset pipeline: Blender → glTF 2.0 only
- Save format: JSON + Serde (PRD)
- Save location: per-OS via `directories` crate (PRD)
- Platform matrix: Windows + Linux + macOS, all first-class
- CI matrix, release profile, cargo feature selection (from Starter step)

**N/A for this project:**
- Authentication & Security (offline single-player, no accounts, no payments, no PII)
- API/Backend Communication (fully offline by design per PRD E#5)

**Critical Decisions (block implementation):**
- Save-file write strategy (atomicity)
- Runtime state machine pattern
- Physics scheduling
- Shader organization
- UI framework

**Important Decisions (shape architecture):**
- String-table format
- HUD rendering strategy
- Input abstraction
- Debug UI tooling
- macOS signing workflow

**Deferred Decisions (Post-MVP):**
- Steam Cloud / Steamworks SDK integration (M6)
- HRTF spatial audio backend (R#6 stage 2)
- Rhai modding scripting (Vision)
- WASM build (Vision)
- Automated macOS notarization in CI (M6)

---

### State & Persistence Architecture

**Save-File Write Strategy:** Atomic temp-file + rename pattern on all three platforms. Write to `<savepath>.tmp`, fsync, then `rename()` (Unix) / `MoveFileEx` with `MOVEFILE_REPLACE_EXISTING` (Windows). Satisfies NFR-R2 (atomicity), NFR-R4 (no meta-currency loss), NFR-R3 (missing-save graceful recovery via first-launch default creation).

**Runtime Game State:** Bevy's `States` API (idiomatic). Top-level states: `MainMenu`, `Arena`, `Caravan`, `PostRun`, `PhotoMode`, `Paused`. `OnEnter` / `OnExit` / `in_state()` scheduling for state-transition logic. Sub-states (e.g., `MainMenu::Title` / `MainMenu::Settings`) as nested state hierarchy.

**ECS Data Modeling:** Component-composition-first. Small reusable components shared across entity archetypes (Ship / Asteroid / Projectile / Enemy). Concrete component catalogue in Structure step. Anti-requirements: god-structs, inheritance-shaped hierarchies, OOP-in-ECS. This is load-bearing for the "fluent in ECS-idiomatic Bevy" learning goal (Project Success criterion).

**String-Table Format:** RON (Rusty Object Notation) for player-facing strings. Rationale: supports comments (translator context hints), multi-line strings, better editing ergonomics than JSON for this use case. Save data stays JSON — different role, different format is acceptable. Pulled via `bevy_asset` at startup, hot-reloadable during development.

---

### Rendering & Visual Architecture

**Shader Organization:** Two-layer decomposition:
1. Custom WGSL Toon `Material` (authored by Till — this is the primary M1 learning target and a portfolio-quality code artifact).
2. `bevy_mod_outline` plugin for silhouette outlines (pinned, fork-ready, not a learning priority).

Validated on Metal / Vulkan / DX12 at M1 tech-spike gate. Fallback: flat-shaded + rim-light per PRD Phase-3 M#10 resolution.

**HUD Rendering Strategy:** Hybrid screen-space + world-space:
- `bevy_ui` screen-space: Shields, Hull, Ammo, Salvage-currency, Post-run summary, Menus, Settings.
- World-space (meshes in cockpit model): Radar scope, Waypoint pointer, Economic-yield-delta indicator. Matches Design Philosophy "scientific-instrument panel over military HUD."
- Out-of-FOV target indicator (FR4 decoupled-aim ripple): `bevy_ui` edge-of-screen overlay positioned from world-space target coordinates.

**Photo-Mode Camera:** Shared `FreeOrbitCamera` component with two activation gates:
- Debug: `cfg(debug_assertions)` + F3 toggle (per E#6 Phase-3 resolution).
- Photo Mode: `in_state(GameState::PhotoMode)`.
One implementation, two registration paths.

---

### System Communication & Scheduling

**Physics Scheduling:** Avian physics runs in `FixedUpdate` at 60 Hz (fixed-step). Rendering runs at display refresh via `Update`. Interpolation of `Transform` between physics frames via Bevy's standard interpolation pattern. Rationale: XPBD determinism (required for trajectory prediction — Design Principle 3), frame-rate-independent collision stability.

**Inter-System Communication Patterns:**
- **Events** for discrete signals: `EnemyDetected`, `AsteroidDestroyed`, `HullDamaged`, `SalvageCollected`, `RunStarted`, `RunEnded`, `StateTransitionRequested`. Systems that care subscribe via `EventReader<>`.
- **Resources** for shared continuous state: `CurrentEconomy`, `ActiveCaravan`, `PlayerInputAxes`, `PhysicsConfig`, `AudioChannels`.
- **Components** for entity-bound state.
- Systems read via `Query<>`, emit via `EventWriter<>`, consume shared context via `Res<>` / `ResMut<>`. No systemwide mutable locks.

**Input Abstraction:** `leafwing-input-manager` crate (version verified at M0 start, pinned, fork-ready). Abstracts keyboard/mouse/gamepad into logical `Action` enums. Benefits: clean FR37 (mouse sensitivity) surface, gamepad support as an MVP-stretch toggle, rebinding post-MVP is a configuration change rather than a refactor.

---

### UI, Menu & Debug Architecture

**Menu System:** `bevy_ui` + States-transitions. `MainMenu` / `Settings` / `Credits` as nested states. No external menu framework (egui-based menus rejected — egui is dev-only per debug-UI decision below).

**Debug UI:** `bevy_egui` behind `cfg(debug_assertions)`. Panels for FPS / physics state / entity inspector / trigger-zone visualizer / economy-balance tuner. Stripped from release builds (zero binary-size cost). Available from M2 onward when gameplay tuning starts.

**Diagnostic / Profiling:**
- `FrameTimeDiagnosticsPlugin` + `LogDiagnosticsPlugin` from M0 (Bevy built-in, no cost).
- `tracy-client` integration from M2 per NFR-P4.
- `cargo flamegraph` as ad-hoc CPU profiling tool.
- All profiling gated behind dev-builds where feasible to keep release-binary lean.

---

### Infrastructure & Deployment

**Release Workflow (MVP / Itch.io):**
GitHub Actions release workflow triggered by version-tag push. Per-platform jobs produce ZIPs (Windows-x64, Linux-x64, macOS-universal). Itch.io upload via `butler` CLI in workflow. No manual ZIP assembly.

**macOS Code-Signing & Notarization (FR48):**
- M3 establishment: manual workflow via `codesign` + `xcrun notarytool` with App-Specific Password. 2–4 h budget per PRD.
- M6 automation: GitHub Actions secrets + notarization job in release workflow. Deferred until Steam integration work is happening anyway.

**Environment Configuration:** None required. No `.env`, no config server, no remote config. User settings (volume, mouse sensitivity) persist in the save file. Build-time constants in `const` or feature flags only where meaningful.

**Monitoring / Logging:**
- Local file logging via `tracing` + `tracing-subscriber` with `RUST_LOG` env-var support.
- Log file location: user-log-dir via `directories` crate.
- Panic hook writes stack trace to log file before exit for crash forensics.
- **No remote telemetry** (PRD E#5 commitment). No analytics SDK.

**Deferred: Steamworks Integration (M6):**
- Steamworks SDK wrapper (likely `steamworks-rs` pinned crate).
- Steam Cloud Save bridging to existing JSON save.
- Steam Input as alternative input path (`leafwing-input-manager` abstraction makes this drop-in).
- Achievements + optional leaderboards (if scoped in M6).

---

### Decision Impact Analysis

**Implementation Sequence (M0 → M3 order of introduction):**
1. M0 — Hello Bevy: Bevy 0.18 + Avian 0.6, States skeleton, `leafwing-input-manager`, `bevy_ui` splash screen.
2. M1 — Vector Spike: Custom Toon WGSL Material + `bevy_mod_outline`, three-backend validation gate.
3. M2 — Arena Tutorial: FixedUpdate physics schedule, HUD hybrid setup, `bevy_egui` debug panels, `tracy-client`.
4. M3 — Enemies alive: Save system (JSON + atomic write + `directories`), title screen + restart flow, macOS signing+notarization workflow.

**Cross-Component Dependencies:**
- `FixedUpdate` physics ↔ Rendering interpolation — must land together, not incrementally.
- Custom Toon Material ↔ `bevy_mod_outline` — both must be Bevy-0.18-compatible at M1; if outline plugin lags, fork-and-maintain-inline per PRD Tech-Risk mitigation.
- States API ↔ Menu system ↔ Save system — three-way coupling at M3 (menu triggers save-load on entry to `Caravan` from `MainMenu`).
- `leafwing-input-manager` ↔ Settings (FR37) ↔ future Steam Input — Input abstraction layer is load-bearing for M6 Steam release path.
- String-table (RON) ↔ all player-facing UI surfaces — must be adopted from M0 to satisfy NFR-L3 without later refactor.

## Implementation Patterns & Consistency Rules

### Rationale

Rust + Bevy have strong community conventions enforced by `rustfmt` + `clippy`. This section does NOT re-litigate those — it locks in project-specific patterns plus community conventions that matter for cross-session consistency (Till in one session, Claude in another, future-self months later).

---

### Naming Patterns

**Rust Identifiers (enforced by rustfmt + clippy):**
- Types, traits, enum variants: `UpperCamelCase`
- Functions, variables, modules: `snake_case`
- Constants, static: `SCREAMING_SNAKE_CASE`

**Bevy ECS Identifiers:**
- **Components:** PascalCase noun/adjective describing a property or capability. Prefer one-word when possible: `HullHP`, `ShieldHP`, `Damageable`, `Salvageable`, `Thrusters`, `TractorTarget`, `Faction`. Multi-capability components forbidden (god-struct anti-pattern).
- **Systems:** `snake_case` verb phrase describing the action: `apply_thrust_from_input`, `detect_enemy_from_sensor_range`, `bank_salvage_on_run_end`. Never nouns. Never prefixed with `system_`.
- **Events:** PascalCase **past-tense** verb describing what has already happened: `HullDamaged`, `AsteroidDestroyed`, `EnemyDetected`, `RunStarted`, `RunEnded`, `SalvageCollected`. Past tense communicates: "fires after the fact, consumers react." Commands/requests (rare) use imperative: `RequestSave`, `RequestStateTransition`.
- **Resources:** PascalCase suffix by role — `EconomyConfig` (tunable), `ActiveCaravan` (runtime state), `AudioChannels` (handle bundle), `SaveData` (persistent state).
- **Plugins:** `<Feature>Plugin` — `FlightPlugin`, `CombatPlugin`, `PerceptionPlugin`, `AudioPlugin`.
- **SystemSets:** `<Feature>Systems` enum — `enum FlightSystems { Input, Physics, PostPhysics }`.

**File & Directory Naming:**
- Source files: `snake_case.rs` (Rust convention).
- Asset directories grouped by **type** at the top level, then by **feature** inside: `assets/meshes/ship/cockpit.gltf`, `assets/audio/sfx/tractor_beam.ogg`, `assets/audio/music/`, `assets/shaders/toon.wgsl`, `assets/strings/en.ron`.
- Shader files: `.wgsl`, named by material/purpose: `toon.wgsl`, `outline.wgsl`, `shared/math.wgsl` for reusable helpers.

**String-Table Keys (RON):**
- Dot-separated scope path: `ui.menu.start_run`, `ui.hud.shields`, `ui.post_run.retry_button`, `settings.volume.master`, `settings.sensitivity.mouse`.
- Enables scoped localization and prevents collisions. Keys never change post-shipping; values may change freely.
- `en.ron` is the canonical set. Any future locale (`de.ron`) must mirror key set; missing keys fall back to English at runtime with a `warn!` log.

---

### Structure Patterns

**Module / Plugin Organization (preview — formalized in Structure step):**
Each feature module:
- Lives at `src/<feature>/mod.rs` (with sub-files for components, systems, events as needed).
- Exposes a single `<Feature>Plugin` type.
- Exposes a `SystemSet` enum (`<Feature>Systems`) for ordering.
- Declares components / resources / events locally.
- Never reaches into another feature's internals — inter-feature communication via Events or shared Resources only.

**Tests:**
- Co-located Rust convention: `#[cfg(test)] mod tests { ... }` at the bottom of each module file.
- Pure-logic modules (economy math, trajectory math, save/load serialization) are **first-class test targets**.
- Integration tests (`tests/` directory) deferred post-M3 unless a specific regression forces them.

**Constants & Tuning:**
- **Compile-time constants** (`const`) only for structural invariants (e.g., `const MAX_TRIGGER_RANGE_M: f32 = 500.0;` if it's a hard architectural limit).
- **Runtime-tunable gameplay values** live in `assets/config/tuning.ron`, loaded into a `TuningConfig` resource at startup. Hot-reloadable during development. Examples: enemy HP, shot cost, tractor beam force, salvage yield multipliers.
- Rationale: gameplay tuning happens weekly in M2–M9; recompile-per-tweak burns motivation.

---

### Format Patterns

**Error Handling:**
- **Boundary errors** (asset loading, save I/O, file system, Bevy plugin init): return `Result<T, E>` with `thiserror`-derived custom enum per module (`SaveError`, `AssetLoadError`).
- **Internal invariants** (should-never-happen): `expect("message explaining why this cannot fail")` with a comment. `unwrap()` forbidden without a comment explaining the invariant.
- **User-facing degradation** (missing save file, corrupted save, missing asset): log via `tracing::warn!` + fall back to defaults. Never crash on user-facing failure paths (per NFR-R3).

**Panic Policy:**
- **Panic OK:** programmer invariant violations that prove the engine/OS is broken (can't create GPU context, can't read `Cargo.toml`-declared asset).
- **Panic NEVER:** save-file corruption, audio device unavailable, missing texture, network timeout (doesn't exist), user input edge cases.
- Panic hook (`std::panic::set_hook`) writes stack trace to log file before exit.

**Logging:**
- `tracing` + `tracing-subscriber` with `RUST_LOG` env-var support.
- Levels:
  - `error!`: crashed-but-recovered paths, asset load failures that broke a feature.
  - `warn!`: recovered degradations (save missing → defaults used, asset missing → placeholder shown, hot-reload failed).
  - `info!`: lifecycle events (state transitions, save/load success, run started/ended, milestone-gate-adjacent signals).
  - `debug!`: gameplay diagnostics (enemy AI decision, trajectory prediction, audio event routing).
  - `trace!`: per-frame verbose, hidden by default.
- Log format: structured (`tracing` spans) rather than raw string concatenation.

**Save-File Versioning:**
- Save struct frontmatter includes `version: u32`. Load path checks version, migrates if older, refuses newer with user-facing error state.
- Version bumped when schema changes break deserialization. Pre-M3 bumps are free; post-M3 shipped bumps need migration code.

---

### Communication Patterns

**Event Conventions:**
- Past-tense naming (see Naming section).
- Event payload: named-field struct, never tuple struct. Field names use the same noun as the domain concept.
  ```rust
  #[derive(Event)]
  pub struct AsteroidDestroyed {
      pub entity: Entity,
      pub position: Vec3,
      pub salvage_awarded: u32,
      pub destroyed_by: Faction,
  }
  ```
- Events carry **all context a consumer needs** — consumers should not have to re-query ECS to reconstruct state at emission time. This is load-bearing for async/deferred consumers.
- Events are cleared per frame by Bevy — consumers must read within the same frame or register an explicit buffer.

**System Ordering:**
- **Within a plugin:** use a `SystemSet` enum variants and `.chain()` between sets. Example:
  ```rust
  enum FlightSystems { ReadInput, ApplyForces, IntegratePhysics }
  app.configure_sets(FixedUpdate, (FlightSystems::ReadInput, FlightSystems::ApplyForces, FlightSystems::IntegratePhysics).chain());
  ```
- **Cross-plugin ordering:** use shared `SystemSet` trait implementations for cross-cutting phases (e.g., `enum AppPhase { Input, Simulation, Perception, Rendering }`). Plugins register their systems to the appropriate phase.
- **Forbidden:** `.after(specific_system_function)` — breaks at function rename and is opaque at call site. Always order by SystemSet.

**State Transitions (Bevy `States`):**
- Trigger transitions by `NextState<GameState>` resource mutation. Never mutate `State<GameState>` directly.
- `OnEnter(state)` / `OnExit(state)` systems are idempotent: they run once per transition, must not assume prior state.
- State cleanup: entities spawned for a state tag themselves with a marker component (e.g., `ArenaEntity`) and are despawned by a `cleanup_on_exit::<ArenaEntity>` system in `OnExit(GameState::Arena)`.

---

### Process Patterns

**Asset Loading:**
- Typed `Resource` wrapper per asset group: `AsteroidModels { small: Handle<Scene>, medium: Handle<Scene>, ... }`, `WeaponSounds { laser_fire: Handle<AudioSource>, ... }`.
- Load in `OnEnter(GameState::Loading)` systems; await completion via `AssetServer::get_load_state()` then transition to next state.
- Forbidden: `AssetServer::load(&str)` scattered inside gameplay systems — creates load-on-first-use hitches (violates NFR-P4).

**Startup Sequencing:**
1. `Startup`: register Plugins, load `TuningConfig`, load string table, create splash-screen resource.
2. `OnEnter(GameState::Loading)`: kick off asset loads, show splash.
3. Asset-completion watcher system transitions to `GameState::MainMenu`.
4. Main menu: user input → `NextState(GameState::Arena)` for first run, else `Caravan`.

**Concurrency:**
- No `std::thread::spawn`. All background work via Bevy's `AsyncComputeTaskPool` or `IoTaskPool`.
- No shared mutable state outside `Resource` + system scheduling. No `Arc<Mutex<>>` in gameplay code.
- No `unsafe` in gameplay code. `unsafe` in shader loading or FFI only, with audit comment explaining the invariant.

---

### Enforcement Guidelines

**All AI Agents (and Till) MUST:**
- Run `cargo fmt` + `cargo clippy -- -D warnings` before committing. CI enforces.
- Follow the naming conventions above. Renames to conform require no justification; non-conforming new code requires explicit justification in commit message.
- Write components as single-responsibility. If adding a field feels wrong on an existing component, create a new component instead.
- Emit events rather than mutating cross-feature state. If tempted to reach into another plugin's resource/component, ask: "Should this be an event?"
- Prefer `Result<T, E>` + `?` over `panic!` at function boundaries.

**Pattern Deviation Process:**
- If a pattern feels wrong for a specific case, document the deviation in a `// PATTERN DEVIATION:` comment with reasoning. These get reviewed during M3 / M6 / M9 retros and either the pattern updates or the deviation reverts.

---

### Anti-Patterns (explicit red flags)

- **God-structs:** `struct Ship { hp, shield, engine, weapons, sensors, inventory }` → split into components.
- **Inheritance-shaped ECS:** `struct Enemy : Entity` pattern (Rust doesn't support it, but OO-thinkers try to simulate via nested structs). Use component composition.
- **Direct cross-plugin state mutation:** Plugin A writing into Plugin B's internal Resource/Component. Always via Events or shared well-known Resources.
- **Magic numbers in gameplay code:** `if distance < 500.0` without reference to `tuning.ron`. Exceptions: physics constants that can't be tuned (speed of light, etc. — N/A here).
- **`unwrap()` / `expect()` on user-data paths:** save files, asset loads, network input (N/A here). Always handle gracefully.
- **Scattered `AssetServer::load`:** triggers load-on-first-use hitches. Load at state-entry phase only.
- **`.after(specific_function)` for system ordering:** breaks at rename. Use SystemSet.
- **`std::thread::spawn`:** use Bevy task pools.

### Good Pattern Examples

**Good component:**
```rust
#[derive(Component)]
pub struct HullHP(pub f32);

#[derive(Component)]
pub struct ShieldHP {
    pub current: f32,
    pub max: f32,
    pub regen_rate: f32,
    pub cooldown_remaining: f32,
}
```

**Good event:**
```rust
#[derive(Event)]
pub struct HullDamaged {
    pub entity: Entity,
    pub amount: f32,
    pub source: DamageSource,
}
```

**Good system:**
```rust
fn apply_hull_damage_from_events(
    mut events: EventReader<HullDamaged>,
    mut hulls: Query<&mut HullHP>,
) {
    for event in events.read() {
        if let Ok(mut hp) = hulls.get_mut(event.entity) {
            hp.0 = (hp.0 - event.amount).max(0.0);
        }
    }
}
```

**Good SystemSet use:**
```rust
#[derive(SystemSet, Debug, Clone, Hash, PartialEq, Eq)]
pub enum CombatSystems { EvaluateHits, ApplyDamage, CheckDeath }

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(FixedUpdate, (
            CombatSystems::EvaluateHits,
            CombatSystems::ApplyDamage,
            CombatSystems::CheckDeath,
        ).chain());
        app.add_systems(FixedUpdate, (
            evaluate_projectile_hits.in_set(CombatSystems::EvaluateHits),
            apply_hull_damage_from_events.in_set(CombatSystems::ApplyDamage),
            despawn_hull_zero_entities.in_set(CombatSystems::CheckDeath),
        ));
    }
}
```

## Project Structure & Boundaries

### Complete Project Directory Structure

```
asteriods3d/
├── Cargo.toml                         # pinned Bevy 0.18 + Avian 0.6 + plugins
├── Cargo.lock                         # committed (binary project convention)
├── rust-toolchain.toml                # pins stable toolchain for CI reproducibility
├── rustfmt.toml                       # project style
├── clippy.toml                        # project lint config
├── README.md                          # build/run instructions, link to architecture.md
├── .gitignore                         # Rust + Bevy asset cache conventions
├── .github/
│   └── workflows/
│       ├── ci.yml                     # Windows + Linux + macOS: build, test, clippy, fmt
│       └── release.yml                # tagged release → per-OS ZIPs + butler (Itch.io)
├── src/
│   ├── main.rs                        # App::new() assembly, register all plugins
│   ├── state.rs                       # GameState enum + phase SystemSets (AppPhase)
│   ├── core/                          # shared types across plugins
│   │   ├── mod.rs
│   │   ├── faction.rs                 # enum Faction { Player, Enemy, Neutral, Salvageable }
│   │   ├── damage.rs                  # DamageSource enum, damage-related shared types
│   │   └── markers.rs                 # state-scoped entity markers (ArenaEntity, CaravanEntity)
│   ├── tuning/                        # hot-reloadable gameplay tuning
│   │   ├── mod.rs                     # TuningPlugin
│   │   └── config.rs                  # TuningConfig resource struct + RON deserializer
│   ├── flight/                        # FR1–FR8: Flight & Controls
│   │   ├── mod.rs                     # FlightPlugin, FlightSystems SystemSet
│   │   ├── components.rs              # Thrusters, InertialDampener, Boost, TractorEmitter
│   │   ├── input.rs                   # leafwing Action enum + input-to-force systems
│   │   ├── physics.rs                 # Newtonian integration + dampener toggle
│   │   └── camera.rs                  # cockpit-view Camera3d setup + decoupled-aim reticle
│   ├── combat/                        # FR9–FR16: Combat
│   │   ├── mod.rs                     # CombatPlugin, CombatSystems SystemSet
│   │   ├── components.rs              # HullHP, ShieldHP, Weapon, ProjectileLifetime
│   │   ├── weapons.rs                 # 3 prefab weapon archetypes, firing systems
│   │   ├── projectiles.rs             # projectile spawn, ballistics, collision
│   │   ├── damage.rs                  # HullDamaged/ShieldDamaged event handlers
│   │   └── enemy_ai.rs                # 1 enemy type: detect/pursue/attack state machine
│   ├── salvage/                       # FR17–FR21: Economy & Salvage
│   │   ├── mod.rs                     # SalvagePlugin, SalvageSystems SystemSet
│   │   ├── components.rs              # Salvageable, TractorTarget, YieldValue
│   │   ├── tractor.rs                 # tractor-beam constraint/force system
│   │   ├── economy.rs                 # shot-cost deduction, yield math, currency resource
│   │   └── unlocks.rs                 # meta-currency shop UI hook + PersistentMeta resource
│   ├── perception/                    # FR22–FR26: Perception & Sensors
│   │   ├── mod.rs                     # PerceptionPlugin
│   │   ├── sensor.rs                  # radar entity-range detection, threat markers
│   │   ├── audio_cues.rs              # EnemyDetected/HazardDetected → spatial audio events
│   │   └── threat_events.rs           # cross-plugin Event definitions
│   ├── run/                           # FR27–FR35: Run Structure & Progression
│   │   ├── mod.rs                     # RunPlugin
│   │   ├── arena.rs                   # hand-designed tutorial zone, one-shot
│   │   ├── caravan.rs                 # run director: skeleton instantiation, waypoint target
│   │   ├── pockets.rs                 # rendering-distance combat-pocket triggers
│   │   ├── waypoint.rs                # waypoint-pointer HUD system (feeds into ui/)
│   │   └── director.rs                # Run lifecycle: RunStarted/RunEnded events
│   ├── ui/                            # FR36–FR43: UI & Feedback
│   │   ├── mod.rs                     # UiPlugin
│   │   ├── main_menu.rs               # title screen + menu flow
│   │   ├── hud.rs                     # bevy_ui screen-space: shields, hull, ammo, salvage
│   │   ├── hud_cockpit.rs             # world-space instruments: radar, waypoint, yield delta
│   │   ├── settings.rs                # volume, sensitivity (FR37) + rebinding (post-MVP)
│   │   ├── post_run.rs                # FR38: death summary, restart/menu/photo buttons
│   │   ├── photo_mode.rs              # FR40–FR42: free-cam, DoF, PNG export, aspect presets
│   │   └── strings.rs                 # RON string-table loader + lookup helper
│   ├── persistence/                   # FR44–FR48: Persistence & Platform
│   │   ├── mod.rs                     # PersistencePlugin
│   │   ├── save.rs                    # atomic write (temp + rename), load, migration
│   │   ├── schema.rs                  # SaveData struct (versioned), Serde derives
│   │   └── paths.rs                   # directories-crate wrapper, per-OS path resolution
│   ├── visual/                        # FR49–FR50: Visual Presentation
│   │   ├── mod.rs                     # VisualPlugin
│   │   ├── toon_material.rs           # custom WGSL Material impl (M1 learning target)
│   │   ├── outline.rs                 # bevy_mod_outline integration + fallback switch
│   │   └── palette.rs                 # SemanticAccent enum → color lookup
│   ├── audio/                         # Audio architecture (NFR-U, R#6)
│   │   ├── mod.rs                     # AudioPlugin
│   │   ├── channels.rs                # bevy_kira_audio channels: sfx, music, ambient
│   │   └── spatial.rs                 # stereo positioning MVP, HRTF post-MVP hook
│   └── debug/                         # dev-only (cfg(debug_assertions))
│       ├── mod.rs                     # DebugPlugin (compiled out of release)
│       ├── egui_panels.rs             # bevy_egui panels: physics state, entity inspector
│       └── free_camera.rs             # F3 debug 3rd-person camera
├── assets/
│   ├── config/
│   │   └── tuning.ron                 # gameplay tunables, hot-reloadable
│   ├── strings/
│   │   └── en.ron                     # canonical English string table (NFR-L3)
│   ├── meshes/
│   │   ├── ship/
│   │   │   └── cockpit.gltf           # sole player cockpit
│   │   ├── asteroid/                  # 3-5 asteroid variants
│   │   ├── enemy/                     # 1 enemy ship MVP
│   │   └── projectile/                # weapon visuals
│   ├── audio/
│   │   ├── sfx/                       # weapon fire, impact, tractor, death, UI clicks
│   │   ├── music/                     # ambient-drone layers (per Design Philosophy)
│   │   └── ambient/                   # environmental beds (cockpit hum, space hush)
│   └── shaders/
│       ├── toon.wgsl                  # hand-authored custom material (M1)
│       ├── outline.wgsl               # only if extending/replacing bevy_mod_outline
│       └── shared/
│           └── math.wgsl              # reusable shader helpers
├── tests/                             # integration tests (deferred post-M3)
│   └── .gitkeep
└── target/                            # cargo build output, gitignored
```

### Architectural Boundaries

**Plugin Boundaries (each is a `Plugin` impl):**

| Plugin | Owns | Publishes (Events/Resources) | Consumes |
|---|---|---|---|
| `FlightPlugin` | Thrusters, dampener, cockpit Camera3d | `ShipTransformUpdated`, `BoostActivated` | `PlayerInputAxes` (Res), `TuningConfig` (Res) |
| `CombatPlugin` | HullHP, ShieldHP, Weapon, projectiles, enemy AI | `HullDamaged`, `ShieldDamaged`, `AsteroidDestroyed`, `EnemyDestroyed`, `HullDepleted` | `TuningConfig`, weapon-fire intents from `flight/input` |
| `SalvagePlugin` | Salvageable, TractorEmitter, currency resource | `SalvageCollected`, `WeaponFired` (debits currency) | `AsteroidDestroyed`, `AsteroidCaptured` events |
| `PerceptionPlugin` | Sensor range, threat detection | `EnemyDetected`, `HazardDetected` | entity positions (Query), `Faction` |
| `RunPlugin` | Caravan/Arena director, pocket triggers, waypoint | `RunStarted`, `RunEnded`, `PocketEntered` | `HullDepleted` (triggers RunEnded) |
| `UiPlugin` | bevy_ui nodes, world-space HUD meshes, photo-mode camera | `PhotoModeRequested`, `RestartRequested`, `MenuTransitionRequested` | `SalvageCurrency`, `HullHP`, `ShieldHP` queries, all lifecycle events |
| `PersistencePlugin` | SaveData resource, save service | `SaveCompleted`, `LoadCompleted`, `SaveFailed` | `RunEnded` (triggers save), `UnlockPurchased` |
| `VisualPlugin` | Toon `Material` asset, outline plugin wiring, palette | — | `SemanticAccent` component on any rendered entity |
| `AudioPlugin` | bevy_kira_audio channels, spatial source pool | — | `EnemyDetected`, `AsteroidDestroyed`, `HullDamaged`, `WeaponFired` |
| `TuningPlugin` | `TuningConfig` Resource, hot-reload watcher | `TuningReloaded` | reads `tuning.ron` asset |

**Rule:** Plugin A never writes into Plugin B's internal Resources/Components. All cross-plugin effects flow through Events or shared well-known Resources (listed as "Consumes" above).

**Cross-Cutting Resources (registered in `main.rs`, read by many plugins):**
- `PlayerInputAxes` — populated by flight/input, read by flight systems and anything else needing input context.
- `SalvageCurrency` — mutated by salvage, read by UI + combat (shot cost check).
- `TuningConfig` — read-only for gameplay plugins, written only by `TuningPlugin` on reload.
- `State<GameState>` — read by any plugin that cares about run phase.
- `SaveData` — owned by PersistencePlugin, read by UI (unlock shop), salvage (currency state).

### Requirements to Structure Mapping

**FR Mapping (detailed):**

| FR(s) | Location |
|---|---|
| FR1 keyboard+mouse input | `src/flight/input.rs` (leafwing Action enum) |
| FR2 6-direction translation | `src/flight/physics.rs` (thrust vector application) |
| FR3 3-axis rotation | `src/flight/physics.rs` (angular velocity application) |
| FR4 decoupled aim | `src/flight/camera.rs` + `src/ui/hud.rs` (reticle + out-of-FOV indicator) |
| FR5 inertial dampener | `src/flight/physics.rs` (toggleable damping coefficient) |
| FR6 boost | `src/flight/components.rs` (Boost) + `src/flight/physics.rs` |
| FR7 tractor beam | `src/salvage/tractor.rs` (Avian constraint/impulse) |
| FR8 cockpit-only camera | `src/flight/camera.rs` (no alternative camera in GameState::Arena/Caravan) |
| FR9–FR10 weapons | `src/combat/weapons.rs` (3 archetypes, up-to-3 equipped) |
| FR11 pay-to-shoot | `src/salvage/economy.rs` (debit on `WeaponFired` event) |
| FR12 projectile damage | `src/combat/projectiles.rs` + `src/combat/damage.rs` |
| FR13 intact > destroyed yield | `src/salvage/economy.rs` (yield math from `AsteroidDestroyed` vs `AsteroidCaptured`) |
| FR14 enemy AI | `src/combat/enemy_ai.rs` |
| FR15 Hull + Shields | `src/combat/components.rs` + `src/combat/damage.rs` |
| FR16 permadeath | `src/run/director.rs` (`HullDepleted` → `RunEnded(cause: Death)`) |
| FR17–FR21 economy + meta + unlocks | `src/salvage/economy.rs` + `src/salvage/unlocks.rs` + `src/persistence/schema.rs` |
| FR22 radar | `src/perception/sensor.rs` + `src/ui/hud_cockpit.rs` (world-space radar mesh) |
| FR23 spatial audio | `src/perception/audio_cues.rs` + `src/audio/spatial.rs` |
| FR24 HUD state display | `src/ui/hud.rs` |
| FR25 yield delta indicator | `src/ui/hud_cockpit.rs` (world-space indicator on visible salvageable targets) |
| FR26 headphone splash | `src/ui/main_menu.rs` (first-launch splash) |
| FR27 Arena tutorial | `src/run/arena.rs` |
| FR28 no tutorial text | enforced by pattern deviation process — no `tutorial_text_*` strings in `en.ron` |
| FR29 Arena→Caravan transition | `src/run/director.rs` + `src/state.rs` (state transitions) |
| FR30 5–8 min Caravan | `src/run/caravan.rs` (duration parameter in TuningConfig) |
| FR31 3 difficulties | `src/run/caravan.rs` + `src/ui/main_menu.rs` (difficulty picker) |
| FR32 combat pockets | `src/run/pockets.rs` |
| FR33 waypoint pointer | `src/run/waypoint.rs` + `src/ui/hud_cockpit.rs` |
| FR34–FR35 salvage banking | `src/run/director.rs` (`RunEnded` → bank → save) |
| FR36 title screen | `src/ui/main_menu.rs` |
| FR37 settings | `src/ui/settings.rs` |
| FR38 post-run summary | `src/ui/post_run.rs` |
| FR39 restart without menu | `src/ui/post_run.rs` (direct `NextState(Caravan)`) |
| FR40–FR42 photo mode | `src/ui/photo_mode.rs` |
| FR43 pause on focus loss | `src/main.rs` (window-focus event handler) |
| FR44–FR46 save system | `src/persistence/save.rs` + `src/persistence/schema.rs` + `src/persistence/paths.rs` |
| FR47–FR48 cross-platform + macOS notarization | CI workflows (`.github/workflows/release.yml`) + build config (`Cargo.toml` release profile) |
| FR49 toon shader | `src/visual/toon_material.rs` + `assets/shaders/toon.wgsl` |
| FR50 semantic accent colors | `src/visual/palette.rs` |

**NFR Cross-Cutting Concerns:**

| NFR | Enforcement Location |
|---|---|
| NFR-P1 60 FPS target | CI smoke-test + `tracy` profiling from M2 in `src/debug/` |
| NFR-P4 no hitches > 100 ms | Asset-load-at-state-entry pattern (enforced in every `OnEnter(...)` system) |
| NFR-R2 atomic saves | `src/persistence/save.rs` (temp + rename implementation) |
| NFR-R3 graceful missing save | `src/persistence/save.rs` (first-launch default creation) |
| NFR-A1/A2 colorblind redundancy | `src/visual/palette.rs` (semantic accents + shape/position/audio redundancy) |
| NFR-L3 externalized strings | `src/ui/strings.rs` + `assets/strings/en.ron` |

### Integration Points

**Internal Communication (Event flow):**

```
PlayerInput
  → flight::input → PlayerInputAxes (Res)
  → flight::physics → Transform (Component)
  → Avian FixedUpdate → collisions → combat::projectiles
  → combat::damage → HullDamaged / ShieldDamaged (Event)
  → combat::death → HullDepleted (Event)
  → run::director → RunEnded (Event)
  → salvage::economy → SalvageCollected (Event, accumulated into PersistentMeta)
  → persistence::save → SaveCompleted (Event)
  → ui::post_run → display summary
```

**External Integrations (MVP):**
- File system via `directories` crate — save + log paths.
- Window system via wgpu — Bevy-managed, no direct calls.
- No network. No telemetry. No cloud services.

**External Integrations (Post-MVP / M6):**
- Steamworks SDK (future `src/steam/` plugin): auto-update, Steam Cloud Save bridge, Steam Input.

**Data Flow (save/load):**
- Save write: any `SaveRequested` event → `PersistencePlugin` serializes `SaveData` (versioned) → temp-file write → fsync → atomic rename → `SaveCompleted` event.
- Save load: `Startup` → `PersistencePlugin` reads file (or creates default), deserializes, checks version, migrates if needed → populates `SaveData` Resource.

### File Organization Patterns

**Configuration Files:** Flat at project root (Rust convention). No nested `config/` directory for tooling (simpler is better).

**Source Organization:** One directory per feature plugin under `src/`. No "by-type" grouping (no `src/components/`, `src/systems/`, `src/resources/`) — co-location by feature is the Bevy-idiomatic structure and prevents refactor-pain when features move.

**Test Organization:** Co-located `#[cfg(test)] mod tests` inside each `*.rs` file. Integration tests under `tests/` deferred until a specific regression needs a multi-plugin test harness.

**Asset Organization:** Top-level by asset type (meshes / audio / shaders / strings / config), then feature-scoped inside. Rationale: asset pipeline (Blender → glTF) operates per-type, and per-type top-level makes CI asset-validation straightforward.

### Development Workflow Integration

**Development Server Structure:** `cargo run` from project root. `bevy_asset` hot-reloads textures, shaders, and audio on file-save during development. Code changes require recompile (hot-code-reload via `subsecond` evaluated later, not MVP).

**Build Process Structure:**
- Dev builds: `cargo run` → assets loaded from `./assets/` relative to cwd.
- Release builds: `cargo build --release` → binary + `assets/` folder ship together. Release profile from Starter decision (LTO fat, codegen-units 1, opt-level 3).

**Deployment Structure:**
- MVP (Itch.io): per-OS ZIP containing binary + `assets/` sibling folder. macOS: signed + notarized `.app` bundle inside ZIP. GitHub Actions `release.yml` produces all three on tag push.
- M6 (Steam): Steamworks SDK integration adds Steam-branch upload step to same release workflow. Binary structure unchanged.

### Boundaries the Architecture Explicitly Rejects

These are documented as anti-decisions — future contributors (human or AI) should resist proposing them without a strong brainstorming-level justification:

- **No global mutable singletons** outside `Resource`. No `static mut`, no `OnceCell<Mutex<T>>` hand-rolled patterns.
- **No "god plugin"** that owns multiple FR clusters. If a concept crosses clusters (e.g., economy touches combat+salvage), it lives in the cluster that owns the primary state and communicates via events.
- **No feature flags for gameplay** (e.g., `#[cfg(feature = "caravan")]`). All MVP scope is always compiled. Feature flags reserved for platform-specific code paths and dev-vs-release (`cfg(debug_assertions)`).
- **No dynamic loading of Rust code**. All gameplay systems are compiled in. Rhai scripting is Vision-stage, not MVP.
- **No separate client/server modules**. Offline single-player. If Steam P2P is ever added post-MVP, it will be its own opt-in plugin behind a feature flag — never the default structure.

## Architecture Validation Results

### Coherence Validation ✅

**Decision Compatibility:**
- Bevy 0.18 + Avian 0.6: confirmed compatible per April 2026 web verification.
- FixedUpdate physics schedule + Avian XPBD deterministic stepping: aligned (XPBD requires fixed timestep for stability).
- `leafwing-input-manager` + Bevy States: both Bevy-0.18-compatible per current crate release notes (final pin verification is an M0-start task, see Gap Analysis).
- `bevy_ui` (screen-space HUD) + custom world-space cockpit instruments + `bevy_egui` (dev-only debug): three UI systems coexist without overlap because each has a disjoint render layer + disjoint activation gate.
- RON (strings, tuning) + JSON (save): three-format choice is scoped cleanly — no format decides anything the others touch.
- Component-composition-first principle consistent across all 10 feature plugins (no god-structs in component catalogue).

**Pattern Consistency:**
- Naming conventions applied uniformly in the Structure step's FR-mapping and plugin table (HullHP, AsteroidDestroyed, FlightPlugin, FlightSystems).
- Event-driven cross-plugin communication applied consistently — zero cases in the Plugin Boundaries table where Plugin A reads Plugin B's internal state directly.
- SystemSet ordering discipline applied per plugin (each plugin owns a `<Feature>Systems` enum).
- Asset-load-at-state-entry pattern consistently applied (no scattered `AssetServer::load` calls in any feature module).

**Structure Alignment:**
- 10 feature plugins + 2 support modules (core, tuning) + 1 dev-only module (debug) = 13 logical modules, within the 12-15 range estimated in Step 2.
- Every FR has an explicit location in the FR-mapping table.
- Every NFR has an enforcement location.
- Integration flow (save/load, event chain from input to UI feedback) has a documented path.

### Requirements Coverage Validation ✅

**Functional Requirements Coverage:** 50/50 (100%)

| FR Cluster | Count | Architecture Home | Status |
|---|---|---|---|
| Flight & Controls (FR1–FR8) | 8 | `src/flight/` | ✓ All 8 mapped |
| Combat (FR9–FR16) | 8 | `src/combat/` + run/director for FR16 | ✓ All 8 mapped |
| Economy & Salvage (FR17–FR21) | 5 | `src/salvage/` + `src/persistence/` for FR19 | ✓ All 5 mapped |
| Perception & Sensors (FR22–FR26) | 5 | `src/perception/` + `src/ui/hud_cockpit.rs` + `src/audio/` | ✓ All 5 mapped |
| Run Structure & Progression (FR27–FR35) | 9 | `src/run/` + `src/ui/` for FR33 overlay | ✓ All 9 mapped |
| UI & Feedback (FR36–FR43) | 8 | `src/ui/` + `src/main.rs` for FR43 | ✓ All 8 mapped |
| Persistence & Platform (FR44–FR48) | 5 | `src/persistence/` + CI workflows | ✓ All 5 mapped |
| Visual Presentation (FR49–FR50) | 2 | `src/visual/` | ✓ All 2 mapped |

**Non-Functional Requirements Coverage:** 18/18 (100% mapped or architecturally supported)

| NFR | Coverage Mechanism |
|---|---|
| NFR-P1 60 FPS @ 1080p | `tracy` profiling (from M2) + milestone-gate playtest (per PRD). CI runs build+test; GPU-FPS verification is not CI-enforced — noted in Gap Analysis. |
| NFR-P2 load ≤ 10 s | Bevy asset-server + state-entry loading phase; monitored by playtest. |
| NFR-P3 menu→gameplay ≤ 5 s | Same as NFR-P2. |
| NFR-P4 no hitches > 100 ms | Asset-load-at-state-entry pattern (forbids scattered `AssetServer::load`). |
| NFR-P5 < 4 GB process memory | Bevy feature pruning (default-features=false, 3d-only), low-res textures, glTF Draco compression. Monitored via debug panels. |
| NFR-R1 zero-crash | Panic policy + graceful degradation patterns. |
| NFR-R2 atomic saves | `src/persistence/save.rs` temp-file + rename. |
| NFR-R3 graceful missing save | First-launch default creation in `src/persistence/save.rs`. |
| NFR-R4 no between-run meta loss | Save triggered on `RunEnded` event, atomic, before state transition. |
| NFR-A1 colorblind-distinguishable | `src/visual/palette.rs` semantic accents + redundant encoding (shape/position/audio). |
| NFR-A2 no color-only information | Enforced by palette.rs design + redundant encoding requirement. |
| NFR-A3 HUD legibility at 60–80 cm | UI-level design concern; architecture supports (bevy_ui flexbox), playtest-verified. |
| NFR-U1 aha in 5 min | Architecture supports via yield-delta indicator (FR25 → `src/ui/hud_cockpit.rs`); playtest-verified at M3 gate. |
| NFR-U2 HUD simultaneously visible | `src/ui/hud.rs` single-frame render of all critical elements. |
| NFR-U3 at-a-glance subsystems | `src/ui/hud_cockpit.rs` world-space instrument panel. |
| NFR-L1 English MVP | `assets/strings/en.ron` canonical. |
| NFR-L2 German post-MVP | `assets/strings/de.ron` drop-in, zero code change. |
| NFR-L3 externalized strings | `src/ui/strings.rs` RON loader, no hard-coded UI strings. |

**Design Principles Coverage:** 5/5

| Principle | Architectural Support |
|---|---|
| 1. No tutorial text | Enforced by `assets/strings/en.ron` review — no `tutorial_*` keys allowed; pattern-deviation process catches violations. |
| 2. No visible numeric score | Enforced by HUD design in `src/ui/hud.rs` — economy/currency/unlocks replace score. No `Score` component or `ScoreResource` exists in the architecture. |
| 3. Asteroid motion predictable | Avian XPBD FixedUpdate + deterministic trajectory parameters (Kepler/spline) in `TuningConfig`. |
| 4. Death as feedback, not punishment | `src/ui/post_run.rs` FR38 — no "GAME OVER" overlay, no red screen; framing in string table. |
| 5. Graceful degradation at novel points | Fallback paths documented: toon-shader → flat-shaded (M1), audio-first → sensor-UI-primary (R#6), cockpit-only → accepted trade-off. |

### Implementation Readiness Validation ✅

**Decision Completeness:**
- Critical decisions documented: save atomicity, state machine, physics scheduling, shader org, UI framework, input abstraction. All 6 resolved with versions where applicable.
- Important decisions documented: string format, HUD strategy, debug tooling, macOS signing. All 4 resolved.
- Deferred decisions clearly marked as Post-MVP with stage assignment (M6, Vision).

**Structure Completeness:**
- Full directory tree documented (25 source files + 7 asset subdirs + 2 CI workflows).
- Plugin boundary table names Owns / Publishes / Consumes for all 10 plugins.
- Cross-cutting Resources enumerated.
- Integration flow diagrammed.

**Pattern Completeness:**
- Naming: 5 categories covered (Rust idents, Bevy ECS, files/dirs, string-table keys, assets).
- Structure: module layout, tests, constants/tuning all covered.
- Format: error handling, panic, logging, save versioning covered.
- Communication: events, system ordering, state transitions covered.
- Process: asset loading, startup, concurrency covered.
- Anti-patterns explicit.
- 4 concrete Good-Example code blocks provided.

### Gap Analysis Results

**Critical Gaps:** 0

**Important Gaps (tracked, not blocking):**

1. **Third-party crate Bevy-0.18 compatibility verification.** `bevy_mod_outline`, `bevy_kira_audio`, `bevy_egui`, `leafwing-input-manager` are all flagged for compatibility verification at M0 start. Mitigation: fork-and-maintain-inline path documented per PRD Tech-Risk strategy. **Resolution:** The first story of M0 must be "verify + pin plugin versions, fork-and-patch if required, commit `Cargo.lock`." Everything else waits on this gate.

2. **M1 tech-spike fallback implementation scaffolding.** If custom WGSL toon shader + `bevy_mod_outline` underwhelms at M1, PRD mandates fallback to flat-shaded + rim-light. Architecture has `src/visual/outline.rs` with a documented "fallback switch" comment, but the fallback path isn't scaffolded. **Resolution:** At M1 tech-spike completion gate, if decision is "go with toon," proceed as-designed. If decision is "fallback," replace `toon_material.rs` contents with a flat-shaded + rim-light implementation and disable `outline.rs` plugin registration. Fallback scope is not pre-built (YAGNI until the gate); M1 tech-spike evaluation itself is the trigger.

3. **CI performance enforcement limitation.** NFR-P1 60 FPS target cannot be reliably enforced by GitHub Actions (runners have inconsistent GPU access). **Resolution:** CI enforces build + test + clippy + fmt only. 60-FPS verification is a milestone-gate playtest on Till's reference hardware (GTX 1060 / RX 580 / Apple M1). Documented here so it doesn't appear as unmet requirement.

**Nice-to-Have Gaps (optional polish):**

- `rust-toolchain.toml` version not specified — defer to "latest stable at M0 start."
- `Cargo.toml` feature string for Linux windowing not specified (x11 vs wayland vs both) — defer to Linux-build verification at M0.
- Specific `bevy_mod_outline` / `bevy_kira_audio` versions not pinned — intentional; first M0 task verifies and pins.

### Validation Issues Addressed

No critical issues required resolution. Three advisory gaps converted into explicit M0-start tasks and documented M1-gate conditionals.

### Architecture Completeness Checklist

**✅ Requirements Analysis**
- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed (medium-high, 10-14 month solo horizon)
- [x] Technical constraints identified (pinned stack, 4-8h/week budget, motivation-preservation)
- [x] Cross-cutting concerns mapped (9 concerns documented)

**✅ Architectural Decisions**
- [x] Critical decisions documented with versions (6 critical, all resolved)
- [x] Technology stack fully specified (Bevy 0.18, Avian 0.6, plugin pins pending M0 verification)
- [x] Integration patterns defined (event-driven cross-plugin, FixedUpdate physics)
- [x] Performance considerations addressed (60 FPS, <4GB memory, no hitches > 100ms)

**✅ Implementation Patterns**
- [x] Naming conventions established (5 categories)
- [x] Structure patterns defined (per-feature module layout, co-located tests)
- [x] Communication patterns specified (events, resources, SystemSets)
- [x] Process patterns documented (error handling, asset loading, startup, concurrency)

**✅ Project Structure**
- [x] Complete directory structure defined (~25 source files + full asset tree)
- [x] Component boundaries established (plugin table: Owns / Publishes / Consumes)
- [x] Integration points mapped (event flow diagram)
- [x] Requirements to structure mapping complete (50/50 FRs, 18/18 NFRs, 5/5 Design Principles)

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION

**Confidence Level:** HIGH

Rationale for HIGH confidence:
- Stack decisions pre-resolved by brainstorming and validated against current (April 2026) ecosystem state.
- PRD validation passed at 5/5 Excellent — the requirements this architecture covers are themselves well-formed.
- 100% requirements coverage with explicit file-level mapping.
- No critical gaps; all advisory gaps converted into concrete M0/M1 tasks.
- Solo-dev + beginner-learning constraints reflected in choices (hybrid-manual starter, bevy_egui for dev-only debug, pattern discipline favoring idiomatic Bevy over custom abstractions).

**Key Strengths:**
1. **Tight traceability from concept to file.** Every feature has a documented FR → plugin → file path.
2. **Learning-goal alignment.** Custom Toon WGSL Material is authored by Till (portfolio-quality artifact); infrastructure (CI, gitignore) is adopted boilerplate. High learning-ROI split.
3. **Motivation-preservation awareness.** M5 slicing note, hot-reloadable tuning config, staged decisions all explicitly designed to avoid invisible stretches.
4. **Event-driven discipline.** Plugin boundaries enforced via event interfaces — prevents cross-module spaghetti that would be particularly painful for a solo dev learning ECS.
5. **Pattern-deviation process.** Documented escape hatch prevents pattern discipline from becoming dogma.

**Areas for Future Enhancement (post-M3 / post-M6):**
- Steamworks SDK integration plan (M6): scaffolding for `src/steam/` plugin with Cloud Save bridge.
- HRTF spatial audio deepening (R#6 stage 2): architecture reserves the extension point in `src/audio/spatial.rs`.
- Rhai modding scripting (Vision): would introduce a `src/scripting/` module; architecture's plugin boundary pattern should extend naturally.
- Integration test harness: post-M3 when multi-plugin regressions first warrant dedicated test infrastructure.

### Implementation Handoff

**AI Agent & Human Guidelines:**
- Follow architectural decisions exactly as documented. Deviations require `// PATTERN DEVIATION:` comment with reasoning, reviewed at milestone retros.
- Use the naming conventions, system-ordering, and event-driven communication patterns without shortcut.
- Respect plugin boundaries — never reach into another plugin's internal state; use Events.
- Refer to this architecture document for structural questions; refer to the PRD for scope/intent questions; refer to the brainstorming doc for concept-level rationale.

**First Implementation Priority (M0 start):**

The first story of M0 ("Hello Bevy") is NOT writing gameplay code. It is verifying the plugin compatibility matrix:

```bash
# Story M0.1 — Plugin compatibility verification
cargo new --bin asteriods3d
cd asteriods3d

# Author Cargo.toml by hand:
#   bevy = { version = "0.18", default-features = false, features = ["3d", "png", "x11"] }
#   avian3d = "0.6"
#   bevy_mod_outline = "<latest Bevy-0.18-compatible>"
#   bevy_kira_audio = "<latest Bevy-0.18-compatible>"
#   leafwing-input-manager = "<latest Bevy-0.18-compatible>"
#   [target.'cfg(debug_assertions)'.dependencies]
#   bevy_egui = "<latest Bevy-0.18-compatible>"
#   + serde, serde_json, ron, thiserror, tracing, tracing-subscriber, directories

# Verify compilation on all three platforms:
cargo check  # local first

# If any plugin lacks a Bevy-0.18-compatible release: evaluate fork-and-inline per PRD Tech-Risk strategy.
# Commit Cargo.lock.
```

Only after the compatibility gate passes does actual M0 scaffolding begin:
- `main.rs` with `App::new().add_plugins(DefaultPlugins).run()`.
- `state.rs` with `GameState` enum + `Startup` → `Loading` → `MainMenu` scaffolding.
- Single splash-screen `bevy_ui` Node.
- `.github/workflows/ci.yml` (borrowed from NiklasEi's template, stripped).
- `rustfmt.toml`, `clippy.toml`, `.gitignore`.

M0 completion criterion: `cargo run` opens a window showing "asteriods3D" on all three platforms, with CI passing.
