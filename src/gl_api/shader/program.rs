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
        self.program.bind();
        unsafe {
            use std::ffi::CString;
            let c_string = CString::new(name).unwrap();
            // UNWRAP: program ID is valid, and the program has been successfully linked
            let location = gl_call!(GetUniformLocation(self.program.program.id, c_string.as_ptr())).unwrap();
            if location == -1 {
                Err(UniformError::NameError(c_string.into_string().unwrap_or_default()))
            } else {
                Ok(Uniform::new(location))
            }
        }
    }

    pub fn shader_storage<A: VertexAttribute>(&mut self, name: &str) -> Result<ShaderStorageBuffer<A>, UniformError> {
        self.program.bind();
        unsafe {
            use std::ffi::CString;
            let c_string = CString::new(name).unwrap();
            let bind_point = self.buffer_bind_point;
            let ssbo = ShaderStorageBuffer::new(bind_point);
            ssbo.bind();
            let block_index = gl_call!(GetProgramResourceIndex(
                self.program.program.id,
                ::gl::SHADER_STORAGE_BLOCK,
                c_string.as_ptr())
            ).unwrap();
            // TODO: Unwrap
            if block_index == gl::INVALID_INDEX {
                Err(UniformError::NameError(c_string.into_string().unwrap_or_default()))
            } else {
                gl_call!(ShaderStorageBlockBinding(self.program.program.id, block_index, bind_point)).unwrap();
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

// builder.build(|builder|);

pub struct Program<I, E> {
    raw: RawLinkedProgram,
    environment: E,
    _marker: PhantomData<I>,
}

impl<In: VertexAttribute, Env> Program<In, Env> {
    pub fn environment_mut(&mut self) -> &mut Env {
        &mut self.environment
    }

    pub fn bind(&self) {
        self.raw.bind();
    }

    // TODO: Remove and lift to `Env`
    // pub fn 
}

#[derive(Debug)]
pub struct RawProgram {
    id: GLuint,
    _marker: ::std::marker::PhantomData<*mut ()>,
}

#[repr(u32)]
enum AttributeType {
    Float = gl::FLOAT,
    FloatVec2 = gl::FLOAT_VEC2,
    FloatVec3 = gl::FLOAT_VEC3,
    FloatVec4 = gl::FLOAT_VEC4,
    FloatMatrix2 = gl::FLOAT_MAT2,
    FloatMatrix3 = gl::FLOAT_MAT3,
    FloatMatrix4 = gl::FLOAT_MAT4,
    FloatMatrix2x3 = gl::FLOAT_MAT2x3,
    FloatMatrix2x4 = gl::FLOAT_MAT2x4,
    FloatMatrix3x2 = gl::FLOAT_MAT3x2,
    FloatMatrix3x4 = gl::FLOAT_MAT3x4,
    FloatMatrix4x2 = gl::FLOAT_MAT4x2,
    FloatMatrix4x3 = gl::FLOAT_MAT4x3,
    Double = gl::DOUBLE,
    DoubleVec2 = gl::DOUBLE_VEC2,
    DoubleVec3 = gl::DOUBLE_VEC3,
    DoubleVec4 = gl::DOUBLE_VEC4,
    DoubleMatrix2 = gl::DOUBLE_MAT2,
    DoubleMatrix3 = gl::DOUBLE_MAT3,
    DoubleMatrix4 = gl::DOUBLE_MAT4,
    DoubleMatrix2x3 = gl::DOUBLE_MAT2x3,
    DoubleMatrix2x4 = gl::DOUBLE_MAT2x4,
    DoubleMatrix3x2 = gl::DOUBLE_MAT3x2,
    DoubleMatrix3x4 = gl::DOUBLE_MAT3x4,
    DoubleMatrix4x2 = gl::DOUBLE_MAT4x2,
    DoubleMatrix4x3 = gl::DOUBLE_MAT4x3,
    Int = gl::INT,
    IntVec2 = gl::INT_VEC2,
    IntVec3 = gl::INT_VEC3,
    IntVec4 = gl::INT_VEC4,
    UnsignedInt = gl::UNSIGNED_INT,
    UnsignedIntVec2 = gl::UNSIGNED_INT_VEC2,
    UnsignedIntVec3 = gl::UNSIGNED_INT_VEC3,
    UnsignedIntVec4 = gl::UNSIGNED_INT_VEC4,
}

const MAX_NAME_LEN: usize = 64;

struct ActiveAttribute {
    size: usize,
    ty: AttributeType,
    name_len: usize,
    name: [u8; MAX_NAME_LEN],
}

impl ActiveAttribute {
    pub fn name(&self) -> &str {
        ::std::str::from_utf8(&self.name[..self.name_len]).unwrap()
    }
}

impl RawProgram {
    pub fn new() -> Option<Self> {
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

    pub fn bind(&self) {
        // glUseProgram fails, even though program validation succeeds, and using the program
        // seems to bind it just fine... Smells like a driver bug to me.
        unsafe {
            let _ = gl_call!(UseProgram(self.id));
        }
    }

    pub fn attach_shader(&self, shader: CompiledShader) {
        self.bind();
        unsafe {
            gl_call!(AttachShader(self.id, shader.shader.id)).unwrap();
        }
    }

    pub fn link(self) -> Result<RawLinkedProgram, ProgramError> {
        self.bind();
        unsafe {
            assert!(self.id != 0);
            gl_call!(LinkProgram(self.id))?;
            check_program_status(self.id, gl::LINK_STATUS)?;
            gl_call!(ValidateProgram(self.id))?;
            check_program_status(self.id, gl::VALIDATE_STATUS)?;
        }
        Ok(RawLinkedProgram {
            program: self,
            // uniform_cache: HashMap::new(),
        })
    }
}

#[derive(Debug)]
pub struct RawLinkedProgram {
    program: RawProgram,
    // uniform_cache: HashMap<String, UniformLocation>,
}

impl RawLinkedProgram {
    // pub fn set_uniform<U: BoundUniform>(&mut self, name: &str, uniform: &U) {
    //     self.program.bind();
    //     uniform.set(&if let Some(&location) = self.uniform_cache.get(name) {
    //         Uniform::new(location)
    //     } else {
    //         let location = self.get_uniform_location(name);
    //         self.uniform_cache.insert(name.into(), location);
    //         Uniform::new(location)
    //     });
    // }

    // void glGetActiveAttrib(GLuint program​, GLuint index​, GLsizei bufSize​,
    // GLsizei *length​, GLint *size​, GLenum *type​, GLchar *name​);
    fn active_attrib(&self, index: u32) -> ActiveAttribute {
        use std::os::raw::c_char;
        let mut name_buf = [0u8; MAX_NAME_LEN];
        let mut length = 0;
        let mut size = 0;
        let mut ty = 0;
        // gl_call!(GetActiveAttrib(self.program.id, index, MAX_NAME_LEN as i32, &mut length, &mut size, &mut ty, name_buf[..].as_mut_ptr() as *mut c_char)).unwrap();
        // ActiveAttribute {

        // }
        unimplemented!()
    }

    pub fn bind(&self) {
        self.program.bind();
    }

    // fn get_uniform_location(&self, name: &str) -> UniformLocation {
    //     self.program.bind();
    //     unsafe {
    //         use std::ffi::CString;
    //         let c_string = CString::new(name).unwrap();
    //         // UNWRAP: program ID is valid, and the program has been successfully linked
    //         gl_call!(GetUniformLocation(self.program.id, c_string.as_ptr())).unwrap()
    //     }
    // }
}

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