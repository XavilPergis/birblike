use super::error::GlResult;
use gl::types::*;
use gl_api::buffer::VertexBuffer;
use gl_api::layout::VertexAttribute;

#[derive(Debug)]
pub struct VertexArray {
    crate id: GLuint,
    index: usize,
    _marker: ::std::marker::PhantomData<*mut ()>,
}

impl VertexArray {
    pub fn new() -> Self {
        let mut id = 0;
        // UNWRAP: Can only fail if count is negative
        unsafe {
            gl_call!(GenVertexArrays(1, &mut id)).unwrap();
        }
        VertexArray {
            id,
            index: 0,
            _marker: ::std::marker::PhantomData,
        }
    }

    pub fn bind(&self) {
        // UNWRAP: our ID should always be valid
        unsafe {
            gl_call!(BindVertexArray(self.id)).unwrap();
        }
    }

    // NOTE: need explicit lifetimes here because the buffer needs to outlive
    // `self`
    pub fn add_buffer<'s, 'b: 's, T: VertexAttribute>(&'s mut self, buffer: &'b VertexBuffer<T>) -> GlResult<()> {
        self.bind();
        buffer.bind();

        self.index += T::define_attribs(self.index as u32, 0)? as usize;

        Ok(())
    }
}
