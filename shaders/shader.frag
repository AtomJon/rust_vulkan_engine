#version 450

layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform UniformBufferObject {
    float time;

    float width;
    float height;
    // vec2 resolution;
} ubo;

void main() {
    vec2 uv = gl_FragCoord.xy / vec2(ubo.width, ubo.height);
    vec3 col = 0.5 + 0.5*cos(ubo.time+uv.xyx+vec3(0,2,4));

    // float distanceFromCenter = distance(vec2(1.0), vec2(0));

    // outColor = vec4(vec3(distanceFromCenter), 1.0);
    // outColor = vec4(col, 1.0);
    outColor = vec4(uv.x, uv.y, uv.x, 1.0);
}

// float distance(vec2 a, vec2 b) {
//     return max(a, b) - min(a, b)
// }