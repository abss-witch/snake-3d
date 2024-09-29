#version 140
out vec4 colour;
in vec2 uv;
uniform vec2 size;
uniform vec2 offset;
uniform sampler2D tex;

void main() {
    colour = texture(tex, uv*size + offset);
}
