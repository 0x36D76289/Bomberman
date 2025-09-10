#version 460

layout(binding = 0) uniform sampler2D renderedImage;

layout(location = 0) in vec2 inTexCoord;
layout(location = 0) out vec4 outColor;

// Optional: color palette reduction
vec3 reduceColorPalette(vec3 color, int levels) {
    return floor(color * levels) / levels;
}

// Optional: scanline effect
float scanlineEffect(vec2 coord, float resolution) {
    return sin(coord.y * resolution * 3.14159) * 0.1 + 0.9;
}

void main() {
    // Sample the low-resolution texture
    vec4 color = texture(renderedImage, inTexCoord);
    
    // Apply pixel art effects (optional)
    // color.rgb = reduceColorPalette(color.rgb, 10); // 5-color palette
    // color.rgb *= scanlineEffect(inTexCoord, RENDER_HEIGHT); // Scanlines
    
    outColor = color;
}