use gl_api::error::GlError;
use super::error::GlResult;
use gl;
use gl::types::*;
use std::marker::PhantomData;

mod sealed {
    pub trait Sealed {}
}

pub trait BufferTarget: sealed::Sealed {
    const TARGET: GLenum;
}

// Might allow for glBindBufferBase and whatnot in the future
pub trait IndexedTarget: BufferTarget {}

macro_rules! buffer_target {
    ($name:ident : $enum:expr) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
        pub struct $name;
        impl sealed::Sealed for $name {}
        impl BufferTarget for $name {
            const TARGET: GLenum = $enum;
        }
    };

    ($name:ident : indexed $enum:expr) => {
        buffer_target!($name: $enum);
        impl IndexedTarget for $name {}
    };
}

// Buffer targets described in section 6.1 of spec
buffer_target!(Array: gl::ARRAY_BUFFER);
buffer_target!(Element: gl::ELEMENT_ARRAY_BUFFER);
buffer_target!(Uniform: indexed gl::UNIFORM_BUFFER);
buffer_target!(PixelPack: gl::PIXEL_PACK_BUFFER);
buffer_target!(PixelUnpack: gl::PIXEL_UNPACK_BUFFER);
buffer_target!(Query: gl::QUERY_BUFFER);
buffer_target!(ShaderStorage: indexed gl::SHADER_STORAGE_BUFFER);
buffer_target!(Texture: gl::TEXTURE_BUFFER);
buffer_target!(TransformFeedback: indexed gl::TRANSFORM_FEEDBACK_BUFFER);
buffer_target!(AtomicCounter: indexed gl::ATOMIC_COUNTER_BUFFER);

// Values from section 6.2 of spec
/// Usage type for buffers, provided as a performance hint. These values do not affect the behavior
/// of the buffer.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum UsageType {
    /// The data store contents will be specified once by the application, and sourced at most a few times.
    StreamDraw = gl::STREAM_DRAW,
    /// The data store contents will be specified once by reading data from the GL, and queried at most a few times by the application.
    StreamRead = gl::STREAM_READ,
    /// The data store contents will be specified once by reading data from the GL, and sourced at most a few times
    StreamCopy = gl::STREAM_COPY,
    /// The data store contents will be specified once by the application, and sourced many times.
    StaticDraw = gl::STATIC_DRAW,
    /// The data store contents will be specified once by reading data from the GL, and queried many times by the application.
    StaticRead = gl::STATIC_READ,
    /// The data store contents will be specified once by reading data from the GL, and sourced many times.
    StaticCopy = gl::STATIC_COPY,
    /// The data store contents will be respecified repeatedly by the application, and sourced many times.
    DynamicDraw = gl::DYNAMIC_DRAW,
    /// The data store contents will be respecified repeatedly by reading data from the GL, and queried many times by the application.
    DynamicRead = gl::DYNAMIC_READ,
    /// The data store contents will be respecified repeatedly by reading data from the GL, and sourced many times.
    DynamicCopy = gl::DYNAMIC_COPY,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Buffer<T, B: BufferTarget> {
    pub(crate) id: GLuint,
    length: usize,
    _phantom: PhantomData<(*mut T, B)>,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct IndexedBuffer<T, B: IndexedTarget> {
    crate buf: Buffer<T, B>,
    bind_point: GLuint,
}

impl<T, B: BufferTarget> Buffer<T, B> {
    pub fn new() -> Self {
        let mut id = 0;
        // UNWRAP: Could only error if the amount is negative
        unsafe {
            gl_call!(GenBuffers(1, &mut id)).unwrap();
        }
        Buffer {
            id,
            length: 0,
            _phantom: PhantomData,
        }
    }

    pub fn bind(&self) {
        // UNWRAP: Could only error if the buffer type is invalid
        unsafe {
            gl_call!(BindBuffer(B::TARGET, self.id)).unwrap();
        }
    }

    /// Copies data from `data` to the gpu's memory
    pub fn upload(&mut self, data: &[T], usage_type: UsageType) -> GlResult<()> {
        unsafe {
            self.bind();
            self.length = data.len();
            // Could fail if OOM
            gl_call!(BufferData(
                B::TARGET,
                (::std::mem::size_of::<T>() * data.len()) as isize,
                data.as_ptr() as *const _,
                usage_type as GLenum
            ))
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }
}

impl<T, B: BufferTarget> Drop for Buffer<T, B> {
    fn drop(&mut self) {
        unsafe {
            // UNWRAP: can only fail if count is negative, which it isn't
            gl_call!(DeleteBuffers(1, &self.id)).unwrap();
        }
    }
}

impl<T, B: IndexedTarget> IndexedBuffer<T, B> {
    crate fn new(bind_point: GLuint) -> Self {
        IndexedBuffer { buf: Buffer::new(), bind_point }
    }

    pub fn bind(&self) {
        unsafe {
            // TODO: unwrap
            gl_call!(BindBufferBase(B::TARGET, self.bind_point, self.buf.id)).unwrap();
        }
    }

    pub fn upload(&mut self, data: &[T], usage_type: UsageType) -> GlResult<()> {
        self.bind();
        self.buf.upload(data, usage_type)
    }

    pub fn len(&self) -> usize { self.buf.len() }

    pub fn map_mut<'b>(&'b mut self) -> GlResult<Option<BufferMapMut<'b, T, B>>> {
        BufferMapMut::new(&mut self.buf)
    }
}

