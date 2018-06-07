use cgmath::{Vector2, Vector3, Vector4, Matrix2, Matrix3, Matrix4};

pub type UniformLocation = ::gl::types::GLint;

pub struct Uniform<T: ?Sized> {
    crate location: UniformLocation,
    _marker: ::std::marker::PhantomData<T>,
}

impl<T: ?Sized> Uniform<T> {
    crate fn new(location: UniformLocation) -> Self {
        Uniform { location, _marker: ::std::marker::PhantomData }
    }

    pub fn set(&self, value: T) where T: BoundUniform + Sized {
        value.set(self);
    }
}

pub trait BoundUniform {
    fn set(&self, uniform: &Uniform<Self>);
}

use gl_api::error::GlError;

macro_rules! uniform_array {
    ($self:ident, $type:ty => $func:ident($($expr:expr),*)) => (
        impl BoundUniform for [$type] {
            #[inline(always)]
            fn set(&$self, uniform: &Uniform<Self>) {
                unsafe { GlError::map_value(::gl::$func(uniform.location, $($expr,)*)).unwrap() }
            }
        }
    )
}

#[allow(unused_macros)]
macro_rules! uniform {
    // Macro cleanliness means that we can't use `self` in the macro invocation scope
    // without first introducing it into scope there (slightly unfortunate)
    ($self:ident, $type:ty => $func:ident($($expr:expr),*)) => (
        impl BoundUniform for $type {
            #[inline(always)]
            fn set(&$self, uniform: &Uniform<Self>) {
                unsafe { GlError::map_value(::gl::$func(uniform.location, $($expr,)*)).unwrap() }
            }
        }
    )
}

#[allow(unused_macros)]
macro_rules! uniform_vector {
    // Macro cleanliness means that we can't use `self` in the macro invocation scope
    // without first introducing it into scope there (slightly unfortunate)
    ($self:ident, $type:ty => $func:ident($($expr:expr),*)) => (
        impl Uniform for $type {
            #[inline(always)]
            fn set(&$self, uniform: &Uniform<Self>) {
                use ::gl;
                let _res = unsafe { gl::$func(location, $($expr,)*) };
                // println!("{} {:?}", stringify!($type), res);
                let error = unsafe { gl::GetError() };
                if error != 0 { panic!("OpenGL Returned error {}", error); }
            }
        }
    )
}

uniform!(self, f32 => Uniform1f(*self));
uniform!(self, [f32; 1] => Uniform1f(self[0]));
uniform!(self, [f32; 2] => Uniform2f(self[0], self[1]));
uniform!(self, [f32; 3] => Uniform3f(self[0], self[1], self[2]));
uniform!(self, [f32; 4] => Uniform4f(self[0], self[1], self[2], self[3]));
uniform!(self, (f32,) => Uniform1f(self.0));
uniform!(self, (f32, f32) => Uniform2f(self.0, self.1));
uniform!(self, (f32, f32, f32) => Uniform3f(self.0, self.1, self.2));
uniform!(self, (f32, f32, f32, f32) => Uniform4f(self.0, self.1, self.2, self.3));
uniform!(self, Vector2<f32> => Uniform2f(self.x, self.y));
uniform!(self, Vector3<f32> => Uniform3f(self.x, self.y, self.z));
uniform!(self, Vector4<f32> => Uniform4f(self.x, self.y, self.z, self.w));

uniform!(self, f64 => Uniform1d(*self));
uniform!(self, [f64; 1] => Uniform1d(self[0]));
uniform!(self, [f64; 2] => Uniform2d(self[0], self[1]));
uniform!(self, [f64; 3] => Uniform3d(self[0], self[1], self[2]));
uniform!(self, [f64; 4] => Uniform4d(self[0], self[1], self[2], self[3]));
uniform!(self, (f64,) => Uniform1d(self.0));
uniform!(self, (f64, f64) => Uniform2d(self.0, self.1));
uniform!(self, (f64, f64, f64) => Uniform3d(self.0, self.1, self.2));
uniform!(self, (f64, f64, f64, f64) => Uniform4d(self.0, self.1, self.2, self.3));
uniform!(self, Vector2<f64> => Uniform2d(self.x, self.y));
uniform!(self, Vector3<f64> => Uniform3d(self.x, self.y, self.z));
uniform!(self, Vector4<f64> => Uniform4d(self.x, self.y, self.z, self.w));

