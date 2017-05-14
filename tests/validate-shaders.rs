#[macro_use]
extern crate log;

extern crate env_logger;
extern crate tempfile;

use std::process::{Command, Stdio, ExitStatus};
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::fs::File;
use std::env;
use tempfile::NamedTempFile;

#[derive(Debug)]
enum Error {
    IoError(io::Error),
    CommandFailed(ExitStatus, String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

fn glslang_validator(args: &[&str]) -> Result<String, Error> {
    let child = Command::new("glslangValidator")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(args)
        .spawn()?;

    let res = child.wait_with_output()?;
    let mut output = String::new();
    output.push_str(&String::from_utf8_lossy(&res.stdout));
    output.push_str(&String::from_utf8_lossy(&res.stderr));

    if res.status.success() {
        Ok(output)
    } else {
        Err(Error::CommandFailed(res.status, output))
    }
}

#[test]
fn validate_shaders() {
    env_logger::init().unwrap();

    // Check for existence of glslangValidator.
    match glslang_validator(&["-v"]) {
        Err(_) => {
            error!("Could not run glslangValidator. Skipping shader validation.");
            return;
        }

        Ok(out) => {
            print!("{}", out);
        }
    }

    let shaders_dir = {
        let mut s = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        s.push("src");
        s.push("shaders");
        s
    };

    // Validate each shader in each mode.
    for &(ty, filename) in &[
        ("vert", "vertex.glsl"),
        ("frag", "blit.glsl"),
        ("frag", "feedback.glsl"),
    ] {
        let mut path = shaders_dir.clone();
        path.push(filename);

        let shader_in = {
            let mut buf = String::new();
            let mut file = File::open(&path).unwrap();
            file.read_to_string(&mut buf).unwrap();
            buf
        };

        for vers in &["#version 150", "#version 300 es"] {
            info!("Validating {} shader {} with {}",
                ty, filename, vers);

            let mut temp = NamedTempFile::new().unwrap();
            write!(&mut temp, "{}\n\n{}", vers, shader_in).unwrap();

            if let Err(e) = glslang_validator(&[
                "-S",
                ty,
                temp.path().to_str().expect("non-UTF-8 file path")
            ]) {
                error!("Failed to validate {} shader {} with {}",
                    ty, filename, vers);

                match e {
                    Error::IoError(e) => error!("IO error: {:?}", e),
                    Error::CommandFailed(s, out) => {
                        error!("glslValidator exited with code {}. Output:\n{}",
                            s, out);
                    }
                }

                panic!("Shader validation failed.");
            }
        }
    }
}
