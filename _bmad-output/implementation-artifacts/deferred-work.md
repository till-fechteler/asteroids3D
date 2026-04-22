# Deferred Work

Cross-story backlog of review findings intentionally deferred. Each entry lists the source review, original severity, and the reason for deferral so future sessions can re-evaluate.

## Deferred from: code review of 1-1-bootstrap-cargo-project-with-hand-authored-cargo-toml (2026-04-22)

- **`[package].rust-version` MSRV field not set** — Bevy 0.18 / `bevy_egui` 0.39 require rustc 1.89. Deferred to Story 1.3 (rust-toolchain.toml authoring) where MSRV metadata naturally lives.
- **`[profile.dev.build-override] opt-level = 0` not added** — Bevy's fully-recommended dev-profile snippet includes both `build-override` and `package."*"`; only the latter is in the current skeleton. Small compile-speed win; revisit in Story 1.3 toolchain polish or at an M4/M6/M9 upgrade window.
- **`default-features = false` may omit winit/wgpu-Metal on Windows/macOS** — Bevy 0.18's `"3d"` feature collection should transitively pull the needed renderer/windowing crates; this will be empirically validated when Story 1.5 opens the first window on each platform. If Story 1.5 surfaces a missing-feature error, patch back into Cargo.toml.
- **No explicit `[[bin]] name = "asteriods3d"` (lowercase)** — Binary inherits mixed-case package name (`asteriods3D`). Spec explicitly routes release-binary naming to Story 4.10 / Epic 10 packaging work. Leave as-is for now.
