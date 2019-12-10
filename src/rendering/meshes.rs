/*
    Usage of DynamicGeometry:

        let mat = Box::new(SimpleMaterial::new());

        let vertices = vec![
            Vertex::new(Vec3::new(0.5, -0.5, 0.0),  Vec3::new(1.0, 0.0, 0.0)),
            Vertex::new(Vec3::new(-0.5, -0.5, 0.0), Vec3::new(0.0, 1.0, 0.0)),
            Vertex::new(Vec3::new(0.0, 0.5, 0.0),   Vec3::new(0.0, 0.0, 1.0))
        ];

        let mut geometry = ExtendableBasicGeometry::from_vertices(&vertices);
        geometry.add_attr::<f32>("alpha");
        {
            let attrib = geometry.attr_by_label_mut("alpha");

            attrib.local_buffer = vec![
                1.0, 0.15, 0.0
            ]
        }

        let mesh = Mesh::new(PrimitiveType::Triangles, mat, &geometry);

*/


use nalgebra_glm::{Vec3, Mat4};
use std::mem::*;
use std::any::Any;
use gl::types::*;
use std::ptr::*;
use std::collections::*;
use std::borrow::*;
use crate::rendering::RenderParameters;
use crate::rendering::buffers::{VertexArray, VertexBuffer, VertexBufferBase};
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
    /// Create new vertex without a normal vector. It will be set to the normal vector.
    pub fn new(pos: Vec3, clr: Vec3) -> Vertex {
        Vertex {
            position: pos,
            color: clr,
            normal: Vec3::zeros()
        }
    }

    /// Create new vertex with given normal vector
    pub fn new_with_normal(pos: Vec3, clr: Vec3, nrml: Vec3) -> Vertex {
        Vertex {
            position: pos,
            color: clr,
            normal: nrml
        }
    }
}

/// A struct containg information about a single vertex attribute in a shader.
#[derive(Clone)]
pub struct AttributeDescriptor {
    /// The index/location of the attribute
    pub index: usize,
    /// A human-readable label for the attribute
    pub label: String
}

impl AttributeDescriptor {
    /// Create new attribute descriptor from index and label string
    pub fn new(index: usize, label: &str) -> AttributeDescriptor {
        AttributeDescriptor {
            index: index,
            label: label.to_string()
        }
    }
}

pub trait AttributeArrayBase {
    /// Setup this attribute for the currently bound VAO.
    /// The corresponding VBO needs to be bound by the calling code.
    /// This function is located in this trait in order to avoid having to share
    /// extensive information about the stored types.
    fn setup_attribute(&self, vao: &VertexArray);

    /// Create vertex buffer from this attribute array.
    fn to_vertex_buffer(&self) -> Box<dyn VertexBufferBase>;

    /// How many elements are currently stored in the local buffer.
    fn len(&self) -> usize;

    /// Retrieve this instance as a reference to Any. This is used for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Retrieve this instance as a mutable reference to Any. This is used for downcasting.
    fn as_mut_any(&mut self) -> &mut dyn Any;

    /// Retrieve attribute label
    fn label(&self) -> &str;
}

/// A struct managing the local data buffer and information for a single vertex attribute
#[derive(Clone)]
pub struct AttributeArray<T: GPUType + 'static> {
    /// The local buffer containing the data meant for this attribute array
    pub local_buffer: Vec<T>,
    /// Information about this attribute
    descriptor: AttributeDescriptor
}

impl<T: GPUType + 'static> AttributeArray<T> {
    /// Create a new attribute array with an empty backing buffer.
    pub fn new(index: usize, label: &str) -> AttributeArray<T> {
        AttributeArray::<T> {
            descriptor: AttributeDescriptor::new(index, label),
            local_buffer: Vec::new()
        }
    }

    /// Create a new attribute array with a backing buffer with given capacity.
    pub fn with_capacity(index: usize, label: &str, capacity: usize) -> AttributeArray<T> {
        AttributeArray::<T> {
            descriptor: AttributeDescriptor::new(index, label),
            local_buffer: Vec::with_capacity(capacity)
        }
    }
}

impl<T: 'static> AttributeArrayBase for AttributeArray<T> where T: GPUType {
    fn setup_attribute(&self, vao: &VertexArray) {
        vao.activate_attribute::<T>(&self.descriptor);
    }

    fn to_vertex_buffer(&self) -> Box<dyn VertexBufferBase> {
        Box::new(VertexBuffer::<T>::new(&self.local_buffer))
    }

    /// How many elements are currently stored in the local buffer.
    fn len(&self) -> usize {
        self.local_buffer.len()
    }

    /// Retrieve this instance as a reference to Any. This is used for downcasting.
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Retrieve this instance as a mutable reference to Any. This is used for downcasting.
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    /// Retrieve attribute label
    fn label(&self) -> &str {
        &self.descriptor.label
    }
}

