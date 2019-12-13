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
    meshes: Vec<Mesh>,
    /// Control point visualisation
    control_point_meshes: Vec<Mesh>,
    /// Control curve visualisation
    control_curve_meshes: Vec<Mesh>,
    /// Point clouds approximating the patch (for debugging purposes)
    patch_approximations: Vec<Mesh>
}

impl BezierEditorScene {
    pub fn new(model: RcCell<BezierModelParameters>, w: u32, h: u32) -> BezierEditorScene {
        let working_copy = model.borrow().clone();
        let mut scene = BezierEditorScene {
            working_copy: working_copy,
            model: model,
            camera: Camera::new(w, h, ProjectionType::Perspective(75.0)),
            meshes: Vec::new(),
            control_point_meshes: Vec::new(),
            control_curve_meshes: Vec::new(),
            patch_approximations: Vec::new()
        };

        scene.refresh_meshes();

        scene
    }
}

impl BezierEditorScene {
    /// Just refresh the mesh for the patch with given index
    fn refresh_mesh_for(&mut self, index: usize) {
        let patch = &self.working_copy.patches[index];

        let mesh = self.create_mesh(patch);
        self.meshes[index] = mesh;

        let control_point_mesh = self.create_control_point_mesh(patch);
        self.control_point_meshes[index] = control_point_mesh;

        let control_curve_mesh = self.create_control_curve_mesh(patch);
        self.control_curve_meshes[index] = control_curve_mesh;

        let point_cloud_mesh = self.create_point_cloud(patch);
        self.patch_approximations[index] = point_cloud_mesh;
    }

    /// Refresh all patch meshes
    fn refresh_meshes(&mut self) {
        self.meshes = Vec::new();
        self.control_point_meshes = Vec::new();
        self.control_curve_meshes = Vec::new();
        self.patch_approximations = Vec::new();

        for patch in &self.working_copy.patches {
            self.meshes.push(self.create_mesh(patch));
            self.control_point_meshes.push(self.create_control_point_mesh(patch));
            self.control_curve_meshes.push(self.create_control_curve_mesh(patch));
            self.patch_approximations.push(self.create_point_cloud(patch));
        }
    }

    fn create_mesh(&self, patch: &BezierPatchParameters) -> Mesh {
        let plane = PlaneGeometry::with_displacement(
            30, 30, Vec3::new(0.4, 0.4, 0.4),
            &|u, v| patch.evaluate(u, v)
        );

        let mat = Box::new(SimpleMaterial::new());

        let mut mesh = Mesh::new_indexed(PrimitiveType::TriangleStrip, mat, &plane);
        mesh.draw_wireframe = true;
        mesh
    }

    fn create_control_point_mesh(&self, patch: &BezierPatchParameters) -> Mesh {
        let mut points = Vec::new();

        for curve in &patch.curves {
            for i in 0..4 {
                points.push(curve.control_points[i].clone());
            }
        }

        let mut geom = BasicGeometry::new();
        geom.colors.local_buffer = vec![Vec3::new(1.0, 1.0, 1.0); points.len()];
        geom.normals.local_buffer = vec![Vec3::new(0.0, 0.0, 0.0); points.len()];
        geom.positions.local_buffer = points;

        let mat = Box::new(SimpleMaterial::new());

        let mut mesh = Mesh::new(PrimitiveType::Points, mat, &geom);
        mesh.point_size = 3.0;

        mesh
    }

    fn create_control_curve_mesh(&self, patch: &BezierPatchParameters) -> Mesh {
        let mut points = Vec::new();

        for curve in &patch.curves {
            for i in 1..4 {
                points.push(curve.control_points[i-1].clone());
                points.push(curve.control_points[i].clone());
            }
        }

        let mut geom = BasicGeometry::new();
        geom.colors.local_buffer = vec![Vec3::new(1.0, 1.0, 0.0); points.len()];
        geom.normals.local_buffer = vec![Vec3::new(0.0, 0.0, 0.0); points.len()];
        geom.positions.local_buffer = points;

        let mat = Box::new(SimpleMaterial::new());

        let mut mesh = Mesh::new(PrimitiveType::Lines, mat, &geom);

        mesh
    }


    fn create_point_cloud(&self, patch: &BezierPatchParameters) -> Mesh {
        let points_x = 60;
        let points_y = 60;

        let mut points = Vec::new();

        for y in 0..60 {
            for x in 0..60 {
                points.push(patch.evaluate((x as f32) / (points_x as f32), (y as f32) / (points_y as f32)));
            }
        }

        let mut geom = BasicGeometry::new();
        geom.colors.local_buffer = vec![Vec3::new(0.0, 0.0, 1.0); points.len()];
        geom.normals.local_buffer = vec![Vec3::new(0.0, 0.0, 0.0); points.len()];
        geom.positions.local_buffer = points;

        let mat = Box::new(SimpleMaterial::new());

        let mut mesh = Mesh::new(PrimitiveType::Points, mat, &geom);
        mesh.point_size = 2.0;

        mesh
    }
}

impl Scene for BezierEditorScene {
    /// Render scene to screen. This also includes any GUI components.
    fn render(&self) {
        let mut rp = self.camera.to_render_parameters();

        for mesh in &self.meshes {
            mesh.render(&mut rp);
        }

        for mesh in &self.control_point_meshes {
            mesh.render(&mut rp);
        }

        for mesh in &self.control_curve_meshes {
            mesh.render(&mut rp);
        }

        for mesh in &self.patch_approximations {
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

                    let mut modified: Option<usize> = None;
                    let mut refresh_all = false;
 
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
                                            modified = Some(i);
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
                        refresh_all = true;
                    }
                
                    colors.pop(ui);

                    ui.unindent();

                    if refresh_all {
                        self.refresh_meshes();
                    } else if let Some(i) = modified {
                        self.refresh_mesh_for(i);
                    }
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
                }

                ui.same_line(0.0);

                if ui.button(im_str!("Save and Exit"), [0.0, 0.0]) {
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