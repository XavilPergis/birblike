#version 430

uniform sampler2D tilemap;

in vec4 out_fg_color;
in vec4 out_bg_color;
in vec2 out_uv;
out vec4 final_color;

void main() {
    vec4 tex_color = texture(tilemap, out_uv);
    if (tex_color.a == 0.0) {
        final_color = out_bg_color;
    } else {
        final_color = tex_color * out_fg_color;
    }
}