uniform!(self, i32 => Uniform1i(*self));
uniform!(self, [i32; 1] => Uniform1i(self[0]));
uniform!(self, [i32; 2] => Uniform2i(self[0], self[1]));
uniform!(self, [i32; 3] => Uniform3i(self[0], self[1], self[2]));
uniform!(self, [i32; 4] => Uniform4i(self[0], self[1], self[2], self[3]));
uniform!(self, (i32,) => Uniform1i(self.0));
uniform!(self, (i32, i32) => Uniform2i(self.0, self.1));
uniform!(self, (i32, i32, i32) => Uniform3i(self.0, self.1, self.2));
uniform!(self, (i32, i32, i32, i32) => Uniform4i(self.0, self.1, self.2, self.3));
uniform!(self, Vector2<i32> => Uniform2i(self.x, self.y));
uniform!(self, Vector3<i32> => Uniform3i(self.x, self.y, self.z));
uniform!(self, Vector4<i32> => Uniform4i(self.x, self.y, self.z, self.w));

uniform!(self, u32 => Uniform1ui(*self));
uniform!(self, [u32; 1] => Uniform1ui(self[0]));
uniform!(self, [u32; 2] => Uniform2ui(self[0], self[1]));
uniform!(self, [u32; 3] => Uniform3ui(self[0], self[1], self[2]));
uniform!(self, [u32; 4] => Uniform4ui(self[0], self[1], self[2], self[3]));
uniform!(self, (u32,) => Uniform1ui(self.0));
uniform!(self, (u32, u32) => Uniform2ui(self.0, self.1));
uniform!(self, (u32, u32, u32) => Uniform3ui(self.0, self.1, self.2));
uniform!(self, (u32, u32, u32, u32) => Uniform4ui(self.0, self.1, self.2, self.3));
uniform!(self, Vector2<u32> => Uniform2ui(self.x, self.y));
uniform!(self, Vector3<u32> => Uniform3ui(self.x, self.y, self.z));
uniform!(self, Vector4<u32> => Uniform4ui(self.x, self.y, self.z, self.w));

uniform_array!(self, f32 => Uniform1fv(self.len() as i32, self.as_ptr()));
uniform_array!(self, [f32; 1] => Uniform1fv(self.len() as i32, self.as_ptr() as *const f32));
uniform_array!(self, [f32; 2] => Uniform2fv(2 * self.len() as i32, self.as_ptr() as *const f32));
uniform_array!(self, [f32; 3] => Uniform3fv(3 * self.len() as i32, self.as_ptr() as *const f32));
uniform_array!(self, [f32; 4] => Uniform4fv(4 * self.len() as i32, self.as_ptr() as *const f32));
uniform_array!(self, (f32,) => Uniform1fv(self.len() as i32, self.as_ptr() as *const f32));
uniform_array!(self, (f32, f32) => Uniform2fv(2 * self.len() as i32, self.as_ptr() as *const f32));
uniform_array!(self, (f32, f32, f32) => Uniform3fv(3 * self.len() as i32, self.as_ptr() as *const f32));
uniform_array!(self, (f32, f32, f32, f32) => Uniform4fv(4 * self.len() as i32, self.as_ptr() as *const f32));
uniform_array!(self, Vector2<f32> => Uniform2fv(2 * self.len() as i32, self.as_ptr() as *const f32));
uniform_array!(self, Vector3<f32> => Uniform3fv(3 * self.len() as i32, self.as_ptr() as *const f32));
uniform_array!(self, Vector4<f32> => Uniform4fv(4 * self.len() as i32, self.as_ptr() as *const f32));

uniform_array!(self, f64 => Uniform1dv(self.len() as i32, self.as_ptr()));
uniform_array!(self, [f64; 1] => Uniform1dv(self.len() as i32, self.as_ptr() as *const f64));
uniform_array!(self, [f64; 2] => Uniform2dv(2 * self.len() as i32, self.as_ptr() as *const f64));
uniform_array!(self, [f64; 3] => Uniform3dv(3 * self.len() as i32, self.as_ptr() as *const f64));
uniform_array!(self, [f64; 4] => Uniform4dv(4 * self.len() as i32, self.as_ptr() as *const f64));
uniform_array!(self, (f64,) => Uniform1dv(self.len() as i32, self.as_ptr() as *const f64));
uniform_array!(self, (f64, f64) => Uniform2dv(2 * self.len() as i32, self.as_ptr() as *const f64));
uniform_array!(self, (f64, f64, f64) => Uniform3dv(3 * self.len() as i32, self.as_ptr() as *const f64));
uniform_array!(self, (f64, f64, f64, f64) => Uniform4dv(4 * self.len() as i32, self.as_ptr() as *const f64));
uniform_array!(self, Vector2<f64> => Uniform2dv(2 * self.len() as i32, self.as_ptr() as *const f64));
uniform_array!(self, Vector3<f64> => Uniform3dv(3 * self.len() as i32, self.as_ptr() as *const f64));
uniform_array!(self, Vector4<f64> => Uniform4dv(4 * self.len() as i32, self.as_ptr() as *const f64));

