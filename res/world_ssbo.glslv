#version 430

buffer positions {
    vec2 positions_buf[];
};

buffer colors {
    vec4 colors_buf[];
};

// uniform float time;
// uniform float scale;
// uniform float offset;
uniform ivec2 tile_amounts;

in vec2 position;
out vec4 out_color;

void main() {
    vec2 pos = positions_buf[gl_InstanceID];
    // Map world coords to normalized space
    vec2 norm_pos = vec2(pos.x / float(tile_amounts.x), pos.y / float(tile_amounts.y));
    vec2 ndc_pos = 2.0 * norm_pos - vec2(1.0);

    vec2 vert_offset = 2.0 * vec2(position.x / float(tile_amounts.x), position.y / float(tile_amounts.y));
    // vec2 toffset = vec2(0.33 * sin(time), 0.33 * cos(time));
    gl_Position = vec4(ndc_pos + vert_offset, 0.0, 1.0);
    out_color = colors_buf[gl_InstanceID];
}
