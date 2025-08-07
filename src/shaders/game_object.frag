#version 460
#extension GL_EXT_nonuniform_qualifier : enable
#define MAX_LIGHT_NUMBER 100

struct PointLight {
  vec4 position;
  vec4 color;
};

layout(location = 0) in vec3 in_color;
layout(location = 1) in vec3 in_position_world;
layout(location = 2) in vec3 in_normal_world;
layout(location = 3) in vec2 in_uv;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform GlobalUbo {
    mat4 projection;
    mat4 view;
    mat4 inverse_view;
    vec4 ambient_light_color;
    PointLight lights[MAX_LIGHT_NUMBER];
    int light_number;
} ubo;
layout(set = 0, binding = 1) uniform sampler s;
layout(set = 0, binding = 2) uniform texture2D tex[];

layout(push_constant) uniform Push {
  mat4 model_matrix;
  mat4 normal_matrix;
  vec3 color;
  int tex_index;
} push;

void main() {
    vec3 diffuse_light = ubo.ambient_light_color.xyz * ubo.ambient_light_color.w;
    vec3 specular_light = vec3(0.0);
    vec3 surface_normal = normalize(in_normal_world);

    vec3 camera_pos_world = ubo.inverse_view[3].xyz;
    vec3 view_direction = normalize(camera_pos_world - in_position_world);

    for (int i = 0; i < ubo.light_number; i++) {
        PointLight light = ubo.lights[i];
        vec3 direction_to_light = light.position.xyz - in_position_world;
        float attenuation = 1.0 / dot(direction_to_light, direction_to_light); // distance squared
        direction_to_light = normalize(direction_to_light);

        float cos_angle_incidence = max(dot(surface_normal, direction_to_light), 0);
        vec3 intensity = light.color.xyz * light.color.w * attenuation;

        diffuse_light += intensity * cos_angle_incidence;

        // specular lighting
        vec3 half_angle = normalize(direction_to_light + view_direction);
        float blinn_term = dot(surface_normal, half_angle);
        blinn_term = clamp(blinn_term, 0, 1);
        blinn_term = pow(blinn_term, 512.0); // higher values -> sharper highlight
        specular_light += intensity * blinn_term;
    }

    vec3 color;
    if (push.tex_index >= 0) {
        color = texture(nonuniformEXT(sampler2D(tex[push.tex_index], s)), in_uv).xyz;
    } else {
        color = in_color;
    }

    f_color = vec4(diffuse_light * color + specular_light * color, 1.0);
}