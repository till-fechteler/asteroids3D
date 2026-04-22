---
stepsCompleted:
  - step-01-document-discovery
  - step-02-prd-analysis
  - step-03-epic-coverage-validation
  - step-04-ux-alignment
  - step-05-epic-quality-review
  - step-06-final-assessment
filesIncluded:
  prd: _bmad-output/planning-artifacts/prd.md
  architecture: _bmad-output/planning-artifacts/architecture.md
  epics: _bmad-output/planning-artifacts/epics.md
  ux: null (intentionally omitted per Till, 2026-04-22)
---

# Implementation Readiness Assessment Report

**Date:** 2026-04-22
**Project:** asteriods3D

## Step 1: Document Discovery

### Inventory

| Type | File | Notes |
|---|---|---|
| PRD | `_bmad-output/planning-artifacts/prd.md` (54 KB, 2026-04-21) | Whole document, no shards |
| Architecture | `_bmad-output/planning-artifacts/architecture.md` (66 KB, 2026-04-22) | Whole document, no shards |
| Epics & Stories | `_bmad-output/planning-artifacts/epics.md` (173 KB, 2026-04-22) | 10 Epics / 86 Stories |
| UX Design | — | **Intentionally omitted** by Till (confirmed 2026-04-22). UX concerns expected to live in PRD/Architecture for this cockpit-only, reduced-MVP hobby project. |

### Additional artifacts (not used as primary sources)

- `_bmad-output/planning-artifacts/prd-validation-report.md` — prior PRD validation run (reference only)
- `_bmad-output/brainstorming/brainstorming-session-2026-04-21-1114.md` — concept source-of-truth (reference only)

### Issues

- No duplicate whole/sharded formats
- UX document absent **by design** — not treated as a gap

## Step 2: PRD Analysis

### Functional Requirements

**Flight & Controls (FR1–FR8)**

- **FR1:** Player can pilot a ship through 3D space via keyboard + mouse input.
- **FR2:** Player can translate the ship in six directions (thrust forward, reverse, lateral strafe left/right, vertical up/down).
- **FR3:** Player can rotate the ship around all three axes (pitch, yaw, roll).
- **FR4:** Player can aim weapons independently of ship heading (decoupled aim).
- **FR5:** Player can toggle an inertial dampener that modulates Newtonian drift against arcade tightness.
- **FR6:** Player can initiate a boost that temporarily increases thrust at the cost of a rechargeable resource.
- **FR7:** Player can tractor-beam intact asteroids and debris toward the ship for salvage pickup.
- **FR8:** Player views gameplay exclusively through a first-person cockpit view; no external camera toggle during active gameplay.

**Combat System (FR9–FR16)**

- **FR9:** Player can fire weapons that emit projectiles with ballistic trajectories.
- **FR10:** Player ship equips up to 3 weapons drawn from a pool of 3 prefab archetypes.
- **FR11:** Each fired projectile deducts a configurable amount from the player's salvage currency (pay-to-shoot economy).
- **FR12:** Projectiles can damage asteroids, enemy ships, and debris.
- **FR13:** Destroyed asteroids yield salvage at a configurably lower rate than intact asteroids captured via tractor beam.
- **FR14:** Enemy ships detect, pursue, and attack the player within a configurable engagement range.
- **FR15:** Player ship has Hull and Shields subsystems: Shields regenerate after a cooldown following damage; Hull does not regenerate during a run.
- **FR16:** When player Hull reaches zero, the run ends (permadeath).

**Economy & Salvage (FR17–FR21)**

- **FR17:** Player accumulates salvage currency during a run from intact pickups, destroyed asteroids, and other configurable sources.
- **FR18:** On run completion (successful or failed), banked salvage converts to persistent meta-currency at a configurable rate.
- **FR19:** Meta-currency persists across runs via save data.
- **FR20:** Player can spend meta-currency between runs at an unlock shop to acquire permanent upgrades.
- **FR21:** The unlock shop offers between 5 and 10 distinct permanent upgrades affecting ship capabilities.

**Perception & Sensors (FR22–FR26)**

- **FR22:** Cockpit HUD displays a radar showing threat markers, range, and relative direction of detected entities.
- **FR23:** Game emits spatial stereo audio cues for enemies, hazards, and salvage-of-interest; cues indicate approximate direction.
- **FR24:** Cockpit HUD displays current shields, hull, ammunition status, and salvage-currency balance.
- **FR25:** Cockpit HUD indicates the economic yield delta between intact-capture and destroy for salvageable targets in view.
- **FR26:** On first launch, Game displays a splash screen recommending headphones for optimal spatial audio perception.

**Run Structure & Progression (FR27–FR35)**

