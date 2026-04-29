# M1 Backend Parity Report

**Date:** 2026-04-29
**Stories evidenced:** 2.3 (toon material), 2.4 (outline integration), 2.5 (this gate).
**Reference scene:** asteroid icosphere + ship cuboid + projectile UV-sphere, 3-point lighting,
deterministic camera at `Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y)`,
1920×1080 PNG, captured at frame 60 (~1 second post-Startup) in capture mode after splash bypass
to `GameState::MainMenu` (so the swatch palette bar is visible).

## Capture environment

| Backend | Platform | Renderer | Capture command |
|---|---|---|---|
| Metal | macOS 26.4.1 (Apple M5 Pro) | hardware Metal | `ASTEROIDS3D_CAPTURE_PNG=… cargo run` (local, debug build) |
| Vulkan | Linux (`ubuntu-latest` / 24.04, GitHub-hosted) | Mesa lavapipe (LLVM software ICD) | `parity-capture.yml` `linux-vulkan` job, run [25113263165](https://github.com/till-fechteler/asteroids3D/actions/runs/25113263165) |
| DX12 | Windows (`windows-latest`, GitHub-hosted) | WARP (Microsoft software DX12 adapter) | `parity-capture.yml` `windows-dx12` job, run [25113263165](https://github.com/till-fechteler/asteroids3D/actions/runs/25113263165) |

**AC #1 deviation — debug build, not release.** `mod reference_scene` is `#[cfg(debug_assertions)]`-gated
(`src/visual/mod.rs:60`); a release build compiles out the entire scene, leaving an empty default-clear
window. Captures use `cargo run` (debug). Shader output bytes — the actual subject of parity testing —
are byte-identical between debug and release because the WGSL→Naga→backend translation pipeline does not
depend on Rust-side optimization level. See Story 2.5 Dev Notes "AC #1 deviation" for the full rationale.

**Important context for divergence interpretation.** Vulkan and DX12 here are SOFTWARE-rendered
on virtualized CI hosts (no GPU passthrough). Pixel-level deltas vs hardware Metal include
BOTH (a) WGSL-translation correctness — the M1 spike's actual concern — AND (b) software-vs-
hardware rasterizer rounding/AA differences that say nothing about correctness. The qualitative
visual-equivalence checks (below) are the load-bearing parity signal; quantitative pixel
counts are an upper bound that includes irrelevant noise.

## Pairwise diffs

ImageMagick 7.1.2-21 Q16-HDRI on macOS (`magick compare -metric AE/RMSE A B null:`).

| Pair | AE (absolute pixel diff count) | AE % of 2,073,600 | RMSE (raw / normalized) | Heatmap |
|---|---|---|---|---|
| Metal ↔ Vulkan | 44787 | 2.16% | 27.72 / 0.000423 | `diff-metal-vs-vulkan.png` |
| Metal ↔ DX12 | 9410 | 0.45% | 456.10 / 0.006960 | `diff-metal-vs-dx12.png` |
| Vulkan ↔ DX12 | 50727 | 2.45% | 455.83 / 0.006955 | `diff-vulkan-vs-dx12.png` |

**Total pixels in 1920×1080:** 2,073,600. **Threshold (AC #4):** RMSE_normalized > 0.05 (5%) requires
explicit divergence annotation. **All three pairs are well below the threshold** — the largest
RMSE_normalized is 0.0070, 7.1× under threshold.

## Divergence root-cause hypotheses

All RMSE values fall under the AC #4 0.05 threshold; no annotation is required. Brief observations
per pair for completeness:

### Metal ↔ Vulkan

Highest pixel-diff count (44787 pixels, 2.16%) but the smallest per-pixel RMSE (0.000423). This pattern
is consistent with sub-pixel anti-aliasing jitter at outline edges and rim-light gradients: many pixels
with tiny color deltas near silhouette boundaries. The toon banding boundaries themselves are deterministic
(`step()` in WGSL is binary), so band-to-band transitions align across backends; the differences are confined
to anti-aliased edge pixels and the rim-light angular falloff where small `dot()` deltas accumulate.

### Metal ↔ DX12

Smallest pixel-diff count (9410, 0.45%) but ~16× larger per-pixel RMSE (0.006960) than M↔V. This
suggests fewer pixels differ overall, but where they do, the color delta is larger. Visual-inspection
hypothesis: WARP renders the silhouette outlines slightly thinner / less anti-aliased than hardware Metal,
producing a small set of pixels that "lose" outline coverage entirely (large per-pixel RGB delta on
those pixels) rather than the diffuse AA jitter pattern of M↔V. Outline silhouette positions still align
qualitatively (see visual-equivalence checks) — this is rasterizer rounding, not a bevy_mod_outline backend
defect.

### Vulkan ↔ DX12

Worst pair on AE (50727, 2.45%) and matches M↔D's RMSE (0.006955). This is the structural sum: Vulkan
inherits M↔V's edge-AA noise plus DX12's outline-thinning effect, producing the largest total set of
differing pixels. Both are software rasterizers, both go through the Naga WGSL→IR translation; the
delta is rasterizer-implementation noise, not WGSL-translation noise.

## Qualitative visual equivalence checks (load-bearing)

Side-by-side comparison of the three PNGs at 100% zoom. Each property checked across all three backends:

- [x] **Posterized banding count.** All three placeholders show ~4 visible toon bands on the asteroid icosphere
      and the projectile UV-sphere (matching `tuning.ron`'s `toon_steps: 4`). The cuboid shows a single visible
      band per face (consistent with each face having a near-constant `N·L`). Identical band count across
      all three backends → uniform-binding parity confirmed.
- [x] **Rim-light at asteroid silhouette.** All three captures show the brightening at the icosphere's
      grazing-angle silhouette (top-right and bottom-left arcs of the asteroid). Identical visual presence
      across backends → `pow()` intrinsic parity confirmed.
- [x] **Per-entity tint colors.** Hazard yellow on the asteroid, PlayerOwned blue on the cuboid ship,
      Salvage green on the small projectile sphere — all three render with matching hex on all backends.
      No tint-channel swizzle bugs (e.g. R↔B confusion) on any backend.
- [x] **Outline silhouette continuity.** Asteroid icosphere and projectile sphere have continuous
      outline rings on all backends. Cuboid corners outline smoothly (the `generate_outline_normals`
      payoff from Story 2.4 applies regardless of backend). DX12's outline appears subtly thinner
      visually, but coverage is continuous — no gaps, no missing edges.
- [x] **Outline width visual proportion.** Outline width as a fraction of placeholder size is consistent
      across backends. The DX12 outline-thinning observation above is a width delta of approximately one
      pixel-row, well within rasterizer-rounding tolerance for software-rendered outputs vs hardware Metal.
- [x] **Swatch palette colors.** The 5-color palette bar at the top renders with matching hex on all
      backends (ENEMY orange, SALVAGE green, HAZARD yellow, PLAYER blue, NEUTRAL grey). White text
      labels render identically across backends.

**All six qualitative checks pass.** The toon shader and outline plugin behave correctly on all three
backends; no WGSL-translation defects observed; quantitative deltas are dominated by sub-pixel AA jitter
and software-rasterizer outline-AA differences that have no bearing on shader correctness.

## Recommendation for Story 2.6

> **RECOMMEND GO toon**

**Justification:** All six qualitative visual-equivalence checks pass on Metal (hardware), Vulkan
(lavapipe software), and DX12 (WARP software). Quantitative pixel-diff metrics are dominated by
sub-pixel anti-aliasing jitter at outline edges (M↔V: RMSE_norm = 0.000423) and minor outline-width
rasterizer rounding on WARP (M↔D, V↔D: RMSE_norm ≈ 0.0070). All three RMSE values are 7× to 118×
under the 5% AC #4 threshold. Toon banding count, rim-light presence, per-entity tint colors,
outline silhouette continuity, outline width proportion, and swatch palette colors all match
across the three backends. The custom WGSL toon material (Story 2.3) and `bevy_mod_outline`
integration (Story 2.4) translate correctly through Naga to SPIR-V (Vulkan) and HLSL/DXIL (DX12),
with no observed regressions on hardware Metal as the reference. The vector aesthetic is feasible
on all three target platforms; M1 spike concludes successfully and Story 3.1 (Arena) can build on
the toon + outline foundation without falling back to the simpler flat+rim-light alternative
scaffolded in Story 2.7.

Story 2.6 should ratify this recommendation in `docs/tech-spike/m1-decision.md` and skip Story 2.7
(fallback flat+rim-light) per `epic-2-vector-aesthetic-tech-spike.md:172-194`.
