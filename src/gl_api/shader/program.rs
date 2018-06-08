use gl_api::buffer::ShaderStorageBuffer;
use gl_api::uniform::BoundUniform;
use gl_api::shader::shader::ShaderError;
use gl_api::shader::shader::ShaderResult;
use std::path::Path;
use gl_api::shader::shader::ShaderType;
use gl_api::error::GlResult;
use gl_api::shader::shader::Shader;
use gl_api::layout::VertexAttribute;
use std::marker::PhantomData;
use super::super::error::GlError;
use super::shader::CompiledShader;
use gl;
use gl::types::*;
use gl_api::uniform::Uniform;

pub struct StorageBindPoint<A> {
    buffer: ShaderStorageBuffer<A>,
    bind_point: u32,
}
impl<A> ::std::ops::Deref for StorageBindPoint<A> {
    type Target = ShaderStorageBuffer<A>;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
impl<A> ::std::ops::DerefMut for StorageBindPoint<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

pub struct UniformBlockBuilder<'p> {
    program: &'p RawLinkedProgram,
    buffer_bind_point: u32,
}

#[derive(Clone, Debug)]
pub enum UniformError {
    NameError(String),
}

impl<'p> UniformBlockBuilder<'p> {
    pub fn uniform<U: BoundUniform>(&self, name: &str) -> Result<Uniform<U>, UniformError> {
        self.program.0.bind();
        unsafe {
            use std::ffi::CString;
            let c_string = CString::new(name).unwrap();
            // UNWRAP: program ID is valid, and the program has been successfully linked
            let location = gl_call!(GetUniformLocation(self.program.0.id, c_string.as_ptr())).unwrap();
            if location == -1 {
                Err(UniformError::NameError(c_string.into_string().unwrap_or_default()))
            } else {
                Ok(Uniform::new(location))
            }
        }
    }

    pub fn shader_storage<A: VertexAttribute>(&mut self, name: &str) -> Result<ShaderStorageBuffer<A>, UniformError> {
        self.program.0.bind();
        unsafe {
            use std::ffi::CString;
            let c_string = CString::new(name).unwrap();
            let bind_point = self.buffer_bind_point;
            let ssbo = ShaderStorageBuffer::new(bind_point);
            ssbo.bind();
            let block_index = gl_call!(GetProgramResourceIndex(
                self.program.0.id,
                ::gl::SHADER_STORAGE_BLOCK,
                c_string.as_ptr())
            ).unwrap();
            if block_index == gl::INVALID_INDEX {
                // TODO: better error? This returns a useless empty string if
                // the C string was invalid in some way.
                Err(UniformError::NameError(c_string.into_string().unwrap_or_default()))
            } else {
                gl_call!(ShaderStorageBlockBinding(self.program.0.id, block_index, bind_point)).unwrap();
                self.buffer_bind_point += 1;
                Ok(ssbo)
            }
        }
    }
}

pub struct ProgramBuilder {
    program: RawProgram,
    vertex: Shader,
    fragment: Shader,
    geometry: Option<Shader>,
    tess: Option<(Shader, Shader)>,
}

impl ProgramBuilder {
    pub fn new(vertex: Shader, fragment: Shader) -> Option<Self> {
        Some(ProgramBuilder {
            program: RawProgram::new()?, vertex, fragment,
            geometry: None,
            tess: None,
        })
    }

    pub fn with_geometry(mut self, shader: Shader) -> Self {
        assert_eq!(shader.shader_type, ShaderType::Geometry);
        self.geometry = Some(shader);
        self
    }

    pub fn with_tesselation(mut self, tess_control: Shader, tess_eval: Shader) -> Self {
        assert_eq!(tess_control.shader_type, ShaderType::TessControl);
        assert_eq!(tess_eval.shader_type, ShaderType::TessEvaluation);
        self.tess = Some((tess_control, tess_eval));
        self
    }

