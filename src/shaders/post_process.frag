#version 460

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 f_color;

layout(binding = 0) uniform sampler2D rendered_image;

void main() {
    f_color = texture(rendered_image, in_uv);
}