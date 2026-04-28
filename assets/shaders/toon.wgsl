// FR49 toon shader — posterized N·L + rim-light, tinted by per-material `tint` uniform.
// See src/visual/toon_material.rs for the Rust binding; field order MUST match this struct.

#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct ToonMaterial {
    tint: vec4<f32>,
    steps: u32,
    rim_power: f32,
    rim_intensity: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: ToonMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Hardcoded forward-up-right diagonal light direction. Lights stay out of M1 scope;
    // M2 may replace this with a uniform fed from a Bevy DirectionalLight.
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let normal = normalize(in.world_normal);

    // Posterized N·L: floor(max(N·L, 0) * steps) / steps  (AC #1)
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let steps_f = f32(material.steps);
    let posterized = floor(n_dot_l * steps_f) / steps_f;

    // Rim light: pow(1 - N·V, rim_power) * rim_intensity  (AC #1)
    let view_dir = normalize(view.world_position - in.world_position.xyz);
    let n_dot_v = max(dot(normal, view_dir), 0.0);
    let rim = pow(1.0 - n_dot_v, material.rim_power) * material.rim_intensity;

    let lit = posterized + rim;
    return vec4<f32>(material.tint.rgb * lit, material.tint.a);
}
