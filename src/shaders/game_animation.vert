#version 460

const int MAX_JOINTS = 100;
const int MAX_WEIGHTS = 3;

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_uv;
layout(location = 3) in uvec3 in_joint_indices;
layout(location = 4) in vec3 in_joint_weights;

layout(location = 0) out vec3 out_color;
layout(location = 1) out vec3 out_position_world;
layout(location = 2) out vec3 out_normal_world;
layout(location = 3) out vec2 out_uv;

layout(set = 0, binding = 0) uniform GlobalUbo {
    mat4 projection;
    mat4 view;
    mat4 inverse_view;
    vec4 ambient_light_color;
    vec3 direction_to_light;
    vec4 directional_light_color;
} ubo;

layout(set = 0, binding = 1) uniform JointsUbo {
    mat4 joint_transforms[MAX_JOINTS];
} joints;

layout(push_constant) uniform GamePush {
    mat4 model_matrix;
    mat4 normal_matrix;
    vec3 color;
    int tex_index;
} push;

void main() {
    vec4 total_local_pos = vec4(0.0);
    vec4 total_normal = vec4(0.0);
    for (int i = 0; i < MAX_WEIGHTS; ++i) {
        if (in_joint_weights[i] == 0.0)
            break;

        vec4 local_pos = joints.joint_transforms[in_joint_indices[i]] * vec4(in_position, 1.0);
        total_local_pos += local_pos * in_joint_weights[i];

        vec4 world_normal = joints.joint_transforms[in_joint_indices[i]] * vec4(in_normal, 0.0);
        total_normal += world_normal * in_joint_weights[i];
    }


    vec4 position_world = push.model_matrix * total_local_pos;
    gl_Position = ubo.projection * ubo.view * position_world;

    out_color = push.color;
    out_position_world = position_world.xyz;
    out_normal_world = normalize(mat3(push.normal_matrix) * total_normal.xyz);
    out_uv = in_uv;

    // vec4 position_world = push.model_matrix * vec4(in_position, 1.0);
    // gl_Position = ubo.projection * ubo.view * position_world;

    // out_color = push.color;
    // out_position_world = position_world.xyz;
    // out_normal_world = normalize(mat3(push.normal_matrix) * in_normal);
    // out_uv = in_uv;
}