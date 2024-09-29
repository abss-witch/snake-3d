use thin_engine::{
    prelude::*, glium::{uniforms::*, self},
    glium::texture::RawImage2d
};
use std::fs::File;
pub fn sound(sound: &str) -> Result<Box<dyn awedio::Sound>, ()> {
    awedio::sounds::open_file(format!("sounds/{sound}.mp3")).map_err(|i| println!("{i}"))
}
pub fn play_sound<T: awedio::Sound + 'static>(sound: Result<T, ()>, manager: &mut Result<awedio::manager::Manager, ()>) {
    let Ok(sound) = sound else { return; };
    let Ok(manager) = manager else { return; };
    manager.play(Box::new(sound));
}
pub struct Mesh {
    index: IndexBuffer<u32>,
    vertex: VertexBuffer<Vertex>,
    normal: VertexBuffer<Normal>,
    uv: VertexBuffer<TextureCoords>
}
impl Mesh {
    pub fn load(mesh: &str, display: &thin_engine::Display) -> Self {
        let mesh = &tobj::load_obj(format!("meshes/{mesh}.obj"), &tobj::GPU_LOAD_OPTIONS).unwrap().0[0].mesh;
        let pos: Vec<Vertex> = mesh.positions.chunks(3).map(|i|
            vec3(i[0], i[1], i[2]).into()
        ).collect();
        let normals: Vec<Normal> = mesh.normals.chunks(3).map(|i|
            vec3(i[0], i[1], i[2]).into()
        ).collect();
        let uvs: Vec<TextureCoords> = mesh.texcoords.chunks(2).map(|i|
            vec2(i[0], i[1]).into()
        ).collect();
        let (index, vertex, normal, uv) = mesh!(display, &mesh.indices, &pos, &normals, &uvs);
        Self { index, vertex, normal, uv }
    }
    pub fn mesh(&self) -> (&VertexBuffer<Vertex>, &VertexBuffer<Normal>, &VertexBuffer<TextureCoords>) {
        (&self.vertex, &self.normal, &self.uv)
    }
    pub fn index(&self) -> &IndexBuffer<u32> { &self.index }
}

pub fn image(image: &str, display: &thin_engine::Display) -> Texture2d {
    let file = File::open(format!("sprites/{image}.png")).unwrap();
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();

    let tex = RawImage2d::from_raw_rgba_reversed(
        &buf, (info.width, info.height)
    );
    Texture2d::new(display, tex).unwrap()
}
pub fn sampler(tex: &Texture2d) -> Sampler<'_, Texture2d> {
    Sampler::new(tex)
        .magnify_filter(MagnifySamplerFilter::Nearest)
        .wrap_function(SamplerWrapFunction::Repeat)
}
use std::io::Read;
pub fn shader(file: &str, screen_vertex: bool, display: &thin_engine::Display) -> Program {
    let mut fragment = String::new();
    File::open(format!("shaders/{file}.glsl"))
        .unwrap()
        .read_to_string(&mut fragment)
        .unwrap(); 
    let vertex = if screen_vertex { shaders::SCREEN_VERTEX } else { shaders::VERTEX };
    Program::from_source(display, vertex, &fragment, None).unwrap()
}

pub struct ImageDrawer<'a> {
    pub screen_mesh: (&'a VertexBuffer<Vertex>, &'a VertexBuffer<TextureCoords>),
    pub screen_indices: &'a IndexBuffer<u32>,
    pub shader: &'a Program,
    pub image_params: &'a DrawParameters<'a>,
    pub view2d: Mat4,
    pub frame: &'a mut Frame
}
impl ImageDrawer<'_> {
    pub fn draw(
        &mut self, tex: &Texture2d, pos: Vec2,
        scale: Vec2, size: Vec2, offset: Vec2
    ) {
        let model = Mat4::from_pos_and_scale(pos.extend(0.0), scale.extend(1.0));
        self.frame.draw(
            self.screen_mesh, self.screen_indices, self.shader, &uniform! {
                tex: sampler(tex), camera: Mat4::default(), model: model,
                size: size, offset: offset, view: self.view2d
            }, self.image_params
        ).unwrap()
    }
    pub fn draw_simple(&mut self, tex: &Texture2d, pos: Vec2, scale: f32) {
        let x = tex.width() as f32 / tex.height() as f32;
        self.draw(tex, pos, vec2(scale*x, scale), Vec2::ONE, Vec2::ZERO);
    }
}
