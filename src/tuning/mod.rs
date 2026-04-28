//! TuningPlugin — owns the TuningConfig asset handle and propagates hot-reload signals.
//! Story 2.3 introduces the plugin alongside the toon-shader knobs; the TuningSystems::Reload
//! set is reserved for future tuning-driven systems per architecture.md:347.

pub mod config;

use bevy::prelude::*;
use config::{TuningConfig, TuningConfigLoader};

pub struct TuningPlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TuningSystems {
    Reload,
}

#[derive(Resource, Default)]
pub struct TuningHandle(pub Handle<TuningConfig>);

#[derive(Message, Debug, Clone)]
pub struct TuningReloaded(pub TuningConfig);

impl Plugin for TuningPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TuningConfig>()
            .init_asset_loader::<TuningConfigLoader>()
            .init_resource::<TuningHandle>()
            .add_message::<TuningReloaded>()
            .configure_sets(Update, TuningSystems::Reload)
            .add_systems(Startup, load_tuning)
            .add_systems(
                Update,
                propagate_tuning_reload.in_set(TuningSystems::Reload),
            );
    }
}

fn load_tuning(asset_server: Res<AssetServer>, mut handle: ResMut<TuningHandle>) {
    handle.0 = asset_server.load("config/tuning.ron");
}

fn propagate_tuning_reload(
    mut events: MessageReader<AssetEvent<TuningConfig>>,
    assets: Res<Assets<TuningConfig>>,
    handle: Res<TuningHandle>,
    mut writer: MessageWriter<TuningReloaded>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } if *id == handle.0.id() => {
                if let Some(cfg) = assets.get(handle.0.id()) {
                    info!(
                        "TuningReloaded: toon_steps={} rim_power={} rim_intensity={}",
                        cfg.toon_steps, cfg.toon_rim_power, cfg.toon_rim_intensity
                    );
                    writer.write(TuningReloaded(cfg.clone()));
                }
            }
            _ => {}
        }
    }
}
