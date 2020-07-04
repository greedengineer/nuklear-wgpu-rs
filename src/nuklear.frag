#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragUv;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform texture2D fontTexture;
layout(set = 0, binding = 2) uniform sampler fontSampler;

void main() {
    vec4 texColor = texture(sampler2D(fontTexture,fontSampler), fragUv);
    outColor = fragColor * texColor;
}