use std::fmt::*;
use imgui::*;
use nalgebra_glm::*;
use ncollide3d::*;
use ncollide3d::math::*;
use ncollide3d::query::*;
use gl::*;
extern crate nalgebra;
use crate::scene::*;
use crate::data::patches::*;
use crate::rendering::*;
use crate::rendering::camera::*;
use crate::rendering::meshes::*;
use crate::rendering::materials::*;
use crate::rendering::traits::*;
use crate::rendering::model::*;
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
    control_point_models: Vec<MultiModel>,
    /// Control curve visualisation
    control_curve_meshes: Vec<Mesh>,
    /// Point clouds approximating the patch (for debugging purposes)
    patch_approximations: Vec<Mesh>,
    /// Whether to draw the point cloud approximation
    draw_approximation: bool,
    /// Whether to draw the control curves
    draw_control_curves: bool,
    /// Screen width
    width: u32,
    /// Screen height
    height: u32,
    /// The sphere mesh used to visualize the control points. Its shared with all control point models.
    sphere_mesh: Rc<Box<Mesh>>,
    /// Where the mouse drag started
    drag_begin: Option<(u32, u32)>,
    /// Depth of the point we are dragging
    drag_depth: Option<f32>,
    /// The indices of the patch, curve and point that is currently being dragged.
    dragged_point: Option<(usize, usize, usize)>,
    /// Whether we are currently dragging
    in_drag: bool
}

