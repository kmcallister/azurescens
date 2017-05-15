// Load shaders, as GLSL source code.

#[cfg(target_os = "android")]
pub static GLSL_VERSION: &'static str = "#version 300 es\n\n";

#[cfg(not(target_os = "android"))]
pub static GLSL_VERSION: &'static str = "#version 150\n\n";

#[cfg(feature = "dynamic-shaders")]
pub fn load_dynamic_shader(which: &str) -> String {
    use std::io;
    use std::path::PathBuf;

    let mut path = PathBuf::from("src/");
    path.push(which);

    let inner = || -> Result<String, io::Error> {
        use std::io::prelude::*;
        use std::fs::File;

        let mut file = File::open(&path)?;
        let mut buffer = String::from(GLSL_VERSION);
        file.read_to_string(&mut buffer)?;

        Ok(buffer)
    };

    match inner() {
        Ok(s) => s,
        Err(e) => {
            error!("Could not load shader {}", path.to_string_lossy());
            panic!("{}", e)
        }
    }
}

macro_rules! shader_loader {
    ($name:ident, $path:expr) => {
        #[cfg(not(feature = "dynamic-shaders"))]
        fn $name() -> String {
            format!("{}{}", shader_loader::GLSL_VERSION, include_str!($path))
        }

        #[cfg(feature = "dynamic-shaders")]
        fn $name() -> String {
            shader_loader::load_dynamic_shader($path)
        }
    }
}
