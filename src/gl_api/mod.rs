#[macro_use]
pub mod error;
#[macro_use]
pub mod layout;

pub mod buffer;
pub mod misc;
pub mod shader;
pub mod texture;
pub mod uniform;
pub mod vertex_array;

// // program, render target, data source

// pub struct DefaultFramebuffer;

// pub trait RenderTarget {
//     fn draw<In, Env>(&mut self, program: LinkedProgram<In, Env>) -> GlResult<()>;
// }

// // fn render<In, Env>() {}
