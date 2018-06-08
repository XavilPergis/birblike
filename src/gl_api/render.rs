pub enum RenderTarget {
    Default,
    // Offscreen(Framebuffer)
}

pub struct RenderData {
    
}

pub fn draw(buffer: VertexArray<V>, program: Program<V, E>, target: RenderTarget) {
    buffer.bind();
    program.bind();
}
