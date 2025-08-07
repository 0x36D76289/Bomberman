#version 460
#define MAX_LIGHT_NUMBER 100

const vec2 OFFSETS[6] = vec2[](
  vec2(-1.0, -1.0),
  vec2(-1.0, 1.0),
  vec2(1.0, -1.0),
  vec2(1.0, -1.0),
  vec2(-1.0, 1.0),
  vec2(1.0, 1.0)
);

struct PointLight {
  vec4 position;
  vec4 color;
};

layout(location = 0) out vec2 frag_offset;

layout(set = 0, binding = 0) uniform GlobalUbo {
    mat4 projection;
    mat4 view;
    mat4 inverse_view;
    vec4 ambient_light_color;
    PointLight lights[MAX_LIGHT_NUMBER];
    int light_number;
} ubo;

layout(push_constant) uniform Push {
    vec4 position;
    vec4 color;
    float radius;
} push;

void main() {
    frag_offset = OFFSETS[gl_VertexIndex];
    vec3 camera_right_world = {ubo.view[0][0], ubo.view[1][0], ubo.view[2][0]};
    vec3 camera_up_world = {ubo.view[0][1], ubo.view[1][1], ubo.view[2][1]};

    vec3 position_world = push.position.xyz
        + push.radius * frag_offset.x * camera_right_world
        + push.radius * frag_offset.y * camera_up_world;

    gl_Position = ubo.projection * ubo.view * vec4(position_world, 1.0);
}
