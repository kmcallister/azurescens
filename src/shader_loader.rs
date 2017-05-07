// Load shaders, as GLSL source code.

use std::borrow::Cow;

pub type ShaderSrc = Cow<'static, str>;

#[cfg(feature = "dynamic-shaders")]
pub fn load_dynamic_shader(which: &str) -> ShaderSrc {
    use std::io;
    use std::io::Write;
    use std::path::PathBuf;

    let mut path = PathBuf::from("src/");
    path.push(which);

    let inner = || -> Result<ShaderSrc, io::Error> {
        use std::io::prelude::*;
        use std::fs::File;

        let mut file = File::open(&path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        Ok(Cow::Owned(buffer))
    };

    match inner() {
        Ok(s) => s,
        Err(e) => {
            let _ = write!(&mut io::stderr(),
                           "\nERROR: Could not load shader {}\n\n",
                           path.to_string_lossy());
            panic!("{}", e)
        }
    }
}

macro_rules! shader_loader {
    ($name:ident, $path:expr) => {
        #[cfg(not(feature = "dynamic-shaders"))]
        fn $name() -> shader_loader::ShaderSrc {
            use std::borrow::Cow;
            Cow::Borrowed(include_str!($path))
        }

        #[cfg(feature = "dynamic-shaders")]
        fn $name() -> shader_loader::ShaderSrc {
            shader_loader::load_dynamic_shader($path)
        }
    }
}