uniform_array!(self, i32 => Uniform1iv(self.len() as i32, self.as_ptr()));
uniform_array!(self, [i32; 1] => Uniform1iv(self.len() as i32, self.as_ptr() as *const i32));
uniform_array!(self, [i32; 2] => Uniform2iv(2 * self.len() as i32, self.as_ptr() as *const i32));
uniform_array!(self, [i32; 3] => Uniform3iv(3 * self.len() as i32, self.as_ptr() as *const i32));
uniform_array!(self, [i32; 4] => Uniform4iv(4 * self.len() as i32, self.as_ptr() as *const i32));
uniform_array!(self, (i32,) => Uniform1iv(self.len() as i32, self.as_ptr() as *const i32));
uniform_array!(self, (i32, i32) => Uniform2iv(2 * self.len() as i32, self.as_ptr() as *const i32));
uniform_array!(self, (i32, i32, i32) => Uniform3iv(3 * self.len() as i32, self.as_ptr() as *const i32));
uniform_array!(self, (i32, i32, i32, i32) => Uniform4iv(4 * self.len() as i32, self.as_ptr() as *const i32));
uniform_array!(self, Vector2<i32> => Uniform2iv(2 * self.len() as i32, self.as_ptr() as *const i32));
uniform_array!(self, Vector3<i32> => Uniform3iv(3 * self.len() as i32, self.as_ptr() as *const i32));
uniform_array!(self, Vector4<i32> => Uniform4iv(4 * self.len() as i32, self.as_ptr() as *const i32));

uniform_array!(self, u32 => Uniform1uiv(self.len() as i32, self.as_ptr()));
uniform_array!(self, [u32; 1] => Uniform1uiv(self.len() as i32, self.as_ptr() as *const u32));
uniform_array!(self, [u32; 2] => Uniform2uiv(2 * self.len() as i32, self.as_ptr() as *const u32));
uniform_array!(self, [u32; 3] => Uniform3uiv(3 * self.len() as i32, self.as_ptr() as *const u32));
uniform_array!(self, [u32; 4] => Uniform4uiv(4 * self.len() as i32, self.as_ptr() as *const u32));
uniform_array!(self, (u32,) => Uniform1uiv(self.len() as i32, self.as_ptr() as *const u32));
uniform_array!(self, (u32, u32) => Uniform2uiv(2 * self.len() as i32, self.as_ptr() as *const u32));
uniform_array!(self, (u32, u32, u32) => Uniform3uiv(3 * self.len() as i32, self.as_ptr() as *const u32));
uniform_array!(self, (u32, u32, u32, u32) => Uniform4uiv(4 * self.len() as i32, self.as_ptr() as *const u32));
uniform_array!(self, Vector2<u32> => Uniform2uiv(2 * self.len() as i32, self.as_ptr() as *const u32));
uniform_array!(self, Vector3<u32> => Uniform3uiv(3 * self.len() as i32, self.as_ptr() as *const u32));
uniform_array!(self, Vector4<u32> => Uniform4uiv(4 * self.len() as i32, self.as_ptr() as *const u32));

use cgmath::Matrix;

uniform!(self, Matrix4<f32> => UniformMatrix4fv(1, ::gl::FALSE, self.as_ptr() as *const f32));
uniform!(self, Matrix4<f64> => UniformMatrix4dv(1, ::gl::FALSE, self.as_ptr() as *const f64));
uniform!(self, Matrix3<f32> => UniformMatrix3fv(1, ::gl::FALSE, self.as_ptr() as *const f32));
uniform!(self, Matrix3<f64> => UniformMatrix3dv(1, ::gl::FALSE, self.as_ptr() as *const f64));
uniform!(self, Matrix2<f32> => UniformMatrix4fv(1, ::gl::FALSE, self.as_ptr() as *const f32));
uniform!(self, Matrix2<f64> => UniformMatrix4dv(1, ::gl::FALSE, self.as_ptr() as *const f64));
