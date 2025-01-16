use wgpu::{vertex_attr_array, BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
}

const ATTRIBUTES: &[VertexAttribute] = &vertex_attr_array![0 => Float32x3];

impl Vertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            // Width of a vertex:
            array_stride: size_of::<Vertex>() as BufferAddress,
            // Choose between per-vertex or per-instance data
            step_mode: VertexStepMode::Vertex,
            // Attributes: mapping to our struct fields
            attributes: ATTRIBUTES,
        }
    }
}
