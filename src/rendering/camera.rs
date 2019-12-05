use nalgebra_glm::{Mat4, Vec3};
use crate::rendering::RenderParameters;

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

