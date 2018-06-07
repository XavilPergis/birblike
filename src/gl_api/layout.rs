use gl_api::error::GlResult;
use cgmath::{Vector2, Vector3, Vector4};

pub unsafe trait VertexAttribute {
    /// Issue the appropriate calls to `glVertexAttrib{I}Pointer` based on the
    /// layout of this type. Returns the total amount of individual "base" items
    /// in this layout. This is used to calculate the offset of each attribute
    /// index.
    fn define_attribs(base_slot: u32, offset: u32) -> GlResult<u32>;
    const NUM_ATTRS: usize;
}

macro_rules! offset_of {
    ($father:ty, $($field:tt)+) => ({
        #[allow(unused_unsafe)]
        let root: $father = unsafe { $crate::std::mem::uninitialized() };

        let base = &root as *const _ as usize;

        // Future error: borrow of packed field requires unsafe function or block (error E0133)
        #[allow(unused_unsafe)]
        let member =  unsafe { &root.$($field)* as *const _ as usize };

        $crate::std::mem::forget(root);

        member - base
    });
}

macro_rules! vertex {
    (vertex $name:ident {
        $($attrib:ident: $attrib_type:ty,)*
    }) => {
        #[derive(Copy, Clone, Debug)]
        #[repr(C)]
        pub struct $name {
            $($attrib: $attrib_type),*
        }

        unsafe impl ::gl_api::layout::VertexAttribute for $name {
            fn define_attribs(mut slot: u32, offset: u32) -> ::gl_api::error::GlResult<u32> {
                $(
                    let offset = offset + offset_of!($name, $attrib) as u32;
                    slot += <$attrib_type as ::gl_api::layout::VertexAttribute>::define_attribs(slot, offset)?;
                )*
                Ok(slot)
            }

            // fn num_attrs() -> usize {
            //     let mut num = 0;
            //     $(num += <$attrib_type as VertexAttribute>::num_attrs();)*
            //     num
            // }

            const NUM_ATTRS: usize = 0 $(+ <$attrib_type as ::gl_api::layout::VertexAttribute>::NUM_ATTRS)*;
        }
    }
}

vertex! {
    vertex SomeType {
        foo: i32,
        bar: Vector3<f64>,
    }
}

macro_rules! layout_simple {
    ($type:ty: $gl_type:ident $amount:expr) => {
        unsafe impl VertexAttribute for $type {
            fn define_attribs(slot: u32, offset: u32) -> GlResult<u32> {
                unsafe {
                    let size = ::std::mem::size_of::<Self>() as i32;
                    let offset = offset as *const _;
                    let normalized = false as u8; // TODO: this
                    gl_call!(EnableVertexAttribArray(slot))?;
                    gl_call!(VertexAttribPointer(
                        slot,
                        $amount,
                        ::gl::$gl_type,
                        normalized,
                        size,
                        offset
                    ))?;
                    Ok(1)
                }
            }

            const NUM_ATTRS: usize = 1;
        }
    };
    ($type:ty: iptr $gl_type:ident $amount:expr) => {
        unsafe impl VertexAttribute for $type {
            fn define_attribs(slot: u32, offset: u32) -> GlResult<u32> {
                unsafe {
                    let size = ::std::mem::size_of::<Self>() as i32;
                    let offset = offset as *const _;
                    gl_call!(EnableVertexAttribArray(slot))?;
                    gl_call!(VertexAttribIPointer(
                        slot,
                        $amount,
                        ::gl::$gl_type,
                        size,
                        offset
                    ))?;
                    Ok(1)
                }
            }

            const NUM_ATTRS: usize = 1;
        }
    }
}

unsafe impl VertexAttribute for () {
    fn define_attribs(_slot: u32, _offset: u32) -> GlResult<u32> { Ok(0) }
    const NUM_ATTRS: usize = 0;
}

layout_simple!(f32: FLOAT 1);
layout_simple!((f32,): FLOAT 1);
layout_simple!((f32, f32): FLOAT 2);
layout_simple!((f32, f32, f32): FLOAT 3);
layout_simple!((f32, f32, f32, f32): FLOAT 4);
layout_simple!([f32; 1]: FLOAT 1);
layout_simple!([f32; 2]: FLOAT 2);
layout_simple!([f32; 3]: FLOAT 3);
layout_simple!([f32; 4]: FLOAT 4);
layout_simple!(Vector2<f32>: FLOAT 2);
layout_simple!(Vector3<f32>: FLOAT 3);
layout_simple!(Vector4<f32>: FLOAT 4);

layout_simple!(f64: DOUBLE 1);
layout_simple!((f64,): DOUBLE 1);
layout_simple!((f64, f64): DOUBLE 2);
layout_simple!((f64, f64, f64): DOUBLE 3);
layout_simple!((f64, f64, f64, f64): DOUBLE 4);
layout_simple!([f64; 1]: DOUBLE 1);
layout_simple!([f64; 2]: DOUBLE 2);
layout_simple!([f64; 3]: DOUBLE 3);
layout_simple!([f64; 4]: DOUBLE 4);
layout_simple!(Vector2<f64>: DOUBLE 2);
layout_simple!(Vector3<f64>: DOUBLE 3);
layout_simple!(Vector4<f64>: DOUBLE 4);

