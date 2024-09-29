#![windows_subsystem = "windows"]
use thin_engine::{
    meshes::screen, prelude::*,
    glium::{self, framebuffer::SimpleFrameBuffer},
    ResizableTexture2D,
    ResizableDepthTexture2D,
};
use crate::snake::*;
use crate::load::*;
use awedio::Sound;
mod load;
mod snake;
#[derive(ToUsize)]
enum Action {
    Up, Down, Left, Right, Forward, Back,
    ExpandMap, ShrinkMap, SpeedUp, SlowDown,
    ToggleFullscreen, Exit, Mute
}
use Action::*;
fn main() {
    let mut input = input_map!(
        (Exit,             KeyCode::Escape),
        (ToggleFullscreen, KeyCode::KeyF),
        (Mute,             KeyCode::KeyM),
        (SpeedUp,          KeyCode::Digit1),
        (SlowDown,         KeyCode::Digit2),
        (ExpandMap,        KeyCode::Equal),
        (ShrinkMap,        KeyCode::Minus),
        (Forward,          KeyCode::KeyW, KeyCode::ArrowUp),
        (Back,             KeyCode::KeyS, KeyCode::ArrowDown),
        (Down,             KeyCode::KeyQ, KeyCode::Enter),
        (Up,               KeyCode::KeyE, KeyCode::Space),
        (Left,             KeyCode::KeyA, KeyCode::ArrowLeft),
        (Right,            KeyCode::KeyD, KeyCode::ArrowRight)
    );

    //create window
    let (event_loop, window, display) = thin_engine::set_up().unwrap();
    window.set_cursor_visible(false);
    window.set_title("Snake 3D");
    let _ = window.set_cursor_grab(CursorGrabMode::Locked);
    
    // try to create audio
    let backend = awedio::start().map_err(|i| println!("{i}"));
    let (mut manager, _backend) = match backend { 
        Ok((a, b)) => (Ok(a), Ok(b)),
        Err(a) => (Err(a), Err(a)) 
    };
    let manager = &mut manager;
    
    // load music and add controller to it
    // if failed wont exit because you dont need sound to play video games.
    let music_sound = sound("music")
        .and_then(|i| i
            .loop_from_memory()
            .map(|i| i.pausable().controllable())
            .map_err(|i| println!("{i}")
    ));
    let (music_sound, mut music) = match music_sound {
        Ok((a, b)) => (Ok(a), Ok(b)),
        Err(a) => (Err(a), Err(a))
    };
    play_sound(music_sound, manager);

    let eat_sound = sound("eat")
        .and_then(|i| i
            .into_memory_sound()
            .map_err(|i| println!("{i}")
    ));
    let ow_sound  = sound("ow")
        .and_then(|i| i
            .into_memory_sound()
            .map_err(|i| println!("{i}")
    ));
    // load textures
    let controls_tex = image("controls", &display);
    let speed_tex    = image("speed",    &display);
    let map_tex      = image("map",      &display);
    let board_tex    = image("board",    &display);
    let face_tex     = image("faces",    &display);
    let start_tex    = image("start",    &display);
    let re_tex       = image("re",       &display);
    let win_tex      = image("win",      &display);
    let mut depth  = ResizableDepthTexture2D::default();
    let mut colour = ResizableTexture2D::default();

    // load meshes
    let apple  = Mesh::load("apple", &display);
    let snake  = Mesh::load("snake", &display);
    let face   = Mesh::load("face",  &display);
    let board  = Mesh::load("board", &display);
    let (screen_indices, screen_vertices, screen_uvs) = mesh!(
        &display, &screen::INDICES, &screen::VERTICES, &screen::UVS
    );
    let screen_mesh = (&screen_vertices, &screen_uvs);

    // load shaders
    let shaded_shader     = shader("shaded",     false, &display);
    let background_shader = shader("background", true,  &display);
    let image_shader      = shader("image",      false, &display);
    let shadow_shader     = shader("shadow",     false, &display);
    let fxaa_shader = shaders::fxaa_shader(&display).unwrap();

    // create draw parameters
    let mesh_parameters = params::alias_3d();
    let image_parameters = DrawParameters{
        blend: draw_parameters::Blend::alpha_blending(),
        backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
        ..Default::default()
    };

    // create game
    let mut size = 4;
    let mut rng = rand::thread_rng();
    let mut game = Board::new(size, size, size);
    game.state = State::Wait;

    let mut prev_dir = Direction::Forward.dir();
    let mut prev_length = 1;
    let mut cam_rot = vec2(0.0, 0.4);
    let mut play_sounds = true;

    // create time
    let mut speed = 3;
    let mut fixed_loop_timer = 0.4;
    let mut fixed_loop = Instant::now();
    let menu_loop_timer = 0.17;
    let mut menu_loop = Instant::now();
    let now = Instant::now();
    let mut delta = 0.0;

    let (mut apple_mat, mut snake_parts_mat, mut shadows_mat) = game.matrices();
    thin_engine::run(event_loop, &mut input, |input, target| {
        let elapsed = Instant::now();
        let screen_size = window.inner_size().into();
        display.resize(screen_size);
        depth.resize_to_display(&display);
        colour.resize_to_display(&display);
        let depth = depth.texture();
        let colour = colour.texture();
        let mut frame = SimpleFrameBuffer::with_depth_buffer(
            &display, colour, depth
        ).unwrap();
        let view   = Mat4::view_matrix_3d(screen_size, 1.0, 1024.0, 1.0);
        let view2d = Mat4::view_matrix_2d(screen_size);

        let menu_timer_looped = menu_loop.elapsed().as_secs_f32() >= menu_loop_timer;

        // controls
        if input.pressed(ToggleFullscreen) {
            window.set_fullscreen( match window.fullscreen() {
                None => Some(Fullscreen::Borderless(None)),
                _ => None
            }
        ) }

        if input.pressed(Mute) {
            play_sounds = !play_sounds;
            if let Ok(ref mut i) = music { 
                i.set_paused(!play_sounds)
            }
        }

        let changed_map = input.pressed(ExpandMap) || input.pressed(ShrinkMap);
        if game.state != State::Alive && (menu_timer_looped || changed_map) {
            let change = input.axis(ShrinkMap, ExpandMap) as i32;
            size = 2.max(size as i32 - change).min(255) as usize;
            if change != 0 { menu_loop = Instant::now(); }
        }
        
        let changed_speed = input.pressed(SpeedUp) || input.pressed(SlowDown);
        if game.state != State::Alive && (changed_speed || menu_timer_looped) {
            let change = input.axis(SpeedUp, SlowDown) as i8;
            speed = (speed + change).rem_euclid(7);        // map from 0 to 5
            fixed_loop_timer = speed_timer(speed);
            if change != 0 { menu_loop = Instant::now() }
        }

        if input.pressed(Exit) { target.exit() }

        let mut move_input = true;
        if input.pressed(Forward)    { game.snake.direction = Direction::Forward }
        else if input.pressed(Back)  { game.snake.direction = Direction::Back    }
        else if input.pressed(Left)  { game.snake.direction = Direction::Left    }
        else if input.pressed(Right) { game.snake.direction = Direction::Right   }
        else if input.pressed(Up)    { game.snake.direction = Direction::Up      }
        else if input.pressed(Down)  { game.snake.direction = Direction::Down    }
        else { move_input = false }

        // reset
        if game.state != State::Alive && move_input && menu_timer_looped {
            game = Board::new(size, size, size);
        }
        
        cam_rot += input.mouse_move.scale(delta);

        //update game every `fixed_loop_timer` seconds
        if (fixed_loop.elapsed().as_secs_f32() >= fixed_loop_timer || move_input) && game.state == State::Alive {
            fixed_loop = Instant::now();
            game.update(&mut rng);
            (apple_mat, snake_parts_mat, shadows_mat) = game.matrices();

            if prev_length != game.snake_body.len() && play_sounds { 
                play_sound(eat_sound.clone(), manager);
            }
            if game.state == State::Dead && play_sounds { 
                play_sound(ow_sound.clone(), manager);
            }
            prev_length = game.snake_body.len();
            prev_dir = game.snake.direction.dir();
        }
        
        // aplied in inverse order because vertex shader takes inverse camera.
        // simplify (Vec3::splat(size as f32)).length() to size* sqrt(3)
        let camera = Mat4::from_pos_and_rot(
            vec3(0.0, 0.0, size as f32*3.0_f32.sqrt()),
            Quaternion::from_x_rot(-cam_rot.y) * Quaternion::from_y_rot(-cam_rot.x)
        );
        let board_transform = Mat4::from_scale_and_rot(
            Vec3::splat(size as f32),
            Quaternion::from_y_rot(std::f32::consts::PI/2.0)
        );
        let (width, height) = frame.get_dimensions();
        let right_side = width as f32 / height as f32;

        //draw background
        frame.draw(
            (&screen_vertices, &screen_uvs), &screen_indices, &background_shader,
            &uniform! {
                model: Mat4::default(), view: Mat4::default(),
                camera: camera,
                colour1: vec4(0.3, 0.3, 0.7, 1.0),
                colour2: vec4(0.2, 0.2, 0.3, 1.0)
            },
            &image_parameters
        ).unwrap();
        frame.clear_depth(1.0);

        //draw board
        frame.draw(
            board.mesh(), board.index(), &image_shader,
            &uniform! {
                camera: camera, model: board_transform,
                tex: sampler(&board_tex),
                view: view, offset: Vec2::ZERO,
                size: Vec2::splat(size as f32)
            },
            &image_parameters
        ).unwrap();

        //draw shadows
        shadows_mat.iter().for_each(|i| {
            let is_lighter = (i.x % 2 == 0) ^ (i.y % 2 == 0); // changes colour based on grid
            let colour = if is_lighter { vec3(45.0, 92.0, 57.0) } else { vec3(39.0, 75.0, 49.0) }.scale(1.0/255.0);
            frame.draw(
                screen_mesh, &screen_indices, &shadow_shader,
                &uniform! { camera: camera, model: game.shadow_matrix(*i), view: view, col: colour },
                &image_parameters
            ).unwrap()
        });

        //draw face
        let face_pos = game.board_to_space(game.snake.pos);
        let face_rot = game.snake.direction.rot();
        frame.draw(
            face.mesh(), face.index(), &image_shader,
            &uniform! {
                tex: sampler(&face_tex),
                camera: camera, view: view,
                model: Mat4::from_pos_and_rot(face_pos, face_rot),
                size: Vec2::ONE, offset: Vec2::ZERO,
            }, 
            &mesh_parameters
        ).unwrap();

        //draw snake
        snake_parts_mat.iter().for_each(|i| frame.draw(
            snake.mesh(), snake.index(), &shaded_shader,
            &uniform! {
                camera: camera, model: *i, view: view,
                albedo:   vec4(0.2, 0.6,  0.3, 1.0),
                shadow:   vec4(0.2, 0.5,  0.2, 1.0),
                specular: vec4(0.3, 0.65, 0.4, 1.0)
            },
            &mesh_parameters
        ).unwrap());

        //draw apples
        if game.state != State::Win { frame.draw(
            apple.mesh(), apple.index(), &shaded_shader,
            &uniform! {
                camera: camera, model: apple_mat, view: view,
                albedo:   vec4(1.0, 0.3, 0.5, 1.0),
                shadow:   vec4(0.6, 0.2, 0.3, 1.0),
                specular: vec4(1.0, 0.5, 0.6, 1.0)
            },
            &mesh_parameters
        ).unwrap(); }
        
        // apply fxaa
        let mut frame = display.draw();
        frame.draw(
            screen_mesh, &screen_indices, &fxaa_shader,
            &shaders::fxaa_uniforms(colour), &image_parameters
        ).unwrap();
    
        // since thin_engine is so thin, it is encouraged to make stuff to help you.
        let mut image = ImageDrawer {
            screen_mesh, screen_indices: &screen_indices, shader: &image_shader,
            image_params: &image_parameters, view2d, frame: &mut frame
        };
        // draw ui
        match game.state {
            State::Alive => (),
            State::Win => image.draw_simple(&win_tex, Vec2::ZERO, 0.5),
            _ => {
                let elapsed = now.elapsed().as_secs_f32();

                //draw controls
                image.draw_simple(&controls_tex, vec2(0.6 - right_side, 0.35), 0.6);
                image.draw_simple(&map_tex, vec2(right_side - 0.4, 0.25), 0.7);
                image.draw_simple(&start_tex, vec2(0.0, elapsed.sin()*0.05), 0.25);
                if game.state != State::Wait{
                    image.draw_simple(&re_tex, vec2(-0.55, (elapsed-0.5).sin()*0.06 - 0.15), 0.1);
                }
                image.draw(
                    &speed_tex, vec2(right_side - 0.6 / 4.0, -0.9),
                    vec2(0.6 / 4.0, 0.1), vec2(1.0, 1.0 / 7.0),
                    vec2(0.0, speed as f32 / 7.0),
                );
            }
        }
        frame.finish().unwrap();
        thread::sleep(Duration::from_nanos(16666666u64).saturating_sub(elapsed.elapsed()));
        delta = elapsed.elapsed().as_secs_f32();
    }).unwrap();
}

fn speed_timer(value: i8) -> f32 {
    match value {
        0 => 0.1,
        1 => 0.2,
        2 => 0.4,
        3 => 0.5,
        4 => 0.7,
        5 => 1.0,
        6 => f32::INFINITY,
        _ => unreachable!()
    }
}
