# M1 Toon Shader — Tech Spike Notes (Story 2.3)

Captured 2026-04-28 against `cargo run` (debug build, macOS / Apple M5 Pro / Metal backend).

## Rendered tints (per AC #4)

The reference scene materializes three placeholders with `ToonMaterial`; per-entity `SemanticAccent` is attached so future spawn-systems can read the component (Story 4.5).

| Placeholder | `SemanticAccent` | Hex tint (Wong 2011) |
|---|---|---|
| Asteroid (icosphere, left) | `Hazard` | `#F0E442` (yellow) |
| Ship-cockpit (cuboid, center) | `PlayerOwned` | `#56B4E9` (sky-blue) |
| Projectile (small sphere, right) | `Salvage` | `#009E73` (bluish-green) |

The projectile's `Salvage` tint is arbitrary at this stage; Story 4.5 will re-tag projectiles with their faction semantic.

## Posterization (AC #5)

Verified via `cargo run` + live edits to `assets/config/tuning.ron`. The hot-reload pipeline emits `TuningReloaded` log lines; per-edit log evidence in `/tmp/story-2-3-run.log`:

```
2026-04-28T15:14:28Z TuningReloaded: toon_steps=4 rim_power=2 rim_intensity=0.3   # cold-start (Default)
2026-04-28T15:14:40Z TuningReloaded: toon_steps=8 rim_power=2 rim_intensity=0.3   # edit 1
2026-04-28T15:14:50Z TuningReloaded: toon_steps=3 rim_power=2 rim_intensity=0.3   # edit 2
2026-04-28T15:14:57Z TuningReloaded: toon_steps=3 rim_power=2 rim_intensity=0.7   # edit 3 (rim bump)
2026-04-28T15:15:07Z TuningReloaded: toon_steps=4 rim_power=2 rim_intensity=0.3   # restore
```

Hot-reload latency: ~1–2s save-to-event (file-watcher poll cadence). Comfortable for iterative tweaking.

## Qualitative checks (manual, screenshot-anchored)

These visual claims are anchored by the three PNGs in this directory:

- **Posterized banding visible at default `steps: 4`** → see `toon-baseline.png`.
- **Bands count matches uniform value within ±1** → `toon-steps-3.png` shows ~3 bands; `toon-steps-8.png` shows ~8 bands.
- **Rim-light visible at grazing angles on the asteroid silhouette** → see `toon-baseline.png` (asteroid edge brightening).
- **Per-entity `SemanticAccent` tints render correctly** → all three placeholders show their assigned hex above.

## Backend caveats

Validated only on macOS / Metal in this story. Cross-backend parity (Vulkan / DX12) is Story 2.5's gate.

## Bevy 0.18 deviations from the story spec

Discovered while implementing Task 1 / Task 6 / Task 10:

1. **`@group(2)` → `@group(#{MATERIAL_BIND_GROUP})`** — Bevy 0.18 reserves `@group(2)` for mesh-related bindings (morph targets, skinning) and shifted custom-material uniforms to a dynamic slot resolved at pipeline-bind time. Hard-coding `@group(2)` produced a wgpu validation error (`opaque_mesh_pipeline` layout mismatch). The shader now uses Bevy's `#{MATERIAL_BIND_GROUP}` shader-def substitution per the canonical pattern in `bevy_pbr::material`'s docs.
2. **`bevy::render::render_resource::ShaderRef` → `bevy::shader::ShaderRef`** — `ShaderRef` moved to the new `bevy_shader` crate in 0.18.
3. **`#[derive(Event)]` → `#[derive(Message)]`, `EventReader` → `MessageReader`, `EventWriter` → `MessageWriter`, `App::add_event` → `App::add_message`** — Bevy 0.18's Event-infrastructure rename.
4. **`AssetLoader: TypePath`** — the trait now requires `TypePath`; `TuningConfigLoader` derives it.
5. **`file_watcher` Bevy feature required for hot-reload** — `AssetPlugin::watch_for_changes_override = Some(true)` is silently inert without the `file_watcher` cargo feature; added to `Cargo.toml`'s Bevy feature set on both the cross-platform and the Linux-target dependency lines.
