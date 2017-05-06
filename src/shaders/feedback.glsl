#version 130

// Fragment shader which runs video feedback between two textures.

// Source texture (previous frame)
uniform sampler2D src;

uniform float scale;
uniform vec2 param_c;
uniform float param_t;

in vec2 frag_pos;

out vec4 color;

// Convert HSV colorspace to RGB.
vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

// Convert a position (-1, 1) x (-1, 1) to a complex number.
vec2 pos_to_z(vec2 pos) {
    return scale * pos;
}

// Convert a complex number to a texture coordinate.
vec2 z_to_tex(vec2 z) {
    return vec2( ((z.x / scale) + 1.0) / 2.0,
                 ((z.y / scale) + 1.0) / 2.0);
}

// Multiply two complex numbers.
vec2 cmult(vec2 a, vec2 b) {
    return vec2((a.x * b.x) - (a.y * b.y),
                (a.x * b.y) + (a.y * b.x));
}

void main() {
    vec2 z = pos_to_z(frag_pos);

    // The central feedback equation.
    // z is the current fragment coordinate
    // zprev is where we get its color from in the previous frame.
    vec2 zprev = cmult(z, z) + 0.8*sin(param_c);

    color = texture(src, z_to_tex(zprev));

    // Color the borders, as an initial condition for the iteration.
    float lim = 0.9*scale;
    if ((zprev.x <= -lim)
        || (zprev.x >= lim)
        || (zprev.y <= -lim)
        || (zprev.y >= lim)) {

        float x1 = 0.2*(2.0*cos(0.128*param_t + zprev.x) + 1.0);
        float x2 = 2.0*sin(0.240*param_t + zprev.y) + 1.0;

        color += vec4(hsv2rgb(vec3(x1, x2, 1.0)), 1.0);
    }

    // Final color mapping / inversion.
    color = vec4(0.8 * vec3(1.0, 1.0, 1.0) - color.gbr, 1.0);
}

// vim: ft=glsl
