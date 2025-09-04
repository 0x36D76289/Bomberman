#version 460

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform GlobalUbo {
    mat4 projection;
    mat4 view;
    mat4 inverse_view;
    mat4 light_view;
    vec4 ambient_light_color;
    vec3 direction_to_light;
    vec4 directional_light_color;
} ubo;

void main() {
    gl_Position = ubo.projection * ubo.light_view * vec4(position, 1.0);
}