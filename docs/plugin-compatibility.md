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
| `bevy_egui` | `0.39` | `0.39.1` | bevy 0.18 sub-crates (bevy_app/bevy_render/etc.) | Dev-only debug panels | Dev-path only. Currently leaks into release builds — see Known Issues. |

## Resolution Log

<!-- Populated only if a plugin fails to compile. Format: plugin | error | resolution path | link -->

_(empty — no resolutions required at M0 start; all four plugins resolved cleanly on first clean `cargo check`)_

## Known Issues / Deferred

- **`[target.'cfg(debug_assertions)'.dependencies]` does not strip `bevy_egui` from release builds.** Cargo emits the warning `Found 'debug_assertions' in target.'cfg(...)'.dependencies. This value is not supported for selecting dependencies and will not work as expected.` on every `cargo check`. The compile succeeds, but the design intent ("dev-only egui, stripped from release") is silently broken. Full detail and candidate fixes: `_bmad-output/implementation-artifacts/deferred-work.md` → *Review correction (2026-04-22)*. Scheduled fix: Story 1.5, when the egui plugin is first registered.

## Upgrade policy

Version bumps happen only at M4, M6, M9 milestone-gate windows, with a 4–6 h budget per minor-version migration. No ad-hoc mid-milestone upgrades. [Source: prd.md:406; requirements-inventory.md:114-115]

## Change Log

| Date | Event |
|---|---|
| 2026-04-22 | Initial gate pass. Clean `cargo check` on macOS 26.4.1 / arm64 (Rust 1.94.1, Bevy 0.18.1). All four plugins compile. One known warning (`cfg(debug_assertions)`), deferred to Story 1.5. |
