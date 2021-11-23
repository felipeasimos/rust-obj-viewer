use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {

    position: [f32; 3],
    _padding: u32,
    color: [f32; 3]
}

impl LightUniform {

    fn new(position: [f32; 3], color: [f32; 3]) -> Self {

        Self {
            position,
            _padding: 0,
            color
        }
    }
}

#[derive(Debug)]
pub struct LightData {
    pub position: cgmath::Point3<f32>,
    pub color: (f32, f32, f32)
}

impl LightData {

    pub fn new<P: Into<cgmath::Point3<f32>>>(position: P, color: (f32, f32, f32)) -> Self {

        Self {
            position: position.into(),
            color
        }
    }

    fn into_uniform(&self) -> LightUniform {
       LightUniform::new([self.position.x, self.position.y, self.position.z], [self.color.0, self.color.1, self.color.2])
    }
}

pub struct Light {

    data: LightData,
    uniform: LightUniform,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Light {

    pub fn new(device: &wgpu::Device, data: LightData) -> (Self, wgpu::BindGroupLayout) {

        let mut uniform = data.into_uniform();

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ],
            label: Some("light_bind_group_layout")
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {

            layout: &light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding()
                }
            ],
            label: Some("light_bind_group")
        });

        (
            Self {
                data,
                uniform,
                buffer,
                bind_group,
            },
            light_bind_group_layout
        )

    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    pub fn update_buffers(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {

        // create staging buffer with new data
        let staging_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light Staging Buffer"),
                contents: bytemuck::cast_slice(&[self.uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC
            }
        );

        // copy contents of staging buffer to the actual camera buffer
        encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.buffer, 0, std::mem::size_of::<LightUniform>() as wgpu::BufferAddress);
    }
}
