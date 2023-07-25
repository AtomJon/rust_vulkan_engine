#version 450

layout(binding = 0) uniform UniformBufferObject {
    float time;
    vec2 resolution;
} ubo;

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec3 inColor;

void main() {
    gl_Position = vec4(inPosition, 0.0, 1.0);
}
