#version 460

layout(location = 0) in vec3 in_color;
layout(location = 1) in vec3 in_position_world;
layout(location = 2) in vec3 in_normal_world;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform GlobalUbo {
    mat4 view;
    mat4 projection;
    vec4 ambient_light_color;
    vec3 light_position;
    vec4 light_color;
} ubo;

void main() {
    vec3 direction_to_light = ubo.light_position - in_position_world;
    float attenuation = 1.0 / dot(direction_to_light, direction_to_light);

    vec3 light_color = ubo.light_color.xyz * ubo.light_color.w * attenuation;
    vec3 ambient_light = ubo.ambient_light_color.xyz * ubo.ambient_light_color.w;
    vec3 diffuse_light = light_color * max(dot(normalize(in_normal_world), normalize(direction_to_light)), 0);

    f_color = vec4((diffuse_light + ambient_light) * in_color, 1.0);
}