/// Basic geometry trait.
pub trait Geometry {
    fn retrieve_attributes(&self) -> Vec<&dyn AttributeArrayBase>;
}

/// Special geometry trait that allows the dynamic additon of vertex attributes.
/// This is useful for geometry implementations that are expected to be used with custom
/// materials that might require additional data in addition to the default vertex data.
/// This would save the user from having to declare their own geometry kind.
pub trait DynamicGeometry: Geometry {
    /// Add a new vertex attribute with given gpu type and label to this geometry.
    fn add_attr<T: GPUType + 'static>(&mut self, label: &str);
    /// Retrieve mutable reference to the vertex attribute with given label.
    fn attr_by_label_mut<T: GPUType + 'static>(&mut self, label: &str) -> &mut AttributeArray<T>;
    /// Retrieve shared reference to the vertex attribute with given label.
    fn attr_by_label<T: GPUType + 'static>(&self, label: &str) -> & AttributeArray<T>;
}

/// A basic geometry that can dynamically be extended with additional attributes.
pub struct ExtendableBasicGeometry {
    /// Position value for each vertex
    positions: AttributeArray<Vec3>,
    /// Color value for each vertex
    colors: AttributeArray<Vec3>,
    /// Normal vector for each vertex
    normals: AttributeArray<Vec3>,
    /// Dynamically added attributes
    dynamic: Vec<Box<dyn AttributeArrayBase>>
}

impl ExtendableBasicGeometry {
    fn next_index(&self) -> usize {
        self.dynamic.len() + 3
    }
}

impl Geometry for ExtendableBasicGeometry {
    fn retrieve_attributes(&self) -> Vec<&dyn AttributeArrayBase> {
        let mut vec = Vec::<&dyn AttributeArrayBase>::new(); 

        vec.push(&self.positions);
        vec.push(&self.colors);
        vec.push(&self.normals);

        let dynamics: Vec<&dyn AttributeArrayBase> = self.dynamic.iter().map(|a| a.as_ref()).collect();

        vec.extend_from_slice(&dynamics);

        vec
    }
}

impl ExtendableBasicGeometry {
    /// Construct geometry with default attributes from given slice of vertices
    pub fn from_vertices(vertices: &[Vertex]) -> ExtendableBasicGeometry {
        let mut geometry = ExtendableBasicGeometry {
            positions: AttributeArray::with_capacity(0, "position", vertices.len()),
            colors: AttributeArray::with_capacity(1, "color", vertices.len()),
            normals: AttributeArray::with_capacity(2, "normal", vertices.len()),
            dynamic: Vec::new()
        };

        for v in vertices {
            geometry.positions.local_buffer.push(v.position);
            geometry.colors.local_buffer.push(v.color);
            geometry.normals.local_buffer.push(v.normal);
        }

        geometry
    }

    /// Construct empty geometry instance
    pub fn new() -> ExtendableBasicGeometry {
        ExtendableBasicGeometry {
            positions: AttributeArray::new(0, "position"),
            colors: AttributeArray::new(1, "color"),
            normals: AttributeArray::new(2, "normal"),
            dynamic: Vec::new()
        }
    }
}

impl DynamicGeometry for ExtendableBasicGeometry {
    fn add_attr<T: GPUType + 'static>(&mut self, label: &str) {
        let mut attrib = Box::new(AttributeArray::<T>::new(
            self.next_index(),
            label
        ));
        self.dynamic.push(attrib);
    }

    fn attr_by_label_mut<T: GPUType + 'static>(&mut self, label: &str) -> &mut AttributeArray<T> {
        let rf = self.dynamic.iter_mut().find(|a| a.label() == label).expect("Attribute with requested label not found");
        let mut_rf = &mut **rf;

        let any_rf = mut_rf.as_mut_any();

        let concrete_ref: &mut AttributeArray<T> = match any_rf.downcast_mut::<AttributeArray<T>>() {
            Some(a) => a,
            None => panic!("attr_by_label_mut: Type mismatch!"),
        };

        concrete_ref
    }

    fn attr_by_label<T: GPUType + 'static>(&self, label: &str) -> &AttributeArray<T> {
        let rf = self.dynamic.iter().find(|a| a.label() == label).expect("Attribute with requested label not found");
        let direct_rf = &**rf;

        let any_rf = direct_rf.as_any();

        let concrete_ref: &AttributeArray<T> = match any_rf.downcast_ref::<AttributeArray<T>>() {
            Some(a) => a,
            None => panic!("attr_by_label: Type mismatch!"),
        };

        concrete_ref
    }
}


