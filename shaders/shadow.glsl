#version 140
out vec4 colour;
in vec2 uv;
uniform vec3 col;
void main() {
    colour = vec4(col, float(length(uv - vec2(0.5)) < 0.5));
}
