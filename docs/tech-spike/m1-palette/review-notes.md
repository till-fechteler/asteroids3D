# M1 Palette — Color-Blindness Distinguishability Review

**Date:** 2026-04-28
**Tool:** Sim Daltonism v2.0.5 / macOS 26.4.1
**Source:** docs/tech-spike/m1-palette/normal.png
**Palette source:** Wong (2011) 8-color colorblind-safe palette (citation in `src/visual/palette.rs`).

## Methodology

Pairwise swatch comparison across 3 simulations (protanopia, deuteranopia, tritanopia).
Pass criterion: every pair clearly distinguishable by hue OR luminance under each simulation.

## Results — Protanopia

| Pair | Distinguishable? | Notes |
|---|---|---|
| Enemy ↔ Salvage | yes | … |
| Enemy ↔ Hazard | yes | … |
| Enemy ↔ Player | yes | … |
| Enemy ↔ Neutral | yes | … |
| Salvage ↔ Hazard | yes | … |
| Salvage ↔ Player | yes | … |
| Salvage ↔ Neutral | yes | … |
| Hazard ↔ Player | yes | … |
| Hazard ↔ Neutral | yes | … |
| Player ↔ Neutral | yes | … |

## Results — Deuteranopia

| Pair | Distinguishable? | Notes |
|---|---|---|
| Enemy ↔ Salvage | yes | … |
| Enemy ↔ Hazard | yes | … |
| Enemy ↔ Player | yes | … |
| Enemy ↔ Neutral | yes | … |
| Salvage ↔ Hazard | yes | … |
| Salvage ↔ Player | yes | … |
| Salvage ↔ Neutral | yes | … |
| Hazard ↔ Player | yes | … |
| Hazard ↔ Neutral | yes | … |
| Player ↔ Neutral | yes | … |

## Results — Tritanopia

| Pair | Distinguishable? | Notes |
|---|---|---|
| Enemy ↔ Salvage | yes | … |
| Enemy ↔ Hazard | yes | … |
| Enemy ↔ Player | yes | … |
| Enemy ↔ Neutral | yes | … |
| Salvage ↔ Hazard | yes | … |
| Salvage ↔ Player | yes | … |
| Salvage ↔ Neutral | yes | … |
| Hazard ↔ Player | yes | … |
| Hazard ↔ Neutral | yes | … |
| Player ↔ Neutral | yes | … |

## Failing Pairs (if any)

No failing pairs identified across all simulations.

## Conclusion

GO — palette accepted as-is
