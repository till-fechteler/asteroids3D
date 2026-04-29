# M1 Vector Aesthetic Tech-Spike — Go/Fallback Decision

**Date:** 2026-04-29
**Decision Owner:** Till Fechteler (project author)
**Source evidence:** [docs/tech-spike/m1-backends/parity-report.md](./m1-backends/parity-report.md)
**Stories closing:** Story 2.6 (this document); Story 2.7 marked `not-needed`.
**Milestone:** M1 — Vector Aesthetic Tech Spike (closing).

## Decision

GO toon

## Rationale

The Story 2.5 parity report ([parity-report.md:75-102](./m1-backends/parity-report.md)) records six qualitative-equivalence checks across Metal (hardware, Apple M5 Pro), Vulkan (Mesa lavapipe software ICD on `ubuntu-latest`), and DX12 (WARP software adapter on `windows-latest`): posterized banding count, rim-light at the asteroid silhouette, per-entity tint colors, outline silhouette continuity, outline width visual proportion, and swatch palette colors. **All six checks pass on all three backends.** Quantitative pixel-diff metrics computed via ImageMagick (`magick compare -metric RMSE`) yield three RMSE_normalized values — Metal↔Vulkan: `0.000423` (118× under threshold); Metal↔DX12: `0.006960` (7× under); Vulkan↔DX12: `0.006955` (7× under) — against the AC #4 (Story 2.5) threshold of `0.05`. The deltas are dominated by sub-pixel anti-aliasing jitter at outline edges and a one-pixel-row outline-thinning observation on WARP, neither of which indicates a WGSL-translation defect.

This story formally resolves PRD risk **R#2** ("WGSL shader complexity for a beginner on three graphics backends, Metal/Vulkan/DX12" — [`prd.md:441`](../../_bmad-output/planning-artifacts/prd.md)). The M1 tech-spike's three-backend validation gate (Story 2.5) demonstrates that the custom WGSL toon material (Story 2.3, `assets/shaders/toon.wgsl` + `src/visual/toon_material.rs`) and the `bevy_mod_outline` integration (Story 2.4, `src/visual/outline.rs`) translate correctly through Naga to SPIR-V (Vulkan) and HLSL/DXIL (DX12), with no observed regressions versus hardware Metal as the reference. Cross-references: PRD M#10 fallback condition ([`prd.md:347`](../../_bmad-output/planning-artifacts/prd.md)) is **not** met, and the architecture-document M1 completion gate ([`architecture.md:295`](../../_bmad-output/planning-artifacts/architecture.md)) is satisfied.

Till (project author and decision owner) has independently inspected the three captured PNGs at [`docs/tech-spike/m1-backends/{metal,vulkan,dx12}.png`](./m1-backends/) side-by-side and concurs with the parity report's recommendation. The toon material is therefore confirmed as the **M2 production shader**: `assets/shaders/toon.wgsl` and `src/visual/{toon_material.rs, outline.rs, palette.rs}` graduate from "M1-tech-spike artifact" to "permanent M2-production code" status. The reference scene (`src/visual/reference_scene.rs`) and the parity capture mode (`src/visual/capture.rs` + `.github/workflows/parity-capture.yml`) remain M1-spike-only; their removal is scheduled for Story 3.1 (Arena state) per the cross-linked deferred-work entry.

## Risks Accepted

