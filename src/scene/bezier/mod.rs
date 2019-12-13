use imgui::*;
use crate::scene::*;
extern crate glfw;


pub struct BezierEditorScene {

}


impl Scene for BezierEditorScene {
    /// Render scene to screen. This also includes any GUI components.
    fn render(&self) {

    }

    /// Show imgui GUI if needed.
    fn do_gui(&mut self, ui: &Ui) -> SceneAction {
        let mut action = SceneAction::Nothing;

        imgui::Window::new(im_str!("Bezier Model Editor"))
            .size([450.0, 550.0], Condition::FirstUseEver)
            .position([0.0, 60.0], Condition::FirstUseEver)
            .build(&ui, || {
                if ui.button(im_str!("back"), [0.0, 0.0]) {
                    action = SceneAction::PopScene;
                }
        });

        action
    }

    /// Handle input event. This is only called if the UI does not want to grab input.
    fn handle_event(&mut self, window: &glfw::Window, event: &glfw::WindowEvent) {
        
    }

    /// Handle window resize event.
    fn handle_resize(&mut self, w: u32, h: u32) {
        
    }
}