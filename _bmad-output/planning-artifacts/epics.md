---
stepsCompleted: ['step-01-validate-prerequisites', 'step-02-design-epics', 'step-03-create-stories']
storiesCompletedForEpics: ['E1', 'E2', 'E3', 'E4', 'E5', 'E6', 'E7', 'E8', 'E9', 'E10']
resumeAt: 'step-04-final-validation'
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/planning-artifacts/prd-validation-report.md
  - _bmad-output/brainstorming/brainstorming-session-2026-04-21-1114.md
project_name: 'asteroids3D'
user_name: 'Till'
date: '2026-04-22'
---

# asteroids3D - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for asteroids3D, decomposing the requirements from the PRD and Architecture into implementable stories. No UX Design document exists (cockpit-only game; UX is embedded in PRD Design Philosophy + Architecture HUD strategy — confirmed with Till on 2026-04-22).

## Requirements Inventory

### Functional Requirements

**Flight & Controls (FR1–FR8)**
- FR1: Player can pilot a ship through 3D space via keyboard + mouse input.
- FR2: Player can translate the ship in six directions (thrust forward, reverse, lateral strafe left/right, vertical up/down).
- FR3: Player can rotate the ship around all three axes (pitch, yaw, roll).
- FR4: Player can aim weapons independently of ship heading (decoupled aim).
- FR5: Player can toggle an inertial dampener that modulates Newtonian drift against arcade tightness.
- FR6: Player can initiate a boost that temporarily increases thrust at the cost of a rechargeable resource.
- FR7: Player can tractor-beam intact asteroids and debris toward the ship for salvage pickup.
- FR8: Player views gameplay exclusively through a first-person cockpit view; no external camera toggle is available during active gameplay.

**Combat System (FR9–FR16)**
- FR9: Player can fire weapons that emit projectiles with ballistic trajectories.
- FR10: Player ship equips up to 3 weapons drawn from a pool of 3 prefab archetypes.
- FR11: Each fired projectile deducts a configurable amount from the player's salvage currency (pay-to-shoot economy).
- FR12: Projectiles can damage asteroids, enemy ships, and debris.
- FR13: Destroyed asteroids yield salvage at a configurably lower rate than intact asteroids captured via tractor beam.
- FR14: Enemy ships detect, pursue, and attack the player within a configurable engagement range.
- FR15: Player ship has Hull and Shields subsystems: Shields regenerate after a cooldown following damage; Hull does not regenerate during a run.
- FR16: When player Hull reaches zero, the run ends (permadeath).

**Economy & Salvage (FR17–FR21)**
- FR17: Player accumulates salvage currency during a run from intact pickups, destroyed asteroids, and other configurable sources.
- FR18: On run completion (successful or failed), banked salvage converts to persistent meta-currency at a configurable rate.
- FR19: Meta-currency persists across runs via save data.
- FR20: Player can spend meta-currency between runs at an unlock shop to acquire permanent upgrades.
- FR21: The unlock shop offers between 5 and 10 distinct permanent upgrades affecting ship capabilities.

**Perception & Sensors (FR22–FR26)**
- FR22: The cockpit HUD displays a radar showing threat markers, range, and relative direction of detected entities.
- FR23: The Game emits spatial stereo audio cues for enemies, hazards, and salvage-of-interest; cues indicate approximate direction.
- FR24: The cockpit HUD displays current shields, hull, ammunition status, and salvage-currency balance.
- FR25: The cockpit HUD indicates the economic yield delta between intact-capture and destroy for salvageable targets in view.
- FR26: On first launch, Game displays a splash screen recommending headphones for optimal spatial audio perception.

**Run Structure & Progression (FR27–FR35)**
- FR27: On the first session, Player is placed in a hand-designed Arena tutorial zone.
- FR28: Game presents no written tutorial text; all learning occurs through play and diegetic HUD cues.
- FR29: After completing the Arena tutorial, Player transitions to Caravan mode for subsequent runs.
- FR30: A Caravan run lasts 5 to 8 minutes from start to target destination.
- FR31: Caravan runs use a single MVP run-skeleton template with three selectable difficulty variants (easy, medium, hard).
- FR32: Caravan runs contain in-route combat pockets that trigger when the player enters configurable rendering-distance thresholds.
- FR33: Player can navigate to the run's target destination via a waypoint-pointer indicator rendered in the cockpit.
- FR34: On successful Caravan completion, Player banks accumulated salvage currency.
- FR35: On Caravan death, Player banks accumulated salvage currency; previously purchased meta-unlocks persist.

**UI & Feedback (FR36–FR43)**
- FR36: Player can access a title screen with options to start a new run, access settings, view credits, or quit.
- FR37: Player can adjust volume (master and SFX) and mouse sensitivity in a settings menu.
- FR38: On death, Player sees a post-run summary showing cause of death, salvage banked this run, and options to retry immediately, access Photo Mode, or return to menu. No "GAME OVER" overlay, no red screen, no defeat music (Design Principle 4).
- FR39: Player can restart a run immediately after death without returning to the title screen.
- FR40: Player can enter a Photo Mode accessible only from the post-run or death screen; it is not accessible during active gameplay.
- FR41: Photo Mode provides free-cam orbital/dolly movement, adjustable depth-of-field, and time-frozen simulation.
- FR42: Player can export Photo Mode screenshots as PNG images in 16:9 landscape, 9:16 portrait, and 1:1 square aspect ratios.
- FR43: Game pauses the simulation when Player opens the in-run pause menu or when the application window loses focus.

**Persistence & Platform (FR44–FR48)**
- FR44: Game persists meta-currency, unlocked upgrades, and settings to an OS-convention save location.
- FR45: On first launch, Game creates a default save file at the OS-convention save location.
- FR46: Save data survives unexpected termination (crash, force-quit, power loss) without corruption.
- FR47: Game runs as a native binary on Windows 10+, Linux (major distros: Ubuntu LTS, Fedora, Arch equivalents), and macOS (Apple Silicon and Intel x86_64).
- FR48: The macOS binary is code-signed and notarized for distribution outside the Apple App Store.

**Visual Presentation (FR49–FR50)**
- FR49: Game renders all 3D geometry using a toon-shading material with silhouette outlines.
- FR50: Game applies semantic accent colors to entity categories (enemies, salvage, hazards, player-owned) against a restrained base palette; accent colors are distinguishable under the vector aesthetic.

### NonFunctional Requirements

**Performance**
- NFR-P1: Sustained 60 FPS minimum at 1080p on reference hardware baseline (GTX 1060 / RX 580 / Apple M1) with vector aesthetic shader active.
- NFR-P2: Load from double-click to title screen within 10 seconds on reference hardware with SSD storage.
- NFR-P3: Title-screen to active Caravan gameplay within 5 seconds on reference hardware.
- NFR-P4: No visible frame hitches exceeding 100 ms during steady-state gameplay. Hitches up to 50 ms at transition boundaries are acceptable but logged.
- NFR-P5: Process memory usage during steady-state gameplay stays below 4 GB.

**Reliability**
- NFR-R1: Zero-crash during normal play across all four documented user journeys. Crash-free playtest is a milestone gate criterion at M3, M6, and M9.
- NFR-R2: Save writes are atomic — either the new save is complete or the previous save remains valid. Survives process kill, power loss, OS crash, alt-F4.
- NFR-R3: Game recovers gracefully from missing save file — first launch creates default; subsequent missing files prompt "restart with default save?" rather than silent data loss or crash.
- NFR-R4: Meta-currency and unlocked upgrades are never lost between runs during normal play.

**Accessibility**
- NFR-A1: Semantic accent colors remain visually distinguishable under common color-blindness conditions (protanopia, deuteranopia, tritanopia). Color is not the sole signal — shape, position, and audio cues provide redundant encoding.
- NFR-A2: No information critical to gameplay is conveyed by color alone.
- NFR-A3: HUD text is legible at a viewing distance of 60–80 cm from a 1080p display at the UI's default scale.

**Usability**
- NFR-U1: A new player reaches the first "aha" moment (holds fire on an intact asteroid for higher salvage yield) within their first 5-minute Arena session. Validated in playtest at M3 gate.
- NFR-U2: All HUD elements required for tactical decisions (shields, hull, salvage, radar, economic yield delta) are simultaneously visible in cockpit view.
- NFR-U3: Player can identify current state of all ship subsystems (Hull, Shields) at a glance without looking away from primary cockpit view.

**Localization**
- NFR-L1: MVP ships in English only.
- NFR-L2: German localization is a post-MVP deferred target.
- NFR-L3: All player-facing strings loaded from an external string table (RON per architecture decision), not hard-coded in source.

**Not Applicable (explicitly skipped):** Security (no PII, no payments), Scalability (offline single-player), Integration (no external APIs).

### Additional Requirements

**Starter Template & M0 Gate (Architecture Starter Template decision):**
- The Architecture specifies a **Hybrid Manual** starter (not greenfield blank, not community template). Epic 1 Story 1 MUST be plugin-compatibility verification, not gameplay code. Concretely:
  - `cargo new --bin asteroids3d`
  - Author `Cargo.toml` by hand with pinned versions: Bevy 0.18 (`default-features = false`, features `["3d", "png", "x11"]`), Avian 0.6 (via `avian3d`), `bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager`, `bevy_egui` (dev-only), plus `serde`, `serde_json`, `ron`, `thiserror`, `tracing`, `tracing-subscriber`, `directories`.
  - Verify all plugins have Bevy-0.18-compatible releases. If any lag, fork-and-maintain-inline path documented.
  - Commit `Cargo.lock`.
  - Borrow infrastructure (stripped) from NiklasEi `bevy_game_template`: `.github/workflows/ci.yml` (Windows+Linux+macOS matrix — strip iOS/Android/Web), `.gitignore`, `rustfmt.toml`, `clippy.toml`, `rust-toolchain.toml`.
- M0 completion criterion: `cargo run` opens a window showing "asteroids3D" splash on all three platforms, with CI passing.

**Version-Pinning Governance:**
- Bevy + Avian + all plugins pinned at M0. Upgrades batched at M4, M6, M9 milestone gates only, with 4–6 h migration budget. No ad-hoc mid-milestone upgrades.

**Cross-Platform CI Matrix (from M0):**
- GitHub Actions CI on `windows-latest`, `ubuntu-latest`, `macos-latest` (Apple Silicon runner). Jobs: build + test + clippy + fmt-check per platform. Release build verification per platform at milestone gates.
- Release workflow (M3+): per-OS ZIP artifacts (Windows-x64, Linux-x64, macOS-universal). Itch.io upload via `butler` CLI.

**macOS Code-Signing & Notarization (FR48 implementation):**
- M3: Manual workflow via `codesign` + `xcrun notarytool` with App-Specific Password (2–4 h budget). Apple Developer account (€99/year) required.
- M6: Automated via GitHub Actions secrets + notarization job.

**ECS / Architecture Discipline (load-bearing for learning goal):**
- Component-composition-first: small reusable components shared across Ship/Asteroid/Projectile/Enemy. God-structs forbidden. Anti-pattern: inheritance-shaped ECS.
- Plugin-per-module structure: 10 feature plugins (Flight, Combat, Salvage, Perception, Run, Ui, Persistence, Visual, Audio, Tuning) + `core/` shared types + `debug/` (dev-only). Each plugin exposes single `<Feature>Plugin` + `<Feature>Systems` SystemSet enum.
- Cross-plugin communication via Events (past-tense PascalCase) or shared Resources only — never direct state mutation.
- Naming conventions enforced: Components PascalCase single-responsibility, systems snake_case verb phrases, events past-tense.
- No `.after(specific_fn)` ordering — only SystemSet.

**Runtime State & Scheduling:**
- Bevy `States` API for top-level states: `MainMenu`, `Arena`, `Caravan`, `PostRun`, `PhotoMode`, `Paused`. `NextState<GameState>` mutation for transitions.
- Avian physics in `FixedUpdate` at 60 Hz; rendering in `Update` at display refresh; Transform interpolation between physics frames.
- State cleanup via marker components (e.g., `ArenaEntity`) and `cleanup_on_exit::<Marker>` systems.

**Save System (FR44–FR46 implementation):**
- Atomic write: `<savepath>.tmp` → fsync → `rename()` (Unix) / `MoveFileEx` (Windows).
- JSON + Serde format.
- Versioned schema (`version: u32` in save frontmatter); migration path from older versions.
- Save location via `directories` crate (Windows `%APPDATA%`, Linux `$XDG_DATA_HOME`, macOS `~/Library/Application Support/asteroids3D/`).

**String-Table & Tuning (NFR-L3 + hot-reload requirement):**
- `assets/strings/en.ron` canonical string table, loaded via `bevy_asset`, hot-reloadable during dev. Dot-separated scoped keys (e.g., `ui.hud.shields`).
- `assets/config/tuning.ron` runtime-tunable gameplay values (enemy HP, shot cost, tractor force, salvage yields), loaded into `TuningConfig` resource, hot-reloadable.

**Input Abstraction:**
- `leafwing-input-manager` with logical `Action` enums. Enables clean FR37 mouse-sensitivity surface, gamepad as MVP-stretch toggle, and post-MVP rebinding without refactor.

**Shader & Rendering (FR49, FR50):**
- Custom WGSL Toon `Material` authored by Till (primary M1 learning target). Validated on Metal (macOS) / Vulkan (Linux) / DX12 (Windows) at M1 tech-spike gate.
- `bevy_mod_outline` plugin for silhouette outlines (pinned, fork-ready).
- Fallback plan: flat-shaded low-poly + rim-light if M1 spike underwhelms.
- Hybrid HUD: `bevy_ui` screen-space for menus + hull/shield/ammo/salvage; world-space (cockpit meshes) for radar, waypoint pointer, yield-delta indicator (Design Philosophy "scientific-instrument panel" styling).

**Debug Tooling:**
- `bevy_egui` behind `cfg(debug_assertions)` — panels for FPS, physics state, entity inspector, trigger-zone visualizer, economy-balance tuner. Stripped from release builds.
- `FrameTimeDiagnosticsPlugin` + `LogDiagnosticsPlugin` from M0.
- `tracy-client` + `cargo flamegraph` from M2.
- F3 debug free-camera toggle (shares `FreeOrbitCamera` component with Photo Mode).

**Error Handling & Logging:**
- `Result<T, E>` + `thiserror`-derived custom enums at boundaries (save I/O, asset loading).
- `tracing` + `tracing-subscriber` with `RUST_LOG`. Panic hook writes stack trace to log file before exit.
- Log location via `directories` crate.
- Panic policy: panic OK only on programmer invariants; forbidden on user-facing paths (missing save, missing asset, user input edges).

**Asset Pipeline:**
- Blender → glTF 2.0 only (no FBX). Textures low-res to match vector aesthetic.
- Asset loading gated at `OnEnter(State)` — scattered `AssetServer::load` forbidden (NFR-P4 enforcement).
- Typed `Resource` wrappers per asset group (`AsteroidModels`, `WeaponSounds`).

**Distribution (MVP / Itch.io):**
- Single ZIP per platform: binary + `assets/` folder. macOS ships as signed + notarized `.app` inside ZIP. Binary size target < 100 MB including assets.

