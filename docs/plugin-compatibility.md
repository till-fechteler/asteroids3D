# Plugin Compatibility Matrix

**Status:** ✅ GATE PASSED — all four third-party plugins compile against Bevy 0.18 on macOS 26.4.1 / arm64, 2026-04-22.

**Verification date:** 2026-04-22
**Platform verified:** macOS 26.4.1 / arm64 (Till's dev machine, Apple Silicon)
**Verification scope:** local `cargo clean && cargo check` only. Cross-platform verification (Windows, Linux) is Story 1.4's responsibility via the GitHub Actions CI matrix.

## Toolchain

| Component | Version |
|---|---|
| Rust (`rustc`) | 1.94.1 (e408947bf 2026-03-25) |
| Cargo | 1.94.1 (29ea6fb6a 2026-03-24) |
| Edition | 2024 |

## Core engine

| Crate | Declared pin | Resolved | Role |
|---|---|---|---|
| `bevy` | `0.18` | `0.18.1` | Engine / ECS |
| `avian3d` | `0.6` | `0.6.1` | Physics (XPBD) |

## Third-party plugins (gated)

| Crate | Declared pin | Resolved | Bevy compat | Role | Risk |
|---|---|---|---|---|---|
| `bevy_mod_outline` | `0.12` | `0.12.0` | bevy 0.18.1 (direct) | FR49 silhouette outlines | Upgrade-churn; fork-ready. [Source: prd.md:401-403] |
| `bevy_kira_audio` | `0.25` | `0.25.0` | bevy 0.18.1 (direct) | FR23 spatial audio channels | Upgrade-churn; fork-ready. |
| `leafwing-input-manager` | `0.20` | `0.20.0` | bevy 0.18.1 (direct) | FR1–FR8 input abstraction | Well-maintained. |

## Deferred / Planned re-introduction

- **`bevy_egui`** — removed 2026-04-23 by Story 1.5. The original `[target.'cfg(debug_assertions)'.dependencies]` gating did not work: Cargo evaluates `cfg(debug_assertions)` as always-true in dependency tables and emitted the `Found 'debug_assertions' in target.'cfg(...)'.dependencies` manifest warning on every `cargo check`/`cargo build`. Re-introduction is planned at the M2 debug-panels story, as an optional feature-flag dep:
  ```toml
  [dependencies]
  bevy_egui = { version = "<pin re-verified against M2-era Bevy>", optional = true }

  [features]
  dev-tools = ["dep:bevy_egui"]
  ```
  Registration will be `#[cfg(feature = "dev-tools")]`-gated so plain `cargo build` / `cargo run` strip it cleanly. Last verified-compatible pin (pre-removal): `0.39.1` against Bevy `0.18.1`.

## Resolution Log

<!-- Populated only if a plugin fails to compile. Format: plugin | error | resolution path | link -->

_(empty — no resolutions required at M0 start; all four plugins resolved cleanly on first clean `cargo check`)_

## Known Issues / Deferred

_(empty — the prior `cfg(debug_assertions)` manifest warning was resolved by Story 1.5's removal of `bevy_egui` from `Cargo.toml`; re-introduction plan is documented under **Deferred / Planned re-introduction** above.)_

## Upgrade policy

Version bumps happen only at M4, M6, M9 milestone-gate windows, with a 4–6 h budget per minor-version migration. No ad-hoc mid-milestone upgrades. [Source: prd.md:406; requirements-inventory.md:114-115]

## Change Log

| Date | Event |
|---|---|
| 2026-04-22 | Initial gate pass. Clean `cargo check` on macOS 26.4.1 / arm64 (Rust 1.94.1, Bevy 0.18.1). All four plugins compile. One known warning (`cfg(debug_assertions)`), deferred to Story 1.5. |
| 2026-04-23 | Story 1.5: removed `bevy_egui` from `Cargo.toml` (the `cfg(debug_assertions)` gating was broken; Cargo treats that predicate as always-true in dependency tables). Manifest warning eliminated. Re-introduction deferred to M2 debug-panels story as a feature-flag dep. |
