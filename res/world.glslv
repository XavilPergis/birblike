#version 430

in vec3 color;
in vec2 position;

out vec4 vert_color;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    vert_color = vec4(color, 1.0);
}
