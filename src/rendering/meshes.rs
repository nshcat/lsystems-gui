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


use nalgebra_glm::{Vec3, Mat4, UVec3};
use std::mem::*;
use std::any::Any;
use std::f32::*;
use std::cmp::*;
use gl::types::*;
use std::ptr::*;
use std::collections::*;
use std::borrow::*;
use crate::rendering::RenderParameters;
use crate::rendering::buffers::{VertexArray, Buffer, BufferBase};
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
    fn to_vertex_buffer(&self) -> Box<dyn BufferBase>;

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

    fn to_vertex_buffer(&self) -> Box<dyn BufferBase> {
        Box::new(Buffer::<T>::new_vertex_buffer(&self.local_buffer))
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

/// Trait for geometries that are rendered via indexed drawing.
pub trait IndexedGeometry: Geometry {
    fn retrieve_indices(&self) -> &[u32];
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

/// Struct bundling functions that allow automatic generation of normal vectors.
/// This only works for triangle meshes.
pub struct NormalGenerator;

impl NormalGenerator {
    /// Calculate all faces of indexed geometry.
    fn calculate_indexed_faces(pt: PrimitiveType, indices: &[u32]) -> Vec<UVec3> {
        let mut faces = Vec::new();

        match pt {
            // Triangle fans contain triangles that all share the same root vertex, which 
            // always is the vertex with index 0.
            PrimitiveType::TriangleFan => {
                for i in 2..indices.len() {
                    faces.push(UVec3::new(indices[0], indices[(i-1)], indices[i]));
                }
            },
            // Unconnected triangle soup, which means that each vertex is only used exactly once.
            PrimitiveType::Triangles => {
                if indices.len() % 3 != 0 {
                    panic!("Can't create faces for given vertex count: not multiple of 3");
                }

                for i in (0..indices.len()).step_by(3) {
                    faces.push(UVec3::new(indices[i], indices[i+1], indices[i+2]));
                }
            },
            // A strip uses a sliding window of width 3 to assign triangles to vertices.
            PrimitiveType::TriangleStrip => {
                for i in 2..indices.len() {
                    faces.push(UVec3::new(indices[i-2], indices[i-1], indices[i]));
                }
            },
            _ => panic!("Primitive type not supported by calculate_indexed_faces!")
        };

        faces
    }

    /// Determine all faces of the geometry. This returns vectors containg three indices
    /// into the vertex slice.
    fn calculate_faces(pt: PrimitiveType, num_vertices: usize) -> Vec<UVec3> {
        // At least three vertices are required
        if num_vertices < 3 {
            panic!("Expected at least three vertices, found {}", num_vertices);
        }
        
        let mut faces = Vec::new();

        match pt {
            // Triangle fans contain triangles that all share the same root vertex, which 
            // always is the vertex with index 0.
            PrimitiveType::TriangleFan => {
                for i in 2..num_vertices {
                    faces.push(UVec3::new(0_u32, (i-1) as _, i as _));
                }
            },
            // Unconnected triangle soup, which means that each vertex is only used exactly once.
            PrimitiveType::Triangles => {
                if num_vertices % 3 != 0 {
                    panic!("Can't create faces for given vertex count: not multiple of 3");
                }

                for i in (0..num_vertices).step_by(3) {
                    faces.push(UVec3::new(i as _, (i+1) as _, (i+2) as _));
                }
            },
            // A strip uses a sliding window of width 3 to assign triangles to vertices.
            PrimitiveType::TriangleStrip => {
                for i in 2..num_vertices {
                    faces.push(UVec3::new((i-2) as _, (i-1) as _, i as _));
                }
            },
            _ => panic!("Primitive type not supported by calculate_faces!")
        };

        faces
    }

    /// Generates normal vectors for given geometry faces.
    fn generate_face_normals(positions: &[Vec3], faces: &[UVec3]) -> Vec<Vec3> {
        let mut face_normals = Vec::with_capacity(faces.len());

        for face in faces {
            let vA = &positions[face.x as usize];
            let vB = &positions[face.y as usize];
            let vC = &positions[face.z as usize];

            let cb = vC - vB;
            let ab = vA - vB;

            let normal = cb.cross(&ab).normalize();

            face_normals.push(normal);
        }

        face_normals
    }

    /// Generate vertex normals for given vertices interpreted as given primitive type
    pub fn generate_normals(pt: PrimitiveType, positions: &[Vec3]) -> Vec<Vec3> {
        // Create sequence of empty normal vectors
        let mut normals = Vec::with_capacity(positions.len());

        for _ in 0..positions.len() {
            normals.push(Vec3::zeros());
        }

        // Determine faces
        let faces = Self::calculate_faces(pt, positions.len());

        // Calculate face normals
        let face_normals = Self::generate_face_normals(positions, &faces);

        // For each face, add the normal vector to each of its participating vertices
        for (i, face) in faces.iter().enumerate() {
            let normal = face_normals[i];

            normals[face.x as usize] += normal;
            normals[face.y as usize] += normal;
            normals[face.z as usize] += normal;
        }

        // Finally, the normal vectors need to be normalized
        for i in 0..normals.len() {
            *normals[i] = *normals[i].normalize();
        }

        normals
    }

    /// Generate vertex normals for given indexed vertices interpreted as given primitive type
    pub fn generate_indexed_normals(pt: PrimitiveType, positions: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
        // Create sequence of empty normal vectors
        let mut normals = Vec::with_capacity(positions.len());

        for _ in 0..positions.len() {
            normals.push(Vec3::zeros());
        }

        // Determine faces
        let faces = Self::calculate_indexed_faces(pt, indices);

        // Calculate face normals
        let face_normals = Self::generate_face_normals(positions, &faces);

        // For each face, add the normal vector to each of its participating vertices
        for (i, face) in faces.iter().enumerate() {
            let normal = face_normals[i];

            // TODO normally this would be +=, why is it not working with +=?
            normals[face.x as usize] = normal;
            normals[face.y as usize] = normal;
            normals[face.z as usize] = normal;
        }

        // Finally, the normal vectors need to be normalized
        for i in 0..normals.len() {
            *normals[i] = *normals[i].normalize();
        }

        normals
    }
}

/// A struct storing vertex information for basic drawing operations: position, color and
/// normal vector.
#[derive(Clone)]
pub struct BasicGeometry {
    /// Position value for each vertex
    pub positions: AttributeArray<Vec3>,
    /// Color value for each vertex
    pub colors: AttributeArray<Vec3>,
    /// Normal vector for each vertex
    pub normals: AttributeArray<Vec3>
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

    /// Construct geometry from given slice of vertices, and automatically generate normal vectors. 
    /// If normal vectors are already present they will be discarded.
    pub fn with_auto_normals(pt: PrimitiveType, vertices: &[Vertex]) -> BasicGeometry {
        let mut geometry = BasicGeometry {
            positions: AttributeArray::with_capacity(0, "position", vertices.len()),
            colors: AttributeArray::with_capacity(1, "color", vertices.len()),
            normals: AttributeArray::new(2, "normal")
        };

        for v in vertices {
            geometry.positions.local_buffer.push(v.position);
            geometry.colors.local_buffer.push(v.color);
        }

        geometry.normals.local_buffer = NormalGenerator::generate_normals(pt, &geometry.positions.local_buffer);

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

/// A geometry that generates a sphere. Uses GL_TRIANGLES
pub struct SphereGeometry {
    positions: AttributeArray<Vec3>,
    colors: AttributeArray<Vec3>,
    normals: AttributeArray<Vec3>,
    indices: Vec<u32>
}

impl Geometry for SphereGeometry {
    fn retrieve_attributes(&self) -> Vec<&dyn AttributeArrayBase> {
        vec![&self.positions, &self.colors, &self.normals]
    }
}

impl IndexedGeometry for SphereGeometry {
    fn retrieve_indices(&self) -> &[u32] {
        &self.indices
    }
}

impl SphereGeometry {
    pub fn new(radius: f32, slices: u32, tiles: u32, color: Vec3) -> SphereGeometry {
        let mut geom = SphereGeometry {
            positions: AttributeArray::new(0, "position"),
            colors: AttributeArray::new(1, "color"),
            normals: AttributeArray::new(2, "normal"),
            indices: Vec::new()
        };

        let dfi = consts::PI / (tiles as f32);
        let dth = (2.0 * consts::PI) / (slices as f32);

        let mut cs_fi = Vec::new();
        let mut cs_th = Vec::new();
        let mut sn_fi = Vec::new();
        let mut sn_th = Vec::new();

        let mut fi: f32 = 0.0;
        let mut th: f32 = 0.0;

        for _ in 0..= max(tiles, slices) {
            cs_fi.push(fi.cos());
            cs_th.push(th.cos());

            sn_fi.push(fi.sin());
            sn_th.push(th.sin());

            fi = fi + dfi;
            th = th + dth;
        }


        for i in 0..=tiles {
            for j in 0..=slices {
                let k = j % slices;

                let normal = Vec3::new(
                    sn_fi[i as usize] * cs_th[k as usize],
                    sn_fi[i as usize] * sn_th[k as usize],
                    cs_fi[i as usize]
                );

                let position = normal * radius;

                geom.colors.local_buffer.push(color.clone());
                geom.positions.local_buffer.push(position);
                geom.normals.local_buffer.push(normal);
            }
        }

        let mut offset: u32 = 0;

        for j in 0..tiles {
            for i in 1..slices {
                let idx = i + offset;

                geom.indices.push(idx - 1);
                geom.indices.push(idx + slices - 1);
                geom.indices.push(idx);


                geom.indices.push(idx);
                geom.indices.push(idx + slices - 1);
                geom.indices.push(idx + slices);
            }
            offset = offset + slices;
        }

        geom
    }
}

/// A geometry that generates a tesselated plane, that goes from 0 to 1 in each dimensions
/// and is in XZ plane
pub struct PlaneGeometry {
    positions: AttributeArray<Vec3>,
    colors: AttributeArray<Vec3>,
    normals: AttributeArray<Vec3>,
    indices: Vec<u32>
}

impl Geometry for PlaneGeometry {
    fn retrieve_attributes(&self) -> Vec<&dyn AttributeArrayBase> {
        vec![&self.positions, &self.colors, &self.normals]
    }
}

impl IndexedGeometry for PlaneGeometry {
    fn retrieve_indices(&self) -> &[u32] {
        &self.indices
    }
}

impl PlaneGeometry {
    /// Create a new plane and displace vertices using given closure
    pub fn with_displacement(rows: u32, cols: u32, color: Vec3, f: &dyn Fn(f32, f32) -> Vec3) -> PlaneGeometry {
        let mut plane = Self::new(rows, cols, color);

        {
            let vertices = &mut plane.positions.local_buffer;

            for y in 0..=rows {
                let base = y * (cols+1);

                for x in 0..=cols {
                    let u = (x as f32) / (cols) as f32;
                    let v = (y as f32) / (rows) as f32;

                    let newVertex = f(u, v);
                    vertices[(base + x) as usize] = newVertex;
                }
            }
        }

        plane.regenerate_normals();
        
        plane
    }

    /// Regenerate all vertex normals.
    pub fn regenerate_normals(&mut self) {
        self.normals.local_buffer = NormalGenerator::generate_indexed_normals(
            PrimitiveType::TriangleStrip,
            &self.positions.local_buffer,
            &self.indices
        );
    }

    /// Create a new plane geometry
    pub fn new(rows: u32, cols: u32, color: Vec3) -> PlaneGeometry {
        let total_vertices = (rows + 1) * (cols + 1);
        let num_indices_per_row = (cols * 2) + 2;
        let num_index_degens_req = (rows - 1) * 2;
        let total_indices = num_indices_per_row * rows + num_index_degens_req;

        let mut vertices: Vec<Vec3> = vec![Vec3::zeros(); total_vertices as _];
        let mut indices: Vec<u32> = Vec::with_capacity(total_indices as _);

        let rows = rows + 1;
        let cols = cols + 1;

        // Create vertices
        for y in 0..rows {
            let base = y * cols;

            for x in 0..cols {
                let index = (base + x) as usize;
                vertices[index] = Vec3::new(
                    (x as f32) / ((cols-1) as f32),
                    (y as f32) / ((rows-1)  as f32),
                    0.0
                );
            }
        }

        // Create indices
        let rows = rows - 1;

        for y in 0..rows {
            let base = y * cols;

            for x in 0..cols {
                indices.push(base + x);
                indices.push(base + cols + x);
            }

            if y < (rows - 1) {
                indices.push((y + 1) * cols + (cols - 1));
                indices.push((y + 1) * cols);
            }
        }

        let mut geometry = PlaneGeometry {
            positions: AttributeArray::new(0, "position"),
            colors: AttributeArray::new(1, "color"),
            normals: AttributeArray::new(2, "normal"),
            indices: indices
        };

        geometry.colors.local_buffer = vec![color; vertices.len()];
        geometry.normals.local_buffer = vec![Vec3::new(0.0, 1.0, 0.0); vertices.len()];
        geometry.positions.local_buffer = vertices;

        geometry
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
    LineLoop = gl::LINE_LOOP,
    Points = gl::POINTS
}

/// A mesh is a combination of geometry and a material, which can not be changed.
/// TODO: Normal vector generation on construction
pub struct Mesh {
    /// The primitive type used to interpret the associated vertices
    primitive_type: PrimitiveType,
    /// Vertex buffer objects storing the vertex attribute data. There is a buffer
    /// for each attribute in the geometry.
    buffers: Vec<Box<dyn BufferBase>>,
    /// Vertex array object defining vertex attributes
    vao: VertexArray,
    /// Associated material trait object
    material: Box<dyn Material>,
    /// Number of vertices supplied
    num_vertices: usize,
    /// Whether to draw this mesh as a wireframe
    pub draw_wireframe: bool,
    /// Index buffer, which is only present if the geometry was indexed.
    index_buffer: Option<Box<dyn BufferBase>>,
    /// Size of rendered points. Only used if primitive type is "Points".
    pub point_size: f32
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
            draw_wireframe: false,
            num_vertices: Self::retrieve_vertex_count(&attributes).expect("Geometry attribute buffer sizes inconsistent"),
            index_buffer: None,
            point_size: 1.0
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

    /// Create a new mesh with given primitive type from given indexed geometry
    pub fn new_indexed(pt: PrimitiveType, mat: Box<dyn Material>, geometry: &dyn IndexedGeometry) -> Mesh {
        let attributes = geometry.retrieve_attributes();
        let indices = geometry.retrieve_indices();
        
        let mut mesh = Mesh {
            primitive_type: pt,
            material: mat,
            vao: VertexArray::new(),
            buffers: Vec::new(),
            draw_wireframe: false,
            num_vertices: indices.len(),
            index_buffer: None,
            point_size: 1.0
        };

        // Create buffers and register attributes with vao for each attribute in the geometry
        for attribute in &attributes {
            let buffer = attribute.to_vertex_buffer();

            buffer.enable();
            attribute.setup_attribute(&mesh.vao);
            buffer.disable();

            mesh.buffers.push(buffer);
        }

        // Create index buffer
        let index_buffer = Box::new(Buffer::new_index_buffer(indices));
        mesh.index_buffer = Some(index_buffer);

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
            if self.draw_wireframe {
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            }

            if let PrimitiveType::Points = self.primitive_type {
                gl::PointSize(self.point_size as _);
            }

            if let Some(idxbuf) = &self.index_buffer {
                idxbuf.enable();

                gl::DrawElements(
                    self.primitive_type as _,
                    self.num_vertices as _,
                    gl::UNSIGNED_INT,
                    0 as _
                );

                idxbuf.disable();
            } else {
                gl::DrawArrays(self.primitive_type as _, 0, self.num_vertices as _);
            } 

            if self.draw_wireframe {
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            }
        }

        self.vao.disable_array();
    }
}