impl BezierEditorScene {
    pub fn new(model: RcCell<BezierModelParameters>, w: u32, h: u32) -> BezierEditorScene {
        let mat = Box::new(SimpleMaterial::new());
        let sphere_geom = SphereGeometry::new(0.01, 40, 40, Vec3::new(1.0, 1.0, 1.0));

        let mut mesh = Mesh::new_indexed(PrimitiveType::TriangleStrip, mat, &sphere_geom);
        mesh.draw_wireframe = false;

        let working_copy = model.borrow().clone();
        let mut scene = BezierEditorScene {
            working_copy: working_copy,
            model: model,
            camera: Camera::new(w, h, ProjectionType::Perspective(75.0)),
            meshes: Vec::new(),
            control_point_models: Vec::new(),
            control_curve_meshes: Vec::new(),
            patch_approximations: Vec::new(),
            draw_approximation: true,
            draw_control_curves: true,
            width: w,
            height: h,
            sphere_mesh: Rc::new(Box::new(mesh)),
            in_drag: false,
            drag_depth: None,
            drag_begin: None,
            dragged_point: None
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

        let control_point_model = self.create_control_point_model(patch);
        self.control_point_models[index] = control_point_model;

        let control_curve_mesh = self.create_control_curve_mesh(patch);
        self.control_curve_meshes[index] = control_curve_mesh;

        let point_cloud_mesh = self.create_point_cloud(patch);
        self.patch_approximations[index] = point_cloud_mesh;
    }

    /// Refresh all patch meshes
    fn refresh_meshes(&mut self) {
        self.meshes = Vec::new();
        self.patch_approximations = Vec::new();

        for patch in &self.working_copy.patches {
            self.meshes.push(self.create_mesh(patch));
            self.patch_approximations.push(self.create_point_cloud(patch));
        }

        self.refresh_control_meshes();
    }

    fn refresh_control_meshes(&mut self) {
        self.control_point_models = Vec::new();
        self.control_curve_meshes = Vec::new();

        for patch in &self.working_copy.patches {
            self.control_point_models.push(self.create_control_point_model(patch));
            self.control_curve_meshes.push(self.create_control_curve_mesh(patch));
        }
    }

    fn create_mesh(&self, patch: &BezierPatchParameters) -> Mesh {
        let plane = PlaneGeometry::with_displacement(
            30, 30, Vec3::new(0.4, 0.4, 0.4),
            &|u, v| patch.evaluate(u, v)
        );

        let mat = Box::new(SimpleMaterial::new());

        let mut mesh = Mesh::new_indexed(PrimitiveType::TriangleStrip, mat, &plane);
        mesh
    }

    fn create_control_point_model(& self, patch: &BezierPatchParameters) -> MultiModel { 
        let mut spheres = Vec::new();
        
        for curve in &patch.curves {
            for i in 0..4 {
                let point = curve.control_points[i].clone();

                spheres.push(
                    Model::with_position(self.sphere_mesh.clone(), &point)
                );
            }
        }

        MultiModel {
            models: spheres
        }
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

    /// Unproject a given window position to a point in world space
    fn unproject(&self, x: u32, y: u32, depth: f32) -> Vec3 {
        unproject(
            &Vec3::new(x as _, (self.height - y) as _, depth),
            &self.camera.view,
            &self.camera.projection,
            Vec4::new(0.0, 0.0, self.width as _, self.height as _)
        )
    }

    /// Returns clicked control point and its depth
    fn find_clicked_control_point(&mut self, x: u32, y: u32) -> Option<(f32, usize, usize, usize)> {
        // Retrieve depth value
        let mut depth: f32 = 0.0;
        unsafe {
            gl::ReadPixels(
                x as _,
                (self.height - y) as _,
                1 as _,
                1 as _,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                &mut depth as *mut f32 as _
            );
        }

        let position = self.unproject(x, y, depth);

        // We now have the rough position. We now have to intersect a ball around it with the balls around all
        // the other control points to find the closest point to it.

        // The sphere used for all points
        let sphere = shape::Ball::<f32>::new(0.01);
        
        // Create translation for ball around point
        let position_isometry = Isometry::new(position.clone(), nalgebra::zero());    

        // Collect intersection results
        let mut control_point: Option<&mut Vec3> = None;

        for (i, patch) in self.working_copy.patches.iter().enumerate() {
            for (j, curve) in patch.curves.iter().enumerate() {
                for k in 0..4 {
                    let point = &curve.control_points[k];

                    let translation = Isometry::new(point.clone(), nalgebra::zero());

                    let result = proximity(
                        &position_isometry, &sphere,
                        &translation, &sphere, 0.01);

                    match result {
                        Proximity::Intersecting => {
                            return Some((depth, i, j, k));
                        },
                        _ => {}
                    };
                }
            }
        }

        return None;
    }
}

impl Scene for BezierEditorScene {
    /// Render scene to screen. This also includes any GUI components.
    fn render(&self) {
        let mut rp = self.camera.to_render_parameters();

        for mesh in &self.meshes {
            mesh.render(&mut rp);
        }

        if self.draw_control_curves {
            for mesh in &self.control_curve_meshes {
                mesh.render(&mut rp);
            }
        }

        if self.draw_approximation {
            for mesh in &self.patch_approximations {
                mesh.render(&mut rp);
            }
        }

        for model in &self.control_point_models {
            model.render(&mut rp);
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

                    ui.checkbox(im_str!("Draw control curves"), &mut self.draw_control_curves);
                    ui.checkbox(im_str!("Draw point cloud approximation"), &mut self.draw_approximation);

                    ui.unindent();
                }

                ui.spacing();

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
        // MouseButton(MouseButton, Action, Modifiers)
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, glfw::Action::Press, _) => {
                let (x, y) = window.get_cursor_pos();
                if let Some((d, i, j, k)) = self.find_clicked_control_point(x as _, y as _) {
                    self.drag_begin = Some((x as _, y as _));
                    self.drag_depth = Some(d);
                    self.in_drag = true;
                    self.dragged_point = Some((i, j, k));
                }
            },
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, glfw::Action::Release, _) => {
                if self.in_drag {
                    self.in_drag = false;
                    self.refresh_meshes();
                }
            },
            glfw::WindowEvent::CursorPos(x, y) => {
                if self.in_drag {
                    // If the user drags the cursor outside of the window, stop dragging process.
                    if *x >= 0.0 && *x <= (self.width as f64) && *y >= 0.0 && *y <= (self.height as f64) {
                        // If we are in drag, we project the new mouse screen position into the scene with the same
                        // depth as the control point at the old position, and use that new 3D position
                        // as our new control position.
                        let curX = *x as u32;
                        let curY = *y as u32;

                        let (oldX, oldY) = self.drag_begin.unwrap();
                        let new_point = self.unproject(curX, curY, self.drag_depth.unwrap());
                        
                        let (i, j, k) = self.dragged_point.unwrap();
                        let p = &mut self.working_copy.patches[i].curves[j].control_points[k];
                        *p = new_point.clone();
            
                        self.drag_begin = Some((curX, curY));            
                        self.refresh_control_meshes();  
                    } else {
                        self.in_drag = false;
                        self.refresh_meshes();
                    }
                }
            },
            _ => {}
        };

        if !self.in_drag {
            self.camera.handle_event(window, event);
        }
    }

    /// Handle window resize event.
    fn handle_resize(&mut self, w: u32, h: u32) {
        self.camera.update(w, h);

        self.width = w;
        self.height = h;
    }
}