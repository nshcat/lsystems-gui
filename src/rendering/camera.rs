use nalgebra_glm::{Mat4, IVec2, Vec3, perspective_fov, ortho, look_at, two_pi, pi};
use glfw::{Window, WindowEvent, MouseButton, Action};
use crate::rendering::RenderParameters;

/// An enumeration describing how the camera projects the scene to the screen.
#[derive(Clone, Copy)]
pub enum ProjectionType {
    /// Use orthographic projection
    Orthographic,
    /// Use perspective projection with given FOV value (in degrees)
    Perspective(f32)
}


/// A set of vectors describing the current camera state. This does not contain any
/// matrices, since both the projection and view matrix are derived from this.
/// External navigation algorithms can supply this data and thus cause the camera view
/// to change.
#[derive(Clone, Copy, Debug)]
pub struct CameraState {
    /// The cameras position in 3D space
    pub position: Vec3,
    /// A vector describing the direction up from the view of the camera
    pub up: Vec3,
    /// Target the camera points at
    pub target: Vec3
}

impl CameraState {
    /// Create a new camera state instance with given values.
    pub fn new(pos: Vec3, up: Vec3, target: Vec3) -> CameraState {
        CameraState {
            position: pos,
            up: up,
            target: target
        }
    }
}

/// Enumeration describing in what type of move operation the dragged camere is at the moment
#[derive(Clone)]
enum MoveMode {
    None,
    Rotate,
    Pan
}

/// A class describing a camera that observes the 3D scene and simulates a trackball.
/// https://computergraphics.stackexchange.com/questions/151/how-to-implement-a-trackball-in-opengl
#[derive(Clone)]
pub struct Camera {
    /// Set of vectors describing the state of the camera
    state: CameraState,
    /// Projection type to use
    proj_type: ProjectionType,
    /// Screen width
    width: u32,
    /// Screen height
    height: u32,
    /// Current projection matrix
    projection: Mat4,
    /// Current view matrix
    view: Mat4,
    /// Current drag start position
    drag_start: IVec2,
    /// Whether the user is currently dragging the mouse
    is_dragging: bool,
    /// Pixel coordinates of the screen center
    center: IVec2,
    /// First rotation angle
    theta: f64,
    /// Second rotation angle
    phi: f64,
    /// The trackball camera radius. Can be modified by zooming.
    radius: f64,
    /// The current move mode
    move_mode: MoveMode
}

impl Camera {
    /// Create new camera instance
    pub fn new(width: u32, height: u32, proj_type: ProjectionType, state: CameraState) -> Camera {
        let mut cam = Camera {
            width,
            height,
            proj_type,
            state,
            projection: Mat4::identity(),
            view: Mat4::identity(),
            drag_start: IVec2::zeros(),
            is_dragging: false,
            center: IVec2::new((width / 2) as _, (height / 2) as _),
            theta: 0.0,
            phi: pi::<f64>() / 2.0,
            radius: 1.0,
            move_mode: MoveMode::None
        };

        cam.update_state();
        cam.update_proj();
        cam.update_view();

        cam
    }


    /// Extract relevant camera information to use in a rendering operation
    pub fn to_render_parameters(&self) -> RenderParameters {
        RenderParameters::new(self.view, self.projection)
    }

    /// An internal method used to update the view matrix after camera state
    /// has changed
    fn update_view(&mut self) {
        self.view = look_at(
            &self.state.position,
            &self.state.target,
            &self.state.up.normalize()
        );
    }

    /// Update projection matrix. This is required when either the viewport or the projection
    /// type changes.
    fn update_proj(&mut self) {
        match self.proj_type {
            ProjectionType::Orthographic => {
                let aspect = self.width as f32 / self.height as f32;

                self.projection = ortho(
                    -aspect/2.0,
                    aspect/2.0,
                    0.5,
                    -0.5, 
                    0.1,
                    1000.0
                );
            },
            ProjectionType::Perspective(fov) => {
                self.projection = perspective_fov(
                    fov.to_radians(),   // The field of view, in radians
                    self.width as _,    // Width of the screen
                    self.height as _,   // Height of the screen
                    0.1,                // Near clip plane  
                    1000.0              // Far clip plane
                );
            }
        }
    }

