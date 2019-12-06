use nalgebra_glm::{Vec3, Mat4};
use std::mem::*;
use gl::types::*;
use std::ptr::*;
use crate::rendering::RenderParameters;
use crate::rendering::buffers::{VertexArray, VertexBuffer};
use crate::rendering::materials::*;
use crate::rendering::traits::*;

/// The data of single vertex. In general OpenGL applications, the vertex format can significantly vary
/// from use case to use case, but in this project, we always use the exact same structure. This means
/// that some shaders might elect to ignore certain attributes. The mesh struct will automatically fill
/// the missing data with zeroes if the user code did not supply data for them.
/// This is the struct the user most often interacts with.
#[derive(Clone, Copy)]
pub struct Vertex {
    /// The position of the vertex in model space
    position: Vec3,
    /// The color for this vertex
    color: Vec3,
    /// The normal vector for this vertex
    normal: Vec3
}

impl Vertex {
    pub fn new(pos: Vec3, clr: Vec3) -> Vertex {
        Vertex {
            position: pos,
            color: clr,
            normal: Vec3::zeros()
        }
    }

    pub fn new_with_normal(pos: Vec3, clr: Vec3, nrml: Vec3) -> Vertex {
        Vertex {
            position: pos,
            color: clr,
            normal: nrml
        }
    }
}

/// Enumeration describing the various primtive types that can be used to interpret and draw
/// the vertex data contained within a mesh instance.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum PrimitiveType {
    Triangles = gl::TRIANGLES,
    TriangleStrip = gl::TRIANGLE_STRIP,
    TriangleFan = gl::TRIANGLE_FAN,
    Lines = gl::LINES,
    LineStrip = gl::LINE_STRIP,
    LineLoop = gl::LINE_LOOP
}

/// A mesh is a combination of geometry and a material, which can not be changed.
/// TODO: Normal vector generation on construction
pub struct Mesh {
    /// The primitive type used to interpret the associated vertices
    primitive_type: PrimitiveType,
    /// The associated vertex buffer object containing the vertex data
    vbo: VertexBuffer<Vec3>,
    /// Vertex array object defining vertex attributes
    vao: VertexArray<Vec3>,
    /// Associated material trait object
    material: Box<dyn Material>,
    /// Number of vertices supplied
    num_vertices: usize
}

impl Mesh {
    /// Create a new mesh with given primitive type from given material and vertices
    pub fn new(pt: PrimitiveType, mat: Box<dyn Material>,  vertices: &[Vertex]) -> Mesh {
        let mut buffer: Vec<Vec3> = Vec::with_capacity(vertices.len() * 3);

        for vertex in vertices {
            buffer.push(vertex.position);
            buffer.push(vertex.color);
            buffer.push(vertex.normal);
        }

        let vbo = VertexBuffer::new(&buffer);
        let vao = VertexArray::new(&vbo, 3);

        Mesh {
            primitive_type: pt,
            vbo: vbo,
            vao: vao,
            material: mat,
            num_vertices: vertices.len()
        }
    }
}

impl Render for Mesh {
    fn render(&self, params: &mut RenderParameters) {
        self.material.enable_material(params);
        self.vao.enable_array();

        unsafe{
            gl::DrawArrays(self.primitive_type as _, 0, self.num_vertices as _);
        }

        self.vao.disable_array();
    }
}
