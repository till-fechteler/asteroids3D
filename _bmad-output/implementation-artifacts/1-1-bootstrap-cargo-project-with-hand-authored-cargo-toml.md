# Story 1.1: Bootstrap Cargo Project with Hand-Authored Cargo.toml

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want the project directory initialized with a hand-authored `Cargo.toml` containing all pinned dependencies,
So that every dependency is committed and reproducible from day one, and I internalize the Bevy setup rather than inheriting it from a template.

## Acceptance Criteria

1. **Cargo bootstrap exists.** `src/main.rs` and `Cargo.toml` are present at the project root (produced by `cargo new --bin`).
2. **Hand-authored `Cargo.toml`** replaces the default with these pinned dependencies:
   - `bevy = { version = "0.18", default-features = false, features = ["3d", "png", <platform-windowing>] }` — Linux adds `"x11"` and/or `"wayland"`; Windows/macOS need no extra windowing features.
   - `avian3d = "0.6"`
   - `bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager` pinned at the latest Bevy-0.18-compatible versions (see **Dev Notes → Plugin Version Resolution**).
   - `bevy_egui` declared under `[target.'cfg(debug_assertions)'.dependencies]` (dev-only; excluded from release builds).
   - `serde`, `serde_json`, `ron`, `thiserror`, `tracing`, `tracing-subscriber`, `directories` pinned to current stable versions.
3. **Release profile** sets `lto = "fat"`, `codegen-units = 1`, `opt-level = 3`.
4. **Dev profile** sets `opt-level = 1` for dependencies (Bevy dev-speed convention). Workspace code `opt-level` stays default (`0`).
5. **Resolution succeeds.** `cargo check` completes without errors on the local machine.
6. **`Cargo.lock` is committed** to the repository (binary-project convention). [Source: architecture.md:537,985]

## Tasks / Subtasks