/// A struct storing vertex information for basic drawing operations: position, color and
/// normal vector.
#[derive(Clone)]
pub struct BasicGeometry {
    /// Position value for each vertex
    positions: AttributeArray<Vec3>,
    /// Color value for each vertex
    colors: AttributeArray<Vec3>,
    /// Normal vector for each vertex
    normals: AttributeArray<Vec3>
}

impl BasicGeometry {
    /// Construct geometry with default attributes from given slice of vertices
    pub fn from_vertices(vertices: &[Vertex]) -> BasicGeometry {
        let mut geometry = BasicGeometry {
            positions: AttributeArray::with_capacity(0, "position", vertices.len()),
            colors: AttributeArray::with_capacity(1, "color", vertices.len()),
            normals: AttributeArray::with_capacity(2, "normal", vertices.len())
        };

        for v in vertices {
            geometry.positions.local_buffer.push(v.position);
            geometry.colors.local_buffer.push(v.color);
            geometry.normals.local_buffer.push(v.normal);
        }

        geometry
    }

    /// Construct empty geometry instance
    pub fn new() -> BasicGeometry {
        BasicGeometry {
            positions: AttributeArray::new(0, "position"),
            colors: AttributeArray::new(1, "color"),
            normals: AttributeArray::new(2, "normal")
        }
    }
}

impl Geometry for BasicGeometry {
    fn retrieve_attributes(&self) -> Vec<&dyn AttributeArrayBase> {
        vec![&self.positions, &self.colors, &self.normals]
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
    /// Vertex buffer objects storing the vertex attribute data. There is a buffer
    /// for each attribute in the geometry.
    buffers: Vec<Box<dyn VertexBufferBase>>,
    /// Vertex array object defining vertex attributes
    vao: VertexArray,
    /// Associated material trait object
    material: Box<dyn Material>,
    /// Number of vertices supplied
    num_vertices: usize
}

impl Mesh {
    /// Create a new mesh with given primitive type from given geometry
    pub fn new(pt: PrimitiveType, mat: Box<dyn Material>, geometry: &dyn Geometry) -> Mesh {
        let attributes = geometry.retrieve_attributes();
        
        let mut mesh = Mesh {
            primitive_type: pt,
            material: mat,
            vao: VertexArray::new(),
            buffers: Vec::new(),
            num_vertices: Self::retrieve_vertex_count(&attributes).expect("Geometry attribute buffer sizes inconsistent")
        };

        // Create buffers and register attributes with vao for each attribute in the geometry
        for attribute in &attributes {
            let buffer = attribute.to_vertex_buffer();

            buffer.enable();
            attribute.setup_attribute(&mesh.vao);
            buffer.disable();

            mesh.buffers.push(buffer);
        }

        mesh
    }

    /// Retrieve downcasted material reference
    pub fn retrieve_material_ref<T: Material + 'static>(&self) -> &T {
        let rf = &*self.material;
        let any_rf = rf.as_any();

        let concrete_ref: & T = match any_rf.downcast_ref::<T>() {
            Some(a) => a,
            None => panic!("retrieve_material_ref: Type mismatch!"),
        };

        concrete_ref
    }

    /// Retrieve mutable downcasted material reference
    pub fn retrieve_material_mut_ref<T: Material + 'static>(&mut self) -> &mut T {
        let rf = &mut *self.material;
        let any_rf = rf.as_mut_any();

        let concrete_ref: &mut T = match any_rf.downcast_mut::<T>() {
            Some(a) => a,
            None => panic!("retrieve_material_mut_ref: Type mismatch!"),
        };

        concrete_ref
    }

    /// Try to retrieve the total vertex count for given set of attributes.
    /// This can fail if the number of entries in each of the attribute arrays is not the same.
    fn retrieve_vertex_count(attributes: &Vec<&dyn AttributeArrayBase>) -> Option<usize> {
        let lengths: Vec<usize> = attributes.iter().map(|a| a.len()).collect();

        fn all_same(arr: &[usize]) -> bool {
            if arr.is_empty() {
                return false;
            } else {
                let first = arr[0];
                return arr.iter().all(|&item| item == first);
            }
        }

        match all_same(&lengths) {
            true => Some(lengths[0]),
            _ => None
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
