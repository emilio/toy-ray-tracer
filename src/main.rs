extern crate euclid;
extern crate glutin;
extern crate gleam;
extern crate tracer;

use euclid::Size2D;
use gleam::gl;
use tracer::WorldPosition;

const VERTEX_SHADER_SOURCE: &'static [u8] = b"\
#version 150                                                                   \n\
in vec2 vPosition;                                                             \n\
out vec2 fPosition;                                                            \n\
void main() {                                                                  \n\
    fPosition = vPosition;                                                     \n\
    gl_Position = vec4(fPosition, 0.0, 1.0);                                   \n\
}";


const FRAGMENT_SHADER_SOURCE: &'static [u8] = b"\
#version 150                                                                   \n\
uniform sampler2D uTexture;                                                    \n\
in vec2 fPosition;                                                             \n\
out vec4 oFragColor;                                                           \n\
void main() {                                                                  \n\
    oFragColor = texture2D(uTexture, fPosition * 0.5 + 0.5);                   \n\
    // oFragColor = vec4(fPosition * 0.5 + 0.5, 0.0, 1.0);                   \n\
}";

fn main() {
    let window = glutin::WindowBuilder::new()
        .with_title("Hello world!")
        .build()
        .unwrap();

    unsafe { window.make_current() };

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    gl::clear_color(0.0, 1.0, 0.0, 1.0);

    let gl_version = gl::get_string(gl::VERSION);
    let gl_renderer = gl::get_string(gl::RENDERER);

    println!("OpenGL version {}, {}", gl_version, gl_renderer);
    println!("hidpi factor: {}", window.hidpi_factor());

    let mut scene = tracer::Scene::new(Size2D::new(800, 800),
                                       WorldPosition::new(400.0, 200.0, -150.),
                                       WorldPosition::new(1000.0, 1000.0, -5000.));
    let sphere = tracer::ObjectKind::Sphere { radius: 120. };
    let sphere = tracer::Object::new(sphere, WorldPosition::new(400., 400., 100.));
    scene.add_object(sphere);

    let sphere = tracer::ObjectKind::Sphere { radius: 90. };
    let sphere = tracer::Object::new(sphere, WorldPosition::new(500., 500., -100.));
    scene.add_object(sphere);
    let buffer = scene.draw();

    let size = scene.size();
    let texture = gl::gen_textures(1)[0];
    gl::bind_texture(gl::TEXTURE_2D, texture);

    gl::tex_image_2d(gl::TEXTURE_2D, 0, gl::RGBA as gl::GLint,
                     size.width as i32, size.height as i32, 0,
                     gl::RGBA, gl::UNSIGNED_BYTE, Some(&buffer));
    gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as gl::GLint);
    gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as gl::GLint);
    gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as gl::GLint);
    gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as gl::GLint);
    gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as gl::GLint);

    let program = gl::create_program();
    let vshader = gl::create_shader(gl::VERTEX_SHADER);

    gl::shader_source(vshader, &[VERTEX_SHADER_SOURCE]);
    gl::compile_shader(vshader);

    println!("VShader: {}", gl::get_shader_info_log(vshader));

    let fshader = gl::create_shader(gl::FRAGMENT_SHADER);
    gl::shader_source(fshader, &[FRAGMENT_SHADER_SOURCE]);
    gl::compile_shader(fshader);

    println!("FShader: {}", gl::get_shader_info_log(fshader));

    gl::attach_shader(program, vshader);
    gl::attach_shader(program, fshader);

    gl::link_program(program);
    println!("program: {}", gl::get_program_info_log(program));
    gl::validate_program(program);
    println!("program: {}", gl::get_program_info_log(program));

    gl::use_program(program);


    let vao = gl::gen_vertex_arrays(1)[0];
    gl::bind_vertex_array(vao);

    let gl_buffer = gl::gen_buffers(1)[0];
    gl::bind_buffer(gl::ARRAY_BUFFER, gl_buffer);

    let data: [f32; 12] = [
        -1.0, 1.0,
         1.0, 1.0,
        -1.0, -1.0,
        -1.0, -1.0,
         1.0, 1.0,
         1.0, -1.0,
    ];

    gl::buffer_data(gl::ARRAY_BUFFER, &data, gl::STATIC_DRAW);

    let pos = gl::get_attrib_location(program, "vPosition");
    gl::enable_vertex_attrib_array(pos as gl::GLuint);
    gl::vertex_attrib_pointer(0, 2, gl::FLOAT, false, 0, 0);
    let tex_uniform = gl::get_uniform_location(program, "uTexture");
    gl::uniform_1i(tex_uniform, 0);
    gl::active_texture(gl::TEXTURE0);

    for event in window.wait_events() {
        gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        gl::draw_arrays(gl::TRIANGLES, 0, 6);
        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => {},
        }
    }

    gl::delete_textures(&[texture]);
}
