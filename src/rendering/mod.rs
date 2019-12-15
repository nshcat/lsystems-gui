
pub mod shaders;
pub mod types;
pub mod uniforms;
pub mod camera;
pub mod meshes;
pub mod buffers;
pub mod traits;
pub mod materials;
pub mod model;

use nalgebra_glm::{Mat4, Vec3};

/// Enumeration describing OpenGL value types
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum ValueType {
    Float = gl::FLOAT,
    UInt = gl::UNSIGNED_INT,
}

/// A class holding important rendering parameters, such as the view and projection matrices.
/// It also allows the implementation of hierachical rendering by offering methods to accumulate
/// model transformations.
#[derive(Clone)]
pub struct RenderParameters {
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
    pub fn new(view: Mat4, proj: Mat4) -> RenderParameters {
        RenderParameters {
            view: view,
            projection: proj,
            matrix_stack: Vec::new(),
            model: Mat4::identity()
        }
    }

    /// Create a new render parameters instance that contains only identity matrices.
    /// For testing purposes only.
    pub fn identity() -> RenderParameters {
        Self::new(Mat4::identity(), Mat4::identity())
    }

    /// Push the current model matrix on to the matrix stack, saving it for later restoration.
    pub fn push_matrix(&mut self) {
        self.matrix_stack.push(self.model);
    }

    /// Restore the model matrix by replacing it with the top element of the matrix stack. 
    pub fn pop_matrix(&mut self) {
        self.model = self.matrix_stack.pop().expect("pop_matrix called on empty matrix stack");
    }

    /// Add given translation to the model matrix.
    pub fn translate(&mut self, v: &Vec3) {
        let mat = Mat4::new_translation(v);
        self.add_matrix(&mat);
    }

    /// Add given uniform scaling to the model matrix.
    pub fn scale(&mut self, f: f32) {
        let mat = Mat4::new_scaling(f);
        self.add_matrix(&mat);
    }
    
    /// Add given transformation matrix to the model matrix.
    pub fn add_matrix(&mut self, mat: &Mat4) {
        self.model *= mat;
    }
}

/// A structure managing the OpenGL viewport
pub struct Viewport {
    x: u32,
    y: u32,
    w: u32,
    h: u32
}

impl Viewport {
    /// Create a new viewport for a window with given dimensions.
    pub fn for_window(w: u32, h: u32) -> Viewport {
        Viewport {
            x: 0,
            y: 0,
            w,
            h
        }
    }

    /// Update the dimensions of the viewport.
    pub fn update(&mut self, w: u32, h: u32) {
        self.w = w;
        self.h = h;
    }

    /// Enable this viewport.
    pub fn enable(&self) {
        unsafe {
            gl::Viewport(self.x as _, self.y as _, self.w as _, self.h as _);
        }
    }
}
