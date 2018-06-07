use std::io;
use std::path::Path;

pub mod program;
pub mod shader;

use self::program::*;
use self::shader::*;

#[derive(Debug)]
pub enum PipelineError {
    Shader(ShaderError),
    Io(io::Error),
    ProgramCreation,
}

impl From<ShaderError> for PipelineError {
    fn from(err: ShaderError) -> Self {
        PipelineError::Shader(err)
    }
}

impl From<io::Error> for PipelineError {
    fn from(err: io::Error) -> Self {
        PipelineError::Io(err)
    }
}

pub fn simple_pipeline<P1: AsRef<Path>, P2: AsRef<Path>>(
    vert: P1,
    frag: P2,
) -> Result<RawLinkedProgram, PipelineError> {
    let program = RawProgram::new().ok_or(PipelineError::ProgramCreation)?;
    let vert_shader = Shader::new(ShaderType::Vertex)?;
    let frag_shader = Shader::new(ShaderType::Fragment)?;

    vert_shader.source_from_file(vert)?;
    frag_shader.source_from_file(frag)?;

    program.attach_shader(vert_shader.compile()?);
    program.attach_shader(frag_shader.compile()?);

    Ok(program.link().unwrap())
}
