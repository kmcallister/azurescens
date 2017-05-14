#![deny(warnings)]

#[macro_use]
extern crate glium;
extern crate time;
extern crate image;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate tunapanel;

use std::mem;
use std::thread;
use std::sync::{Arc, Mutex};
use std::default::Default;

use glium::{DisplayBuild, Program, Surface, VertexBuffer, IndexBuffer};
use glium::texture::Texture2d;
use glium::index::PrimitiveType;
use glium::glutin::{Event, ElementState, VirtualKeyCode, WindowBuilder};
use glium::glutin::{Touch, TouchPhase};
use glium::backend::Facade;
use glium::backend::glutin_backend::GlutinFacade;
use glium::uniforms::{Sampler, MagnifySamplerFilter};

use params::Params;
use fps::FPSTracker;
use screenshot::screenshot;

mod params;
mod fps;
mod screenshot;

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
    // Create params struct.
    let param_shared = Arc::new(Mutex::new(Params::default()));
    let param_for_thread = param_shared.clone();

    // Start up control panel.
    thread::spawn(|| {
        tunapanel::serve(move |p: Params| {
            *param_for_thread.lock().unwrap() = p;
        }).unwrap();
    });

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
    let mut fps = FPSTracker::new();

    loop {
        // Run video feedback from one texture into the other.
        // We do this twice before drawing to the screen.
        // The reason is that each frame inverts colors and
        // we want to avoid a distracting blinky effect.
        for _ in 0..2 {
            {
                let uniforms = {
                    let params = param_shared.lock().unwrap();

                    uniform! {
                        src_near: Sampler::new(&read_texture)
                                   .magnify_filter(MagnifySamplerFilter::Nearest),
                        src_lin: Sampler::new(&read_texture)
                                   .magnify_filter(MagnifySamplerFilter::Linear),
                        scale: SCALE,
                        param_c: param_c,
                        param_t: time::precise_time_s() as f32,
                        invert: params.invert,
                        permute_colors: params.permute_colors,
                        fade: params.fade,
                        color_cycle_rate: params.color_cycle_rate,
                        mix_linear: params.mix_linear,
                        mix_linear_tv: params.mix_linear_tv,
                    }
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
        fps.tick();

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
