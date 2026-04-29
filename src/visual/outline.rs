//! bevy_mod_outline integration — silhouette outlines for FR49.
//! Story 2.4 wires OutlinePlugin and the TuningConfig→OutlineVolume hot-reload propagator.
//! The fallback switch (Story 2.7, conditional on Story 2.6's go/fallback decision) is NOT
//! pre-scaffolded here — YAGNI per architecture.md:887.

use bevy::prelude::*;
use bevy_mod_outline::OutlineVolume;

use crate::tuning::TuningReloaded;

/// Listens for TuningReloaded messages and propagates outline_width / outline_color into
/// every entity that carries an OutlineVolume. Subscribes via the existing TuningSystems::Reload
/// SystemSet so future tuning-driven systems can chain on it via .after(TuningSystems::Reload).
pub(super) fn apply_tuning_to_outlines(
    mut events: MessageReader<TuningReloaded>,
    mut outlines: Query<&mut OutlineVolume>,
) {
    for event in events.read() {
        let [r, g, b, a] = event.0.outline_color;
        let new_color = Color::srgba(r, g, b, a);
        let new_width = event.0.outline_width;
        for mut volume in &mut outlines {
            volume.width = new_width;
            volume.colour = new_color;
        }
    }
}
