# Deferred Work

Cross-story backlog of review findings intentionally deferred. Each entry lists the source review, original severity, and the reason for deferral so future sessions can re-evaluate.

## Deferred from: code review of 1-1-bootstrap-cargo-project-with-hand-authored-cargo-toml (2026-04-22)

- **`[package].rust-version` MSRV field not set** — Bevy 0.18 / `bevy_egui` 0.39 require rustc 1.89. Deferred to Story 1.3 (rust-toolchain.toml authoring) where MSRV metadata naturally lives.
- **`[profile.dev.build-override] opt-level = 0` not added** — Bevy's fully-recommended dev-profile snippet includes both `build-override` and `package."*"`; only the latter is in the current skeleton. Small compile-speed win; revisit in Story 1.3 toolchain polish or at an M4/M6/M9 upgrade window.
- **`default-features = false` may omit winit/wgpu-Metal on Windows/macOS** — Bevy 0.18's `"3d"` feature collection should transitively pull the needed renderer/windowing crates; this will be empirically validated when Story 1.5 opens the first window on each platform. If Story 1.5 surfaces a missing-feature error, patch back into Cargo.toml.
- **No explicit `[[bin]] name = "asteroids3d"` (lowercase)** — Binary inherits mixed-case package name (`asteroids3D`). Spec explicitly routes release-binary naming to Story 4.10 / Epic 10 packaging work. Leave as-is for now.

## Review correction (2026-04-22) — formerly dismissed finding now confirmed real

- **🐛 `[target.'cfg(debug_assertions)'.dependencies]` does NOT strip `bevy_egui` from release builds** — Cargo emits this warning on `cargo check`:
  `warning: Found 'debug_assertions' in target.'cfg(...)'.dependencies. This value is not supported for selecting dependencies and will not work as expected.`
  Cargo evaluates `cfg()` predicates in dependency tables BEFORE rustc-flag processing, so `debug_assertions` is treated as always-true and `bevy_egui` is pulled into every build (debug AND release). The Blind Hunter and Edge Case Hunter both raised this in the original 2026-04-22 review; both findings were incorrectly dismissed under the assumption that the cargo compile success implied correct semantics. The compile DID succeed — but the design intent ("dev-only egui, stripped from release") is silently broken.
  **Source of error:** the `Cargo.toml Skeleton` in story 1.1's Dev Notes (and presumably the upstream `architecture.md`) prescribes this exact (broken) pattern. Architecture doc may need an erratum.
  **Candidate fixes (architectural, NOT a Story 1.1 micro-patch):**
  1. Move `bevy_egui` to `[features] dev-tools = ["dep:bevy_egui"]` and run dev with `cargo run --features dev-tools`. Clean, explicit, but requires a features-section convention.
  2. Move `bevy_egui` to `[dev-dependencies]`. Only available in tests/examples — does NOT solve the "I want it in `cargo run` debug" use case.
  3. Keep `bevy_egui` as an unconditional dep, gate USAGE in Rust code with `#[cfg(debug_assertions)]`. Egui still in Cargo.lock and compile graph for release (binary bloat persists), but at least no runtime overhead.
  Recommended: option 1 (feature flag) — but defer to Story 1.5 (first time the egui plugin actually gets registered) so the fix and the first usage land together.
