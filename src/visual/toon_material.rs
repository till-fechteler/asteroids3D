//! Custom WGSL Toon Material — FR49 portfolio-quality shader artifact.
//! See assets/shaders/toon.wgsl for the fragment-stage WGSL.
//! Field order MUST match the WGSL `struct ToonMaterial` (vec4 first, then scalars).

use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct ToonMaterial {
    #[uniform(0)]
    pub tint: LinearRgba,
    #[uniform(0)]
    pub steps: u32,
    #[uniform(0)]
    pub rim_power: f32,
    #[uniform(0)]
    pub rim_intensity: f32,
}

impl Default for ToonMaterial {
    fn default() -> Self {
        Self {
            tint: LinearRgba::WHITE,
            steps: 4,
            rim_power: 2.0,
            rim_intensity: 0.3,
        }
    }
}

impl Material for ToonMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/toon.wgsl".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuning::config::TuningConfig;

    #[test]
    fn toon_material_default_matches_tuning_default() {
        let m = ToonMaterial::default();
        let t = TuningConfig::default();
        assert_eq!(m.steps, t.toon_steps);
        assert_eq!(m.rim_power, t.toon_rim_power);
        assert_eq!(m.rim_intensity, t.toon_rim_intensity);
    }
}