**Deferred (explicitly out of MVP scope):**
- Modular weapon crafting UI (Post-MVP-1)
- Engine / Weapons / Sensors subsystems beyond Hull+Shields (Post-MVP)
- Partial Death, Heat modifiers, Bullet-time, Rewind, Precognition (Post-MVP)
- Blind Flight hardcore mode, Tower Defense mode (Post-MVP)
- Cockpit-pet unlock, Kill-cam, Mega-wreck, Boss asteroids (Post-MVP polish)
- HRTF / deeper spatial audio (Post-MVP R#6 stage 2)
- Multiple run-skeleton templates (Post-MVP; MVP has 1 template × 3 difficulties)
- Short-burst / Expedition session length presets (Post-MVP)
- Full key/button rebinding UI (Post-MVP)
- Closed captions for audio cues (Post-MVP consideration)
- Steam Cloud Save / Steamworks SDK integration (M6 gate if Steam release)
- Rhai modding scripting, seed-based level sharing (Vision)
- VR port of cockpit mode (Vision)

### UX Design Requirements

N/A — No UX Design document exists for this project. UX guidance is embedded in:
- PRD *Design Philosophy* (5 design principles: no tutorial text; no visible numeric score; predictable asteroid motion; death-as-feedback; graceful degradation)
- PRD *User Journeys* (4 journey arcs with implicit acceptance criteria)
- Architecture *Rendering & Visual Architecture* (hybrid bevy_ui + world-space cockpit HUD, scientific-instrument-panel styling)
- Architecture *UI, Menu & Debug Architecture* (menu system, HUD rendering strategy)

Confirmed with Till on 2026-04-22: no separate UX-DR extraction required.

### FR Coverage Map

| FR | Epic | Description |
|---|---|---|
| FR1 | E3 | Pilot ship via keyboard + mouse input |
| FR2 | E3 | 6-direction ship translation |
| FR3 | E3 | 3-axis ship rotation (pitch/yaw/roll) |
| FR4 | E5 | Decoupled aim (weapon aim independent of ship heading) |
| FR5 | E3 | Inertial dampener toggle |
| FR6 | E6 | Boost (rechargeable) |
| FR7 | E6 | Tractor beam intact asteroid capture |
| FR8 | E3 | Cockpit-only first-person view (no external camera) |
| FR9 | E3 | Weapons fire ballistic projectiles |
| FR10 | E4 | Up to 3 weapons from pool of 3 archetypes |
| FR11 | E6 | Pay-to-shoot (salvage currency debit per shot) |
| FR12 | E3 | Projectile damage to asteroids / enemies / debris |
| FR13 | E6 | Destroyed asteroid yield < intact-capture yield |
| FR14 | E4 | Enemy AI detect / pursue / attack |
| FR15 | E5 | Hull + Shields subsystems (regen model) |
| FR16 | E4 | Permadeath on Hull zero (basic Hull in E4; formal in E5) |
| FR17 | E6 | In-run salvage currency accumulation |
| FR18 | E7 | Salvage → meta-currency conversion on run end |
| FR19 | E7 | Meta-currency persists across runs |
| FR20 | E7 | Spend meta-currency in unlock shop |
| FR21 | E7 | 5–10 permanent upgrade unlocks |
| FR22 | E8 | Cockpit HUD radar with threat markers |
| FR23 | E8 | Spatial stereo audio cues (enemies / hazards / salvage) |
| FR24 | E3 | HUD displays shields / hull / ammo / salvage |
| FR25 | E8 | HUD yield-delta indicator (intact vs destroyed) |
| FR26 | E8 | First-launch headphone-recommendation splash |
| FR27 | E3 | Arena tutorial zone (first-session placement) |
| FR28 | E3 | No written tutorial text (diegetic cues only) |
| FR29 | E6 | Arena → Caravan transition |
| FR30 | E6 | Caravan run 5–8 min duration |
| FR31 | E6 | 3 difficulty variants (easy / medium / hard) |
| FR32 | E6 | Combat pockets triggered by render-distance thresholds |
| FR33 | E6 | Waypoint-pointer navigation indicator |
| FR34 | E6 | Salvage bank on successful Caravan completion |
| FR35 | E6 | Salvage bank on Caravan death (meta-unlocks persist) |
| FR36 | E4 | Title screen with start / settings / credits / quit |
| FR37 | E4 | Settings menu: volume (master + SFX), mouse sensitivity |
| FR38 | E4 | Post-run summary (cause of death, banked, retry options) |
| FR39 | E4 | Restart without returning to title screen |
| FR40 | E9 | Photo Mode accessible only from post-run / death screen |
| FR41 | E9 | Photo Mode free-cam orbital/dolly + DoF + time-frozen |
| FR42 | E9 | PNG export in 16:9 / 9:16 / 1:1 aspect ratios |
| FR43 | E3 | Pause on window focus loss / pause menu |
| FR44 | E4 | Persistence to OS-convention save location |
| FR45 | E4 | First-launch default save creation |
| FR46 | E4 | Atomic save write (crash-safe) |
| FR47 | E1/E4 | Cross-platform baseline (E1) → full 3-OS shipping (E4) |
| FR48 | E10 | macOS code-signing + notarization (deferred E4→E7→E10 per Till 2026-04-22; MVP ships unsigned macOS through M6, signed at M9 polish gate) |
| FR49 | E2 | Toon-shading material + silhouette outlines |
| FR50 | E2/E4 | Semantic accent palette (foundation E2 → enemies/salvage E4) |

**Coverage: 50 / 50 FRs mapped. 0 orphans.**

## Epic List

### Epic 1: Foundation & Plugin Compatibility Gate

**User outcome (dev-foundational):** Project compiles and runs on Windows, Linux, and macOS. `cargo run` opens a window showing "asteroids3D" splash on all three platforms. Plugin compatibility matrix (Bevy 0.18, Avian 0.6, bevy_mod_outline, bevy_kira_audio, leafwing-input-manager, bevy_egui) verified and version-pinned. CI matrix green. No gameplay — this is the compatibility gate per Architecture Starter decision.

**FRs covered:** FR47 (cross-platform binary baseline)

**Scope:** Hybrid-Manual starter initialization, `Cargo.toml` authored by hand with pinned versions, Bevy-0.18 compatibility verification (fork-and-inline any plugin that lags), CI workflow (Windows/Linux/macOS — strip iOS/Android/Web from NiklasEi template), `rustfmt.toml` + `clippy.toml` + `rust-toolchain.toml`, `App::new()` skeleton with `GameState` enum + `leafwing-input-manager` + bevy_ui splash screen Node.

**M-alignment:** M0

**Completion gate:** `cargo run` opens window with "asteroids3D" splash on Win/Linux/macOS; CI green.

---

### Epic 2: Vector Aesthetic Tech Spike

**User outcome:** Custom WGSL Toon Material + bevy_mod_outline render identically on Metal (macOS), Vulkan (Linux), and DX12 (Windows). M1 go/fallback decision documented. Portfolio-quality shader artifact authored by Till.

**FRs covered:** FR49 (toon shading + outlines), FR50 (semantic accent palette foundation)

**Scope:** Custom WGSL Toon `Material` impl at `src/visual/toon_material.rs`, `bevy_mod_outline` integration wiring, palette primitives (`SemanticAccent` enum + color lookup), three-backend validation gate (render reference scene on all three GPUs and compare visually), go/fallback decision doc. Fallback path (flat-shaded + rim-light) scaffolded only if decision = fallback.

**M-alignment:** M1

**Completion gate:** Reference scene renders with toon + outline on Metal/Vulkan/DX12; decision "go toon" or "fall back" committed.

---

### Epic 3: Arena Flight & First Combat (First Playable)

**User outcome:** Player flies a cockpit ship in the Arena, fires a prefab weapon, destroys asteroids, sees HUD with ship state. No enemies yet. Diegetic learning — no tutorial text.

**FRs covered:** FR1, FR2, FR3, FR5, FR8, FR9, FR12, FR24, FR27, FR28, FR43

**Scope:** FlightPlugin (input → thrust/rotation via leafwing + Avian XPBD in FixedUpdate 60Hz + dampener toggle), cockpit `Camera3d` with wingtip framing, CombatPlugin weapon-firing + projectile ballistics + damage-on-asteroid, HUD screen-space baseline (shields/hull/ammo/salvage placeholders), Arena zone (hand-designed), pause on focus-loss, basic title screen stub.

**M-alignment:** M2

**Completion gate:** Player flies Arena, shoots asteroids, HUD visible, pause works on Alt-Tab.

---

### Epic 4: Enemies Alive & Stop-Ship (Itch.io Prototype)

**User outcome:** The Itch.io-shippable small game. Full combat loop: 3 weapons, 1 enemy type with AI, permadeath on Hull-zero, post-run summary, immediate restart, title screen, settings (volume + sensitivity), saved settings + currency "high-score", signed+notarized macOS binary. The M3 stop-and-ship waypoint.

**FRs covered:** FR10, FR14, FR16, FR36, FR37, FR38, FR39, FR44, FR45, FR46, FR47, FR50 (FR48 deferred to E7 per Till 2026-04-22; macOS ships unsigned for M3)

**Scope:** 2 more weapon archetypes (total 3), enemy AI state machine (detect → pursue → attack), basic single-HP Hull + permadeath flow, title screen (start / settings / credits / quit), settings UI (volume master+SFX, mouse sensitivity), post-run summary screen (cause of death, salvage banked, retry/menu — no "GAME OVER"), restart flow, PersistencePlugin save service (atomic temp+rename, JSON+Serde, versioned schema, `directories` crate per-OS paths), first-launch default save creation, macOS codesign + notarytool workflow (Apple Developer account), release.yml for per-OS ZIPs + butler-to-Itch.io, semantic accent colors wired to enemies + salvage.

**M-alignment:** M3 🏁 (stop-and-ship)

**Completion gate:** Itch.io-ready ZIPs on all 3 platforms (macOS unsigned, right-click-open per M3 decision), zero-crash 3-run Arena playtest passes.

---

### Epic 5: Ship Subsystem State & Formal Save Schema

**User outcome:** Formal Hull + Shields subsystems with regen mechanics. Shields regenerate after cooldown; Hull does not regen mid-run. Ship state readable at-a-glance. Save schema formalized with versioning for post-MVP expansion. Decoupled aim reticle system.

**FRs covered:** FR4, FR15

**NFRs covered:** NFR-R3 (graceful missing-save), NFR-R4 (no between-run meta loss), NFR-U2, NFR-U3 (HUD subsystem at-a-glance)

**Scope:** Formal `HullHP` + `ShieldHP` components (regen_rate, cooldown) per architecture good-pattern example, Shield regen system, Hull-zero → `HullDepleted` event wiring, decoupled-aim reticle overlay (world-space target coords → bevy_ui edge indicator), at-a-glance instrument-panel HUD styling, save schema `version: u32` + migration scaffold, missing-save recovery prompt.

**M-alignment:** M4

**Completion gate:** Shield regen tunable in `tuning.ron`, Hull damage visible in cockpit HUD at a glance, save schema migrates old → new without data loss.

---

### Epic 6: Caravan Run Framework

**User outcome:** Player flies a Caravan run from start to target destination (5–8 min), with waypoint-pointer navigation, selectable difficulty (easy/medium/hard), trigger-volume combat pockets, tractor-beam intact-asteroid pickup, boost, pay-to-shoot economy with intact > destroyed yield math, salvage banking on success and death.

**FRs covered:** FR6, FR7, FR11, FR13, FR17, FR29, FR30, FR31, FR32, FR33, FR34, FR35

**Scope:** RunPlugin run-director (RunStarted/RunEnded lifecycle), single Caravan skeleton template with 3 difficulty parameter variants, waypoint-pointer world-space HUD, render-distance pocket-trigger system (Avian sensors), `BoostActivated` event + recharge resource, SalvagePlugin tractor-beam constraint/impulse on intact asteroids, economy math (shot-cost debit on WeaponFired, yield calc on AsteroidDestroyed vs AsteroidCaptured), Arena → Caravan state transition, salvage banking on both outcomes.

**M-alignment:** M5 ⚠️ **Danger Stretch** — stories in this epic MUST be sliced into weekly sub-milestones where each delivers one visible feature (waypoint pointer → first pocket trigger → difficulty curve → tractor beam → economy math → difficulty variants). No "I'll build Caravan for 6 weeks" commitments.

**Completion gate:** Player completes a 5-min easy Caravan run from start to destination, tractor-captures at least one intact asteroid, pocket combat triggers at least once.

---

### Epic 7: Roguelite Loop (EA-Viable)

**User outcome:** Meta-currency earned from runs, spendable in unlock shop for 5–10 permanent upgrades. "One more run" retention loop closed. Commercially viable as Itch.io release or Steam Early Access. Also: Intel x86_64 macOS binary added alongside arm64 (universal, still unsigned — FR48 further deferred to E10 per Till 2026-04-22).

**FRs covered:** FR18, FR19, FR20, FR21 (FR48 deferred E4→E7→E10; MVP ships unsigned macOS through M6)

**NFRs covered:** NFR-R4 (meta never lost between runs during normal play)

**Scope:** `PersistentMeta` resource extending save schema (meta-currency balance, unlocked upgrade IDs), run→meta conversion rate (configurable in `tuning.ron`), unlock shop UI accessible from main menu and post-run screen, 5–10 initial unlock definitions (e.g., ammo capacity +20%, sensor range +15%, boost recharge faster, etc.), `UnlockPurchased` event + save trigger, unlock effects wired to ship tunables.

**M-alignment:** M6 🏁 (EA-viable)

**Completion gate:** 10-run playtest with save persisting meta-currency across crashes, 3+ unlocks purchased and effects visible in gameplay.

---

### Epic 8: Perception — Sensors & Spatial Audio

**User outcome:** Player perceives unseen threats via cockpit radar and spatial stereo audio cues. Yield-delta indicator shows intact-vs-destroyed math on visible salvageable targets. First-launch headphone recommendation.

**FRs covered:** FR22, FR23, FR25, FR26

**NFRs covered:** NFR-A1 (colorblind redundant encoding), NFR-A2 (no color-only information)

**Scope:** PerceptionPlugin sensor range + threat detection (`EnemyDetected` / `HazardDetected` events), world-space radar mesh on cockpit model (scientific-instrument styling per Design Philosophy), AudioPlugin spatial channel setup (`bevy_kira_audio`), audio-cue routing from threat events, world-space yield-delta indicator on visible Salvageable entities, first-launch splash with headphone recommendation (`bevy_ui` modal shown before main menu), **damage-direction-indicator** on cockpit HUD (red arrow at screen edge pointing toward the origin of incoming fire when shields/hull take a hit — added during E5 decomposition 2026-04-22).

**M-alignment:** M7

**Completion gate:** Player identifies unseen enemy from audio direction before it enters visual range, radar shows correct markers, yield-delta visible on at least 3 salvage types.

---

### Epic 9: Post-Run Photo Mode

**User outcome:** From post-run / death screen, player enters Photo Mode with free-cam orbital/dolly movement, adjustable depth-of-field, time-frozen scene. Exports PNG screenshots in 16:9 landscape, 9:16 portrait, or 1:1 square aspect ratios. Marketing-ready aesthetic artifact pipeline.

**FRs covered:** FR40, FR41, FR42

**Scope:** `FreeOrbitCamera` component (shared with debug F3 camera — one impl, two gates), PhotoMode state entry from post-run screen only (no in-run access), DoF post-processing node, time-freeze on state entry (already paused), PNG export system (aspect-ratio preset enum, resolution scaling, file save to user-pictures dir or game subdirectory), optional toggleable watermark.

**M-alignment:** M8

**Completion gate:** Player exports PNGs in all 3 aspect ratios; toon + outline shader renders correctly at 360° camera angles (external-viewing shader validation).

---

### Epic 10: Polish Pass & MVP Completion

**User outcome:** Balance tuning, audio-pass-2, UI polish, 2–5 additional unlocks beyond M6's baseline, crash fixes, all MVP NFRs satisfied. Full polished MVP ready for Steam release.

**FRs covered:** FR48 (deferred E4→E7→E10 per Till 2026-04-22). Otherwise this epic hardens previously-delivered capabilities.

**NFRs covered:** NFR-P1 (60 FPS sustained), NFR-P2 (load ≤ 10 s), NFR-P3 (title→gameplay ≤ 5 s), NFR-P4 (no hitches > 100 ms), NFR-P5 (< 4 GB memory), NFR-R1 (zero-crash across all 4 user journeys), NFR-U1 (5-min Aha validated), NFR-A3 (HUD legibility 60–80 cm), NFR-L3 (no hard-coded strings audit), NFR-L1 (English ship-ready)

**Scope:** Profiling pass with tracy + flamegraph to hit 60 FPS on GTX 1060 / RX 580 / M1, asset-load-at-state-entry audit (no scattered AssetServer::load), audio SFX pass-2 (mixing, ambient drone layer polish per Design Philosophy "cosmic mystery"), UI polish (scientific-instrument styling refinement), 2–5 additional unlock definitions, crash-fix backlog from M6–M8 playtesting, string-table audit (no hard-coded player-facing strings), playtest validation of all 4 user journeys on all 3 platforms, **shield-absorb VFX** (brief toon-style flash on shield hit, tuned for readability without being intrusive — added during E5 decomposition 2026-04-22), **macOS code-signing + notarization** (FR48, deferred E4→E7→E10; requires Apple Developer Account €99/year at this milestone — enables signed MVP release for Steam if pursued, or Itch.io polish-release).

**M-alignment:** M9 🏁 (MVP polished)

**Completion gate:** All MVP success criteria from PRD true; zero-crash 10-run playtest on all 3 platforms at 60 FPS; Steam-release or Itch.io-polished-release ready.

---

<!-- RESUMPTION MARKER: Step 3 complete — all 10 Epics decomposed into stories. Next: Step 4 final validation. -->

## Epic 1: Foundation & Plugin Compatibility Gate

Project compiles and runs on Windows, Linux, and macOS. `cargo run` opens a window showing "asteroids3D" splash. Plugin compatibility matrix verified and version-pinned. CI matrix green. No gameplay code — this is the compatibility gate per Architecture Starter decision. M-alignment: M0.

### Story 1.1: Bootstrap Cargo Project with Hand-Authored Cargo.toml

As a developer,
I want the project directory initialized with a hand-authored `Cargo.toml` containing all pinned dependencies,
So that every dependency is committed and reproducible from day one, and I internalize the Bevy setup rather than inheriting it from a template.

**Acceptance Criteria:**

**Given** an empty working directory at `~/Projekte/rust/asteroids3D`
**When** `cargo new --bin asteroids3d` is executed
**Then** `src/main.rs` and `Cargo.toml` are created by cargo

**Given** the default `Cargo.toml` is replaced by hand
**When** the edit is saved
**Then** Bevy is pinned at `0.18` with `default-features = false` and features `["3d", "png"]` plus platform-appropriate windowing (`x11`/`wayland` on Linux)
**And** avian3d is pinned at `0.6`
**And** `bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager` are pinned at their latest Bevy-0.18-compatible versions
**And** `bevy_egui` is declared under `[target.'cfg(debug_assertions)'.dependencies]`
**And** `serde`, `serde_json`, `ron`, `thiserror`, `tracing`, `tracing-subscriber`, `directories` are pinned
**And** release profile sets `lto = "fat"`, `codegen-units = 1`, `opt-level = 3`
**And** dev profile sets dependency `opt-level = 1`

**Given** all dependencies are pinned
**When** `cargo check` runs
**Then** resolution succeeds
**And** `Cargo.lock` is committed

### Story 1.2: Plugin Compatibility Verification Gate

As a developer,
I want explicit verification that every pinned plugin has a working Bevy-0.18-compatible release,
So that I discover fork-or-substitute decisions before writing gameplay code, not three weeks into M2.

**Acceptance Criteria:**

**Given** `Cargo.toml` from Story 1.1
**When** `cargo check` is executed on the local machine
**Then** all four third-party plugins (`bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager`, `bevy_egui`) compile without errors

**Given** any plugin fails to compile
**When** the failure is reviewed
**Then** a resolution is documented in `docs/plugin-compatibility.md` with (plugin name, error summary, resolution path)
**And** the resolution path is one of: (a) upstream patch exists → pin updated, (b) fork-and-inline per PRD Tech-Risk strategy, (c) substitute alternative plugin

**Given** all plugins resolve
**When** the verification is complete
**Then** `docs/plugin-compatibility.md` lists verification date, Rust toolchain version, Bevy version, and each plugin version
**And** this story's gate is passed — subsequent stories may proceed

### Story 1.3: Toolchain, Lint, and Format Configuration

As a developer,
I want reproducible toolchain + lint + format configs committed,
So that local dev and CI share the same rules and formatting drift is impossible.

**Acceptance Criteria:**

**Given** the project root has no toolchain config
**When** `rust-toolchain.toml` is added pinning the latest stable Rust channel
**Then** `rustup show` inside the project reports the pinned channel
**And** CI (when added in Story 1.4) uses the same channel deterministically

**Given** no format/lint configs exist
**When** `rustfmt.toml` and `clippy.toml` are added with project style rules
**Then** `cargo fmt --check` passes on all committed code
**And** `cargo clippy -- -D warnings` passes on all committed code

**Given** no ignore rules exist
**When** `.gitignore` is added per Rust + Bevy conventions
**Then** `target/` is excluded
**And** Bevy asset-cache directories are excluded
**And** IDE-local files (`.vscode/`, `.idea/`) and OS artifacts (`.DS_Store`, `Thumbs.db`) are excluded

### Story 1.4: Three-Platform CI Matrix

As a developer,
I want GitHub Actions CI running on Windows, Linux, and macOS from commit one,
So that the cross-platform parity commitment (FR47) is verified on every push instead of discovered at a milestone gate.

**Acceptance Criteria:**

**Given** `.github/workflows/ci.yml` is added, adapted from NiklasEi's `bevy_game_template` CI
**When** a commit is pushed to any branch
**Then** parallel jobs run on `windows-latest`, `ubuntu-latest`, and `macos-latest` (Apple Silicon runner)
**And** each job executes `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`
**And** iOS, Android, and Web/WASM jobs from the source template are stripped out

**Given** any of the three OS jobs fails
**When** the CI result is reported
**Then** the pull request / commit status reports red
**And** the failing log identifies which OS and which step failed

**Given** all three OS jobs pass
**When** CI completes
**Then** FR47 baseline (cross-platform binary) is verified for the current commit

### Story 1.5: Minimal Bevy App Opens a Window on All Three Platforms

As a first-time observer of the project,
I want `cargo run` to open a window on Windows, Linux, and macOS,
So that the "asteroids3D project exists and runs" signal is demonstrable from day one — the motivation-preservation baseline.

**Acceptance Criteria:**

**Given** `src/main.rs` contains `App::new().add_plugins(DefaultPlugins).run()`
**When** `cargo run` is invoked on Windows 10+
**Then** a native window opens with default Bevy title and size
**And** no panics or unexpected error logs are emitted

**Given** the same `src/main.rs`
**When** `cargo run` is invoked on a Linux desktop (Ubuntu LTS or equivalent)
**Then** a native window opens using the Vulkan backend via wgpu
**And** no panics or unexpected error logs are emitted

**Given** the same `src/main.rs`
**When** `cargo run` is invoked on macOS (Apple Silicon)
**Then** a native window opens using the Metal backend via wgpu
**And** no panics or unexpected error logs are emitted

### Story 1.6: GameState Enum with Bevy States Skeleton

As a developer,
I want a `GameState` enum registered with Bevy's `States` API,
So that future plugins can hook `OnEnter`/`OnExit`/`in_state()` scheduling from M1 onward without retrofit.

**Acceptance Criteria:**

**Given** `src/state.rs` is created
**When** `GameState` is defined with variants `Loading`, `MainMenu`, `Arena`, `Caravan`, `PostRun`, `PhotoMode`, `Paused`
**Then** it derives `States`, `Default` (default = `Loading`), `Debug`, `Clone`, `Eq`, `PartialEq`, `Hash`

**Given** `App::init_state::<GameState>()` is called in `main.rs`
**When** the app starts
**Then** `State<GameState>::get()` returns `GameState::Loading` on first frame

**Given** a debug system registered on `OnEnter(GameState::Loading)` emits an `info!` log
**When** the app starts
**Then** the log contains the expected "entered Loading" line
**And** no further state transitions happen automatically in this story (the transition to `MainMenu` is Story 1.7)

### Story 1.7: Splash Screen Shows "asteroids3D" and Transitions to MainMenu

As a player launching the game,
I want to see "asteroids3D" displayed when the app opens,
So that I immediately know the app launched and I'm in the right program.

**Acceptance Criteria:**

**Given** the app is in `GameState::Loading`
**When** `OnEnter(GameState::Loading)` runs
**Then** a `bevy_ui` text Node is spawned with content `"asteroids3D"`
**And** the Node uses centered flexbox layout that scales to window size
**And** the text entity carries a `LoadingStateEntity` marker component

**Given** the splash is visible
**When** a configurable splash-duration elapses (duration loaded from a `SplashConfig` resource, default 2.0 seconds)
**Then** the app mutates `NextState<GameState>` to `MainMenu`

**Given** the state transitions from `Loading` to `MainMenu`
**When** `OnExit(GameState::Loading)` runs
**Then** all entities tagged with `LoadingStateEntity` are despawned
**And** no orphaned splash text remains in the hierarchy

**Given** the app is now in `GameState::MainMenu`
**When** the window is inspected visually
**Then** the splash text is gone (MainMenu UI is a later epic's responsibility — this story ends at the transition)

### Story 1.8: Tracing-Based Logging with Panic Hook to Log File

As a developer,
I want `tracing`-based logging with a panic hook that writes stack traces to a log file in the user-log-dir,
So that crashes during CI runs or future playtesting can be forensically reviewed after process exit.

**Acceptance Criteria:**

**Given** `tracing_subscriber` is initialized in `main.rs` before `App::new()`
**When** the app runs
**Then** `info!` / `warn!` / `error!` events from Bevy and app code are output to stderr
**And** `RUST_LOG=debug cargo run` increases verbosity to `debug!` level

**Given** the `directories` crate resolves the per-OS user-log-dir (Windows `%APPDATA%\asteroids3D\logs\`, Linux `$XDG_STATE_HOME/asteroids3d/logs/` or fallback, macOS `~/Library/Logs/asteroids3D/`)
**When** a log file is opened at startup
**Then** logs are written to both stderr and the file simultaneously

**Given** a panic hook is installed via `std::panic::set_hook`
**When** a panic is triggered (e.g., via a `#[cfg(test)]` panic test or manual `panic!()` in a dev-only build)
**Then** the panic message and backtrace are written to the log file before process exit
**And** the default panic behavior (printing to stderr) is preserved

<!-- Epic 1 complete — 8 stories cover M0 completion criterion. Next epic to decompose: Epic 2 (Vector Aesthetic Tech Spike / M1). -->

## Epic 2: Vector Aesthetic Tech Spike

Custom WGSL Toon `Material` + `bevy_mod_outline` render identically on Metal (macOS), Vulkan (Linux), and DX12 (Windows). M1 go/fallback decision documented. Portfolio-quality shader artifact authored by Till. M-alignment: M1. FRs covered: FR49, FR50.

### Story 2.1: VisualPlugin Skeleton + Reference Scene

As a developer,
I want a `VisualPlugin` module and a committed dev-only reference scene (asteroid + ship-cockpit + projectile placeholders with 3-point lighting),
So that Stories 2.2–2.5 have a stable, reproducible stage to validate shaders and outlines against across all three GPU backends.

**Acceptance Criteria:**

**Given** the project from Epic 1
**When** `src/visual/mod.rs` is authored with a `VisualPlugin: Plugin` struct and added via `App::add_plugins(VisualPlugin)` in `main.rs`
**Then** the plugin declares a `VisualSystems` SystemSet enum per the architecture naming convention
**And** the plugin builds on all three platforms

**Given** a reference-scene module behind `cfg(debug_assertions)`
**When** it runs on `OnEnter(GameState::Loading)`
**Then** the scene contains exactly three placeholder meshes: an icosphere asteroid, a cuboid ship-cockpit placeholder, a small sphere projectile
**And** three `PointLight` entities form a 3-point lighting setup (key, fill, back)
**And** every spawned entity carries a `ReferenceSceneEntity` marker component

**Given** the reference scene is in the scene graph
**When** `cargo run` is invoked in a debug build
**Then** all three placeholders render using Bevy's default `StandardMaterial` (toon comes in Story 2.3)
**And** all three placeholders are inside the camera frustum

**Given** a release build (`cargo build --release`)
**When** the binary is inspected for the symbol `ReferenceSceneEntity`
**Then** the symbol is absent (reference scene compiled out of release)

### Story 2.2: SemanticAccent Palette Primitives

As a developer,
I want a `SemanticAccent` enum with a color-lookup function and a committed visual distinguishability reference under three color-blindness simulations,
So that FR50 semantic accent colors rest on a tested NFR-A1 foundation before any shader consumes them.

**Acceptance Criteria:**

**Given** `src/visual/palette.rs` is authored
**When** it defines `SemanticAccent` as an enum with variants `Enemy`, `Salvage`, `Hazard`, `PlayerOwned`, `Neutral`
**Then** each variant has a specified `Color` with its hex value documented as a comment
**And** `pub fn color_for(accent: SemanticAccent) -> Color` returns the mapped color

**Given** a dev-only visualization scene (extension of Story 2.1's reference scene or a standalone example)
**When** the 5 accent colors are rendered as labeled swatches side-by-side
**Then** screenshots are captured under: (a) normal vision, (b) protanopia simulation, (c) deuteranopia simulation, (d) tritanopia simulation
**And** all 4 screenshots are committed to `docs/tech-spike/m1-palette/`

**Given** the simulated-vision screenshots
**When** visually inspected
**Then** every accent color remains distinguishable from every other accent color under all three simulations
**And** failing pairs (if any) are documented in `docs/tech-spike/m1-palette/review-notes.md` with a proposed color adjustment

**Given** the `SemanticAccent` enum
**When** later stories need per-entity accent tagging
**Then** they may attach `SemanticAccent` as a component so shaders and outlines can read it without entity-level hardcoding

### Story 2.3: WGSL Toon Material Implementation

As the primary shader author,
I want a hand-written WGSL `ToonMaterial` implementing N·L posterization with configurable step count, rim-light term, and `SemanticAccent` tinting,
So that FR49 toon-shading ships as a portfolio-quality self-authored artifact — the primary M1 learning target.

**Acceptance Criteria:**

**Given** `assets/shaders/toon.wgsl` is authored by hand
**When** its fragment shader is reviewed
**Then** shading is computed as `floor(max(dot(N,L), 0.0) * steps) / steps` posterization
**And** a rim-light term `pow(1.0 - dot(N,V), rim_power) * rim_intensity` is additive to the posterized base
**And** a `tint: vec4<f32>` uniform multiplies the final color
**And** uniforms `steps: u32`, `rim_power: f32`, `rim_intensity: f32`, `tint: vec4<f32>` are declared in a single uniform buffer

**Given** `src/visual/toon_material.rs` is authored
**When** it defines `ToonMaterial` implementing Bevy's `Material` trait
**Then** `fragment_shader()` returns a handle to `assets/shaders/toon.wgsl`
**And** `AsBindGroup` is derived and matches the WGSL uniform layout
**And** `MaterialPlugin::<ToonMaterial>::default()` is registered inside `VisualPlugin`

**Given** `src/tuning.rs` defines a `TuningConfig` resource loaded from `assets/config/tuning.ron` with fields `toon_steps: u32`, `toon_rim_power: f32`, `toon_rim_intensity: f32`
**When** `tuning.ron` is edited during `cargo run` (dev hot-reload enabled via `AssetPlugin::watch_for_changes_override`)
**Then** `ToonMaterial` uniforms update live in the reference scene without restart

**Given** the reference scene's three placeholders
**When** they are re-materialized with `ToonMaterial` instead of `StandardMaterial`
**Then** each placeholder shows visible posterized banding
**And** the rim-light term is visible at grazing angles on the asteroid silhouette
**And** entities carrying a `SemanticAccent` component render with the corresponding `tint`

**Given** `toon_steps` is set to 3, then 5, then 8 via hot-reload
**When** each value is observed on the asteroid
**Then** the number of visible shading bands matches the uniform value within ±1 band (anti-aliasing tolerance)

### Story 2.4: bevy_mod_outline Integration + Wiring

As a developer,
I want `bevy_mod_outline` integrated with `OutlineBundle` attached to every toon-shaded mesh in the reference scene, width and color tunable via `TuningConfig`,
So that FR49 silhouette outlines render consistently without per-entity hardcoding.

**Acceptance Criteria:**

**Given** `bevy_mod_outline`'s plugin is added to `VisualPlugin`
**When** the app starts
**Then** the plugin's systems are scheduled per its documented requirements
**And** the app still launches on all three platforms

**Given** `TuningConfig` is extended with `outline_width: f32` and `outline_color: Color` fields loaded from `assets/config/tuning.ron`
**When** the reference scene spawns its three placeholders
**Then** each placeholder is spawned with an `OutlineBundle` whose `width` and `color` read from `TuningConfig`

**Given** outlines are applied
**When** the reference scene renders in a debug build
**Then** the asteroid, ship, and projectile each show a continuous silhouette outline visible against any background
**And** outlines do not z-fight with mesh surfaces at the default camera distance

**Given** `assets/config/tuning.ron` is edited at runtime
**When** `outline_width` changes from 2.0 to 4.0
**Then** the running reference scene updates to thicker outlines without restart

### Story 2.5: Three-Backend Parity Validation Gate

As the project author,
I want the reference scene (toon + outlines) rendered on Metal, Vulkan, and DX12 with committed 1080p screenshots and a pairwise-diff report,
So that M1's completion criterion has objective, reviewable evidence and any backend divergence is documented before M2.

**Acceptance Criteria:**

**Given** the reference scene is complete from Stories 2.3 and 2.4
**When** `cargo run --release` is executed on macOS (Apple Silicon, Metal)
**Then** a screenshot is captured at a fixed deterministic camera transform (hardcoded `Transform` in a capture-mode code path)
**And** scene time is frozen at `t=0` (no animation) to ensure reproducibility
**And** the 1920×1080 PNG is committed to `docs/tech-spike/m1-backends/metal.png`

**Given** the same reference scene
**When** `cargo run --release` is executed on Linux with `WGPU_BACKEND=vulkan`
**Then** a 1920×1080 PNG at the same camera transform is committed to `docs/tech-spike/m1-backends/vulkan.png`

**Given** the same reference scene
**When** `cargo run --release` is executed on Windows with `WGPU_BACKEND=dx12`
**Then** a 1920×1080 PNG at the same camera transform is committed to `docs/tech-spike/m1-backends/dx12.png`

**Given** all three screenshots exist
**When** they are compared (ImageMagick `compare`, Beyond Compare, or manual overlay)
**Then** `docs/tech-spike/m1-backends/parity-report.md` documents each pairwise diff summary
**And** any >5% pixel divergence is annotated with a root-cause hypothesis
**And** the report closes with a go / no-go recommendation for Story 2.6

### Story 2.6: Go/Fallback Decision Document

As the project author,
I want a committed decision whether to proceed with the custom toon shader or fall back to flat + rim-light,
So that M1 closes with explicit scope resolution for M2 and the rationale is auditable later.

**Acceptance Criteria:**

**Given** the parity report from Story 2.5
**When** `docs/tech-spike/m1-decision.md` is authored
**Then** it contains sections: `Decision`, `Rationale`, `Risks Accepted`, `Fallback Trigger Criteria`, `M2 Impact`
**And** `Decision` is exactly one of: `GO toon`, `GO toon with scope reduction`, `FALLBACK flat+rim-light`

**Given** the decision is `GO toon` or `GO toon with scope reduction`
**When** M1 is declared complete
**Then** Story 2.7 is marked `Not Needed`
**And** `ToonMaterial` is confirmed as the M2 production shader

**Given** the decision is `FALLBACK flat+rim-light`
**When** M1 is declared complete
**Then** Story 2.7 is unblocked
**And** `ToonMaterial` is scheduled for removal or deprecation in Story 2.7

### Story 2.7: Fallback Material Scaffold (Conditional on Story 2.6)

As a developer,
I want a flat-shaded + rim-light fallback material scaffolded only if Story 2.6's decision is `FALLBACK flat+rim-light`,
So that M1 closes with a viable aesthetic path even when custom WGSL proves untenable across backends.

**Acceptance Criteria:**

**Given** Story 2.6's decision is `GO toon` or `GO toon with scope reduction`
**When** this story is reviewed at M1 closeout
**Then** this story is marked `Not Needed` and skipped
**And** no code changes are made

**Given** Story 2.6's decision is `FALLBACK flat+rim-light`
**When** this story is executed
**Then** `src/visual/flat_rim_material.rs` is authored with a Bevy `Material` impl using a `StandardMaterial`-compatible flat base plus a minimal rim-light fragment term
**And** `ToonMaterial` is either deleted or retained with a `#[deprecated]` attribute pointing at the fallback material
**And** the reference scene re-materializes its placeholders with the fallback material

**Given** the fallback material is applied
**When** the reference scene is re-rendered on all three backends
**Then** parity screenshots are captured to `docs/tech-spike/m1-backends-fallback/{metal,vulkan,dx12}.png`
**And** `docs/tech-spike/m1-backends/parity-report.md` is appended with a fallback-parity section

<!-- Epic 2 complete — 7 stories (incl. 1 conditional) cover M1 go/fallback gate. Next epic to decompose: Epic 3 (Arena Flight & First Combat / M2). -->

## Epic 3: Arena Flight & First Combat (First Playable)

Player flies a cockpit ship in the Arena, fires a prefab weapon, destroys asteroids, sees HUD with ship state. No enemies yet. Diegetic learning — no tutorial text (FR28 upheld as design constraint across all stories; not its own story). M-alignment: M2. FRs covered: FR1, FR2, FR3, FR5, FR8, FR9, FR12, FR24, FR27, FR43.

### Story 3.1: Title Screen Stub — MainMenu → Arena Transition

As a player launching the game,
I want a minimal title screen that lets me start a run with a single key press,
So that I reach gameplay from the first Epic-3 commit without dev hacks like default-to-Arena.

**Acceptance Criteria:**

**Given** the app is in `GameState::MainMenu` after Epic 1 Story 1.7 transition
**When** `OnEnter(GameState::MainMenu)` runs
**Then** a `bevy_ui` text Node is spawned with title "asteroids3D" plus subtitle "Press Enter to start"
**And** all spawned entities carry a `MainMenuEntity` marker component

**Given** the title screen is visible
**When** the player presses Enter / Return
**Then** `NextState<GameState>` is set to `Arena`

**Given** the state transitions `MainMenu → Arena`
**When** `OnExit(GameState::MainMenu)` runs
**Then** all `MainMenuEntity`-marked entities are despawned
**And** no orphaned title or subtitle text remains

Note: Stub. Full FR36 title screen (start / settings / credits / quit) is Epic 4.

### Story 3.2: Avian Physics Foundation + Arena State Skeleton

As a developer,
I want Avian XPBD registered in `FixedUpdate` at 60 Hz with gravity disabled, plus a `GameState::Arena` skeleton with OnEnter/OnExit hooks,
So that subsequent flight and combat stories attach to a deterministic physics world and a clean state-lifecycle.

**Acceptance Criteria:**

**Given** `avian3d` is added via `App::add_plugins(PhysicsPlugins::default())`
**When** the app runs
**Then** the physics schedule ticks at 60 Hz inside `FixedUpdate` per the architecture decision
**And** `Gravity(Vec3::ZERO)` is inserted as a Resource (zero-g space environment)

**Given** `src/arena/mod.rs` is authored
**When** `ArenaPlugin: Plugin` is added via `App::add_plugins`
**Then** the plugin declares an `ArenaSystems` SystemSet enum
**And** an `ArenaEntity` marker component is defined
**And** a `cleanup_on_exit::<ArenaEntity>` system is registered on `OnExit(GameState::Arena)`

**Given** the Arena state is entered from Story 3.1's trigger
**When** `OnEnter(GameState::Arena)` runs
**Then** an `info!` log `"entered Arena"` is emitted
**And** the scene is otherwise empty (zone content is Story 3.3)

**Given** the player transitions `Arena → MainMenu` (triggered later in Epic 4)
**When** `OnExit(GameState::Arena)` runs
**Then** every entity carrying `ArenaEntity` is despawned before the next state enters

### Story 3.3: Hand-Designed Arena Zone with Static Asteroid Field

As a player,
I want a hand-designed Arena zone with a visible asteroid field when I start a run,
So that I enter a perceivable 3D space with navigational reference points, not a void.

**Acceptance Criteria:**

**Given** `src/arena/zone.rs` is authored with a `spawn_arena_zone` system on `OnEnter(GameState::Arena)`
**When** the system runs
**Then** 15–25 asteroid entities are spawned at hand-picked `Transform` positions covering a roughly 200×200×200 m volume
**And** each asteroid uses a placeholder icosphere `Mesh3d` with radius 3.0–12.0 m
**And** each asteroid uses `ToonMaterial` from Epic 2 with `SemanticAccent::Neutral`
**And** each asteroid is an Avian `RigidBody::Static` with a `Collider::sphere` sized to its mesh radius
**And** each asteroid carries the `ArenaEntity` marker

**Given** toon shading needs a directional light for readable posterization
**When** `OnEnter(GameState::Arena)` runs
**Then** exactly one `DirectionalLight` is spawned with the `ArenaEntity` marker
**And** no ambient light beyond Bevy defaults (dark-space aesthetic preserved)

**Given** the Arena state exits
**When** `OnExit` runs
**Then** all asteroids and the `DirectionalLight` are despawned via `cleanup_on_exit::<ArenaEntity>`

### Story 3.4: Pause on Focus Loss + Pause Menu Stub

As a player,
I want the game to pause when I Alt-Tab away or press Escape,
So that the simulation never advances while I'm not looking and I never take invisible damage.

**Acceptance Criteria:**

**Given** `src/pause/mod.rs` is authored with a `PausePlugin`
**When** a `WindowFocused { focused: false, .. }` event arrives while `GameState` is `Arena` (or any in-gameplay state)
**Then** `NextState<GameState>` is set to `Paused`
**And** the physics simulation is paused per the applicable Bevy-0.18 / Avian-0.6 convention (e.g., `Time::<Virtual>::pause()` or the Avian-exposed physics-time control — resolved concretely at implementation time)

**Given** the app is in `GameState::Paused` via focus-loss
**When** `WindowFocused { focused: true, .. }` arrives
**Then** `NextState<GameState>` returns to the state stored in a `PausedFrom(GameState)` resource captured on pause entry
**And** the physics simulation resumes via the inverse of the pause convention

**Given** the app is in `GameState::Arena`
**When** the player presses Escape
**Then** `NextState<GameState>` is set to `Paused`
**And** a `bevy_ui` text Node "PAUSED — Esc to resume" is spawned with a `PauseOverlayEntity` marker

**Given** the app is in `GameState::Paused` via Escape
**When** the player presses Escape again
**Then** `NextState<GameState>` returns to `PausedFrom`
**And** `PauseOverlayEntity`-marked entities are despawned on `OnExit(GameState::Paused)`

### Story 3.5: Cockpit Camera + PlayerShip Entity

As a player,
I want a first-person cockpit camera attached to a visible ship placed in the Arena,
So that I see the game world from inside the ship — the FR8 cockpit-only commitment from frame one.

**Acceptance Criteria:**

**Given** `src/flight/mod.rs` is authored with a `FlightPlugin`
**When** `OnEnter(GameState::Arena)` runs (after Story 3.3's zone spawn)
**Then** exactly one `PlayerShip` entity is spawned with the `ArenaEntity` marker
**And** the `PlayerShip` has a placeholder cockpit mesh (cuboid or simple fighter silhouette) rendered with `ToonMaterial`
**And** the `PlayerShip` is an Avian `RigidBody::Dynamic` (gravity inherited zero from the world)
**And** the `PlayerShip` has a sphere or capsule `Collider` sized to the placeholder mesh

**Given** a cockpit camera is attached as a child
**When** the `PlayerShip` hierarchy is inspected
**Then** exactly one `Camera3d` entity is a child of `PlayerShip`
**And** the camera's local `Transform` places it at the pilot-seat wingtip-framing position (slightly behind and above the mesh origin, angled slightly downward)
**And** the camera carries a `CockpitCamera` marker component

**Given** the `PlayerShip` spawns in Arena
**When** the initial `Transform` is chosen relative to Story 3.3's zone layout
**Then** the ship has line of sight to at least 3 asteroids within 50 m
**And** the ship has zero initial linear and angular velocity (no drift at spawn)

**Given** no flight-input systems exist yet
**When** the game enters Arena
**Then** the player sees the asteroid field from the cockpit, stationary

### Story 3.6: Flight Input → 6-DOF Translation

As a player,
I want keyboard input to translate my ship forward, reverse, strafe left/right, and up/down in ship-local space,
So that I can navigate the Arena in 3D per FR2.

**Acceptance Criteria:**

**Given** `leafwing-input-manager` is registered in `FlightPlugin`
**When** `src/flight/input.rs` defines a `FlightAction` enum
**Then** it includes variants `ThrustForward`, `ThrustReverse`, `StrafeLeft`, `StrafeRight`, `ThrustUp`, `ThrustDown`
**And** default bindings are W/S (forward/reverse), A/D (strafe), Space/LCtrl (up/down)
**And** the `PlayerShip` from Story 3.5 is spawned with an `InputManagerBundle<FlightAction>` using the defaults

**Given** `TuningConfig` is extended with `ship_thrust_newtons: f32` (default 500.0)
**When** a thrust-application system runs in `FixedUpdate` inside `FlightSystems`
**Then** for each pressed `FlightAction` variant, an `ExternalForce` is applied in the ship-local direction (+Z forward, -Z reverse, ±X strafe, ±Y up/down) at magnitude `ship_thrust_newtons`

**Given** the player presses `ThrustForward` in Arena
**When** 2 seconds elapse
**Then** the ship's world linear velocity is approximately `(ship_thrust_newtons / mass) * 2` m/s forward within 10% integration tolerance
**And** the ship continues to drift after release (pure Newtonian — dampener is Story 3.8)

**Given** `ThrustForward` + `StrafeRight` are pressed simultaneously
**When** the composite force is inspected
**Then** the forces sum (diagonal motion in ship-local space)
**And** ship rotation is unchanged (translation does not induce rotation)

### Story 3.7: Flight Input → 3-Axis Rotation (Pitch / Yaw / Roll)

As a player,
I want mouse input to pitch and yaw my ship, plus Q/E to roll,
So that I can aim the ship freely per FR3.

**Acceptance Criteria:**

**Given** `FlightAction` is extended
**When** new variants are added
**Then** they include `Pitch` (DualAxis), `Yaw` (DualAxis), `RollLeft`, `RollRight`
**And** default bindings are Mouse Y → Pitch, Mouse X → Yaw, Q → RollLeft, E → RollRight

**Given** `TuningConfig` is extended with `mouse_sensitivity: f32` (default 1.0) and `ship_torque_nm: f32` (default 80.0)
**When** a rotation system runs in `FixedUpdate` inside `FlightSystems`
**Then** mouse delta values scaled by `mouse_sensitivity` are applied as `ExternalTorque` around ship-local pitch and yaw axes
**And** `RollLeft` / `RollRight` apply constant `±ship_torque_nm` around the ship-local roll axis

**Given** the player moves the mouse up
**When** the frame advances
**Then** the ship pitches up relative to its current roll orientation (pitch is ship-local, not world-up)

**Given** the player presses Q for 1 second
**When** the action ends
**Then** the ship has rotated approximately `ship_torque_nm / moment_of_inertia` radians around its local +Z axis
**And** angular velocity persists (no dampener yet)

**Given** the Arena state is active
**When** the mouse cursor is inspected
**Then** the cursor is confined (`CursorGrabMode::Confined`) and hidden (`visible: false`) so mouse motion maps cleanly to rotation

### Story 3.8: Inertial Dampener Toggle

As a player,
I want a toggleable inertial dampener that bleeds my linear and angular velocity toward zero when active,
So that I can modulate between Newtonian drift and arcade-tight control per FR5.

**Acceptance Criteria:**

**Given** `FlightAction` is extended
**When** a new variant is added
**Then** it is `ToggleDampener`
**And** the default binding is X (press to toggle)

**Given** a `DampenerState { active: bool }` component on the `PlayerShip` (initial `active = true`)
**When** the player presses `ToggleDampener`
**Then** `DampenerState.active` flips
**And** an `info!` log records the new state for dev feedback

**Given** `TuningConfig` is extended with `dampener_linear_strength: f32` (default 2.0) and `dampener_angular_strength: f32` (default 3.0)
**When** the dampener system runs in `FixedUpdate` inside `FlightSystems` and `DampenerState.active == true`
**Then** `ExternalForce` equal to `-linear_velocity * dampener_linear_strength * mass` is applied
**And** `ExternalTorque` equal to `-angular_velocity * dampener_angular_strength * moment_of_inertia` is applied

**Given** the dampener is active and the player releases all thrust/rotation input
**When** 3 seconds elapse
**Then** both linear and angular velocities are within 5% of zero

**Given** the dampener is inactive
**When** the player releases all input
**Then** velocities persist indefinitely (pure drift)

### Story 3.9: Weapon Firing + Projectile Ballistics

As a player,
I want to fire projectiles from my ship when I hold the primary-fire trigger,
So that I have an offensive capability in the Arena per FR9.

**Acceptance Criteria:**

**Given** a `CombatAction` enum is introduced in `src/combat/input.rs`
**When** the enum is defined
**Then** it includes `FirePrimary`
**And** the default binding is Left Mouse Button
**And** the `PlayerShip` bundle is extended to include `InputManagerBundle<CombatAction>`

**Given** `src/combat/mod.rs` is authored with a `CombatPlugin`
**When** the app starts
**Then** `CombatPlugin` declares a `CombatSystems` SystemSet
**And** `CombatPlugin` is registered in `main.rs`

**Given** `TuningConfig` is extended with `projectile_speed: f32` (120.0 m/s), `projectile_fire_rate_hz: f32` (4.0), `projectile_ttl_seconds: f32` (3.0)
**When** the player holds `FirePrimary`
**Then** a projectile-spawn system emits shots at `projectile_fire_rate_hz`, enforced via a `PrimaryWeaponCooldown { remaining: f32 }` component on the `PlayerShip`
**And** each shot spawns a `Projectile` entity with a placeholder sphere `Mesh3d` + `ToonMaterial`
**And** the `Projectile` is an Avian `RigidBody::Dynamic` with initial `LinearVelocity = ship_linear_velocity + ship_forward_vector * projectile_speed`
**And** the `Projectile` carries a `Collider`, a `Projectile { ttl: f32, damage: u32 }` component (default damage = 1), and the `ArenaEntity` marker

**Given** a `Projectile` with `ttl > 0`
**When** `FixedUpdate` runs
**Then** `ttl` decrements by the fixed timestep
**And** when `ttl <= 0` the `Projectile` entity is despawned

**Given** the player fires while stationary versus while drifting forward at 30 m/s
**When** projectile trajectories are measured
**Then** the drifting case's world velocity equals `30 + projectile_speed` forward
**And** the stationary case's world velocity equals `projectile_speed` forward

### Story 3.10: Projectile-Asteroid Collision & Damage

As a player,
I want my projectiles to destroy asteroids on contact,
So that I have a visible combat outcome per FR12, closing the core Arena interaction loop.

**Acceptance Criteria:**

**Given** Story 3.3's zone-spawner is extended
**When** each asteroid spawns
**Then** it carries an `AsteroidHp { current: u32 }` component with `current = 1` (single-hit destruction in Epic 3; multi-hit is Epic 4/5)

**Given** projectiles and asteroids share an Avian `CollisionLayers` group that allows contact
**When** a `Projectile` collides with an asteroid entity carrying `AsteroidHp`
**Then** a `ProjectileHitAsteroid { projectile: Entity, asteroid: Entity, damage: u32 }` event is emitted (past-tense PascalCase per architecture)
**And** `damage` is read from the `Projectile::damage` field

**Given** a `ProjectileHitAsteroid` event
**When** the damage-application system runs
**Then** `AsteroidHp.current = max(0, current - damage)`
**And** the projectile entity is despawned (single-hit-per-projectile — cluster-hit cases resolve to first contact only in Epic 3)

**Given** an asteroid reaches `AsteroidHp.current == 0`
**When** the destruction system runs
**Then** an `AsteroidDestroyed { asteroid: Entity }` event is emitted
**And** the asteroid entity is despawned
**And** the `cleanup_on_exit::<ArenaEntity>` pattern remains valid (mid-state despawn of individual ArenaEntity children does not break state-exit cleanup)

**Given** the player shoots an asteroid
**When** behaviour is observed
**Then** the projectile visibly disappears at impact
**And** the asteroid visibly disappears
**And** no error logs are emitted

### Story 3.11: HUD Baseline (Screen-Space Placeholders)

As a player,
I want a HUD showing placeholder shields, hull, ammo, and salvage values,
So that I learn where to look for tactical state before real values arrive in Epic 5 and 6.

**Acceptance Criteria:**

**Given** `src/ui/hud.rs` is authored with a `HudPlugin`
**When** `OnEnter(GameState::Arena)` runs
**Then** a full-window-absolute HUD root `Node` is spawned with a `HudEntity` marker

**Given** the HUD root exists
**When** child Nodes are spawned
**Then** four labeled fields are positioned in the four screen corners:
- top-left: `"SHIELDS 100"`
- top-right: `"HULL 100"`
- bottom-left: `"AMMO ∞"`
- bottom-right: `"SALVAGE 0"`

**And** each value is wired to a `HudPlaceholder { field: HudField }` component (enum variants Shields / Hull / Ammo / Salvage) for later Epic-5/6 connection

**Given** the HUD is visible
**When** the player flies, shoots, destroys asteroids
**Then** the four placeholder values remain static (real wiring is Epic 5/6)
**And** HUD elements do not obstruct the central line of sight to the asteroid field (transparent Node backgrounds, corner-anchored)

**Given** the state transitions `Arena → Paused` or `Arena → MainMenu`
**When** `OnExit(GameState::Arena)` runs
**Then** all `HudEntity`-marked entities are despawned (HUD is Arena-scoped in Epic 3; cross-state HUD persistence is Epic 4+)

<!-- Epic 3 complete — 11 stories deliver First Playable (M2). Next epic to decompose: Epic 4 (Enemies Alive & Stop-Ship / M3). -->

## Epic 4: Enemies Alive & Stop-Ship (Itch.io Prototype)

The Itch.io-shippable small game. Full combat loop: 3 weapons, 1 enemy with AI, permadeath on Hull-zero, post-run summary, immediate restart, title screen, settings (volume + sensitivity), persistent settings, unsigned macOS binary (signing deferred to E7), release workflow for per-OS ZIPs. The M3 stop-and-ship waypoint. M-alignment: M3 🏁. FRs covered: FR10, FR14, FR16, FR36, FR37, FR38, FR39, FR44, FR45, FR46, FR47, FR50 (FR48 → E7).

### Story 4.1: Enemy Entity Foundation + SemanticAccent::Enemy

As a player,
I want a visible enemy ship present in the Arena when I spawn,
So that I have a new target type to recognize before AI and combat dynamics come online in Story 4.2.

**Acceptance Criteria:**

**Given** `src/combat/enemy.rs` is authored
**When** `Enemy` and `EnemyShip` components are defined and integrated with `CombatPlugin`
**Then** `Enemy` is an empty marker component and `EnemyShip` optionally holds archetype data (currently a single variant)

**Given** `OnEnter(GameState::Arena)` runs after Story 3.3's zone spawn
**When** an enemy-spawn system runs
**Then** exactly one `EnemyShip` entity is spawned at a hand-picked `Transform` within ~80 m of the PlayerShip
**And** the enemy uses a placeholder mesh (distinct from PlayerShip) rendered with `ToonMaterial` + `SemanticAccent::Enemy` + `OutlineBundle`
**And** the enemy is an Avian `RigidBody::Dynamic` with a `Collider` sized to its mesh
**And** the enemy carries the `ArenaEntity` marker

**Given** the enemy is in the Arena
**When** the player approaches within line-of-sight
**Then** the enemy's red accent tint is visibly distinct from salvage (yellow) asteroids and the PlayerShip
**And** the enemy is stationary (AI is Story 4.2)

**Given** the Arena state exits
**When** `OnExit(GameState::Arena)` runs
**Then** the enemy is despawned via `cleanup_on_exit::<ArenaEntity>`

### Story 4.2: Enemy AI State Machine — Detect / Pursue / Attack

As a player,
I want the enemy to detect, pursue, and fire on me when in range,
So that the Arena has a live adversary per FR14 and combat has stakes.

**Acceptance Criteria:**

**Given** a unified `Health { current: u32, max: u32 }` component is introduced in `src/combat/health.rs`
**When** Story 3.10's `AsteroidHp` is refactored
**Then** asteroids now carry `Health { current: 1, max: 1 }` and all prior `AsteroidHp` references use `Health`
**And** the enemy is spawned with `Health { current: 2, max: 2 }` (2-shot enemy for first playable)

**Given** `src/combat/enemy_ai.rs` defines `EnemyAiState`
**When** the enum is declared
**Then** it includes variants `Idle`, `Detect`, `Pursue`, `Attack`
**And** each `EnemyShip` carries `EnemyAiState` (default `Idle`)

**Given** `TuningConfig` is extended with `enemy_detection_range: f32 = 100.0`, `enemy_engagement_range: f32 = 50.0`, `enemy_speed: f32 = 20.0`, `enemy_fire_rate_hz: f32 = 1.0`, `enemy_ai_hysteresis_pct: f32 = 0.1`
**When** an AI-transition system runs in `FixedUpdate` inside `CombatSystems`
**Then** the enemy transitions by distance-to-PlayerShip:
- `distance > detection_range` → `Idle`
- `detection_range ≥ distance > engagement_range` → `Detect` (rotates to face player)
- `engagement_range ≥ distance > engagement_range * 0.5` → `Pursue` (moves toward player)
- `distance ≤ engagement_range * 0.5` → `Attack` (stops, fires)

**And** each transition threshold applies `enemy_ai_hysteresis_pct` as a dead-band to prevent state flicker at boundaries

**Given** the enemy is in `Pursue`
**When** the movement system runs
**Then** `ExternalForce` is applied toward the PlayerShip position
**And** the enemy orients to face the PlayerShip
**And** speed is clamped to `enemy_speed`

**Given** the enemy is in `Attack`
**When** `enemy_fire_rate_hz` cooldown elapses
**Then** a projectile is spawned toward the PlayerShip using the same `Projectile` component as Story 3.9
**And** the projectile carries an `EnemyProjectile` marker so collision filtering distinguishes player vs enemy projectiles

**Given** a player projectile hits the enemy
**When** the damage system runs
**Then** the enemy's `Health.current` decreases by projectile damage
**And** at `Health.current == 0` an `EnemyDestroyed { enemy: Entity }` event is emitted, the enemy is despawned, and no respawn occurs (single-enemy encounter per Arena entry in Epic 4)

### Story 4.3: Hull Component + Permadeath → PostRun State

As a player,
I want my ship to take damage from enemy projectiles and the run to end on hull-zero,
So that permadeath per FR16 is real and combat has stakes.

**Acceptance Criteria:**

**Given** the unified `Health` component from Story 4.2
**When** PlayerShip is extended
**Then** it carries `Health { current: 3, max: 3 }` sourced from `TuningConfig.player_hull_max: u32 = 3`
**And** no regen applies in Epic 4 (regen is Epic 5)

**Given** enemy projectiles carrying `EnemyProjectile` from Story 4.2
**When** an enemy projectile collides with the PlayerShip
**Then** a `ProjectileHitPlayer { projectile: Entity, player: Entity, damage: u32 }` event is emitted
**And** the projectile is despawned
**And** `Health.current = max(0, current - damage)` on the PlayerShip

**Given** `HudPlaceholder::Hull` from Story 3.11
**When** PlayerShip `Health.current` changes
**Then** the HUD Hull field updates live to show the current value (first-playable real HUD wiring for Hull only; Shields stays placeholder until Epic 5)

**Given** PlayerShip Health reaches 0
**When** the permadeath system runs
**Then** a `HullDepleted { player: Entity, cause: DeathCause }` event is emitted
**And** `DeathCause` is an enum with variants `EnemyFire`, `AsteroidCollision`, `Unknown`
**And** a `RunResult { cause: DeathCause, salvage_banked: u32, run_duration_seconds: f32 }` resource is inserted (consumed by Story 4.9)
**And** `NextState<GameState>` is set to `PostRun`

**Given** the Arena → PostRun transition
**When** `OnExit(GameState::Arena)` runs
**Then** all `ArenaEntity`-marked entities are despawned per existing cleanup pattern

### Story 4.4: Weapon Archetype System + Shotgun / Railgun Archetypes

As a player,
I want three distinct weapon archetypes equipped in slots I can cycle,
So that I have tactical weapon choices per FR10.

**Acceptance Criteria:**

**Given** `src/combat/weapons.rs` is authored
**When** `WeaponArchetype` enum is defined
**Then** it includes `Pulse`, `Shotgun`, `Railgun`
**And** stats are loaded from `TuningConfig`:
- `Pulse`: damage=1, fire_rate=4.0 Hz, speed=120, projectiles=1, spread=0°
- `Shotgun`: damage=1, fire_rate=1.5 Hz, speed=80, projectiles=5, spread=15°
- `Railgun`: damage=5, fire_rate=0.5 Hz, speed=300, projectiles=1, spread=0°

**Given** PlayerShip is extended with a `WeaponLoadout { slots: [Option<WeaponArchetype>; 3], active_slot: usize }` component
**When** the PlayerShip spawns in Arena
**Then** the default loadout is `[Some(Pulse), Some(Shotgun), Some(Railgun)]` with `active_slot = 0`
**And** `CombatAction` is extended with `CycleWeapon` (default binding: Q, or 1/2/3 direct-select)

**Given** the player presses `CycleWeapon`
**When** the input system runs
**Then** `active_slot` advances cyclically (0 → 1 → 2 → 0)
**And** an `info!` log records the new active archetype

**Given** the player holds `FirePrimary` with an active archetype
**When** the fire system runs
**Then** projectiles spawn per archetype stats:
- Shotgun spawns 5 projectiles simultaneously in a cone within ±spread° around ship-forward
- Railgun spawns 1 projectile with high speed and high damage but a long cooldown
- Pulse behaviour equals the Story 3.9 default

**Given** each archetype
**When** projectiles spawn
**Then** shotgun / pulse / railgun projectiles are visually distinguishable (e.g., different mesh scales) — placeholder-grade, final visuals are Epic 10 polish

### Story 4.5: SemanticAccent Wiring — Asteroids=Salvage, PlayerShip+Projectiles=PlayerOwned

As a player,
I want the semantic accent palette visibly distinguishing salvage, enemies, and my own ship,
So that I can identify entity categories at a glance per FR50 and NFR-A1 redundant encoding.

**Acceptance Criteria:**

**Given** Story 3.3's asteroid spawner uses `SemanticAccent::Neutral`
**When** updated
**Then** asteroids use `SemanticAccent::Salvage` (yellow)

**Given** Story 3.5's PlayerShip spawner
**When** updated
**Then** PlayerShip uses `SemanticAccent::PlayerOwned` (cyan)

**Given** Story 3.9's player-projectile spawner
**When** updated
**Then** player projectiles use `SemanticAccent::PlayerOwned`
**And** Story 4.2's enemy projectiles use `SemanticAccent::Enemy`

**Given** all accent wiring applies
**When** the Arena is rendered with all entity types simultaneously visible
**Then** four accent tints (Salvage yellow / Enemy red / PlayerOwned cyan / any remaining Neutral) are simultaneously distinguishable under toon shading

### Story 4.6: PersistencePlugin + Save Schema v1

As a developer,
I want a `PersistencePlugin` that reads and atomically writes a versioned save at the OS-convention path,
So that settings, meta-currency, and future unlocks share one durable, crash-safe persistence mechanism per FR44/FR45/FR46.

**Acceptance Criteria:**

**Given** `src/persistence/mod.rs` is authored
**When** `PersistencePlugin` is added
**Then** the plugin declares `PersistenceSystems` SystemSet

**Given** `src/persistence/save_data.rs` defines the schema
**When** `SaveData` struct is declared with Serde derives
**Then** fields are: `version: u32 = 1`, `settings: Settings { master_volume: f32, sfx_volume: f32, mouse_sensitivity: f32 }`, `meta_currency: u32 = 0` (Epic 7 placeholder), `unlocked_upgrades: Vec<String> = []` (Epic 7 placeholder)

**Given** the `directories` crate resolves the per-OS data dir
**When** `save_path()` is called
**Then** it returns:
- Windows `%APPDATA%\asteroids3D\save.json`
- Linux `$XDG_DATA_HOME/asteroids3D/save.json` (or `~/.local/share/asteroids3D/save.json` fallback)
- macOS `~/Library/Application Support/asteroids3D/save.json`

**And** the directory is created if absent

**Given** `save(data: &SaveData) -> Result<(), SaveError>`
**When** called
**Then** `save.json.tmp` is written, fsync'd, then atomically renamed to `save.json` via `std::fs::rename` on Unix or the Windows-equivalent atomic-rename API
**And** `SaveError` is a `thiserror`-derived enum

**Given** `load() -> Result<SaveData, LoadError>`
**When** called and the file exists
**Then** JSON is parsed and returned
**And** schema-version mismatch returns `LoadError::VersionMismatch { found, expected }` (full migration path is Epic 5)

**Given** no save file exists on first launch
**When** `OnEnter(GameState::Loading)` runs and `load()` returns `FileNotFound`
**Then** `save(&SaveData::default())` creates a default save
**And** on subsequent launches the default is readable

**Given** load succeeds
**When** the app initializes
**Then** `SaveData` is inserted as a `Resource`
**And** settings are applied to audio + `TuningConfig.mouse_sensitivity`

**Given** a save write interrupted mid-flight (kill -9)
**When** the app relaunches
**Then** either the new save or the prior save is valid (never partial)
**And** if both are corrupt, `LoadError::Corrupt` surfaces, a warn! is logged, and a default save is written

### Story 4.7: Title Screen — Full FR36 (Start / Settings / Credits / Quit)

As a player opening the game,
I want a proper title screen with Start, Settings, Credits, and Quit,
So that I have a front-door menu per FR36, replacing the Story 3.1 stub.

**Acceptance Criteria:**

**Given** Story 3.1's stub title screen
**When** Story 4.7 is implemented
**Then** the stub is replaced with `src/ui/title.rs` under a `TitleScreenPlugin`
**And** `OnEnter(GameState::MainMenu)` spawns four `bevy_ui` buttons: `Start Run`, `Settings`, `Credits`, `Quit`
**And** each button has hover and press styling and carries a `MainMenuEntity` marker

**Given** the player selects a button via mouse click or keyboard (arrow keys + Enter)
**When** the action fires
**Then** the corresponding transition runs:
- `Start Run` → `NextState<GameState>` to `Arena`
- `Settings` → `NextState<GameState>` to `Settings`
- `Credits` → `NextState<GameState>` to `Credits`
- `Quit` → emits `AppExit::Success`

**Given** `GameState` enum from Story 1.6
**When** Story 4.7 is merged
**Then** `Settings` and `Credits` variants are added

**Given** `OnEnter(GameState::Credits)` runs
**When** the credits screen is built
**Then** a static text Node shows: title, `env!("CARGO_PKG_VERSION")`, "by Till Fechteler, 2026", key plugin credits (Bevy, Avian, bevy_mod_outline, bevy_kira_audio, leafwing-input-manager), and "Press Esc or Backspace to return"
**And** Esc or Backspace returns to `MainMenu`

### Story 4.8: Settings Menu (Master / SFX Volume + Mouse Sensitivity)

As a player,
I want a Settings menu with sliders for master volume, SFX volume, and mouse sensitivity that persist across launches,
So that I can customize audio and feel per FR37.

**Acceptance Criteria:**

**Given** `src/ui/settings.rs` is authored
**When** `OnEnter(GameState::Settings)` runs
**Then** three slider UI elements are spawned for `master_volume` (0.0–1.0), `sfx_volume` (0.0–1.0), `mouse_sensitivity` (0.1–3.0)
**And** initial values are read from the `SaveData` Resource's `settings` field
**And** a "Back" button and `SettingsEntity` marker are present
**And** slider control is mouse-only in Epic 4 (keyboard nav is Epic 10 polish)

**Given** the player drags a slider
**When** the value changes
**Then** `SaveData.settings` is updated in-memory
**And** the change applies live:
- `master_volume` → `bevy_kira_audio` master mixer
- `sfx_volume` → SFX channel mixer
- `mouse_sensitivity` → `TuningConfig.mouse_sensitivity`

**Given** the player clicks Back or presses Esc
**When** the transition triggers
**Then** `save(&save_data)` is called
**And** `NextState<GameState>` returns to `MainMenu`
**And** on next launch, values are restored

**Given** a save-write error (e.g., disk full)
**When** the Back button attempts to save
**Then** the error is `warn!`-logged, the transition still completes (no modal error — Principle 4 death-as-feedback extends to non-critical errors)

### Story 4.9: Post-Run Summary Screen (Retry / Main Menu)

As a player,
I want a post-run summary showing why my run ended, with options to retry or return to menu,
So that I receive feedback on death per FR38 and re-engage quickly per FR39 — without any "GAME OVER" defeat frame per Design Principle 4.

**Acceptance Criteria:**

**Given** `src/ui/post_run.rs` is authored with a `PostRunPlugin`
**When** `OnEnter(GameState::PostRun)` runs
**Then** a summary Node is spawned showing:
- Header: "Run ended" (neutral — no "GAME OVER", no "YOU DIED", no red overlay)
- Cause-of-death text from `RunResult.cause` (e.g., "Hull depleted by enemy fire")
- "Salvage banked: 0" (placeholder for Epic 6 economy)
- Two buttons: `Retry` and `Main Menu`

**And** all spawned entities carry a `PostRunEntity` marker

**Given** the player clicks `Retry`
**When** the action fires
**Then** `NextState<GameState>` is set to `Arena` directly (does not route through `MainMenu`)

**Given** the player clicks `Main Menu`
**When** the action fires
**Then** `NextState<GameState>` is set to `MainMenu`

**Given** Design Principle 4 compliance is reviewed
**When** PostRun is audited
**Then** no "GAME OVER" string exists anywhere, no red-tinted overlay, no defeat music crossfade, no death-specific screen shake
**And** ambient audio (once added in Epic 8) continues seamlessly across Arena → PostRun

**Given** the state transitions `PostRun → Arena` or `PostRun → MainMenu`
**When** `OnExit(GameState::PostRun)` runs
**Then** all `PostRunEntity`-marked entities are despawned

### Story 4.10: Cross-Platform Release Workflow — per-OS ZIPs

As the project author,
I want a GitHub Actions release workflow that builds per-OS release binaries and ZIPs them on tag push,
So that M3 Itch.io shipping has reproducible artifacts per FR47 with minimal manual overhead.

**Acceptance Criteria:**

**Given** `.github/workflows/release.yml` is authored
**When** a git tag matching `v*` is pushed
**Then** three parallel jobs run on `windows-latest`, `ubuntu-latest`, `macos-latest` (Apple Silicon arm64)

**Given** each job executes
**When** it runs
**Then** it:
- Runs `cargo build --release`
- Stages the binary + full `assets/` directory into `asteroids3D-<os>-<version>/`
- ZIPs to `asteroids3D-{windows-x64|linux-x64|macos-arm64}-<version>.zip`
- Uploads the ZIP as a GitHub Actions artifact

**And** the macOS build is **unsigned** per Till's decision on 2026-04-22 (signing deferred to Epic 7 / M6)
**And** Intel x86_64 macOS support is explicitly deferred to Epic 7 / M6 — M3 ships macOS arm64 only

**Given** all three jobs pass
**When** the workflow completes
**Then** a GitHub draft Release is created attached to the tag with all three ZIPs as assets

**Given** the draft Release exists
**When** Till manually reviews
**Then** the release-process runbook `docs/release-process.md` documents: (1) version bump in Cargo.toml, (2) `git tag v<version>`, (3) `git push --tags`, (4) wait for CI, (5) download artifacts, (6) smoke-test ZIP launches locally on at least one platform, (7) upload to Itch.io via web UI (or `butler push` CLI — optional), (8) publish GitHub Release
**And** the runbook flags that macOS users must right-click → Open on first launch to bypass Gatekeeper (unsigned build)

<!-- Epic 4 complete — 10 stories deliver M3 stop-and-ship (Itch.io prototype). FR48 deferred to Epic 7. Next epic to decompose: Epic 5 (Ship Subsystem State & Formal Save Schema / M4). -->

## Epic 5: Ship Subsystem State & Formal Save Schema

Formal Hull + Shields subsystems with regen mechanics. Shields regenerate after cooldown; Hull does not regen mid-run. Ship state readable at-a-glance. Save schema formalized with versioning for post-MVP expansion. Decoupled aim reticle system. M-alignment: M4. FRs covered: FR4, FR15. NFRs covered: NFR-R3, NFR-R4, NFR-U2, NFR-U3.

### Story 5.1: Formal HullHp + ShieldHp Components (Refactor from Health)

As a developer,
I want formal `HullHp` and `ShieldHp` components on PlayerShip, replacing the unified `Health` introduced in Epic 4,
So that FR15's distinct subsystem regen behaviours have a clean component boundary and asteroids/enemies can keep the simple `Health` model unchanged.

**Acceptance Criteria:**

**Given** `src/combat/health.rs` has a unified `Health { current: u32, max: u32 }` from Story 4.2
**When** Story 5.1 is implemented
**Then** two new components are added to the same module:
- `HullHp { current: u32, max: u32 }` — no regen
- `ShieldHp { current: u32, max: u32, regen_rate_per_sec: f32, regen_cooldown_sec: f32, last_damage_time_sec: f32 }`

**And** `Health` remains for asteroids and enemies

**Given** PlayerShip from Story 4.3 currently carries `Health`
**When** PlayerShip is updated
**Then** `Health` is removed from PlayerShip
**And** `HullHp { current: 3, max: 3 }` is added, sourced from `TuningConfig.player_hull_max: u32 = 3`
**And** `ShieldHp { current: 5, max: 5, regen_rate_per_sec: 1.0, regen_cooldown_sec: 3.0, last_damage_time_sec: 0.0 }` is added, sourced from `TuningConfig.player_shield_*`

**Given** the refactor is complete
**When** the compiler runs
**Then** no references to PlayerShip `Health` remain
**And** all prior damage-routing systems compile (routing logic is re-wired in Story 5.3)

**Given** asteroids and enemies from Epics 3–4
**When** they spawn
**Then** they continue to carry `Health` (shields/hull are player-only in Epic 5)

### Story 5.2: Shield Regen System + Damage Cooldown Tracking

As a player,
I want my shield to regenerate after a cooldown when I avoid damage,
So that I have a renewable defence layer per FR15, incentivizing tactical combat pacing.

**Acceptance Criteria:**

**Given** `src/combat/shield_regen.rs` is authored
**When** a regen system is registered in `FixedUpdate` within `CombatSystems`
**Then** for each entity with `ShieldHp`:
- If `shield.current < shield.max` AND `time.elapsed_sec - shield.last_damage_time_sec >= shield.regen_cooldown_sec`
- Then `shield.current += shield.regen_rate_per_sec * time.delta_sec()`, clamped to `shield.max`

**Given** the player takes shield damage (site is Story 5.3)
**When** the damage-routing system updates `shield.current`
**Then** it also updates `shield.last_damage_time_sec = time.elapsed_sec` (cross-story contract)

**Given** the player is undamaged for `regen_cooldown_sec` seconds
**When** the regen system runs each frame
**Then** `shield.current` increases at `regen_rate_per_sec` toward `shield.max`
**And** no regen occurs during the cooldown window

**Given** regen would push `current` above `max` in a single tick
**When** the clamp applies
**Then** `shield.current` saturates at `shield.max` (no overshoot)

### Story 5.3: Damage Routing — Shields Absorb First, Spill to Hull

As a player,
I want incoming damage to deplete my shields first, with spillover hitting my hull,
So that shields meaningfully protect me per FR15 and permadeath only triggers on hull-zero.

**Acceptance Criteria:**

**Given** Story 4.3's `ProjectileHitPlayer` event
**When** Story 5.3 updates the damage-application system
**Then** the system reads `ShieldHp` and `HullHp` on PlayerShip and applies damage in sequence:
- `shield_damage = min(incoming, shield.current)`
- `shield.current -= shield_damage`
- `remaining = incoming - shield_damage`
- `hull_damage = min(remaining, hull.current)`
- `hull.current -= hull_damage`

**And** `shield.last_damage_time_sec = time.elapsed_sec` is set iff `shield_damage > 0`

**Given** damage is routed
**When** the system emits events
**Then** a `DamageApplied { entity: Entity, shield_damage: u32, hull_damage: u32 }` event is emitted

**Given** `HullHp.current` reaches 0
**When** the permadeath system from Story 4.3 observes
**Then** `HullDepleted` fires and transitions to `PostRun` (behaviour unchanged)

**Given** a 1-damage projectile hits a player with `shield.current = 5`, `hull.current = 3`
**When** damage resolves
**Then** `shield.current = 4`, `hull.current = 3`

**Given** a 5-damage projectile hits a player with `shield.current = 2`, `hull.current = 3`
**When** damage resolves
**Then** `shield.current = 0`, `hull.current = 0` (overkill) and `HullDepleted` fires

### Story 5.4: HUD Wiring for Shields + Hull — Instrument-Panel Styling

As a player,
I want shield and hull state as color-coded bars in my peripheral vision,
So that I can read ship status mid-combat without looking away from targets per NFR-U2 and NFR-U3.

**Acceptance Criteria:**

**Given** Story 3.11's HUD placeholders and Story 4.3's live-Hull text
**When** Story 5.4 replaces Shield + Hull fields
**Then** the placeholder texts are replaced with bar visuals
**And** each bar is a `bevy_ui` Node with a fill child sized via `Style.width = Val::Percent(current / max * 100)`

**Given** instrument-panel styling per architecture
**When** bars render
**Then**:
- Shield bar: cyan (`SemanticAccent::PlayerOwned`), subtle tick-mark decoration
- Hull bar: orange-red (`SemanticAccent::Hazard` or dedicated hull color), urgency palette
- Both bars have a thin toon-outline border
- Numeric value ("3 / 3") overlaid in small text adjacent

**Given** NFR-U2 (simultaneous visibility)
**When** PlayerShip is in Arena
**Then** both bars are visible without overlap or occlusion from ammo/salvage fields
**And** bars sit in instrument-panel corner layout (shield top-left, hull top-right)

**Given** NFR-U3 (at-a-glance)
**When** shield is 5/5 vs 2/5 vs 0/5
**Then** fill-percentage is visibly different under a 1-second glance from 60–80 cm at 1080p
**And** a playtest check confirms testers report ship state correctly in >90% of 1-second glances at random HP values

**Given** shields deplete to 0
**When** the bar hits empty
**Then** a subtle "shield-offline" visual cue triggers (brief flash or red warning chevron adjacent to bar) — audio cue is Epic 8

**Given** shields are regenerating
**When** the bar updates each frame
**Then** fill animates smoothly (no visible quantized jumps at default `regen_rate_per_sec = 1.0`)

### Story 5.5: Decoupled Aim Reticle — Hold-to-Enable

As a player,
I want to aim my weapons independently of ship heading by holding Right Mouse Button,
So that I can attack targets at oblique angles without realigning the whole ship per FR4.

**Acceptance Criteria:**

**Given** `CombatAction` from Story 3.9
**When** extended
**Then** `HoldDecoupledAim` is added bound to Right Mouse Button (hold-to-enable)

**Given** `src/combat/aim.rs` is authored
**When** `AimDirection { world_vector: Vec3 }` is defined on PlayerShip
**Then** `AimDirection` default-initializes to `ship_forward_vector` on spawn
**And** when `HoldDecoupledAim` transitions released→held, `AimDirection` is reset to `ship_forward_vector` (decoupled aim starts centered on current facing)

**Given** `HoldDecoupledAim` is held
**When** mouse-delta arrives
**Then** Story 3.7's ship-rotation pitch/yaw is suspended for that frame (no ExternalTorque on pitch/yaw)
**And** mouse deltas rotate `AimDirection.world_vector` around ship-local pitch/yaw axes at the same sensitivity as ship rotation
**And** roll (Q/E) continues to rotate the ship normally (roll is nonsensical for aim — decoupling is pitch/yaw only)

**Given** `HoldDecoupledAim` is released
**When** the frame advances
**Then** Story 3.7's ship-rotation input resumes
**And** `AimDirection` persists at its last decoupled value until the next hold

**Given** the weapon-firing system from Story 3.9 / 4.4
**When** the player fires with `HoldDecoupledAim` held
**Then** projectiles spawn with `LinearVelocity = ship_velocity + aim_direction * projectile_speed`
**And** with `HoldDecoupledAim` NOT held
**Then** projectiles spawn along `ship_forward_vector` as before

**Given** `HoldDecoupledAim` held AND aim within the camera frustum
**When** the reticle HUD system runs
**Then** a reticle UI element renders at the screen-space projection of `ship_position + aim_direction * 100 m`
**And** the reticle shape conveys the active WeaponArchetype (shotgun: wider cone hint; pulse: medium crosshair; railgun: tight precision crosshair)

**Given** `HoldDecoupledAim` held AND aim projects off-screen
**When** the edge-indicator system runs
**Then** an arrow/edge-marker renders on the nearest screen edge toward the off-screen aim
**And** the indicator disappears when aim re-enters the frustum

**Given** `HoldDecoupledAim` NOT held
**When** the reticle system runs
**Then** a static ship-forward crosshair renders at screen-center (simple, WeaponArchetype-agnostic)
**And** no edge indicator is shown

### Story 5.6: Save Schema Migration Scaffold + Missing-Save Recovery Prompt

As a developer and as a player,
I want a save-schema migration path plus graceful recovery prompts when the save is missing or corrupt,
So that future schema changes don't brick existing users per NFR-R3/NFR-R4 and players never lose progression silently.

**Acceptance Criteria:**

**Given** `SaveData.version: u32` is currently 1 from Story 4.6
**When** Story 5.6 is implemented
**Then** `src/persistence/migration.rs` is authored with `migrate(raw: serde_json::Value, found_version: u32) -> Result<SaveData, MigrationError>`
**And** the function dispatches to per-version handlers (`fn v0_to_v1(v: serde_json::Value) -> serde_json::Value`, etc.)
**And** `current_version = 1` — no production migration active in Epic 5; scaffold exists for future Epics

**Given** a `#[cfg(test)]` synthetic v0 fixture (missing `version` field or missing a later-added field)
**When** the migration chain runs against it
**Then** the fixture upgrades cleanly to valid `SaveData` at `current_version = 1`
**And** the test lives in `src/persistence/migration_tests.rs`

**Given** a save file whose `version` is higher than current
**When** the load system encounters it
**Then** `LoadError::VersionTooNew { found, current }` is returned
**And** the app refuses to auto-clobber (does not create default over it)
**And** the recovery prompt is triggered

**Given** a `.initialized` sentinel file in the save dir distinguishes first-launch from existing-user
**When** the app starts
**Then**:
- No save AND no sentinel → **first launch**: default save created silently, sentinel written, proceed to MainMenu (preserves Story 4.6 first-launch UX)
- No save AND sentinel exists → **save-missing recovery prompt**
- Save unreadable (malformed JSON) → **save-corrupt recovery prompt**
- Save version-too-new → **version-too-new recovery prompt**

**Given** a recovery prompt is triggered
**When** `OnEnter(GameState::Loading)` proceeds
**Then** a modal `bevy_ui` Node shows: human-readable cause, a Yes button (accept default-save with "meta progression lost" warning), a Quit button (`AppExit::Success` no disk write)
**And** Yes:
- Missing-save: creates default, proceeds to MainMenu
- Corrupt / version-too-new: backs up the broken save to `<path>.corrupt-<UTC-timestamp>` first, then creates default, then proceeds

**And** Quit exits cleanly with no disk side-effects

**Given** a sentinel-write is interrupted
**When** the app relaunches
**Then** absence of sentinel is safe — worst case is first-launch UX shown a second time; no data loss beyond what the interrupted run had written

<!-- Epic 5 complete — 6 stories cover M4 subsystem formalization + save migration scaffold. Next epic to decompose: Epic 6 (Caravan Run Framework / M5). -->

## Epic 6: Caravan Run Framework

Player flies a Caravan run from start to target destination (5–8 min) with waypoint navigation, selectable difficulty (easy/medium/hard), trigger-volume combat pockets, tractor-beam intact-asteroid pickup, boost, pay-to-shoot economy with intact > destroyed yield math, salvage banking on success, death, and abort-forfeit. **⚠️ Danger Stretch (M5):** stories sliced into weekly sub-milestones per epic-summary guidance. M-alignment: M5. FRs covered: FR6, FR7, FR11, FR13, FR17, FR29, FR30, FR31, FR32, FR33, FR34, FR35.

### Story 6.1: Caravan State + RunPlugin Skeleton

As a developer,
I want a `Caravan` GameState and a `RunPlugin` with run-lifecycle events,
So that subsequent Epic 6 stories have a common state-and-lifecycle foundation.

**Acceptance Criteria:**

**Given** `GameState` enum from Story 1.6
**When** extended
**Then** `Caravan` and `DifficultyPicker` variants are added

**Given** `src/run/mod.rs` is authored with `RunPlugin`
**When** added to main.rs
**Then** it declares `RunSystems` SystemSet, defines `CaravanEntity` marker + `cleanup_on_exit::<CaravanEntity>` on OnExit(GameState::Caravan)

**Given** events are registered
**When** inspected
**Then** `RunStarted { difficulty, start_time_sec }` and `RunEnded { outcome, salvage_banked, duration_sec }` exist
**And** `Difficulty` enum = `Easy, Medium, Hard` (parameters in Story 6.13)
**And** `RunOutcome` enum = `TargetReached, HullDepleted, Aborted`

**Given** OnEnter(GameState::Caravan) fires
**When** run-start runs
**Then** `RunStarted` is emitted
**And** `CurrentRun { start_time_sec, difficulty, salvage_accumulated: 0 }` resource is inserted

**Given** OnExit(GameState::Caravan) fires
**When** run-end runs
**Then** `RunEnded` is emitted with computed outcome and duration
**And** `CaravanEntity`-marked entities are despawned
**And** `CurrentRun` resource is removed

### Story 6.2: Arena → Caravan Gate — Tutorial-Complete Sentinel + Difficulty-Picker Stub

As a player,
I want Arena on first run only, subsequent runs starting via a difficulty picker into Caravan,
So that FR29's tutorial-once-then-main-game flow works.

**Acceptance Criteria:**

**Given** SaveData from Story 4.6
**When** extended
**Then** a `tutorial_complete: bool` (default `false`) is added
**And** `SaveData.version` is bumped to 2 with a `v1_to_v2` migration injecting the default — the first real use of Story 5.6's migration scaffold

**Given** the player destroys the first asteroid ever (first `AsteroidDestroyed` in `GameState::Arena`)
**When** the tutorial-complete system observes
**Then** `SaveData.tutorial_complete = true` is set in-memory
**And** `save(&save_data)` persists the flag

**Given** the player clicks "Start Run" from MainMenu
**When** the router decides next state
**Then** `tutorial_complete == false` → `Arena`, else → `DifficultyPicker`

**Given** OnEnter(GameState::DifficultyPicker)
**When** the picker is built
**Then** three buttons spawn (Easy / Medium / Hard) + a Back button to MainMenu
**And** entities carry `DifficultyPickerEntity` marker

**Given** the player clicks a difficulty
**When** the action fires
**Then** `PickedDifficulty(Difficulty)` resource is inserted
**And** `NextState<GameState>` = `Caravan`
**And** OnEnter(Caravan) reads `PickedDifficulty` into `CurrentRun.difficulty`

**Given** OnExit(DifficultyPicker)
**When** cleanup runs
**Then** `DifficultyPickerEntity`-marked entities are despawned

### Story 6.3: Caravan Zone Layout — Start + Target + Landmarks

As a player,
I want a hand-designed Caravan zone with clear start, distant target, and a few navigational landmarks,
So that I have perceivable 3D space with orientation cues.

**Acceptance Criteria:**

**Given** `src/run/zone.rs` with `spawn_caravan_zone` on OnEnter(Caravan)
**When** the system runs
**Then** start-point = PlayerShip spawn; target-point ~800 m away (500–1000 m bracket)
**And** the target entity carries a `CaravanTarget` marker
**And** path is non-linear — 2–3 landmark asteroid clusters hand-placed to require gentle course correction

**Given** asteroid clusters spawn
**When** placed
**Then** each cluster has 5–15 asteroids (3.0–12.0 m radius)
**And** asteroids use `ToonMaterial` + `SemanticAccent::Salvage` + `OutlineBundle`
**And** each is `RigidBody::Static` with sphere `Collider`
**And** each has `Health { current: 1, max: 1 }` and `CaravanEntity` marker

**Given** combat-pocket hooks are needed
**When** the zone spawner places hidden spawn data
**Then** 2–4 `CombatPocketSpawn { pocket_id: u32, spawn_positions: Vec<Vec3> }` entities are spawned at hand-picked positions along the path
**(Trigger logic itself is Story 6.8)**

**Given** toon shading readability
**When** OnEnter(Caravan) runs
**Then** one `DirectionalLight` with `CaravanEntity` marker is spawned

**Given** the PlayerShip spawn
**When** placed
**Then** it spawns at start-point with zero velocity, oriented toward target (so Story 6.4's pointer is initially on-screen)

### Story 6.4: Waypoint-Pointer World-Space HUD

As a player,
I want a cockpit indicator showing direction + distance to target,
So that I can navigate per FR33.

**Acceptance Criteria:**

**Given** `src/ui/waypoint.rs` integrated into `HudPlugin`
**When** OnEnter(Caravan) runs
**Then** a `WaypointPointer` entity spawns as a child of the cockpit camera
**And** the entity uses a simple 3D arrow/chevron placeholder mesh (final mesh is Epic 10 polish)
**And** it is world-space but pinned at a fixed cockpit-frame position (e.g., top-center just under windshield rim)

**Given** the waypoint pointer exists
**When** the update system runs each frame
**Then** the pointer rotates to aim from its cockpit position toward `CaravanTarget` world position
**And** rotation uses shortest-arc quaternion interpolation (no flip)

**Given** target is at distance D
**When** the distance-display system runs
**Then** a small text Node below the pointer shows "<D> m" rounded to nearest 10 m
**And** it updates each frame

**Given** the player approaches target
**When** D drops below 50 m
**Then** the pointer + distance pulse subtly (color shift or scale) as a "near" cue — arrival detection is Story 6.6

**Given** target is reached (Story 6.6)
**When** OnExit(Caravan) runs
**Then** the pointer disappears with Caravan cleanup

### Story 6.5: Salvage Currency Resource + HUD Wiring

As a player,
I want my live salvage count visible on HUD during Caravan,
So that I can track economy per FR17 and judge pay-to-shoot affordability.

**Acceptance Criteria:**

**Given** Story 3.11's "SALVAGE 0" placeholder
**When** Story 6.5 wires real data
**Then** the field reads `CurrentRun.salvage_accumulated` live each frame

**Given** `src/run/salvage.rs` is authored
**When** salvage APIs are defined
**Then**:
- `fn add_salvage(current_run: &mut CurrentRun, amount: u32, source: SalvageSource)` exposed; emits `SalvageAdded` event
- `fn spend_salvage(current_run: &mut CurrentRun, amount: u32) -> Result<(), InsufficientFunds>` exposed
- `SalvageSource` enum = `IntactCapture, DestroyedAsteroid, Other`
- `InsufficientFunds` is a simple marker error type

**Given** `spend_salvage` is called with amount > `salvage_accumulated`
**When** returned
**Then** `Err(InsufficientFunds)` is returned and nothing is modified

**Given** HUD reflects the resource
**When** accumulation or spending occurs
**Then** the HUD updates with at most one-frame lag (reactive, not cached stale)

### Story 6.6: Caravan Duration + RunCompleted on Target Reached

As a player,
I want the run to complete when I reach the target within 5–8 min,
So that FR30's duration commitment has a clear success condition.

**Acceptance Criteria:**

**Given** `CaravanTarget` from Story 6.3
**When** Story 6.6 is implemented
**Then** an Avian sensor `Collider` (sphere, radius 20 m, `TuningConfig.target_trigger_radius`) is attached
**And** the sensor emits Avian collision events on PlayerShip entry

**Given** PlayerShip enters the target sensor
**When** target-reached system observes
**Then** salvage is banked via Story 6.7's banking system
**And** `RunEnded { outcome: TargetReached, salvage_banked, duration_sec }` is emitted
**And** `NextState<GameState>` = `PostRun`

**Given** arrival in <5 min or >8 min
**When** run-ended system logs
**Then** an `info!` log records actual duration (FR30 is design target, not hard constraint — tuning via zone layout Story 6.3)

**Given** the player presses Esc during Caravan
**When** the abort path is triggered
**Then** `NextState<GameState>` = `MainMenu`
**And** `RunEnded { outcome: Aborted, .. }` is emitted
**And** abort processing proceeds to Story 6.7 (forfeit, no banking)

### Story 6.7: Salvage Banking on Run End — Success + Death + Abort

As a player,
I want salvage banked into meta-currency on run end, with different policies per outcome,
So that FR34/FR35 progression commitments and abort-anti-abuse both hold.

**Acceptance Criteria:**

**Given** Epic 7's `meta_currency: u32` placeholder in SaveData (Story 4.6)
**When** Story 6.7 banks
**Then** `SaveData.meta_currency += CurrentRun.salvage_accumulated * banking_rate`
**And** `banking_rate: f32` from `TuningConfig.salvage_banking_rate = 1.0` default (Epic 7 may retune)

**Given** `RunEnded { outcome: TargetReached }`
**When** banking runs
**Then** full salvage banks, `save(&save_data)` persists

**Given** `RunEnded { outcome: HullDepleted }`
**When** banking runs
**Then** salvage STILL banks at full rate (FR35 no-death-penalty)
**And** `save(&save_data)` persists

**Given** `RunEnded { outcome: Aborted }` (Esc → Menu)
**When** banking runs
**Then** salvage is NOT banked (forfeit — prevents Esc-abuse to dodge death)
**And** an `info!` log records "run aborted, salvage forfeited"

**Given** FR35's "meta-unlocks persist" clause
**When** banking runs
**Then** `SaveData.unlocked_upgrades` is untouched
**And** a unit test verifies: start with 3 unlocks → run ends (any outcome) → reload → still 3 unlocks

### Story 6.8: Combat-Pocket Trigger System

As a player,
I want enemies to ambush me at predetermined points when I get close,
So that the route feels populated per FR32.

**Acceptance Criteria:**

**Given** `CombatPocketSpawn` entities from Story 6.3
**When** Story 6.8 is implemented
**Then** each has an Avian sensor `Collider` (sphere, radius `TuningConfig.pocket_trigger_radius: f32 = 150.0`)
**And** the sensor detects PlayerShip entry

**Given** PlayerShip enters a pocket trigger
**When** trigger-fire system observes
**Then** the pocket's `triggered: bool` flag is set (prevents retrigger)
**And** for each spawn position in `spawn_positions`, one `EnemyShip` entity spawns at that position (clamped to [`pocket_enemy_count_min = 1`, `pocket_enemy_count_max = 3`, `spawn_positions.len()`])
**And** spawned enemies carry `CaravanEntity` marker

**Given** enemies spawn
**When** Story 4.2's `EnemyAiState` system runs
**Then** newly-spawned enemies start in `Idle` and transition normally

**Given** a pocket was triggered
**When** the player leaves and re-enters
**Then** no new enemies spawn (`triggered == true` persists)
**And** existing enemies remain until destroyed

### Story 6.9: Pay-to-Shoot Economy

As a player,
I want each shot to cost salvage,
So that I have economic tension per FR11.

**Acceptance Criteria:**

**Given** Story 3.9 / 4.4's primary-fire flow
**When** Story 6.9 updates pre-fire
**Then** `spend_salvage(&mut current_run, shot_cost)` is called from Story 6.5 API
**And** `shot_cost: u32` is archetype-specific from `TuningConfig.shot_cost_per_weapon` (Pulse=1, Shotgun=3, Railgun=5)

**Given** `spend_salvage` returns `Ok(())`
**When** fire proceeds
**Then** the projectile spawns normally (all existing fire mechanics preserved)
**And** a `SalvageSpent { amount, reason: SpendReason::Shot }` event is emitted (for future analytics / Epic 10 tuning)

**Given** `spend_salvage` returns `Err(InsufficientFunds)`
**When** fire aborts
**Then** no projectile spawns
**And** the SALVAGE HUD field flashes red for 0.3 s
**And** no weapon audio / fire animation plays (no phantom feedback)

**Given** 0-salvage bankruptcy per Till's decision 2026-04-22
**When** the player is at 0 salvage with enemies pursuing
**Then** no shots are possible until salvage is earned (Story 6.11 yields)
**And** this is a design feature (tension, tactical choice)

**Given** `GameState::Arena` (tutorial)
**When** the player fires in Arena
**Then** pay-to-shoot is DISABLED — shots fire without salvage check
**And** gating is via a `ShotEconomyActive: bool` resource, set true on OnEnter(Caravan), false on OnExit

### Story 6.10: Tractor-Beam Intact Asteroid Capture

As a player,
I want a hold-to-tractor beam that pulls an asteroid until it collides with me for intact capture,
So that FR7's intact-capture mechanic gives me a higher-yield alternative to shooting.

**Acceptance Criteria:**

**Given** `CombatAction` from Story 3.9
**When** extended
**Then** `HoldTractorBeam` is added bound to E key (or F — implementation choice)

**Given** `src/run/tractor.rs` is authored
**When** systems are registered
**Then** a target-acquisition system runs each frame:
- While held, find the asteroid whose direction-from-player is closest to `aim_direction` (or `ship_forward_vector` if not in decoupled aim), distance ≤ `TuningConfig.tractor_range: f32 = 60.0`
- Best candidate is stored in a `TractorTarget(Option<Entity>)` Resource
- No candidate → `TractorTarget = None`

**Given** `TractorTarget = Some(asteroid)`
**When** the pull system runs in FixedUpdate
**Then** an `ExternalForce` magnitude `TuningConfig.tractor_force: f32 = 500.0` is applied toward PlayerShip
**And** if the asteroid is `RigidBody::Static`, it is converted to `RigidBody::Dynamic` one-time on first engagement (so it can actually move)

**Given** a tractored asteroid is being pulled
**When** VFX system runs
**Then** a world-space line (bevy_gizmos or simple cylinder mesh) connects PlayerShip → asteroid
**And** the line pulses or flows as a "beam active" placeholder cue (final VFX Epic 10)

**Given** a tractored asteroid contacts the PlayerShip
**When** the capture system observes the Avian collision
**Then** `AsteroidCaptured { asteroid, size_category }` is emitted (past-tense PascalCase)
**And** `SizeCategory` = `Small / Medium / Large` computed from mesh radius with thresholds in `TuningConfig`
**And** the asteroid is despawned
**And** yield calc (Story 6.11) fires

**Given** the player releases `HoldTractorBeam`
**When** the pull system observes no-hold
**Then** ExternalForce on the formerly-tractored asteroid is removed
**And** the asteroid keeps its current velocity (no snap-stop)
**And** the visual beam disappears
**And** the asteroid stays `RigidBody::Dynamic` (once-dynamic-always-dynamic policy — simpler)

### Story 6.11: Intact Capture vs Destroyed Yield Math

As a player,
I want intact-captured asteroids to yield strictly more than destroyed ones,
So that FR13's tactical incentive to tractor-capture is real.

**Acceptance Criteria:**

**Given** `TuningConfig` is extended
**When** yield constants are defined
**Then** six yields loaded (captured > destroyed invariant per size):
- `yield_captured_small = 5`, `yield_captured_medium = 15`, `yield_captured_large = 40`
- `yield_destroyed_small = 2`, `yield_destroyed_medium = 6`, `yield_destroyed_large = 15`

**And** a `#[cfg(test)]` unit test asserts `yield_captured_<size> > yield_destroyed_<size>` for every size

**Given** `AsteroidDestroyed` from Story 3.10 / 4.2
**When** Story 6.11 extends handling
**Then** the asteroid's SizeCategory is read from its mesh radius
**And** `add_salvage(current_run, yield_destroyed_<size>, SalvageSource::DestroyedAsteroid)` is called

**Given** `AsteroidCaptured` from Story 6.10
**When** handling runs
**Then** `add_salvage(current_run, yield_captured_<size>, SalvageSource::IntactCapture)` is called

**Given** the yield-delta indicator (FR25 / Epic 8)
**When** Story 6.11 exposes data
**Then** a `PotentialYield { destroy: u32, capture: u32 }` component is attached to each asteroid on spawn (Epic 8 renders it as world-space cockpit HUD)

**Given** `GameState::Arena` (tutorial)
**When** an asteroid is destroyed in Arena
**Then** NO salvage is added — `ShotEconomyActive` resource gates the salvage-add path (same gate as Story 6.9's shot-cost)

### Story 6.12: Boost + Rechargeable Resource

As a player,
I want a boost mechanic granting a short thrust burst off a rechargeable resource,
So that I have a mobility option per FR6.

**Acceptance Criteria:**

**Given** `FlightAction` from Story 3.6
**When** extended
**Then** `ActivateBoost` is added bound to Left Shift

**Given** PlayerShip is extended
**When** the boost component is added
**Then** `BoostResource { current: f32 = 1.0, max: f32 = 1.0, recharge_rate_per_sec: f32 = 0.2, active_duration_sec: f32 = 2.0, active_remaining_sec: f32 = 0.0 }` is on the PlayerShip
**And** defaults come from `TuningConfig.boost_*`

**Given** the player presses `ActivateBoost` while `current >= 1.0`
**When** the boost system observes in FixedUpdate
**Then** `active_remaining_sec = active_duration_sec`, `current = 0.0`
**And** a `BoostActivated { entity }` event is emitted

**Given** `active_remaining_sec > 0`
**When** Story 3.6's thrust system reads boost state
**Then** `ship_thrust_newtons` is multiplied by `TuningConfig.boost_thrust_multiplier: f32 = 3.0`
**And** `active_remaining_sec -= delta` each frame
**And** when `active_remaining_sec ≤ 0`, multiplier returns to 1.0

**Given** `active_remaining_sec == 0` AND `current < max`
**When** the recharge system runs
**Then** `current += recharge_rate_per_sec * delta`, clamped to `max`

**Given** the player presses `ActivateBoost` while `current < 1.0`
**When** activation is checked
**Then** the action is rejected (no event, no effect)

**Given** HUD needs a boost indicator per Till's decision 2026-04-22
**When** Story 6.12 renders UI
**Then** a small bar is rendered as a world-space element on the cockpit frame (not screen-space corner)
**And** fill reflects `current / max`, colored green when ready, amber dim while recharging or active

### Story 6.13: Difficulty Variant System — Easy / Medium / Hard

As a player,
I want three difficulty variants that meaningfully change the run experience,
So that I can pick a challenge level per FR31.

**Acceptance Criteria:**

**Given** `Difficulty` enum from Story 6.1
**When** Story 6.13 finalizes
**Then** `Easy / Medium / Hard` variants are the finalized three
**And** `PickedDifficulty` is read on OnEnter(Caravan) into `CurrentRun.difficulty`

**Given** `TuningConfig` is extended with per-difficulty multipliers
**When** Caravan systems read them
**Then**:
- **Easy**: `enemy_health_mult = 0.75`, `enemy_damage_mult = 0.75`, `enemy_fire_rate_mult = 0.75`, `pocket_enemy_count_mult = 0.75`, `yield_mult = 1.25`
- **Medium**: all multipliers = 1.0 (baseline = existing TuningConfig values)
- **Hard**: `enemy_health_mult = 1.5`, `enemy_damage_mult = 1.5`, `enemy_fire_rate_mult = 1.5`, `pocket_enemy_count_mult = 1.5`, `yield_mult = 0.75`

**Given** enemies spawn in Caravan
**When** Story 6.8's pocket-spawn runs
**Then** count is `(base_count * pocket_enemy_count_mult).floor()`, clamped to [1, spawn_positions.len()]

**Given** an enemy is spawned
**When** stats are applied
**Then** `Health.max` and `Health.current` are scaled by `enemy_health_mult`
**And** projectile damage dealt is scaled by `enemy_damage_mult`
**And** fire rate is scaled by `enemy_fire_rate_mult`

**Given** salvage yields
**When** yield is added via Story 6.11
**Then** final yield is `(base_yield * yield_mult).floor()`

**Given** the three variants
**When** a full Caravan is played at each difficulty
**Then** runs feel tangibly different; defaults above are starting-point tuning to be refined via playtest at M5 gate

<!-- Epic 6 complete — 13 stories deliver M5 Caravan Run Framework (Danger Stretch). Next epic to decompose: Epic 7 (Roguelite Loop / M6 EA-Viable). -->

## Epic 7: Roguelite Loop (EA-Viable)

Meta-currency from runs spendable in an unlock shop for 8 permanent upgrades. "One more run" retention loop closed. Intel x86_64 macOS binary added alongside arm64 (universal, still unsigned — FR48 further deferred to E10 per Till 2026-04-22). Commercially viable as Itch.io release or Steam EA if pursued. M-alignment: M6 🏁. FRs covered: FR18, FR19, FR20, FR21, FR47 completion. NFR covered: NFR-R4.

### Story 7.1: Meta-Currency Display on MainMenu + PostRun `banked` Real Wiring

As a player,
I want to see my lifetime meta-currency on MainMenu and per-run banked amount on PostRun,
So that FR19 persistent progression is visible and Story 4.9's PostRun placeholder becomes real.

**Acceptance Criteria:**

**Given** SaveData.meta_currency exists from Story 4.6 (banked by Story 6.7)
**When** MainMenu renders (Story 4.7)
**Then** a "META: <meta_currency>" text Node is added to MainMenu layout (top-right or above title)
**And** the text updates reactively when SaveData.meta_currency changes

**Given** Story 4.9's PostRun shows "Salvage banked: 0" placeholder
**When** Story 7.1 wires the real values
**Then** PostRun summary reads:
- "Salvage banked this run: `<RunResult.salvage_banked>`"
- "Meta-currency total: `<SaveData.meta_currency>`"

**Given** Epic 6 Story 6.7 banks salvage
**When** Story 7.1 verifies the cross-story contract
**Then** `RunResult.salvage_banked` is populated on ALL run-end paths (TargetReached, HullDepleted, and Aborted with salvage_banked=0 for forfeit)

**Given** first launch (no runs completed)
**When** MainMenu is shown
**Then** display reads "META: 0" (always visible for consistency)

### Story 7.2: Unlock Definition Data Model + 8-Unlock Catalog

As a developer,
I want a data model for unlocks and a catalog of 8 definitions with exponential stacking costs,
So that FR21 has concrete purchasable content.

**Acceptance Criteria:**

**Given** `src/meta/unlocks.rs` is authored
**When** the data model is defined
**Then**:
- `UnlockDefinition { id: String, display_name: String, description: String, effect: UnlockEffect, stackable_max: u32, base_cost: u32 }`
- `UnlockEffect` enum: `HullMaxDelta(i32)`, `ShieldMaxDelta(i32)`, `ThrustMult(f32)`, `DetectionRangeMult(f32)`, `BoostRechargeMult(f32)`, `TractorRangeDelta(f32)`, `ShotCostMult(f32)`, `YieldCapturedMult(f32)`
- `stackable_max=1` = non-stackable; higher = cap

**Given** `src/meta/catalog.rs` is authored
**When** the catalog is defined
**Then** these 8 entries exist:
- `hull_plating` — `HullMaxDelta(1)` — max=3 — base=100
- `shield_capacitor` — `ShieldMaxDelta(2)` — max=3 — base=150
- `thruster_tuning` — `ThrustMult(1.2)` — max=1 — base=200
- `sensor_range` — `DetectionRangeMult(1.5)` — max=1 — base=250 (consumed by Epic 8 radar)
- `boost_recharge` — `BoostRechargeMult(1.5)` — max=1 — base=200
- `tractor_reach` — `TractorRangeDelta(20.0)` — max=2 — base=300
- `weapon_efficiency` — `ShotCostMult(0.8)` — max=1 — base=350
- `salvage_refinery` — `YieldCapturedMult(1.1)` — max=1 — base=400

**Given** exponential 1.5× stacking per Till's decision
**When** `cost_for_next_stack(&unlock_def, current_stacks: u32) -> u32` is called
**Then** returns `(base_cost as f32 * 1.5_f32.powi(current_stacks as i32)).round() as u32`
**And** returns `Err(MaxStacked)` when `current_stacks >= stackable_max`

**Given** SaveData.unlocked_upgrades was `Vec<String>` from Story 4.6
**When** Story 7.2 refactors representation
**Then** it becomes `HashMap<String, u32>` (id → stack_count) for cleaner stacking
**And** a `v2_to_v3` migration via Story 5.6's scaffold converts existing `Vec<String>` by counting duplicates
**And** SaveData.version bumps to 3

### Story 7.3: UnlockShop UI State + Access from MainMenu + PostRun

As a player,
I want to visit the UnlockShop from MainMenu or PostRun,
So that FR20 spend-between-runs and immediate-retention loops both work.

**Acceptance Criteria:**

**Given** `GameState` enum
**When** extended
**Then** `UnlockShop` variant is added

**Given** Story 4.7 MainMenu and Story 4.9 PostRun
**When** Story 7.3 extends both layouts
**Then** each adds a "Shop" button (MainMenu: placed between Settings and Credits; PostRun: alongside Retry and Main Menu)
**And** clicking either sets `NextState<GameState>` = `UnlockShop`
**And** a `ShopReturnTo(GameState)` resource is set to the originating state on entry

**Given** `src/ui/shop.rs` is authored
**When** OnEnter(UnlockShop) runs
**Then** UI shows:
- Header: "META: `<meta_currency>`" (reactive)
- Scrollable list of all 8 catalog entries
- For each entry: display_name, description, `current_stacks / stackable_max`, next_cost (or "MAX"), "Buy" button (disabled if insufficient funds or at max)
- "Back" button routes via `ShopReturnTo`

**And** all spawned entities carry a `ShopEntity` marker

**Given** OnExit(UnlockShop)
**When** cleanup runs
**Then** `ShopEntity`-marked entities are despawned
**And** `ShopReturnTo` resource is removed

### Story 7.4: Purchase Flow — Validate + Deduct + Event + Save

As a player,
I want "Buy" to validate funds, deduct cost, apply the stack, and save,
So that FR20 transactions are atomic and persistent.

**Acceptance Criteria:**

**Given** the shop UI from Story 7.3
**When** the player clicks "Buy" for an unlock at cost C
**Then** the purchase system:
- Validates `SaveData.meta_currency >= C` (else abort silently, subtle cost-text flash)
- Validates `current_stacks < stackable_max` (else abort)
- Deducts `SaveData.meta_currency -= C`
- Increments stack count in `SaveData.unlocked_upgrades` HashMap
- Emits `UnlockPurchased { id: String, stack_after: u32, cost_paid: u32 }` event
- Calls `save(&save_data)`

**Given** the save write succeeds
**When** UI re-renders
**Then** META header updates reactively
**And** the purchased unlock's stack count and next cost update
**And** input debounce (100 ms cooldown or `Interaction::Pressed`-once) prevents double-purchase

**Given** the save write fails
**When** the purchase system detects the error
**Then** in-memory SaveData changes are rolled back (currency and stack reverted)
**And** an inline error "Purchase failed: could not save. Try again." is shown
**And** no `UnlockPurchased` event is emitted

**Given** the `UnlockPurchased` event is emitted
**When** Story 7.5's system observes
**Then** downstream effects-wiring fires per Story 7.5

### Story 7.5: Unlock Effects Wiring — Runtime TuningConfig Overlay

As a player,
I want my purchased unlocks to actually improve my ship in the next run,
So that FR21 upgrades have tangible gameplay effect.

**Acceptance Criteria:**

**Given** SaveData.unlocked_upgrades (HashMap<String, u32>) from Story 7.2
**When** OnEnter(Caravan) OR OnEnter(Arena) fires
**Then** an `apply_unlock_effects` system runs that builds a `RuntimeTuning` resource from `TuningConfig` + unlock overlays:
- For each `(id, stacks)`, look up effect, apply N times:
  - `HullMaxDelta(d)` × N → `effective_player_hull_max = base + d*N`
  - `ShieldMaxDelta(d)` × N → similarly
  - `ThrustMult(m)` → `effective_ship_thrust_newtons = base * m^N`
  - `DetectionRangeMult(m)` → `effective_enemy_detection_range = base * m^N`
  - `BoostRechargeMult(m)` → `effective_boost_recharge_rate = base * m^N`
  - `TractorRangeDelta(d)` → `effective_tractor_range = base + d*N`
  - `ShotCostMult(m)` → `effective_shot_cost = max(1, (base as f32 * m.powi(N)).floor() as u32)`
  - `YieldCapturedMult(m)` → `effective_yield_captured_<size> = (base as f32 * m.powi(N)).floor() as u32`

**Given** gameplay systems previously read `TuningConfig` directly
**When** Story 7.5 is merged
**Then** all ship/enemy/tractor/shot/yield consumers are updated to read from `RuntimeTuning` instead
**And** `RuntimeTuning` rebuilds on each state-entry (newly-purchased unlocks take effect next run, not mid-run)

**Given** no unlocks purchased
**When** `RuntimeTuning` is built
**Then** it exactly matches `TuningConfig` (identity operation — regression safety)

**Given** the Retry loop (PostRun → Shop → Back → Retry)
**When** a new Caravan run starts
**Then** the newly-purchased unlock is reflected (verified via playtest)

**Given** `hull_plating` purchased 3 times (100 + 150 + 225 = 475 salvage)
**When** next run starts
**Then** `effective_player_hull_max = 3 + 1*3 = 6`
**And** 4th purchase attempt is rejected (at stackable_max=3)

### Story 7.6: macOS Universal Binary — Intel x86_64 + arm64

As a player on an Intel Mac,
I want a native-speed macOS binary,
So that FR47's Apple Silicon + Intel x86_64 commitment is fulfilled without emulation.

**Acceptance Criteria:**

**Given** Epic 4 Story 4.10's release workflow produces `macos-arm64` only
**When** Story 7.6 extends the macOS job
**Then** the workflow adds `rustup target add x86_64-apple-darwin` + a second `cargo build --release --target x86_64-apple-darwin` step

**Given** both architectures are built
**When** the universal-binary step runs
**Then** `lipo -create -output asteroids3D target/release/asteroids3D target/x86_64-apple-darwin/release/asteroids3D` produces a combined Mach-O
**And** `lipo -info asteroids3D` reports both architectures in the CI log
**And** a CI check asserts both slices are present

**Given** the universal binary is staged
**When** packaging runs
**Then** the ZIP is renamed to `asteroids3D-macos-universal-<version>.zip` (replaces prior `macos-arm64` ZIP)

**Given** both Intel and arm64 Mac users download the ZIP
**When** they launch
**Then** each CPU runs its native slice at native performance
**And** no Rosetta emulation occurs on either architecture (verified via Activity Monitor or `file` output check)

**Given** the binary is still unsigned per Till's decision (FR48 deferred E7 → E10 / M9 Polish)
**When** users run it
**Then** the right-click-Open Gatekeeper workaround from Story 4.10's runbook still applies
**And** the runbook is updated: "universal binary, unsigned; signing is Epic 10 / M9"

<!-- Epic 7 complete — 6 stories deliver M6 Roguelite Loop + macOS universal binary. FR48 further deferred to Epic 10. Next epic to decompose: Epic 8 (Perception — Sensors & Spatial Audio / M7). -->

## Epic 8: Perception — Sensors & Spatial Audio

Player perceives unseen threats via cockpit radar and spatial stereo audio cues. Yield-delta indicator on tractorable salvage. First-launch headphone recommendation. Plus: damage-direction indicator (visual + audio). Minimal gameplay SFX included per Till's decision 2026-04-22 (weapon, impact, UI). M-alignment: M7. FRs covered: FR22, FR23, FR25, FR26. NFRs covered: NFR-A1, NFR-A2.

### Story 8.1: PerceptionPlugin + Threat Detection Events

As a developer,
I want a PerceptionPlugin emitting structured detection events when entities enter sensor range,
So that Radar + Audio stories consume a clean event stream instead of querying entity states directly.

**Acceptance Criteria:**

**Given** `src/perception/mod.rs` with `PerceptionPlugin`
**When** added to main.rs
**Then** it declares `PerceptionSystems` SystemSet

**Given** TuningConfig is extended
**When** `player_sensor_range: f32 = 150.0` is added
**Then** Story 7.5's Effects-Wiring update is extended so `effective_player_sensor_range = base * m^N` for `sensor_range` unlock stacks (fixes Epic 7 cross-story wiring — the `DetectionRangeMult` unlock now multiplies player sensor range, not enemy AI detection)

**Given** `src/perception/events.rs` defines detection events
**When** registered with the app
**Then** past-tense PascalCase events exist:
- `EnemyDetected { enemy: Entity, direction_from_player: Vec3, distance: f32 }` — edge-triggered on entry
- `EnemyLost { enemy: Entity }` — edge-triggered on exit
- `SalvageOfInterestDetected { asteroid, direction_from_player, distance }` — Large-size salvage only
- `SalvageOfInterestLost { asteroid }`
- `HazardDetected { hazard, .. }` — **scaffolded, never emitted in MVP** (Post-MVP placeholder)

**Given** the perception scan runs in FixedUpdate within `PerceptionSystems`
**When** it iterates each frame
**Then** for each Enemy within `effective_player_sensor_range`:
- Newly-detected → emit `EnemyDetected`, add to `TrackedEnemies: HashSet<Entity>` resource
- Still detected → no event
- Newly-lost → emit `EnemyLost`, remove from tracking

**And** same edge-triggered logic for salvage-of-interest (Large asteroids only, per Story 6.10 SizeCategory)

**Given** state entry/exit
**When** Arena or Caravan is entered
**Then** `TrackedEnemies` and `TrackedSalvage` resources initialize empty
**And** on state exit, they clear (no stale references across runs)

### Story 8.2: AudioPlugin + bevy_kira_audio Spatial Channel Setup

As a developer,
I want AudioPlugin with per-purpose bevy_kira_audio channels,
So that Stories 8.4/8.5 and future Epic 10 audio have clean routing targets.

**Acceptance Criteria:**

**Given** `src/audio/mod.rs` with `AudioPlugin`
**When** added
**Then** `bevy_kira_audio::AudioPlugin` is registered as a dependency
**And** the custom AudioPlugin declares `AudioSystems` SystemSet

**Given** per-purpose channels via `AudioChannel<T>` marker types
**When** inspected
**Then** these channels exist:
- `SfxChannel` — weapon/impact/hit/UI (populated in MVP)
- `AlertChannel` — threat/detection cues (populated in MVP)
- `AmbientChannel` — **scaffolded, empty in MVP** (Epic 10 Polish)
- `MusicChannel` — **scaffolded, empty in MVP** (Epic 10 Polish)

**Given** Story 4.8's volume settings
**When** sliders change
**Then** `master_volume` routes to bevy_kira_audio master
**And** `sfx_volume` applies to SfxChannel + AlertChannel (the two MVP-active channels)
**And** Story 4.8's sfx_volume wiring (implementation-deferred from Epic 4) is concretized here to target these channels

**Given** spatial audio per FR23
**When** a spatial sound plays
**Then** it uses bevy_kira_audio's `SpatialAudioEmitter` bundle at a world position
**And** an `AudioListener` is attached to CockpitCamera (Story 3.5)

**Given** user runs without headphones
**When** audio plays
**Then** the stereo speaker fallback renders without crashes (bevy_kira_audio spatial mix graceful degradation — verified via trivial spatial test source on state entry)

### Story 8.3: World-Space Radar Mesh on Cockpit

As a player,
I want a 2D planar radar disc on my cockpit frame showing threats in sensor range,
So that I perceive unseen enemies per FR22 with scientific-instrument styling.

**Acceptance Criteria:**

**Given** `src/ui/radar.rs` integrated into `HudPlugin`
**When** OnEnter(Caravan) OR OnEnter(Arena) fires
**Then** a `RadarDisc` entity is spawned as a child of the cockpit camera
**And** it uses a circular 2D disc mesh (placeholder, final polish Epic 10) with ToonMaterial or flat material
**And** it is positioned on the cockpit frame at the top-center dashboard below the windshield (scientific-instrument styling)
**And** it carries the appropriate state marker (ArenaEntity or CaravanEntity)

**Given** the radar disc exists
**When** the radar-update system runs each frame
**Then** for each entity in `TrackedEnemies` (Story 8.1):
- Compute enemy position relative to PlayerShip in ship-local space
- Project onto the XZ plane (top-down) of the disc
- Normalize distance to [0, 1] as `distance / effective_player_sensor_range`
- Spawn or update a threat-pip at the mapped 2D position, colored red (`SemanticAccent::Enemy`)

**And** for each entity in `TrackedSalvage`, pip is yellow (`SemanticAccent::Salvage`)

**Given** `EnemyLost` event fires
**When** the radar system processes
**Then** the corresponding pip is despawned

**Given** the radar disc
**When** observed
**Then** the player can distinguish enemy position by pip location at a glance (NFR-A1 playtest-validated: 2D spatial layout + color redundancy)
**And** entities within line-of-sight and camera frustum are still shown on radar (simpler model; no always-visible-versus-radar partitioning)

**Given** the player has no `sensor_range` unlock
**When** radar evaluates
**Then** `effective_player_sensor_range = 150.0`
**And** enemies beyond 150 m are not on the radar

**Given** `sensor_range` unlock purchased (Story 7.5 applied)
**When** radar evaluates
**Then** `effective_player_sensor_range = 150.0 * 1.5 = 225.0`
**And** previously-hidden enemies (150–225 m) now appear — visible progression demo

### Story 8.4: Audio Cues for Threats + Salvage + Minimal Gameplay SFX

As a player,
I want audio feedback for detection and core gameplay actions,
So that I perceive hidden entities per FR23 and the game feels alive (per Till's decision 2026-04-22 to include minimal gameplay SFX in MVP).

**Acceptance Criteria:**

**Given** AudioPlugin from Story 8.2
**When** Story 8.4 adds cues
**Then** an `AudioCueCatalog` resource loads on app start with typed handles for:
- `cue_enemy_detected` (AlertChannel, spatial) — short warning chirp
- `cue_salvage_detected` (AlertChannel, spatial) — resonant chime
- `sfx_weapon_pulse_fire` (SfxChannel, non-spatial) — placeholder pulse tone
- `sfx_weapon_shotgun_fire` (SfxChannel, non-spatial) — placeholder noise burst
- `sfx_weapon_railgun_fire` (SfxChannel, non-spatial) — placeholder sharp pew
- `sfx_projectile_impact_asteroid` (SfxChannel, spatial) — placeholder thud
- `sfx_projectile_impact_enemy` (SfxChannel, spatial) — placeholder metallic impact
- `sfx_ui_click` (SfxChannel, non-spatial) — placeholder beep

**And** all audio files are placeholder-quality (royalty-free or synthesized); final pass Epic 10

**Given** `EnemyDetected` event (edge-triggered by Story 8.1)
**When** the audio system observes
**Then** `cue_enemy_detected` plays as a spatial source at the enemy's world position (stereo direction reflects enemy location)
**And** the cue does NOT repeat while the same enemy stays tracked

**Given** `SalvageOfInterestDetected` event
**When** observed
**Then** `cue_salvage_detected` plays spatially at the asteroid position

**Given** a weapon fires (existing `WeaponFired`-style event from Story 3.9 / 4.4 — or equivalent archetype-aware trigger)
**When** the audio system observes
**Then** the appropriate `sfx_weapon_<archetype>_fire` plays non-spatially on SfxChannel

**Given** a projectile hits an asteroid or enemy (events from Stories 3.10, 4.2, 4.3)
**When** observed
**Then** `sfx_projectile_impact_<target>` plays spatially at the hit location

**Given** UI button clicks (MainMenu, Settings, Shop, PostRun)
**When** Interaction::Pressed fires
**Then** `sfx_ui_click` plays non-spatially

**Given** Story 4.8's `sfx_volume`
**When** the player adjusts it
**Then** SfxChannel + AlertChannel attenuate proportionally in real-time

### Story 8.5: Damage-Direction Indicator (Visual + Audio)

As a player,
I want to know which direction incoming damage came from,
So that I can orient to off-screen threats per the E5 scope-addition.

**Acceptance Criteria:**

**Given** Epic 5 Story 5.3's `DamageApplied` event
**When** Story 8.5 extends it
**Then** the event is augmented with `damage_origin: Option<Vec3>` (world-position of the projectile at impact, or None if unknown)
**And** Story 5.3's damage-routing system populates `damage_origin` from the hitting projectile's Transform

**Given** `DamageApplied { damage_origin: Some(origin), .. }` on PlayerShip
**When** the damage-direction UI system runs
**Then** a red arrow edge-indicator renders on the screen edge closest to the screen-space projection of `origin - player_position`
**And** the indicator pulses for `TuningConfig.damage_indicator_duration_sec: f32 = 1.5` then fades

**Given** multiple DamageApplied events in the same frame from different origins
**When** processed
**Then** multiple concurrent edge indicators render
**And** indicators mapping to the same edge region visually merge into one brighter arrow

**Given** `damage_origin: None`
**When** the system runs
**Then** no directional indicator shows (the existing HUD hull/shield updates already convey "you took damage")

**Given** the audio component
**When** DamageApplied fires
**Then** a spatial `sfx_damage_hit_stinger` plays at `damage_origin` (if present)
**And** if `damage_origin: None`, non-spatial fallback plays
**And** the stinger is distinct from `sfx_projectile_impact_enemy` — this is the "I-was-hit" tense stinger

**Given** Damage-direction UI uses the palette
**When** it renders
**Then** arrow color is `SemanticAccent::Hazard` (orange-red)
**And** arrow shape is placeholder chevron; final visual polish Epic 10

### Story 8.6: World-Space Yield-Delta Indicator on Tractorable Salvage

As a player,
I want to see the yield difference between destroying vs tractor-capturing an asteroid,
So that I make informed tactical choices per FR25 right in my cockpit view.

**Acceptance Criteria:**

**Given** Epic 6 Story 6.11's `PotentialYield { destroy: u32, capture: u32 }` on each asteroid
**When** Story 8.6 adds the indicator system in OnEnter(Caravan)
**Then** a query runs each frame to identify asteroids that are:
- Within `effective_tractor_range` (60 m baseline, extensible via `tractor_reach` unlock)
- Inside the cockpit camera's frustum

**Given** a qualifying asteroid is found
**When** the yield-delta UI renders
**Then** a world-space text element is spawned near the asteroid at an anchor position (10 m above its center) showing `+<capture - destroy>` (e.g., "+3")
**And** the text is colored yellow (`SemanticAccent::Salvage`) to signal "bonus if captured"
**And** the text entity carries a `YieldDeltaIndicator { target_asteroid: Entity }` component
**And** the text is billboarded to camera and scaled by distance for consistent readability

**Given** `PotentialYield.capture > PotentialYield.destroy` always (Story 6.11 invariant)
**When** the indicator renders
**Then** it always shows a positive delta (never shows 0 or negative)

**Given** an asteroid leaves the qualifying zone
**When** the system checks
**Then** its indicator is despawned

**Given** 5 qualifying asteroids are visible simultaneously
**When** UI renders
**Then** all 5 indicators are visible without unreadable clutter

**Given** `GameState::Arena` (tutorial)
**When** the system checks active state
**Then** yield-delta indicators are DISABLED (no economy in Arena)

### Story 8.7: First-Launch Headphone Recommendation Splash

As a player on first launch,
I want a brief splash recommending headphones for optimal spatial audio,
So that FR26 sets audio expectations before I enter the game.

**Acceptance Criteria:**

**Given** SaveData from Stories 4.6 / 5.6 / 7.2
**When** Story 8.7 extends it
**Then** `headphone_splash_shown: bool` (default `false`) is added
**And** SaveData.version bumps to v4 with a migration injecting `false` via Story 5.6's scaffold

**Given** the app is in GameState::Loading and the save has loaded
**When** the launch-flow system routes
**Then** if `headphone_splash_shown == false` → route through a new transient `GameState::HeadphoneSplash` before MainMenu
**And** if `true` → route directly to MainMenu (no regression on repeat launches)

**Given** `GameState` enum
**When** extended
**Then** `HeadphoneSplash` variant is added

**Given** OnEnter(GameState::HeadphoneSplash)
**When** the splash UI builds
**Then** a centered bevy_ui Node shows:
- Title: "🎧 Recommended: headphones" (emoji or plain text)
- Subtitle: "asteroids3D uses spatial audio cues for hidden threats"
- "OK / Continue" button

**And** entities carry `HeadphoneSplashEntity` marker

**Given** the player clicks "OK / Continue"
**When** the action fires
**Then** `SaveData.headphone_splash_shown = true`
**And** `save(&save_data)` persists
**And** `NextState<GameState>` = `MainMenu`

**Given** OnExit(GameState::HeadphoneSplash)
**When** cleanup runs
**Then** `HeadphoneSplashEntity`-marked entities are despawned

**Given** future escape-hatch need
**When** `headphone_splash_shown = false` is manually set in save
**Then** the next launch triggers the splash again (simple reset mechanism)

<!-- Epic 8 complete — 7 stories deliver M7 Perception (sensors + spatial audio). Cross-story fixes: Epic 7 sensor-range wiring, Story 4.8 sfx-volume concretization, Story 5.3 DamageApplied.damage_origin extension, SaveData v3→v4. Next epic to decompose: Epic 9 (Post-Run Photo Mode / M8). -->

## Epic 9: Post-Run Photo Mode

From post-run / death screen, player enters Photo Mode with free-cam orbital/dolly + click-to-focus DoF + time-frozen scene. Exports PNG screenshots in 16:9 landscape, 9:16 portrait, or 1:1 square. Marketing-ready aesthetic artifact pipeline. M-alignment: M8. FRs covered: FR40, FR41, FR42.

### Story 9.1: FreeOrbitCamera Component + F3 Dev Toggle

As a developer,
I want a `FreeOrbitCamera` component with orbit/dolly/pan math + an F3 debug toggle gated to `cfg(debug_assertions)`,
So that Photo Mode and dev-time gameplay debugging share one camera implementation without bleeding into release.

**Acceptance Criteria:**

**Given** `src/camera/free_orbit.rs` is authored
**When** `FreeOrbitCamera` component is defined
**Then** fields: `anchor_point: Vec3`, `distance: f32`, `yaw: f32`, `pitch: f32`, `pan_offset: Vec3`
**And** a system computes Transform from these fields each frame (spherical coords around anchor + pan offset)

**Given** `cfg(debug_assertions)` (dev build)
**When** `DebugCameraPlugin` is registered
**Then** F3 toggles between CockpitCamera (gameplay) and a dev FreeOrbitCamera
**And** on enable, FreeOrbitCamera spawns with `anchor_point = PlayerShip position`, `distance = 20.0`, `yaw = 0.0`, `pitch = -0.3`
**And** on disable, the dev camera despawns and CockpitCamera resumes

**Given** release build (`cfg(not(debug_assertions))`)
**When** compiled
**Then** F3 does nothing — `DebugCameraPlugin` is not registered
**And** no 3rd-person camera is accessible in release (FR8 cockpit-only enforcement intact)

**Given** FreeOrbitCamera control mappings (shared between dev F3 and PhotoMode Story 9.3)
**When** the control system runs
**Then**:
- Left mouse drag → yaw/pitch around anchor
- Mouse wheel → distance (dolly)
- WASD → pan_offset translation in camera-local axes
- Space / LCtrl → pan_offset up/down

**Given** both gates (F3 dev + PhotoMode)
**When** either activates
**Then** they reuse the same component + control system (one impl, two gates)

### Story 9.2: PhotoMode State + Entry/Exit from PostRun + Time-Freeze + Overlay

As a player,
I want a PhotoMode state I enter from PostRun, with time frozen, a "PHOTO MODE" badge, and a controls hint,
So that FR40's post-run-only access is real and the workflow is discoverable.

**Acceptance Criteria:**

**Given** `GameState` enum from Story 1.6
**When** extended
**Then** `PhotoMode` variant is fully realized (was a placeholder in Story 1.6)

**Given** Epic 4 Story 4.9's PostRun layout
**When** Story 9.2 extends it
**Then** a "Photo Mode" button is added alongside Retry / Main Menu / Shop
**And** clicking it sets `NextState<GameState>` = `PhotoMode`

**Given** FR40 constraint (post-run only, not during gameplay)
**When** the constraint is audited
**Then** NO keybinding or UI affordance routes to PhotoMode from Arena or Caravan
**And** a `#[cfg(debug_assertions)]` assertion can verify this invariant on state-entry

**Given** OnEnter(GameState::PhotoMode)
**When** state entry runs
**Then** a FreeOrbitCamera spawns with `anchor_point = PlayerShip position, distance = 20.0, yaw = 0.0, pitch = -0.3` (death-pose per Till's decision)
**And** CockpitCamera (Story 3.5) is disabled (`Camera::active = false`)
**And** time is frozen per Story 3.4's pause mechanism (reused)
**And** all audio is muted (SfxChannel + AlertChannel silenced per Till's decision)

**Given** the PhotoMode overlay UI
**When** it renders on entry
**Then** a `bevy_ui` Node shows:
- "PHOTO MODE" badge (top-left corner, small semi-transparent text)
- Controls hint: "Drag = rotate · Wheel = zoom · WASD = pan · F = focus · E = export · Esc = back"

**And** entities carry `PhotoModeEntity` marker

**Given** the player presses Esc in PhotoMode
**When** the exit system runs
**Then** `NextState<GameState>` = `PostRun` (returns, allowing further Retry/Menu/Shop)

**Given** OnExit(GameState::PhotoMode)
**When** cleanup runs
**Then** FreeOrbitCamera is despawned
**And** CockpitCamera re-enables
**And** time-freeze lifts (inverse of Story 3.4 pause)
**And** audio channels resume
**And** `PhotoModeEntity`-marked entities are despawned

**Given** the player re-enters PhotoMode from PostRun multiple times per Till's decision
**When** each entry happens
**Then** a fresh FreeOrbitCamera spawns each time (no stale cross-entry state)
**And** multiple exports (Story 9.5) within a single entry are supported without forced exit

### Story 9.3: PhotoMode Free-Cam Orbital + Dolly Controls

As a player in PhotoMode,
I want to orbit, zoom, and pan the camera freely,
So that I can frame any angle per FR41.

**Acceptance Criteria:**

**Given** Story 9.1's FreeOrbitCamera controls exist
**When** Story 9.3 scopes them to PhotoMode
**Then** the control system is gated to `GameState == PhotoMode`
**And** input is consumed by the camera, not by any gameplay system (gameplay is time-frozen anyway)

**Given** standard orbital controls (shared mappings with dev F3)
**When** input arrives
**Then**:
- Left mouse drag → `yaw` + `pitch` (orbit around anchor)
- Mouse wheel → `distance` (clamped to `TuningConfig.photo_min_distance = 2.0`, `photo_max_distance = 200.0`)
- WASD → `pan_offset` translation in camera-local XZ
- Space / LCtrl → `pan_offset` translation in camera-local Y

**And** mouse sensitivity reuses `mouse_sensitivity` from TuningConfig for consistency with flight

**Given** pitch extremes
**When** the player drags pitch past ±89°
**Then** pitch is clamped to [-89°, +89°] (prevents gimbal lock / flip)

**Given** Tilt/Roll is Post-MVP per Till 2026-04-22
**When** input bindings are inspected
**Then** roll input is NOT bound in MVP — camera stays world-up-aligned
**And** a source comment notes "tilt/roll deferred to Post-MVP / Epic 10 Polish"

**Given** the player is in PhotoMode
**When** mouse movement occurs
**Then** cursor is NOT confined (visible + free) so the player can click UI buttons (focus, export, watermark, back)
**And** drag detection uses mouse-button-hold semantics (click-to-start-drag, release-to-end)

### Story 9.4: Depth-of-Field Post-Processing + Click-to-Focus

As a player,
I want adjustable depth-of-field with click-to-focus,
So that I can compose cinematic shots per FR41.

**Acceptance Criteria:**

**Given** a DoF post-processing node is added to the PhotoMode rendering pipeline
**When** PhotoMode is active
**Then** the node is enabled — either Bevy 0.18's built-in DoF (if available) or a custom Gaussian-blur-based node at `src/camera/photo_dof.rs`
**And** outside PhotoMode, DoF is disabled (zero rendering cost in gameplay)

**Given** DoF state
**When** `PhotoDofState { focus_distance: f32, bokeh_intensity: f32, enabled: bool }` resource is defined
**Then** OnEnter(PhotoMode) initializes it to `focus_distance = 20.0, bokeh_intensity = 0.5, enabled = true`

**Given** DoF UI controls are shown in the overlay
**When** the UI renders
**Then** two sliders:
- "Focus distance" (0.5–200.0 m) → updates `focus_distance`
- "Bokeh" (0.0–1.0) → updates `bokeh_intensity`

**And** a checkbox toggles `enabled` (completely off for sharp-focus shots)

**Given** click-to-focus per Till 2026-04-22
**When** the player presses `F` key
**Then** a raycast from camera through cursor position is performed
**And** if it hits any entity, `focus_distance` is set to the hit distance
**And** the slider UI updates reactively

**Given** DoF is actively blurring
**When** the frame renders
**Then** pixels at `focus_distance` ± small tolerance are sharp
**And** further pixels are progressively blurred proportional to `bokeh_intensity`
**And** no flicker / no crash; minor banding acceptable

**Given** PhotoMode exits
**When** DoF node deactivates
**Then** the resource remains in-memory (next PhotoMode entry starts with last-used values, soft state persistence within app session)
**And** gameplay rendering is unchanged

### Story 9.5: PNG Export — 16:9 / 9:16 / 1:1 Aspect Ratio Presets

As a player,
I want to export the current PhotoMode view as a PNG in stream-friendly aspect ratios,
So that FR42 is functional.

**Acceptance Criteria:**

**Given** `src/camera/photo_export.rs` is authored
**When** the export system is registered
**Then** three aspect-ratio presets at fixed resolutions per Till 2026-04-22:
- 16:9 landscape → 1920×1080
- 9:16 portrait → 1080×1920
- 1:1 square → 1080×1080

**(4K / variable-resolution deferred to Epic 10 Polish)**

**Given** PhotoMode overlay UI
**When** rendered
**Then** three export buttons are shown: "Export 16:9", "Export 9:16", "Export 1:1"
**And** the `E` keyboard shortcut triggers a quick 16:9 export (default) — user can click buttons for other ratios

**Given** the player triggers an export
**When** the export system runs
**Then** a render-to-texture pass renders the scene at the preset resolution (temporary camera + Bevy render-target)
**And** the PhotoMode overlay UI is excluded from capture (overlay hidden for the render frame, restored immediately after)
**And** the texture is encoded to PNG via the `image` crate (or Bevy's screenshot primitives if available in 0.18)

**Given** export file location per Till 2026-04-22
**When** the PNG is written
**Then** the file goes to:
- Unix/macOS: `~/Pictures/asteroids3D/screenshots/`
- Windows: `%USERPROFILE%\Pictures\asteroids3D\screenshots\`

**And** the filename is `asteroids3D-<YYYYMMDD>-<HHMMSS>-<ratio>.png` (e.g., `asteroids3D-20260815-143022-16x9.png`)
**And** the directory is created if absent
**And** `directories` crate (already a dependency from Story 4.6) resolves the Pictures dir per-OS

**Given** the export succeeds
**When** the UI is notified
**Then** a toast-style notification appears in the overlay: "Exported: <filename>" (visible 3 s)
**And** the player remains in PhotoMode (no state change) for further exports

**Given** the export fails (disk full, permission error)
**When** the error is caught
**Then** a `warn!` log is written
**And** a toast shows "Export failed: <reason>"
**And** PhotoMode persists without crash

**Given** multiple exports per PhotoMode session per Till 2026-04-22
**When** the player exports repeatedly
**Then** each creates a new PNG with a distinct timestamp
**And** no forced state-exit after export

### Story 9.6: Toggleable Watermark

As a player,
I want an optional "asteroids3D" watermark I can toggle on for credited screenshots,
So that exported PNGs can be identified as from this game.

**Acceptance Criteria:**

**Given** SaveData
**When** Story 9.6 extends it
**Then** `watermark_enabled: bool` (default `false`) is added
**And** SaveData.version bumps to v5 with a migration injecting `false` via Story 5.6's scaffold

**Given** PhotoMode overlay (Story 9.2)
**When** rendered
**Then** a checkbox "Watermark on export" is shown alongside DoF controls
**And** its value binds to `SaveData.watermark_enabled`
**And** toggling updates SaveData and calls `save(&save_data)` immediately (persists across sessions)

**Given** Story 9.5's export system
**When** rendering the final PNG
**Then** if `watermark_enabled == true`:
- Small "asteroids3D" text overlay in the bottom-right corner
- Text is semi-transparent (~70% opacity) with a subtle drop-shadow for readability on any background
- Neutral color (light gray / off-white), not palette-colored

**And** if `watermark_enabled == false`:
- No watermark — pure scene render

**Given** the player toggles the watermark multiple times and exports
**When** each export runs
**Then** each PNG reflects the watermark state at its export time
**And** previously-exported PNGs are not retroactively modified

**Given** watermark text rendering approach
**When** implementation is chosen
**Then** either bevy_ui into a render-to-texture step, OR overlay at image-encode step via the `image` crate — either acceptable; pragmatic choice at implementation time

<!-- Epic 9 complete — 6 stories deliver M8 Post-Run Photo Mode (PNG export pipeline). SaveData v4→v5 (watermark_enabled). Next epic to decompose: Epic 10 (Polish Pass & MVP Completion / M9). -->

## Epic 10: Polish Pass & MVP Completion

Balance tuning, profiling to 60 FPS, asset-load audit, audio pass-2 (SFX + ambient), UI polish (instrument-panel refinement), 3 additional unlock definitions, crash-fix backlog, string-table audit, shield-absorb VFX, optional macOS codesign (FR48 stretch), 4-journey playtest validation across Windows + macOS + Linux-CI. Full polished MVP ready for Itch.io / Steam release. M-alignment: M9 🏁. FRs covered: FR48 (optional stretch per Till 2026-04-22). NFRs: P1, P2, P3, P4, P5, R1, U1, A3, L1, L3.

### Story 10.1: Performance Profiling + 60 FPS Target on Reference Hardware

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

### Story 10.2: Asset-Load-at-State-Entry Audit + Typed Resource Wrappers

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

### Story 10.3: Load-Time Budget — Title ≤10s, Title→Gameplay ≤5s

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

### Story 10.4: String-Table Audit — No Hard-Coded Player-Facing Strings

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

### Story 10.5: HUD Legibility Audit — 60–80 cm @ 1080p

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

### Story 10.6: Shield-Absorb VFX (Toon-Style Flash)

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

### Story 10.7: Audio Pass-2 — SFX Mixing + Ambient Drone Bed

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

### Story 10.8: UI Polish — Scientific-Instrument Styling Refinement

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

### Story 10.9: 3 Additional Unlock Definitions

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

### Story 10.10: macOS Codesign + Notarization (Optional Stretch)

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

### Story 10.11: Crash-Fix Backlog from M6–M8 Playtesting

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

### Story 10.12: 4-Journey Playtest Validation — 3 Platforms, 60 FPS, Zero-Crash

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
**Then** asteroids3D is declared MVP-ready
**And** release.yml produces final ZIPs per Story 4.10 / 7.6
**And** Itch.io publication follows the runbook (updated unsigned-or-signed depending on Story 10.10 decision)

**Given** the playtest uncovers new bugs
**When** they occur
**Then** they route to Story 10.11's backlog
**And** Story 10.12 re-runs until all 4 journeys pass clean

### Story 10.13: Final Mesh Assets — Asteroid Variants + Cockpit Interior + Ship Silhouette + Waypoint Arrow

As the project author,
I want hand-authored glTF meshes replacing the icosphere asteroids, cuboid/silhouette ship-cockpit, and chevron waypoint-arrow placeholders carried through Epics 3–6,
So that M9 closes with the visual identity the toon-shader tech spike was always pointing toward, instead of shipping recognizable Bevy primitives in the polished MVP.

**Acceptance Criteria:**

**Given** the typed Resource wrappers from Story 10.2 (`AsteroidModels`, `CockpitMesh`, `ShipModel`)
**When** Blender source files are authored under `assets/source/blender/` (committed for reproducibility, excluded from release builds)
**Then** the following glTF outputs land in `assets/meshes/`:
- `assets/meshes/asteroids/{small,medium,large}.gltf` — three size variants matching Story 3.4's 3.0–12.0 m radius range
- `assets/meshes/ship/cockpit.gltf` — first-person cockpit interior with HUD-mount surfaces compatible with Story 10.5's HUD layout
- `assets/meshes/ship/exterior.gltf` — external ship silhouette visible in Photo Mode (Epic 9), FR8 cockpit-only respected in gameplay
- `assets/meshes/ui/waypoint_arrow.gltf` — replaces the chevron placeholder from Epic 6 (final mesh deferred there with explicit "Epic 10 polish" pointer)

**Given** the new glTF assets exist
**When** `OnEnter(GameState::Loading)` runs (per Story 10.2's centralized loading)
**Then** the typed wrappers hold `Handle<Scene>` to the final glTF meshes
**And** no `Mesh3d::from(Sphere/Cuboid/Icosphere)` literals remain in non-debug code paths (grep-verified in CI; debug-only reference scenes from Story 2.1 retain primitives by design)

**Given** the Epic 2 toon visual pipeline (M1 GO-toon decision per Story 2.6)
**When** the new meshes render in-game
**Then** each mesh carries `ToonMaterial` (or M1 fallback if Story 2.7 was triggered) plus `OutlineBundle` per the existing reference-scene pattern
**And** posterized banding and silhouette outlines remain visible at gameplay viewing distances

**Given** NFR-P1 (60 FPS sustained) baselined by Story 10.1
**When** profiling re-runs after asset import
**Then** per-asset triangle counts are documented in `docs/perf/mesh-budget.md`
**And** the 60 FPS budget on the GTX 1060 / RX 580 / M1 reference set holds with 30 simultaneous asteroids on screen
**And** any asset exceeding budget is decimated in Blender and re-exported (single-LOD only — LOD system is post-MVP)

**Given** the Blender→glTF pipeline established in tech decisions
**When** assets are exported
**Then** `docs/art/asset-pipeline.md` documents Blender version, export settings (Y-up, single-take animation if any, embedded textures only if used), and a reproducible export-runbook
**And** export settings disable `KHR_materials_unlit` so `ToonMaterial` overrides the base material

**Given** projectile placeholders from Story 3.x
**When** this story is reviewed
**Then** projectile sphere meshes are intentionally retained (visible <100 ms per shot; primitive geometry is sufficient — recorded as deliberate non-replacement in `docs/art/asset-pipeline.md`)

**Given** the asset replacement is complete
**When** Story 10.12 4-journey playtest validation runs
**Then** Story 10.13 has status `done` (hard sequencing dependency — final playtest validates the final-look game)
**And** if Story 10.5 (HUD legibility) or Story 10.8 (UI polish) completed before this story, those audits re-run against the final cockpit interior before Story 10.12

<!-- Epic 10 complete — 13 stories deliver M9 Polish + MVP Completion. Story 10.13 added 2026-05-04 per Till's "final assets ship in MVP" decision. All 10 Epics decomposed. Next: Step 4 final validation. -->

