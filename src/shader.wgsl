// shader.wgsl

// The input structure for the vertex shader.
// It must match the Vertex struct in resources.rs.
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

// The output structure for the vertex shader, which becomes the
// input for the fragment shader.
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // The position is already in clip-space, so we just need to convert
    // it to a vec4 for the @builtin(position).
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // The fragment shader simply returns the color passed from the vertex shader.
    return in.color;
}
