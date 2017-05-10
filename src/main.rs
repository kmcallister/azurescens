#[macro_use]
extern crate glium;
extern crate time;
extern crate image;

use std::mem;
use std::io;
use std::io::Write;

use glium::{DisplayBuild, Program, Surface, VertexBuffer, IndexBuffer};
use glium::texture::{Texture2d, RawImage2d, ClientFormat};
use glium::index::PrimitiveType;
use glium::glutin::{Event, ElementState, VirtualKeyCode, WindowBuilder};
use glium::glutin::{Touch, TouchPhase};
use glium::backend::Facade;
use glium::backend::glutin_backend::GlutinFacade;
use glium::uniforms::{Sampler, MagnifySamplerFilter};

// Our vertices are boring. We only ever draw 2 triangles
// covering the whole surface.
#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 2],
}

implement_vertex!(Vertex, pos);

fn whole_surface_triangles<F>(facade: &F)
    -> (VertexBuffer<Vertex>, IndexBuffer<u8>)
    where F: Facade,
{
    let vertices = [
        Vertex { pos: [-1.0, -1.0] },
        Vertex { pos: [-1.0,  1.0] },
        Vertex { pos: [ 1.0,  1.0] },
        Vertex { pos: [ 1.0, -1.0] },
    ];

    let indices = [
        0, 1, 2,
        2, 3, 0,
    ];

    let vertex_buffer = VertexBuffer::new(facade, &vertices).unwrap();
    let index_buffer = IndexBuffer::new(facade,
                                        PrimitiveType::TrianglesList,
                                        &indices).unwrap();

    (vertex_buffer, index_buffer)
}


// Size in pixels for the feedback textures.
#[cfg(target_os = "android")]
const FEEDBACK_TEXTURE_SIZE: u32 = 1024;

#[cfg(not(target_os = "android"))]
const FEEDBACK_TEXTURE_SIZE: u32 = 2048;

// Create a feedback texture.
fn feedback_texture<F>(facade: &F) -> Texture2d
    where F: Facade,
{
    // Returns a texture with undefined contents.
    Texture2d::empty(facade,
                     FEEDBACK_TEXTURE_SIZE,
                     FEEDBACK_TEXTURE_SIZE).unwrap()
}

// Shaders.
#[macro_use]
mod shader_loader;

// Trivial vertex shader.
shader_loader!(vertex_shader_src, "shaders/vertex.glsl");

// Simple fragment shader, used to copy a texture to the screen.
shader_loader!(blit_shader_src, "shaders/blit.glsl");

// Fragment shader which runs video feedback between two textures.
shader_loader!(feedback_shader_src, "shaders/feedback.glsl");


// The screen and the feedback textures are interpreted as
// subsets of the complex plane:
//
//     (-SCALE to SCALE) + i*(-SCALE to SCALE)
//
const SCALE: f32 = 1.4;

// Convert a window position in pixels to a complex number.
//
// TODO: Is it slow to look up the size on every mouse move?
fn window_px_to_z(facade: &GlutinFacade, (x, y): (f32, f32))
    -> (f32, f32)
{
    let window = facade.get_window().unwrap();
    let (sx, sy) = window.get_inner_size().unwrap();

    ((x / ((sx-1) as f32) * 2.0 - 1.0) * SCALE,
     (y / ((sy-1) as f32) * 2.0 - 1.0) * SCALE)
}


// Take a screenshot.
fn screenshot(tex: &Texture2d) {
    let raw_image: RawImage2d<u8> = tex.read();
    assert_eq!(raw_image.format, ClientFormat::U8U8U8U8);

    let image: image::ImageBuffer<image::Rgba<u8>, &[u8]>
        = image::ImageBuffer::from_raw(
            FEEDBACK_TEXTURE_SIZE, FEEDBACK_TEXTURE_SIZE,
            &*raw_image.data).unwrap();

    let path_string = format!("az_shot_{}.png", time::precise_time_ns());
    match image.save(&path_string) {
        Ok(()) => println!("Saved screenshot {}", path_string),

        Err(e) => {
            let _ = write!(&mut io::stderr(),
                           "\nFAILED to save image {}: {}\n\n",
                           path_string, e);
        }
    }
}


