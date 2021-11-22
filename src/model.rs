use std::{fs::File, io::{BufRead, BufReader}};

use wgpu::util::DeviceExt;
// represents a type of vertex, and thus must be able to describe a buffer layout for it
pub trait Vertex: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable + FromIterator<f32> {
    fn describe<'a>() -> wgpu::VertexBufferLayout<'a>;
}
pub trait Mesh {
    type VertexType : Vertex;
    fn describe<'a>() -> wgpu::VertexBufferLayout<'a> {
        Self::VertexType::describe()
    }
}

pub trait Model: Mesh {

    fn get_vertex_buffer(&self) -> &wgpu::Buffer;
    fn get_index_buffer(&self) -> &wgpu::Buffer;
    fn get_index_buffer_len(&self) -> u32;
}

struct MeshBufferFactory {}
impl MeshBufferFactory {
    fn create_vertex_buffer<T: Vertex>(vertices: &[T], device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX
            }
        )
    }
    fn create_index_buffer(indices: &[u32], device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX
            }
        )
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    position: [f32; 3]
}

impl FromIterator<f32> for ModelVertex {

    fn from_iter<T: IntoIterator<Item=f32>>(iter: T) -> Self {

        let mut vec : Vec<f32> = Vec::new();
        for v in iter {
            vec.push(v)
        }

        Self {
            position: [vec[0], vec[1], vec[2]]
        }
    }
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

pub struct SimpleFileModel {

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_buffer_len: u32
}

impl Mesh for SimpleFileModel {
    type VertexType = ModelVertex;
}

impl Model for SimpleFileModel {
    fn get_vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    fn get_index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    fn get_index_buffer_len(&self) -> u32 {
        self.index_buffer_len
    }
}

impl SimpleFileModel {

    pub fn new(device: &wgpu::Device, filename: &str) -> Result<Self, std::io::Error> {

        let file = File::open(&filename)?;

        let mut reader = BufReader::new(file);
        let mut line = String::new();
        let mut vertices : Vec<<SimpleFileModel as Mesh>::VertexType>= Vec::new();
        let mut indices : Vec<u32> = Vec::new();
        loop {

            match reader.read_line(&mut line) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }

                    match line.remove(0) {

                        'v' => {
                            let verts = line[1..].trim().split(' ').filter_map(|s| s.parse::<f32>().ok());
                            if verts.clone().count() == 3 {
                                vertices.push(verts.collect())
                            }
                        },
                        'f' => {
                            let idxs = line[1..].trim().split(' ').filter_map(|s| s.parse::<u32>().ok());
                            if idxs.clone().count() == 3 {
                                indices.extend(idxs.map(|n| n-1).collect::<Vec<u32>>());
                            }
                        },
                        _ => ()
                    }

                    line.clear();
                }
                Err(err) => return Err(err)
            }
        } 

        Ok(Self {
            vertex_buffer: MeshBufferFactory::create_vertex_buffer(&vertices[..], &device),
            index_buffer: MeshBufferFactory::create_index_buffer(&indices[..], &device),
            index_buffer_len: indices.len() as u32
        })
    }
}
