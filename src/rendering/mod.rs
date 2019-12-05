
pub mod shaders;
pub mod types;
pub mod uniforms;

use nalgebra_glm::{Mat4, Vec3};

/// A class holding important rendering parameters, such as the view and projection matrices.
/// It also allows the implementation of hierachical rendering by offering methods to accumulate
/// model transformations.
#[derive(Clone)]
struct RenderParameters {
    /// A stack of model matrices, used to implement hierachical rendering.
    matrix_stack: Vec<Mat4>,
    /// The scene projection matrix
    view: Mat4,
    /// The camera view matrix
    projection: Mat4,
    /// The model transformation matrix
    model: Mat4
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


/// A set of vectors describing the current camera state. This does not contain any
/// matrices, since both the projection and view matrix are derived from this.
/// External navigation algorithms can supply this data and thus cause the camera view
/// to change.
#[derive(Clone, Copy, Debug)]
struct CameraState {
    /// The cameras position in 3D space
    pub position: Vec3,

    /// A vector describing the direction up from the view of the camera
    pub up: Vec3,

    /// Target the camera points at
    pub target: Vec3
}

impl CameraState {
    /// Create a new camera state instance with given values.
    fn new(pos: Vec3, up: Vec3, target: Vec3) -> CameraState {
        CameraState {
            position: pos,
            up: up,
            target: target
        }
    }
}

/// A class describing a camera that observes the 3D scene.
#[derive(Clone)]
struct Camera {
    state: CameraState,
    projection: Mat4,
    view: Mat4
}

impl Camera {
    /// Extract relevant camera information to use in a rendering operation
    fn to_render_parameters(&self) -> RenderParameters {
        RenderParameters::new(self.view, self.projection)
    }
}

