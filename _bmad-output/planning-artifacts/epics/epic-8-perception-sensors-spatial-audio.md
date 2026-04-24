# Epic 8: Perception — Sensors & Spatial Audio

Player perceives unseen threats via cockpit radar and spatial stereo audio cues. Yield-delta indicator on tractorable salvage. First-launch headphone recommendation. Plus: damage-direction indicator (visual + audio). Minimal gameplay SFX included per Till's decision 2026-04-22 (weapon, impact, UI). M-alignment: M7. FRs covered: FR22, FR23, FR25, FR26. NFRs covered: NFR-A1, NFR-A2.

## Story 8.1: PerceptionPlugin + Threat Detection Events

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

## Story 8.2: AudioPlugin + bevy_kira_audio Spatial Channel Setup

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

## Story 8.3: World-Space Radar Mesh on Cockpit

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

## Story 8.4: Audio Cues for Threats + Salvage + Minimal Gameplay SFX

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

## Story 8.5: Damage-Direction Indicator (Visual + Audio)

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

## Story 8.6: World-Space Yield-Delta Indicator on Tractorable Salvage

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

## Story 8.7: First-Launch Headphone Recommendation Splash

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
