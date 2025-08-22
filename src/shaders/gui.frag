#version 460
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 f_color;

layout(push_constant) uniform GuiPush {
    vec4 color;
    int tex_index;
} push;
layout(set = 0, binding = 0) uniform sampler s;
layout(set = 0, binding = 1) uniform texture2D tex[];

void main() {
    if (push.tex_index >= 0) {
        f_color = texture(nonuniformEXT(sampler2D(tex[push.tex_index], s)), in_uv);
    } else {
        f_color = push.color;
    }
}