- **FR27:** On the first session, Player is placed in a hand-designed Arena tutorial zone.
- **FR28:** Game presents no written tutorial text; all learning occurs through play and diegetic HUD cues.
- **FR29:** After completing the Arena tutorial, Player transitions to Caravan mode for subsequent runs.
- **FR30:** A Caravan run lasts 5 to 8 minutes from start to target destination.
- **FR31:** Caravan runs use a single MVP run-skeleton template with three selectable difficulty variants (easy, medium, hard).
- **FR32:** Caravan runs contain in-route combat pockets that trigger when the player enters configurable rendering-distance thresholds.
- **FR33:** Player can navigate to the run's target destination via a waypoint-pointer indicator rendered in the cockpit.
- **FR34:** On successful Caravan completion, Player banks accumulated salvage currency.
- **FR35:** On Caravan death, Player banks accumulated salvage currency; previously purchased meta-unlocks persist.

**UI & Feedback (FR36–FR43)**

- **FR36:** Player can access a title screen with options to start a new run, access settings, view credits, or quit.
- **FR37:** Player can adjust volume (master and SFX) and mouse sensitivity in a settings menu.
- **FR38:** On death, Player sees post-run summary (cause of death, salvage banked, retry / Photo Mode / menu). No "GAME OVER" overlay / red screen / defeat music.
- **FR39:** Player can restart a run immediately after death without returning to the title screen.
- **FR40:** Photo Mode accessible only from post-run or death screen; not during active gameplay.
- **FR41:** Photo Mode provides free-cam orbital/dolly movement, adjustable depth-of-field, time-frozen simulation.
- **FR42:** Player can export Photo Mode screenshots as PNG in 16:9, 9:16, and 1:1 aspect ratios.
- **FR43:** Game pauses the simulation on in-run pause menu or when the application window loses focus.

**Persistence & Platform (FR44–FR48)**

- **FR44:** Game persists meta-currency, unlocked upgrades, and settings to an OS-convention save location.
- **FR45:** On first launch, Game creates a default save file at the OS-convention save location.
- **FR46:** Save data survives unexpected termination (crash, force-quit, power loss) without corruption.
- **FR47:** Game runs as a native binary on Windows 10+, Linux (Ubuntu LTS / Fedora / Arch), and macOS (Apple Silicon and Intel x86_64).
- **FR48:** macOS binary is code-signed and notarized for distribution outside the Apple App Store.

**Visual Presentation (FR49–FR50)**

- **FR49:** Game renders all 3D geometry using a toon-shading material with silhouette outlines.
- **FR50:** Game applies semantic accent colors to entity categories (enemies / salvage / hazards / player-owned) against a restrained base palette; accent colors are distinguishable under the vector aesthetic.

**Total FRs: 50**

### Non-Functional Requirements

**Performance (NFR-P1–P5)**

- **NFR-P1:** Sustained 60 FPS @ 1080p on GTX 1060 / RX 580 / Apple M1 with vector shader active.
- **NFR-P2:** Double-click to title screen ≤ 10 s on reference hardware with SSD.
- **NFR-P3:** Title screen → active Caravan gameplay ≤ 5 s on reference hardware.
- **NFR-P4:** No visible frame hitches > 100 ms during steady-state gameplay; > 50 ms during transitions/save-load acceptable but logged.
- **NFR-P5:** Process memory usage during steady-state gameplay < 4 GB.

**Reliability (NFR-R1–R4)**

- **NFR-R1:** No crash during normal play across all four documented user journeys; crash-free playtest is a gate at M3/M6/M9.
- **NFR-R2:** Save data not corrupted by ungraceful termination; save writes atomic.
- **NFR-R3:** Game recovers gracefully from missing save (first launch → default; later missing → prompt).
- **NFR-R4:** Meta-currency and unlocks never lost between runs during normal play.

**Accessibility (NFR-A1–A3)**

- **NFR-A1:** Semantic accent colors visually distinguishable under protanopia / deuteranopia / tritanopia; color is never the sole signal.
- **NFR-A2:** No information critical to gameplay conveyed by color alone.
- **NFR-A3:** HUD text legible at 60–80 cm viewing distance from 1080p display at default scale.

**Usability (NFR-U1–U3)**

- **NFR-U1:** New player reaches first "aha" (holds fire on intact asteroid for higher salvage yield) within first 5-min Arena session; validated at M3.
- **NFR-U2:** All HUD tactical decision elements (shields / hull / salvage / radar / yield delta) simultaneously visible in cockpit view.
- **NFR-U3:** Player can identify Hull / Shields state at a glance without looking away from primary cockpit view.

**Localization (NFR-L1–L3)**

- **NFR-L1:** MVP ships in English only.
- **NFR-L2:** German localization is post-MVP deferred.
- **NFR-L3:** All player-facing strings loaded from external string table (JSON or RON), not hard-coded — preserves localization readiness.

**Not applicable (explicitly skipped):** Security, Scalability, Integration.

**Total NFRs: 18**

### Additional Requirements

**Design Principles (5 — principle-layer, tie-breakers, each surfaces in FR/NFR):**

