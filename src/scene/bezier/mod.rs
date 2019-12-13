use std::fmt::*;
use imgui::*;
use nalgebra_glm::Vec3;
use crate::scene::*;
use crate::data::patches::*;
extern crate glfw;


pub struct BezierEditorScene {
    /// Reference to the model to edit. This will only be modifed once the user
    /// hit "save".
    model: RcCell<BezierModelParameters>,
    /// The copy of the model parameters the GUI operates on.
    working_copy: BezierModelParameters
}

impl BezierEditorScene {
    pub fn new(model: RcCell<BezierModelParameters>) -> BezierEditorScene {
        let working_copy = model.borrow().clone();
        BezierEditorScene {
            working_copy: working_copy,
            model: model
        }
    }
}

impl Scene for BezierEditorScene {
    /// Render scene to screen. This also includes any GUI components.
    fn render(&self) {

    }

    fn do_logic(&mut self) {

    }

    /// Show imgui GUI if needed.
    fn do_gui(&mut self, ui: &Ui) -> SceneAction {
        let mut action = SceneAction::Nothing;

        imgui::Window::new(im_str!("Bezier Model Editor"))
            .size([250.0, 500.0], Condition::FirstUseEver)
            .position([0.0, 60.0], Condition::FirstUseEver)
            .build(&ui, || {
                if ui.collapsing_header(im_str!("Models"))
                    .default_open(true)
                    .build() {
                    ui.indent();
 
                    for (i, patch) in self.working_copy.patches.iter_mut().enumerate() {
                        let patch_id = ui.push_id(i as i32);

                        let mut label = ImString::with_capacity(128);
                        label.push_str(&format!("Control Points for Model '{}'", i));

                        if ui.collapsing_header(&label)
                            .default_open(false)
                            .build() {
                            ui.indent();
                            
                            for (j, curve) in patch.curves.iter_mut().enumerate() {
                                let token = ui.push_id(j as i32);

                                let mut label = ImString::with_capacity(48);
                                label.push_str(&format!("Curve {}", j));

                                ui.text(&label);

                                ui.indent();

                                for k in 0..4 {
                                    let point = &mut curve.control_points[k];

                                    let mut data = [point.x, point.y, point.z];

                                    let mut label = ImString::with_capacity(48);
                                    label.push_str(&format!("Point {}", k));

                                    if ui.drag_float3(&label, &mut data) 
                                        .min(-500.0)
                                        .max(500.0)
                                        .display_format(im_str!("%.2lf"))
                                        .speed(0.06)
                                        .build() {
                                            *point = Vec3::new(data[0], data[1], data[2]);
                                    }
                                }

                                ui.unindent();

                                token.pop(ui);
                            }

                            ui.unindent();
                        }

                        patch_id.pop(ui);
                    }

                    let colors = ui.push_style_colors(&[
                        (StyleColor::Button, [0.349, 0.6, 0.239, 1.0]),
                        (StyleColor::ButtonHovered, [0.3568, 0.7019, 0.2117, 1.0]),
                        (StyleColor::ButtonActive, [0.3529, 0.8, 0.1607, 1.0])
                    ]);
                
                    if ui.button(im_str!("+"), [0.0, 0.0]) {
                        self.working_copy.patches.push(BezierPatchParameters::empty());
                    }
                
                    colors.pop(ui);

                    ui.unindent();
                }

                if ui.collapsing_header(im_str!("Settings"))
                    .default_open(false)
                    .build() {
                    ui.indent();          

                    ui.unindent();
                }

                if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
                    action = SceneAction::PopScene;
                }

                ui.same_line(0.0);

                if ui.button(im_str!("Save"), [0.0, 0.0]) {
                    *self.model.borrow_mut() = self.working_copy.clone();
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