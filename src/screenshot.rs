use std::io;
use std::io::Write;
use glium::texture::{Texture2d, RawImage2d, ClientFormat};
use time;
use image;

use FEEDBACK_TEXTURE_SIZE;

// Take a screenshot.
pub fn screenshot(tex: &Texture2d) {
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