1. Information-design discipline: no tutorial text (surfaces via FR28).
2. No visible numeric score (implicit in FR17–21, FR24 — economy is the score).
3. Asteroid motion is predictable, not random (Kepler orbits or scripted splines per A#3). **⚠️ Not explicitly covered by any FR — traceability candidate.**
4. Death is feedback, not punishment (surfaces via FR38 styling constraint).
5. Graceful degradation at every novel point (documented fallback paths required).

**Tonal Direction (non-testable, load-bearing for asset/shader/audio work):** Cosmic Mystery over War — survey vessel framing, scientific-instrument HUD, ambient/harmonic audio, rare visual-only "relic" suggestions, zero written lore.

**Fixed Inputs (from frontmatter):**
- Tech: Bevy (version-pinned), Avian (Bevy-native XPBD physics).
- Design decisions: Cockpit-only, Caravan core, Narrative-light, Vector aesthetic + toon shader.
- Binding constraint: motivation preservation across 10–14 months @ 4–8 h/week.
- Stop-and-ship waypoints: M3 (Itch.io prototype), M6 (Early Access), M9 (Full MVP).

**Accessibility scope boundaries (deferred / skipped in MVP — not a gap):** motion-sickness mitigation, full rebinding, screen-reader, closed captions.

### PRD Completeness Assessment

- **Structure:** Excellent — Executive Summary, Classification, Success Criteria, Scope, Journeys, Innovation, Project-Type Requirements, Risk/Scoping, Design Philosophy, FRs, NFRs all present.
- **Requirement clarity:** FRs are implementation-agnostic, testable, and cleanly numbered. NFRs have quantitative targets (60 FPS, ≤10 s load, <4 GB RAM, etc.).
- **Traceability risk:** **Design Principle 3** (predictable asteroid motion / Kepler / splines) is a **load-bearing design commitment** that is not explicitly enshrined as an FR. Flag for epic-coverage check — must land somewhere in an asteroid/motion epic.
- **Scope discipline:** Very strong — explicit MVP vs. Growth vs. Vision splits, deferred-list is complete, degradation paths documented for each innovation claim.
- **Hobby-cadence alignment:** Scope, risk mitigations, and stop-and-ship waypoints are all consistent with 4–8 h/week / 10-month budget.

No contradictions detected. PRD is implementation-ready; proceeding to epic coverage validation.

## Step 3: Epic Coverage Validation

### Epic Inventory

10 Epics / 86 Stories; epics.md frontmatter reports `stepsCompleted: ['step-01-validate-prerequisites', 'step-02-design-epics', 'step-03-create-stories']`, `storiesCompletedForEpics: E1–E10`, `resumeAt: 'step-04-final-validation'`.

| Epic | Title | M-alignment | Stories |
|---|---|---|---|
| E1 | Foundation & Plugin Compatibility Gate | M0 | 8 |
| E2 | Vector Aesthetic Tech Spike | M1 | 7 (incl. 1 conditional) |
| E3 | Arena Flight & First Combat | M2 | 11 |
| E4 | Enemies Alive & Stop-Ship (Itch.io) 🏁 | M3 | 10 |
| E5 | Ship Subsystem State & Formal Save | M4 | 6 |
| E6 | Caravan Run Framework ⚠️ | M5 | 13 |
| E7 | Roguelite Loop (EA-Viable) 🏁 | M6 | 6 |
| E8 | Perception — Sensors & Spatial Audio | M7 | 7 |
| E9 | Post-Run Photo Mode | M8 | 6 |
| E10 | Polish Pass & MVP Completion 🏁 | M9 | 12 |
| **Total** | | | **86** |

### FR Coverage Matrix

| FR | PRD Requirement (abbr.) | Epic / Story | Status |
|---|---|---|---|
| FR1 | Pilot ship via KB+mouse | E3 / S3.6+3.7 | ✓ |
| FR2 | 6-direction translation | E3 / S3.6 | ✓ |
| FR3 | 3-axis rotation | E3 / S3.7 | ✓ |
| FR4 | Decoupled aim | E5 / S5.5 | ✓ |
| FR5 | Inertial dampener toggle | E3 / S3.8 | ✓ |
| FR6 | Boost (rechargeable) | E6 / S6.12 | ✓ |
| FR7 | Tractor-beam intact capture | E6 / S6.10 | ✓ |
| FR8 | Cockpit-only first-person | E3 / S3.5 | ✓ |
| FR9 | Fire ballistic projectiles | E3 / S3.9 | ✓ |
| FR10 | Up to 3 weapons / 3 archetypes | E4 / S4.4 | ✓ |
| FR11 | Pay-to-shoot salvage debit | E6 / S6.9 | ✓ |
| FR12 | Projectile damage | E3 / S3.10 + E4 / S4.2 | ✓ |
| FR13 | Destroyed yield < intact | E6 / S6.11 | ✓ |
| FR14 | Enemy AI detect/pursue/attack | E4 / S4.2 | ✓ |
| FR15 | Hull + Shields regen model | E5 / S5.1–S5.3 | ✓ |
| FR16 | Permadeath on Hull-zero | E4 / S4.3 | ✓ |
| FR17 | Salvage currency accumulation | E6 / S6.5 | ✓ |
| FR18 | Salvage → meta-currency | E6 / S6.7 + E7 / S7.1 | ✓ |
| FR19 | Meta persists across runs | E6 / S6.7 + E7 / S7.1 | ✓ |
| FR20 | Spend meta in unlock shop | E7 / S7.3+7.4 | ✓ |
| FR21 | 5–10 unlocks | E7 / S7.2 (8) + E10 / S10.9 (+3) = 11 | ✓ |
| FR22 | Radar threat markers | E8 / S8.3 | ✓ |
| FR23 | Spatial stereo audio cues | E8 / S8.4 (+ S8.2 setup) | ✓ |
| FR24 | HUD shields/hull/ammo/salvage | E3 / S3.11 → E5 / S5.4 + E6 / S6.5 | ✓ |
| FR25 | Yield-delta indicator | E8 / S8.6 (data: E6 / S6.11) | ✓ |
| FR26 | Headphone-recommendation splash | E8 / S8.7 | ✓ |
| FR27 | Arena tutorial on first session | E3 / S3.3 + E6 / S6.2 (gate) | ✓ |
| FR28 | No tutorial text | E3 (design constraint across stories) | ⚠ Constraint-level, not its own story — acceptable |
| FR29 | Arena → Caravan transition | E6 / S6.2 | ✓ |
| FR30 | Caravan 5–8 min | E6 / S6.6 (target; "info!" logs actuals for tuning via S6.3 layout) | ✓ |
| FR31 | 3 difficulty variants | E6 / S6.13 | ✓ |
| FR32 | Combat pockets at render-distance | E6 / S6.8 (+ S6.3 placement) | ✓ |
| FR33 | Waypoint-pointer navigation | E6 / S6.4 | ✓ |
| FR34 | Bank salvage on success | E6 / S6.7 | ✓ |
| FR35 | Bank salvage on death (meta persists) | E6 / S6.7 | ✓ |
| FR36 | Title screen (start/settings/credits/quit) | E4 / S4.7 | ✓ |
| FR37 | Volume + sensitivity settings | E4 / S4.8 (volume-wiring concretized in E8 / S8.2) | ✓ |
| FR38 | Post-run summary (no GAME OVER) | E4 / S4.9 | ✓ |
| FR39 | Restart without title-screen detour | E4 / S4.9 Retry button | ✓ |
| FR40 | Photo Mode only from post-run/death | E9 / S9.2 | ✓ |
| FR41 | Free-cam + DoF + time-frozen | E9 / S9.3 + S9.4 | ✓ |
| FR42 | PNG export 16:9 / 9:16 / 1:1 | E9 / S9.5 | ✓ |
| FR43 | Pause on focus-loss | E3 / S3.4 | ✓ |
| FR44 | Persist to OS save location | E4 / S4.6 | ✓ |
| FR45 | First-launch default save | E4 / S4.6 (+ recovery E5 / S5.6) | ✓ |
| FR46 | Atomic save writes | E4 / S4.6 | ✓ |
| FR47 | Cross-platform binary (Win/Linux/macOS arm64+Intel) | E1 / S1.5 + E4 / S4.10 + E7 / S7.6 | ✓ |
| FR48 | macOS code-sign + notarize | E10 / S10.10 (**optional stretch**) | ⚠ **Conditional** — ships unsigned unless Till commits to €99/yr (per memory + epics.md deferral chain E4→E7→E10) |
| FR49 | Toon shading + outlines | E2 / S2.3 + S2.4 (fallback S2.7) | ✓ |
| FR50 | Semantic accent colors | E2 / S2.2 + E4 / S4.5 | ✓ |

**Coverage Statistics:**

- Total PRD FRs: **50**
- FRs fully covered: **49**
- FRs constraint-covered: **1** (FR28 — cross-epic design constraint)
- FRs conditionally covered: **1** (FR48 — optional stretch, consciously waived per Till)
- FRs missing / orphaned: **0**
- Coverage percentage: **100%** (accepting FR28 constraint-treatment and FR48 stretch-treatment)

### Missing Requirements (none — all FRs accounted for)

No FR is orphaned. FR48's "conditional stretch" status matches Till's documented hobby-cadence acceptance (see memory: *FR48 macOS signing waived / stretch*).

### Reverse-Check (FR coverage of NFRs explicitly mapped in epics.md)

The epics include explicit NFR tags at epic headers. Collected NFR coverage:

| NFR | Covered by | Status |
|---|---|---|
| NFR-P1 (60 FPS) | E10 / S10.1 | ✓ |
| NFR-P2 (title ≤10 s) | E10 / S10.3 | ✓ |
| NFR-P3 (title→game ≤5 s) | E10 / S10.3 | ✓ |
| NFR-P4 (no >100 ms hitches) | E10 / S10.1 + S10.2 | ✓ |
| NFR-P5 (<4 GB RAM) | E10 / S10.1 | ✓ |
| NFR-R1 (zero-crash across journeys) | E10 / S10.11 + S10.12 | ✓ |
| NFR-R2 (atomic save) | E4 / S4.6 | ✓ |
| NFR-R3 (graceful missing save) | E5 / S5.6 | ✓ |
| NFR-R4 (meta never lost) | E5 / S5.6 + E6 / S6.7 + E7 / S7.4 | ✓ |
| NFR-A1 (colorblind safe) | E2 / S2.2 | ✓ |
| NFR-A2 (no color-only info) | E8 / S8.3 (radar position+color) | ✓ |
| NFR-A3 (HUD legibility 60–80 cm) | E10 / S10.5 | ✓ |
| NFR-U1 (5-min aha) | E10 / S10.12 | ✓ |
| NFR-U2 (HUD simultaneous) | E5 / S5.4 | ✓ |
| NFR-U3 (at-a-glance subsystems) | E5 / S5.4 | ✓ |
| NFR-L1 (English-only) | E10 / S10.4 | ✓ |
| NFR-L2 (German post-MVP) | E10 / S10.4 (schema-ready) | ✓ |
| NFR-L3 (external string table) | E10 / S10.4 | ✓ |

**NFR Coverage: 18 / 18 (100%).**

### Design-Principle Traceability Check

Beyond numbered FRs, the PRD names 5 Design Principles as load-bearing. Re-checking from step 2 PRD analysis:

| Principle | Implementation site | Status |
|---|---|---|
| 1. No tutorial text | Epic 3 constraint; S4.7 title screen has no tutorial; S4.9 PostRun no "GAME OVER" | ✓ |
| 2. No visible numeric score | HUD design in S3.11/S5.4/S6.5 shows salvage/meta only, no score counter | ✓ |
| 3. **Asteroid motion predictable, not random** (Kepler / splines) | **NOT IMPLEMENTED.** All asteroid stories (S3.3, S6.3) spawn asteroids as `RigidBody::Static` — they do not move. S6.10 converts them to `Dynamic` only when tractored. | 🔴 **GAP — see note below** |
| 4. Death is feedback, not punishment | S4.9 PostRun explicitly forbids "GAME OVER" / red fill / defeat music | ✓ |
| 5. Graceful degradation at novel points | S2.6/S2.7 toon fallback; PRD risk section documents pacifism / audio / cockpit-only degradation paths | ✓ |

**Principle-level finding:** PRD Design Principle 3 (*"Asteroid motion is predictable, not random. Asteroids follow configurable trajectories — Kepler-like orbits or scripted splines, not randomized paths"*) is **not reflected** in any epic or story. The MVP ships static asteroids. This may be an intentional scope-reduction consistent with Till's reduced-MVP pattern (memory: *Staged rollout preference*), but it contradicts a principle the PRD flags as "re-raise if violated." **Decision needed** at step-5 or explicit acknowledgement that MVP deviates from Principle 3 and a defer-target (post-MVP?) should be recorded.

### Over-Coverage Check (scope-additions not tracked to PRD FRs)

Two scope-additions documented in story text, each anchored to a PRD cue or a Till-dated decision:

1. **Damage-direction indicator** (S8.5 + S8.4 damage-hit stinger) — not an FR, added 2026-04-22 during E5 decomposition. Supports Journey 3 ("what killed me, where was it") which is implicitly in PRD but not an FR. ✓ reasonable.
2. **Shield-absorb VFX** (S10.6) — not an FR, added 2026-04-22. Supports Design Principle 4 (feedback not punishment) with visceral damage-feedback that fits toon aesthetic. ✓ reasonable.
3. **Abort-forfeit policy** (S6.7) — not an FR. Anti-abuse design decision: Aborted runs do NOT bank salvage. Documented explicitly. ✓ reasonable.
4. **F3 dev FreeOrbit camera** (S9.1) — dev tooling behind `cfg(debug_assertions)`; shares impl with PhotoMode. Does not violate FR8 in release builds. ✓ reasonable.

No orphan epic content requiring PRD amendment.

## Step 4: UX Alignment Assessment

### UX Document Status

**Not Found — intentionally omitted** (confirmed with Till 2026-04-22).

UX responsibility is distributed across three other documents, as epics.md §"UX Design Requirements" (lines 211–219) records explicitly:

| UX concern | Source document / section |
|---|---|
| Principles & tonal direction | `prd.md` → *Design Philosophy* (5 principles + Cosmic-Mystery tonal anchor) |
| User journeys & acceptance cues | `prd.md` → *User Journeys* (4 arcs, cross-journey capability inventory) |
| HUD / rendering strategy | `architecture.md` → *Rendering & Visual Architecture* (hybrid screen-space + world-space HUD, "scientific-instrument panel over military HUD") |
| Menu system & UI framework choice | `architecture.md` → *UI, Menu & Debug Architecture* (bevy_ui + States; egui dev-only) |
| Per-FR UI module map | `architecture.md` → FR traceability table (FR4, FR22, FR24, FR25, FR26, FR31, FR33, FR36, FR37, FR38, FR40–FR42 all mapped to specific `src/ui/*` modules) |
| String-table discipline | `architecture.md` (dot-scoped keys, hot-reload) → epic E10/S10.4 |

### UX ↔ PRD Alignment

Architecture's HUD strategy and module breakdown support every FR and Design Principle surfaced by the PRD:

- FR22 radar → `src/ui/hud_cockpit.rs` world-space radar mesh on cockpit frame (scientific-instrument styling = Design-Philosophy match).
- FR24 HUD core → `src/ui/hud.rs` screen-space bars, matches NFR-U2 (simultaneous visibility) and NFR-U3 (at-a-glance).
- FR38 PostRun → `src/ui/post_run.rs`, and S4.9 acceptance criterion explicitly prohibits "GAME OVER" / red overlay / defeat music per Design Principle 4.
- FR40–FR42 Photo Mode → `src/ui/photo_mode.rs` + shared `FreeOrbitCamera` with dev F3 (E9/S9.1).
- FR26 headphone splash → `src/ui/main_menu.rs` first-launch splash (S8.7).

### UX ↔ Architecture Alignment

- Hybrid HUD (screen-space for bars, world-space for cockpit instruments) consistent across Architecture, Epics (S3.11 screen-space baseline → S5.4 bars → S8.3 radar world-space → S8.6 world-space yield-delta).
- Performance: World-space HUD meshes budgeted inside the 60-FPS target (NFR-P1) — audited in E10/S10.1.
- Legibility: HUD text audited at 60–80 cm / 1080p in E10/S10.5 (NFR-A3).
- Localization-readiness: all user-facing strings externalized via `tr()` (NFR-L3 / S10.4). Dot-scoped keys architecturally mandated from M0.

### Warnings / Gaps

1. **No UX document, by design** — standard in a single-player cockpit-only game where PRD+Architecture carry the UX load. **Not a defect.** Flagged per step-4 protocol for transparency only.
2. **Motion-sickness mitigation explicitly absent** in MVP (NFR-A accessibility scope boundary). PRD documents this as an accepted trade-off (E#6 Phase-3 resolution). No architectural or epic gap — acceptance is documented.
3. **Full key / button rebinding deferred** to post-MVP. Input abstraction (`leafwing-input-manager`, Action enums) architecturally preserves the retrofit path per architecture decisions + Story 6.12 ADR reference. No MVP gap.
4. **Closed captions for spatial audio cues** deferred to post-MVP. Directional mechanic cannot be preserved by text — PRD explicitly frames this as "not applicable." No MVP gap.
5. **Design Principle 3 (predictable asteroid motion) un-wired** — already flagged in step 3; also a UX/feel concern (player-skill-as-trajectory-reading depends on motion).

**No alignment gaps between UX (as distributed), PRD, and Architecture.** The principled UX substitution is coherent.

## Step 5: Epic Quality Review

### Per-Epic Compliance Matrix

| Epic | User value | Independence | Story sizing | AC quality | Deps | Notes |
|---|---|---|---|---|---|---|
| E1 Foundation | 🟡 dev-foundational (see note 1) | ✓ | ✓ | ✓ | ✓ | Starter-Template mandate from Architecture |
| E2 Tech Spike | 🟡 dev/tech-spike (see note 1) | ✓ (uses E1 only) | ✓ | ✓ | ✓ | S2.7 correctly conditional on S2.6 |
| E3 First Playable | ✓ | ✓ (uses E1, E2) | ✓ | ✓ | ✓ | 11 stories, well-sliced |
| E4 Stop-Ship | ✓ (shippable) | ✓ (uses E1–E3) | 🟡 S4.6 heavy (see note 2) | ✓ | ✓ | FR48 deferred E4→E7→E10 |
| E5 Ship State | ✓ | ✓ | ✓ | ✓ | ✓ | 6 stories, clean refactor of E4 Health |
| E6 Caravan | ✓ | ✓ | 🟡 S6.10 heavy, S6.13 balanced (see note 2) | ✓ | ✓ | Danger-Stretch acknowledged, sub-milestoned |
| E7 Roguelite | ✓ | ✓ | ✓ | ✓ | 🟡 S7.5 re-fixed in E8 (see note 3) | Unlocks catalog = 8 at M6 |
| E8 Perception | ✓ | ✓ | ✓ | ✓ | 🟡 S8.5 modifies E5 event (see note 4) | Fixes S7.5 wiring; extends S5.3 |
| E9 Photo Mode | ✓ | ✓ | ✓ | ✓ | ✓ | Shares FreeOrbitCamera with dev F3 |
| E10 Polish | ✓ | ✓ | 🟡 S10.7 adds v5→v6 save bump, could combine | ✓ | ✓ | 12 stories; S10.10 optional stretch |

### 🔴 Critical Violations

**None.** No technical-milestone epics that break the user-value rule (E1/E2 are explicit tech-spike / starter-gate exceptions justified by greenfield Rust/Bevy + architecture's Hybrid-Manual starter-decision + PRD's M1 gate). No forward-dependencies. No stories requiring future stories to function.

### 🟠 Major Issues

1. **Design Principle 3 not implemented (carried over from Step 3).**
   - PRD Principle 3: *"Asteroid motion is predictable, not random — Kepler-like orbits or scripted splines."*
   - Every asteroid-spawning story (S3.3, S6.3) uses `RigidBody::Static`. Asteroids never move in MVP except when tractored (S6.10).
   - **Impact:** The "reading trajectories" skill the PRD promises is absent from MVP.
   - **Remediation options:**
     - (a) Add a story to E6 (or E10) that spawns asteroids with `RigidBody::Kinematic` + scripted-spline motion (minimal-viable Principle-3 compliance).
     - (b) Retract / defer Principle 3 explicitly in the PRD: reduced-MVP accepts static asteroids, Principle 3 becomes Growth-stage.
     - Till's staged-rollout preference (memory: *feedback_staged_rollout*) suggests (b) is likely the preferred call, but it needs explicit capture.

2. **FR48 coverage is conditional.**
   - S10.10 is labeled "Optional Stretch" — ships unsigned unless Till commits €99/yr Apple Developer Account at M9.
   - **Consistent with memory** (*FR48 macOS signing waived / stretch*), so formally acceptable.
   - **Gate impact:** M9 MVP ships with FR48 formally unsatisfied in the "no €99" branch. Explicit acknowledgement is in Story AC; PRD has not been retroactively softened to match. **Recommendation:** note the deferral in the PRD itself (or an addendum) so the MVP definition is internally consistent.

### 🟡 Minor Concerns

1. **E1 and E2 are dev-foundational (no player user value).** Acceptable for a Bevy greenfield with an explicit Hybrid-Manual starter gate (E1) and a scheduled M1 shader tech-spike (E2) — the PRD and Architecture both mandate these as milestones. Standard BMad best-practice says avoid technical epics; this project justifies the exception. Note, don't fix.

2. **Heavy stories vs. 4–8 h/week hobby budget.**
   - S4.6 (PersistencePlugin + Save Schema v1) has 9 AC blocks covering atomic write, load success paths, fallback for missing/corrupt/interrupted data. Single-story scope ≈ 8–12 h. Consider slicing into S4.6a (write+load happy path) + S4.6b (recovery & atomicity) if week-4 estimates slip.
   - S6.10 (Tractor-Beam) has 7 AC blocks covering acquisition / pull / visuals / capture / release / state-transitions. Similar scope.
   - These align with Till's acceptance that week-long stories are fine given cadence. Not blockers.

3. **Cross-epic retro-modifications (documented but coupling).**
   - S8.1 "fixes" S7.5's Effects-Wiring interpretation of `DetectionRangeMult` (player sensor range vs enemy AI detection).
   - S8.5 extends S5.3's `DamageApplied` event with a `damage_origin: Option<Vec3>` field.
   - Both are explicit and dated. Future readers of E5 or E7 in isolation may miss these extensions. **Recommendation (optional):** add back-references in E5/E7 pointing to the extension locations. Low-priority cleanup.

4. **Save schema version bump frequency.**
   - SaveData versions: v1 (S4.6) → v2 (S6.2 tutorial_complete) → v3 (S7.2 unlocks HashMap refactor) → v4 (S8.7 headphone_splash_shown) → v5 (S9.6 watermark_enabled) → v6 (S10.7 ambient_volume).
   - Six schema migrations across MVP. All covered by S5.6's migration scaffold — no orphan migration. Minor concern: migration test coverage is only scaffolded in S5.6; each subsequent bump should include a unit test for its `v_{n-1}_to_v_n` fixture. **Recommendation:** add a standing AC "include migration test fixture for this version bump" in S6.2, S7.2, S8.7, S9.6, S10.7.

### Best-Practices Compliance Checklist (per epic summary)

- [✓] Epic delivers user value — E1/E2 with documented greenfield/tech-spike justification
- [✓] Epic can function independently (reading back only) — all 10 verified
- [✓] Stories appropriately sized — 3 stories borderline-heavy, all within hobby-acceptable scope
- [✓] No forward dependencies — verified across all 86 stories
- [✓] Database/save schema created when needed — per-epic schema increments, not upfront
- [✓] Clear acceptance criteria — BDD Given/When/Then consistent throughout
- [✓] Traceability to FRs maintained — explicit FR Coverage Map + epic-header FR lists
- [✓] Starter Template requirement — S1.1 "Bootstrap Cargo Project with Hand-Authored Cargo.toml" matches Architecture Hybrid-Manual decision
- [✓] Greenfield indicators — CI from M0 (S1.4), toolchain config (S1.3), initial project setup (S1.1–1.2)

### Remediation Priority

| Priority | Item | Action |
|---|---|---|
| P1 | Design Principle 3 gap | Decide (a) add motion story OR (b) defer principle in PRD. **Blocking for clean M3→M9 coherence.** |
| P2 | FR48 conditional coverage | Note deferral in PRD or accept as addendum. Non-blocking, cosmetic. |
| P3 | Migration test fixtures per schema bump | Add standing AC or checklist. Non-blocking. |
| P4 | Back-references for cross-epic retro-mods | Optional documentation tidy. Non-blocking. |

## Step 6: Summary and Recommendations

### Overall Readiness Status

**READY. P1 resolved 2026-04-22 — proceed to implementation.**

The planning artifacts (PRD, Architecture, Epics, 86 Stories) form a coherent, traceable, implementation-ready package for a hobby-cadence Rust/Bevy solo project. All 50 FRs and 18 NFRs have traceable implementation paths. No critical best-practice violations. The outstanding **design-principle trade-off (Principle 3 asteroid motion)** was resolved via option (b): PRD amended to defer predictable-motion to Growth stage.

### Critical Issues — Resolved

1. ~~**Decide Design Principle 3 (predictable asteroid motion).**~~ **✅ Resolved 2026-04-22 via option (b).**
   - PRD *Design Principles* §3 amended: MVP ships static asteroids by explicit deferral; rationale recorded inline.
   - PRD *Growth Features* amended: new item #11 "Asteroid motion — Kepler orbits / scripted splines (Design Principle 3 restoration)" with elevated strategic priority (first or second post-MVP slot per M6 playtest signals).
   - No epic / story changes required. Epics as-authored remain valid.

### Non-Blocking P2–P4 Items

- **P2:** FR48 deferral chain (E4→E7→E10→waived) should be noted in the PRD or a short addendum so MVP definition is internally consistent with Till's €99/yr waiver.
- **P3:** Add a standing AC pattern to schema-bumping stories (S6.2, S7.2, S8.7, S9.6, S10.7) requiring a `v_{n-1}_to_v_n` migration test fixture.
- **P4:** Add optional back-references in E5 and E7 pointing at E8 retro-modifications (S8.5 adds `damage_origin`; S8.1 corrects S7.5 wiring).

### Recommended Next Steps

1. ~~**Decide on P1 (Principle 3).**~~ ✅ **Done 2026-04-22 — option (b).** PRD amended inline.
2. **Resolve P2 (FR48 PRD note)** — 10-minute PRD patch or addendum. Optional if status-quo is acceptable.
3. **Proceed to Phase 4 / implementation.** Start with E1 (M0 foundation gate). No structural changes to planning artifacts required.
4. **(Mid-implementation)** Apply P3/P4 cleanups during the first schema-bumping story (E6/S6.2) rather than as a dedicated artifact-maintenance session.

### Evidence of Quality

- **100% FR coverage** (50/50, accepting FR28 constraint-treatment and FR48 stretch-treatment).
- **100% NFR coverage** (18/18) with explicit epic/story anchors.
- **Clean epic independence** — no forward dependencies across 10 epics.
- **Consistent BDD acceptance criteria** across 86 stories.
- **Greenfield best-practices observed** — starter-template gate at E1/S1.1, CI matrix at M0 (S1.4), per-epic schema increments (no upfront DB/save buildout).
- **Hobby-cadence realism** — M5 "Danger Stretch" explicitly sliced into weekly sub-milestones; scope discipline documented throughout.
- **Tonal / Design-Philosophy compliance** — no "GAME OVER" mandate (S4.9), information-design discipline (FR28 as cross-epic constraint), scientific-instrument HUD styling (S5.4, S8.3, S10.8).

### Final Note

This assessment identified **5 issues** across **3 categories** (requirements gap, coverage conditional, documentation tidy). Exactly **one issue (Design Principle 3) warrants a decision before implementation begins**; the other four are non-blocking and can be addressed opportunistically during implementation. The artifact stack is unusually thorough for a solo hobby project — strong traceability, explicit trade-offs, named deferrals, and motivation-preservation structure throughout. **Proceed to implementation after resolving P1.**

---

**Assessor:** Claude (BMad Implementation Readiness skill), 2026-04-22
**Project:** asteriods3D
**User:** Till Fechteler





