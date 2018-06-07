use gl::types::GLenum;
use gl;

pub type GlResult<T> = Result<T, GlError>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct GlError {
    code: GLenum,
}

impl GlError {
    fn get_raw() -> GLenum { unsafe { gl::GetError() } }

    pub fn map_value<T>(val: T) -> GlResult<T> {
        match Self::get_raw() {
            0 => Ok(val),
            // GL specification states that it is undefined to issue any GL
            // calls after an out of memory error is received.
            gl::OUT_OF_MEMORY => ::std::process::abort(),
            code => Err(GlError { code }),
        }
    }
}

macro_rules! gl_call {
    ($name:ident($($args:expr),*)) => {{
        $crate::gl_api::error::GlError::map_value(::gl::$name($($args),*))
    }}
}