    /// Signal beginning of mouse drag
    fn drag_start(&mut self, pos: &IVec2) {
        self.is_dragging = true;
        self.drag_update(pos);
    }

    /// Update drag position
    fn drag_update(&mut self, pos: &IVec2) {
        if self.is_dragging {
            self.drag_start = *pos/* - self.center*/;
        }
    }

    /// Convert current rotation angles to caresian coordinated of camera location,
    /// relativ to the currently looked at location.
    fn to_cartesian(&self) -> Vec3 {
        Vec3::new(
            (self.radius * self.phi.sin() * self.theta.sin()) as _,
            (self.radius * self.phi.cos()) as _,
            (self.radius * self.phi.sin() * self.theta.cos()) as _
        )
    }

    /// Update the stored camera state based on the rotation angles
    fn update_state(&mut self) {
        self.state.position = self.camera_position();
    }

    /// Retrieve actual camera position in world space
    fn camera_position(&self) -> Vec3 {
        self.to_cartesian() + self.state.target
    }

    /// Signal end of mouse drag
    fn drag_end(&mut self) {
        if self.is_dragging {
            self.drag_update(&IVec2::zeros());
            self.is_dragging = false;
        }
    }

    /// Check if camera is currently being dragged by the user
    pub fn dragging(&self) -> bool {
        self.is_dragging
    }

    /// Notify camera of updated screen dimensions
    pub fn update(&mut self, w: u32, h: u32) {
        self.width = w;
        self.height = h;
        self.center = IVec2::new((w / 2) as _, (h / 2) as _);
        self.update_proj();
    }

    /// Pan camera
    fn pan(&mut self, pos: &IVec2) {
        let dif =  self.drag_start - pos;

        let look = (self.state.target - self.camera_position()).normalize();

        let right = look.cross(&self.state.up);
        let up = look.cross(&right);

        self.state.target += (right * (dif.x as f32 * 0.0018)) + (up * (dif.y as f32 * 0.0018));

        self.update_state();
        self.update_view();
    }

    /// Rotate camera
    fn rotate(&mut self, pos: &IVec2) {
        let dif =  self.drag_start - pos;

        // Calculate delta angles
        let delta_theta = dif.x as f64 / 300.0;
        let delta_phi = dif.y as f64 / 300.0;

        if self.state.up == Vec3::new(0.0, 1.0, 0.0) {
            self.theta += delta_theta;
        } else {
            self.theta -= delta_theta;
        }

        self.phi += delta_phi;

        // Keep angles in interval -2PI to +2PI
        if self.phi > two_pi() {
            self.phi -= two_pi::<f64>();
        } else if self.phi < -two_pi::<f64>() {
            self.phi += two_pi::<f64>();
        }

        if (self.phi > 0.0 && self.phi < pi::<f64>()) || (self.phi < -pi::<f64>() && self.phi > -two_pi::<f64>()) {
            self.state.up = Vec3::new(0.0, 1.0, 0.0);
        } else {
            self.state.up = Vec3::new(0.0, -1.0, 0.0);
        }

        self.update_state();
        self.update_view();
    }

    /// Helper function thats extracts integral mouse position from window
    fn retrieve_mouse_pos(window: &Window) -> IVec2 {
        let (x, y) = window.get_cursor_pos();
        IVec2::new(x as _, y as _)
    }

    /// Handle input event in order to implement trackball controls
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) {
        match event {
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                let pos = Self::retrieve_mouse_pos(window);
                self.drag_start(&pos);
                self.move_mode = MoveMode::Rotate;
            },
            WindowEvent::MouseButton(MouseButton::Button2, Action::Press, _) => {
                let pos = Self::retrieve_mouse_pos(window);
                self.drag_start(&pos);
                self.move_mode = MoveMode::Pan;
            },
            WindowEvent::MouseButton(_, Action::Release, _) => {
                self.drag_end();
            },
            WindowEvent::CursorPos(x, y) => {
                let pos = IVec2::new(*x as _, *y as _);

                if self.is_dragging {
                    match self.move_mode {
                        MoveMode::Rotate => self.rotate(&pos),
                        MoveMode::Pan => self.pan(&pos),
                        _ => {}
                    }
                    self.drag_update(&pos);
                }
            },
            _ => {},
        }
    }
}

