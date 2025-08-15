#version 460
#extension GL_EXT_nonuniform_qualifier : enable

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
    vec3 direction_to_light;
    vec4 directional_light_color;
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
    vec3 surface_normal = normalize(in_normal_world);

    // ambient lighting
    vec3 ambient_light = ubo.ambient_light_color.rgb * ubo.ambient_light_color.a;

    // diffuse lighting
    vec3 diffuse_light = ubo.directional_light_color.rgb * ubo.directional_light_color.a;
    float light_intensity = max(dot(surface_normal, ubo.direction_to_light), 0);

    // specular lighting
    vec3 camera_world_pos = ubo.inverse_view[3].xyz;
    vec3 view_direction = normalize(camera_world_pos - in_position_world);
    vec3 half_angle = normalize(ubo.direction_to_light + view_direction);
    float blinn_term = dot(surface_normal, half_angle);
    blinn_term = clamp(blinn_term, 0, 1);
    blinn_term = pow(blinn_term, 512.0);
    vec3 specular_light = vec3(0.3) * blinn_term;

    // get the texture color if the object has one or get the object color
    vec3 color;
    if (push.tex_index >= 0) {
        color = texture(nonuniformEXT(sampler2D(tex[push.tex_index], s)), in_uv).rgb;
    } else {
        color = in_color;
    }

    f_color = vec4((ambient_light + light_intensity + specular_light) * color, 1.0);
}