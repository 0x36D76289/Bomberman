#version 460
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_color;
layout(location = 2) flat in int in_tex;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler s;
layout(set = 0, binding = 1) uniform texture2D tex[];

void main() {
    if (in_tex >= 0)
        f_color = texture(nonuniformEXT(sampler2D(tex[in_tex], s)), in_uv) * in_color;
    else
        f_color = in_color;
}
