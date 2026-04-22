# Requirements Inventory

## Functional Requirements

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

## NonFunctional Requirements

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

## Additional Requirements

**Starter Template & M0 Gate (Architecture Starter Template decision):**
- The Architecture specifies a **Hybrid Manual** starter (not greenfield blank, not community template). Epic 1 Story 1 MUST be plugin-compatibility verification, not gameplay code. Concretely:
  - `cargo new --bin asteriods3d`
  - Author `Cargo.toml` by hand with pinned versions: Bevy 0.18 (`default-features = false`, features `["3d", "png", "x11"]`), Avian 0.6 (via `avian3d`), `bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager`, `bevy_egui` (dev-only), plus `serde`, `serde_json`, `ron`, `thiserror`, `tracing`, `tracing-subscriber`, `directories`.
  - Verify all plugins have Bevy-0.18-compatible releases. If any lag, fork-and-maintain-inline path documented.
  - Commit `Cargo.lock`.
  - Borrow infrastructure (stripped) from NiklasEi `bevy_game_template`: `.github/workflows/ci.yml` (Windows+Linux+macOS matrix — strip iOS/Android/Web), `.gitignore`, `rustfmt.toml`, `clippy.toml`, `rust-toolchain.toml`.
- M0 completion criterion: `cargo run` opens a window showing "asteriods3D" splash on all three platforms, with CI passing.

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
- Save location via `directories` crate (Windows `%APPDATA%`, Linux `$XDG_DATA_HOME`, macOS `~/Library/Application Support/asteriods3D/`).

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

## UX Design Requirements

N/A — No UX Design document exists for this project. UX guidance is embedded in:
- PRD *Design Philosophy* (5 design principles: no tutorial text; no visible numeric score; predictable asteroid motion; death-as-feedback; graceful degradation)
- PRD *User Journeys* (4 journey arcs with implicit acceptance criteria)
- Architecture *Rendering & Visual Architecture* (hybrid bevy_ui + world-space cockpit HUD, scientific-instrument-panel styling)
- Architecture *UI, Menu & Debug Architecture* (menu system, HUD rendering strategy)

Confirmed with Till on 2026-04-22: no separate UX-DR extraction required.

## FR Coverage Map

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
