#version 460

const int MAX_JOINTS = 100;
const int MAX_WEIGHTS = 4;

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_uv;
layout(location = 3) in uvec4 in_joint_indices;
layout(location = 4) in vec4 in_joint_weights;

layout(location = 0) out vec3 out_color;
layout(location = 1) out vec3 out_normal_world;
layout(location = 2) out vec2 out_uv;

layout(set = 0, binding = 0) uniform GlobalUbo {
    mat4 projection;
    mat4 view;
    vec4 ambient_light_color;
    vec3 direction_to_light;
    vec4 directional_light_color;
} ubo;

layout(set = 1, binding = 0) uniform JointsUbo {
    mat4 joint_transforms[MAX_JOINTS];
} joints;

layout(push_constant) uniform AnimationPush {
    mat4 model_matrix;
    mat4 normal_matrix;
    vec3 color;
    int tex_index;
    int tint_whites;
} push;

void main()
{
    mat4 skin_matrix = mat4(0.0);
    float weight_sum = 0.0;

    for (int i = 0; i < MAX_WEIGHTS; ++i)
    {
        skin_matrix += joints.joint_transforms[in_joint_indices[i]] * in_joint_weights[i];
        weight_sum += in_joint_weights[i];
    }

    if (weight_sum == 0.0)
        skin_matrix = mat4(1.0);

    vec4 position_world = push.model_matrix * skin_matrix * vec4(in_position, 1.0);

    gl_Position = ubo.projection * ubo.view * position_world;
    out_color = push.color;

    out_normal_world = normalize(mat3(push.normal_matrix) * mat3(skin_matrix) * in_normal);
    out_uv = in_uv;
}
