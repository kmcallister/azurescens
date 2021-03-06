// Fragment shader which runs video feedback between two textures.

precision mediump float;

// Source texture (previous frame)
uniform sampler2D src_near;  // Nearest-pixel sampling
uniform sampler2D src_lin;   // Linear interpolation

uniform float scale;
uniform vec2 param_c;
uniform float param_t;
uniform bool invert;
uniform bool permute_colors;
uniform float fade;
uniform float color_cycle_rate;
uniform float mix_linear;
uniform float mix_linear_tv;

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

// Complex modulus.
float cmod(vec2 z) {
    return length(z);
}

// Complex argument (principal).
float carg(vec2 z) {
    return atan(z.y, z.x);
}

// Complex logarithm.
vec2 clog(vec2 z) {
    return vec2(log(cmod(z)), carg(z));
}

void main() {
    vec2 z = pos_to_z(frag_pos);

    // The central feedback equation.
    // z is the current fragment coordinate
    // zprev is where we get its color from in the previous frame.
    vec2 zprev = cmult(z, z) + 1.2*sin(param_c);
    float a = z_to_tex(param_c).y;
    zprev.y = mix(sin(zprev.y), zprev.y, a);

    vec2 zt = z_to_tex(zprev);
    color = mix(texture(src_near, zt), texture(src_lin, zt),
        mix_linear + mix_linear_tv*fract(0.1*param_t));

    // Color the borders, as an initial condition for the iteration.
    float lim = 0.9*scale;
    if ((zprev.x <= -lim)
        || (zprev.x >= lim)
        || (zprev.y <= -lim)
        || (zprev.y >= lim)) {

        float h = 2.0*cos(0.128*color_cycle_rate*param_t + 0.1*zprev.x) + 1.0;
        float s = 0.6 + 0.2*(sin(0.240*color_cycle_rate*param_t + zprev.y) + 1.0);

        color += vec4(hsv2rgb(vec3(h, s, 1.0)), 1.0);
    }

    // Final color mapping / inversion.
    if (invert) {
        color = vec4(vec3(1.0, 1.0, 1.0) - color.rgb, 1.0);
    } else {
        color = vec4(fade*color.rgb, 1.0);
    }

    if (permute_colors) {
        color = vec4(color.gbr, 1.0);
    }
}

// vim: ft=glsl
