#version 460
#extension GL_EXT_shader_explicit_arithmetic_types : require

layout(location = 0) out vec2 out_uv;

layout(push_constant) uniform PostProcessPush {
    vec2 positions[6];
} push;

vec2 tex_coords[6] = vec2[](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0),
    vec2(1.0, 0.0)
);

void main() {
    gl_Position = vec4(push.positions[gl_VertexIndex], 0.0, 1.0);
    out_uv = tex_coords[gl_VertexIndex];
}