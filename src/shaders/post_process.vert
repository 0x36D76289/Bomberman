#version 460

layout(location = 0) in vec2 in_position;
layout(location = 1) in vec2 in_uv;

layout(location = 0) out vec2 out_uv;

// vec2 tex_coords[6] = vec2[](
//     vec2(0.0, 0.0),
//     vec2(1.0, 0.0),
//     vec2(0.0, 1.0),
//     vec2(0.0, 1.0),
//     vec2(1.0, 1.0),
//     vec2(1.0, 0.0)
// );

vec2 positions[6] = vec2[](
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(-1.0, 1.0),
    vec2(1.0, 1.0),
    vec2(1.0, -1.0)
);

vec2 tex_coords[6] = vec2[](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0),
    vec2(1.0, 0.0)
);

void main() {
    out_uv = in_uv;
    gl_Position = vec4(in_position, 0.0, 1.0);
}