#[derive(Debug)]
pub enum BufferMapError {
    Gl(GlError),
    ZeroLength,
}

impl From<GlError> for BufferMapError {
    fn from(err: GlError) -> Self { BufferMapError::Gl(err) }
}

pub struct BufferMapMut<'b, T: 'b, B: BufferTarget + 'b> {
    buf: &'b mut Buffer<T, B>,
    mapped: *mut T,
}

impl<'b, T: 'b, B: BufferTarget + 'b> BufferMapMut<'b, T, B> {
    crate fn new(buf: &'b mut Buffer<T, B>) -> GlResult<Option<Self>> {
        unsafe {
            buf.bind();
            let mut mapped = 0;
            let mut ptr = ::std::ptr::null_mut();
            gl_call!(GetBufferParameteriv(B::TARGET, gl::BUFFER_MAPPED, &mut mapped)).unwrap();
            gl_call!(GetBufferPointerv(B::TARGET, gl::BUFFER_MAP_POINTER, &mut ptr)).unwrap();
            assert!(mapped == 0);
            assert!(ptr.is_null());
            assert!(buf.id != 0);
            assert!(buf.len() > 0);
            if buf.len() > 0 {
                let ptr = gl_call!(MapBufferRange(B::TARGET, 0, buf.len() as isize, gl::MAP_READ_BIT | gl::MAP_WRITE_BIT))?;
                Ok(Some(BufferMapMut { buf, mapped: ptr as *mut T }))
            } else { Ok(None) }
        }
    }
}

use std::ops::{Index, IndexMut};

impl<'b, T: 'b, B: BufferTarget + 'b> Index<usize> for BufferMapMut<'b, T, B> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.buf.len());
        unsafe { &*self.mapped.offset(index as isize) }
    }
}

impl<'b, T: 'b, B: BufferTarget + 'b> IndexMut<usize> for BufferMapMut<'b, T, B> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.buf.len());
        unsafe { &mut *self.mapped.offset(index as isize) }
    }
}

impl<'b, T: 'b, B: BufferTarget + 'b> Drop for BufferMapMut<'b, T, B> {
    fn drop(&mut self) {
        unsafe {
            self.buf.bind();
            if gl_call!(UnmapBuffer(B::TARGET)).unwrap() == 0 {
                // The data store is in an undefined state, wat to do???
                // TODO: Do something other than panic.
                // FIXME: NOT GOOD.
                panic!("Buffer data store was corrupted while it was mapped.");
            }
        }
    }
}

pub type VertexBuffer<T> = Buffer<T, Array>;
pub type ElementBuffer<T> = Buffer<T, Element>;
pub type ShaderStorageBuffer<T> = IndexedBuffer<T, ShaderStorage>;
