#version 450

precision mediump float;

layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform UniformBufferObject {
    float time;

    float width;
    float height;
    // vec2 resolution;
} ubo;


float dot2( in vec2 a, in vec2 b )
{
    return a.x * b.x + a.y * b.y;
}

float dot2( in vec2 a )
{
    return dot(a, a);
}

float sdHeart( in vec2 p )
{
    p.x = abs(p.x);

    if( p.y+p.x>1.0 )
        return sqrt(dot2(p-vec2(0.25,0.75))) - sqrt(2.0)/4.0;
    return sqrt(min(dot2(p-vec2(0.00,1.00)),
                    dot2(p-0.5*max(p.x+p.y,0.0)))) * sign(p.x-p.y);
}

void main() {
    // vec2 pos = vec2(gl_FragCoord.x, -gl_FragCoord.y);
    vec2 pos = gl_FragCoord.xy;

    vec2 uv = pos / vec2(ubo.width, ubo.height) * 2.0 - 1.0;
    uv.x *= (ubo.width / ubo.height);

    // vec3 col = 0.5 + 0.5*cos(ubo.time+uv.xyx+vec3(0,2,4));

    // float distanceFromCenter = distance(uv, vec2(0.5));
    float d = sdHeart(uv);

    outColor = vec4(d, d, d, 1.0);
    // outColor = vec4(col, 1.0);
    // outColor = vec4(uv.x, uv.y, uv.x, 1.0);
}