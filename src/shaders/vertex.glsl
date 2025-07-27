#version 460

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_uv;

layout(location = 0) out vec3 out_color;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    vec3 color;
} uniforms;

void main() {
    out_color = uniforms.color;
    mat4 projection_view = uniforms.proj * uniforms.view;
    mat4 transform = projection_view * uniforms.world;
    gl_Position = transform * vec4(in_position, 1.0);
}