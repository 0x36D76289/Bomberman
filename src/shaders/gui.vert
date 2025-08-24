#version 460

layout(location = 0) in vec2 in_position;
layout(location = 1) in vec2 in_uv;

layout(location = 0) out vec2 out_uv;

layout(push_constant) uniform GuiPush {
    vec4 color;
    int tex_index;
} push;

void main() {
    out_uv = in_uv;
    gl_Position = vec4(in_position, 0.0, 1.0);
}