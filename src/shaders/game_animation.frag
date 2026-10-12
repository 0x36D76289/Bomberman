#version 460
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec3 in_color;
layout(location = 1) in vec3 in_normal_world;
layout(location = 2) in vec2 in_uv;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform GlobalUbo {
    mat4 projection;
    mat4 view;
    vec4 ambient_light_color;
    vec3 direction_to_light;
    vec4 directional_light_color;
} ubo;

layout(set = 0, binding = 1) uniform sampler s;
layout(set = 0, binding = 2) uniform texture2D tex[];

layout(push_constant) uniform AnimationPush {
    mat4 model_matrix;
    mat4 normal_matrix;
    vec3 color;
    int tex_index;
    int tint_whites;
} push;

void main()
{
    if (push.tex_index == -2)
    {
        f_color = vec4(in_color, 1.0);
        return;
    }

    vec3 surface_normal = normalize(in_normal_world);
    vec3 ambient_light = ubo.ambient_light_color.rgb * ubo.ambient_light_color.a;
    vec3 diffuse_light = ubo.directional_light_color.rgb * ubo.directional_light_color.a;

    float light_intensity = max(dot(surface_normal, ubo.direction_to_light), 0);
    diffuse_light *= floor(light_intensity * 3.0 + 0.5) / 3.0;

    vec3 color;

    if (push.tex_index >= 0)
    {
        vec3 texel = texture(nonuniformEXT(sampler2D(tex[push.tex_index], s)), in_uv).rgb;
        float whiteness = min(texel.r, min(texel.g, texel.b));
        color = push.tint_whites != 0 ? texel * mix(vec3(1.0), in_color, whiteness) : texel * in_color;
    }
    else
        color = in_color;

    f_color = vec4((ambient_light + diffuse_light) * color, 1.0);
}
