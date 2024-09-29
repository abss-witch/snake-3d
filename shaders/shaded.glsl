#version 140
out vec4 colour;
in vec3 v_normal;
in vec3 v_position;
uniform mat4 camera;
uniform vec4 albedo;
uniform vec4 shadow;
uniform vec4 specular;

const vec3 light = normalize(vec3(0.2, 1.0, 0.2));
void main() {
	vec3 camera_dir = inverse(mat3(camera)) * vec3(0, 0, -1);
	vec3 half_direction = normalize(light + camera_dir);
	bool specular_cut = dot(half_direction, normalize(v_normal)) > 0.95;

	if (specular_cut) {
        	colour = specular;
	} else if (0.0 < dot(normalize(v_normal), light)){ 
		colour = albedo;
	} else {
        	colour = shadow;
    	}
}
