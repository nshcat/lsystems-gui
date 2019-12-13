
use std::rc::Rc;
use std::cell::RefCell;
use imgui::Ui;
use glfw::{Window, WindowEvent};

/// Module containg scene that allows rendering and display of a L-System
pub mod lsystem;
/// Module containing interactive bezier patch editor
pub mod bezier;

/// Shortcut type for a ref cell inside a reference counted pointer
pub type RcCell<T> = Rc<RefCell<T>>;

/// Create a new reference counted ref cell by moving given value into it
pub fn make_rc_cell<T>(value: T) -> RcCell<T> {
    Rc::new(RefCell::new(value))
}

/// A trait describing a single scene in the application.
/// A scene consumes input events and might display a GUI.
/// If a scene wants to display 3D content, it is expected to provide its own
/// camera, and thus, its own rendering parameters.
pub trait Scene {
    /// Render scene to screen. This also includes any GUI components.
    fn render(&self);

    /// Show imgui GUI if needed.
    fn do_gui(&mut self, ui: &Ui) -> SceneAction;

    /// Handle input event. This is only called if the UI does not want to grab input.
    fn handle_event(&mut self, window: &Window, event: &WindowEvent);

    /// Handle window resize event.
    fn handle_resize(&mut self, w: u32, h: u32);
}

/// A struct that manages a stack of scenes.
pub struct SceneManager {
    /// Scene stack
    scenes: Vec<RcCell<dyn Scene>>
}

impl SceneManager {
    /// Create new scene manager instance
    pub fn new() -> SceneManager {
        SceneManager {
            scenes: Vec::new()
        }
    }

    /// Push given scene on top of the scene stack. It will be the scene rendered from this point on,
    /// until it either gets popped of the stack, or some other scene gets pushed.
    pub fn push_scene(&mut self, scene: RcCell<dyn Scene>) {
        self.scenes.push(scene);
    }

    /// Remove the topmost scene from the scene stack.
    pub fn pop_scene(&mut self) {
        self.scenes.pop().expect("pop called on empty scene stack");
    }

    /// Retrieve the current scene
    pub fn current_scene(&self) -> &RcCell<dyn Scene> {
        self.scenes.last().expect("current_scene called on empty scene stack")
    }

    /// Process given scene action command object.
    pub fn process_action(&mut self, action: SceneAction) {
        match action {
            SceneAction::PushScene(scene) => self.push_scene(scene),
            SceneAction::PopScene => self.pop_scene(),
            _ => {}
        }
    }
}

/// An enumeration implementing the Command pattern in order to tell the scene manager
/// what action to take after a scene has been rendered. It can be used to push/pop scenes
/// from within the current scene, while avoiding borrowing issues.
pub enum SceneAction {
    /// Do nothing
    Nothing,
    /// Pop the current scene from the scene stack
    PopScene,
    /// Push given new scene to the scene stack. It will become the new
    /// current scene.
    PushScene(RcCell<dyn Scene>)
}