- [x] **Task 1 — Verify bootstrap preconditions (AC: #1)**
  - [x] Confirm `Cargo.toml` and `src/main.rs` exist at `/Users/tillfechteler/Projekte/rust/asteriods3D/`.
  - [x] Current state check: `cargo new --bin` was already run (package name `asteriods3D`, edition `2024`, empty `[dependencies]`). No additional bootstrap step needed. See **Project Structure Notes → Package name casing** below.
- [x] **Task 2 — Resolve plugin versions against Bevy 0.18 (AC: #2)**
  - [x] Query crates.io (or `cargo search`) for the latest `bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager`, `bevy_egui` releases and confirm each declares a Bevy `0.18` dependency.
  - [x] If any plugin lags (no 0.18-compatible release), do **not** fork yet — record the gap as a finding for Story 1.2 (Plugin Compatibility Verification Gate). Pin the closest release and let 1.2 own the fork-or-substitute decision. [Source: architecture.md:56-70,401-403]
  - [x] Capture the exact resolved versions in the story's **File List** for Story 1.2 to reference.
- [x] **Task 3 — Author `Cargo.toml` (AC: #2, #3, #4)**
  - [x] Replace `/Users/tillfechteler/Projekte/rust/asteriods3D/Cargo.toml` contents with the hand-authored manifest (template in **Dev Notes → Cargo.toml Skeleton**).
  - [x] Include `[dependencies]`, `[target.'cfg(debug_assertions)'.dependencies]`, `[profile.release]`, and `[profile.dev.package."*"]` sections.
  - [x] Keep Linux windowing features behind `[target.'cfg(target_os = "linux")'.dependencies]` so Windows/macOS builds do not pull unnecessary windowing crates. [Source: architecture.md:132-133]
- [x] **Task 4 — Resolve and commit lockfile (AC: #5, #6)**
  - [x] Run `cargo check` at the project root.
  - [x] If resolution fails, diagnose version conflicts and adjust pins. Document any pin adjustments in **Completion Notes**.
  - [x] Confirm `Cargo.lock` was generated and is tracked (not in `.gitignore`). Note: `.gitignore` is authored in Story 1.3 — for now just ensure `Cargo.lock` is committed. [Source: architecture.md:537; requirements-inventory.md:110]
- [x] **Task 5 — Scope guardrails (what this story does NOT do)**
  - [x] Do **not** author `src/main.rs` body beyond the cargo-generated `fn main() { println!("Hello, world!"); }` — Bevy app assembly is Story 1.5.
  - [x] Do **not** add `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.gitignore` rules — these are Story 1.3.
  - [x] Do **not** add CI workflows — Story 1.4.
  - [x] Do **not** introduce any `src/<module>/` structure — module layout comes with the plugins that need it (Epics 2+).

### Review Findings

_Added 2026-04-22 by `bmad-code-review` (3-layer adversarial review: Blind Hunter + Edge Case Hunter + Acceptance Auditor; 14 of 19 raw findings dismissed as context-blind false positives or spec-explicit decisions)._

- [x] [Review][Decision] **AC #6 literal compliance — `Cargo.lock` not yet committed** — Resolved: initial commit `4ca3869` ("chore: bootstrap Cargo project (Story 1.1)") stages `Cargo.toml` + `Cargo.lock` + `src/main.rs` + pre-existing `.gitignore`. AC #6 now literally satisfied.
- [x] [Review][Patch] **Dev Agent Record → File List mischaracterizes Cargo.toml as "Modified" and main.rs as "Unchanged"** — Resolved: File List rewritten to reflect the initial-commit reality (all entries are "Added", with subcategorization by hand-authored vs generated vs cargo-init-default).
- [x] [Review][Defer] **`[package].rust-version` MSRV not set (Bevy 0.18 requires rustc 1.89)** — Story 1.3 owns `rust-toolchain.toml`; MSRV metadata naturally rides with that work.
- [x] [Review][Defer] **`[profile.dev.build-override] opt-level = 0` not added alongside `[profile.dev.package."*"]`** — Bevy's fully-recommended dev-profile snippet includes both; only the latter is in the skeleton. Small compile-speed win available; fold into Story 1.3 (toolchain/lint polish) or a later milestone-gate upgrade window.
- [x] [Review][Defer] **`default-features = false` may omit winit/wgpu-Metal for Windows/macOS** — Bevy 0.18's `"3d"` feature collection should transitively pull `bevy_winit` + `bevy_render` + platform wgpu backends (Metal via wgpu's default); this will be *proven* when Story 1.5 actually opens a window. If Story 1.5 surfaces a missing-feature error, patch back into 1.1's manifest.
- [x] [Review][Defer] **No explicit `[[bin]] name = "asteriods3d"` — binary inherits mixed-case package name** — Spec explicitly routes packaging/release-binary naming to Story 4.10 / Epic 10 polish. Leave as-is.

## Dev Notes

### Why this story exists (scope clarification)

This is the **plugin-compatibility manifest step** of the Architecture's "Hybrid Manual" starter decision. The goal is a reproducible, hand-authored `Cargo.toml` whose dependency graph resolves cleanly — nothing more. The *proof* that every plugin compiles against Bevy 0.18 (not just resolves) is Story 1.2. Keeping these separate lets 1.2 own the fork-or-substitute decision with a clean signal. [Source: architecture.md:104-108,961-985]

### Cargo.toml Skeleton

Use this as the shape of the manifest. Versions marked `<latest-0.18-compatible>` must be resolved in Task 2 against crates.io.

```toml
[package]
name = "asteriods3D"          # see Project Structure Notes for casing rationale
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = { version = "0.18", default-features = false, features = ["3d", "png"] }
avian3d = "0.6"
bevy_mod_outline = "<latest-0.18-compatible>"
bevy_kira_audio = "<latest-0.18-compatible>"
leafwing-input-manager = "<latest-0.18-compatible>"

serde = { version = "1", features = ["derive"] }
serde_json = "1"
ron = "0.8"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
directories = "5"

# Linux needs an explicit windowing backend — Windows/macOS get theirs from Bevy defaults.
[target.'cfg(target_os = "linux")'.dependencies.bevy]
version = "0.18"
default-features = false
features = ["3d", "png", "x11", "wayland"]

# Dev-only GUI tooling (egui panels for FPS, entity inspector, tuning) — stripped from release.
[target.'cfg(debug_assertions)'.dependencies]
bevy_egui = "<latest-0.18-compatible>"

[profile.release]
lto = "fat"
codegen-units = 1
opt-level = 3

# Bevy dev-speed convention: dependencies compiled with -O1 in dev builds; workspace code stays -O0.
[profile.dev.package."*"]
opt-level = 1
```

Notes on the skeleton:
- `"2d"` is intentionally **not** enabled — the game is 3D-only and 0.18's feature collections let us exclude the 2D pipeline to cut compile time. [Source: architecture.md:133]
- `bevy_window` is pulled transitively by `"3d"` in 0.18; no need to enable explicitly.
- `avian3d` is the crate name; `avian` is the project's colloquial name. [Source: architecture.md:93]
- `edition = "2024"` matches the Rust edition pinned by Story 1.3's `rust-toolchain.toml` (latest stable). Keep as-is. [Source: architecture.md:130]

### Plugin Version Resolution

For the four plugins below, find the latest crates.io release that declares a Bevy `^0.18` (or `=0.18`) dependency. Record the exact version strings into `Cargo.toml` and into **Completion Notes**:

| Plugin | Purpose | Risk note |
|---|---|---|
| `bevy_mod_outline` | Silhouette outlines (FR49) | Upgrade-churn risk; fork-ready if stagnates. [Source: prd.md:401-403; architecture.md:56-70] |
| `bevy_kira_audio` | Spatial audio channels (FR23) | Same fork-ready policy. |
| `leafwing-input-manager` | Logical `Action` input abstraction (FR1–FR8 foundation) | Well-maintained, lower risk. [Source: requirements-inventory.md:147-148] |
| `bevy_egui` | Dev-only debug panels (cfg(debug_assertions)) | Dev-path only; never shipped. [Source: requirements-inventory.md:156-157] |

**If a plugin has no 0.18 release:** pin the closest version, flag it in **Completion Notes**, and let Story 1.2 decide the resolution path (upstream patch / fork-and-inline / substitute). Do **not** fork in this story — 1.2 owns that decision. [Source: epics/epic-1-…md:42-52]

### Platform Matrix Context

All three platforms are first-class from M0; no platform may be de-prioritized. The Cargo manifest must produce a clean build on Windows 10+, Linux (Ubuntu LTS / Fedora / Arch), and macOS (Apple Silicon + Intel x86_64). Platform verification across OSes is Story 1.4's responsibility (CI matrix); this story only needs `cargo check` to pass locally. [Source: prd.md:74-75,115-116,359-362; requirements-inventory.md:64 (FR47)]

### Dependency Pinning Governance

Bevy + Avian + plugins are hard-pinned at M0. Upgrade budget is planned only at M4, M6, M9 milestone gates (4–6 h per minor-version migration). No ad-hoc mid-milestone upgrades. This story *establishes* the pins; do not chase "latest" versions outside the constraints above. [Source: prd.md:401,406; requirements-inventory.md:114-115]

### Testing Standards

- No automated tests required for this story. AC validation is `cargo check` exit status + visual inspection of `Cargo.toml`.
- Bevy-integration tests are deferred post-M3 per architecture. [Source: architecture.md:143-146]

### Project Structure Notes

**Package name casing.** The existing `Cargo.toml` declares `name = "asteriods3D"` (capital D). Cargo tolerates this but conventional Rust package names are snake_case lowercase. The architecture's bootstrap snippet uses `asteriods3d` (lowercase) in the `cargo new` example. [Source: architecture.md:114,967]

**Decision:** Keep the existing `name = "asteriods3D"` casing. Rationale: (a) the project already exists under that name, (b) the user-facing concept docs consistently use `asteriods3D`, (c) changing it now would churn any future artifact path already referencing the package name. Note any cargo-warning output and move on. If cargo emits a hard error (not just a warning) on the casing, fall back to `asteriods3d` and flag in **Completion Notes**.

**Binary name.** Cargo will produce `asteriods3D` (or `asteriods3D.exe` on Windows) as the binary. Release-workflow packaging (Story 4.10 / 10.x) will rename as needed for ZIP artifacts — not this story's concern.

**Module layout.** This story does NOT create `src/core/`, `src/flight/`, or any other module directories. The full target layout (see architecture.md:534-639) is built incrementally by the plugins that need it starting in Epic 2. Keep `src/` containing only the default `main.rs`.

**Repo layout preservation.** The repo already contains `_bmad/`, `_bmad-output/`, `.claude/`, `docs/`, and `.gitignore`. None of these are touched by this story.

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Selected-Starter-Hybrid-Manual (lines 104-125)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Cargo-Configuration (lines 132-137)]
- [Source: _bmad-output/planning-artifacts/architecture.md#First-Implementation-Priority-M0-start (lines 961-985)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Current-Versions-verified-April-2026 (lines 90-96)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Project-Structure-Diagram (lines 534-639)]
- [Source: _bmad-output/planning-artifacts/prd.md#Fixed-Inputs (lines 27-30, 54-60)]
- [Source: _bmad-output/planning-artifacts/prd.md#Cross-platform-parity (lines 115-116, 359-362)]
- [Source: _bmad-output/planning-artifacts/prd.md#Tech-Risk-Third-party-crate-risk (lines 401-403)]
- [Source: _bmad-output/planning-artifacts/prd.md#Build-CI (line 406)]
- [Source: _bmad-output/planning-artifacts/epics/epic-1-foundation-plugin-compatibility-gate.md#Story-1.1 (lines 5-31)]
- [Source: _bmad-output/planning-artifacts/epics/requirements-inventory.md#Starter-Template-M0-Gate (lines 105-112)]
- [Source: _bmad-output/planning-artifacts/epics/requirements-inventory.md#Version-Pinning-Governance (lines 114-115)]
- [Source: _bmad-output/planning-artifacts/epics/requirements-inventory.md#FR47 (line 64)]

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Claude Code)

### Debug Log References

- `cargo check` output: clean build; all bevy_* sub-crates resolved to 0.18.1; all plugins compiled against Bevy 0.18.1. Duration: 49.56s (cold cache).
- Crates.io resolution verified in throwaway workspace (/tmp/bevy-compat-check) with `cargo tree -e normal` — confirmed every plugin pulls transitive `bevy_* v0.18.1`.

### Completion Notes List

**Resolved plugin versions (Bevy 0.18-compatible, as of 2026-04-22):**

| Plugin | Pinned | Resolved | Bevy dep |
|---|---|---|---|
| `bevy` | `0.18` | 0.18.1 | — |
| `avian3d` | `0.6` | 0.6.1 | bevy 0.18.1 (direct) |
| `bevy_mod_outline` | `0.12` | 0.12.0 | bevy 0.18.1 (direct) |
| `bevy_kira_audio` | `0.25` | 0.25.0 | bevy 0.18.1 (direct) |
| `leafwing-input-manager` | `0.20` | 0.20.0 | bevy 0.18.1 (direct) |
| `bevy_egui` | `0.39` | 0.39.1 | bevy_app/bevy_render/etc. 0.18.1 (fine-grained sub-crate deps, not full `bevy`) |

No plugin version gaps detected — all four plugins have a current release targeting Bevy 0.18. **Story 1.2 can proceed without fork-or-substitute work.**

**Package casing.** Kept `name = "asteriods3D"` (capital D) per Project Structure Notes. `cargo check` produced no casing warning or error. Proceeding.

**Lockfile status.** `Cargo.lock` was generated by `cargo check`. No `.gitignore` file exists yet that would exclude it (confirmed via `git check-ignore Cargo.lock` → exit 1 / not ignored). `.gitignore` authorship is deferred to Story 1.3 per scope guardrails. Per AC #6, `Cargo.lock` should be committed alongside `Cargo.toml` at the next user-initiated commit — this workflow does not auto-commit.

**No pin adjustments needed.** All six declared dependency versions resolved on first `cargo check`.

**Out of scope (deferred to future stories, per Task 5):**
- `src/main.rs` body: still the cargo-default `println!("Hello, world!")`. Bevy App assembly is Story 1.5.
- No `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, or `.gitignore` authored. Those belong to Story 1.3.
- No CI workflows. Story 1.4.
- No `src/<module>/` directories. Plugin modules land in Epic 2+.

### File List

All entries below are **new files in the initial commit** `4ca3869` — the repo had no prior HEAD before this story. The "pre-1.1 working-tree state" (from cargo-init) is implicit, never committed.

**Added (hand-authored for this story):**
- `Cargo.toml` — full hand-authored manifest: pinned direct deps, Linux windowing target, `cfg(debug_assertions)` bevy_egui, `[profile.release]` (lto=fat / codegen-units=1 / opt-level=3), `[profile.dev.package."*"]` opt-level=1. Replaces the cargo-init default stub (empty `[dependencies]`).

**Added (generated by cargo on first `cargo check`):**
- `Cargo.lock` — dependency resolution artifact; 684 locked packages. Committed per AC #6.

**Added (unchanged from cargo-init output, retained in-scope per Task 5 guardrails):**
- `src/main.rs` — cargo-default body (`fn main() { println!("Hello, world!"); }`). Bevy app assembly is Story 1.5.
- `.gitignore` — cargo-init default (`/target` only). Augmentation is Story 1.3's scope.

## Change Log

| Date | Author | Change |
|---|---|---|
| 2026-04-22 | claude-opus-4-7 (Dev) | Story 1.1 implemented. Cargo.toml authored with pinned Bevy 0.18 + 5 plugins + utility crates. `cargo check` passes. Status: ready-for-dev → review. |
| 2026-04-22 | claude-opus-4-7 (Review) | 3-layer adversarial code review (Blind + Edge + Auditor). 19 raw findings → 1 decision + 1 patch + 4 defer + 13 dismissed. Decision: initial commit `4ca3869` created (AC #6 satisfied). Patch: File List rewritten. Deferred items logged to `deferred-work.md`. Status: review → done. |
