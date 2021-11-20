use wgpu::util::DeviceExt;

pub trait Vertex {
    fn describe<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3]
}

impl Vertex for ModelVertex {

    fn describe<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3
                }
            ]
        }
    } 
}

impl ModelVertex {

    pub fn create_buffer(device: &wgpu::Device, verts: &[ModelVertex]) -> wgpu::Buffer {

        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(verts),
                usage: wgpu::BufferUsages::VERTEX
            }
        )
    }
}
