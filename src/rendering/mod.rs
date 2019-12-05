
pub mod shaders;
pub mod types;
pub mod uniforms;
pub mod camera;
pub mod meshes;
pub mod buffers;
pub mod traits;
pub mod materials;

use nalgebra_glm::{Mat4, Vec3};

/// Enumeration describing OpenGL value types
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum ValueType {
    Float = gl::FLOAT
}

/// A class holding important rendering parameters, such as the view and projection matrices.
/// It also allows the implementation of hierachical rendering by offering methods to accumulate
/// model transformations.
#[derive(Clone)]
struct RenderParameters {
    /// A stack of model matrices, used to implement hierachical rendering.
    matrix_stack: Vec<Mat4>,
    /// The scene projection matrix
    pub view: Mat4,
    /// The camera view matrix
    pub projection: Mat4,
    /// The model transformation matrix
    pub model: Mat4
}

impl RenderParameters {
    /// Create a new instance based on given projection and view matrices.
    fn new(view: Mat4, proj: Mat4) -> RenderParameters {
        RenderParameters {
            view: view,
            projection: proj,
            matrix_stack: Vec::new(),
            model: Mat4::identity()
        }
    }

    /// Push the current model matrix on to the matrix stack, saving it for later restoration.
    fn push_matrix(&mut self) {
        self.matrix_stack.push(self.model);
    }

    /// Restore the model matrix by replacing it with the top element of the matrix stack. 
    fn pop_matrix(&mut self) {
        self.model = self.matrix_stack.pop().expect("pop_matrix called on empty matrix stack");
    }

    /// Add given translation to the model matrix.
    fn translate(&mut self, v: &Vec3) {
        let mat = Mat4::new_translation(v);
        self.add_matrix(&mat);
    }

    /// Add given uniform scaling to the model matrix.
    fn scale(&mut self, f: f32) {
        let mat = Mat4::new_scaling(f);
        self.add_matrix(&mat);
    }
    
    /// Add given transformation matrix to the model matrix.
    fn add_matrix(&mut self, mat: &Mat4) {
        self.model *= mat;
    }
}