- **Software-rasterizer-noise residual.** ImageMagick RMSE on the M↔V and V↔D pairs is dominated by sub-pixel AA jitter and software-vs-hardware rasterizer rounding (lavapipe and WARP are both software). Real-hardware Vulkan and DX12 testing has NOT happened — only software rasterizers were exercised in CI. **Mitigation:** deferred to first user-reported render artifact + reference-hardware playtest at Epic 10's Story 10.12 ("3-platform 60-FPS zero-crash playtest").
- **WARP outline-thinning observation.** DX12's WARP renders silhouette outlines approximately one pixel-row thinner than hardware Metal. Within visual tolerance per Story 2.5's qualitative check #5 (outline width proportion). **Mitigation:** HUD/UI uses `bevy_ui` (screen-space), not `bevy_mod_outline` — only world-space mesh silhouettes are affected; an Epic 10 polish-pass fine-line UI choice cannot cross this threshold.
- **Bevy version-bump risk (M4 / M6).** A future Bevy or Naga upgrade could regress WGSL→backend translation. **Mitigation:** the `parity-capture.yml` workflow stays in place through Story 3.1, and any Bevy bump's PR description must note re-running the parity capture as a step. Capture mode itself is removed at Story 3.1, but the workflow file removal is part of the same cleanup; re-introduction at the version-bump window is tracked bidirectionally in `deferred-work.md`.
- **Hardware coverage gap.** Apple Silicon M5 Pro (capture host) ≠ the PRD's NFR-P1 baseline of "Apple M1." Metal capture used Till's actual development hardware; M1-class parity is implicitly assumed from the M5 Pro evidence (newer-and-faster hardware). **Mitigation:** deferred to Apple-Silicon-M1-class playtest in M3 / Story 4.10 readiness or M9 / Story 10.12 if no M1-class hardware reaches Till before then.

## Fallback Trigger Criteria

These are tripwires, not pre-emptive escape hatches. If any condition is met, this decision is overridden by a new dated decision document at `docs/tech-spike/m1-decision-revisit-<date>.md`.

1. **Cross-backend qualitative regression observed.** Any of the six qualitative-equivalence checks fails on a fresh `parity-capture.yml` dispatch. Drift in RMSE_normalized values is NOT a trigger; missing rim-light, mismatched band count, swizzled tint channel, broken outline continuity, or lost swatch color ARE.
2. **Real-hardware GPU divergence.** When the parity report is re-run on real GPUs (Till's GTX 1060 / RX 580 reference hardware at Story 10.12, or any user playtest report), if the qualitative checks fail on hardware in a way that the software-rasterizer path masked, this decision flips to `FALLBACK flat+rim-light` and Story 2.7 work is opened.
3. **NFR-P1 60-FPS regression attributable to the toon shader.** Profiling at Story 10.1 reveals that the toon material is the dominant cost preventing 60 FPS at 1080p on the GTX 1060 / RX 580 / Apple M1 baseline. Performance trigger, not correctness trigger. Mitigation path is `GO toon with scope reduction` (drop rim-light, reduce step count) before flipping to `FALLBACK`.
4. **Bevy / Naga version-bump regression.** A future M4 / M6 Bevy upgrade introduces WGSL→backend translation issues that the upstream maintainers do not fix within ~30 days. Pin the prior Bevy version OR flip to fallback if the prior pin is no longer viable for security or dependency-graph reasons.

## M2 Impact

- **Production code confirmed.** `assets/shaders/toon.wgsl` and `src/visual/{toon_material.rs, outline.rs, palette.rs}` are confirmed M2-production code. They graduate from "M1-tech-spike artifact" to "permanent project code" status; no `[deprecated]` attribute, no scope reduction.
- **Story 2.7 disposition: `not-needed`.** New sprint-status.yaml status value introduced by this decision (see `_bmad-output/implementation-artifacts/sprint-status.yaml` definition block). No fallback material scaffold work; no `src/visual/flat_rim_material.rs`; the `_fallback`-suffixed paths in [`epics/epic-2-vector-aesthetic-tech-spike.md:191-194`](../../_bmad-output/planning-artifacts/epics/epic-2-vector-aesthetic-tech-spike.md) will not exist.
- **Cleanup at Story 3.1.** `src/visual/capture.rs`, the `pub mod capture;` line in `src/visual/mod.rs`, the env-var lookup + conditional `WindowPlugin` override + conditional `CapturePlugin` registration in `src/main.rs`, AND `.github/workflows/parity-capture.yml` are removed when the Arena state replaces the cfg-gated reference scene. Already tracked in [`deferred-work.md`](../../_bmad-output/implementation-artifacts/deferred-work.md) (Story 2.5 deferral block); cross-linked from this document for traceability.
- **Audit trail preserved.** `docs/tech-spike/m1-backends/{metal,vulkan,dx12}.png`, `docs/tech-spike/m1-backends/parity-report.md`, the three diff heatmaps in `m1-backends/`, AND this decision document STAY. They are the auditable evidence that the M1 gate was satisfied; they do not get cleaned up at Story 3.1.
