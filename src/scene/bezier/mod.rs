use std::fmt::*;
use imgui::*;
use nalgebra_glm::Vec3;
use crate::scene::*;
use crate::data::patches::*;
use crate::rendering::*;
use crate::rendering::camera::*;
use crate::rendering::meshes::*;
use crate::rendering::materials::*;
use crate::rendering::traits::*;
extern crate glfw;


pub struct BezierEditorScene {
    /// Reference to the model to edit. This will only be modifed once the user
    /// hit "save".
    model: RcCell<BezierModelParameters>,
    /// The copy of the model parameters the GUI operates on.
    working_copy: BezierModelParameters,
    /// Camera used to view the patches
    camera: Camera,
    /// All meshes to render.
    meshes: Vec<Mesh>
}

impl BezierEditorScene {
    pub fn new(model: RcCell<BezierModelParameters>, w: u32, h: u32) -> BezierEditorScene {
        let working_copy = model.borrow().clone();
        BezierEditorScene {
            working_copy: working_copy,
            model: model,
            camera: Camera::new(w, h, ProjectionType::Perspective(75.0)),
            meshes: Vec::new()
        }
    }
}

impl BezierEditorScene {
    /// Just refresh the mesh for the patch with given index
    fn refresh_mesh_for(&mut self, index: usize) {

    }

    /// Refresh all patch meshes
    fn refresh_meshes(&mut self) {

    }
}

impl Scene for BezierEditorScene {
    /// Render scene to screen. This also includes any GUI components.
    fn render(&self) {
        let mut rp = self.camera.to_render_parameters();

        for mesh in &self.meshes {
            mesh.render(&mut rp);
        }
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
        self.camera.handle_event(window, event);
    }

    /// Handle window resize event.
    fn handle_resize(&mut self, w: u32, h: u32) {
        self.camera.update(w, h);
    }
}