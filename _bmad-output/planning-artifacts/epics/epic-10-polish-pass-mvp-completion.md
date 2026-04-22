# Epic 10: Polish Pass & MVP Completion

Balance tuning, profiling to 60 FPS, asset-load audit, audio pass-2 (SFX + ambient), UI polish (instrument-panel refinement), 3 additional unlock definitions, crash-fix backlog, string-table audit, shield-absorb VFX, optional macOS codesign (FR48 stretch), 4-journey playtest validation across Windows + macOS + Linux-CI. Full polished MVP ready for Itch.io / Steam release. M-alignment: M9 🏁. FRs covered: FR48 (optional stretch per Till 2026-04-22). NFRs: P1, P2, P3, P4, P5, R1, U1, A3, L1, L3.

## Story 10.1: Performance Profiling + 60 FPS Target on Reference Hardware

As a player,
I want 60 FPS sustained on reference hardware during steady-state gameplay,
So that NFR-P1/P4/P5 are met and the combat loop feels responsive.

**Acceptance Criteria:**

**Given** PRD reference baseline (GTX 1060 / RX 580 / Apple M1)
**When** profiling runs per Till's hardware access 2026-04-22
**Then** profiling runs on:
- Windows machine (Till's Dev/Test) — primary Windows validation
- Mac M5 Pro (Till's Dev) — Apple Silicon validation (note: M5 Pro ≥ M1 baseline; M1 parity is inferred, not hardware-verified — flagged as known gap)
- Linux: CI-based verification only (no local hardware per Till)

**And** tracy-client + cargo flamegraph capture frame-time, system breakdown, allocations

**Given** a Caravan run with steady-state combat (pocket active, tractor on, 3 asteroids visible, 1 enemy attacking)
**When** 60-second continuous capture runs
**Then** sustained frame time ≤ 16.67 ms (60 FPS) on Windows + macOS
**And** 99th-percentile frame time ≤ 20 ms
**And** no individual frame exceeds 100 ms (NFR-P4 hard gate)

**Given** memory profiling
**When** the game has run for 10 min
**Then** process memory ≤ 4 GB (NFR-P5)
**And** no unbounded growth

**Given** state transitions
**When** each is timed
**Then** transition hitches ≤ 50 ms (NFR-P4 transition allowance)
**And** any exceeding 50 ms feeds into Story 10.3

**Given** identified hotspots
**When** fixes are applied (system parallelization, Query caching, LOD, etc.)
**Then** re-profile confirms targets met

## Story 10.2: Asset-Load-at-State-Entry Audit + Typed Resource Wrappers

As a developer,
I want all asset loading to happen at state-entry via typed Resource wrappers,
So that NFR-P4 steady-state hitches are eliminated by moving IO to discrete transition boundaries.

**Acceptance Criteria:**

**Given** scattered `AssetServer::load` calls accumulated across Epics 2–9
**When** Story 10.2 audits
**Then** `docs/perf/asset-loading-audit.md` lists every load-call location + lifecycle (state-entry vs mid-state)

**Given** the audit identifies scattered loads
**When** refactored
**Then** all asset loads are consolidated into `OnEnter(GameState::X)` systems
**And** handles are stored in typed Resource wrappers (`AsteroidModels`, `WeaponSounds`, `CockpitMesh`, `ShipModel`, etc. per architecture)
**And** gameplay systems read via `Res<TypedWrapper>`, not ad-hoc `AssetServer::load`

**Given** refactored pipeline
**When** performance re-profiles (Story 10.1 post-refactor)
**Then** no frame hitches attributable to asset loading during steady-state

**Given** architecture "Asset loading gated at OnEnter(State)" principle
**When** Story 10.2 closes
**Then** `AssetServer::load` calls exist ONLY in OnEnter systems (grep-verified)
**And** hot-reload for dev-time shader/asset edits still works

## Story 10.3: Load-Time Budget — Title ≤10s, Title→Gameplay ≤5s

As a player,
I want the game to reach title within 10s and gameplay within 5s of "Start Run",
So that NFR-P2 and NFR-P3 are met.

**Acceptance Criteria:**

**Given** the app is launched on reference SSD-based hardware
**When** timer starts at launch
**Then** title screen visible within 10 s (NFR-P2), verified on Windows + macOS

**Given** the player clicks "Start Run"
**When** timer starts at click
**Then** Caravan gameplay rendering within 5 s (NFR-P3), verified on Windows + macOS

**Given** a target miss
**When** investigation runs
**Then** bottlenecks identified (glTF asset sizes, shader compilation, font loading, plugin init overhead)
**And** fixes applied: lazy-load non-critical assets, pre-compile shaders in Loading, reduce default asset size

**Given** deterministic measurement
**When** `#[cfg(debug_assertions)]` dev telemetry prints startup time on each launch
**Then** load times are tracked across builds for regression detection

## Story 10.4: String-Table Audit — No Hard-Coded Player-Facing Strings

As a developer,
I want all player-facing strings externalized to `assets/strings/en.ron`,
So that NFR-L3 is met and future localization (NFR-L2 German, Post-MVP) is an asset-swap.

**Acceptance Criteria:**

**Given** Epics 1–9 accumulated literal strings (e.g., "Start Run", "Run ended", "Easy / Medium / Hard", "SHIELDS", etc.)
**When** Story 10.4 audits
**Then** `docs/i18n/string-audit.md` lists every player-facing literal + location + context

**Given** the audit identifies hard-coded strings
**When** refactored
**Then** all are moved to `assets/strings/en.ron` with dot-scoped keys (`ui.menu.start_run`, `ui.postrun.run_ended`, `ui.difficulty.easy`, etc.) per architecture convention
**And** consumers use `fn tr(key: &str) -> String` via `Res<StringTable>`
**And** StringTable is loaded in Loading state (Story 1.6)

**Given** NFR-L1 MVP ships English-only
**When** the catalog is finalized
**Then** `en.ron` is the sole string file included
**And** NFR-L2 German is deferred to Post-MVP (schema ready, no additional file)

**Given** dev-time changes
**When** `en.ron` is edited during `cargo run`
**Then** bevy_asset hot-reload refreshes strings without restart

**Given** audit closure
**When** grep for common UI patterns runs on `src/**`
**Then** returns no hardcoded UI strings (all routed through `tr`)
**And** unknown-key `tr()` calls emit a warn! at runtime

## Story 10.5: HUD Legibility Audit — 60–80 cm @ 1080p

As a player,
I want all HUD text legible at 60–80 cm viewing distance on 1080p,
So that NFR-A3 is met.

**Acceptance Criteria:**

**Given** HUD elements from Epics 3, 5, 6, 7, 8
**When** Story 10.5 audits
**Then** `docs/ui/legibility-audit.md` records font sizes, contrast ratios, and layout per element at 1920×1080 on Windows + macOS

**Given** NFR-A3 target
**When** each element is evaluated
**Then** min font size ≥ 18 px
**And** min contrast ratio ≥ 4.5:1 against background (WCAG AA)

**Given** elements fail the audit
**When** remediation runs
**Then** fonts bumped OR backgrounds darkened OR outlines/shadows added
**And** re-audit confirms pass

**Given** Till's solo playtest at arm's-length distance on both reference machines
**When** he reads each HUD element during combat
**Then** any strained element gets a targeted fix

## Story 10.6: Shield-Absorb VFX (Toon-Style Flash)

As a player,
I want a brief toon-style flash when my shield absorbs a hit,
So that damage-feedback is visceral and fits the aesthetic (E5 scope-addition).

**Acceptance Criteria:**

**Given** Story 5.3's `DamageApplied` event (with `shield_damage > 0`)
**When** Story 10.6 adds the VFX system
**Then** a brief flash triggers on the PlayerShip mesh
**And** the flash uses ToonMaterial by temporarily tinting cyan (SemanticAccent::PlayerOwned) for ~150 ms with an ease-out curve

**Given** component-based implementation (preferred over shader extension)
**When** implemented
**Then** a `ShieldFlashTimer { remaining: f32 }` component is attached to PlayerShip during the flash
**And** the system modifies ToonMaterial's `tint` uniform during the flash

**Given** another hit during active flash
**When** observed
**Then** timer resets to full duration (last-damage-wins, not additive)

**Given** idle state (no active flash)
**When** the system runs
**Then** zero per-frame cost on PlayerShip material

**Given** Design Principle 4 (no defeat overlays)
**When** the flash is tuned
**Then** it's brief and tasteful, NOT aggressive red full-screen strobe
**And** Till's solo playtest confirms "shield absorbed" readability without alarm

## Story 10.7: Audio Pass-2 — SFX Mixing + Ambient Drone Bed

As a player,
I want the audio mix polished with an ambient background drone,
So that Design Philosophy "cosmic mystery" atmosphere is realized.

**Acceptance Criteria:**

**Given** placeholder SFX from Story 8.4
**When** Story 10.7 polishes
**Then** all weapon/impact/cue SFX are replaced with higher-quality samples (royalty-free from Freesound.org or similar; sources recorded in `docs/audio/asset-sources.md`)
**And** SFX volumes are mixed so ambient doesn't mask alerts (detection chirps, damage stingers)

**Given** Story 8.2's AmbientChannel is scaffolded but empty
**When** Story 10.7 populates it
**Then** an ambient drone loop (source TBD: Freesound / Suno / self-composed per Till's future decision) plays in gameplay states (Arena, Caravan, PhotoMode)
**And** the loop is 30–60 s with seamless looping
**And** ambient volume subtle by default (~30% of SfxChannel)

**Given** Story 4.8's master + SFX sliders
**When** Story 10.7 extends settings
**Then** an "Ambient" slider is added (0.0–1.0, default 0.3) mapped to AmbientChannel
**And** persisted in SaveData (SaveData.version bumps v5 → v6 with a migration via Story 5.6's scaffold injecting default ambient_volume)

**Given** a solo playtest Caravan run with ambient active
**When** Till evaluates
**Then** drone doesn't compete with alerts or weapon SFX
**And** overall mix matches "cosmic mystery" aesthetic

## Story 10.8: UI Polish — Scientific-Instrument Styling Refinement

As a player,
I want the HUD to read like a cohesive scientific-instrument panel, not placeholder dev UI,
So that Design Philosophy is realized.

**Acceptance Criteria:**

**Given** HUD elements from Stories 3.11, 5.4, 6.4, 6.5, 6.12, 8.3, 8.5, 8.6
**When** Story 10.8 polishes
**Then**:
- Fonts switch to a consistent monospace / scientific-display font (placeholder OK if final asset is post-MVP)
- Bar borders, tick marks, corner caps added to shield/hull/boost bars
- Waypoint pointer mesh replaced with a more considered arrow/chevron
- Radar disc gains subtle grid lines and optional rotating sweep animation (defer if time-constrained)
- HUD backgrounds get semi-transparent dark panels for contrast
- Color palette consistently applied (SemanticAccent for data-carrying, neutral grays for chrome)

**Given** architecture "Hybrid HUD" guidance
**When** review runs
**Then** screen-space bevy_ui for menus/bars; world-space cockpit meshes for radar, waypoint, yield-delta, damage-direction — all on correct layers

**Given** Story 10.5 legibility
**When** polish applies
**Then** no legibility regression — styling maintains or improves contrast

**Given** MVP shipping
**When** Till does a solo visual-quality check against reference games (Elite Dangerous, Everspace)
**Then** HUD reads as "designed", not "placeholder"

## Story 10.9: 3 Additional Unlock Definitions

As a player,
I want 3 additional unlocks beyond Epic 7's 8,
So that the meta-progression catalog has more depth (Till selected 3 on 2026-04-22).

**Acceptance Criteria:**

**Given** Epic 7 Story 7.2's catalog of 8
**When** Story 10.9 extends
**Then** 3 entries are added:
- `dampener_strength` — `DampenerStrengthMult(1.5)` — max=1 — base_cost=150
- `wider_tractor_cone` — `TractorConeWidenDelta(15°)` — max=1 — base_cost=250
- `projectile_speed` — `ProjectileSpeedMult(1.2)` — max=1 — base_cost=300

**Given** new UnlockEffect variants
**When** Story 7.5's effects-wiring is extended
**Then** each new effect applies:
- `DampenerStrengthMult(m)` → `effective_dampener_linear_strength × m^N` AND `effective_dampener_angular_strength × m^N`
- `TractorConeWidenDelta(d)` → `effective_tractor_cone_angle += d * N` (NEW TuningConfig field `tractor_cone_angle: f32 = 30.0` — formalizes the cone gate; retroactively tightens Story 6.10's logic from "absolute closest to aim" to "closest within ±cone/2 of aim")
- `ProjectileSpeedMult(m)` → `effective_projectile_speed × m^N` (applied uniformly to all weapon archetypes)

**Given** Story 6.10's tractor acquisition is retroactively formalized with the cone gate
**When** re-tested
**Then** default `tractor_cone_angle = 30°` is wide enough that MVP-Epic-6 behaviour is preserved (player doesn't notice tightening)
**And** the unlock widens it to 45° for stacked users

**Given** the catalog total is now 11 (8 from E7 + 3 from E10)
**When** the shop UI (Story 7.3) renders
**Then** all 11 display with same format

## Story 10.10: macOS Codesign + Notarization (Optional Stretch)

As the project author,
I want macOS codesign + notarization available as an optional stretch,
So that FR48 is achievable at M9 — otherwise MVP ships unsigned permanently per Till's decision (c) 2026-04-22.

**Acceptance Criteria:**

**Given** this is an optional stretch story (Till's decision (c): "wenn Steam-Release realistisch, €99 zahlen; wenn Itch.io-only, unsigned ist fein")
**When** Till opts to proceed
**Then** an Apple Developer Account (€99/year) is active
**And** `codesign` is applied to the macOS universal binary using a Developer ID Application certificate
**And** `xcrun notarytool submit` uploads notarization
**And** `xcrun notarytool wait` confirms success
**And** `xcrun stapler staple` attaches the ticket

**Given** signing integrated into GitHub Actions
**When** Story 10.10 proceeds
**Then** release.yml's macOS job adds signing + notarization steps
**And** Apple certificates + App-Specific Password are stored as encrypted GitHub Actions secrets
**And** the signed universal binary replaces the unsigned variant in the release artifact

**Given** Till opts NOT to implement
**When** M9 gate is evaluated
**Then** MVP ships unsigned macOS (universal, right-click-open per existing runbook)
**And** FR48 remains formally unsatisfied (pragmatic trade for Hobby release)
**And** the full deferral chain (E4 → E7 → E10 → waived) is documented

**Given** post-M9 revisit scenario
**When** Till later decides to commit
**Then** this story's implementation plan is ready to execute (no additional design work needed)

## Story 10.11: Crash-Fix Backlog from M6–M8 Playtesting

As a player,
I want zero crashes during normal play across all four user journeys,
So that NFR-R1 is met.

**Acceptance Criteria:**

**Given** M6–M8 playtesting reveals bugs and crashes
**When** each crash is filed per Till's decision 2026-04-22
**Then** it becomes a GitHub Issue with: repro steps, stack trace (from Story 1.8 panic-hook log), OS + version, severity (crash / freeze / visual glitch)

**Given** the issue backlog
**When** Story 10.11 triages
**Then** all crash-severity issues are P1 must-fix before M9 gate
**And** other severities are triaged (fix if easy, defer to Post-MVP if invasive)

**Given** P1 fixes are applied
**When** Story 10.11 closes
**Then** crash-severity issue count = 0

**Given** a fix introduces a new issue
**When** detected
**Then** the new issue joins the backlog
**And** Story does not close until resolved

## Story 10.12: 4-Journey Playtest Validation — 3 Platforms, 60 FPS, Zero-Crash

As the project author,
I want the MVP to pass a structured solo playtest across the 4 PRD user journeys on all 3 platforms,
So that the M9 completion gate is met.

**Acceptance Criteria:**

**Given** PRD User Journeys (4 arcs)
**When** Story 10.12 runs solo playtest per Till's decision 2026-04-22
**Then** all 4 journeys complete end-to-end:
- Journey 1 (first-launch tutorial Arena)
- Journey 2 (first full Caravan run — success)
- Journey 3 (death + retry loop — permadeath, PostRun, retry without menu-detour)
- Journey 4 (meta-progression — earn / bank / buy / see effect next run)

**Given** FR47 3-platform coverage
**When** platform playtesting runs
**Then**:
- **Windows**: full playtest on Till's Windows machine ✅
- **macOS**: full playtest on Mac M5 Pro (note: Apple Silicon; M5 Pro newer than M1 — M1 parity inferred, not hardware-verified, flagged as known gap)
- **Linux**: CI-based smoke test on `ubuntu-latest` (build + minimal startup test only, no full playtest — Till has no Linux hardware per 2026-04-22)

**And** Linux gap is explicitly documented as known limitation — Linux release is best-effort via CI-validated build

**Given** per-session metrics
**When** captured
**Then**:
- Frame rate 60 FPS sustained (NFR-P1) via Bevy diagnostics overlay
- Zero crashes across 4 journeys (NFR-R1)
- NFR-U1 5-minute aha moment — Till simulates first-time-player perspective (external validation declined for solo MVP per Till's decision)

**Given** the playtest completes successfully on accessible platforms
**When** the M9 gate is evaluated
**Then** asteriods3D is declared MVP-ready
**And** release.yml produces final ZIPs per Story 4.10 / 7.6
**And** Itch.io publication follows the runbook (updated unsigned-or-signed depending on Story 10.10 decision)

**Given** the playtest uncovers new bugs
**When** they occur
**Then** they route to Story 10.11's backlog
**And** Story 10.12 re-runs until all 4 journeys pass clean

<!-- Epic 10 complete — 12 stories deliver M9 Polish + MVP Completion. All 10 Epics decomposed. Next: Step 4 final validation. -->