#[cfg(target_os = "android")]
fn request_gl_version(bld: WindowBuilder) -> WindowBuilder {
    use glium::glutin::{GlRequest, Api};
    bld.with_gl(GlRequest::Specific(Api::OpenGlEs, (3, 1)))
}

#[cfg(not(target_os = "android"))]
fn request_gl_version(bld: WindowBuilder) -> WindowBuilder {
    bld
}

fn main() {
    // Create a glutin / glium window.
    let display_builder = WindowBuilder::new()
        .with_title("a z u r e s c e n s".to_owned())
        .with_vsync();

    let display = request_gl_version(display_builder).build_glium().unwrap();

    println!("OpenGL {:?}", display.get_opengl_version());
    println!("GLSL   {:?}", display.get_supported_glsl_version());

    // Create two textures for feedback.
    let mut read_texture  = feedback_texture(&display);
    let mut write_texture = feedback_texture(&display);

    // Prepare to draw triangles which cover a whole surface.
    let (vertex_buffer, index_buffer) = whole_surface_triangles(&display);
    let draw_params = Default::default();

    // Prepare shader programs.
    let vertex_shader_src = vertex_shader_src();

    let blit_program = Program::from_source(&display,
        &vertex_shader_src, &blit_shader_src(), None).unwrap();

    let feedback_program = Program::from_source(&display,
        &vertex_shader_src, &feedback_shader_src(), None).unwrap();

    // Clear the screen.
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 1.0);
    target.finish().unwrap();

    // Initial value for the complex parameter 'c'.
    let mut param_c = (0.3, 0.3);

    // FPS tracking.
    let mut last_frame_time = time::precise_time_ns();
    let mut last_fps_output_time = last_frame_time;
    let mut smoothed_fps = 0.0;
    let fps_smoothing = 0.9;  // amount to keep for EWMA

    loop {
        // Run video feedback from one texture into the other.
        // We do this twice before drawing to the screen.
        // The reason is that each frame inverts colors and
        // we want to avoid a distracting blinky effect.
        for _ in 0..2 {
            {
                let uniforms = uniform! {
                    src: Sampler::new(&read_texture)
                             .magnify_filter(MagnifySamplerFilter::Nearest),
                    scale: SCALE,
                    param_c: param_c,
                    param_t: time::precise_time_s() as f32,
                };

                let mut target = write_texture.as_surface();
                target.draw(&vertex_buffer, &index_buffer, &feedback_program,
                            &uniforms, &draw_params).unwrap();
            }

            mem::swap(&mut read_texture, &mut write_texture);
        }

        // Draw the most recently written texture to the screen.
        let uniforms = uniform! {
            src: &read_texture,
        };

        let mut target = display.draw();
        target.draw(&vertex_buffer, &index_buffer, &blit_program,
                    &uniforms, &draw_params).unwrap();
        target.finish().unwrap();

        // Update FPS.
        let this_frame_time = time::precise_time_ns();
        let instant_fps = 1e9 / ((this_frame_time - last_frame_time) as f32);
        smoothed_fps = fps_smoothing * smoothed_fps
                     + (1.0-fps_smoothing)*instant_fps;
        last_frame_time = this_frame_time;

        if (this_frame_time - last_fps_output_time) >= 5_000_000_000 {
            println!("Frames per second: {:7.2}", smoothed_fps);
            last_fps_output_time = this_frame_time;
        }

        // Handle events.
        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,

                Event::MouseMoved(x, y) => {
                    // Update the parameter 'c' according to
                    // the mouse position.
                    param_c = window_px_to_z(&display, (x as f32, y as f32));
                }

                Event::Touch(t) => match t {
                      Touch { phase: TouchPhase::Started, location: (x, y), .. }
                    | Touch { phase: TouchPhase::Moved,   location: (x, y), .. } => {
                        param_c = window_px_to_z(&display, (x as f32, y as f32));
                    }

                    _ => (),
                },

                Event::KeyboardInput(ElementState::Pressed,
                                     _, Some(kc)) => match kc {
                    VirtualKeyCode::S => screenshot(&read_texture),
                    _ => (),
                },

                _ => (),
            }
        }
    }
}
