#version 130

// Simple fragment shader, used to copy a texture to the screen.

uniform sampler2D src;

in vec2 frag_pos;

out vec4 color;

void main() {
    vec2 prev_coord = vec2((frag_pos.x + 1.0)/2.0, (frag_pos.y + 1.0)/2.0);

    color = vec4(texture(src, prev_coord).rgb, 1.0);
}

// vim: ft=glsl
