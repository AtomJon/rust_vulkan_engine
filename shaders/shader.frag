#version 450

layout(binding = 0) uniform UniformBufferObject {
    float time;
} ubo;

layout(location = 0) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(fragColor.xy, 0.5, 1.0);
}