    pub fn build<I, E, F: Fn(UniformBlockBuilder) -> Result<E, ProgramError>>(self, func: F) -> Result<Program<I, E>, ProgramError> {
        // Attach all the shaders. Not sure if they have to be attached in order
        // or not, so I'm going to assume for now that they do.
        self.program.attach_shader(self.vertex.compile()?);
        if let Some((tess_control, tess_eval)) = self.tess {
            self.program.attach_shader(tess_control.compile()?);
            self.program.attach_shader(tess_eval.compile()?);
        }
        if let Some(geometry) = self.geometry {
            self.program.attach_shader(geometry.compile()?);
        }
        self.program.attach_shader(self.fragment.compile()?);
        // Link the program and build the user-defined uniform interface. We
        // have to do it here in a closure because we can't access the linked
        // program before this point, and we need the uniform interface for the
        // program! We parameterize types on the function here because this is
        // where the actual verification for the types happens.
        // TODO: make sure input type is correct.
        let raw = self.program.link()?;
        let environment = func(UniformBlockBuilder { program: &raw, buffer_bind_point: 0 })?;
        Ok(Program {
            raw, environment, _marker: PhantomData,
        })
    }
}

pub struct Program<I, E> {
    raw: RawLinkedProgram,
    environment: E,
    _marker: PhantomData<I>,
}

impl<In: VertexAttribute, Env> Program<In, Env> {
    pub fn env_mut(&mut self) -> &mut Env {
        &mut self.environment
    }

    pub fn bind(&self) {
        self.raw.0.bind();
    }

    // TODO: Remove and lift to `Env`
    // pub fn 
}

#[derive(Debug)]
pub struct RawProgram {
    id: GLuint,
    _marker: ::std::marker::PhantomData<*mut ()>,
}

impl RawProgram {
    crate fn new() -> Option<Self> {
        // UNWRAP: this function never sets an error state
        let id = unsafe { gl_call!(CreateProgram()).unwrap() };
        match id {
            0 => None,
            id => Some(RawProgram {
                id,
                _marker: ::std::marker::PhantomData,
            }),
        }
    }

    crate fn bind(&self) {
        // glUseProgram fails, even though program validation succeeds, and using the program
        // seems to bind it just fine... Smells like a driver bug to me.
        unsafe {
            let _ = gl_call!(UseProgram(self.id));
        }
    }

    crate fn attach_shader(&self, shader: CompiledShader) {
        self.bind();
        unsafe {
            gl_call!(AttachShader(self.id, shader.shader.id)).unwrap();
        }
    }

    crate fn link(self) -> Result<RawLinkedProgram, ProgramError> {
        self.bind();
        unsafe {
            assert!(self.id != 0);
            gl_call!(LinkProgram(self.id))?;
            check_program_status(self.id, gl::LINK_STATUS)?;
            gl_call!(ValidateProgram(self.id))?;
            check_program_status(self.id, gl::VALIDATE_STATUS)?;
        }
        Ok(RawLinkedProgram(self))
    }
}

#[derive(Debug)]
pub struct RawLinkedProgram(RawProgram);

#[derive(Debug)]
pub enum ProgramError {
    Uniform(UniformError),
    Other(String),
    Shader(ShaderError),
    Gl(GlError),
}

impl From<::gl_api::error::GlError> for ProgramError {
    fn from(err: ::gl_api::error::GlError) -> Self {
        ProgramError::Gl(err)
    }
}

impl From<ShaderError> for ProgramError {
    fn from(err: ShaderError) -> Self {
        ProgramError::Shader(err)
    }
}

impl From<UniformError> for ProgramError {
    fn from(err: UniformError) -> Self {
        ProgramError::Uniform(err)
    }
}

fn check_program_status(id: GLuint, ty: GLenum) -> Result<(), ProgramError> {
    let mut status = 1;
    unsafe {
        gl_call!(GetProgramiv(id, ty, &mut status)).unwrap();
    }

    if status == 0 {
        Err(ProgramError::Other(
            program_info_log(id).unwrap_or(String::new()),
        ))
    } else {
        Ok(())
    }
}

fn program_info_log(id: GLuint) -> Option<String> {
    unsafe {
        let mut length = 0;
        gl_call!(GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut length)).unwrap();
        if length == 0 {
            None
        } else {
            let mut buffer = Vec::<u8>::with_capacity(length as usize);
            gl_call!(GetProgramInfoLog(
                id,
                length,
                ::std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut i8
            )).unwrap();
            buffer.set_len((length - 1) as usize);

            Some(String::from_utf8(buffer).expect("Program info log was not UTF-8"))
        }
    }
}
