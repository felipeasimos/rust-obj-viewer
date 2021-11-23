use std::{fs::File, io::{BufRead, BufReader}};

use wgpu::util::DeviceExt;
// represents a type of vertex, and thus must be able to describe a buffer layout for it
pub trait Vertex: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {
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
    position: [f32; 3],
    normal: [f32; 3]
}

impl ModelVertex {
    fn new(position: [f32; 3], normal: [f32; 3]) -> Self {
        Self {
            position,
            normal
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
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
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
        let mut vertices : Vec<[f32; 3]> = Vec::new();
        let mut vertex_normals : Vec<[f32; 3]> = Vec::new();
        let mut indices : Vec<u32> = Vec::new();
        let mut indexed_references : bool = false;
        loop {

            match reader.read_line(&mut line) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }

                    match line.remove(0) {

                        'v' => {
                            match line.remove(0) {

                                'n' => {
                                    let vert_normal = line.trim().split(' ').filter_map(|s| s.parse::<f32>().ok());
                                    if vert_normal.clone().count() == 3 {
                                        let mut final_array : [f32; 3] = [0.0; 3];
                                        for (i, val) in vert_normal.enumerate() {
                                            final_array[i] = val;
                                        }
                                        vertex_normals.push(final_array);
                                    }
                                }
                                ' ' => {
                                    let vert = line.trim().split(' ').filter_map(|s| s.parse::<f32>().ok());
                                    if vert.clone().count() == 3 {
                                        let mut final_array : [f32; 3] = [0.0; 3];
                                        for (i, val) in vert.enumerate() {
                                            final_array[i] = val;
                                        }
                                        vertices.push(final_array);
                                    }
                                },
                                _ => ()
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

        // if indices don't use references to normals or textures
        let mut final_vertices : Vec<ModelVertex> = Vec::with_capacity(vertices.len());
        if !indexed_references && vertex_normals.len() > 0 {

            for (vert, normal) in vertices.iter().zip(vertex_normals.iter()) {

                final_vertices.push(ModelVertex::new(*vert, *normal));
            }
        } else if vertex_normals.len() == 0 {

            for vert in vertices {
                final_vertices.push(ModelVertex::new(vert, [0.0, 1.0, 0.0]));
            }
        }

        Ok(Self {
            vertex_buffer: MeshBufferFactory::create_vertex_buffer(&final_vertices[..], &device),
            index_buffer: MeshBufferFactory::create_index_buffer(&indices[..], &device),
            index_buffer_len: indices.len() as u32
        })
    }
}
