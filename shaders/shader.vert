#version 450

layout(binding = 0) uniform UniformBufferObject {
    float time;
} ubo;

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;

void main() {
    //gl_VertexIndex
    gl_Position = vec4(inPosition, 0.0, 1.0);
    fragColor = inColor; //vec3(sin(ubo.time), gl_Position.xy);
}
