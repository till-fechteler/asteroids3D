//! asteroids3D — minimal Bevy app entry point.
//! Opens a default native window via DefaultPlugins.

use bevy::prelude::*;

fn main() {
    App::new().add_plugins(DefaultPlugins).run();
}
