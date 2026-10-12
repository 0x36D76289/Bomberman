#version 460

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 f_color;

layout(binding = 0) uniform sampler2D rendered_image;

layout(push_constant) uniform PostPush {
    int pixel;
} push;

const float bayer[16] = float[](
        0.0, 8.0, 2.0, 10.0,
        12.0, 4.0, 14.0, 6.0,
        3.0, 11.0, 1.0, 9.0,
        15.0, 7.0, 13.0, 5.0
    );

const float LEVELS = 8.0;

void main()
{
    vec4 color = texture(rendered_image, in_uv);
    if (push.pixel == 0)
    {
        f_color = color;
        return;
    }

    ivec2 texel = ivec2(in_uv * vec2(textureSize(rendered_image, 0)));
    float threshold = (bayer[(texel.y & 3) * 4 + (texel.x & 3)] + 0.5) / 16.0 - 0.5;
    vec3 steps = floor(color.rgb * (LEVELS - 1.0) + 0.5 + threshold * 0.35) / (LEVELS - 1.0);
    float scanline = mod(floor(gl_FragCoord.y), 2.0) < 1.0 ? 0.88 : 1.0;

    f_color = vec4(clamp(steps, 0.0, 1.0) * scanline, color.a);
}
