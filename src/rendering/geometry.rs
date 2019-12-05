use nalgebra_glm::{Vec3, Mat4};
use std::mem::*;
use gl::types::*;
use std::ptr::*;
use crate::rendering::RenderParameters;

/// A struct used to store and modify mesh geometry data such as vertex positions, normal vectors
/// and colors. The point is that the contents of instances of this struct can be directly and freely 
/// modified by the using code, and only when creating a mesh based on it is the data verified and uploaded
/// to the GPU.
#[derive(Clone)]
pub struct Geometry {
    /// The vertex positions
    pub vertex_positions: Vec<Vec3>,
    /// The color component for each vertex
    pub colors: Vec<Vec3>,
    /// The normal vectors for each vertex
    pub normals: Vec<Vec3>
}

impl Geometry {
    pub fn new() -> Geometry {
        Geometry {
            vertex_positions: Vec::new(),
            colors: Vec::new(),
            normals: Vec::new()
        }
    }
}

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