layout_simple!(i32: iptr INT 1);
layout_simple!((i32,): iptr INT 1);
layout_simple!((i32, i32): iptr INT 2);
layout_simple!((i32, i32, i32): iptr INT 3);
layout_simple!((i32, i32, i32, i32): iptr INT 4);
layout_simple!([i32; 1]: iptr INT 1);
layout_simple!([i32; 2]: iptr INT 2);
layout_simple!([i32; 3]: iptr INT 3);
layout_simple!([i32; 4]: iptr INT 4);
layout_simple!(Vector2<i32>: iptr INT 2);
layout_simple!(Vector3<i32>: iptr INT 3);
layout_simple!(Vector4<i32>: iptr INT 4);

layout_simple!(u32: iptr UNSIGNED_INT 1);
layout_simple!((u32,): iptr UNSIGNED_INT 1);
layout_simple!((u32, u32): iptr UNSIGNED_INT 2);
layout_simple!((u32, u32, u32): iptr UNSIGNED_INT 3);
layout_simple!((u32, u32, u32, u32): iptr UNSIGNED_INT 4);
layout_simple!([u32; 1]: iptr UNSIGNED_INT 1);
layout_simple!([u32; 2]: iptr UNSIGNED_INT 2);
layout_simple!([u32; 3]: iptr UNSIGNED_INT 3);
layout_simple!([u32; 4]: iptr UNSIGNED_INT 4);
layout_simple!(Vector2<u32>: iptr UNSIGNED_INT 2);
layout_simple!(Vector3<u32>: iptr UNSIGNED_INT 3);
layout_simple!(Vector4<u32>: iptr UNSIGNED_INT 4);

layout_simple!(i16: iptr SHORT 1);
layout_simple!((i16,): iptr SHORT 1);
layout_simple!((i16, i16): iptr SHORT 2);
layout_simple!((i16, i16, i16): iptr SHORT 3);
layout_simple!((i16, i16, i16, i16): iptr SHORT 4);
layout_simple!([i16; 1]: iptr SHORT 1);
layout_simple!([i16; 2]: iptr SHORT 2);
layout_simple!([i16; 3]: iptr SHORT 3);
layout_simple!([i16; 4]: iptr SHORT 4);
layout_simple!(Vector2<i16>: iptr SHORT 2);
layout_simple!(Vector3<i16>: iptr SHORT 3);
layout_simple!(Vector4<i16>: iptr SHORT 4);

layout_simple!(u16: iptr UNSIGNED_SHORT 1);
layout_simple!((u16,): iptr UNSIGNED_SHORT 1);
layout_simple!((u16, u16): iptr UNSIGNED_SHORT 2);
layout_simple!((u16, u16, u16): iptr UNSIGNED_SHORT 3);
layout_simple!((u16, u16, u16, u16): iptr UNSIGNED_SHORT 4);
layout_simple!([u16; 1]: iptr UNSIGNED_SHORT 1);
layout_simple!([u16; 2]: iptr UNSIGNED_SHORT 2);
layout_simple!([u16; 3]: iptr UNSIGNED_SHORT 3);
layout_simple!([u16; 4]: iptr UNSIGNED_SHORT 4);
layout_simple!(Vector2<u16>: iptr UNSIGNED_SHORT 2);
layout_simple!(Vector3<u16>: iptr UNSIGNED_SHORT 3);
layout_simple!(Vector4<u16>: iptr UNSIGNED_SHORT 4);

layout_simple!(i8: iptr BYTE 1);
layout_simple!((i8,): iptr BYTE 1);
layout_simple!((i8, i8): iptr BYTE 2);
layout_simple!((i8, i8, i8): iptr BYTE 3);
layout_simple!((i8, i8, i8, i8): iptr BYTE 4);
layout_simple!([i8; 1]: iptr BYTE 1);
layout_simple!([i8; 2]: iptr BYTE 2);
layout_simple!([i8; 3]: iptr BYTE 3);
layout_simple!([i8; 4]: iptr BYTE 4);
layout_simple!(Vector2<i8>: iptr BYTE 2);
layout_simple!(Vector3<i8>: iptr BYTE 3);
layout_simple!(Vector4<i8>: iptr BYTE 4);

layout_simple!(u8: iptr UNSIGNED_BYTE 1);
layout_simple!((u8,): iptr UNSIGNED_BYTE 1);
layout_simple!((u8, u8): iptr UNSIGNED_BYTE 2);
layout_simple!((u8, u8, u8): iptr UNSIGNED_BYTE 3);
layout_simple!((u8, u8, u8, u8): iptr UNSIGNED_BYTE 4);
layout_simple!([u8; 1]: iptr UNSIGNED_BYTE 1);
layout_simple!([u8; 2]: iptr UNSIGNED_BYTE 2);
layout_simple!([u8; 3]: iptr UNSIGNED_BYTE 3);
layout_simple!([u8; 4]: iptr UNSIGNED_BYTE 4);
layout_simple!(Vector2<u8>: iptr UNSIGNED_BYTE 2);
layout_simple!(Vector3<u8>: iptr UNSIGNED_BYTE 3);
layout_simple!(Vector4<u8>: iptr UNSIGNED_BYTE 4);
