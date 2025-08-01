#version 460

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_uv;

layout(location = 0) out vec3 out_color;
layout(location = 1) out vec3 out_position_world;
layout(location = 2) out vec3 out_normal_world;

layout(set = 0, binding = 0) uniform GlobalUbo {
    mat4 projection;
    mat4 view;
    mat4 inverse_view;
    vec4 ambient_light_color;
    vec3 light_position;
    vec4 light_color;
} ubo;

layout(push_constant) uniform Push {
  mat4 model_matrix;
  mat4 normal_matrix;
  vec3 color;
} push;

void main() {
    vec4 position_world = push.model_matrix * vec4(in_position, 1.0);
    gl_Position = ubo.projection * ubo.view * position_world;

    out_color = push.color;
    out_position_world = position_world.xyz;
    out_normal_world = normalize(mat3(push.normal_matrix) * in_normal);
}