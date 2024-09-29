#version 140
out vec4 colour;
in vec2 uv;

uniform mat4 camera;
uniform vec4 colour1;
uniform vec4 colour2;

void main(){
	float gradient = dot(
		normalize(inverse(mat3(camera)) * vec3(uv - vec2(0.5), 0.5)),
		vec3(0, 0.5, 0)
	) + 0.5;
    colour = mix(colour2, colour1, gradient);
}
