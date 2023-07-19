use std::{process::Command, path::{PathBuf, Path}, fs};
use anyhow::{Result, Ok};
use log::*;

pub struct ShaderByteCode {
    pub vertex: Vec<u8>,
    pub fragment: Vec<u8>
}

#[derive(Clone, Debug)]
pub struct ShaderManager {
    shaders_dir: PathBuf
}

impl ShaderManager {
    pub fn create() -> Result<Self> {
        let shaders_dir = find_shaders_path()?;

        Ok( Self { shaders_dir })
    }

    fn convert_shaders(&self) {
        let result = Command::new("sh")
            .current_dir(&self.shaders_dir)
            .arg("./compile.sh")
            .output();
    
        if result.is_err() {
            error!("Shader compiler error:\n{:#?}", result.as_ref().err());
        }
    
        let output = result.unwrap();
    
        if output.stderr.is_empty() == false {
            info!("Shader compilation failed: {:#?}", output);
        }
    }

    pub fn get_shaders_bytecode(&self) -> Result<ShaderByteCode> {
        self.convert_shaders();

        let vert = fs::read(self.shaders_dir.join("vert.spv"))?;
        let frag = fs::read(self.shaders_dir.join("frag.spv"))?;

        Ok( ShaderByteCode { vertex: vert, fragment: frag } )
    }


}

fn find_shaders_path() -> Result<PathBuf> {
    let result = std::env::var("SHADER_DIR");

    let dir: PathBuf;

    if result.is_ok() {
        let path = result.unwrap();
        dir = Path::new(&path).to_path_buf()
    } else {
        error!("SHADER_DIR env variable is not set, defaulting to ./shaders/");
        dir = std::env::current_dir()?.join("shaders/");
    }
    
    assert!(dir.exists(), "The dir containing the shaders: '{:?}' doesn't exist, exiting.", dir);
    info!("Shaders dir: {:?}", dir);

    Ok(dir)
}