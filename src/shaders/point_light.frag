#version 460
#define MAX_LIGHT_NUMBER 100

struct PointLight {
  vec4 position;
  vec4 color;
};

layout(location = 0) in vec2 frag_offset;
layout(location = 0) out vec4 out_color;

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

const float M_PI = 3.1415926538;

void main() {
    float dis = sqrt(dot(frag_offset, frag_offset));
    if (dis >= 1.0) {
        discard;
    }

    float cos_dis = 0.5 * (cos(dis * M_PI) + 1.0);
    out_color = vec4(push.color.xyz + 0.5 * cos_dis, cos_dis);
}