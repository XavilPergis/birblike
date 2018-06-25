#version 430

buffer positions { vec2 positions_buf[]; };
buffer fg_colors { vec4 fg_colors_buf[]; };
buffer bg_colors { vec4 bg_colors_buf[]; };
buffer uvs { vec2 uv_buf[]; };

// uniform float time;
// uniform float scale;
// uniform float offset;
uniform ivec2 tile_amounts;

in vec2 position;
out vec4 out_fg_color;
out vec4 out_bg_color;
out vec2 out_uv;

void main() {
    vec2 pos = positions_buf[gl_InstanceID];
    // Map world coords to normalized space
    vec2 norm_pos = vec2(pos.x / float(tile_amounts.x), pos.y / float(tile_amounts.y));
    vec2 ndc_pos = 2.0 * norm_pos - vec2(1.0);

    vec2 vert_offset = 2.0 * vec2(position.x / float(tile_amounts.x), position.y / float(tile_amounts.y));
    gl_Position = vec4(ndc_pos + vert_offset, 0.0, 1.0);
    out_fg_color = fg_colors_buf[gl_InstanceID];
    out_bg_color = bg_colors_buf[gl_InstanceID];
    vec2 uv = uv_buf[gl_InstanceID];
    out_uv = (vec2(uv.x, 15.0 - uv.y) / 16.0) + position / 16.0;
}
